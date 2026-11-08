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

        // the bridge holds its 7.2 s even when Adopt has already finished (E14)
        let bridge_key = format!("bridge_t:{}", it.id);
        let bridge_t0 = self.temp.get(&bridge_key).and_then(|s| s.parse::<f64>().ok());
        let in_bridge = it.stage == "bridge" || (it.stage == "reveal" && bridge_t0.map_or(false, |t0| self.now - t0 < BRIDGE_MIN_S && self.shot.is_none()));
        if in_bridge {
            self.ij_bridge(ui, p, &mut it);
        } else if it.stage == "reveal" {
            self.ij_reveal(ui, p, &mut it);
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
                intake_rail(ui, rail, step, rail_w < (space::RAIL + space::RAIL_NARROW) / 2.0);
                let pane = egui::Rect::from_min_max(egui::pos2(rail.right(), body.top()), body.max);
                ui.painter().rect_filled(pane, 0.0, p.bg);
                let side = stage == "questions" && !busy;
                let heard_w = if wide { space::HEARD_W_WIDE } else { space::HEARD_W };
                let room = pane.width() - space::PANE_SIDE * 2.0;
                let col = match stage.as_str() {
                    "questions" => (room - heard_w - space::INTAKE_GAP).min(space::INTAKE_COL),
                    _ if whole => room.min(space::INTAKE_COL_WIDE * 1.3),
                    _ if wide => space::INTAKE_COL_WIDE.min(room),
                    "confirm" => space::INTAKE_COL_CONFIRM.min(room),
                    _ => space::INTAKE_COL.min(room),
                }
                .max(space::S40 * 8.0);
                let total = if stage == "questions" { col + heard_w + space::INTAKE_GAP } else { col };
                // left-aligned under 1600, centred in the pane at 1600 and wider (ST6)
                let left = if wide { ((pane.width() - total) / 2.0).max(space::PANE_SIDE) } else if stage == "questions" || whole { space::PANE_SIDE - space::S8 } else { space::PANE_SIDE };
                let mut child = ui.new_child(egui::UiBuilder::new().max_rect(pane));
                self.intake_pane_h = pane.height() - space::PANE_TOP - space::S24;
                egui::ScrollArea::vertical().auto_shrink([false, false]).id_salt(("intake-body", &it.id, &stage)).show(&mut child, |ui| {
                    egui::Frame::none().inner_margin(egui::Margin { left, right: space::S24, top: space::PANE_TOP, bottom: space::S24 }).show(ui, |ui| {
                        ui.horizontal_top(|ui| {
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
                        });
                    });
                });
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
        let what = it.title();
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
