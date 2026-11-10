//! Intake journey (Pack A): one function per step, 01 to 05, plus the waiting cards. Layout and
//! input only; logic is in resolve::journey. Every colour and size is a token, every component
//! is in widgets.rs.

use super::resolve::JobOut;
use super::theme::*;
use super::widgets::*;
use super::*;
use crate::resolve::intake::{self, Intake};
use crate::resolve::journey;

/// Answers given so far (freshness key for the precomputed understanding).
pub(super) fn answered(i: &Intake) -> usize {
    i.qas.iter().filter(|q| !q.answer.trim().is_empty()).count() + usize::from(!i.input.trim().is_empty())
}

/// Freshness key for the precomputed proposal: the confirmed cards.
pub(super) fn und_key(i: &Intake) -> String {
    i.understanding.as_ref().map(|u| format!("{}|{}|{}|{}|{}", u.project, u.goal, u.state, u.uncertainty, u.outcome)).unwrap_or_default()
}

/// The checklist lines of a wait, ticked by real progress events: done ones, then the first
/// not yet done is "doing", the rest to come. `frozen` (the AI limit screen) shows no "doing".
pub(super) fn checklist(id: &str, lines: &[(&'static str, &'static str)], done: &dyn Fn(&str) -> bool, frozen: bool) -> Vec<(&'static str, u8)> {
    let _ = id;
    let mut doing_shown = frozen;
    lines
        .iter()
        .map(|(text, ev)| {
            if done(ev) {
                (*text, 2)
            } else if !doing_shown {
                doing_shown = true;
                (*text, 1)
            } else {
                (*text, 0)
            }
        })
        .collect()
}

impl App {
    /// After an answer: sorting it ("heard") runs in the background and never blocks; choosing
    /// the next question is the only wait, run side by side with the sorting, cut off at 15 s.
    pub(super) fn ij_spawn_answer(&mut self, it: &Intake, qa: Option<String>, want_next: bool, label: &str) {
        // the page sliding out (M02.1) is a picture of the old step: it never starts work
        if self.intake_ghosting {
            return;
        }
        self.put_intake(it);
        let cfg = journey::cfg_for(it);
        let id = it.id.clone();
        journey::clear_progress(&id, &["sorted", "asked", "chosen"]);
        if qa.is_none() {
            // nothing new to note: that line is done
            journey::note_progress(&id, "sorted");
        }
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
        if self.intake_ghosting {
            return;
        }
        self.put_intake(it);
        let snapshot = it.clone();
        let cfg = journey::cfg_for(it);
        let id = it.id.clone();
        journey::clear_progress(&id, &["reading", "understood", "cards"]);
        let n = answered(it);
        let job = move || {
            let (r, calls) = journey::run_understanding(cfg, &snapshot);
            JobOut::Understood(id, n, r, calls)
        };
        if foreground {
            self.spawn_job("Reading everything you said…", job);
        } else {
            self.resolve_ui.bg_understand = Some((it.id.clone(), n));
            self.spawn_bg("understanding", job);
        }
    }

    pub(super) fn ij_spawn_proposal(&mut self, it: &Intake, section: Option<&str>, foreground: bool) {
        if self.intake_ghosting {
            return;
        }
        self.put_intake(it);
        let snapshot = it.clone();
        let cfg = journey::cfg_for(it);
        let id = it.id.clone();
        journey::clear_progress(&id, &["first-part", "second-part", "merged", "proposed"]);
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
        if self.shot.is_some() || journey::cfg_for(it).is_none() || !it.limit_at.is_empty() {
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

    // ---------------------------------------------------------------- 01 name (the front door)

    pub(super) fn ij_name(&mut self, ui: &mut egui::Ui, _p: &Pal, it: &mut Intake, body: egui::Rect) {
        let dark = pal(true);
        let el = ((self.now - self.view_since) * 1000.0) as f32;
        let reduced = super::motion::reduced() || self.shot.is_some();
        let col = space::FRONT_COL;
        let top = body.top() + body.height() * 0.32;
        let area = egui::Rect::from_min_size(egui::pos2(body.center().x - col / 2.0, top), egui::vec2(col, body.height() * 0.5));
        let mut c = ui.new_child(egui::UiBuilder::new().max_rect(area).layout(Layout::top_down(Align::Min)));
        let ui = &mut c;
        ui.spacing_mut().item_spacing.y = space::S10;
        let f = |delay: f32| super::motion::frame_at(el, super::motion::timing(520.0, delay, super::motion::EASE), reduced);
        super::motion::fade_in(ui, f(0.0), egui::vec2(0.0, space::S6), |ui| {
            ui.label(RichText::new("Let's get started").font(fs(size::LABEL)).color(tok::AMBER));
            ui.label(RichText::new("What's your name?").font(fserif(size::DISPLAY)).color(tok::TEXT_ON_DARK));
        });
        let fid = egui::Id::new("intake-name");
        let mut enter = false;
        super::motion::fade_in(ui, f(420.0), egui::vec2(0.0, space::S6), |ui| {
            let r = text_field(ui, &dark, fid, &mut it.name, "Your first name", false, col);
            if self.resolve_ui.focus_qa.as_deref() != Some("name") {
                r.request_focus();
                self.resolve_ui.focus_qa = Some("name".into());
            }
            enter = r.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
        });
        let ok = !it.name.trim().is_empty();
        let mut go = false;
        super::motion::fade_in(ui, f(560.0), egui::vec2(0.0, space::S6), |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = space::S16;
                go = big_button(ui, &dark, ButtonKind::Primary, "Continue  →", ok).clicked();
                ui.label(RichText::new("Press Enter to continue").size(size::ROW_SMALL).color(tok::TEXT_ON_DARK_MUTED));
            });
        });
        if (go || enter) && ok {
            it.name = it.name.trim().to_string();
            it.greeted = true;
            it.stage = "welcome".into();
            self.resolve_ui.focus_qa = None;
            self.view_since = self.now;
        }
        let foot = egui::pos2(body.left() + space::RAIL_PAD, body.bottom() - space::RAIL_PAD);
        ui.painter().text(foot, Align2::LEFT_BOTTOM, "Your answers save as you go.", fp(size::ROW_SMALL), tok::TEXT_ON_DARK_MUTED);
    }

    // ---------------------------------------------------------------- 02a welcome (E1, E19)

    pub(super) fn ij_welcome(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut Intake) {
        // one 7000 ms clock (M01.5); a click anywhere skips to the end
        let key = format!("welcome_t:{}", it.id);
        let t0 = self.temp.entry(key.clone()).or_insert_with(|| self.now.to_string()).clone();
        let t0: f64 = t0.parse().unwrap_or(self.now);
        let skipped = self.temp.contains_key(&format!("welcome_skip:{}", it.id)) || self.shot.is_some();
        let el = if skipped { 1.0e6 } else { ((self.now - t0) * 1000.0) as f32 };
        if !skipped && ui.input(|i| i.pointer.any_click()) {
            self.temp.insert(format!("welcome_skip:{}", it.id), "1".into());
        }
        let reduced = super::motion::reduced();
        let f = |dur: f32, delay: f32| super::motion::frame_at(el, super::motion::timing(dur, delay, super::motion::EASE), reduced);
        ui.add_space(space::S40);
        super::motion::fade_in(ui, f(630.0, 0.0), egui::vec2(0.0, space::S10), |ui| {
            ui.label(RichText::new(format!("Hi {}.", it.name)).font(fserif(size::HERO)).color(p.text));
        });
        ui.add_space(space::S30);
        let mut go = false;
        super::motion::fade_in(ui, f(700.0, 1960.0), egui::vec2(0.0, space::S10), |ui| {
            ui.label(RichText::new("So,").font(fs(size::ROW + 0.5)).color(p.orange));
            ui.label(RichText::new("What do you need help with?").font(fs(size::TITLE + 2.0)).color(p.text));
        });
        ui.add_space(space::S16);
        let paths = [(Icon::Folder, "I have a messy project", "Notes everywhere, old AI chats, no clear next step. Help me make sense of it.", true), (Icon::Bulb, "I have a new idea", "I want to shape it into something real.", false), (Icon::Compass, "I'm stuck, not sure where to start", "Help me figure out what the actual problem is.", false)];
        for (k, (icon, title, line, open)) in paths.iter().enumerate() {
            super::motion::fade_in(ui, f(560.0, 2800.0 + 420.0 * k as f32), egui::vec2(0.0, space::S14), |ui| {
                if path_card(ui, p, *icon, title, line, *open, k == 0).clicked() {
                    go = true;
                }
            });
            ui.add_space(space::S12);
        }
        super::motion::fade_in(ui, f(560.0, 3640.0), egui::vec2(0.0, 0.0), |ui| {
            ui.label(RichText::new("Press Enter or click to continue. The other two paths open in a later release.").size(size::ROW_SMALL).color(p.dim));
        });
        if el < 4300.0 && !reduced {
            ui.ctx().request_repaint();
        }
        if go || ui.input(|i| i.key_pressed(Key::Enter)) {
            it.situation = "messy_project".into();
            it.stage = "bring".into();
            self.resolve_ui.focus_qa = None;
        }
    }

    // ---------------------------------------------------------------- 02b bring it in (E21, E22)

    pub(super) fn ij_bring(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut Intake) {
        let pull = self.resolve_ui.show_import;
        step_head(ui, p, "In your own words", "Tell Resolve what's going on.", if pull { "" } else { "The idea or project, however it comes out. What exists, what's confusing, what you're trying to decide. Don't tidy it up." });
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
        let has_template = INTAKE_TEMPLATE.iter().any(|h| it.input.contains(&format!("{h}:")));
        if pull {
            ui.add_space(space::S12);
            self.ij_pull_in(ui, p);
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
                    // the eight headings go under what is already typed (M03.3)
                    let mut s = it.input.trim_end().to_string();
                    if !s.is_empty() {
                        s.push('\n');
                    }
                    for h in INTAKE_TEMPLATE {
                        s.push_str(&format!("{h}: \n"));
                    }
                    it.input = s;
                }
                if icon_chip_button(ui, p, Icon::Spark, "Try an example", false).on_hover_text("Fills the template with a sample project you can edit").clicked() {
                    it.input = intake::EXAMPLE_INPUT.to_string();
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if link(ui, p, "Coming from another AI chat? Get a prompt to pull it in →").clicked() {
                        self.resolve_ui.show_import = true;
                    }
                });
            });
            ui.add_space(space::S10);
            self.attach_row(ui, p, it);
        }
        let ok = !it.input.trim().is_empty() || !it.attachments.is_empty();
        self.pin_to_foot(ui, space::BUTTON_H_LARGE + space::S40);
        let (mut go, mut back) = (false, false);
        action_bar(ui, p, |ui| {
            back = big_button(ui, p, ButtonKind::Secondary, "Back", true).clicked();
            go = big_button(ui, p, ButtonKind::Dark, "Continue  →", ok).clicked() || (enter && ok);
            ui.label(RichText::new(if pull && !ok { "Paste the answer to continue" } else { "Enter to continue · Shift+Enter for a new line" }).size(size::ROW_SMALL).color(p.dim));
        });
        if back {
            it.stage = "welcome".into();
            self.temp.insert(format!("welcome_skip:{}", it.id), "1".into());
            self.resolve_ui.focus_qa = None;
        }
        if go {
            self.resolve_ui.show_import = false;
            it.stage = "questions".into();
            self.resolve_ui.focus_qa = None;
            if it.qas.is_empty() {
                self.ij_spawn_answer(it, None, true, "Thinking about what to ask next…");
            }
        }
    }

    /// 02c: the pull-in panel, in place under the box (M03.4).
    fn ij_pull_in(&mut self, ui: &mut egui::Ui, p: &Pal) {
        let copied_at: Option<f64> = self.temp.get("pull_copied").and_then(|s| s.parse().ok());
        let copied = copied_at.map_or(false, |t| self.now - t < 1.5);
        let mut close = false;
        card_lift(ui, p, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Bring it in from ChatGPT, Claude or another AI").font(fs(size::SECTION)).color(p.text));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if icon_button(ui, Icon::Close, false, false).on_hover_text("Close").clicked() {
                        close = true;
                    }
                });
            });
            ui.add_space(space::S10);
            ui.horizontal_top(|ui| {
                step_number(ui, 1);
                ui.add_space(space::S8);
                ui.vertical(|ui| {
                    ui.label(RichText::new("Copy this into the chat where you've been working on it.").font(fs(size::ROW_SMALL)).color(p.text));
                    ui.add_space(space::S8);
                    egui::Frame::none().fill(p.surface_raised).stroke(Stroke::new(stroke::HAIR, p.line)).rounding(egui::Rounding::same(radius::FIELD)).inner_margin(egui::Margin::symmetric(space::S14, space::S12)).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        egui::ScrollArea::vertical().max_height(space::S40 * 3.6).id_salt("pull-prompt").show(ui, |ui| {
                            ui.add(egui::Label::new(RichText::new(intake::IMPORT_PROMPT).size(size::ROW_SMALL).color(p.text)).wrap());
                        });
                    });
                    ui.add_space(space::S10);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = space::S12;
                        if button(ui, p, ButtonKind::Dark, if copied { "Copied ✓" } else { "Copy prompt" }, true, true).clicked() {
                            ui.ctx().copy_text(intake::IMPORT_PROMPT.into());
                            self.temp.insert("pull_copied".into(), self.now.to_string());
                        }
                        ui.label(RichText::new("11 questions. Scroll to see them all.").size(size::SMALL).color(p.dim));
                    });
                });
            });
            ui.add_space(space::S14);
            ui.horizontal_top(|ui| {
                step_number(ui, 2);
                ui.add_space(space::S8);
                ui.vertical(|ui| {
                    ui.label(RichText::new("Paste its answer into the box above, then press Continue.").font(fs(size::ROW_SMALL)).color(p.text));
                    ui.label(RichText::new("Its answers follow the same questions Resolve asks you next.").size(size::SMALL).color(p.dim));
                });
            });
        });
        if copied {
            ui.ctx().request_repaint_after(std::time::Duration::from_millis(200));
        }
        if close {
            self.resolve_ui.show_import = false;
        }
    }

    // ---------------------------------------------------------------- 03 questions (E23, E24)

    pub(super) fn ij_question(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut Intake) {
        if self.resolve_ui.rx.is_some() {
            self.ij_wait_next(ui, p, it);
            return;
        }
        if it.qas.is_empty() {
            self.ij_spawn_answer(it, None, true, "Thinking about what to ask next…");
            return;
        }
        let n = it.qas.len();
        it.cursor = it.cursor.min(n - 1);
        let flags: Vec<bool> = it.qas.iter().map(|q| q.needs_review).collect();
        if let Some(k) = question_dots(ui, p, n, it.cursor, &flags) {
            it.cursor = k;
            self.resolve_ui.focus_qa = None;
        }
        ui.add_space(space::S16);
        let cur = it.cursor;
        // M04.5: a new question on screen arrives (and the waiting card it replaces leaves first)
        let shown_key = format!("q_shown:{}", it.id);
        let (q_t, q_out, was) = (format!("q_t:{}", it.id), format!("q_out:{}", it.id), format!("was_waiting:{}", it.id));
        if !self.intake_ghosting && self.temp.get(&shown_key).map(|s| s.as_str()) != Some(it.qas[cur].id.as_str()) {
            let waited = self.temp.remove(&was);
            if self.temp.contains_key(&shown_key) {
                self.temp.insert(q_t.clone(), self.now.to_string());
                match waited {
                    Some(secs) => self.temp.insert(q_out.clone(), secs),
                    None => self.temp.remove(&q_out),
                };
            }
            self.temp.insert(shown_key, it.qas[cur].id.clone());
        }
        let reduced = super::motion::reduced();
        let e_q = self.intake_clock(&q_t);
        let out_secs: Option<f64> = self.temp.get(&q_out).and_then(|s| s.parse().ok());
        let f_q = e_q.map(|e| super::motion::frame_at(e, super::motion::timing(460.0, if out_secs.is_some() { 240.0 } else { 0.0 }, super::motion::E_SPRING), reduced)).unwrap_or(super::motion::SETTLED);
        if let (Some(e), Some(secs)) = (e_q, out_secs) {
            // the waiting card leaves: 240 ms e-in, every line ticked
            let fo = super::motion::frame_at(e, super::motion::timing(240.0, 0.0, super::motion::E_IN), reduced);
            if !fo.done {
                let top = ui.cursor().min;
                let mut gc = ui.new_child(egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(top + egui::vec2(0.0, space::S8 * fo.t), ui.available_size_before_wrap())).layout(Layout::top_down(Align::Min)));
                gc.multiply_opacity(1.0 - fo.alpha);
                let lines: Vec<(&str, u8)> = journey::WAIT_NEXT.iter().map(|(t, _)| (*t, 2u8)).collect();
                wait_card(&mut gc, p, "Thinking about what to ask next…", "Resolve is reading your last answer.", secs, &lines, Some(1.0));
            }
        }
        if !f_q.done {
            ui.ctx().request_repaint();
        }
        let (mut mv, mv_start) = super::motion::begin_move(ui, f_q, egui::vec2(0.0, space::S18));
        let (mut next, mut back, mut skip, mut propose) = (false, false, false, false);
        {
        let ui = &mut mv;
        // ST2: this question came from Resolve's own list because the AI took over 15 s
        if cur + 1 == n && it.qas[cur].source == "fixed question set" && !it.last_error.is_empty() {
            info_note(ui, p, &it.last_error);
            ui.add_space(space::S12);
        }
        {
            let qa = &mut it.qas[cur];
            ui.label(RichText::new(format!("{} · question {}", journey::target_label(&qa.target), cur + 1)).font(fs(size::LABEL)).color(p.orange));
            serif_text(ui, &qa.question, size::QUESTION, size::LH_QUESTION, p.text);
            if qa.needs_review {
                ui.add_space(space::S4);
                tag(ui, "An earlier answer changed. Does this still hold?", p.orange, p.orange_tint);
            }
            ui.add_space(space::S14);
            let aid = egui::Id::new(("answer", &qa.id));
            let focused = ui.memory(|m| m.has_focus(aid));
            let enter = focused && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Enter));
            let r = answer_box(ui, p, aid, &mut qa.answer, "Type or speak your answer", space::ANSWER_MIN_H, false);
            if self.resolve_ui.focus_qa.as_deref() != Some(qa.id.as_str()) {
                r.request_focus();
                self.resolve_ui.focus_qa = Some(qa.id.clone());
            }
            next |= enter;
            ui.add_space(space::S10);
            if !qa.why.is_empty() {
                let open = self.resolve_ui.show_why;
                ui.horizontal_wrapped(|ui| {
                    if ui.add(egui::Label::new(RichText::new(if open { "▾ Why this question?" } else { "▸ Why this question?" }).size(size::ROW_SMALL).color(p.dim)).sense(Sense::click())).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                        self.resolve_ui.show_why = !open;
                    }
                    if open {
                        ui.label(RichText::new(&qa.why).size(size::ROW_SMALL).color(p.text));
                    }
                });
            }
        }
        ui.add_space(space::S14);
        ui.horizontal(|ui| {
            back = big_button(ui, p, ButtonKind::Secondary, "Back", true).clicked();
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = space::S10;
                next |= big_button(ui, p, ButtonKind::Dark, "Next  →", true).clicked();
                skip = big_button(ui, p, ButtonKind::Secondary, "Skip this question", true).clicked();
            });
        });
        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            ui.label(RichText::new("Enter for Next · Shift+Enter for a new line").size(size::SMALL).color(p.dim));
        });
        ui.add_space(space::S16);
        let w = ui.available_width();
        let (r, _) = ui.allocate_exact_size(egui::vec2(w, stroke::HAIR), Sense::hover());
        ui.painter().rect_filled(r, 0.0, p.line);
        ui.add_space(space::S16);
        if it.suggest_stop {
            info_note(ui, p, "Resolve has heard enough for a grounded proposal. You can stop here, or keep going to sharpen it.");
            ui.add_space(space::S10);
        }
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = space::S16;
            propose = link(ui, p, "Skip the rest and see my proposal →").clicked();
            let cov = it.coverage();
            progress_line(ui, p, cov, space::S40 * 2.0);
            ui.label(RichText::new(format!("Covered {:.0}%. More answers make a sharper proposal.", cov * 100.0)).size(size::ROW_SMALL).color(p.dim));
        });
        }
        super::motion::end_move(ui, &mv, mv_start, 1.0);
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

    /// ST1: the wait for the next question, with its checklist and the 15 s bar.
    fn ij_wait_next(&mut self, ui: &mut egui::Ui, p: &Pal, it: &Intake) {
        let secs = self.wait_secs();
        // when the question arrives, this card leaves first (M04.5)
        if !self.intake_ghosting {
            self.temp.insert(format!("was_waiting:{}", it.id), format!("{secs:.0}"));
        }
        ui.label(RichText::new(format!("Question {}", it.qas.len() + 1)).font(fs(size::LABEL)).color(p.orange));
        ui.add_space(space::S10);
        let id = it.id.clone();
        let lines = checklist(&id, &journey::WAIT_NEXT, &|ev| journey::has_progress(&id, ev), false);
        let sub = if it.qas.is_empty() { "Resolve is reading what you shared." } else { "Resolve is reading your last answer." };
        wait_card(ui, p, "Thinking about what to ask next…", sub, secs, &lines, Some(secs as f32 / space::WAIT_CUTOFF_S));
        ui.add_space(space::S10);
        if journey::cfg_for(it).is_some() {
            ui.label(RichText::new("If this takes more than 15 s, Resolve asks from its own question list.").size(size::ROW_SMALL).color(p.dim));
        }
        ui.ctx().request_repaint_after(std::time::Duration::from_millis(250));
    }

    /// "What Resolve has heard" (E23): evidence / assumptions / open questions. A label menu
    /// changes the kind; pencil and × on hover; click the words to edit in place (Enter saves,
    /// Esc cancels); a first-time hint with Got it.
    pub(super) fn ij_heard(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut Intake) {
        let min_h = (self.intake_pane_h - space::S24).max(space::S40 * 8.0);
        let sorting = self.resolve_ui.bg.iter().any(|(l, _)| l == "sorting the answer");
        let edit_key = "heard_edit";
        let editing: Option<String> = self.temp.get(edit_key).cloned();
        let mut hint_done = false;
        let mut remove: Option<usize> = None;
        let mut start_edit: Option<(String, String)> = None;
        let mut commit: Option<(usize, String)> = None;
        let mut cancel = false;
        let mut relabel: Option<(usize, &'static str)> = None;
        // M04.7: items already there when the panel first shows don't move; later ones land
        let open_key = format!("heard_open:{}", it.id);
        let first_draw = !self.temp.contains_key(&open_key);
        let reduced = super::motion::reduced();
        let mut new_k = 0usize;
        let mut gone: Vec<String> = vec![];
        let mut animating = false;
        egui::Frame::none().fill(p.card).stroke(Stroke::new(stroke::HAIR, p.line)).rounding(egui::Rounding::same(radius::CARD + 3.0)).inner_margin(egui::Margin::same(space::S18 + 2.0)).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.set_min_height(min_h);
            ui.label(RichText::new("What Resolve has heard").font(fs(size::SECTION)).color(p.text));
            ui.add_space(space::S8);
            if !self.reg.settings.heard_hint_done && !it.heard.is_empty() {
                egui::Frame::none().fill(tok::AMBER_LIGHT).stroke(Stroke::new(stroke::HAIR, tok::AMBER_EDGE)).rounding(egui::Rounding::same(radius::FIELD)).inner_margin(egui::Margin::symmetric(space::S12, space::S8)).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal_top(|ui| {
                        let (r, _) = ui.allocate_exact_size(egui::vec2(space::S16, space::S16), Sense::hover());
                        paint_icon(ui.painter(), r.center(), Icon::Pencil, p.text);
                        ui.vertical(|ui| {
                            ui.add(egui::Label::new(RichText::new("Click any line to edit it. Click a label to change it.").size(size::ROW_SMALL).color(p.text)).wrap());
                            if link(ui, p, "Got it").clicked() {
                                hint_done = true;
                            }
                        });
                    });
                });
                ui.add_space(space::S8);
            }
            if sorting {
                egui::Frame::none().fill(tok::AMBER_LIGHT).stroke(Stroke::new(stroke::HAIR, tok::AMBER)).rounding(egui::Rounding::same(radius::FIELD)).inner_margin(egui::Margin::symmetric(space::S12, space::S8)).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(RichText::new("Noting your answer…").size(size::ROW_SMALL).color(p.dim));
                });
                ui.add_space(space::S8);
                skeleton(ui, p, ui.available_width() * 0.7, space::S12);
                ui.add_space(space::S6);
                skeleton(ui, p, ui.available_width() * 0.9, space::S12);
                ui.add_space(space::S10);
            }
            if it.heard.is_empty() && !sorting {
                ui.label(RichText::new("Nothing yet. It fills in as you answer.").size(size::ROW_SMALL).color(p.dim));
            }
            for (kind, title, col) in [("evidence", "Evidence", p.green_text), ("assumption", "Assumptions", p.accent_text), ("question", "Open questions", p.orange)] {
                let idx: Vec<usize> = it.heard.iter().enumerate().filter(|(_, h)| h.kind == kind || (kind == "question" && h.kind == "uncertain")).map(|(i, _)| i).collect();
                if idx.is_empty() {
                    continue;
                }
                ui.add_space(space::S6);
                ui.label(RichText::new(title).font(fs(size::SMALL)).color(col));
                ui.add_space(space::S4);
                for i in idx {
                    let h = it.heard[i].clone();
                    let menu_id = egui::Id::new(("heard-menu", &h.id));
                    let is_edit = editing.as_deref() == Some(h.id.as_str());
                    let seen_key = format!("heard_seen:{}", h.id);
                    if !self.temp.contains_key(&seen_key) && !self.intake_ghosting {
                        let v = if first_draw {
                            "0|0".to_string()
                        } else {
                            new_k += 1;
                            format!("{}|{}", self.now, new_k - 1)
                        };
                        self.temp.insert(seen_key.clone(), v);
                    }
                    let (t0, k): (f64, usize) = self.temp.get(&seen_key).and_then(|s| s.split_once('|').map(|(a, b)| (a.parse().unwrap_or(0.0), b.parse().unwrap_or(0)))).unwrap_or((0.0, 0));
                    let delay = 55.0 * k as f32;
                    let e_land = (t0 > 0.0).then(|| self.motion_at.unwrap_or(((self.now - t0) * 1000.0) as f32));
                    let f_land = e_land.map(|e| super::motion::frame_at(e, super::motion::timing(300.0, delay, super::motion::E_OUT), reduced)).unwrap_or(super::motion::SETTLED);
                    // the amber tint holds while it lands, then fades over 1.2 s
                    let tint = e_land.map(|e| (1.0 - (e - delay - 300.0).max(0.0) / 1200.0).max(0.0)).unwrap_or(0.0);
                    let rm_at: Option<f64> = self.temp.get(&format!("heard_rm:{}", h.id)).and_then(|s| s.parse().ok());
                    let f_rm = rm_at.map(|t| super::motion::frame_at(((self.now - t) * 1000.0) as f32, super::motion::timing(340.0, 0.0, super::motion::E_IN), reduced));
                    if !f_land.done || tint > 0.0 || f_rm.map_or(false, |f| !f.done) {
                        animating = true;
                    }
                    if f_rm.map_or(false, |f| f.done) {
                        gone.push(h.id.clone());
                        continue;
                    }
                    // removed: the row closes, height to 0 (with reduced motion it fades in place)
                    let keep = f_rm.map(|f| if reduced { 1.0 } else { 1.0 - f.t }).unwrap_or(1.0);
                    let h_key = format!("heard_h:{}", h.id);
                    let (mut mv, mv_start) = super::motion::begin_move(ui, f_land, egui::vec2(0.0, space::S8));
                    if let Some(f) = f_rm {
                        mv.multiply_opacity(1.0 - f.alpha);
                        let prev_h: f32 = self.temp.get(&h_key).and_then(|s| s.parse().ok()).unwrap_or(space::S40 * 2.0);
                        let clip = egui::Rect::from_min_size(mv_start, egui::vec2(mv.available_width() + space::S8, prev_h * keep));
                        let c = mv.clip_rect().intersect(clip);
                        mv.set_clip_rect(c);
                    }
                    {
                    let ui = &mut mv;
                    let top = ui.cursor().min;
                    let bg_at = ui.painter().add(egui::Shape::Noop);
                    let row = egui::Frame::none().inner_margin(egui::Margin::symmetric(0.0, space::S6)).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        let open = ui.memory(|m| m.is_popup_open(menu_id));
                        let chip = heard_label(ui, p, &h.kind, open);
                        if chip.clicked() {
                            ui.memory_mut(|m| m.toggle_popup(menu_id));
                        }
                        egui::popup_below_widget(ui, menu_id, &chip, egui::PopupCloseBehavior::CloseOnClick, |ui| {
                            ui.set_min_width(space::S40 * 4.0);
                            for (k, name) in [("evidence", "Evidence"), ("assumption", "Assumption"), ("question", "Question")] {
                                ui.horizontal(|ui| {
                                    let (fg, bg) = match k {
                                        "evidence" => (p.green_text, p.green_tint),
                                        "assumption" => (p.accent_text, p.accent_soft),
                                        _ => (p.orange, p.orange_tint),
                                    };
                                    if tag(ui, name, fg, bg).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                        relabel = Some((i, k));
                                    }
                                    if h.kind == k || (k == "question" && h.kind == "uncertain") {
                                        ui.label(RichText::new("✓").color(p.text));
                                    }
                                });
                            }
                        });
                        ui.add_space(space::S4);
                        if is_edit {
                            let mut buf = self.temp.get("heard_buf").cloned().unwrap_or_else(|| h.text.clone());
                            let eid = egui::Id::new(("heard-text", &h.id));
                            let r = text_field(ui, p, eid, &mut buf, "", false, ui.available_width());
                            r.request_focus();
                            self.temp.insert("heard_buf".into(), buf.clone());
                            if ui.input(|x| x.key_pressed(Key::Escape)) {
                                cancel = true;
                            } else if r.lost_focus() {
                                commit = Some((i, buf));
                            }
                        } else {
                            let t = ui.add(egui::Label::new(RichText::new(&h.text).size(size::ROW_SMALL + 0.5).color(p.text)).wrap().sense(Sense::click())).on_hover_cursor(egui::CursorIcon::Text);
                            if t.clicked() {
                                start_edit = Some((h.id.clone(), h.text.clone()));
                            }
                        }
                    });
                    let rect = egui::Rect::from_min_size(top, egui::vec2(ui.available_width(), row.response.rect.height()));
                    let hovered = ui.rect_contains_pointer(rect) && !is_edit;
                    if hovered {
                        ui.painter().set(bg_at, egui::Shape::rect_filled(rect.expand2(egui::vec2(space::S8, 0.0)), radius::CHIP, p.surface_raised));
                        // pencil and × on hover
                        let x_rect = egui::Rect::from_center_size(egui::pos2(rect.right() - space::S8, rect.top() + space::S16), egui::vec2(space::S18, space::S18));
                        let pen_rect = x_rect.translate(egui::vec2(-space::S26, 0.0));
                        let pen = ui.interact(pen_rect, egui::Id::new(("heard-pen", &h.id)), Sense::click());
                        let x = ui.interact(x_rect, egui::Id::new(("heard-x", &h.id)), Sense::click());
                        paint_icon(ui.painter(), pen_rect.center(), Icon::Pencil, if pen.hovered() { p.text } else { p.dim });
                        paint_icon(ui.painter(), x_rect.center(), Icon::Close, if x.hovered() { p.text } else { p.dim });
                        if pen.on_hover_text("Edit").clicked() {
                            start_edit = Some((h.id.clone(), h.text.clone()));
                        }
                        if x.on_hover_text("Remove").clicked() {
                            remove = Some(i);
                        }
                    } else if tint > 0.0 {
                        ui.painter().set(bg_at, egui::Shape::rect_filled(rect.expand2(egui::vec2(space::S8, 0.0)), radius::CHIP, tok::AMBER_LIGHT.gamma_multiply(tint)));
                    }
                    let (r, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), stroke::HAIR), Sense::hover());
                    ui.painter().rect_filled(r, 0.0, p.divider);
                    }
                    if f_rm.is_none() {
                        self.temp.insert(h_key, mv.min_rect().height().to_string());
                    }
                    super::motion::end_move(ui, &mv, mv_start, keep);
                }
            }
        });
        if !self.intake_ghosting {
            self.temp.insert(open_key, "1".into());
        }
        if animating {
            ui.ctx().request_repaint();
        }
        if !gone.is_empty() {
            it.heard.retain(|h| !gone.contains(&h.id));
            for g in &gone {
                self.temp.remove(&format!("heard_rm:{g}"));
            }
        }
        if hint_done {
            self.reg.settings.heard_hint_done = true;
            self.reg.save();
        }
        if let Some((i, k)) = relabel {
            if let Some(h) = it.heard.get_mut(i) {
                h.kind = k.into();
                h.corrected = true;
            }
        }
        if let Some((id, text)) = start_edit {
            self.temp.insert(edit_key.into(), id);
            self.temp.insert("heard_buf".into(), text);
        }
        if let Some((i, text)) = commit {
            let t = text.replace(['\n', '\r'], " ").trim().to_string();
            if let Some(h) = it.heard.get_mut(i) {
                if !t.is_empty() && t != h.text {
                    h.text = t;
                    h.corrected = true;
                }
            }
            self.temp.remove(edit_key);
            self.temp.remove("heard_buf");
        }
        if cancel {
            self.temp.remove(edit_key);
            self.temp.remove("heard_buf");
        }
        if let Some(i) = remove {
            // the row closes first (M04.7), then it is taken out
            if let Some(h) = it.heard.get(i) {
                self.temp.insert(format!("heard_rm:{}", h.id), self.now.to_string());
            }
        }
    }

    // ---------------------------------------------------------------- 04 confirm understanding

    pub(super) fn ij_confirm(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut Intake) {
        let n = answered(it);
        // the cards' clock (M05.1) starts when they first show; until then it isn't running
        if !self.intake_ghosting {
            let waiting = it.limit_at == "understanding" || (self.resolve_ui.rx.is_some() && self.resolve_ui.busy.starts_with("Reading everything")) || it.understanding.is_none() || (it.understood_n != n && !it.editing && self.shot.is_none());
            if waiting {
                self.temp.remove(&format!("cards_t:{}", it.id));
            }
        }
        if it.limit_at == "understanding" {
            self.ij_limit(ui, p, it);
            return;
        }
        if self.resolve_ui.rx.is_some() && self.resolve_ui.busy.starts_with("Reading everything") {
            self.ij_wait_understand(ui, p, it, self.wait_secs());
            return;
        }
        if it.understanding.is_none() && !it.last_error.is_empty() {
            self.ij_error(ui, p, it, "Confirm understanding");
            return;
        }
        // (a screenshot shows what's there rather than starting AI work)
        let stale = it.understanding.is_none() || (it.understood_n != n && !it.editing && self.shot.is_none());
        if stale {
            // precomputed in the background? wait for it (with visible progress), else ask now
            let pending = self.resolve_ui.bg_understand.as_ref().map_or(false, |(x, k)| x == &it.id && *k == n);
            if pending {
                let started = self.temp.entry("wait:understanding".to_string()).or_insert_with(|| self.now.to_string()).parse::<f64>().unwrap_or(self.now);
                self.ij_wait_understand(ui, p, it, (self.now - started).max(0.0));
            } else {
                self.ij_spawn_understand(it, true);
            }
            return;
        }
        // 05: no outcome to build on; one question before going on (E3)
        let u0 = it.understanding.clone().unwrap_or_default();
        if !it.editing && !it.outcome_confirmed && journey::clean_outcome(&u0.outcome).split_whitespace().count() < 3 {
            it.stage = "outcome".into();
            return;
        }
        if link(ui, p, "← Back to the questions").clicked() {
            it.stage = "questions".into();
            it.editing = false;
            return;
        }
        let editing = it.editing;
        step_head(ui, p, "", "Here's what Resolve understood.", if editing { "Change anything that's wrong. The outcome is best in your own words." } else { "Check it. It shapes everything that follows." });
        ui.add_space(space::S14);
        // M05.1: the cards come in once they exist: 460 ms e-spring at 120, 70 apart, from 18 px
        // down, in reading order; M05.2: the action bar rises last, 480 ms at 460, from 30 px
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
        let mut u = u0;
        let card = |ui: &mut egui::Ui, label: &str, v: &mut String, key: &str, min_h: f32| -> f32 {
            let fid = egui::Id::new(("confirm-field", key));
            let inner = egui::Frame::none().fill(p.card).stroke(Stroke::new(stroke::HAIR, if editing && ui.memory(|m| m.has_focus(fid)) { tok::AMBER } else { p.line })).rounding(egui::Rounding::same(radius::CARD)).inner_margin(egui::Margin::same(space::S16)).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.set_min_height(min_h);
                ui.label(RichText::new(label).font(fs(size::SMALL)).color(p.orange));
                ui.add_space(space::S6);
                if editing {
                    answer_box(ui, p, fid, v, "", space::S40 * 2.2, false);
                } else if v.trim().is_empty() {
                    ui.add(egui::Label::new(RichText::new("Not said yet. Press \"Something's wrong\" to add it.").size(size::ROW + 0.5).color(p.dim)).wrap());
                } else {
                    ui.add(egui::Label::new(RichText::new(v.as_str()).size(size::ROW + 0.5).color(p.text)).wrap());
                }
            });
            inner.response.rect.height()
        };
        // each pair of cards shares the taller one's height (remembered from the last frame)
        let pair = |ui: &mut egui::Ui, row: &str, base: usize, a: (&str, &mut String), b: (&str, &mut String)| {
            let key = egui::Id::new(("confirm-row", row, editing));
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
        pair(ui, "1", 0, ("Project", &mut u.project), ("What you want", &mut u.goal));
        ui.add_space(space::S10);
        pair(ui, "2", 2, ("Where it is", &mut u.state), ("Biggest uncertainty", &mut u.uncertainty));
        ui.add_space(space::S10);
        let before_outcome = u.outcome.clone();
        super::motion::fade_in(ui, fc(4), egui::vec2(0.0, space::S18), |ui| card(ui, "The outcome: who is better off, and how you'd know", &mut u.outcome, "outcome", 0.0));
        if u.outcome != before_outcome {
            // editing the outcome is writing it in your own words: that confirms it
            it.outcome_confirmed = true;
        }
        it.understanding = Some(u.clone());
        let ok = !u.outcome.trim().is_empty() && !u.project.trim().is_empty();
        self.pin_to_foot(ui, space::BUTTON_H_LARGE + space::S40);
        let (mut go, mut change, mut cancel) = (false, false, false);
        super::motion::fade_in(ui, f_bar, egui::vec2(0.0, space::S30), |ui| {
            action_bar(ui, p, |ui| {
                if editing {
                    go = big_button(ui, p, ButtonKind::Dark, "Save and continue  →", ok).clicked();
                    cancel = big_button(ui, p, ButtonKind::Secondary, "Cancel", true).clicked();
                    ui.label(RichText::new("Editing the outcome counts as confirming it.").size(size::ROW_SMALL).color(p.dim));
                } else {
                    go = big_button(ui, p, ButtonKind::Dark, "This looks right, show me the proposal  →", ok).clicked();
                    change = big_button(ui, p, ButtonKind::Secondary, "Something's wrong, let me change it", true).clicked();
                }
            })
        });
        if !ok {
            ui.label(RichText::new("Add the project and the outcome to continue").size(size::SMALL).color(p.dim));
        }
        let snap_key = format!("confirm_before:{}", it.id);
        if change {
            self.temp.insert(snap_key, serde_json::to_string(&u).unwrap_or_default());
            it.editing = true;
        } else if cancel {
            if let Some(old) = self.temp.remove(&snap_key).and_then(|s| serde_json::from_str(&s).ok()) {
                it.understanding = Some(old);
            }
            it.editing = false;
        } else if go {
            // pressing "This looks right" is the confirmation; edits stay as written
            for v in [&mut u.project, &mut u.goal, &mut u.state, &mut u.uncertainty, &mut u.outcome] {
                *v = journey::humanise(v);
            }
            it.understanding = Some(u);
            it.outcome_confirmed = true;
            it.editing = false;
            self.temp.remove(&snap_key);
            it.stage = "proposal".into();
        }
    }

    /// The wait for the understanding cards (same card as ST1, no 15 s cut-off).
    fn ij_wait_understand(&mut self, ui: &mut egui::Ui, p: &Pal, it: &Intake, secs: f64) {
        ui.label(RichText::new("Confirm understanding").font(fs(size::LABEL)).color(p.orange));
        ui.add_space(space::S10);
        let id = it.id.clone();
        let lines = checklist(&id, &journey::WAIT_UNDERSTAND, &|ev| journey::has_progress(&id, ev), false);
        wait_card(ui, p, "Reading everything you said…", "Separating what's decided from what's open.", secs, &lines, None);
        ui.ctx().request_repaint_after(std::time::Duration::from_millis(250));
    }

    // ---------------------------------------------------------------- 05 no clear outcome (E3)

    pub(super) fn ij_outcome(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut Intake) {
        const QUESTION: &str = "If Resolve does its job, what is different in three months that isn't true today?";
        let pass_key = format!("outcome_pass:{}", it.id);
        let passed_at: Option<f64> = self.temp.get(&pass_key).and_then(|s| s.parse().ok());
        let el_pass = passed_at.map(|t| ((self.now - t) * 1000.0) as f32);
        let mut go = false;
        let mut fail: Option<String> = self.temp.get(&format!("outcome_fail:{}", it.id)).cloned();
        ui.add_space(space::S24);
        let w = (ui.available_width() - space::INTAKE_GAP) / 2.0;
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                ui.set_width(w);
                ui.label(RichText::new("Before we build this").font(fs(size::LABEL)).color(p.orange));
                ui.add(egui::Label::new(RichText::new("We're not going to guess.").font(fserif(size::DISPLAY)).color(p.text)).wrap());
                ui.add_space(space::S14);
                let mut job = egui::text::LayoutJob::default();
                job.append("You've given us real detail. ", 0.0, egui::text::TextFormat { font_id: fp(size::ROW + 1.5), color: p.text, ..Default::default() });
                // the floor reason, highlighted (M05.6)
                job.append("What's missing is the finish line, so right now the work could lead almost anywhere.", 0.0, egui::text::TextFormat { font_id: fp(size::ROW + 1.5), color: p.text, background: tok::AMBER_LIGHT, ..Default::default() });
                job.wrap.max_width = w;
                ui.label(job);
                ui.add_space(space::S10);
                ui.add(egui::Label::new(RichText::new("We could draw a roadmap that looks convincing. We'd rather ask one thing than hand you a plan built on a blank.").size(size::ROW + 1.5).color(p.dim)).wrap());
            });
            ui.add_space(space::INTAKE_GAP);
            ui.vertical(|ui| {
                ui.set_width(w);
                card_lift(ui, p, |ui| {
                    ui.add_space(space::S6);
                    ui.label(RichText::new("ONE QUESTION").font(fs(size::TAG + 0.5)).color(p.orange));
                    ui.add(egui::Label::new(RichText::new(QUESTION).font(fserif(size::CARD_SERIF - 2.0)).color(p.text)).wrap());
                    ui.add_space(space::S4);
                });
                ui.add_space(space::S14);
                if let Some(el) = el_pass {
                    // pass: ✓, the line, then on (M05.7)
                    let f = super::motion::frame_at(el, super::motion::timing(600.0, 100.0, super::motion::E_SPRING), super::motion::reduced());
                    super::motion::fade_in(ui, f, egui::vec2(0.0, space::S6), |ui| {
                        ui.horizontal(|ui| {
                            let (r, _) = ui.allocate_exact_size(egui::vec2(space::S26, space::S26), Sense::hover());
                            ui.painter().circle_filled(r.center(), space::S26 / 2.0, tok::AMBER);
                            paint_icon(ui.painter(), r.center(), Icon::Check, tok::INK);
                            ui.label(RichText::new("That's the piece we needed.").font(fs(size::SECTION)).color(p.text));
                        });
                    });
                    ui.ctx().request_repaint();
                    if el > 1200.0 {
                        go = true;
                    }
                } else {
                    let aid = egui::Id::new(("outcome-reply", &it.id));
                    answer_box(ui, p, aid, &mut it.outcome_reply, "Answer in a sentence or two.", space::ANSWER_MIN_H, false);
                    ui.add_space(space::S14);
                    let ok = !it.outcome_reply.trim().is_empty();
                    let w2 = ui.available_width();
                    ui.allocate_ui_with_layout(egui::vec2(w2, space::BUTTON_H_LARGE), Layout::top_down_justified(Align::Center), |ui| {
                        if wide_button(ui, p, ButtonKind::Dark, "Continue", ok).clicked() {
                            // the check: a real finish line, not a blank (the same floor the
                            // confirm screen uses)
                            let a = journey::clean_outcome(it.outcome_reply.trim());
                            if a.split_whitespace().count() >= 5 {
                                self.temp.insert(pass_key.clone(), self.now.to_string());
                                self.temp.remove(&format!("outcome_fail:{}", it.id));
                                fail = None;
                            } else {
                                let msg = intake::reask_copy(false, false);
                                self.temp.insert(format!("outcome_fail:{}", it.id), msg.clone());
                                fail = Some(msg);
                            }
                        }
                    });
                    ui.add_space(space::S10);
                    if let Some(f) = &fail {
                        ui.add(egui::Label::new(RichText::new(f).size(size::ROW_SMALL).color(p.orange)).wrap());
                        ui.add_space(space::S6);
                    }
                    ui.add(egui::Label::new(RichText::new("Answering re-runs the check. It might be enough to build on, or there might be one more thing we need. No promises yet.").size(size::SMALL).color(p.dim)).wrap());
                }
            });
        });
        if go {
            let answer = journey::humanise(it.outcome_reply.trim());
            if let Some(u) = it.understanding.as_mut() {
                u.outcome = answer.clone();
            }
            // kept with the other answers, so the transcript has it
            it.qas.push(journey::Qa { id: crate::model::new_id(), target: "outcome".into(), question: QUESTION.into(), answer: answer.clone(), classified: answer, source: "no clear outcome".into(), ..Default::default() });
            it.outcome_confirmed = true;
            it.outcome_reply.clear();
            it.stage = "confirm".into();
            self.temp.remove(&pass_key);
        }
    }

    // ---------------------------------------------------------------- ST4 error, 12 AI limit

    /// ST4 (E8): what failed, that nothing is lost, Try again (focused), the step's manual path
    /// and Change AI settings.
    pub(super) fn ij_error(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut Intake, step: &str) {
        ui.label(RichText::new(step).font(fs(size::LABEL)).color(p.orange));
        ui.add(egui::Label::new(RichText::new("Resolve couldn't read that just now.").font(fserif(size::STEP_TITLE + 2.0)).color(p.text)).wrap());
        ui.add_space(space::S14);
        let what = it.last_error.split(" (").next().unwrap_or(&it.last_error).trim_end_matches('.').to_string() + ".";
        let (manual, next) = match step {
            "Confirm understanding" => ("Fill the cards yourself", "Nothing you wrote is lost. Try again, or fill the cards yourself and carry on."),
            _ => ("Use the version made without AI", "Nothing you wrote is lost. Try again, or carry on with a version made from your own words, without AI."),
        };
        error_panel(ui, p, &what, next);
        ui.add_space(space::S16);
        let (mut retry, mut by_hand, mut settings) = (false, false, false);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = space::S10;
            let r = big_button(ui, p, ButtonKind::Dark, "Try again", true);
            if !self.temp.contains_key("err_focused") {
                r.request_focus();
                self.temp.insert("err_focused".into(), "1".into());
            }
            retry = r.clicked();
            by_hand = big_button(ui, p, ButtonKind::Secondary, manual, true).clicked();
            settings = link(ui, p, "Change AI settings").clicked();
        });
        if retry || by_hand {
            self.temp.remove("err_focused");
            it.last_error.clear();
        }
        match step {
            "Confirm understanding" => {
                if retry {
                    self.ij_spawn_understand(it, true);
                } else if by_hand {
                    // blank cards in edit mode (nothing invented)
                    it.understanding = Some(crate::resolve::intake::Understanding::default());
                    it.understood_n = answered(it);
                    it.editing = true;
                }
            }
            _ => {
                if retry {
                    self.ij_spawn_proposal(it, None, true);
                } else if by_hand {
                    let u = it.understanding.clone().unwrap_or_default();
                    it.proposal = Some(journey::offline_proposal(it, &u));
                    it.proposed_from = und_key(it);
                    it.ending = 0;
                }
            }
        }
        if settings {
            self.settings_ui.tab = 1;
            self.go(View::Settings);
        }
    }

    /// 12 AI limit (E10, E11): what stopped, that nothing is lost, the budget or cap sentence,
    /// Continue without AI, Change AI settings, Leave and come back later; the checklist frozen
    /// where it stopped, with the same lines as the wait.
    pub(super) fn ij_limit(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut Intake) {
        let proposal = it.limit_at == "proposal";
        let lines: &[(&str, &str)] = if proposal { &journey::WAIT_PROPOSAL } else { &journey::WAIT_UNDERSTAND };
        let budget = crate::ai::is_budget(&it.last_error);
        let (mut without, mut settings, mut leave) = (false, false, false);
        ui.add_space(space::S10);
        let w = ui.available_width();
        let left_w = (w * 0.56).min(space::S40 * 12.0);
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                ui.set_width(left_w);
                // the ring breathes on a 3200 ms loop, no spin (M10)
                let d = space::S40 + space::S26;
                let (r, _) = ui.allocate_exact_size(egui::vec2(d, d), Sense::hover());
                let ph = super::motion::loop_phase(ui.ctx(), 3200.0);
                let k = if super::motion::reduced() { 1.0 } else { 0.85 + 0.15 * (0.5 - 0.5 * (ph * std::f32::consts::TAU).cos()) };
                ui.painter().circle_filled(r.center(), d / 2.0 * k, tok::amber_a(0.14));
                ui.painter().circle_stroke(r.center(), d / 2.0 - space::S8, Stroke::new(stroke::ICON * 2.0, tok::AMBER));
                paint_icon(ui.painter(), r.center(), Icon::Pause, p.orange);
                ui.ctx().request_repaint_after(std::time::Duration::from_millis(60));
                ui.add_space(space::S14);
                ui.label(RichText::new(if proposal { "Your proposal" } else { "Confirm understanding" }).font(fs(size::LABEL)).color(p.orange));
                ui.add(egui::Label::new(RichText::new("You've reached your AI limit for now.").font(fserif(size::STEP_TITLE + 4.0)).color(p.text)).wrap());
                ui.add_space(space::S12);
                ui.add(egui::Label::new(RichText::new("Nothing is lost. Your answers, what Resolve understood and the work done so far are saved exactly as you left them.").size(size::ROW + 1.5).color(p.text)).wrap());
                ui.add_space(space::S8);
                let sentence = if budget {
                    "This month's AI budget has been reached. Carry on without AI from here, or change the AI in Settings and pick up from this line, not from the start.".to_string()
                } else {
                    "You've reached your AI's usage limit. Carry on without AI from here, or change the AI in Settings and pick up from this line, not from the start.".to_string()
                };
                ui.add(egui::Label::new(RichText::new(sentence).size(size::ROW + 0.5).color(p.dim)).wrap());
                ui.add_space(space::S4);
                ui.add(egui::Label::new(RichText::new(format!("It said: {}", it.last_error)).size(size::SMALL).color(p.dim)).wrap());
                ui.add_space(space::S18);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = space::S10;
                    without = big_button(ui, p, ButtonKind::Dark, "Continue without AI  →", true).clicked();
                    settings = big_button(ui, p, ButtonKind::Secondary, "Change AI settings", true).clicked();
                });
                ui.add_space(space::S12);
                leave = link(ui, p, "Leave and come back later").clicked();
            });
            ui.add_space(space::INTAKE_GAP);
            ui.vertical(|ui| {
                ui.add_space(space::S10);
                ui.label(RichText::new("Where it stopped").font(fs(size::ROW_SMALL)).color(p.text));
                ui.add_space(space::S10);
                let mut shown_line = false;
                for (text, ev) in lines {
                    let done = it.limit_done.iter().any(|d| d == ev);
                    if !done && !shown_line {
                        shown_line = true;
                        let w = ui.available_width();
                        let (r, _) = ui.allocate_exact_size(egui::vec2(w, stroke::HAIR), Sense::hover());
                        ui.painter().rect_filled(r, 0.0, p.dim);
                        ui.label(RichText::new("Saved here, exactly as it is").font(fs(size::SMALL)).color(p.orange));
                        ui.add_space(space::S8);
                    }
                    frozen_row(ui, p, text, done);
                    ui.add_space(space::S8);
                }
            });
        });
        if without {
            // the rest of this intake runs without AI, from this line on
            it.provider = "offline".into();
            it.limit_at.clear();
            it.last_error.clear();
            if proposal {
                let u = it.understanding.clone().unwrap_or_default();
                it.proposal = Some(journey::offline_proposal(it, &u));
                it.proposed_from = und_key(it);
                it.ending = 0;
            } else {
                it.understanding = Some(journey::offline_understanding(it));
                it.understood_n = answered(it);
            }
        }
        if settings {
            self.settings_ui.tab = 1;
            self.go(View::Settings);
        }
        if leave {
            self.save_now();
            self.go(View::Dashboard);
        }
    }
}
