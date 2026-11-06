//! Intake journey: one function per stage. Layout and input only; logic is in resolve::journey.

use super::resolve::JobOut;
use super::theme::*;
use super::widgets::*;
use super::*;
use crate::resolve::intake::{self, Intake};
use crate::resolve::journey::{self, Item};

fn eyebrow(ui: &mut egui::Ui, p: &Pal, text: &str) {
    ui.label(RichText::new(text).size(size::SMALL).color(p.eyebrow).strong());
}

fn cta(p: &Pal, text: &str) -> egui::Button<'static> {
    egui::Button::new(RichText::new(text.to_string()).font(fs(size::BODY)).color(p.on_ink)).fill(p.ink).rounding(space::RADIUS_SMALL).min_size(egui::vec2(0.0, 34.0))
}

fn basis_color(p: &Pal, b: &str) -> Color32 {
    match b {
        "evidence" => p.green,
        "assumption" => p.accent,
        _ => p.orange,
    }
}

fn next_basis(b: &str) -> &'static str {
    match b {
        "evidence" => "assumption",
        "assumption" => "uncertain",
        _ => "evidence",
    }
}

/// Answers given so far (freshness key for the precomputed understanding).
pub(super) fn answered(i: &Intake) -> usize {
    i.qas.iter().filter(|q| !q.answer.trim().is_empty()).count() + usize::from(!i.input.trim().is_empty())
}

/// Freshness key for the precomputed proposal: the confirmed cards.
pub(super) fn und_key(i: &Intake) -> String {
    i.understanding.as_ref().map(|u| format!("{}|{}|{}|{}|{}", u.project, u.goal, u.state, u.uncertainty, u.outcome)).unwrap_or_default()
}

fn kind_color(p: &Pal, k: &str) -> Color32 {
    match k {
        "evidence" => p.green,
        "assumption" => p.accent,
        _ => p.orange,
    }
}

impl App {
    /// After an answer: sorting it ("heard") runs in the background and never blocks; choosing
    /// the next question is the only wait, run side by side with the sorting, cut off at 15 s.
    pub(super) fn ij_spawn_answer(&mut self, it: &Intake, qa: Option<String>, want_next: bool, label: &str) {
        self.put_intake(it);
        let cfg = journey::cfg_for(it);
        let id = it.id.clone();
        if cfg.is_none() {
            // fixed question set: instant, one job
            let snapshot = it.clone();
            self.spawn_job(label, move || {
                let out = journey::run_answer(None, &snapshot, qa.as_deref(), want_next);
                JobOut::Answer(id, out)
            });
            return;
        }
        if let Some(q) = qa.clone() {
            let (snapshot, c, i2) = (it.clone(), cfg.clone(), id.clone());
            self.spawn_bg("sorting the answer", move || {
                let out = journey::run_answer(c, &snapshot, Some(&q), false);
                JobOut::Answer(i2, out)
            });
        }
        if want_next {
            let snapshot = it.clone();
            self.spawn_job_kind(label, &id.clone(), "next", move || {
                let out = journey::run_answer(cfg, &snapshot, None, true);
                JobOut::Answer(id, out)
            });
        }
    }

    /// Understanding: in the foreground (the person is waiting) or in the background (precompute).
    pub(super) fn ij_spawn_understand(&mut self, it: &Intake, foreground: bool) {
        self.put_intake(it);
        let snapshot = it.clone();
        let cfg = journey::cfg_for(it);
        let id = it.id.clone();
        let n = answered(it);
        let job = move || {
            let (r, calls) = journey::run_understanding(cfg, &snapshot);
            JobOut::Understood(id, n, r, calls)
        };
        if foreground {
            self.spawn_job("Reading everything you said… separating what's decided from what's open", job);
        } else {
            self.resolve_ui.bg_understand = Some((it.id.clone(), n));
            self.spawn_bg("understanding", job);
        }
    }

    pub(super) fn ij_spawn_proposal(&mut self, it: &Intake, section: Option<&str>, foreground: bool) {
        self.put_intake(it);
        let snapshot = it.clone();
        let cfg = journey::cfg_for(it);
        let id = it.id.clone();
        let sec = section.map(|s| s.to_string());
        let key = und_key(it);
        let k2 = key.clone();
        let job = move || {
            let (r, calls) = journey::run_proposal(cfg, &snapshot, sec.as_deref());
            JobOut::Proposed(id, sec, k2, r, calls)
        };
        if foreground {
            self.spawn_job(if section.is_some() { "Regenerating that part…" } else { "Writing your plan and your validation side by side…" }, job);
        } else {
            self.resolve_ui.bg_propose = Some((it.id.clone(), key));
            self.spawn_bg("proposal", job);
        }
    }

    /// Precompute while the person reads or types, so the next screen is ready.
    pub(super) fn ij_precompute(&mut self, it: &Intake) {
        // screenshots never start AI work
        if self.shot.is_some() || journey::cfg_for(it).is_none() {
            return; // offline steps are instant
        }
        match it.stage.as_str() {
            "questions" => {
                let n = answered(it);
                let ready = it.suggest_stop || it.coverage() >= 0.6;
                let pending = self.resolve_ui.bg_understand.as_ref().map_or(false, |(x, k)| x == &it.id && *k == n);
                let fresh = it.understanding.is_some() && it.understood_n == n;
                if ready && n > 0 && !fresh && !pending && self.resolve_ui.bg_understand.is_none() {
                    self.ij_spawn_understand(it, false);
                }
                // once Resolve suggests stopping and the understanding is current, start the
                // proposal too (it is the longest wait); editing the cards later redoes it
                if it.suggest_stop && fresh {
                    let key = und_key(it);
                    let pending = self.resolve_ui.bg_propose.as_ref().map_or(false, |(x, k)| x == &it.id && *k == key);
                    if (it.proposal.is_none() || it.proposed_from != key) && !pending && self.resolve_ui.bg_propose.is_none() {
                        self.ij_spawn_proposal(it, None, false);
                    }
                }
            }
            "confirm" => {
                if it.editing || it.understanding.is_none() {
                    return;
                }
                let key = und_key(it);
                let pending = self.resolve_ui.bg_propose.as_ref().map_or(false, |(x, k)| x == &it.id && *k == key);
                if (it.proposal.is_none() || it.proposed_from != key) && !pending && self.resolve_ui.bg_propose.is_none() {
                    self.ij_spawn_proposal(it, None, false);
                }
            }
            _ => {}
        }
    }

    // ---------------------------------------------------------------- 1. name and welcome

    pub(super) fn ij_name(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut Intake) {
        eyebrow(ui, p, "Let's get started");
        ui.label(RichText::new("What's your name?").font(fserif(size::HERO)));
        ui.add_space(8.0);
        let r = ui.add(egui::TextEdit::singleline(&mut it.name).hint_text("Your first name").font(fp(size::ANSWER)).desired_width(320.0));
        if self.resolve_ui.focus_qa.as_deref() != Some("name") {
            r.request_focus();
            self.resolve_ui.focus_qa = Some("name".into());
        }
        let enter = r.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
        ui.add_space(10.0);
        let ok = !it.name.trim().is_empty();
        if (ui.add_enabled(ok, cta(p, "Continue  →")).clicked() || enter) && ok {
            it.name = it.name.trim().to_string();
            it.greeted = true;
            it.stage = "bring".into();
            self.resolve_ui.focus_qa = None;
        }
        ui.label(RichText::new("Press Enter to continue").size(size::TINY).color(p.faint));
    }

    // ---------------------------------------------------------------- 2. bring it in

    pub(super) fn ij_bring(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut Intake) {
        if it.greeted && it.qas.is_empty() {
            // greet once, here, then never again
            ui.label(RichText::new(format!("Hi {}. Welcome to Resolve.", it.name)).font(fserif(size::TITLE)).color(p.dim));
            ui.add_space(14.0);
        }
        eyebrow(ui, p, "In your own words");
        ui.label(RichText::new("Tell Resolve what's going on.").font(fserif(size::HERO)));
        ui.label(RichText::new("The idea or project, however it comes out. What exists, what's confusing, what you're trying to decide. Don't tidy it up.").size(size::SMALL).color(p.dim));
        ui.add_space(10.0);
        let aid = egui::Id::new(("bring", &it.id));
        let focused = ui.memory(|m| m.has_focus(aid));
        let enter = focused && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Enter));
        let before = it.input.clone();
        let r = ui.add(egui::TextEdit::multiline(&mut it.input).id(aid).desired_rows(9).desired_width(f32::INFINITY).font(fp(size::ANSWER)).hint_text("Type or paste here"));
        if self.resolve_ui.focus_qa.as_deref() != Some("bring") {
            r.request_focus();
            self.resolve_ui.focus_qa = Some("bring".into());
        }
        if it.input != before && !it.qas.is_empty() {
            it.flag_dependents("input");
        }
        ui.add_space(6.0);
        if ui.add(egui::Label::new(RichText::new(if self.resolve_ui.show_import { "▾ Already started this somewhere else? Get a prompt to pull it in" } else { "▸ Already started this somewhere else? Get a prompt to pull it in" }).size(size::SMALL).color(p.dim)).sense(Sense::click())).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
            self.resolve_ui.show_import = !self.resolve_ui.show_import;
        }
        if self.resolve_ui.show_import {
            egui::Frame::none().fill(p.callout).rounding(space::RADIUS_SMALL).inner_margin(12.0).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(RichText::new("Paste this into the ChatGPT or Claude conversation, then paste its answer into the box above:").size(size::SMALL).color(p.dim));
                ui.label(RichText::new(intake::IMPORT_PROMPT).size(size::SMALL).monospace());
                if ui.button("Copy prompt").clicked() {
                    ui.ctx().copy_text(intake::IMPORT_PROMPT.into());
                    self.toast("Prompt copied");
                }
            });
        }
        ui.add_space(12.0);
        let ok = !it.input.trim().is_empty();
        let mut go = false;
        ui.horizontal(|ui| {
            if ui.button("Back").clicked() {
                it.stage = "name".into();
                self.resolve_ui.focus_qa = None;
            }
            go = ui.add_enabled(ok, cta(p, "Continue  →")).clicked() || (enter && ok);
            ui.label(RichText::new("Enter to continue · Shift+Enter for a new line").size(size::TINY).color(p.faint));
        });
        if go {
            it.stage = "questions".into();
            self.resolve_ui.focus_qa = None;
            if it.qas.is_empty() {
                self.ij_spawn_answer(it, None, true, "Reading what you shared… choosing the first question");
            }
        }
    }

    // ---------------------------------------------------------------- 3. one question at a time

    pub(super) fn ij_question(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut Intake) {
        if it.qas.is_empty() {
            self.ij_spawn_answer(it, None, true, "Choosing the first question…");
            return;
        }
        let n = it.qas.len();
        it.cursor = it.cursor.min(n - 1);
        // question dots: jump to any question
        ui.horizontal_wrapped(|ui| {
            for k in 0..n {
                let q = &it.qas[k];
                let label = format!("{}{}", k + 1, if q.needs_review { " !" } else { "" });
                if pill(ui, p, &label, k == it.cursor).on_hover_text(&q.question).clicked() {
                    it.cursor = k;
                    self.resolve_ui.focus_qa = None;
                }
            }
            ui.label(RichText::new(format!("· covered {:.0}%", it.coverage() * 100.0)).size(size::TINY).color(p.faint));
        });
        ui.add_space(18.0);
        let cur = it.cursor;
        let (mut next, mut back, mut skip, mut propose, mut keep_going) = (false, false, false, false, false);
        {
            let qa = &mut it.qas[cur];
            eyebrow(ui, p, &format!("{} · question {}", journey::target_label(&qa.target), cur + 1));
            ui.label(RichText::new(&qa.question).font(fserif(size::QUESTION)));
            if qa.needs_review {
                ui.add_space(4.0);
                chip(ui, "An earlier answer changed — does this still hold?", p.orange);
            }
            ui.add_space(10.0);
            let aid = egui::Id::new(("answer", &qa.id));
            let focused = ui.memory(|m| m.has_focus(aid));
            let enter = focused && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Enter));
            let r = ui.add(egui::TextEdit::multiline(&mut qa.answer).id(aid).desired_rows(4).desired_width(f32::INFINITY).font(fp(size::ANSWER)).hint_text("Your answer"));
            if self.resolve_ui.focus_qa.as_deref() != Some(qa.id.as_str()) {
                r.request_focus();
                self.resolve_ui.focus_qa = Some(qa.id.clone());
            }
            next |= enter;
            ui.add_space(6.0);
            if !qa.why.is_empty() {
                let lbl = if self.resolve_ui.show_why { "▾ Why this question?" } else { "▸ Why this question?" };
                if ui.add(egui::Label::new(RichText::new(lbl).size(size::SMALL).color(p.dim)).sense(Sense::click())).clicked() {
                    self.resolve_ui.show_why = !self.resolve_ui.show_why;
                }
                if self.resolve_ui.show_why {
                    ui.label(RichText::new(&qa.why).size(size::SMALL).color(p.dim));
                }
            }
        }
        ui.add_space(14.0);
        ui.horizontal(|ui| {
            back = ui.button("Back").clicked();
            next |= ui.add(cta(p, "Next  →")).clicked();
            skip = ui.button("Skip").on_hover_text("Leave this one unanswered").clicked();
            ui.label(RichText::new("Enter · Shift+Enter new line · Esc saves and leaves").size(size::TINY).color(p.faint));
        });
        ui.add_space(10.0);
        propose = ui.add(egui::Button::new(RichText::new("Show me the proposal  →").size(size::SMALL)).frame(false)).clicked();
        if it.suggest_stop || it.coverage() >= 0.78 {
            ui.add_space(10.0);
            egui::Frame::none().fill(p.callout).rounding(space::RADIUS_SMALL).inner_margin(12.0).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(RichText::new("Resolve has heard enough for a grounded proposal. You can stop here, or keep going to sharpen it.").size(size::SMALL));
                ui.horizontal(|ui| {
                    propose |= ui.add(cta(p, "Show me the proposal")).clicked();
                    keep_going = ui.button("Keep going").clicked();
                });
            });
        }
        if skip {
            it.qas[cur].answer.clear();
            next = true;
        }
        if back {
            self.resolve_ui.focus_qa = None;
            if cur == 0 {
                it.stage = "bring".into();
            } else {
                it.cursor -= 1;
            }
        } else if keep_going {
            it.suggest_stop = false;
            let qa = it.qas[cur].clone();
            let changed = !qa.answer.trim().is_empty() && qa.answer.trim() != qa.classified.trim();
            self.ij_spawn_answer(it, if changed { Some(qa.id) } else { None }, true, "Choosing another question…");
        } else if propose {
            let qa = it.qas[cur].clone();
            let changed = !qa.answer.trim().is_empty() && qa.answer.trim() != qa.classified.trim();
            it.stage = "confirm".into();
            it.understanding = None;
            it.last_error.clear();
            if changed {
                self.ij_spawn_answer(it, Some(qa.id), false, "Noting that answer…");
            }
        } else if next {
            let qa = it.qas[cur].clone();
            let changed = qa.answer.trim() != qa.classified.trim();
            if changed {
                it.flag_dependents(&qa.id);
            }
            let classify = changed && !qa.answer.trim().is_empty();
            self.resolve_ui.focus_qa = None;
            if cur + 1 < it.qas.len() {
                it.cursor += 1;
                if classify {
                    self.ij_spawn_answer(it, Some(qa.id), false, "Noting that answer…");
                }
            } else if it.suggest_stop && !changed {
                it.stage = "confirm".into();
                it.understanding = None;
                it.last_error.clear();
            } else {
                self.ij_spawn_answer(it, if classify { Some(qa.id) } else { None }, true, "Thinking about what to ask next…");
            }
        }
    }

    /// "What Resolve has heard": evidence / assumptions / open questions, corrected in place.
    pub(super) fn ij_heard(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut Intake) {
        card(ui, p, |ui| {
            ui.label(RichText::new("What Resolve has heard").font(fs(size::SECTION)));
            ui.label(RichText::new("Click a label to change it; edit the words in place.").size(size::TINY).color(p.faint));
            if it.heard.is_empty() {
                ui.label(RichText::new("Nothing yet — it fills in as you answer.").size(size::SMALL).color(p.faint));
            }
            let mut remove: Option<usize> = None;
            for (kind, title) in [("evidence", "Evidence"), ("assumption", "Assumptions"), ("question", "Open questions")] {
                let idx: Vec<usize> = it.heard.iter().enumerate().filter(|(_, h)| h.kind == kind).map(|(i, _)| i).collect();
                if idx.is_empty() {
                    continue;
                }
                ui.add_space(6.0);
                ui.label(RichText::new(title).size(size::SMALL).color(kind_color(p, kind)).strong());
                for i in idx {
                    // label and remove on one line, the words underneath, wrapped to the panel
                    let h = &mut it.heard[i];
                    ui.add_space(2.0);
                    ui.horizontal(|ui| {
                        if chip(ui, &h.kind, kind_color(p, &h.kind)).on_hover_text("Change: evidence → assumption → question").on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                            h.kind = match h.kind.as_str() {
                                "evidence" => "assumption".into(),
                                "assumption" => "question".into(),
                                _ => "evidence".into(),
                            };
                            h.corrected = true;
                        }
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui.small_button("×").on_hover_text("Remove").clicked() {
                                remove = Some(i);
                            }
                        });
                    });
                    if ui.add(egui::TextEdit::multiline(&mut h.text).desired_rows(1).desired_width(f32::INFINITY).font(fp(size::SMALL))).changed() {
                        h.text = h.text.replace(['\n', '\r'], " ");
                        h.corrected = true;
                    }
                }
            }
            if let Some(i) = remove {
                it.heard.remove(i);
            }
        });
    }

    // ---------------------------------------------------------------- 4. confirm understanding

    pub(super) fn ij_confirm(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut Intake) {
        let n = answered(it);
        // (a screenshot shows what's there rather than starting AI work)
        let stale = it.understanding.is_none() || (it.understood_n != n && !it.editing && self.shot.is_none());
        if stale {
            // precomputed in the background? wait for it (with visible progress), else ask now
            let pending = self.resolve_ui.bg_understand.as_ref().map_or(false, |(x, k)| x == &it.id && *k == n);
            if pending {
                self.ij_waiting(ui, p, "Reading everything you said… separating what's decided from what's open", "understanding");
            } else {
                self.ij_spawn_understand(it, true);
            }
            return;
        }
        if ui.add(egui::Label::new(RichText::new("← Back to the questions").size(size::SMALL).color(p.faint)).sense(Sense::click())).clicked() {
            it.stage = "questions".into();
            it.editing = false;
            return;
        }
        ui.label(RichText::new("Here's what Resolve understood.").font(fserif(size::HERO)));
        ui.label(RichText::new(if it.editing { "Change anything that's wrong — the outcome in your own words." } else { "Check it — it shapes everything that follows." }).size(size::SMALL).color(p.dim));
        ui.add_space(12.0);
        let editing = it.editing;
        let mut u = it.understanding.clone().unwrap_or_default();
        // cards grow to fit their text; in edit mode they become editable
        let field = |ui: &mut egui::Ui, label: &str, v: &mut String| {
            egui::Frame::none().fill(p.card).stroke(Stroke::new(1.0, if editing { p.accent } else { p.line })).rounding(space::RADIUS).inner_margin(12.0).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.vertical(|ui| {
                    ui.label(RichText::new(label).size(size::SMALL).color(p.eyebrow).strong());
                    if editing {
                        let rows = (v.len() / 48 + 1).clamp(2, 8);
                        ui.add(egui::TextEdit::multiline(v).desired_rows(rows).desired_width(f32::INFINITY).font(fp(size::BODY)));
                    } else if v.trim().is_empty() {
                        wrap_label(ui, RichText::new("Not said yet — press \"Something's wrong\" to add it").size(size::BODY).color(p.faint));
                    } else {
                        wrap_label(ui, RichText::new(v.as_str()).size(size::BODY));
                    }
                });
            });
        };
        ui.columns(2, |c| {
            c[0].vertical(|ui| field(ui, "Project", &mut u.project));
            c[1].vertical(|ui| field(ui, "What you want", &mut u.goal));
        });
        ui.add_space(6.0);
        ui.columns(2, |c| {
            c[0].vertical(|ui| field(ui, "Where it is", &mut u.state));
            c[1].vertical(|ui| field(ui, "Biggest uncertainty", &mut u.uncertainty));
        });
        ui.add_space(6.0);
        let before_outcome = u.outcome.clone();
        field(ui, "The outcome — who is better off, and how you'd know", &mut u.outcome);
        if u.outcome != before_outcome {
            // editing the outcome is writing it in your own words: that confirms it
            it.outcome_confirmed = true;
        }
        it.understanding = Some(u.clone());
        ui.add_space(14.0);
        let ok = !u.outcome.trim().is_empty() && !u.project.trim().is_empty();
        let (mut go, mut change) = (false, false);
        ui.horizontal_wrapped(|ui| {
            go = ui.add_enabled(ok, cta(p, "This looks right, show me the proposal  →")).clicked();
            if !editing {
                change = ui.button("Something's wrong, let me change it").clicked();
            }
        });
        if !ok {
            ui.label(RichText::new("Add the project and the outcome to continue").size(size::TINY).color(p.faint));
        }
        if change {
            it.editing = true;
        } else if go {
            // pressing "This looks right" is the confirmation; edits stay as written
            for v in [&mut u.project, &mut u.goal, &mut u.state, &mut u.uncertainty, &mut u.outcome] {
                *v = journey::humanise(v);
            }
            it.understanding = Some(u);
            it.outcome_confirmed = true;
            it.editing = false;
            it.stage = "proposal".into();
        }
    }

    /// A wait screen with visible progress: what's happening and how long it has taken.
    pub(super) fn ij_waiting(&mut self, ui: &mut egui::Ui, p: &Pal, label: &str, bg_kind: &str) {
        let started = self.temp.entry(format!("wait:{bg_kind}")).or_insert_with(|| format!("{}", self.now)).parse::<f64>().unwrap_or(self.now);
        let secs = (self.now - started).max(0.0);
        card(ui, p, |ui| {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(RichText::new(label).font(fs(size::SECTION)));
            });
            ui.label(RichText::new(format!("{secs:.0} s — this started while you were answering, so it's nearly ready")).size(size::TINY).color(p.faint));
        });
        ui.ctx().request_repaint_after(std::time::Duration::from_millis(250));
    }
}
