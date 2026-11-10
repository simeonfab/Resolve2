//! Knowledge base (C-E12 to C-E15, C-E17; Pack C "Knowledge"): five tabs, a card library per tab
//! (filter chips with counts on Evidence and Records), the grounding bar, the Add sheet with its
//! five options (a new item lands first, outlined, for 2 s), the preview panel, per-file import
//! errors, and the New page dialog with the built-in templates and your own. The library's
//! contents come from `kb_items` (self-tested). With no project picked, the old Knowledge view
//! stays.

use super::frameworks::{exp_label, split_label};
use super::import_ui::{pick_files, ImportTarget};
use super::nav::{Section, KNOWLEDGE_TABS};
use super::theme::*;
use super::widgets::*;
use super::*;
use crate::resolve::{files, Kind, Record};
use eframe::egui::{self, RichText};

/// What a library card opens.
#[derive(Clone, Debug, PartialEq)]
pub enum KbTarget {
    Page(Id),
    Record(Id),
    Database(Id),
}

#[derive(Clone, Debug)]
pub struct KbItem {
    pub target: KbTarget,
    /// "Page", "File", "Research", "Hypothesis", "Open question"…
    pub kind: String,
    /// "Pinned", "Today", "Mon", "12 Oct", or where a record lives ("Frameworks")
    pub meta: String,
    pub title: String,
    pub snippet: String,
    pub updated: i64,
    /// For the Evidence and Records chips.
    pub record_kind: Option<Kind>,
}

/// "Today", a weekday within the week, else "12 Oct".
pub fn day_label(ms: i64, today: chrono::NaiveDate) -> String {
    let Some(d) = chrono::DateTime::from_timestamp_millis(ms).map(|x| x.with_timezone(&chrono::Local).date_naive()) else { return String::new() };
    let days = (today - d).num_days();
    match days {
        0 => "Today".into(),
        1..=6 => d.format("%a").to_string(),
        _ => d.format("%-d %b").to_string(),
    }
}

fn snippet_of(pg: &Page) -> String {
    pg.blocks.iter().filter(|b| b.kind.is_text()).map(|b| b.text.trim()).find(|t| !t.is_empty()).map(|t| t.lines().next().unwrap_or("").to_string()).unwrap_or_else(|| "Empty page".into())
}

/// The project's brief page, if it has one.
pub fn brief_of(d: &WorkspaceData, pid: &str) -> Option<Id> {
    d.context_for(pid).and_then(|c| c.page_id.clone()).or_else(|| d.project(pid).and_then(|x| x.page_id.clone())).filter(|x| d.page(x).is_some())
}

/// The brief and every page under it.
fn project_pages<'a>(d: &'a WorkspaceData, pid: &str) -> Vec<&'a Page> {
    let Some(b) = brief_of(d, pid) else { return vec![] };
    let mut out: Vec<&Page> = vec![];
    let mut frontier = vec![b.clone()];
    if let Some(pg) = d.page(&b) {
        out.push(pg);
    }
    while let Some(id) = frontier.pop() {
        for c in d.pages.iter().filter(|x| x.parent.as_deref() == Some(id.as_str()) && !x.template) {
            if !out.iter().any(|o| o.id == c.id) {
                out.push(c);
                frontier.push(c.id.clone());
            }
        }
    }
    out
}

fn is_research(pg: &Page) -> bool {
    pg.tags.iter().any(|t| t == "research")
}

/// Where a record is worked on.
fn home_of(k: Kind) -> &'static str {
    match k {
        Kind::Hypothesis | Kind::Experiment | Kind::Evidence => "Frameworks",
        Kind::Roadmap | Kind::Milestone | Kind::Dependency | Kind::Blocker => "Roadmap",
        _ => "Records",
    }
}

/// The chip name for a kind ("Open questions" for questions).
pub fn chip_name(k: Kind) -> &'static str {
    match k {
        Kind::Question => "Open questions",
        _ => k.plural(),
    }
}

/// A record's card kind ("Open question" for a question).
fn card_kind(k: Kind) -> &'static str {
    match k {
        Kind::Question => "Open question",
        Kind::Roadmap => "Roadmap",
        _ => k.label(),
    }
}

/// The library for one tab: 0 brief and documents, 1 research, 3 evidence and records,
/// 4 databases (the cut list, tab 2, is an editor, not a library).
pub fn kb_items(d: &WorkspaceData, pid: &str, tab: usize, today: chrono::NaiveDate) -> Vec<KbItem> {
    let brief = brief_of(d, pid);
    match tab {
        0 | 1 => project_pages(d, pid)
            .into_iter()
            .filter(|pg| if tab == 1 { is_research(pg) || files::category_of(pg) == Some("research") } else { !is_research(pg) && !matches!(files::category_of(pg), Some("research") | Some("evidence")) })
            .map(|pg| {
                let pinned = brief.as_deref() == Some(pg.id.as_str());
                KbItem {
                    target: KbTarget::Page(pg.id.clone()),
                    kind: if tab == 1 { "Research".into() } else if files::category_of(pg).is_some() { "File".into() } else { "Page".into() },
                    meta: if pinned { "Pinned".into() } else { day_label(pg.updated, today) },
                    title: if pg.title.trim().is_empty() { "Untitled".into() } else { pg.title.clone() },
                    snippet: snippet_of(pg),
                    updated: if pinned { i64::MAX } else { pg.updated },
                    record_kind: None,
                }
            })
            .collect(),
        3 => {
            let recs: Vec<&Record> = d.records.iter().filter(|r| r.project_id.as_deref() == Some(pid) && !matches!(r.kind, Kind::Guidance | Kind::Handoff | Kind::Update)).collect();
            recs.iter()
                .map(|r| {
                    let (title, sub) = match r.kind {
                        Kind::Hypothesis => {
                            let (l, s) = split_label(&r.title);
                            let tests: Vec<String> = recs.iter().filter(|e| e.kind == Kind::Experiment && e.parent_id.as_deref() == Some(r.id.as_str())).map(|e| exp_label(d, pid, &e.id)).collect();
                            let under = recs.iter().filter(|h| h.kind == Kind::Hypothesis && h.parent_id.as_deref() == Some(r.id.as_str())).count();
                            let how = if r.field("type") == "H0" && under > 0 {
                                format!("{under} under it")
                            } else if tests.is_empty() {
                                "no test yet".into()
                            } else {
                                format!("tested by {}", tests.join(", "))
                            };
                            (if l.is_empty() { s } else { format!("{l} · {s}") }, format!("{} · {how}", r.status))
                        }
                        Kind::Experiment => {
                            let tests = r.parent_id.as_ref().and_then(|h| d.record(h)).map(|h| split_label(&h.title).0).filter(|l| !l.is_empty());
                            (format!("{} · {}", exp_label(d, pid, &r.id), r.title), match tests {
                                Some(h) => format!("{} · tests {h}", r.status),
                                None => r.status.clone(),
                            })
                        }
                        Kind::Roadmap => (r.title.clone(), format!("{} · {}", super::sections::lane_of(r), r.status)),
                        _ => (r.title.clone(), r.status.clone()),
                    };
                    KbItem { target: KbTarget::Record(r.id.clone()), kind: card_kind(r.kind).into(), meta: home_of(r.kind).into(), title, snippet: sub, updated: r.updated, record_kind: Some(r.kind) }
                })
                .collect()
        }
        4 => d
            .databases
            .iter()
            .filter(|x| !x.template)
            .map(|db| KbItem { target: KbTarget::Database(db.id.clone()), kind: "Database".into(), meta: day_label(db.updated, today), title: db.name.clone(), snippet: format!("{} row{} · {} column{}", db.rows.len(), if db.rows.len() == 1 { "" } else { "s" }, db.columns.len(), if db.columns.len() == 1 { "" } else { "s" }), updated: db.updated, record_kind: None })
            .collect(),
        _ => vec![],
    }
}

/// (pages and files, records) Resolve can search for this project.
pub fn kb_counts(d: &WorkspaceData, pid: &str) -> (usize, usize) {
    let pages = project_pages(d, pid).len();
    let recs = d.records.iter().filter(|r| r.project_id.as_deref() == Some(pid) && !matches!(r.kind, Kind::Guidance | Kind::Handoff | Kind::Update)).count();
    (pages, recs)
}



enum KbAct {
    Tab(usize),
    Open(KbTarget),
    Preview(KbTarget),
    Add,
    NewPage,
    ImportFiles,
    DismissErrors,
    Chip(Option<Kind>),
    NewDatabase,
    ImportCsv,
    Crm,
}

impl App {
    /// The redesigned Knowledge base for one project. Returns false with no project picked.
    pub(super) fn kb_page(&mut self, ui: &mut egui::Ui, p: &Pal) -> bool {
        let Some(pid) = self.current_project.clone() else { return false };
        let name = self.store.data.project(&pid).map(|x| x.name.clone()).unwrap_or_default();
        let tabs: Vec<&str> = KNOWLEDGE_TABS.iter().map(|t| t.0).collect();
        let tab = self.resolve_ui.knowledge_tab.min(tabs.len() - 1);
        let today_ = today();
        let mut items = kb_items(&self.store.data, &pid, tab, today_);
        let (n_pages, n_recs) = kb_counts(&self.store.data, &pid);
        // the one just added lands first, outlined, for 2 s (C-E13)
        let fresh: Option<String> = self.temp.get("kb_new").and_then(|s| s.split_once('\t')).filter(|(_, t)| t.parse::<f64>().map_or(false, |t| self.now - t < 2.0)).map(|(id, _)| id.to_string());
        if self.temp.contains_key("kb_new") && fresh.is_none() {
            self.temp.remove("kb_new");
        } else if fresh.is_some() {
            ui.ctx().request_repaint();
        }
        items.sort_by_key(|i| std::cmp::Reverse(i.updated));
        if let Some(f) = &fresh {
            if let Some(pos) = items.iter().position(|i| matches!(&i.target, KbTarget::Page(x) | KbTarget::Record(x) if x == f)) {
                let it = items.remove(pos);
                items.insert(0, it);
            }
        }
        let query = self.temp.get("kb_find").cloned().unwrap_or_default();
        let q = query.trim().to_lowercase();
        let chip: Option<Kind> = self.resolve_ui.kind_filter;
        let shown: Vec<&KbItem> = items.iter().filter(|i| q.is_empty() || i.title.to_lowercase().contains(&q) || i.snippet.to_lowercase().contains(&q)).filter(|i| tab != 3 || chip.map_or(true, |k| i.record_kind == Some(k))).collect();
        // the databases tab is empty when there are no databases (C-E18)
        let nothing = if tab == 4 { items.is_empty() } else { n_pages == 0 && n_recs == 0 };
        let state = match self.view_state {
            ViewState::Full if nothing => ViewState::Empty,
            s => s,
        };
        // C-E15: files from the last batch that couldn't be read
        let mut failed: Vec<String> = self.import.log.iter().filter(|(id, m)| id.is_none() && m.starts_with('⚠')).map(|(_, m)| m.trim_start_matches('⚠').trim().to_string()).collect();
        let filed = self.import.log.iter().filter(|(id, _)| id.is_some()).count();
        if state == ViewState::Error && failed.is_empty() {
            failed.push("Pricing notes.pdf: This PDF's text couldn't be read (it may be a scan, or use unusual fonts). Copy its text into a .txt or .docx file and add that instead.".into());
        }
        let preview = self.temp.get("kb_preview").cloned();
        let busy_import = self.import.job.is_some();
        let mut acts: Vec<KbAct> = vec![];
        let mut find = query.clone();
        egui::ScrollArea::vertical().auto_shrink([false, false]).id_salt(("kb", &pid, tab)).show(ui, |ui| {
            egui::Frame::none().inner_margin(egui::Margin { left: space::WORK_SIDE, right: space::WORK_SIDE, top: space::WORK_TOP, bottom: space::S40 * 2.0 }).show(ui, |ui| {
                capped(ui, space::CAP_DEFAULT, |ui| {
                    eyebrow_head(ui, p, &name, "Knowledge base", |ui| {
                        ui.spacing_mut().item_spacing.x = space::S10;
                        if button(ui, p, ButtonKind::Primary, "+ Add", false, state != ViewState::Loading).clicked() {
                            acts.push(KbAct::Add);
                        }
                        find_field(ui, p, egui::Id::new("kb-find"), &mut find, "Find in this project", space::FIND_W);
                    });
                    ui.add_space(space::S10);
                    if let Some(t) = underline_tabs(ui, p, &tabs, tab) {
                        acts.push(KbAct::Tab(t));
                    }
                    ui.add_space(space::S14);
                    match state {
                        ViewState::Loading => {
                            let cols = if ui.available_width() >= space::LIB_FOUR_FROM { 4 } else { 3 };
                            let cw = (ui.available_width() - space::LIB_GAP * (cols - 1) as f32) / cols as f32;
                            for _ in 0..2 {
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = space::LIB_GAP;
                                    for _ in 0..cols {
                                        ui.allocate_ui_with_layout(egui::vec2(cw, space::LIB_CARD_H), Layout::top_down(Align::Min), |ui| {
                                            ui.set_width(cw);
                                            let (r, _) = ui.allocate_exact_size(egui::vec2(cw, space::LIB_CARD_H), egui::Sense::hover());
                                            ui.painter().rect(r, radius::CARD, p.card, egui::Stroke::new(stroke::HAIR, p.line));
                                        });
                                    }
                                });
                                ui.add_space(space::LIB_GAP);
                            }
                            ui.label(RichText::new("Reading the knowledge base…").size(size::SMALL).color(p.dim));
                            return;
                        }
                        ViewState::Empty if tab == 4 => {
                            ui.add_space(space::S40 * 1.5);
                            let (a, b) = empty_center_two(ui, p, Icon::Knowledge, "No databases yet.", "A database is a table you sort and filter: clients, hours, prices. Bring in a spreadsheet as CSV, or start from a template.", "Import a CSV", "Blank table");
                            if a {
                                acts.push(KbAct::ImportCsv);
                            }
                            if b {
                                acts.push(KbAct::NewDatabase);
                            }
                            ui.vertical_centered(|ui| {
                                if link(ui, p, "Start from the CRM template").clicked() {
                                    acts.push(KbAct::Crm);
                                }
                            });
                            return;
                        }
                        ViewState::Empty => {
                            ui.add_space(space::S40 * 1.5);
                            let (a, b) = empty_center_two(ui, p, Icon::Knowledge, "Nothing filed yet.", "Drop in notes, PDFs or Word files. Resolve reads each one and files it in the right tab. Or write the first page.", "Add files", "New page");
                            if a {
                                acts.push(KbAct::ImportFiles);
                            }
                            if b {
                                acts.push(KbAct::NewPage);
                            }
                            return;
                        }
                        _ => {}
                    }
                    if busy_import {
                        ui.horizontal(|ui| {
                            spinner_ring(ui, ui.cursor().min + egui::vec2(space::S8, space::S8), space::S6);
                            ui.add_space(space::S22);
                            let secs = (self.now - self.import.started).max(0.0) as u64;
                            ui.label(RichText::new(format!("Reading and filing your files… {secs} s")).size(size::ROW_SMALL).color(p.dim));
                        });
                        ui.add_space(space::S10);
                    }
                    if !failed.is_empty() {
                        // C-E15: each file that couldn't be read, with why; the rest are filed
                        let first_name = failed[0].split_once(": ").map(|x| x.0.to_string()).unwrap_or_else(|| "a file".into());
                        let title = if failed.len() == 1 { format!("Resolve couldn't read {first_name}.") } else { format!("Resolve couldn't read {} files.", failed.len()) };
                        let mut body = failed.iter().map(|f| f.split_once(": ").map(|(n, e)| if failed.len() == 1 { e.to_string() } else { format!("{n}: {e}") }).unwrap_or_else(|| f.clone())).collect::<Vec<_>>().join("\n");
                        if filed > 0 {
                            body.push_str(&format!("\nThe other {filed} file{} {} filed.", if filed == 1 { "" } else { "s" }, if filed == 1 { "is" } else { "are" }));
                        }
                        let a = error_center_as(ui, p, &title, &body, "Choose another file", "Dismiss");
                        if a.retry {
                            acts.push(KbAct::ImportFiles);
                        }
                        if a.manual {
                            acts.push(KbAct::DismissErrors);
                        }
                        ui.add_space(space::S14);
                        if state == ViewState::Error {
                            return;
                        }
                    }
                    if tab == 2 {
                        white_card(ui, p, false, |ui| {
                            ui.spacing_mut().item_spacing.y = space::S8;
                            self.knowledge_cut_body(ui, p, &pid);
                        });
                        ui.add_space(space::S14);
                    } else {
                        if tab == 3 {
                            // filter chips with counts
                            let mut kinds: Vec<Kind> = vec![];
                            for i in &items {
                                if let Some(k) = i.record_kind {
                                    if !kinds.contains(&k) {
                                        kinds.push(k);
                                    }
                                }
                            }
                            kinds.sort_by_key(|k| Kind::ALL.iter().position(|x| x == k).unwrap_or(99));
                            ui.horizontal_wrapped(|ui| {
                                ui.spacing_mut().item_spacing = egui::vec2(space::S8, space::S8);
                                if filter_chip(ui, p, "All", Some(items.len()), chip.is_none()).clicked() {
                                    acts.push(KbAct::Chip(None));
                                }
                                for k in kinds {
                                    let n = items.iter().filter(|i| i.record_kind == Some(k)).count();
                                    if filter_chip(ui, p, chip_name(k), Some(n), chip == Some(k)).clicked() {
                                        acts.push(KbAct::Chip(Some(k)));
                                    }
                                }
                            });
                            ui.add_space(space::S12);
                        }
                        let cols = if ui.available_width() >= space::LIB_FOUR_FROM { 4 } else { 3 };
                        let cw = ((ui.available_width() - space::LIB_GAP * (cols - 1) as f32) / cols as f32).floor();
                        let sel = preview.clone();
                        let tiles: Vec<Option<&KbItem>> = shown.iter().map(|i| Some(*i)).chain(std::iter::once(None)).collect();
                        for row in tiles.chunks(cols) {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = space::LIB_GAP;
                                for t in row {
                                    match t {
                                        Some(i) => {
                                            let key = match &i.target {
                                                KbTarget::Page(x) => format!("page:{x}"),
                                                KbTarget::Record(x) => format!("rec:{x}"),
                                                KbTarget::Database(x) => format!("db:{x}"),
                                            };
                                            let is_fresh = fresh.as_deref().map_or(false, |f| key.ends_with(f));
                                            let c = LibCard { kind: &i.kind, meta: &i.meta, title: &i.title, snippet: &i.snippet, fresh: is_fresh, selected: sel.as_deref() == Some(key.as_str()) };
                                            if library_card(ui, p, egui::Id::new(("kb-card", &key)), cw, &c).clicked() {
                                                acts.push(match &i.target {
                                                    KbTarget::Database(_) => KbAct::Open(i.target.clone()),
                                                    _ => KbAct::Preview(i.target.clone()),
                                                });
                                            }
                                        }
                                        None => {
                                            let (label, act) = match tab {
                                                3 => ("+ Record evidence or a question", KbAct::Add),
                                                4 => ("+ New database", KbAct::NewDatabase),
                                                _ => ("+ New page, or drop files here", KbAct::Add),
                                            };
                                            if library_add_tile(ui, p, cw, label).clicked() {
                                                acts.push(act);
                                            }
                                        }
                                    }
                                }
                            });
                            ui.add_space(space::LIB_GAP);
                        }
                        if !q.is_empty() && shown.is_empty() {
                            ui.label(RichText::new(format!("Nothing in this tab matches \"{}\".", query.trim())).size(size::ROW_SMALL).color(p.dim));
                            ui.add_space(space::S10);
                        }
                    }
                    let line = format!("Resolve searches everything here when it answers about this project. {n_pages} page{} and file{}, {n_recs} record{}.", if n_pages == 1 { "" } else { "s" }, if n_pages == 1 { "" } else { "s" }, if n_recs == 1 { "" } else { "s" });
                    if grounding_bar(ui, p, &line, "Import files") {
                        acts.push(KbAct::ImportFiles);
                    }
                });
            });
        });
        if find != query {
            self.temp.insert("kb_find".into(), find);
        }
        for a in acts {
            match a {
                KbAct::Tab(t) => {
                    self.resolve_ui.knowledge_tab = t;
                    self.resolve_ui.kind_filter = None;
                    self.temp.remove("kb_preview");
                }
                KbAct::Chip(k) => self.resolve_ui.kind_filter = k,
                KbAct::Add => {
                    self.temp.insert("kb_add".into(), self.now.to_string());
                }
                KbAct::NewPage => {
                    let brief = brief_of(&self.store.data, &pid);
                    self.temp.insert("new_page_dialog".into(), brief.unwrap_or_default());
                }
                KbAct::ImportFiles => {
                    let paths = pick_files();
                    self.import.log.retain(|(id, m)| !(id.is_none() && m.starts_with('⚠')));
                    self.import_files(ImportTarget::Knowledge(Some(pid.clone())), paths);
                }
                KbAct::DismissErrors => {
                    self.import.log.retain(|(id, m)| !(id.is_none() && m.starts_with('⚠')));
                    if self.view_state == ViewState::Error {
                        self.view_state = ViewState::Full;
                    }
                }
                KbAct::NewDatabase => {
                    self.new_database();
                }
                KbAct::ImportCsv => self.csv_pick(""),
                KbAct::Crm => {
                    let id = self.create_db_from_template("crm");
                    self.go(View::Database(id));
                }
                KbAct::Preview(t) => {
                    let key = match &t {
                        KbTarget::Page(x) => format!("page:{x}"),
                        KbTarget::Record(x) => format!("rec:{x}"),
                        KbTarget::Database(x) => format!("db:{x}"),
                    };
                    self.temp.insert("kb_preview".into(), key);
                    self.temp.insert("kb_preview_t".into(), self.now.to_string());
                }
                KbAct::Open(t) => self.kb_open(t),
            }
        }
        true
    }

    fn kb_open(&mut self, t: KbTarget) {
        self.temp.remove("kb_preview");
        match t {
            KbTarget::Page(id) => self.go(View::Page(id)),
            KbTarget::Database(id) => self.go(View::Database(id)),
            KbTarget::Record(id) => {
                let k = self.store.data.record(&id).map(|r| r.kind);
                match k {
                    Some(Kind::Hypothesis | Kind::Experiment) => {
                        self.resolve_ui.sel_record = Some(id);
                        self.go(View::Sec(Section::Frameworks));
                    }
                    Some(Kind::Roadmap) => {
                        self.temp.insert("rm_open".into(), id);
                        self.go(View::Sec(Section::Roadmap));
                    }
                    _ => {
                        self.resolve_ui.sel_record = Some(id.clone());
                        self.temp.insert("kb_preview".into(), format!("rec:{id}"));
                    }
                }
            }
        }
    }

    /// The overlays that belong to the Knowledge base and the page editor: the Add sheet, the
    /// preview panel and the New page dialog.
    pub(super) fn kb_overlays(&mut self, ctx: &egui::Context, p: &Pal) {
        let work = self.toaster.work.unwrap_or_else(|| ctx.screen_rect());
        if self.temp.contains_key("kb_add") {
            self.kb_add_sheet(ctx, p, work);
        }
        if let Some(key) = self.temp.get("kb_preview").cloned() {
            if matches!(self.view, View::Sec(Section::Knowledge)) {
                self.kb_preview_panel(ctx, p, &key);
            } else {
                self.temp.remove("kb_preview");
            }
        }
        if let Some(parent) = self.temp.get("new_page_dialog").cloned() {
            self.new_page_dialog(ctx, p, if parent.is_empty() { None } else { Some(parent) });
        }
    }

    /// The Add sheet (C-E13): five ways to add, and the drop note.
    fn kb_add_sheet(&mut self, ctx: &egui::Context, p: &Pal, work: egui::Rect) {
        let screen = ctx.screen_rect();
        let t0: f64 = self.temp.get("kb_add").and_then(|s| s.parse().ok()).unwrap_or(self.now);
        let el = ((self.now - t0) * 1000.0) as f32;
        let reduced = super::motion::reduced();
        let dim = super::motion::frame_at(el, moves::SHEET_DIM, reduced);
        let f = super::motion::frame_at(el, moves::SHEET_IN, reduced);
        if !f.done {
            ctx.request_repaint();
        }
        let mut close = false;
        egui::Area::new(egui::Id::new("kb-add-dim")).order(egui::Order::Foreground).fixed_pos(screen.min).show(ctx, |ui| {
            close |= backdrop(ui, screen, shadow::BACKDROP_SHEET * dim.alpha.clamp(0.0, 1.0));
        });
        let pid = self.current_project.clone();
        let brief = pid.as_deref().and_then(|x| brief_of(&self.store.data, x));
        let mut pick: Option<u8> = None;
        let w = space::ADD_SHEET_W.min(work.width() - space::S40);
        let resp = egui::Area::new(egui::Id::new("kb-add")).order(egui::Order::Foreground).fixed_pos(egui::pos2(work.center().x, screen.bottom() + (1.0 - f.t) * space::S40 * 10.0)).pivot(Align2::CENTER_BOTTOM).show(ctx, |ui| {
            ui.multiply_opacity(f.alpha.clamp(0.0, 1.0));
            egui::Frame::none().fill(p.card).rounding(egui::Rounding { nw: radius::SHEET, ne: radius::SHEET, sw: 0.0, se: 0.0 }).shadow(shadow::SHEET).inner_margin(egui::Margin { left: space::S22, right: space::S22, top: space::S10, bottom: space::S22 }).show(ui, |ui| {
                ui.set_width(w - space::S22 * 2.0);
                ui.spacing_mut().item_spacing.y = space::S12;
                ui.vertical_centered(|ui| {
                    let (r, _) = ui.allocate_exact_size(egui::vec2(space::S40, space::S4), egui::Sense::hover());
                    ui.painter().rect_filled(r, radius::PILL, p.track);
                });
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Add to Knowledge base").font(fserif(size::CARD_SERIF)).color(p.text));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if button(ui, p, ButtonKind::Secondary, "✕", true, true).on_hover_text("Close (Esc)").clicked() {
                            close = true;
                        }
                    });
                });
                let opts: [(&str, &str, &str); 5] = [
                    ("¶", "New page", "Write it here. Markdown underneath."),
                    ("⧉", "From a template", "Meeting notes, Project brief, Weekly review and more"),
                    ("↑", "Add files", "Resolve reads each one and files it in the right tab"),
                    ("◆", "Record evidence", "Something you saw or heard that bears on a hypothesis"),
                    ("?", "Open question", "Something you still need to find out"),
                ];
                for (k, (g, t, s)) in opts.iter().enumerate() {
                    if add_option(ui, p, g, t, s).clicked() {
                        pick = Some(k as u8);
                    }
                }
                dashed_note(ui, p, "Or drop .md, .txt, .docx or .pdf files anywhere on this window");
            });
        });
        let _ = resp;
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            close = true;
        }
        if let Some(k) = pick {
            close = true;
            match k {
                0 => {
                    let id = self.new_page(brief.clone());
                    self.temp.insert("kb_new".into(), format!("{id}\t{}", self.now));
                }
                1 => {
                    self.temp.insert("new_page_dialog".into(), brief.clone().unwrap_or_default());
                }
                2 => {
                    let paths = pick_files();
                    self.import_files(ImportTarget::Knowledge(pid.clone()), paths);
                }
                _ => {
                    let (kind, title, label) = if k == 3 { (Kind::Evidence, "New evidence", "Recorded evidence") } else { (Kind::Question, "New open question", "Added an open question") };
                    let r = Record::new(kind, pid.clone(), title);
                    let id = r.id.clone();
                    self.undoable(label, "add", move |d| d.records.push(r));
                    self.resolve_ui.knowledge_tab = 3;
                    self.resolve_ui.kind_filter = None;
                    self.resolve_ui.sel_record = Some(id.clone());
                    self.temp.insert("kb_new".into(), format!("{id}\t{}", self.now));
                    self.temp.insert("kb_preview".into(), format!("rec:{id}"));
                }
            }
        }
        if close {
            self.temp.remove("kb_add");
        }
    }

    /// The preview panel (C-E14): what the item says, what it's linked to, and Open, Move to…
    /// and Delete. Records open their editor here.
    fn kb_preview_panel(&mut self, ctx: &egui::Context, p: &Pal, key: &str) {
        let screen = ctx.screen_rect();
        let t0: f64 = self.temp.get("kb_preview_t").and_then(|s| s.parse().ok()).unwrap_or(0.0);
        let el = ((self.now - t0) * 1000.0) as f32;
        let reduced = super::motion::reduced();
        let f = super::motion::frame_at(el, moves::SHEET_IN, reduced);
        let dim = super::motion::frame_at(el, moves::SHEET_DIM, reduced);
        if !f.done {
            ctx.request_repaint();
        }
        let mut close = false;
        egui::Area::new(egui::Id::new("kb-preview-dim")).order(egui::Order::Foreground).fixed_pos(screen.min).show(ctx, |ui| {
            close |= backdrop(ui, screen, shadow::BACKDROP_PANEL * dim.alpha.clamp(0.0, 1.0));
        });
        let w = space::PREVIEW_W.min(screen.width() * 0.6);
        let x = screen.right() - w + (1.0 - f.t) * w;
        let panel = egui::Rect::from_min_size(egui::pos2(x, screen.top()), egui::vec2(w, screen.height()));
        let (kind, id) = key.split_once(':').unwrap_or(("", ""));
        let id = id.to_string();
        let mut open: Option<KbTarget> = None;
        let mut delete = false;
        let mut move_to: Option<&'static str> = None;
        let mut research: Option<bool> = None;
        let pid = self.current_project.clone().unwrap_or_default();
        egui::Area::new(egui::Id::new("kb-preview")).order(egui::Order::Foreground).fixed_pos(panel.min).show(ctx, |ui| {
            ui.painter().rect_filled(panel, 0.0, p.card);
            ui.painter().add(egui::Shape::Rect(shadow::SHEET.as_shape(panel, egui::Rounding::ZERO)));
            ui.painter().rect_filled(panel, 0.0, p.card);
            let inner = panel.shrink2(egui::vec2(space::S24, space::S18));
            let mut child = ui.new_child(egui::UiBuilder::new().max_rect(inner).layout(Layout::top_down(Align::Min)));
            let ui = &mut child;
            ui.set_clip_rect(panel);
            ui.spacing_mut().item_spacing.y = space::S12;
            match kind {
                "page" => {
                    let Some(pg) = self.store.data.page(&id).cloned() else {
                        close = true;
                        return;
                    };
                    let cat = files::category_of(&pg).map(|c| c.to_string());
                    let tagk = if is_research(&pg) || cat.as_deref() == Some("research") { "Research" } else if cat.is_some() { "File" } else { "Page" };
                    ui.horizontal(|ui| {
                        let (fg, bg) = kind_pair(tagk);
                        tag(ui, tagk, fg, bg);
                        let added = match day_label(pg.created, today()) {
                            d if d == "Today" => "today".to_string(),
                            d if d.len() <= 3 => format!("on {d}"),
                            d => d,
                        };
                        let src = if cat.is_some() { format!("Imported · added {added}") } else { format!("Page · added {added}") };
                        ui.label(RichText::new(src).size(size::ROW_SMALL).color(p.dim));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if button(ui, p, ButtonKind::Secondary, "→", true, true).on_hover_text("Close").clicked() {
                                close = true;
                            }
                        });
                    });
                    ui.label(RichText::new(if pg.title.trim().is_empty() { "Untitled" } else { pg.title.as_str() }).font(fserif(size::HERO_TITLE)).color(p.text));
                    let text = pg.plain_text();
                    let links: Vec<String> = self.store.data.records.iter().filter(|r| r.links.iter().any(|l| l == &id)).map(|r| {
                        let (l, s) = split_label(&r.title);
                        if l.is_empty() { s } else { l }
                    }).collect();
                    let foot = space::BUTTON_H + space::S40 + if links.is_empty() { 0.0 } else { space::S22 };
                    let body_h = (inner.bottom() - ui.cursor().min.y - foot).max(space::S40 * 2.0);
                    egui::Frame::none().stroke(egui::Stroke::new(stroke::HAIR, p.line)).rounding(egui::Rounding::same(radius::BUTTON)).inner_margin(egui::Margin::same(space::S18)).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.set_height(body_h - space::S18 * 2.0);
                        egui::ScrollArea::vertical().id_salt(("kb-preview-body", &id)).auto_shrink([false, false]).show(ui, |ui| {
                            ui.add(egui::Label::new(RichText::new(if text.trim().is_empty() { "This page is empty." } else { text.as_str() }).size(size::ROW).color(p.text)).wrap());
                        });
                    });
                    if !links.is_empty() {
                        lead_line(ui, p, "Linked to", &links.join(" and "));
                    }
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = space::S8;
                        let bw = ui.available_width() - space::S40 * 5.5;
                        ui.allocate_ui_with_layout(egui::vec2(bw, space::BUTTON_H + space::S8), Layout::top_down_justified(Align::Center), |ui| {
                            if wide_button(ui, p, ButtonKind::Dark, "Open", true).clicked() {
                                open = Some(KbTarget::Page(id.clone()));
                            }
                        });
                        let mv = button(ui, p, ButtonKind::Secondary, "Move to…", false, true);
                        let menu = egui::Id::new("kb-move");
                        if mv.clicked() {
                            ui.memory_mut(|m| m.toggle_popup(menu));
                        }
                        egui::popup_above_or_below_widget(ui, menu, &mv, egui::AboveOrBelow::Above, egui::PopupCloseBehavior::CloseOnClick, |ui| {
                            ui.set_min_width(space::S40 * 5.0);
                            if cat.is_some() {
                                for (k, l) in files::CATEGORIES {
                                    if ui.selectable_label(cat.as_deref() == Some(k), l).clicked() {
                                        move_to = Some(k);
                                    }
                                }
                            } else {
                                if ui.selectable_label(!is_research(&pg), "Brief and documents").clicked() {
                                    research = Some(false);
                                }
                                if ui.selectable_label(is_research(&pg), "Research").clicked() {
                                    research = Some(true);
                                }
                            }
                        });
                        if button(ui, p, ButtonKind::Secondary, "Delete", false, true).on_hover_text("You can undo it").clicked() {
                            delete = true;
                        }
                    });
                }
                "rec" => {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Record").size(size::ROW_SMALL).color(p.dim));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if button(ui, p, ButtonKind::Secondary, "→", true, true).on_hover_text("Close").clicked() {
                                close = true;
                            }
                        });
                    });
                    let k = self.store.data.record(&id).map(|r| r.kind);
                    if k.is_none() {
                        close = true;
                        return;
                    }
                    if matches!(k, Some(Kind::Hypothesis | Kind::Experiment | Kind::Roadmap)) {
                        ui.horizontal(|ui| {
                            if button(ui, p, ButtonKind::Dark, &format!("Open in {}", if k == Some(Kind::Roadmap) { "Roadmap" } else { "Frameworks" }), false, true).clicked() {
                                open = Some(KbTarget::Record(id.clone()));
                            }
                        });
                    }
                    egui::ScrollArea::vertical().id_salt(("kb-preview-rec", &id)).auto_shrink([false, false]).show(ui, |ui| {
                        self.record_editor(ui, p, &pid, &id);
                    });
                }
                _ => close = true,
            }
        });
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            close = true;
        }
        if let Some(t) = open {
            self.kb_open(t);
            return;
        }
        if delete {
            let title = self.store.data.page(&id).map(|x| x.title.clone()).unwrap_or_default();
            let pid2 = id.clone();
            self.undoable(&format!("Deleted \"{title}\""), "delete", move |d| d.delete_page(&pid2));
            close = true;
        }
        if let Some(k) = move_to {
            let pid2 = id.clone();
            self.undoable(&format!("Moved a file to {}", files::label(k)), "edit", move |d| files::recategorise(d, &pid2, k));
        }
        if let Some(r) = research {
            let pid2 = id.clone();
            self.undoable(if r { "Moved a page to Research" } else { "Moved a page to Brief and documents" }, "edit", move |d| {
                if let Some(pg) = d.pages.iter_mut().find(|x| x.id == pid2) {
                    pg.tags.retain(|t| t != "research");
                    if r {
                        pg.tags.push("research".into());
                    }
                    pg.updated = now_ms();
                }
            });
        }
        if close {
            self.temp.remove("kb_preview");
        }
    }

    /// The New page dialog (C-E17): Blank and the built-in templates as tiles, then your own.
    pub(super) fn new_page_dialog(&mut self, ctx: &egui::Context, p: &Pal, parent: Option<Id>) {
        let screen = ctx.screen_rect();
        let work = self.toaster.work.unwrap_or(screen);
        let mut close = false;
        egui::Area::new(egui::Id::new("np-dim")).order(egui::Order::Foreground).fixed_pos(screen.min).show(ctx, |ui| {
            close |= backdrop(ui, screen, shadow::BACKDROP_DIALOG);
        });
        let mut choice: Option<Option<usize>> = None;
        let mut mine_pick: Option<Id> = None;
        let mine: Vec<(Id, String)> = self.store.data.pages.iter().filter(|x| x.template).map(|x| (x.id.clone(), x.title.clone())).collect();
        let w = space::NEW_PAGE_W.min(work.width() - space::S40);
        egui::Area::new(egui::Id::new("np")).order(egui::Order::Foreground).fixed_pos(egui::pos2(work.center().x, screen.top() + space::S40 * 1.75)).pivot(Align2::CENTER_TOP).show(ctx, |ui| {
            dialog_frame(p).inner_margin(egui::Margin::same(space::S22)).show(ui, |ui| {
                ui.set_width(w - space::S22 * 2.0);
                ui.spacing_mut().item_spacing.y = space::S10;
                ui.horizontal(|ui| {
                    ui.label(RichText::new("New page").font(fserif(size::CARD_SERIF)).color(p.text));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if button(ui, p, ButtonKind::Secondary, "✕", true, true).on_hover_text("Close (Esc)").clicked() {
                            close = true;
                        }
                    });
                });
                let cw = ((ui.available_width() - space::S10 * 2.0) / 3.0).floor();
                let tiles: Vec<(Option<usize>, &str, &str)> = std::iter::once((None, "Blank", "An empty page")).chain(crate::templates::PAGE_TEMPLATES.iter().enumerate().map(|(i, t)| (Some(i), t.name, t.about))).collect();
                for row in tiles.chunks(3) {
                    ui.horizontal_top(|ui| {
                        ui.spacing_mut().item_spacing.x = space::S10;
                        for (i, name, about) in row {
                            if template_tile(ui, p, cw, name, about, i.is_none()).clicked() {
                                choice = Some(*i);
                            }
                        }
                    });
                }
                ui.add_space(space::S4);
                ui.label(RichText::new("Your templates").font(fs(size::ROW_SMALL)).color(p.text));
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = space::S10;
                    for (id, title) in &mine {
                        if button(ui, p, ButtonKind::Secondary, title, false, true).clicked() {
                            mine_pick = Some(id.clone());
                        }
                    }
                    ui.label(RichText::new(if mine.is_empty() { "None yet. Any page's ⋯ menu: Save a copy as template" } else { "Any page's ⋯ menu: Save a copy as template" }).size(size::ROW_SMALL).color(p.dim));
                });
            });
        });
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            close = true;
        }
        if let Some(c) = choice {
            close = true;
            let id = match c {
                None => self.new_page(parent.clone()),
                Some(i) => self.page_from_builtin_in(i, parent.clone()),
            };
            self.temp.insert("kb_new".into(), format!("{id}\t{}", self.now));
        }
        if let Some(t) = mine_pick {
            close = true;
            let id = self.page_from_template_in(&t, parent.clone());
            self.temp.insert("kb_new".into(), format!("{id}\t{}", self.now));
        }
        if close {
            self.temp.remove("new_page_dialog");
        }
    }

    /// The cut list's editor, without its old card (the Knowledge base wraps it).
    fn knowledge_cut_body(&mut self, ui: &mut egui::Ui, p: &Pal, pid: &str) {
        let Some(mut ctx) = self.ensure_context(pid) else { return };
        let before = ctx.recommendation.do_not_yet.clone();
        let mut remove: Option<usize> = None;
        let mut add = false;
        ui.label(RichText::new("Cut list").font(fserif(size::CARD_SERIF)).color(p.text));
        ui.label(RichText::new("What you decided not to do yet, and why. Themes you leave off the roadmap during the intake land here.").size(size::ROW_SMALL).color(p.dim));
        if ctx.recommendation.do_not_yet.is_empty() {
            ui.label(RichText::new("Nothing cut yet.").size(size::ROW_SMALL).color(p.dim));
        }
        let w = ui.available_width();
        for (i, n) in ctx.recommendation.do_not_yet.iter_mut().enumerate() {
            edit_row(ui, p, |ui| {
                ui.horizontal_top(|ui| {
                    let tw = ui.available_width() - space::S22 - space::S12;
                    ui.vertical(|ui| {
                        ui.set_width(tw);
                        ui.spacing_mut().item_spacing.y = space::S4;
                        inline_edit(ui, p, egui::Id::new(("kb-cut", i)), &mut n.item, fs(size::ROW + 0.5), p.text, tw, "What you're not doing yet", false);
                        inline_edit(ui, p, egui::Id::new(("kb-cut-why", i)), &mut n.reason, fp(size::ROW_SMALL), p.dim, tw, "Why not now", true);
                    });
                    if remove_x(ui, p).clicked() {
                        remove = Some(i);
                    }
                });
            });
        }
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = space::S10;
            let enter = text_field(ui, p, egui::Id::new("kb-cut-new"), &mut self.resolve_ui.new_cut, "Something you're choosing not to do yet", false, w - space::S40 * 2.5).lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            add = (button(ui, p, ButtonKind::Secondary, "Add", false, !self.resolve_ui.new_cut.trim().is_empty()).clicked() || enter) && !self.resolve_ui.new_cut.trim().is_empty();
        });
        if let Some(i) = remove {
            ctx.recommendation.do_not_yet.remove(i);
        }
        if add {
            ctx.recommendation.do_not_yet.push(crate::resolve::intake::NotYet { item: self.resolve_ui.new_cut.trim().to_string(), reason: String::new() });
            self.resolve_ui.new_cut.clear();
        }
        if ctx.recommendation.do_not_yet != before {
            ctx.updated = now_ms();
            if let Some(x) = self.store.data.contexts.iter_mut().find(|x| x.id == ctx.id) {
                *x = ctx;
            }
            self.touch();
        }
    }
}
