//! The Careers intake's screens (Careers brief, "Careers intake"): 02b and 02c, 04 and 04b, 05,
//! 06 to 09 and the proposal editor. The frame, rail, states and motions are the PM intake's
//! (intake.rs); 01, 02a, 03, 10, 11 and 12 are PM's screens with Careers content where it differs.
//! Logic is in resolve::careers. Every colour and size is a token, every component in widgets.rs.

use super::intake_steps::{answered, checklist};
use super::resolve::JobOut;
use super::theme::*;
use super::widgets::*;
use super::*;
use crate::resolve::careers::{self, Career, Gate, Kit};
use crate::resolve::intake::Intake;
use crate::resolve::journey;

const REASSURE: &str = "Nothing here is final. You can change any of it once your job search is open.";

fn reassure(ui: &mut egui::Ui, p: &Pal) {
    ui.horizontal_top(|ui| {
        let (r, _) = ui.allocate_exact_size(egui::vec2(space::S16, space::S16), Sense::hover());
        paint_icon(ui.painter(), r.center(), Icon::Pencil, p.dim);
        ui.add(egui::Label::new(RichText::new(REASSURE).size(size::ROW_SMALL).color(p.dim)).wrap());
    });
}

fn car(it: &mut Intake) -> &mut Career {
    it.career.get_or_insert_with(Default::default)
}

impl App {
    // ---------------------------------------------------------------- jobs

    pub(super) fn ic_spawn_cards(&mut self, it: &Intake, foreground: bool) {
        if self.intake_ghosting {
            return;
        }
        self.put_intake(it);
        let (snapshot, cfg, id, n) = (it.clone(), journey::cfg_for(it), it.id.clone(), answered(it));
        journey::clear_progress(&id, &["reading", "understood", "cards"]);
        let job = move || {
            let (r, calls) = careers::run_cards(cfg, &snapshot);
            JobOut::CareerCards(id, n, r, calls)
        };
        if foreground {
            self.spawn_job("Reading your answers and your files…", job);
        } else {
            self.resolve_ui.bg_understand = Some((it.id.clone(), n));
            self.spawn_bg("understanding", job);
        }
    }

    pub(super) fn ic_spawn_kit(&mut self, it: &Intake, foreground: bool) {
        if self.intake_ghosting {
            return;
        }
        self.put_intake(it);
        let key = it.career.as_ref().map(|c| c.cards_key()).unwrap_or_default();
        let (snapshot, cfg, id, k2) = (it.clone(), journey::cfg_for(it), it.id.clone(), key.clone());
        journey::clear_progress(&id, &["first-part", "second-part", "merged", "proposed"]);
        let job = move || {
            let (r, calls) = careers::run_kit(cfg, &snapshot);
            JobOut::CareerKit(id, k2, r, calls)
        };
        if foreground {
            self.spawn_job("Building your kit…", job);
        } else {
            self.resolve_ui.bg_propose = Some((it.id.clone(), key));
            self.spawn_bg("proposal", job);
        }
    }

    /// Rebuild my CV: one call in the background (the editor shows its progress).
    pub(super) fn ic_spawn_cv(&mut self, it: &Intake) {
        let Some(cfg) = journey::cfg_for(it) else { return };
        if self.resolve_ui.bg.iter().any(|(l, _)| l == "rebuilding the CV") {
            return;
        }
        self.put_intake(it);
        let (snapshot, id) = (it.clone(), it.id.clone());
        self.temp.remove(&format!("cv_error:{id}"));
        self.spawn_bg("rebuilding the CV", move || {
            let _ = crate::ai::take_trace();
            let r = careers::rebuild_cv(&cfg, &snapshot);
            JobOut::CareerCv(id, r, crate::ai::take_trace())
        });
    }

    /// Get the next screen ready while the person reads or types (as PM's precompute).
    pub(super) fn ic_precompute(&mut self, it: &Intake) {
        if self.shot.is_some() || journey::cfg_for(it).is_none() || !it.limit_at.is_empty() {
            return;
        }
        let n = answered(it);
        let c = it.career.clone().unwrap_or_default();
        match it.stage.as_str() {
            "questions" => {
                let ready = it.suggest_stop || it.coverage() >= 0.6;
                let pending = self.resolve_ui.bg_understand.as_ref().map_or(false, |(x, k)| x == &it.id && *k == n);
                let fresh = c.cards.is_some() && c.cards_n == n;
                if ready && n > 0 && !fresh && !pending && self.resolve_ui.bg_understand.is_none() {
                    self.ic_spawn_cards(it, false);
                }
            }
            "confirm" => {
                if it.editing || c.cards.is_none() || c.cards.as_ref().map_or(false, |x| x.broad) && c.three_months.is_empty() {
                    return;
                }
                let key = c.cards_key();
                let pending = self.resolve_ui.bg_propose.as_ref().map_or(false, |(x, k)| x == &it.id && *k == key);
                if (c.kit.is_none() || c.kit_from != key) && !pending && self.resolve_ui.bg_propose.is_none() {
                    self.ic_spawn_kit(it, false);
                }
            }
            _ => {}
        }
    }

    // ---------------------------------------------------------------- 02b bring it in, 02c pull-in

    pub(super) fn ic_bring(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut Intake) {
        let pull = self.resolve_ui.show_import;
        step_head(ui, p, "In your own words", "Tell Resolve what job you're looking for.", if pull { "" } else { "However messy. Resolve reads your CVs too, so skip anything they already say." });
        ui.add_space(space::S14);
        let aid = egui::Id::new(("bring", &it.id));
        let focused = ui.memory(|m| m.has_focus(aid));
        let enter = focused && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Enter));
        let before = it.input.clone();
        let r = answer_box(ui, p, aid, &mut it.input, if pull { "Paste the other AI's answer here." } else { "Type or paste here" }, if pull { space::S40 * 2.4 } else { space::BRING_MIN_H }, pull);
        if self.resolve_ui.focus_qa.as_deref() != Some("bring") {
            r.request_focus();
            self.resolve_ui.focus_qa = Some("bring".into());
        }
        if it.input != before && !it.qas.is_empty() {
            it.flag_dependents("input");
        }
        let has_template = careers::TEMPLATE.iter().any(|h| it.input.contains(&format!("{h}:")));
        if pull {
            ui.add_space(space::S12);
            self.ic_pull_in(ui, p);
        } else {
            ui.add_space(space::S6);
            if has_template {
                ui.label(RichText::new("Leave any heading blank. Resolve asks about the gaps next.").size(size::ROW_SMALL).color(p.dim));
                ui.add_space(space::S6);
            }
            ui.add_space(space::S6);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = space::S10;
                if icon_chip_button(ui, p, Icon::Doc, "Use a template", has_template).clicked() && !has_template {
                    let mut s = it.input.trim_end().to_string();
                    if !s.is_empty() {
                        s.push('\n');
                    }
                    for h in careers::TEMPLATE {
                        s.push_str(&format!("{h}: \n"));
                    }
                    it.input = s;
                }
                if icon_chip_button(ui, p, Icon::Spark, "Try an example", false).on_hover_text("Fills the template with a sample job search you can edit").clicked() {
                    it.input = careers::EXAMPLE.to_string();
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if link(ui, p, "Bring it in from another AI chat →").clicked() {
                        self.resolve_ui.show_import = true;
                    }
                });
            });
            ui.add_space(space::S14);
            ui.label(RichText::new("Attach your CVs and LinkedIn").font(fs(size::ROW_SMALL)).color(p.text));
            ui.label(RichText::new(".pdf .docx .md .txt · LinkedIn as its PDF export").size(size::SMALL).color(p.dim));
            ui.add_space(space::S6);
            self.attach_row(ui, p, it);
        }
        let ok = !it.input.trim().is_empty() || !it.attachments.is_empty();
        self.pin_to_foot(ui, space::BUTTON_H_LARGE + space::S40);
        let (mut go, mut back) = (false, false);
        action_bar(ui, p, |ui| {
            back = big_button(ui, p, ButtonKind::Secondary, "Back", true).clicked();
            go = big_button(ui, p, ButtonKind::Dark, "Continue  →", ok).clicked() || (enter && ok);
            ui.label(RichText::new(if pull && !ok { "Paste the answer to continue" } else { "Next: up to 8 questions" }).size(size::ROW_SMALL).color(p.dim));
        });
        if back {
            it.stage = "welcome".into();
            self.temp.insert(format!("welcome_skip:{}", it.id), "1".into());
            self.resolve_ui.focus_qa = None;
        }
        if go {
            self.resolve_ui.show_import = false;
            // the targets the notes already answer are skipped (and counted on 03)
            for k in careers::covered_by_bring(&it.input) {
                if !it.ai_covered.contains(&k) {
                    it.ai_covered.push(k);
                }
            }
            it.stage = "questions".into();
            self.resolve_ui.focus_qa = None;
            if it.qas.is_empty() {
                self.ij_spawn_answer(it, None, true, "Thinking about what to ask next…");
            }
        }
    }

    /// 02c: the careers pull-in panel, in place under the box.
    fn ic_pull_in(&mut self, ui: &mut egui::Ui, p: &Pal) {
        let copied_at: Option<f64> = self.temp.get("pull_copied").and_then(|s| s.parse().ok());
        let copied = copied_at.map_or(false, |t| self.now - t < 1.5);
        let prompt = careers::pull_prompt();
        let mut close = false;
        card_lift(ui, p, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Bring in a job search from another AI chat").font(fs(size::SECTION)).color(p.text));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if icon_button(ui, Icon::Close, false, false).on_hover_text("Close").clicked() {
                        close = true;
                    }
                });
            });
            ui.add_space(space::S6);
            ui.add(egui::Label::new(RichText::new("1. Copy this prompt. 2. Paste it into the chat where you ran your search. 3. Paste the answer back here.").size(size::ROW_SMALL).color(p.dim)).wrap());
            ui.add_space(space::S10);
            egui::Frame::none().fill(p.surface_raised).stroke(Stroke::new(stroke::HAIR, p.line)).rounding(egui::Rounding::same(radius::FIELD)).inner_margin(egui::Margin::symmetric(space::S14, space::S12)).show(ui, |ui| {
                ui.set_width(ui.available_width());
                egui::ScrollArea::vertical().max_height(space::S40 * 3.6).id_salt("pull-prompt-careers").show(ui, |ui| {
                    ui.add(egui::Label::new(RichText::new(&prompt).size(size::ROW_SMALL).color(p.text)).wrap());
                });
            });
            ui.add_space(space::S10);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = space::S12;
                if button(ui, p, ButtonKind::Dark, if copied { "Copied ✓" } else { "Copy prompt" }, true, true).clicked() {
                    ui.ctx().copy_text(prompt.clone());
                    self.temp.insert("pull_copied".into(), self.now.to_string());
                }
                ui.label(RichText::new("Then paste the answer into the box above").size(size::SMALL).color(p.dim));
            });
        });
        if copied {
            ui.ctx().request_repaint_after(std::time::Duration::from_millis(200));
        }
        if close {
            self.resolve_ui.show_import = false;
        }
    }

    // ---------------------------------------------------------------- 04 confirm, 04b edit

    pub(super) fn ic_confirm(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut Intake) {
        let n = answered(it);
        let busy_cards = self.resolve_ui.rx.is_some() && self.resolve_ui.busy.starts_with("Reading your answers");
        let c0 = it.career.clone().unwrap_or_default();
        if !self.intake_ghosting && (it.limit_at == "understanding" || busy_cards || c0.cards.is_none() || (c0.cards_n != n && !it.editing && self.shot.is_none())) {
            self.temp.remove(&format!("cards_t:{}", it.id));
        }
        if it.limit_at == "understanding" {
            self.ij_limit(ui, p, it);
            return;
        }
        if busy_cards {
            self.ic_wait_cards(ui, p, it, self.wait_secs());
            return;
        }
        if c0.cards.is_none() && !it.last_error.is_empty() {
            self.ij_error(ui, p, it, "Confirm understanding");
            return;
        }
        let stale = c0.cards.is_none() || (c0.cards_n != n && !it.editing && self.shot.is_none());
        if stale {
            let pending = self.resolve_ui.bg_understand.as_ref().map_or(false, |(x, k)| x == &it.id && *k == n);
            if pending {
                let started = self.temp.entry("wait:understanding".to_string()).or_insert_with(|| self.now.to_string()).parse::<f64>().unwrap_or(self.now);
                self.ic_wait_cards(ui, p, it, (self.now - started).max(0.0));
            } else {
                self.ic_spawn_cards(it, true);
            }
            return;
        }
        let mut c = c0.cards.clone().unwrap_or_default();
        // 05: the target still covers a lot of different jobs
        if !it.editing && c.broad && c0.three_months.trim().is_empty() {
            it.stage = "outcome".into();
            return;
        }
        if link(ui, p, "← Back to the questions").clicked() {
            it.stage = "questions".into();
            it.editing = false;
            return;
        }
        let editing = it.editing;
        step_head(ui, p, "", if editing { "Change anything that's wrong." } else { "Here's what Resolve understood." }, "");
        ui.add_space(space::S14);
        let ck = format!("cards_t:{}", it.id);
        if !self.temp.contains_key(&ck) && !self.intake_ghosting {
            self.temp.insert(ck.clone(), self.now.to_string());
        }
        let reduced = super::motion::reduced();
        let e_cards = self.intake_clock(&ck);
        let fc = |n: usize| e_cards.map(|e| super::motion::frame_at(e, super::motion::timing(460.0, 120.0, super::motion::E_SPRING).stagger(n, 70.0), reduced)).unwrap_or(super::motion::SETTLED);
        let f_bar = e_cards.map(|e| super::motion::frame_at(e, super::motion::timing(480.0, 460.0, super::motion::E_SPRING), reduced)).unwrap_or(super::motion::SETTLED);
        if !f_bar.done {
            ui.ctx().request_repaint();
        }
        let card = |ui: &mut egui::Ui, label: &str, v: &mut String, key: &str, min_h: f32| -> f32 {
            let fid = egui::Id::new(("careers-confirm", key));
            let inner = egui::Frame::none().fill(p.card).stroke(Stroke::new(stroke::HAIR, if editing && ui.memory(|m| m.has_focus(fid)) { tok::AMBER } else { p.line })).rounding(egui::Rounding::same(radius::CARD)).inner_margin(egui::Margin::same(space::S16)).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.set_min_height(min_h);
                ui.label(RichText::new(label).font(fs(size::SMALL)).color(p.orange));
                ui.add_space(space::S6);
                if editing {
                    answer_box(ui, p, fid, v, "", space::S40 * 2.2, false);
                } else if v.trim().is_empty() {
                    ui.add(egui::Label::new(RichText::new("Not said yet. Press Edit to add it.").size(size::ROW + 0.5).color(p.dim)).wrap());
                } else {
                    ui.add(egui::Label::new(RichText::new(v.as_str()).size(size::ROW + 0.5).color(p.text)).wrap());
                }
            });
            inner.response.rect.height()
        };
        let pair = |ui: &mut egui::Ui, row: &str, base: usize, a: (&str, &mut String), b: (&str, &mut String)| {
            let key = egui::Id::new(("careers-confirm-row", row, editing));
            let h: f32 = ui.data(|d| d.get_temp(key)).unwrap_or(0.0);
            let w = (ui.available_width() - space::S10) / 2.0;
            let mut hs = (0.0, 0.0);
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = space::S10;
                ui.vertical(|ui| {
                    ui.set_width(w);
                    hs.0 = super::motion::fade_in(ui, fc(base), egui::vec2(0.0, space::S18), |ui| card(ui, a.0, a.1, &format!("{row}-a"), (h - space::S16 * 2.0).max(0.0)));
                });
                ui.vertical(|ui| {
                    ui.set_width(w);
                    hs.1 = super::motion::fade_in(ui, fc(base + 1), egui::vec2(0.0, space::S18), |ui| card(ui, b.0, b.1, &format!("{row}-b"), (h - space::S16 * 2.0).max(0.0)));
                });
            });
            let natural = hs.0.max(hs.1);
            if (natural - h).abs() > 0.5 {
                ui.data_mut(|d| d.insert_temp(key, natural));
                ui.ctx().request_repaint();
            }
        };
        pair(ui, "1", 0, ("The job you want", &mut c.job), ("Your hard gates", &mut c.hard_gates));
        ui.add_space(space::S10);
        pair(ui, "2", 2, ("Your preferences", &mut c.preferences), ("What a win looks like", &mut c.win));
        ui.add_space(space::S10);
        // the full-width CV card: Strengths, Gaps, What to fix, and Rebuild my CV
        let mut rebuild = false;
        let no_files = it.attachments.is_empty();
        super::motion::fade_in(ui, fc(4), egui::vec2(0.0, space::S18), |ui| {
            egui::Frame::none().fill(p.card).stroke(Stroke::new(stroke::HAIR, p.line)).rounding(egui::Rounding::same(radius::CARD)).inner_margin(egui::Margin::same(space::S16)).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Your CV and LinkedIn").font(fs(size::SMALL)).color(p.orange));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        rebuild = button(ui, p, ButtonKind::Secondary, "Rebuild my CV  →", false, true).on_hover_text("Opens your CV in the full proposal. The rebuilt CV saves into My CV").clicked();
                    });
                });
                ui.add_space(space::S8);
                let w = (ui.available_width() - space::S16 * 2.0) / 3.0;
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = space::S16;
                    for (label, col, v, key) in [("Strengths", p.green_text, &mut c.strengths, "st"), ("Gaps", p.accent_text, &mut c.gaps, "gaps"), ("What to fix", p.orange, &mut c.fix, "fix")] {
                        ui.vertical(|ui| {
                            ui.set_width(w);
                            ui.label(RichText::new(label).font(fs(size::ROW_SMALL)).color(col));
                            if editing {
                                answer_box(ui, p, egui::Id::new(("careers-cv", key)), v, "", space::S40 * 1.8, false);
                            } else {
                                let empty = v.trim().is_empty();
                                let shown = if !empty { v.as_str() } else if no_files { "Attach your CV on the first screen to see this." } else { "Nothing yet." };
                                ui.add(egui::Label::new(RichText::new(shown).size(size::ROW_SMALL + 0.5).color(if empty { p.dim } else { p.text })).wrap());
                            }
                        });
                    }
                });
            });
        });
        car(it).cards = Some(c.clone());
        let ok = !c.job.trim().is_empty();
        self.pin_to_foot(ui, space::BUTTON_H_LARGE + space::S40);
        let (mut go, mut change, mut cancel) = (false, false, false);
        super::motion::fade_in(ui, f_bar, egui::vec2(0.0, space::S30), |ui| {
            action_bar(ui, p, |ui| {
                if editing {
                    go = big_button(ui, p, ButtonKind::Dark, "Save changes", ok).clicked();
                    cancel = big_button(ui, p, ButtonKind::Secondary, "Cancel", true).clicked();
                } else {
                    go = big_button(ui, p, ButtonKind::Dark, "That's right  →", ok).clicked();
                    change = big_button(ui, p, ButtonKind::Secondary, "Edit", true).clicked();
                }
            })
        });
        if !ok {
            ui.label(RichText::new("Add the job you want to continue").size(size::SMALL).color(p.dim));
        }
        let snap_key = format!("careers_confirm_before:{}", it.id);
        if change {
            self.temp.insert(snap_key, serde_json::to_string(&c).unwrap_or_default());
            it.editing = true;
        } else if cancel {
            if let Some(old) = self.temp.remove(&snap_key).and_then(|s| serde_json::from_str(&s).ok()) {
                car(it).cards = Some(old);
            }
            it.editing = false;
        } else if go && editing {
            // Save changes keeps you on the cards, now as written
            it.editing = false;
            self.temp.remove(&snap_key);
        } else if go || (rebuild && ok) {
            for v in [&mut c.job, &mut c.hard_gates, &mut c.preferences, &mut c.win, &mut c.strengths, &mut c.gaps, &mut c.fix] {
                *v = journey::humanise(v);
            }
            c.broad = false;
            car(it).cards = Some(c);
            it.outcome_confirmed = true;
            it.editing = false;
            self.temp.remove(&snap_key);
            it.stage = "proposal".into();
            if rebuild {
                // Rebuild my CV opens the CV part of the full proposal
                car(it).page = "CV templates".into();
                it.editor = true;
                if car(it).cv.trim().is_empty() && !it.attachments.is_empty() {
                    self.ic_spawn_cv(it);
                }
            }
        }
    }

    fn ic_wait_cards(&mut self, ui: &mut egui::Ui, p: &Pal, it: &Intake, secs: f64) {
        ui.label(RichText::new("Confirm understanding").font(fs(size::LABEL)).color(p.orange));
        ui.add_space(space::S10);
        let id = it.id.clone();
        let lines = checklist(&id, &careers::WAIT_CARDS, &|ev| journey::has_progress(&id, ev), false);
        wait_card(ui, p, "Reading your answers and your files…", "Sorting gates from preferences.", secs, &lines, None);
        ui.ctx().request_repaint_after(std::time::Duration::from_millis(250));
    }

    // ---------------------------------------------------------------- 05 no clear target

    pub(super) fn ic_outcome(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut Intake) {
        let fail_key = format!("outcome_fail:{}", it.id);
        let mut fail: Option<String> = self.temp.get(&fail_key).cloned();
        let mut go = false;
        ui.add_space(space::S24);
        let w = (ui.available_width() - space::INTAKE_GAP) / 2.0;
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                ui.set_width(w);
                ui.label(RichText::new("Before we build this").font(fs(size::LABEL)).color(p.orange));
                ui.add(egui::Label::new(RichText::new("We're not going to guess.").font(fserif(size::DISPLAY)).color(p.text)).wrap());
                ui.add_space(space::S14);
                ui.add(egui::Label::new(RichText::new("Your target still covers a lot of different jobs. One answer sets the rest.").size(size::ROW + 1.5).color(p.text)).wrap());
            });
            ui.add_space(space::INTAKE_GAP);
            ui.vertical(|ui| {
                ui.set_width(w);
                card_lift(ui, p, |ui| {
                    ui.add_space(space::S6);
                    ui.label(RichText::new("ONE QUESTION").font(fs(size::TAG + 0.5)).color(p.orange));
                    ui.add(egui::Label::new(RichText::new(careers::THREE_MONTHS).font(fserif(size::CARD_SERIF - 2.0)).color(p.text)).wrap());
                    ui.add_space(space::S4);
                });
                ui.add_space(space::S14);
                answer_box(ui, p, egui::Id::new(("careers-three", &it.id)), &mut it.outcome_reply, "Answer in a sentence or two.", space::ANSWER_MIN_H, false);
                ui.add_space(space::S14);
                let ok = !it.outcome_reply.trim().is_empty();
                let w2 = ui.available_width();
                ui.allocate_ui_with_layout(egui::vec2(w2, space::BUTTON_H_LARGE), Layout::top_down_justified(Align::Center), |ui| {
                    if wide_button(ui, p, ButtonKind::Dark, "Continue  →", ok).clicked() {
                        // one job, named: a few words at least, and not several at once
                        let a = it.outcome_reply.trim();
                        if a.split_whitespace().count() >= 3 && !careers::is_broad(a) {
                            go = true;
                            fail = None;
                        } else {
                            let msg = "That still covers several jobs. Name the one you'd most like to have.".to_string();
                            self.temp.insert(fail_key.clone(), msg.clone());
                            fail = Some(msg);
                        }
                    }
                });
                if let Some(f) = &fail {
                    ui.add_space(space::S10);
                    ui.add(egui::Label::new(RichText::new(f).size(size::ROW_SMALL).color(p.orange)).wrap());
                }
            });
        });
        if go {
            let answer = journey::humanise(it.outcome_reply.trim());
            car(it).three_months = answer.clone();
            if let Some(c) = car(it).cards.as_mut() {
                c.broad = false;
            }
            // kept with the other answers: the cards are written again with it
            it.qas.push(journey::Qa { id: crate::model::new_id(), target: "role".into(), question: careers::THREE_MONTHS.into(), answer: answer.clone(), classified: answer, source: "no clear target".into(), ..Default::default() });
            it.outcome_reply.clear();
            self.temp.remove(&fail_key);
            it.stage = "confirm".into();
        }
    }

    // ---------------------------------------------------------------- 06 to 09, editor

    pub(super) fn ic_proposal(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut Intake) {
        if it.limit_at == "proposal" {
            self.ij_limit(ui, p, it);
            return;
        }
        let c = it.career.clone().unwrap_or_default();
        let key = c.cards_key();
        let fresh = c.kit.is_some() && (c.kit_from == key || c.kit_from.is_empty());
        if !fresh {
            let pending = self.resolve_ui.bg_propose.as_ref().map_or(false, |(x, k)| x == &it.id && *k == key);
            if self.resolve_ui.rx.is_some() {
                let secs = self.wait_secs();
                self.ic_wait_kit(ui, p, it, secs);
            } else if pending {
                let started = self.temp.entry("wait:proposal".to_string()).or_insert_with(|| self.now.to_string()).parse::<f64>().unwrap_or(self.now);
                self.ic_wait_kit(ui, p, it, (self.now - started).max(0.0));
            } else if !it.last_error.is_empty() && c.kit.is_none() {
                self.ij_error(ui, p, it, "Your kit");
            } else if self.shot.is_none() || c.kit.is_none() {
                self.ic_spawn_kit(it, true);
            }
            if !(self.shot.is_some() && c.kit.is_some()) {
                return;
            }
        }
        if it.editor {
            self.ic_editor(ui, p, it);
            return;
        }
        match it.ending {
            0 => self.ic_next_move(ui, p, it),
            1 => self.ic_gates(ui, p, it),
            2 => self.ic_week(ui, p, it),
            _ => self.ic_kit(ui, p, it),
        }
    }

    fn ic_wait_kit(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut Intake, secs: f64) {
        let id = it.id.clone();
        let lines = checklist(&id, &careers::WAIT_KIT, &|ev| journey::has_progress(&id, ev), false);
        let (mut back, mut leave) = (false, false);
        ui.add_space(space::S40 * 2.5);
        ui.vertical_centered(|ui| {
            let d = space::S40 + space::S26;
            let (r, _) = ui.allocate_exact_size(egui::vec2(d, d), Sense::hover());
            spinner_ring(ui, r.center(), d / 2.0 - 2.0);
            ui.add_space(space::S22);
            ui.label(RichText::new("Building your kit…").font(fserif(size::STEP_TITLE + 4.0)).color(p.text));
            ui.label(RichText::new("Writing your gates and your CV kit side by side.").size(size::ROW + 0.5).color(p.dim));
            ui.label(RichText::new(format!("{secs:.0} s")).font(fs(size::ROW_SMALL)).color(p.text));
            ui.add_space(space::S14);
        });
        let w = space::S40 * 8.0;
        let left = ((ui.available_width() - w) / 2.0).max(0.0);
        ui.horizontal(|ui| {
            ui.add_space(left);
            ui.vertical(|ui| {
                ui.set_width(w);
                for (text, state) in &lines {
                    check_line(ui, p, text, *state);
                    ui.add_space(space::S6);
                }
            });
        });
        if secs > space::WAIT_CUTOFF_S as f64 {
            ui.add_space(space::S14);
            let w2 = space::S40 * 11.5;
            let left2 = ((ui.available_width() - w2) / 2.0).max(0.0);
            ui.horizontal(|ui| {
                ui.add_space(left2);
                ui.vertical(|ui| {
                    ui.set_width(w2);
                    egui::Frame::none().fill(p.card).stroke(Stroke::new(stroke::HAIR, p.line)).rounding(egui::Rounding::same(radius::CARD)).inner_margin(egui::Margin::symmetric(space::S16, space::S12)).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.add(egui::Label::new(RichText::new("This step takes about a minute. Your answers are saved, so you can leave and come back to this exact point.").size(size::ROW_SMALL).color(p.text)).wrap());
                    });
                    ui.add_space(space::S10);
                    ui.horizontal(|ui| {
                        ui.add_space(space::S40 * 1.7);
                        ui.spacing_mut().item_spacing.x = space::S16;
                        back = link(ui, p, "← Back to the cards").clicked();
                        leave = link(ui, p, "Leave and come back later").clicked();
                    });
                });
            });
        }
        ui.ctx().request_repaint_after(std::time::Duration::from_millis(250));
        if back {
            it.stage = "confirm".into();
        }
        if leave {
            self.save_now();
            self.go(View::Dashboard);
        }
    }

    // ---- 06 next move
    fn ic_next_move(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut Intake) {
        let editing = self.resolve_ui.theme_edit;
        let Some(k) = car(it).kit.as_mut() else { return };
        ui.label(RichText::new("Your next move").font(fs(size::LABEL)).color(p.orange));
        ui.label(RichText::new("Here's what to do next.").font(fs(size::TITLE + 4.0)).color(p.text));
        ui.add_space(space::S14);
        hero_card(ui, p, |ui| {
            ui.label(RichText::new("Do this first").font(fs(size::SMALL)).color(tok::AMBER));
            ui.add_space(space::S4);
            if editing || k.do_first.trim().is_empty() {
                ui.add(egui::TextEdit::singleline(&mut k.do_first).font(fserif(size::CARD_SERIF)).hint_text("Your first move: one thing you can do this week").desired_width(f32::INFINITY));
                ui.add(egui::TextEdit::multiline(&mut k.why).hint_text("Why it comes first").desired_rows(2).desired_width(f32::INFINITY));
            } else {
                ui.add(egui::Label::new(RichText::new(&k.do_first).font(fserif(size::CARD_SERIF + 2.0)).color(tok::TEXT_ON_DARK)).wrap());
                if !k.why.trim().is_empty() {
                    ui.add_space(space::S8);
                    ui.add(egui::Label::new(RichText::new(format!("Why: {}", k.why)).size(size::ROW + 0.5).color(tok::TEXT_ON_DARK_SOFT)).wrap());
                }
            }
        });
        ui.add_space(space::S22);
        let w = (ui.available_width() - space::S10 * 2.0) / 3.0;
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = space::S10;
            for (phase, title, line) in careers::strip(k) {
                let now = phase == "Now";
                egui::Frame::none().fill(p.card).stroke(Stroke::new(stroke::HAIR, if now { tok::AMBER } else { p.line })).rounding(egui::Rounding::same(radius::CARD)).inner_margin(egui::Margin::symmetric(space::S16, space::S14)).show(ui, |ui| {
                    ui.set_width(w - space::S16 * 2.0);
                    ui.set_min_height(space::S40 + space::S22);
                    ui.vertical(|ui| {
                        ui.label(RichText::new(phase).font(fs(size::SMALL)).color(if now { p.orange } else { p.dim }));
                        ui.add(egui::Label::new(RichText::new(title).font(fs(size::ROW + 0.5)).color(p.text)).wrap());
                        ui.add(egui::Label::new(RichText::new(line).size(size::ROW_SMALL).color(p.dim)).wrap());
                    });
                });
            }
        });
        ui.add_space(space::S14);
        reassure(ui, p);
        let ok = !k.do_first.trim().is_empty();
        self.pin_to_foot(ui, space::BUTTON_H_LARGE + space::S40);
        let (mut back, mut go, mut change, mut skip) = (false, false, false, false);
        action_bar(ui, p, |ui| {
            back = big_button(ui, p, ButtonKind::Secondary, "Back", true).clicked();
            go = big_button(ui, p, ButtonKind::Dark, "Looks right  →", ok).clicked();
            if !editing {
                change = big_button(ui, p, ButtonKind::Secondary, "Change it", true).clicked();
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                skip = link(ui, p, "Skip to your kit →").clicked();
            });
        });
        if change {
            self.resolve_ui.theme_edit = true;
        } else if back {
            self.resolve_ui.theme_edit = false;
            it.stage = "confirm".into();
        } else if go {
            self.resolve_ui.theme_edit = false;
            it.ending = 1;
            it.theme_ix = 0;
        } else if skip {
            self.resolve_ui.theme_edit = false;
            it.ending = 3;
        }
    }

    // ---- 07 gates, one card at a time
    fn ic_gates(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut Intake) {
        let editing = self.resolve_ui.theme_edit;
        let ix = it.theme_ix;
        let Some(k) = car(it).kit.as_mut() else { return };
        let n = k.gates.len();
        ui.label(RichText::new("Your gates").font(fs(size::LABEL)).color(p.orange));
        ui.add(egui::Label::new(RichText::new("Is this one of your gates?").font(fserif(size::STEP_TITLE + 2.0)).color(p.text)).wrap());
        ui.add(egui::Label::new(RichText::new("Reviewer checks every job against these. A hard gate fails a job outright.").size(size::ROW + 0.5).color(p.dim)).wrap());
        ui.add_space(space::S8);
        let (mut back, mut skip) = (false, false);
        if n == 0 {
            ui.add_space(space::S10);
            ui.label(RichText::new("There are no gates yet. Add one, or carry on and set them later in Frameworks.").size(size::ROW_SMALL).color(p.dim));
            ui.add_space(space::S10);
            let (mut add, mut on) = (false, false);
            ui.horizontal(|ui| {
                add = big_button(ui, p, ButtonKind::Secondary, "+ Add a gate", true).clicked();
                on = big_button(ui, p, ButtonKind::Dark, "Continue  →", true).clicked();
            });
            if add {
                k.gates.push(Gate { hard: true, ..Default::default() });
                self.resolve_ui.theme_edit = true;
            }
            if on {
                it.ending = 2;
            }
            return;
        }
        if ix >= n {
            it.ending = 2;
            return;
        }
        let card_w = (ui.available_width() * 0.62).min(space::S40 * 13.0);
        segment_bar(ui, p, n, ix, card_w);
        ui.add_space(space::S18);
        let (mut yes, mut not_quite, mut flip, mut save, mut drop) = (false, false, false, false, false);
        let kept: Vec<(String, bool)> = k.gates.iter().take(ix).filter(|g| g.check == "kept").map(|g| (g.rule.clone(), g.hard)).collect();
        let checked = k.gates.iter().filter(|g| !g.check.is_empty()).count();
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                ui.set_width(card_w);
                let behind_at = ui.painter().add(egui::Shape::Noop);
                let g = &mut k.gates[ix];
                let inner = egui::Frame::none().fill(p.card).stroke(Stroke::new(stroke::HAIR, if editing { tok::AMBER } else { p.line })).rounding(egui::Rounding::same(radius::DIALOG)).inner_margin(egui::Margin::same(space::S24)).show(ui, |ui| {
                    ui.set_width(card_w - space::S24 * 2.0);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = space::S8;
                        gate_tag(ui, p, g.hard, false);
                        ui.label(RichText::new(format!("{} of {n}", ix + 1)).size(size::ROW_SMALL).color(p.dim));
                    });
                    ui.add_space(space::S8);
                    if editing {
                        ui.add(egui::TextEdit::singleline(&mut g.rule).font(fs(size::SECTION)).desired_width(f32::INFINITY).hint_text("The rule"));
                        ui.horizontal(|ui| {
                            if tab_chip(ui, p, "Hard gate", None, g.hard).clicked() {
                                g.hard = true;
                            }
                            if tab_chip(ui, p, "Preference", None, !g.hard).clicked() {
                                g.hard = false;
                            }
                        });
                        ui.label(RichText::new("A hard gate fails the job. A preference only lowers its score.").size(size::SMALL).color(p.dim));
                        ui.add(egui::TextEdit::multiline(&mut g.why).desired_rows(2).desired_width(f32::INFINITY).hint_text("Why it matters"));
                    } else {
                        ui.add(egui::Label::new(RichText::new(&g.rule).font(fserif(size::CARD_SERIF + 2.0)).color(p.text)).wrap());
                        ui.add_space(space::S4);
                        ui.add(egui::Label::new(RichText::new(careers::why_or_note(g)).size(size::ROW + 0.5).color(p.dim)).wrap());
                    }
                    ui.add_space(space::S16);
                    let bw = (ui.available_width() - space::S10) / 2.0;
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = space::S10;
                        ui.allocate_ui_with_layout(egui::vec2(bw, space::BUTTON_H_LARGE + 4.0), Layout::top_down_justified(Align::Center), |ui| {
                            if editing {
                                save = wide_button(ui, p, ButtonKind::Dark, "Save and continue", !g.rule.trim().is_empty()).clicked();
                            } else {
                                yes = wide_button(ui, p, ButtonKind::Primary, "Yes, keep this gate", true).clicked();
                            }
                        });
                        ui.allocate_ui_with_layout(egui::vec2(bw, space::BUTTON_H_LARGE + 4.0), Layout::top_down_justified(Align::Center), |ui| {
                            if editing {
                                drop = wide_button(ui, p, ButtonKind::Secondary, "Leave it out", true).clicked();
                            } else {
                                not_quite = wide_button(ui, p, ButtonKind::Secondary, "Not quite", true).clicked();
                            }
                        });
                    });
                    if !editing {
                        ui.add_space(space::S10);
                        flip = link(ui, p, if g.hard { "Make it a preference" } else { "Make it a hard gate" }).clicked();
                    }
                });
                let r = inner.response.rect;
                let mut shapes: Vec<egui::Shape> = vec![];
                if !p.dark {
                    shapes.extend(shadow::CARD_LIFT.iter().map(|s| egui::Shape::Rect(s.as_shape(r, egui::Rounding::same(radius::DIALOG)))));
                }
                if ix + 1 < n {
                    let behind = r.translate(egui::vec2(space::S10, space::S10));
                    shapes.push(egui::Shape::rect_filled(behind, radius::DIALOG, p.card));
                    shapes.push(egui::Shape::rect_stroke(behind, radius::DIALOG, Stroke::new(stroke::HAIR, p.line)));
                }
                ui.painter().set(behind_at, egui::Shape::Vec(shapes));
            });
            ui.add_space(space::INTAKE_GAP);
            ui.vertical(|ui| {
                ui.add_space(space::S22);
                ui.label(RichText::new(format!("Your gates · {checked} of {n}")).font(fs(size::ROW_SMALL)).color(p.text));
                ui.add_space(space::S8);
                if kept.is_empty() {
                    ui.add(egui::Label::new(RichText::new("Nothing yet. Each gate you keep lands here.").size(size::ROW_SMALL).color(p.dim)).wrap());
                } else {
                    for (rule, hard) in &kept {
                        ui.horizontal(|ui| {
                            ui.add(egui::Label::new(RichText::new(rule).size(size::ROW_SMALL).color(p.text)).truncate());
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                gate_tag(ui, p, *hard, true);
                            });
                        });
                        ui.add_space(space::S4);
                    }
                }
                let more = n - ix;
                ui.add_space(space::S8);
                ui.label(RichText::new(format!("{more} more to check")).size(size::ROW_SMALL).color(p.dim));
                ui.add_space(space::S14);
                reassure(ui, p);
            });
        });
        self.pin_to_foot(ui, space::S40);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = space::S16;
            back = ui.add(egui::Label::new(RichText::new("← Back").font(fs(size::LABEL)).color(p.dim)).sense(Sense::click())).on_hover_cursor(egui::CursorIcon::PointingHand).clicked();
            skip = link(ui, p, "Skip to your kit →").clicked();
        });
        let g = &mut k.gates[ix];
        if flip {
            g.hard = !g.hard;
        } else if not_quite {
            self.resolve_ui.theme_edit = true;
        } else if yes || save {
            g.check = "kept".into();
            self.resolve_ui.theme_edit = false;
            it.theme_ix += 1;
        } else if drop {
            g.check = "not quite".into();
            self.resolve_ui.theme_edit = false;
            it.theme_ix += 1;
        }
        if back {
            self.resolve_ui.theme_edit = false;
            if it.theme_ix > 0 {
                it.theme_ix -= 1;
            } else {
                it.ending = 0;
            }
        } else if skip {
            self.resolve_ui.theme_edit = false;
            it.ending = 3;
        }
    }

    // ---- 08 first week
    fn ic_week(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut Intake) {
        let editing = self.resolve_ui.theme_edit;
        let Some(k) = car(it).kit.as_mut() else { return };
        step_head(ui, p, "Your first week", "Here's where to begin.", "Five tasks from your kit. They go into Tasks when you adopt.");
        ui.add_space(space::S14);
        list_box(ui, p, |ui| {
            for (n, t) in k.week.iter_mut().enumerate() {
                let w = ui.available_width();
                let top = ui.cursor().min;
                egui::Frame::none().inner_margin(egui::Margin::symmetric(space::S18, space::S14)).show(ui, |ui| {
                    ui.set_width(w - space::S18 * 2.0);
                    ui.horizontal(|ui| {
                        let (r, _) = ui.allocate_exact_size(egui::vec2(space::S26, space::S26), Sense::hover());
                        ui.painter().circle_stroke(r.center(), space::S26 / 2.0 - 1.0, Stroke::new(stroke::HAIR, if n == 0 { tok::AMBER } else { p.line }));
                        ui.painter().text(r.center(), Align2::CENTER_CENTER, (n + 1).to_string(), fs(size::SMALL), if n == 0 { tok::AMBER } else { p.text });
                        ui.add_space(space::S12);
                        if editing {
                            let tw = ui.available_width() - space::S40 * 2.5;
                            ui.add(egui::TextEdit::singleline(&mut t.title).desired_width(tw));
                            ui.add(egui::TextEdit::singleline(&mut t.day).desired_width(space::S40 * 2.0));
                        } else {
                            ui.add(egui::Label::new(RichText::new(&t.title).font(fs(size::ROW + 0.5)).color(p.text)).wrap());
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(RichText::new(&t.day).size(size::ROW_SMALL).color(p.dim));
                            });
                        }
                    });
                });
                if n > 0 {
                    ui.painter().line_segment([top, egui::pos2(top.x + w, top.y)], Stroke::new(stroke::HAIR, p.divider));
                }
            }
        });
        ui.add_space(space::S10);
        reassure(ui, p);
        self.pin_to_foot(ui, space::BUTTON_H_LARGE + space::S40);
        let (mut back, mut go, mut change) = (false, false, false);
        action_bar(ui, p, |ui| {
            back = big_button(ui, p, ButtonKind::Secondary, "Back", true).clicked();
            go = big_button(ui, p, ButtonKind::Dark, "Looks right  →", true).clicked();
            if !editing {
                change = big_button(ui, p, ButtonKind::Secondary, "Change these", true).clicked();
            }
        });
        if change {
            self.resolve_ui.theme_edit = true;
        } else if back {
            self.resolve_ui.theme_edit = false;
            it.ending = 1;
            it.theme_ix = it.career.as_ref().and_then(|c| c.kit.as_ref()).map_or(0, |k| k.gates.len().saturating_sub(1));
        } else if go {
            self.resolve_ui.theme_edit = false;
            it.ending = 3;
        }
    }

    // ---- 09 your kit → Adopt
    fn ic_kit(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut Intake) {
        let busy = self.resolve_ui.rx.is_some();
        let err = it.last_error.clone();
        let Some(k) = car(it).kit.as_mut() else { return };
        step_head(ui, p, "Your kit", "Here's your kit.", "");
        ui.add_space(space::S10);
        ui.label(RichText::new("Search name").font(fs(size::ROW_SMALL)).color(p.text));
        ui.add_space(space::S4);
        let fw = (ui.available_width() * 0.6).max(space::S40 * 8.0).min(ui.available_width());
        text_field(ui, p, egui::Id::new("careers-search-name"), &mut k.search_name, "Name this search", false, fw);
        ui.add_space(space::S14);
        let lines = careers::kit_lines(k);
        let w = (ui.available_width() - space::S10 * 2.0) / 3.0;
        for row in lines.chunks(3) {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = space::S10;
                for (title, line) in row {
                    kit_block(ui, p, title, line, w);
                }
            });
            ui.add_space(space::S10);
        }
        ui.add_space(space::S6);
        hero_card(ui, p, |ui| {
            ui.label(RichText::new("Do this first").font(fs(size::SMALL)).color(tok::AMBER));
            ui.add(egui::Label::new(RichText::new(&k.do_first).font(fserif(size::CARD_SERIF)).color(tok::TEXT_ON_DARK)).wrap());
            if !k.why.trim().is_empty() {
                ui.add(egui::Label::new(RichText::new(&k.why).size(size::ROW + 0.5).color(tok::TEXT_ON_DARK_SOFT)).wrap());
            }
        });
        if !err.is_empty() {
            ui.add_space(space::S10);
            ui.add(egui::Label::new(RichText::new(&err).size(size::ROW_SMALL).color(p.orange)).wrap());
        }
        let ok = !k.search_name.trim().is_empty();
        self.pin_to_foot(ui, space::BUTTON_H_LARGE + space::S40);
        let (mut back, mut adopt, mut edit) = (false, false, false);
        action_bar(ui, p, |ui| {
            back = big_button(ui, p, ButtonKind::Secondary, "Back", true).clicked();
            adopt = big_button(ui, p, ButtonKind::Dark, "Adopt, start my job search  →", ok && !busy).clicked();
            edit = big_button(ui, p, ButtonKind::Secondary, "Edit the full proposal  →", true).clicked();
        });
        if back {
            it.ending = 2;
        } else if edit {
            car(it).page = "Target".into();
            it.editor = true;
        } else if adopt {
            it.last_error.clear();
            self.adopt_begin(it);
        }
    }

    // ---- the proposal editor (Careers): left nav, one page at a time
    fn ic_editor(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut Intake) {
        let ai = journey::cfg_for(it).is_some();
        let has_cv = !it.attachments.is_empty();
        let rebuilding = self.resolve_ui.bg.iter().any(|(l, _)| l == "rebuilding the CV");
        let cv_err = self.temp.get(&format!("cv_error:{}", it.id)).cloned();
        let iid = it.id.clone();
        let c = car(it);
        let page = if careers::EDITOR_PAGES.contains(&c.page.as_str()) { c.page.clone() } else { "Target".to_string() };
        let mut goto: Option<&'static str> = None;
        let (mut back, mut rebuild) = (false, false);
        let nav_w = space::PE_LIST.min(ui.available_width() * 0.3);
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                ui.set_width(nav_w);
                ui.spacing_mut().item_spacing.y = space::S4 / 2.0;
                ui.add_space(space::S40);
                for name in careers::EDITOR_PAGES {
                    if section_item(ui, p, nav_w, name, None, page == name).clicked() {
                        goto = Some(name);
                    }
                }
            });
            ui.add_space(space::S24);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = space::S10;
                ui.label(RichText::new("Edit the full proposal").font(fs(size::LABEL)).color(p.orange));
                ui.label(RichText::new(&page).font(fserif(size::STEP_TITLE + 4.0)).color(p.text));
                let here = c.here;
                let Some(k) = c.kit.as_mut() else { return };
                let field = |ui: &mut egui::Ui, label: &str, v: &mut String, id: (&str, usize)| {
                    ui.label(RichText::new(label).font(fs(size::ROW_SMALL)).color(p.text));
                    let w = ui.available_width();
                    text_field(ui, p, egui::Id::new(("careers-editor", id.0, id.1)), v, "", false, w);
                };
                let edit_card = |ui: &mut egui::Ui, on: bool, add: &mut dyn FnMut(&mut egui::Ui)| {
                    egui::Frame::none().fill(p.card).stroke(Stroke::new(stroke::HAIR, if on { tok::AMBER } else { p.line })).rounding(egui::Rounding::same(radius::CARD)).inner_margin(egui::Margin::symmetric(space::S16, space::S14)).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.spacing_mut().item_spacing.y = space::S6;
                        add(ui);
                    });
                };
                match page.as_str() {
                    "Target" => {
                        ui.label(RichText::new("The job this search is for, and what Adopt creates.").size(size::ROW + 0.5).color(p.dim));
                        edit_card(ui, false, &mut |ui| {
                            field(ui, "The job you want", &mut k.target, ("target", 0));
                            field(ui, "Search name", &mut k.search_name, ("search", 0));
                        });
                        if !here {
                            if c.workspace.trim().is_empty() {
                                c.workspace = "Job search".into();
                            }
                            edit_card(ui, false, &mut |ui| {
                                field(ui, "Workspace name", &mut c.workspace, ("workspace", 0));
                                ui.label(RichText::new("Adopt creates this workspace, in Careers mode.").size(size::SMALL).color(p.dim));
                            });
                        }
                    }
                    "Gates" => {
                        ui.label(RichText::new("A hard gate fails the job. A preference only lowers its score.").size(size::ROW + 0.5).color(p.dim));
                        let mut remove = None;
                        for (n, g) in k.gates.iter_mut().enumerate() {
                            edit_card(ui, false, &mut |ui| {
                                ui.horizontal(|ui| {
                                    if tab_chip(ui, p, "Hard gate", None, g.hard).clicked() {
                                        g.hard = true;
                                    }
                                    if tab_chip(ui, p, "Preference", None, !g.hard).clicked() {
                                        g.hard = false;
                                    }
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        if remove_x(ui, p).on_hover_text("Leave it out").clicked() {
                                            remove = Some(n);
                                        }
                                    });
                                });
                                field(ui, "The rule", &mut g.rule, ("rule", n));
                                field(ui, "Why it matters", &mut g.why, ("why", n));
                            });
                        }
                        if let Some(n) = remove {
                            k.gates.remove(n);
                        }
                        if { let w = ui.available_width(); dashed_button(ui, p, w, "+ Add a gate") }.clicked() {
                            k.gates.push(Gate { hard: true, ..Default::default() });
                        }
                    }
                    "CV templates" => {
                        ui.label(RichText::new("Tailor starts from these. Each one says which roles it suits.").size(size::ROW + 0.5).color(p.dim));
                        let mut make_default = None;
                        for (n, t) in k.templates.iter_mut().enumerate() {
                            let on = t.default;
                            edit_card(ui, on, &mut |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(&t.name).font(fs(size::ROW + 0.5)).color(p.text));
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        if t.default {
                                            tag(ui, "Default", p.green_text, p.green_tint);
                                        } else if button(ui, p, ButtonKind::Secondary, "Set as default", false, true).clicked() {
                                            make_default = Some(n);
                                        }
                                    });
                                });
                                ui.label(RichText::new(format!("Suits: {}", t.suits)).size(size::ROW_SMALL).color(p.dim));
                                if t.default {
                                    field(ui, "Headline", &mut t.headline, ("headline", n));
                                }
                            });
                        }
                        if let Some(n) = make_default {
                            for (m, t) in k.templates.iter_mut().enumerate() {
                                t.default = m == n;
                            }
                        }
                        // the rebuilt CV: saved into My CV on Adopt
                        ui.add_space(space::S6);
                        ui.label(RichText::new("Your CV").font(fs(size::SECTION)).color(p.text));
                        ui.label(RichText::new("Saved into My CV when you adopt.").size(size::ROW_SMALL).color(p.dim));
                        if rebuilding {
                            skeleton_card(ui, p, 4, "Rebuilding your CV from your own file…");
                        } else {
                            if let Some(e) = &cv_err {
                                ui.add(egui::Label::new(RichText::new(e).size(size::ROW_SMALL).color(p.orange)).wrap());
                            }
                            answer_box(ui, p, egui::Id::new(("careers-cv-text", &iid)), &mut c.cv, if has_cv { "Rebuild it from your CV, or paste your CV here" } else { "Paste your CV here" }, space::S40 * 4.0, false);
                            if ai && has_cv {
                                rebuild = button(ui, p, ButtonKind::Secondary, if c.cv.trim().is_empty() { "Rebuild my CV" } else { "Rebuild it again" }, false, true).on_hover_text("Rewrites your CV for the job you want. Every fact stays from your own file").clicked();
                            } else if !ai {
                                ui.label(RichText::new("Rebuilding needs an AI. Paste or write your CV here instead.").size(size::SMALL).color(p.dim));
                            }
                        }
                    }
                    "Cover letter" => {
                        ui.label(RichText::new("Tailor writes each letter from this.").size(size::ROW + 0.5).color(p.dim));
                        edit_card(ui, false, &mut |ui| {
                            field(ui, "Structure", &mut k.letter_structure, ("structure", 0));
                            ui.label(RichText::new("The template").font(fs(size::ROW_SMALL)).color(p.text));
                            answer_box(ui, p, egui::Id::new(("careers-letter", 0)), &mut k.letter, "Write the letter here, with [BRACKETS] for each application's details", space::S40 * 4.0, false);
                        });
                    }
                    "Evidence bank" => {
                        ui.label(RichText::new("Proof points Tailor and Coach use. A number makes each one stronger.").size(size::ROW + 0.5).color(p.dim));
                        let mut remove = None;
                        for (n, e) in k.evidence.iter_mut().enumerate() {
                            edit_card(ui, false, &mut |ui| {
                                ui.horizontal(|ui| {
                                    if e.number.trim().is_empty() {
                                        tag(ui, "Needs a number", p.accent_text, p.accent_soft);
                                    } else {
                                        tag(ui, &e.number, p.green_text, p.green_tint);
                                    }
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        if remove_x(ui, p).on_hover_text("Remove").clicked() {
                                            remove = Some(n);
                                        }
                                    });
                                });
                                field(ui, "Proof point", &mut e.point, ("point", n));
                                field(ui, "Number", &mut e.number, ("number", n));
                                field(ui, "Theme", &mut e.theme, ("theme", n));
                                field(ui, "Source role", &mut e.role, ("role", n));
                            });
                        }
                        if let Some(n) = remove {
                            k.evidence.remove(n);
                        }
                        if { let w = ui.available_width(); dashed_button(ui, p, w, "+ Add a proof point") }.clicked() {
                            k.evidence.push(careers::Proof { source: "Intake".into(), ..Default::default() });
                        }
                    }
                    "LinkedIn fixes" => {
                        ui.label(RichText::new("What to change on LinkedIn, area by area.").size(size::ROW + 0.5).color(p.dim));
                        let mut remove = None;
                        for (n, l) in k.linkedin.iter_mut().enumerate() {
                            edit_card(ui, false, &mut |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(&l.area).font(fs(size::ROW + 0.5)).color(p.text));
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        if remove_x(ui, p).on_hover_text("Remove").clicked() {
                                            remove = Some(n);
                                        }
                                    });
                                });
                                field(ui, "Area", &mut l.area, ("area", n));
                                field(ui, "The fix", &mut l.fix, ("fix", n));
                            });
                        }
                        if let Some(n) = remove {
                            k.linkedin.remove(n);
                        }
                        if { let w = ui.available_width(); dashed_button(ui, p, w, "+ Add a fix") }.clicked() {
                            k.linkedin.push(careers::LinkedinFix { area: "Headline".into(), fix: String::new() });
                        }
                    }
                    _ => {
                        ui.label(RichText::new("Five tasks from your kit. They go into Tasks when you adopt.").size(size::ROW + 0.5).color(p.dim));
                        edit_card(ui, false, &mut |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Applications a week").font(fs(size::ROW_SMALL)).color(p.text));
                                ui.add(egui::DragValue::new(&mut k.weekly_target).range(1..=100));
                                ui.add_space(space::S16);
                                ui.label(RichText::new("Roles found a day").font(fs(size::ROW_SMALL)).color(p.text));
                                ui.add(egui::DragValue::new(&mut k.daily_found).range(1..=200));
                            });
                        });
                        for (n, t) in k.week.iter_mut().enumerate() {
                            edit_card(ui, false, &mut |ui| {
                                field(ui, &format!("Task {}", n + 1), &mut t.title, ("task", n));
                                field(ui, "Day", &mut t.day, ("day", n));
                            });
                        }
                    }
                }
                ui.add_space(space::S8);
                back = big_button(ui, p, ButtonKind::Dark, "Back to your kit", true).clicked();
            });
        });
        if let Some(g) = goto {
            car(it).page = g.into();
        }
        if rebuild {
            car(it).cv.clear();
            self.ic_spawn_cv(it);
        }
        if back {
            it.editor = false;
            it.ending = 3;
        }
    }
}

/// The Careers bridge's line: its kit counts.
pub fn bridge_counts(k: &Kit) -> String {
    format!("Your job search: {}, {}, {}", careers::count(k.gates.iter().filter(|g| g.check != "not quite").count(), "gate", "gates"), careers::count(k.templates.len(), "CV template", "CV templates"), careers::count(k.evidence.len(), "proof point", "proof points"))
}
