//! Adding files: "Add files" in Knowledge, "Attach a brief or notes" in the intake, and drag and
//! drop anywhere. Files are read and sorted on a background thread (the AI sorts when one is
//! connected, otherwise the file name and headings decide, and Resolve says it was a guess).

use super::theme::*;
use super::widgets::*;
use super::*;
use crate::resolve::files;
use eframe::egui::{self, RichText};
use std::path::PathBuf;
use std::sync::mpsc::Receiver;

/// Where a batch of files is going.
#[derive(Clone)]
pub enum ImportTarget {
    /// Knowledge, for this project (None = no project picked).
    Knowledge(Option<Id>),
    /// An intake's attachments.
    Intake(Id),
}

pub struct ImportOut {
    pub title: String,
    pub text: String,
    pub place: String,
    pub how: String,
    pub why: String,
    pub error: Option<String>,
}

#[derive(Default)]
pub struct ImportUi {
    pub job: Option<(ImportTarget, Receiver<Vec<ImportOut>>)>,
    pub started: f64,
    /// What the last batch did: (page id, message).
    pub log: Vec<(Option<Id>, String)>,
}

pub fn pick_files() -> Vec<PathBuf> {
    rfd::FileDialog::new().add_filter("Briefs and notes", &files::ACCEPTED).pick_files().unwrap_or_default()
}

impl App {
    pub(super) fn import_files(&mut self, target: ImportTarget, paths: Vec<PathBuf>) {
        if paths.is_empty() {
            return;
        }
        if self.import.job.is_some() {
            self.toast("Still adding the last files. Try again in a moment.");
            return;
        }
        let cfg = crate::ai::current_config();
        let use_ai = cfg.provider != "offline" && self.shot.is_none();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let mut out = vec![];
            for p in paths {
                match files::extract(&p) {
                    Ok((title, text)) => {
                        let (place, why, how) = if use_ai {
                            match files::sort_with_ai(&cfg, &title, &text) {
                                Ok((pl, why)) => (pl, why, "ai".to_string()),
                                Err(_) => {
                                    let (pl, why) = files::guess(&title, &text);
                                    (pl, format!("{why} (the AI couldn't sort it)"), "guess".to_string())
                                }
                            }
                        } else {
                            let (pl, why) = files::guess(&title, &text);
                            (pl, why, "guess".to_string())
                        };
                        out.push(ImportOut { title, text, place, how, why, error: None });
                    }
                    Err(e) => out.push(ImportOut { title: p.file_name().map(|x| x.to_string_lossy().to_string()).unwrap_or_default(), text: String::new(), place: String::new(), how: String::new(), why: String::new(), error: Some(e) }),
                }
            }
            let _ = tx.send(out);
        });
        self.import.job = Some((target, rx));
        self.import.started = self.now;
    }

    /// Apply finished imports (every frame).
    pub(super) fn poll_import(&mut self, ctx: &egui::Context) {
        // drag and drop: into the intake when it's asking for material, otherwise into Knowledge
        let dropped: Vec<PathBuf> = ctx.input(|i| i.raw.dropped_files.iter().filter_map(|f| f.path.clone()).collect());
        if !dropped.is_empty() {
            let target = match &self.view {
                View::Intake(id) => ImportTarget::Intake(id.clone()),
                _ => ImportTarget::Knowledge(self.current_project.clone()),
            };
            self.import_files(target, dropped);
        }
        let done = match &self.import.job {
            Some((t, rx)) => rx.try_recv().ok().map(|r| (t.clone(), r)),
            None => None,
        };
        let Some((target, results)) = done else {
            if self.import.job.is_some() {
                ctx.request_repaint_after(std::time::Duration::from_millis(250));
            }
            return;
        };
        self.import.job = None;
        self.import.log.clear();
        match target {
            ImportTarget::Knowledge(pid) => {
                let mut placed: Vec<(Id, String)> = vec![];
                let label = format!("Added {} file(s) to Knowledge", results.iter().filter(|r| r.error.is_none()).count());
                self.undoable(&label, "import", |d| {
                    for r in results.iter().filter(|r| r.error.is_none()) {
                        let id = files::place(d, pid.as_deref(), &r.title, &r.text, &r.place, &r.how);
                        placed.push((id, String::new()));
                    }
                });
                let mut i = 0;
                for r in &results {
                    match &r.error {
                        Some(e) => {
                            self.import.log.push((None, format!("⚠ {}: {e}", r.title)));
                            crate::ai::set_last_error(e);
                        }
                        None => {
                            let msg = format!("Put \"{}\" in {} ({})", r.title, files::label(&r.place), if r.how == "ai" { format!("sorted by the AI: {}", r.why) } else { r.why.clone() });
                            self.import.log.push((placed.get(i).map(|x| x.0.clone()), msg));
                            i += 1;
                        }
                    }
                }
                if let Some((_, m)) = self.import.log.first() {
                    let m = m.clone();
                    self.toast(if self.import.log.len() > 1 { format!("{m} (and {} more, see Knowledge)", self.import.log.len() - 1) } else { m });
                }
                self.resolve_ui.knowledge_tab = 0;
                self.go(View::Sec(nav::Section::Knowledge));
            }
            ImportTarget::Intake(iid) => {
                let mut notes = vec![];
                if let Some(it) = self.store.data.intakes.iter_mut().find(|x| x.id == iid) {
                    for r in &results {
                        match &r.error {
                            Some(e) => notes.push(format!("⚠ {}: {e}", r.title)),
                            None => {
                                it.attachments.retain(|a| a.name != r.title);
                                it.attachments.push(crate::resolve::intake::Attachment { name: r.title.clone(), text: r.text.clone(), place: r.place.clone(), how: r.how.clone(), why: r.why.clone() });
                            }
                        }
                    }
                    it.touch();
                }
                self.touch();
                for n in notes {
                    self.toast(n);
                }
            }
        }
    }

    /// "Add files" (Knowledge): the button, the working line, and what the last batch did.
    pub(super) fn add_files_bar(&mut self, ui: &mut egui::Ui, p: &Pal) {
        ui.horizontal_wrapped(|ui| {
            if ui.add_enabled(self.import.job.is_none(), egui::Button::new("📎 Add files")).on_hover_text(".md, .txt, .docx or .pdf, or drag files onto the window").clicked() {
                let paths = pick_files();
                let target = ImportTarget::Knowledge(self.current_project.clone());
                self.import_files(target, paths);
            }
            if self.import.job.is_some() {
                ui.spinner();
                let secs = (self.now - self.import.started).max(0.0) as u64;
                ui.label(RichText::new(format!("Reading and sorting… {secs} s")).size(size::SMALL).color(p.dim));
            } else {
                ui.label(RichText::new("or drag files onto the window. Resolve files each one in the right place.").size(size::TINY).color(p.faint));
            }
        });
        let log = self.import.log.clone();
        for (_, m) in log {
            wrap_label(ui, RichText::new(m.clone()).size(size::SMALL).color(if m.starts_with('⚠') { p.orange } else { p.green }));
        }
    }

    /// Imported files in scope, each with its place; one click moves it.
    pub(super) fn imported_files_card(&mut self, ui: &mut egui::Ui, p: &Pal, brief: Option<Id>) {
        let scope_pid = self.current_project.clone();
        let list: Vec<(Id, String, String, String)> = self
            .store
            .data
            .pages
            .iter()
            .filter(|pg| files::category_of(pg).is_some())
            .filter(|pg| scope_pid.is_none() || (brief.is_some() && (pg.parent == brief || Some(&pg.id) == brief.as_ref())))
            .map(|pg| (pg.id.clone(), pg.title.clone(), files::category_of(pg).unwrap_or("reference").to_string(), pg.tags.iter().find(|t| t.starts_with("sorted:")).map(|t| t.trim_start_matches("sorted:").to_string()).unwrap_or_default()))
            .collect();
        if list.is_empty() {
            return;
        }
        let mut moves: Vec<(Id, &'static str)> = vec![];
        let mut open: Option<Id> = None;
        card(ui, p, |ui| {
            ui.label(RichText::new("Imported files").font(fs(size::SECTION)));
            wrap_label(ui, RichText::new("Where each file was put. Click another place to move it.").size(size::TINY).color(p.faint));
            for (id, title, cat, how) in &list {
                ui.add_space(6.0);
                ui.horizontal_wrapped(|ui| {
                    if wrap_link(ui, RichText::new(title).size(size::BODY)).clicked() {
                        open = Some(id.clone());
                    }
                    let by = match how.as_str() {
                        "ai" => "sorted by the AI",
                        "you" => "placed by you",
                        _ => "a guess (no AI)",
                    };
                    ui.label(RichText::new(by).size(size::TINY).color(p.faint));
                });
                ui.horizontal_wrapped(|ui| {
                    for (k, l) in files::CATEGORIES {
                        if pill(ui, p, l, cat == k).clicked() && cat != k {
                            moves.push((id.clone(), k));
                        }
                    }
                });
            }
        });
        if let Some(id) = open {
            self.go(View::Page(id));
        }
        for (id, k) in moves {
            self.undoable(&format!("Moved a file to {}", files::label(k)), "edit", |d| files::recategorise(d, &id, k));
            self.toast(format!("Moved to {}", files::label(k)));
        }
    }

    /// The intake's "Attach a brief or notes" row.
    pub(super) fn attach_row(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut crate::resolve::Intake) {
        ui.horizontal_wrapped(|ui| {
            let careers = it.mode == Mode::Careers;
            if ui.add_enabled(self.import.job.is_none(), egui::Button::new(if careers { "📎 Attach" } else { "📎 Attach a brief or notes" })).on_hover_text(".md, .txt, .docx or .pdf, or drag files onto the window").clicked() {
                let paths = pick_files();
                self.import_files(ImportTarget::Intake(it.id.clone()), paths);
            }
            if self.import.job.is_some() {
                ui.spinner();
                ui.label(RichText::new("Reading…").size(size::SMALL).color(p.dim));
            } else if !careers {
                ui.label(RichText::new("Resolve reads them like your own words; a claim counts as evidence only when the file states it as fact.").size(size::TINY).color(p.faint));
            }
        });
        let mut remove: Option<usize> = None;
        for (i, a) in it.attachments.iter().enumerate() {
            ui.horizontal_wrapped(|ui| {
                chip(ui, &format!("📎 {}", a.name), p.blue);
                // Careers keeps them as your CV files (a PM intake files them in Knowledge)
                let place = if it.mode == Mode::Careers { "kept with your CV".to_string() } else { format!("will go to {}", files::label(&a.place)) };
                ui.label(RichText::new(format!("{} words · {place}", a.text.split_whitespace().count())).size(size::TINY).color(p.faint));
                if ui.small_button("×").on_hover_text("Remove").clicked() {
                    remove = Some(i);
                }
            });
        }
        if let Some(i) = remove {
            it.attachments.remove(i);
            it.touch();
        }
    }
}
