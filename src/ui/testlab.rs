//! Test Intake: the lab's home screen (runs, scenarios, provider per run, compare, export,
//! replay, promote). Runs in its own process and data folder; see resolve/testlab.rs.

use super::theme::*;
use super::widgets::*;
use super::*;
use crate::resolve::intake::Intake;
use crate::resolve::journey;
use crate::resolve::testlab::{self, Scenario};

#[derive(Default)]
pub struct LabUi {
    /// Provider for new runs: "" app default, "offline", or "provider|model|url".
    pub provider: String,
    pub model: String,
    pub url: String,
    pub compare: Vec<Id>,
    pub promote: Option<Id>,
    pub msg: String,
    pub save_name: String,
}

const PROVIDERS: [(&str, &str); 7] = [
    ("", "App default (Settings → AI)"),
    ("offline", "No AI — fixed question set"),
    ("ollama", "Ollama (local)"),
    ("openai", "LM Studio / other OpenAI-compatible"),
    ("claude-code", "Claude Code (your Claude subscription)"),
    ("anthropic", "Anthropic API key"),
    ("openai-cloud", "OpenAI API key"),
];

impl App {
    fn lab_provider_string(&self) -> String {
        let l = &self.lab;
        match l.provider.as_str() {
            "" | "offline" => l.provider.clone(),
            p => format!("{p}|{}|{}", l.model.trim(), l.url.trim()),
        }
    }

    pub fn testlab_new_run(&mut self, scn: Option<Scenario>) {
        let run = testlab::new_run(scn.as_ref(), &self.lab_provider_string());
        let id = run.id.clone();
        self.store.data.intakes.push(run);
        self.touch();
        self.go(View::Intake(id));
    }

    pub(super) fn testlab_branch(&mut self, it: &Intake, k: usize) {
        self.put_intake(it);
        let b = testlab::branch(it, k);
        let id = b.id.clone();
        self.store.data.intakes.push(b);
        self.touch();
        self.toast(format!("Branched from question {} — try another answer", k + 1));
        self.resolve_ui.focus_qa = None;
        self.go(View::Intake(id));
    }

    /// Fill the current question from the run's scenario and continue.
    pub(super) fn testlab_fill(&mut self, it: &mut Intake) {
        let Some(scn) = testlab::scenario(&it.scenario) else { return };
        let k = it.cursor;
        let Some(q) = it.qas.get_mut(k) else { return };
        let Some(a) = testlab::answer_for(&scn, &q.target) else {
            self.toast("The scenario has no answer for this one — type your own");
            return;
        };
        q.answer = a;
        let qid = q.id.clone();
        it.flag_dependents(&qid);
        let last = k + 1 == it.qas.len();
        if last {
            self.put_intake(it);
            let snapshot = it.clone();
            let cfg = journey::cfg_for(it);
            let id = it.id.clone();
            self.spawn_job("Noting the scenario's answer… choosing the next question", move || {
                let out = journey::run_answer(cfg, &snapshot, Some(&qid), true);
                super::resolve::JobOut::Answer(id, out)
            });
        } else {
            it.cursor += 1;
        }
    }

    pub fn testlab_view(&mut self, ui: &mut egui::Ui, p: &Pal) {
        egui::Frame::none().fill(p.banner_bg).inner_margin(egui::Margin::symmetric(20.0, 8.0)).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(format!("TEST MODE — sandbox data in {} · never shown to Claude/ChatGPT over MCP · nothing touches your real workspaces", crate::store::root().display())).size(size::SMALL).color(p.banner_text).strong());
        });
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            centered(ui, 1100.0, |ui| {
                ui.add_space(20.0);
                ui.label(RichText::new("Test Intake").font(fserif(size::HERO)));
                ui.label(RichText::new("Run the full intake journey — questions, proposal, bridge and reveal — on sample scenarios or your own answers, with any AI provider. Inspect every question's reasoning and the raw AI calls.").size(size::SMALL).color(p.dim));
                if !self.lab.msg.is_empty() {
                    ui.add_space(6.0);
                    ui.label(RichText::new(&self.lab.msg).size(size::SMALL).color(p.accent));
                }
                ui.add_space(14.0);
                self.lab_new(ui, p);
                ui.add_space(14.0);
                self.lab_runs(ui, p);
                if self.lab.compare.len() == 2 {
                    ui.add_space(14.0);
                    self.lab_compare(ui, p);
                }
                ui.add_space(40.0);
            });
        });
    }

    fn lab_new(&mut self, ui: &mut egui::Ui, p: &Pal) {
        let scns = testlab::scenarios();
        let mut start: Option<Option<Scenario>> = None;
        card(ui, p, |ui| {
            ui.label(RichText::new("New run").font(fs(size::SECTION)));
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("AI for this run").size(size::SMALL).color(p.dim));
                egui::ComboBox::from_id_salt("lab-provider").width(260.0).selected_text(PROVIDERS.iter().find(|x| x.0 == self.lab.provider).map(|x| x.1).unwrap_or("?")).show_ui(ui, |ui| {
                    for (k, label) in PROVIDERS {
                        if ui.selectable_label(self.lab.provider == k, label).clicked() {
                            self.lab.provider = k.into();
                            let (m, u) = match k {
                                "ollama" => ("qwen3", "http://localhost:11434"),
                                "openai" => ("", "http://localhost:1234"),
                                "anthropic" => (crate::ai::CLAUDE_MODEL, ""),
                                _ => ("", ""),
                            };
                            self.lab.model = m.into();
                            self.lab.url = u.into();
                        }
                    }
                });
                if !matches!(self.lab.provider.as_str(), "" | "offline" | "claude-code") {
                    ui.label(RichText::new("model").size(size::SMALL).color(p.dim));
                    ui.add(egui::TextEdit::singleline(&mut self.lab.model).desired_width(140.0));
                }
                if matches!(self.lab.provider.as_str(), "ollama" | "openai") {
                    ui.label(RichText::new("address").size(size::SMALL).color(p.dim));
                    ui.add(egui::TextEdit::singleline(&mut self.lab.url).desired_width(190.0));
                }
            });
            if let Some(note) = super::settings::model_note(&self.lab.provider, &self.lab.model) {
                ui.label(RichText::new(note).size(size::TINY).color(p.orange));
            }
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                for s in &scns {
                    if ui.button(format!("{}{}", if s.saved { "★ " } else { "" }, s.name)).on_hover_text(&s.opening).clicked() {
                        start = Some(Some(s.clone()));
                    }
                }
                if ui.add(egui::Button::new(RichText::new("Use my own answers").color(p.on_accent)).fill(p.accent)).clicked() {
                    start = Some(None);
                }
            });
            ui.label(RichText::new("Scenario runs start with the scenario's opening text; use \"Answer from scenario\" in the inspector to fill answers, or type your own. ★ = answers you saved from a run (replay).").size(size::TINY).color(p.faint));
        });
        if let Some(s) = start {
            self.testlab_new_run(s);
        }
    }

    fn lab_runs(&mut self, ui: &mut egui::Ui, p: &Pal) {
        let mut runs: Vec<Intake> = self.store.data.intakes.iter().filter(|i| i.is_v2()).cloned().collect();
        runs.sort_by_key(|i| std::cmp::Reverse(i.updated));
        let mut open: Option<Id> = None;
        let mut export: Option<Intake> = None;
        let mut save: Option<Intake> = None;
        let mut promote_go: Option<Intake> = None;
        let mut delete: Option<Id> = None;
        card(ui, p, |ui| {
            ui.label(RichText::new(format!("Runs ({})", runs.len())).font(fs(size::SECTION)));
            ui.label(RichText::new("Tick two runs with proposals to compare them side by side.").size(size::TINY).color(p.faint));
            if runs.is_empty() {
                ui.label(RichText::new("No runs yet — start one above.").color(p.faint));
            }
            for r in &runs {
                ui.separator();
                ui.horizontal_wrapped(|ui| {
                    let mut ticked = self.lab.compare.contains(&r.id);
                    if ui.add_enabled(r.proposal.is_some(), egui::Checkbox::new(&mut ticked, "")).changed() {
                        if ticked {
                            self.lab.compare.push(r.id.clone());
                            if self.lab.compare.len() > 2 {
                                self.lab.compare.remove(0);
                            }
                        } else {
                            self.lab.compare.retain(|x| x != &r.id);
                        }
                    }
                    let title = r.proposal.as_ref().map(|x| x.name.clone()).unwrap_or_else(|| r.title());
                    ui.label(RichText::new(title).font(fs(size::BODY)));
                    chip(ui, &r.stage, p.dim);
                    ui.label(RichText::new(format!("{} · {} · {} questions{}", if r.scenario.is_empty() { "own answers" } else { &r.scenario }, if r.provider.is_empty() { "app default".to_string() } else { r.provider.replace('|', " / ") }, r.qas.len(), r.branched_from.as_ref().map(|(_, k)| format!(" · branched at Q{}", k + 1)).unwrap_or_default())).size(size::TINY).color(p.faint));
                });
                ui.horizontal_wrapped(|ui| {
                    if ui.button("Open").clicked() {
                        open = Some(r.id.clone());
                    }
                    if ui.button("Export Markdown + JSON").clicked() {
                        export = Some(r.clone());
                    }
                    if ui.button("Save answers for replay").clicked() {
                        save = Some(r.clone());
                    }
                    if r.proposal.is_some() {
                        if self.lab.promote.as_deref() == Some(r.id.as_str()) {
                            ui.label(RichText::new("Create this as a real project in your Resolve 2 workspace?").size(size::SMALL).color(p.orange));
                            if ui.button("Yes, promote").clicked() {
                                promote_go = Some(r.clone());
                            }
                            if ui.button("Cancel").clicked() {
                                self.lab.promote = None;
                            }
                        } else if ui.button("Promote to real project…").clicked() {
                            self.lab.promote = Some(r.id.clone());
                        }
                    }
                    if ui.button("Delete").clicked() {
                        delete = Some(r.id.clone());
                    }
                });
            }
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Name for saved answers").size(size::TINY).color(p.faint));
                ui.add(egui::TextEdit::singleline(&mut self.lab.save_name).hint_text("e.g. Bakery v2").desired_width(200.0));
            });
        });
        if let Some(id) = open {
            self.resolve_ui.focus_qa = None;
            self.go(View::Intake(id));
        }
        if let Some(r) = export {
            self.lab.msg = match testlab::export(&r) {
                Ok((md, js)) => {
                    if let Some(dir) = md.parent() {
                        let _ = std::process::Command::new("explorer").arg(dir).spawn();
                    }
                    format!("Exported {} and {}", md.display(), js.display())
                }
                Err(e) => format!("Export failed: {e}"),
            };
        }
        if let Some(r) = save {
            let name = if self.lab.save_name.trim().is_empty() { format!("{} (saved)", r.title()) } else { self.lab.save_name.trim().to_string() };
            self.lab.msg = match testlab::save_answers(&r, &name) {
                Ok(()) => format!("Saved the answers as \"{name}\" — start a new run from it to replay them"),
                Err(e) => format!("Couldn't save: {e}"),
            };
        }
        if let Some(r) = promote_go {
            self.lab.promote = None;
            self.lab.msg = self.testlab_promote(&r);
        }
        if let Some(id) = delete {
            self.undoable("Deleted a test run", "delete", |d| {
                d.bury(&id);
                d.intakes.retain(|x| x.id != id);
            });
        }
    }

    /// Hand one run to the real data folder through a separate process (deliberate action).
    fn testlab_promote(&mut self, r: &Intake) -> String {
        let tmp = std::env::temp_dir().join(format!("resolve2-promote-{}.json", r.id));
        if let Err(e) = crate::store::write_json_atomic(&tmp, r) {
            return format!("Couldn't prepare the run: {e}");
        }
        let exe = std::env::current_exe().unwrap_or_default();
        let out = std::process::Command::new(exe).arg("--promote").arg(&tmp).output();
        let _ = std::fs::remove_file(&tmp);
        match out {
            Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
            Ok(o) => format!("Promotion failed: {}{}", String::from_utf8_lossy(&o.stdout).trim(), String::from_utf8_lossy(&o.stderr).trim()),
            Err(e) => format!("Promotion failed: {e}"),
        }
    }

    fn lab_compare(&mut self, ui: &mut egui::Ui, p: &Pal) {
        let runs: Vec<Intake> = self.lab.compare.iter().filter_map(|id| self.get_intake(id)).collect();
        if runs.len() != 2 {
            return;
        }
        card(ui, p, |ui| {
            ui.label(RichText::new("Compare proposals").font(fs(size::SECTION)));
            ui.columns(2, |cols| {
                for (c, r) in runs.iter().enumerate() {
                    let Some(pr) = &r.proposal else { continue };
                    cols[c].vertical(|ui| {
                    ui.label(RichText::new(&pr.name).font(fserif(size::TITLE)));
                    ui.label(RichText::new(format!("{} · {}", if r.provider.is_empty() { "app default".to_string() } else { r.provider.replace('|', " / ") }, if r.scenario.is_empty() { "own answers" } else { &r.scenario })).size(size::TINY).color(p.faint));
                    ui.label(RichText::new(&pr.summary).size(size::SMALL));
                    ui.add_space(4.0);
                    ui.label(RichText::new(format!("H0: {}", pr.h0.title)).size(size::SMALL));
                    for h in &pr.hypotheses {
                        ui.label(RichText::new(format!("• [{}] {} — NHx: {}", h.basis, h.title, h.negation)).size(size::SMALL).color(p.dim));
                    }
                    for ph in ["Now", "Next", "Later"] {
                        let items: Vec<String> = pr.roadmap.iter().filter(|x| x.phase == ph).map(|x| format!("[{}] {}", x.basis, x.title)).collect();
                        if !items.is_empty() {
                            ui.label(RichText::new(ph).size(size::SMALL).color(p.accent).strong());
                            for i in items {
                                ui.label(RichText::new(i).size(size::SMALL));
                            }
                        }
                    }
                    ui.label(RichText::new(format!("{} tasks · {} experiments · {} open uncertainties · {} questions asked", pr.tasks.len(), pr.experiments.len(), pr.uncertainties.len(), r.qas.len())).size(size::TINY).color(p.faint));
                    });
                }
            });
        });
    }
}
