//! The intake journey screens (Pack A): full window, no sidebar. A dark top bar with Save and
//! exit and a ⋯ menu; a dark rail with the four steps on the left (72 wide on the wide steps);
//! a light work pane with one step at a time. The front door (name), the bridge and the reveal
//! are full-bleed. Logic lives in `resolve::journey`; this file is the frame only. Colours, sizes
//! and spacing come from `theme.rs`, components from `widgets.rs`.

use super::theme::*;
use super::widgets::*;
use super::*;
use crate::resolve::intake::Intake;

/// The bridge's shortest run (E14), seconds.
pub(super) const BRIDGE_MIN_S: f64 = 7.2;

/// Which page of the intake is showing (a change slides the pages, M02.1/2).
fn step_key(it: &Intake) -> String {
    match it.stage.as_str() {
        "proposal" if it.editor => "proposal:editor".into(),
        "proposal" => format!("proposal:{}", it.ending.min(3)),
        s => s.to_string(),
    }
}

/// A page's column width, left margin and heard panel width in a pane `pane_w` wide: 760 (800
/// on Confirm), 960 centred at 1600 and wider (ST6), Questions with its heard panel.
fn pane_layout(stage: &str, whole: bool, pane_w: f32, wide: bool) -> (f32, f32, f32) {
    let heard_w = if wide { space::HEARD_W_WIDE } else { space::HEARD_W };
    let room = pane_w - space::PANE_SIDE * 2.0;
    let col = match stage {
        "questions" => (pane_w - space::PANE_SIDE_QUESTIONS * 2.0 - heard_w - space::INTAKE_GAP).min(space::INTAKE_COL),
        _ if whole => room.min(space::INTAKE_COL_WIDE * 1.3),
        _ if wide => space::INTAKE_COL_WIDE.min(room),
        "confirm" => space::INTAKE_COL_CONFIRM.min(room),
        _ => space::INTAKE_COL.min(room),
    }
    .max(space::S40 * 8.0);
    let total = if stage == "questions" { col + heard_w + space::INTAKE_GAP } else { col };
    // left-aligned under 1600, centred in the pane at 1600 and wider
    let left = if wide { ((pane_w - total) / 2.0).max(space::PANE_SIDE) } else if stage == "questions" || whole { space::PANE_SIDE_QUESTIONS } else { space::PANE_SIDE };
    (col, left, heard_w)
}

/// The pages in order (moving back flips the slide).
fn step_order(key: &str) -> i32 {
    match key {
        "name" => 0,
        "welcome" => 1,
        "bring" => 2,
        "questions" => 3,
        "confirm" => 4,
        "outcome" => 5,
        "proposal:0" => 6,
        "proposal:1" => 7,
        "proposal:2" => 8,
        "proposal:3" => 9,
        "proposal:editor" => 10,
        _ => 11,
    }
}

impl App {
    /// The whole intake, full screen (no sidebar, no app top bar).
    pub fn intake_view(&mut self, ui: &mut egui::Ui, p: &Pal, id: &str) {
        let Some(mut it) = self.get_intake(id) else {
            self.view = View::Dashboard;
            return;
        };
        if !it.is_v2() {
            // an intake from the earlier flow continues in the new one, keeping its text
            it.v = 2;
            it.stage = if it.input.trim().is_empty() && it.attachments.is_empty() { "bring".into() } else { "questions".into() };
            if it.name.is_empty() {
                it.stage = "name".into();
            }
        }
        ui.spacing_mut().item_spacing.y = 0.0;
        let before = serde_json::to_string(&it).unwrap_or_default();
        let busy = self.resolve_ui.rx.is_some();
        let (mut exit, mut restart) = (false, false);
        // M02: a new page starts the step clock (whether a click or a finished job changed it)
        let start_key = step_key(&it);
        let kk = format!("step_key:{}", it.id);
        match self.temp.get(&kk).cloned() {
            Some(prev) if prev != start_key => {
                self.temp.insert(format!("step_t:{}", it.id), self.now.to_string());
                self.temp.insert(format!("step_dir:{}", it.id), if step_order(&start_key) >= step_order(&prev) { "1" } else { "-1" }.into());
                // a page left by a click this view drew is kept to slide out; otherwise none
                if self.intake_ghost.as_ref().map_or(false, |(g, _)| step_key(g) != prev) {
                    self.intake_ghost = None;
                }
            }
            Some(_) => {}
            None => {
                self.intake_ghost = None;
            }
        }
        self.temp.insert(kk, start_key.clone());
        if it.stage != "confirm" {
            self.temp.remove(&format!("cards_t:{}", it.id));
        }

        // the bridge holds its 7.2 s even when Adopt has already finished (E14)
        let bridge_key = format!("bridge_t:{}", it.id);
        let bridge_t0 = self.temp.get(&bridge_key).and_then(|s| s.parse::<f64>().ok());
        let in_bridge = it.stage == "bridge" || (it.stage == "reveal" && bridge_t0.map_or(false, |t0| self.now - t0 < BRIDGE_MIN_S && self.shot.is_none()));
        if in_bridge {
            self.ij_bridge(ui, p, &mut it);
        } else if it.stage == "reveal" {
            self.ij_reveal(ui, p, &mut it);
        } else if it.stage == "proposal" && it.editor && it.proposal.is_some() && !self.testlab {
            // the full proposal editor has its own frame (C-E11, Pack C); Test Intake keeps the
            // inspector's frame
            self.proposal_editor_page(ui, p, &mut it);
        } else {
            let stage = it.stage.clone();
            let front = stage == "name";
            let step = match stage.as_str() {
                "name" | "welcome" => 1,
                "bring" | "questions" => 2,
                "confirm" | "outcome" => 3,
                _ => 4,
            };
            let whole = stage == "proposal" && it.ending >= 3 && !it.editor && it.limit_at.is_empty();
            let narrow = stage == "questions" || whole;
            let pct = match stage.as_str() {
                "name" => 0.04,
                "welcome" => 0.10,
                "bring" => 0.22,
                "questions" => 0.30 + 0.30 * it.coverage(),
                "confirm" | "outcome" => 0.62,
                _ => 0.70 + 0.07 * it.ending.min(3) as f32,
            };
            let bar = intake_topbar(ui, pct, !front, true);
            exit |= bar.exit;
            restart |= bar.restart;
            let body = ui.available_rect_before_wrap();
            let wide = ui.ctx().screen_rect().width() >= space::WIDE_FROM;
            if front {
                ui.painter().rect_filled(body, 0.0, tok::NAV_BG);
                glow(ui.painter(), egui::pos2(body.center().x, body.top() + body.height() * 0.18), body.height() * 0.42, tok::AMBER, 0.16);
                let mut child = ui.new_child(egui::UiBuilder::new().max_rect(body));
                self.ij_name(&mut child, p, &mut it, body);
            } else {
                // the rail: 308, or 72 on the wide steps, on the M02.7 clock
                let rail_w = super::motion::tween(ui.ctx(), egui::Id::new("intake-rail-w"), if narrow { space::RAIL_NARROW } else { space::RAIL }, super::motion::timing(420.0, 0.0, super::motion::E_SPRING));
                let rail = egui::Rect::from_min_size(body.min, egui::vec2(rail_w, body.height()));
                intake_rail(ui, rail, super::dashboard::rail_steps(it.mode), step, rail_w < (space::RAIL + space::RAIL_NARROW) / 2.0);
                let pane = egui::Rect::from_min_max(egui::pos2(rail.right(), body.top()), body.max);
                ui.painter().rect_filled(pane, 0.0, p.bg);
                let (col, left, heard_w) = pane_layout(&stage, whole, pane.width(), wide);
                let mut child = ui.new_child(egui::UiBuilder::new().max_rect(pane));
                self.intake_pane_h = pane.height() - space::PANE_TOP - space::S24;
                // M02.2: the new page arrives 400 ms e-spring at 90 ms, from 28 px along
                let reduced = super::motion::reduced();
                let e_step = self.intake_clock(&format!("step_t:{}", it.id));
                let dir: f32 = self.temp.get(&format!("step_dir:{}", it.id)).and_then(|s| s.parse().ok()).unwrap_or(1.0);
                let f_in = e_step.map(|e| super::motion::frame_at(e, super::motion::timing(400.0, 90.0, super::motion::E_SPRING), reduced)).unwrap_or(super::motion::SETTLED);
                if !f_in.done {
                    ui.ctx().request_repaint();
                }
                egui::ScrollArea::vertical().auto_shrink([false, false]).id_salt(("intake-body", &it.id, &stage)).show(&mut child, |ui| {
                    egui::Frame::none().inner_margin(egui::Margin { left, right: space::S24, top: space::PANE_TOP, bottom: space::S24 }).show(ui, |ui| {
                        super::motion::fade_in(ui, f_in, egui::vec2(space::S26 + 2.0, 0.0) * dir, |ui| ui.horizontal_top(|ui| {
                            ui.vertical(|ui| {
                                ui.set_width(col);
                                ui.set_max_width(col);
                                // get the next screen ready while the person reads or types
                                if !busy {
                                    self.ij_precompute(&it);
                                }
                                match stage.as_str() {
                                    "welcome" => self.ij_welcome(ui, p, &mut it),
                                    "bring" => self.ij_bring(ui, p, &mut it),
                                    "questions" => self.ij_question(ui, p, &mut it),
                                    "confirm" => self.ij_confirm(ui, p, &mut it),
                                    "outcome" => self.ij_outcome(ui, p, &mut it),
                                    _ => self.ij_proposal(ui, p, &mut it),
                                }
                            });
                            if stage == "questions" {
                                ui.add_space(space::INTAKE_GAP);
                                ui.vertical(|ui| {
                                    ui.set_width(heard_w);
                                    self.ij_heard(ui, p, &mut it);
                                    if self.testlab && self.resolve_ui.inspector {
                                        ui.add_space(space::S12);
                                        self.ij_inspector(ui, p, &mut it);
                                    }
                                });
                            }
                        }));
                    });
                });
                // M02.1: the page that was showing leaves, 240 ms e-in, 24 px the other way
                if let (Some((mut g, gdir)), Some(e)) = (self.intake_ghost.take(), e_step) {
                    let fo = super::motion::frame_at(e, super::motion::timing(240.0, 0.0, super::motion::E_IN), reduced);
                    if !fo.done {
                        ui.ctx().request_repaint();
                        let focus = ui.memory(|m| m.focused());
                        let focus_qa = self.resolve_ui.focus_qa.clone();
                        self.intake_ghosting = true;
                        let gs = g.stage.clone();
                        let gwhole = gs == "proposal" && g.ending >= 3 && !g.editor && g.limit_at.is_empty();
                        let (gcol, gleft, gheard) = pane_layout(&gs, gwhole, pane.width(), wide);
                        let off = egui::vec2(-(space::S24) * gdir * fo.t, 0.0);
                        let mut gc = ui.new_child(egui::UiBuilder::new().max_rect(pane.translate(off)).layout(Layout::top_down(Align::Min)));
                        gc.set_clip_rect(pane);
                        gc.disable();
                        gc.multiply_opacity(1.0 - fo.alpha);
                        egui::Frame::none().inner_margin(egui::Margin { left: gleft, right: space::S24, top: space::PANE_TOP, bottom: space::S24 }).show(&mut gc, |ui| {
                            ui.horizontal_top(|ui| {
                                ui.vertical(|ui| {
                                    ui.set_width(gcol);
                                    ui.set_max_width(gcol);
                                    match gs.as_str() {
                                        "welcome" => self.ij_welcome(ui, p, &mut g),
                                        "bring" => self.ij_bring(ui, p, &mut g),
                                        "questions" => self.ij_question(ui, p, &mut g),
                                        "confirm" => self.ij_confirm(ui, p, &mut g),
                                        "outcome" => self.ij_outcome(ui, p, &mut g),
                                        "proposal" => self.ij_proposal(ui, p, &mut g),
                                        _ => {}
                                    }
                                });
                                if gs == "questions" {
                                    ui.add_space(space::INTAKE_GAP);
                                    ui.vertical(|ui| {
                                        ui.set_width(gheard);
                                        self.ij_heard(ui, p, &mut g);
                                    });
                                }
                            });
                        });
                        self.intake_ghosting = false;
                        self.resolve_ui.focus_qa = focus_qa;
                        match focus {
                            Some(f) => ui.memory_mut(|m| m.request_focus(f)),
                            None => ui.memory_mut(|m| m.stop_text_input()),
                        }
                        self.intake_ghost = Some((g, gdir));
                    }
                }
                // ST5: welcome back, on the first visit to an intake part-way through
                self.ij_resume(ui, p, &mut it, pane, &mut restart);
            }
            if self.testlab {
                let r = egui::Rect::from_min_size(ui.max_rect().left_bottom() - egui::vec2(0.0, space::S24), egui::vec2(ui.max_rect().width(), space::S24));
                ui.painter().rect_filled(r, 0.0, p.banner_bg);
                ui.painter().text(r.left_center() + egui::vec2(space::S12, 0.0), Align2::LEFT_CENTER, format!("TEST MODE: sandbox data in {} · provider: {}", crate::store::root().display(), if it.provider.is_empty() { "app default".to_string() } else { it.provider.replace('|', " / ") }), fs(size::SMALL), p.banner_text);
            }
        }
        if !busy && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Escape)) && !in_bridge {
            exit = true;
        }

        // a click moved to another page: keep the page as it was, to slide it out (M02.1)
        let end_key = step_key(&it);
        if end_key != start_key && !matches!(start_key.as_str(), "name" | "bridge" | "reveal") && !matches!(end_key.as_str(), "bridge" | "reveal") {
            if let Ok(old) = serde_json::from_str::<Intake>(&before) {
                let dir = if step_order(&end_key) >= step_order(&start_key) { 1.0 } else { -1.0 };
                self.intake_ghost = Some((old, dir));
            }
        }
        // write UI edits back unless a job/adopt replaced the stored intake this frame
        let stored = self.get_intake(id).map(|x| serde_json::to_string(&x).unwrap_or_default()).unwrap_or_default();
        let now = serde_json::to_string(&it).unwrap_or_default();
        if now != before && stored == before {
            it.touch();
            self.put_intake(&it);
            self.touch();
        }
        if restart {
            if let Some(x) = self.intake_mut(id) {
                let name = x.name.clone();
                let provider = x.provider.clone();
                let scenario = x.scenario.clone();
                let fresh = Intake::new();
                *x = Intake { id: x.id.clone(), name, provider, scenario, stage: "welcome".into(), created: x.created, ..fresh };
                x.greeted = true;
            }
            self.temp.insert(format!("resumed:{id}"), "1".into());
            self.resolve_ui.focus_qa = None;
            self.touch();
        }
        if exit {
            self.save_now();
            if self.testlab {
                self.go(View::TestLab);
            } else {
                self.go(View::Dashboard);
            }
        }
    }

    /// Milliseconds since the intake motion clock `key` started (None if it hasn't), or the
    /// moment `--motion-at` holds every clock at (screenshots mid-motion).
    pub(super) fn intake_clock(&self, key: &str) -> Option<f32> {
        let t0: f64 = self.temp.get(key)?.parse().ok()?;
        Some(self.motion_at.unwrap_or(((self.now - t0) * 1000.0) as f32))
    }

    /// Leave room so a step's action bar sits at the foot of the pane (when the step is short).
    pub(super) fn pin_to_foot(&self, ui: &mut egui::Ui, bar_h: f32) {
        let used = ui.min_rect().height();
        let room = self.intake_pane_h - used - bar_h;
        if room > 0.0 {
            ui.add_space(room);
        } else {
            ui.add_space(space::S24);
        }
    }

    /// ST5 Resume (E9): the first time an intake opens in this session part-way through, a card
    /// over the step says what was being done and what is saved.
    fn ij_resume(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut Intake, pane: egui::Rect, restart: &mut bool) {
        let key = format!("resumed:{}", it.id);
        let forced = self.temp.contains_key("force_resume");
        let mid = !matches!(it.stage.as_str(), "name" | "welcome") && (!it.input.trim().is_empty() || !it.qas.is_empty());
        if !mid || self.temp.contains_key(&key) || (self.shot.is_some() && !forced) {
            return;
        }
        let doing = match it.stage.as_str() {
            "bring" => "telling Resolve what's going on with",
            "questions" => "answering Resolve's questions about",
            "confirm" | "outcome" => "checking what Resolve understood about",
            _ => "looking at your next move for",
        };
        let answers = it.qas.iter().filter(|q| !q.answer.trim().is_empty()).count();
        let mut saved: Vec<String> = vec![];
        if answers > 0 {
            saved.push(format!("{answers} answer{}", if answers == 1 { "" } else { "s" }));
        } else if !it.input.trim().is_empty() {
            saved.push("what you brought in".into());
        }
        if it.understanding.is_some() {
            saved.push("the understanding cards".into());
        }
        if it.proposal.is_some() {
            saved.push("your proposal".into());
        }
        let saved = if saved.len() > 1 { format!("{} and {}", saved[..saved.len() - 1].join(", "), saved[saved.len() - 1]) } else { saved.join("") };
        // the project's name once there is one, else the start of what was brought in
        let what = it.proposal.as_ref().map(|x| x.name.clone()).filter(|n| !n.trim().is_empty()).unwrap_or_else(|| crate::resolve::journey::short_title(&it.title(), 8));
        let mut pick = false;
        egui::Area::new(egui::Id::new(("resume-dim", &it.id))).order(egui::Order::Foreground).fixed_pos(pane.min).show(ui.ctx(), |ui| {
            let _ = backdrop(ui, pane, shadow::BACKDROP_PANEL);
        });
        egui::Area::new(egui::Id::new(("resume", &it.id))).order(egui::Order::Foreground).fixed_pos(pane.min + egui::vec2(space::PANE_SIDE, space::S40 * 3.0)).show(ui.ctx(), |ui| {
            dialog_frame(p).inner_margin(egui::Margin::same(space::HERO_PAD_X)).show(ui, |ui| {
                ui.set_width(space::RESUME_W - space::HERO_PAD_X * 2.0);
                ui.spacing_mut().item_spacing.y = space::S8;
                ui.label(RichText::new("Picking up where you left off").font(fs(size::LABEL)).color(p.orange));
                ui.label(RichText::new(format!("Welcome back, {}.", it.name)).font(fserif(size::STEP_TITLE + 2.0)).color(p.text));
                let mut job = egui::text::LayoutJob::default();
                let f = |font: egui::FontId| egui::text::TextFormat { font_id: font, color: p.text, ..Default::default() };
                job.append(&format!("You were {doing} "), 0.0, f(fp(size::ROW + 1.0)));
                job.append(&what, 0.0, f(fs(size::ROW + 1.0)));
                job.append(".", 0.0, f(fp(size::ROW + 1.0)));
                job.wrap.max_width = ui.available_width();
                ui.label(job);
                if !saved.is_empty() {
                    ui.label(RichText::new(format!("Everything you wrote is saved: {saved}.")).size(size::ROW_SMALL).color(p.dim));
                }
                ui.add_space(space::S8);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = space::S16;
                    pick = big_button(ui, p, ButtonKind::Dark, "Pick up where you left off  →", true).clicked();
                    if link(ui, p, "Start this intake again (keeps your name)").clicked() {
                        *restart = true;
                    }
                });
            });
        });
        if pick || *restart {
            self.temp.insert(key, "1".into());
        }
    }
}
