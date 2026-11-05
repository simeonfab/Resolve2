//! Intake journey: proposal review, the bridge (Adopt, step by step), the roadmap reveal, and the
//! Test Intake inspector.

use super::theme::*;
use super::widgets::*;
use super::*;
use crate::resolve::intake::Intake;
use crate::resolve::journey::{self, Item};
use crate::resolve::Kind;

fn cta(p: &Pal, text: &str) -> egui::Button<'static> {
    egui::Button::new(RichText::new(text.to_string()).font(fs(size::BODY)).color(p.on_ink)).fill(p.ink).rounding(space::RADIUS_SMALL).min_size(egui::vec2(0.0, 34.0))
}

fn basis_chip(ui: &mut egui::Ui, p: &Pal, it: &mut Item) {
    let col = match it.basis.as_str() {
        "evidence" => p.green,
        "assumption" => p.accent,
        _ => p.orange,
    };
    let label = if it.basis.is_empty() { "uncertain" } else { it.basis.as_str() };
    if chip(ui, label, col).on_hover_text("Basis — click to change: evidence → assumption → uncertain").on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
        it.basis = match it.basis.as_str() {
            "evidence" => "assumption".into(),
            "assumption" => "uncertain".into(),
            _ => "evidence".into(),
        };
    }
}

/// Edit a list of items. Returns true if anything was removed/added.
fn item_list(ui: &mut egui::Ui, p: &Pal, salt: &str, items: &mut Vec<Item>, phases: bool, negation: bool, detail: bool, roadmap: &[String]) {
    let mut remove: Option<usize> = None;
    for (n, it) in items.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            basis_chip(ui, p, it);
            if phases {
                egui::ComboBox::from_id_salt((salt, "phase", n)).width(70.0).selected_text(if it.phase.is_empty() { "Now".to_string() } else { it.phase.clone() }).show_ui(ui, |ui| {
                    for ph in ["Now", "Next", "Later"] {
                        ui.selectable_value(&mut it.phase, ph.to_string(), ph);
                    }
                });
            }
            ui.add(egui::TextEdit::singleline(&mut it.title).desired_width(ui.available_width() - 30.0).font(fp(size::BODY)));
            if ui.small_button("×").on_hover_text("Remove").clicked() {
                remove = Some(n);
            }
        });
        if negation {
            ui.horizontal(|ui| {
                ui.add_space(24.0);
                ui.label(RichText::new("NHx").size(size::TINY).color(p.faint));
                ui.add(egui::TextEdit::singleline(&mut it.negation).hint_text("the explicit negation").desired_width(ui.available_width()).font(fp(size::SMALL)));
            });
        }
        if detail {
            ui.horizontal(|ui| {
                ui.add_space(24.0);
                ui.add(egui::TextEdit::multiline(&mut it.detail).hint_text("detail").desired_rows(1).desired_width(ui.available_width()).font(fp(size::SMALL)));
            });
        }
        if !roadmap.is_empty() {
            ui.horizontal(|ui| {
                ui.add_space(24.0);
                ui.label(RichText::new("part of").size(size::TINY).color(p.faint));
                let sel = it.parent.and_then(|x| roadmap.get(x)).cloned().unwrap_or_else(|| "—".into());
                egui::ComboBox::from_id_salt((salt, "parent", n)).width(220.0).selected_text(sel).show_ui(ui, |ui| {
                    ui.selectable_value(&mut it.parent, None, "—");
                    for (k, r) in roadmap.iter().enumerate() {
                        ui.selectable_value(&mut it.parent, Some(k), r);
                    }
                });
            });
        }
        ui.add_space(4.0);
    }
    if let Some(n) = remove {
        items.remove(n);
    }
    if ui.small_button("+ Add").clicked() {
        items.push(Item { basis: "uncertain".into(), phase: if phases { "Now".into() } else { String::new() }, ..Default::default() });
    }
}

impl App {
    // ---------------------------------------------------------------- 5. proposal review

    pub(super) fn ij_proposal(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut Intake) {
        let Some(mut pr) = it.proposal.clone() else {
            if it.last_error.is_empty() {
                self.ij_spawn_proposal_pub(it, None);
            } else if ui.add(cta(p, "Try again")).clicked() {
                it.last_error.clear();
                self.ij_spawn_proposal_pub(it, None);
            }
            return;
        };
        let ai = journey::cfg_for(it).is_some() && !pr.offline;
        ui.label(RichText::new("Here's your proposal.").font(fserif(size::HERO)));
        ui.label(RichText::new("Edit, remove or regenerate anything. Nothing is created until you press Adopt. Labels show what each item rests on.").size(size::SMALL).color(p.dim));
        if pr.offline {
            ui.label(RichText::new("Made without AI, from your own words only — add roadmap items and tasks yourself, or connect a model and regenerate.").size(size::SMALL).color(p.orange));
        }
        ui.add_space(12.0);
        let mut regen: Option<&'static str> = None;
        let mut toggle: Option<&'static str> = None;
        let section = |ui: &mut egui::Ui, key: &'static str, label: &str, removed: bool, regen: &mut Option<&'static str>, toggle: &mut Option<&'static str>, body: &mut dyn FnMut(&mut egui::Ui)| {
            egui::Frame::none().fill(p.card).stroke(Stroke::new(1.0, p.line)).rounding(space::RADIUS).inner_margin(space::CARD_PAD).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(RichText::new(label).font(fs(size::SECTION)).color(if removed { p.faint } else { p.text }));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.small_button(if removed { "Restore" } else { "Remove" }).clicked() {
                            *toggle = Some(key);
                        }
                        if ai && !removed && ui.small_button("↻ Regenerate").clicked() {
                            *regen = Some(key);
                        }
                    });
                });
                if removed {
                    ui.label(RichText::new("Removed — won't be created.").size(size::SMALL).color(p.faint));
                } else {
                    body(ui);
                }
            });
            ui.add_space(8.0);
        };
        let rm = |k: &str| pr.removed.iter().any(|x| x == k);
        let (r_sum, r_obj, r_h0, r_unc, r_exp, r_rm, r_tasks, r_pages) = (rm("summary"), rm("objective"), rm("h0"), rm("uncertainties"), rm("experiments"), rm("roadmap"), rm("tasks"), rm("pages"));
        let roadmap_titles: Vec<String> = pr.roadmap.iter().map(|r| r.title.clone()).collect();
        let hyp_titles: Vec<String> = pr.hypotheses.iter().map(|h| h.title.clone()).collect();
        // name + summary (cannot be removed: the project needs a name)
        egui::Frame::none().fill(p.card).stroke(Stroke::new(1.0, p.line)).rounding(space::RADIUS).inner_margin(space::CARD_PAD).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(RichText::new("Project name").size(size::SMALL).color(p.eyebrow).strong());
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ai && ui.small_button("↻ Regenerate").clicked() {
                        regen = Some("summary");
                    }
                });
            });
            ui.add(egui::TextEdit::singleline(&mut pr.name).font(fserif(size::TITLE)).desired_width(f32::INFINITY).hint_text("Short — name the real thing"));
            if pr.name.split_whitespace().count() > 6 {
                ui.label(RichText::new("Keep it short: a few words naming the real thing.").size(size::TINY).color(p.orange));
            }
            ui.label(RichText::new("One-line summary").size(size::SMALL).color(p.eyebrow).strong());
            ui.add(egui::TextEdit::singleline(&mut pr.summary).desired_width(f32::INFINITY));
        });
        ui.add_space(8.0);
        let _ = r_sum;
        section(ui, "objective", "Objective and outcome", r_obj, &mut regen, &mut toggle, &mut |ui| {
            ui.label(RichText::new("Objective").size(size::SMALL).color(p.dim));
            ui.add(egui::TextEdit::multiline(&mut pr.objective).desired_rows(1).desired_width(f32::INFINITY));
            ui.label(RichText::new("Outcome (your words, confirmed)").size(size::SMALL).color(p.dim));
            ui.add(egui::TextEdit::multiline(&mut pr.outcome).desired_rows(1).desired_width(f32::INFINITY));
        });
        section(ui, "h0", "H0 and the first hypotheses", r_h0, &mut regen, &mut toggle, &mut |ui| {
            ui.label(RichText::new("H0 — the starting objective, as a hypothesis to challenge").size(size::SMALL).color(p.dim));
            ui.horizontal(|ui| {
                basis_chip(ui, p, &mut pr.h0);
                ui.add(egui::TextEdit::singleline(&mut pr.h0.title).desired_width(ui.available_width()));
            });
            ui.horizontal(|ui| {
                ui.add_space(24.0);
                ui.label(RichText::new("NH0").size(size::TINY).color(p.faint));
                ui.add(egui::TextEdit::singleline(&mut pr.h0.negation).desired_width(ui.available_width()).font(fp(size::SMALL)));
            });
            ui.add_space(6.0);
            ui.label(RichText::new("Hx — first-level hypotheses, each with its negation").size(size::SMALL).color(p.dim));
            item_list(ui, p, "hyp", &mut pr.hypotheses, false, true, false, &[]);
        });
        section(ui, "uncertainties", "Open uncertainties", r_unc, &mut regen, &mut toggle, &mut |ui| {
            item_list(ui, p, "unc", &mut pr.uncertainties, false, false, false, &[]);
        });
        section(ui, "experiments", "Suggested validation (E0 first)", r_exp, &mut regen, &mut toggle, &mut |ui| {
            for (n, e) in pr.experiments.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("tests").size(size::TINY).color(p.faint));
                    let sel = e.parent.and_then(|x| hyp_titles.get(x)).cloned().unwrap_or_else(|| "H0".into());
                    egui::ComboBox::from_id_salt(("exp-parent", n)).width(180.0).selected_text(sel).show_ui(ui, |ui| {
                        ui.selectable_value(&mut e.parent, None, "H0");
                        for (k, h) in hyp_titles.iter().enumerate() {
                            ui.selectable_value(&mut e.parent, Some(k), h);
                        }
                    });
                    ui.add(egui::TextEdit::singleline(&mut e.title).desired_width(ui.available_width()));
                });
                for (k, label) in [("prediction", "Prediction (separates Hx from NHx)"), ("support_if", "Supports if"), ("refute_if", "Refutes if"), ("inconclusive_if", "Inconclusive if")] {
                    let mut v = e.fields.get(k).cloned().unwrap_or_default();
                    ui.horizontal(|ui| {
                        ui.add_space(24.0);
                        ui.label(RichText::new(label).size(size::TINY).color(p.faint));
                        if ui.add(egui::TextEdit::singleline(&mut v).desired_width(ui.available_width()).font(fp(size::SMALL))).changed() {
                            e.fields.insert(k.to_string(), v.clone());
                        }
                    });
                }
                ui.add_space(6.0);
            }
            if ui.small_button("+ Add experiment").clicked() {
                pr.experiments.push(Item { title: format!("E{}: ", pr.experiments.len()), basis: "uncertain".into(), ..Default::default() });
            }
        });
        section(ui, "roadmap", "First roadmap — Now, Next, Later", r_rm, &mut regen, &mut toggle, &mut |ui| {
            item_list(ui, p, "rm", &mut pr.roadmap, true, false, true, &[]);
        });
        section(ui, "tasks", "Tasks", r_tasks, &mut regen, &mut toggle, &mut |ui| {
            item_list(ui, p, "tasks", &mut pr.tasks, true, false, true, &roadmap_titles);
        });
        section(ui, "pages", "Knowledge pages (the project brief first)", r_pages, &mut regen, &mut toggle, &mut |ui| {
            for pg in pr.pages.iter_mut() {
                ui.add(egui::TextEdit::singleline(&mut pg.title).font(fs(size::BODY)).desired_width(f32::INFINITY));
                ui.add(egui::TextEdit::multiline(&mut pg.detail).desired_rows(3).desired_width(f32::INFINITY).font(fp(size::SMALL)));
                ui.add_space(6.0);
            }
        });
        if let Some(k) = toggle {
            if pr.removed.iter().any(|x| x == k) {
                pr.removed.retain(|x| x != k);
            } else {
                pr.removed.push(k.to_string());
            }
        }
        it.proposal = Some(pr.clone());
        ui.add_space(8.0);
        let ok = !pr.name.trim().is_empty();
        let (mut back, mut all, mut adopt) = (false, false, false);
        ui.horizontal(|ui| {
            back = ui.button("Back").clicked();
            if ai {
                all = ui.button("↻ Regenerate everything").clicked();
            }
            adopt = ui.add_enabled(ok, cta(p, "Adopt — create this project")).clicked();
        });
        ui.label(RichText::new("Adopt confirms the name and writes the project, roadmap, tasks, validation and pages. You can undo it.").size(size::TINY).color(p.faint));
        if back {
            it.stage = "confirm".into();
        } else if let Some(s) = regen {
            self.ij_spawn_proposal_pub(it, Some(s));
        } else if all {
            self.ij_spawn_proposal_pub(it, Some("all"));
        } else if adopt {
            let snapshot = it.clone();
            if let Some(p2) = it.proposal.as_mut() {
                journey::sanitise(p2, &snapshot);
            }
            it.stage = "bridge".into();
            self.put_intake(it);
            self.resolve_ui.adopt = Some((it.id.clone(), 0, journey::AdoptState::default(), crate::history::snapshot(&self.store.data)));
        }
    }

    pub(super) fn ij_spawn_proposal_pub(&mut self, it: &Intake, section: Option<&str>) {
        let snapshot = it.clone();
        let cfg = journey::cfg_for(it);
        let id = it.id.clone();
        let sec = section.map(|s| s.to_string());
        self.put_intake(it);
        self.spawn_job(if section.is_some() { "Regenerating…" } else { "Shaping your proposal… sequencing the work" }, move || {
            let (r, calls) = journey::run_proposal(cfg, &snapshot, sec.as_deref());
            super::resolve::JobOut::Proposed(id, sec, r, calls)
        });
    }

    // ---------------------------------------------------------------- 6. bridge (real progress)

    pub(super) fn ij_bridge(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut Intake) {
        if self.resolve_ui.adopt.as_ref().map_or(true, |a| a.0 != it.id) {
            // resumed after a restart mid-way: start the writes again
            self.resolve_ui.adopt = Some((it.id.clone(), 0, journey::AdoptState::default(), crate::history::snapshot(&self.store.data)));
        }
        let (iid, step, mut st, snap) = self.resolve_ui.adopt.take().unwrap();
        ui.label(RichText::new("Building your project…").font(fserif(size::HERO)));
        ui.add_space(12.0);
        for (k, label) in journey::ADOPT_STEPS.iter().enumerate() {
            ui.horizontal(|ui| {
                let (mark, col) = if k < step { ("✓", p.green) } else if k == step { ("◌", p.accent) } else { ("·", p.faint) };
                ui.label(RichText::new(mark).color(col).size(size::SECTION));
                ui.label(RichText::new(*label).size(size::BODY).color(if k <= step { p.text } else { p.faint }));
            });
        }
        ui.ctx().request_repaint();
        // one real step per frame
        match journey::adopt_step(&mut self.store.data, &iid, step, &mut st) {
            Ok(true) => {
                let name = it.proposal.as_ref().map(|x| x.name.clone()).unwrap_or_default();
                self.resolve_ui.adopt_cs = self.store.data.keep_change(&format!("Created project \"{name}\""), "adopt", &snap);
                self.save_now();
            }
            Ok(false) => self.resolve_ui.adopt = Some((iid, step + 1, st, snap)),
            Err(e) => {
                it.last_error = format!("Couldn't create the project: {e}");
                it.stage = "proposal".into();
                // roll back anything written so far
                let cs = self.store.data.keep_change("Failed adopt", "adopt", &snap);
                if let Some(cs) = cs {
                    let _ = self.store.data.undo_change(&cs);
                }
            }
        }
        self.touch();
    }

    // ---------------------------------------------------------------- 7. roadmap reveal

    pub(super) fn ij_reveal(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut Intake) {
        let Some(pid) = it.project_id.clone().filter(|x| self.store.data.project(x).is_some()) else {
            ui.label("The project is no longer there (it may have been undone).");
            if ui.button("Back to the proposal").clicked() {
                it.stage = "proposal".into();
                it.status = "in_progress".into();
                it.project_id = None;
            }
            return;
        };
        ui.label(RichText::new("Your roadmap").size(size::SMALL).color(p.eyebrow).strong());
        ui.label(RichText::new("Here's the whole picture.").font(fserif(size::HERO)));
        ui.add_space(14.0);
        let d = &self.store.data;
        let items: Vec<crate::resolve::Record> = d.records.iter().filter(|r| r.project_id.as_deref() == Some(&pid) && r.kind == Kind::Roadmap).cloned().collect();
        ui.columns(3, |cols| {
            for (c, phase) in ["Now", "Next", "Later"].iter().enumerate() {
                let ui = &mut cols[c];
                ui.label(RichText::new(*phase).font(fs(size::SECTION)).color(if c == 0 { p.accent } else { p.dim }));
                let mine: Vec<&crate::resolve::Record> = items.iter().filter(|r| r.field("phase") == *phase).collect();
                if mine.is_empty() {
                    egui::Frame::none().stroke(Stroke::new(1.0, p.line)).rounding(space::RADIUS).inner_margin(12.0).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.label(RichText::new("Nothing yet").size(size::SMALL).color(p.faint));
                    });
                }
                for r in mine {
                    egui::Frame::none().fill(p.card).stroke(Stroke::new(1.0, if c == 0 { p.accent } else { p.line })).rounding(space::RADIUS).inner_margin(12.0).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.label(RichText::new(&r.title).font(fs(size::BODY)));
                        for t in r.links.iter().filter_map(|l| d.task(l)) {
                            ui.label(RichText::new(format!("☐ {}", t.title)).size(size::SMALL).color(p.dim));
                        }
                        chip(ui, r.field("basis"), match r.field("basis") { "evidence" => p.green, "assumption" => p.accent, _ => p.orange });
                    });
                    ui.add_space(6.0);
                }
            }
        });
        ui.add_space(18.0);
        let mut open = false;
        ui.horizontal(|ui| {
            open = ui.add(cta(p, "Open my project  →")).clicked();
            if self.testlab {
                ui.label(RichText::new("Test mode: this project lives in the sandbox. Promote it from the test runs list.").size(size::TINY).color(p.faint));
            }
        });
        if open {
            if let Some(c) = self.store.data.context_for_mut(&pid) {
                c.reveal_seen = true;
                c.updated = now_ms();
            }
            it.stage = "done".into();
            self.put_intake(it);
            self.touch();
            self.resolve_ui.tab = 0;
            self.go(View::Resolve(pid));
            self.back.clear();
            if let Some(cs) = self.resolve_ui.adopt_cs.take() {
                self.undo_toasts.push(("Project created".into(), self.now + 8.0, cs));
            }
        }
    }

    // ---------------------------------------------------------------- inspector (Test Intake)

    pub(super) fn ij_inspector(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut Intake) {
        let Some(qa) = it.qas.get(it.cursor).cloned() else { return };
        let mut branch = false;
        let mut fill = false;
        card(ui, p, |ui| {
            ui.label(RichText::new(format!("Inspector — question {}", it.cursor + 1)).font(fs(size::SECTION)));
            ui.label(RichText::new(format!("Target: {} · source: {}", journey::target_label(&qa.target), qa.source)).size(size::TINY).color(p.faint));
            ui.label(RichText::new("H0 reasoning (why asked)").size(size::SMALL).color(p.eyebrow).strong());
            ui.label(RichText::new(if qa.why.is_empty() { "—" } else { &qa.why }).size(size::SMALL));
            ui.label(RichText::new(format!("Builds on: {}", qa.depends_on.join(", "))).size(size::TINY).color(p.faint));
            ui.label(RichText::new("Classification of this answer").size(size::SMALL).color(p.eyebrow).strong());
            for h in it.heard.iter().filter(|h| h.from == qa.id) {
                ui.label(RichText::new(format!("({}) {}", h.kind, h.text)).size(size::SMALL));
            }
            ui.label(RichText::new(format!("AI calls ({})", qa.calls.len())).size(size::SMALL).color(p.eyebrow).strong());
            for (n, c) in qa.calls.iter().enumerate() {
                egui::CollapsingHeader::new(RichText::new(format!("{} · {} / {} · {}", c.purpose, c.provider, c.model, if c.ok { "ok" } else { "error" })).size(size::TINY)).id_salt(("call", &qa.id, n)).show(ui, |ui| {
                    ui.label(RichText::new("Request").size(size::TINY).color(p.faint));
                    egui::ScrollArea::vertical().max_height(160.0).id_salt(("req", &qa.id, n)).show(ui, |ui| {
                        ui.label(RichText::new(&c.request).monospace().size(10.5));
                    });
                    ui.label(RichText::new("Response").size(size::TINY).color(p.faint));
                    egui::ScrollArea::vertical().max_height(160.0).id_salt(("resp", &qa.id, n)).show(ui, |ui| {
                        ui.label(RichText::new(&c.response).monospace().size(10.5));
                    });
                });
            }
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                branch = ui.button("Branch from here").on_hover_text("Copy this run up to this question into a new run, to try another path").clicked();
                if !it.scenario.is_empty() {
                    fill = ui.button("Answer from scenario").clicked();
                }
            });
        });
        if branch {
            self.testlab_branch(it, it.cursor);
        }
        if fill {
            self.testlab_fill(it);
        }
    }
}
