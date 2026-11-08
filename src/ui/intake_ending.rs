//! The intake's ending (Pack A 06 to 09): 06 next move · 07 roadmap themes one card at a time ·
//! 08 first week · 09 whole picture → Adopt. The full proposal (H0, Hx/NHx, E0, uncertainties,
//! pages) opens from 09 only ("Edit the full proposal"). Plus the proposal's wait (ST3).

use super::intake_steps::{checklist, und_key};
use super::theme::*;
use super::widgets::*;
use super::*;
use crate::resolve::intake::Intake;
use crate::resolve::journey::{self, Item};

const REASSURE: &str = "Nothing here is final. You can change any of it once your project is open.";

/// The basis as a tag: Evidence (green), Assumption (amber), Uncertain (terracotta).
fn basis_tag2(ui: &mut egui::Ui, p: &Pal, b: &str) {
    let (fg, bg, name) = match b {
        "evidence" => (p.green_text, p.green_tint, "Evidence"),
        "assumption" => (p.accent_text, p.accent_soft, "Assumption"),
        _ => (p.orange, p.orange_tint, "Uncertain"),
    };
    tag(ui, name, fg, bg);
}

/// The reassurance line with its pencil.
fn reassure(ui: &mut egui::Ui, p: &Pal) {
    ui.horizontal_top(|ui| {
        let (r, _) = ui.allocate_exact_size(egui::vec2(space::S16, space::S16), Sense::hover());
        paint_icon(ui.painter(), r.center(), Icon::Pencil, p.dim);
        ui.add(egui::Label::new(RichText::new(REASSURE).size(size::ROW_SMALL).color(p.dim)).wrap());
    });
}

/// The first week (08, E27): the first five tasks in roadmap order (theme by theme, Now first).
pub fn first_week(pr: &journey::Proposal) -> Vec<&Item> {
    journey::first_week_ix(pr).into_iter().filter_map(|k| pr.tasks.get(k)).collect()
}

impl App {
    pub(super) fn ij_proposal(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut Intake) {
        if it.limit_at == "proposal" {
            self.ij_limit(ui, p, it);
            return;
        }
        let key = und_key(it);
        let fresh = it.proposal.is_some() && (it.proposed_from == key || it.proposed_from.is_empty());
        if !fresh {
            let pending = self.resolve_ui.bg_propose.as_ref().map_or(false, |(x, k)| x == &it.id && *k == key);
            if self.resolve_ui.rx.is_some() {
                let secs = self.wait_secs();
                self.ij_wait_proposal(ui, p, it, secs);
            } else if pending {
                let started = self.temp.entry("wait:proposal".to_string()).or_insert_with(|| self.now.to_string()).parse::<f64>().unwrap_or(self.now);
                self.ij_wait_proposal(ui, p, it, (self.now - started).max(0.0));
            } else if !it.last_error.is_empty() && it.proposal.is_none() {
                self.ij_error(ui, p, it, "Your next move");
            } else if self.shot.is_none() || it.proposal.is_none() {
                self.ij_spawn_proposal(it, None, true);
            }
            if !(self.shot.is_some() && it.proposal.is_some()) {
                return;
            }
        }
        if it.editor {
            self.ij_editor(ui, p, it);
            return;
        }
        match it.ending {
            0 => self.ie_next_move(ui, p, it),
            1 => self.ie_themes(ui, p, it),
            2 => self.ie_first_week(ui, p, it),
            _ => self.ie_whole_picture(ui, p, it),
        }
    }

    /// ST3: the proposal's wait, centred; after 15 s the long-wait note with Leave and come
    /// back later (E7).
    fn ij_wait_proposal(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut Intake, secs: f64) {
        let id = it.id.clone();
        let lines = checklist(&id, &journey::WAIT_PROPOSAL, &|ev| journey::has_progress(&id, ev), false);
        let (mut back, mut leave) = (false, false);
        ui.add_space(space::S40 * 2.5);
        ui.vertical_centered(|ui| {
            let d = space::S40 + space::S26;
            let (r, _) = ui.allocate_exact_size(egui::vec2(d, d), Sense::hover());
            spinner_ring(ui, r.center(), d / 2.0 - 2.0);
            ui.add_space(space::S22);
            ui.label(RichText::new("Weighing it up…").font(fserif(size::STEP_TITLE + 4.0)).color(p.text));
            ui.label(RichText::new("Writing your plan and your validation side by side.").size(size::ROW + 0.5).color(p.dim));
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

    // ---- 06 next move (E13, E26)
    fn ie_next_move(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut Intake) {
        let Some(pr) = it.proposal.as_mut() else { return };
        ui.label(RichText::new("Your next move").font(fs(size::LABEL)).color(p.orange));
        ui.label(RichText::new("Here's what to do next.").font(fs(size::TITLE + 4.0)).color(p.text));
        ui.add_space(space::S14);
        hero_card(ui, p, |ui| {
            ui.label(RichText::new("Do this first").font(fs(size::SMALL)).color(tok::AMBER));
            ui.add_space(space::S4);
            if pr.next_move.trim().is_empty() {
                ui.add(egui::TextEdit::multiline(&mut pr.next_move).hint_text("Your first move: one thing you can do this week").desired_rows(2).desired_width(f32::INFINITY));
            } else {
                ui.add(egui::Label::new(RichText::new(&pr.next_move).font(fserif(size::CARD_SERIF + 2.0)).color(tok::TEXT_ON_DARK)).wrap());
            }
            if !pr.why.trim().is_empty() {
                ui.add_space(space::S8);
                ui.add(egui::Label::new(RichText::new(&pr.why).size(size::ROW + 0.5).color(tok::TEXT_ON_DARK_SOFT)).wrap());
            }
        });
        ui.add_space(space::S22);
        ui.label(RichText::new("What happens after this").font(fs(size::ROW_SMALL)).color(p.text));
        ui.add_space(space::S8);
        let w = (ui.available_width() - space::S10 * 2.0) / 3.0;
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = space::S10;
            for phase in ["Now", "Next", "Later"] {
                let first = pr.roadmap.iter().find(|r| r.phase == phase || (phase == "Now" && r.phase.is_empty())).map(|r| r.title.clone());
                let now = phase == "Now";
                egui::Frame::none().fill(p.card).stroke(Stroke::new(stroke::HAIR, if now { tok::AMBER } else { p.line })).rounding(egui::Rounding::same(radius::CARD)).inner_margin(egui::Margin::symmetric(space::S16, space::S14)).show(ui, |ui| {
                    ui.set_width(w - space::S16 * 2.0);
                    ui.set_min_height(space::S40 + space::S4);
                    ui.vertical(|ui| {
                    ui.label(RichText::new(phase).font(fs(size::SMALL)).color(if now { p.orange } else { p.dim }));
                    ui.add(egui::Label::new(RichText::new(first.unwrap_or_else(|| "Nothing yet".into())).font(fs(size::ROW + 0.5)).color(p.text)).wrap());
                    });
                });
            }
        });
        ui.add_space(space::S14);
        reassure(ui, p);
        let ok = !pr.next_move.trim().is_empty();
        self.pin_to_foot(ui, space::BUTTON_H_LARGE + space::S40);
        let (mut back, mut go, mut skip) = (false, false, false);
        action_bar(ui, p, |ui| {
            back = big_button(ui, p, ButtonKind::Secondary, "Back", true).clicked();
            go = big_button(ui, p, ButtonKind::Dark, "Continue  →", ok).clicked();
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                skip = link(ui, p, "Skip to the whole picture →").clicked();
            });
        });
        if back {
            it.stage = "confirm".into();
        } else if go {
            it.ending = 1;
            it.theme_ix = 0;
        } else if skip {
            it.ending = 3;
        }
    }

    // ---- 07 roadmap themes, one card at a time (E12)
    fn ie_themes(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut Intake) {
        let Some(pr) = it.proposal.as_mut() else { return };
        let n = pr.roadmap.len();
        ui.label(RichText::new("Building your roadmap").font(fs(size::LABEL)).color(p.orange));
        ui.add(egui::Label::new(RichText::new("Does this belong on your roadmap?").font(fserif(size::STEP_TITLE + 2.0)).color(p.text)).wrap());
        ui.add_space(space::S8);
        let (mut skip, mut back) = (false, false);
        if n == 0 {
            ui.add_space(space::S10);
            ui.label(RichText::new("There are no roadmap themes yet. Add one, or carry on and add them later.").size(size::ROW_SMALL).color(p.dim));
            ui.add_space(space::S10);
            ui.horizontal(|ui| {
                if big_button(ui, p, ButtonKind::Secondary, "+ Add a theme", true).clicked() {
                    pr.roadmap.push(Item { title: String::new(), phase: "Now".into(), basis: "uncertain".into(), ..Default::default() });
                    self.resolve_ui.theme_edit = true;
                }
                if big_button(ui, p, ButtonKind::Dark, "Continue  →", true).clicked() {
                    it.ending = 2;
                }
            });
            return;
        }
        if it.theme_ix >= n {
            it.ending = 2;
            return;
        }
        let k = it.theme_ix;
        let card_w = (ui.available_width() * 0.62).min(space::S40 * 13.0);
        segment_bar(ui, p, n, k, card_w);
        ui.add_space(space::S18);
        let editing = self.resolve_ui.theme_edit;
        let (mut yes, mut not_quite, mut save, mut drop) = (false, false, false, false);
        let confirmed: Vec<String> = pr.roadmap.iter().take(k).filter(|t| t.confirmed).map(|t| t.title.clone()).collect();
        let more = n - k - 1;
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                ui.set_width(card_w);
                // the card, with the next one peeking out behind it
                let top = ui.cursor().min;
                let behind_at = ui.painter().add(egui::Shape::Noop);
                let t = &mut pr.roadmap[k];
                let inner = egui::Frame::none().fill(p.card).stroke(Stroke::new(stroke::HAIR, if editing { tok::AMBER } else { p.line })).rounding(egui::Rounding::same(radius::DIALOG)).inner_margin(egui::Margin::same(space::S24)).show(ui, |ui| {
                    ui.set_width(card_w - space::S24 * 2.0);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = space::S8;
                        ui.label(RichText::new(format!("Theme {} of {n}", k + 1)).size(size::ROW_SMALL).color(p.dim));
                        phase_tag(ui, p, if t.phase.is_empty() { "Now" } else { &t.phase });
                        basis_tag2(ui, p, &t.basis);
                    });
                    ui.add_space(space::S8);
                    if editing {
                        ui.add(egui::TextEdit::singleline(&mut t.title).font(fs(size::SECTION)).desired_width(f32::INFINITY).hint_text("Theme (a few words)"));
                        ui.add(egui::TextEdit::multiline(&mut t.detail).desired_rows(2).desired_width(f32::INFINITY).hint_text("What it involves"));
                        ui.horizontal(|ui| {
                            for ph in ["Now", "Next", "Later"] {
                                if tab_chip(ui, p, ph, None, t.phase == ph).clicked() {
                                    t.phase = ph.into();
                                }
                            }
                        });
                    } else {
                        ui.add(egui::Label::new(RichText::new(&t.title).font(fserif(size::CARD_SERIF + 2.0)).color(p.text)).wrap());
                        if !t.detail.is_empty() {
                            ui.add_space(space::S4);
                            ui.add(egui::Label::new(RichText::new(&t.detail).size(size::ROW + 0.5).color(p.dim)).wrap());
                        }
                    }
                    ui.add_space(space::S16);
                    let bw = (ui.available_width() - space::S10) / 2.0;
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = space::S10;
                        ui.allocate_ui_with_layout(egui::vec2(bw, space::BUTTON_H_LARGE + 4.0), Layout::top_down_justified(Align::Center), |ui| {
                            if editing {
                                save = wide_button(ui, p, ButtonKind::Dark, "Save and continue", !t.title.trim().is_empty()).clicked();
                            } else {
                                yes = wide_button(ui, p, ButtonKind::Primary, "Yes, add to my roadmap", true).clicked();
                            }
                        });
                        ui.allocate_ui_with_layout(egui::vec2(bw, space::BUTTON_H_LARGE + 4.0), Layout::top_down_justified(Align::Center), |ui| {
                            if editing {
                                drop = wide_button(ui, p, ButtonKind::Secondary, "Leave it off the roadmap", true).clicked();
                            } else {
                                not_quite = wide_button(ui, p, ButtonKind::Secondary, "Not quite", true).clicked();
                            }
                        });
                    });
                    if yes || save {
                        t.confirmed = true;
                    }
                });
                let r = inner.response.rect;
                let behind = r.translate(egui::vec2(space::S10, space::S10));
                let mut shapes: Vec<egui::Shape> = vec![];
                if !p.dark {
                    shapes.extend(shadow::CARD_LIFT.iter().map(|s| egui::Shape::Rect(s.as_shape(r, egui::Rounding::same(radius::DIALOG)))));
                }
                if more > 0 {
                    shapes.push(egui::Shape::rect_filled(behind, radius::DIALOG, p.card));
                    shapes.push(egui::Shape::rect_stroke(behind, radius::DIALOG, Stroke::new(stroke::HAIR, p.line)));
                }
                ui.painter().set(behind_at, egui::Shape::Vec(shapes));
                let _ = top;
            });
            ui.add_space(space::INTAKE_GAP);
            ui.vertical(|ui| {
                ui.add_space(space::S22);
                ui.label(RichText::new("On your roadmap").font(fs(size::ROW_SMALL)).color(p.text));
                ui.add_space(space::S8);
                if confirmed.is_empty() {
                    let w = ui.available_width();
                    let top = ui.cursor().min;
                    let bg_at = ui.painter().add(egui::Shape::Noop);
                    let inner = egui::Frame::none().inner_margin(egui::Margin::symmetric(space::S16, space::S16)).show(ui, |ui| {
                        ui.set_width(w - space::S16 * 2.0);
                        ui.add(egui::Label::new(RichText::new("Nothing yet. Each theme you say yes to lands here, in order.").size(size::ROW_SMALL).color(p.dim)).wrap());
                    });
                    let rr = egui::Rect::from_min_size(top, egui::vec2(w, inner.response.rect.height()));
                    let c = [rr.left_top(), rr.right_top(), rr.right_bottom(), rr.left_bottom(), rr.left_top()];
                    let mut shapes = vec![egui::Shape::rect_filled(rr, radius::CARD, tok::AMBER_LIGHT)];
                    shapes.extend(egui::Shape::dashed_line(&c, Stroke::new(stroke::HAIR, tok::AMBER_EDGE), 4.0, 3.0));
                    ui.painter().set(bg_at, egui::Shape::Vec(shapes));
                } else {
                    for (i, t) in confirmed.iter().enumerate() {
                        ui.horizontal(|ui| {
                            let (r, _) = ui.allocate_exact_size(egui::vec2(space::S18, space::S18), Sense::hover());
                            ui.painter().circle_filled(r.center(), space::S18 / 2.0, tok::AMBER);
                            paint_icon(ui.painter(), r.center(), Icon::Check, tok::INK);
                            ui.label(RichText::new(format!("{}. {t}", i + 1)).size(size::ROW_SMALL).color(p.text));
                        });
                        ui.add_space(space::S4);
                    }
                }
                ui.add_space(space::S10);
                if more > 0 {
                    ui.label(RichText::new(format!("{more} more theme{} after this one.", if more == 1 { "" } else { "s" })).size(size::ROW_SMALL).color(p.dim));
                    ui.add_space(space::S14);
                }
                reassure(ui, p);
            });
        });
        self.pin_to_foot(ui, space::S40);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = space::S16;
            back = ui.add(egui::Label::new(RichText::new("← Back").font(fs(size::LABEL)).color(p.dim)).sense(Sense::click())).on_hover_cursor(egui::CursorIcon::PointingHand).clicked();
            skip = link(ui, p, "Skip to the whole picture →").clicked();
        });
        if not_quite {
            self.resolve_ui.theme_edit = true;
        } else if yes || save {
            self.resolve_ui.theme_edit = false;
            it.theme_ix += 1;
        } else if drop {
            self.resolve_ui.theme_edit = false;
            let gone = pr.roadmap.remove(k);
            // kept on the project's cut list (Knowledge → Cut list), not lost
            if !gone.title.trim().is_empty() {
                pr.cut.push(gone.title);
            }
            // tasks pointing at later themes shift down
            for t in pr.tasks.iter_mut() {
                t.parent = match t.parent {
                    Some(x) if x == k => None,
                    Some(x) if x > k => Some(x - 1),
                    o => o,
                };
            }
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

    // ---- 08 first week (E27)
    fn ie_first_week(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut Intake) {
        let Some(pr) = it.proposal.as_ref() else { return };
        step_head(ui, p, "Your first week", "Here's where to begin.", "Built straight from your roadmap. The first concrete moves, in the order they matter.");
        ui.add_space(space::S14);
        let rows = first_week(pr);
        if rows.is_empty() {
            ui.label(RichText::new("No first tasks yet. Add them in the full proposal, or after you adopt.").size(size::ROW_SMALL).color(p.dim));
        } else {
            list_box(ui, p, |ui| {
                for (n, t) in rows.iter().enumerate() {
                    let up = n == 0;
                    let top = ui.cursor().min;
                    let bg_at = ui.painter().add(egui::Shape::Noop);
                    let w = ui.available_width();
                    let inner = egui::Frame::none().inner_margin(egui::Margin::symmetric(space::S18, space::S14)).show(ui, |ui| {
                        ui.set_width(w - space::S18 * 2.0);
                        ui.horizontal_top(|ui| {
                            let (r, _) = ui.allocate_exact_size(egui::vec2(space::S26, space::S26), Sense::hover());
                            ui.painter().circle_stroke(r.center(), space::S26 / 2.0 - 1.0, Stroke::new(stroke::HAIR, if up { tok::AMBER } else { p.line }));
                            ui.painter().text(r.center(), Align2::CENTER_CENTER, (n + 1).to_string(), fs(size::SMALL), if up { tok::AMBER } else { p.text });
                            ui.add_space(space::S12);
                            ui.vertical(|ui| {
                                if up {
                                    ui.label(RichText::new("Up next").font(fs(size::TAG + 0.5)).color(tok::AMBER));
                                }
                                ui.add(egui::Label::new(RichText::new(&t.title).font(fs(size::ROW + 0.5)).color(if up { tok::TEXT_ON_DARK } else { p.text })).wrap());
                                if !t.detail.is_empty() {
                                    ui.add(egui::Label::new(RichText::new(&t.detail).size(size::ROW_SMALL).color(if up { tok::TEXT_ON_DARK_SOFT } else { p.dim })).wrap());
                                }
                            });
                        });
                    });
                    let rect = egui::Rect::from_min_size(top, egui::vec2(w, inner.response.rect.height()));
                    if up {
                        let rr = if rows.len() == 1 { egui::Rounding::same(radius::CARD) } else { egui::Rounding { nw: radius::CARD, ne: radius::CARD, sw: 0.0, se: 0.0 } };
                        ui.painter().set(bg_at, egui::Shape::rect_filled(rect, rr, tok::INK));
                    } else {
                        ui.painter().line_segment([rect.left_top(), rect.right_top()], Stroke::new(stroke::HAIR, p.divider));
                    }
                }
            });
        }
        ui.add_space(space::S10);
        reassure(ui, p);
        self.pin_to_foot(ui, space::BUTTON_H_LARGE + space::S40);
        let (mut back, mut go, mut skip) = (false, false, false);
        action_bar(ui, p, |ui| {
            back = big_button(ui, p, ButtonKind::Secondary, "Back", true).clicked();
            go = big_button(ui, p, ButtonKind::Dark, "See it as a roadmap  →", true).clicked();
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                skip = link(ui, p, "Skip to the whole picture →").clicked();
            });
        });
        if back {
            it.ending = 1;
            it.theme_ix = it.proposal.as_ref().map_or(0, |x| x.roadmap.len().saturating_sub(1));
        } else if go || skip {
            it.ending = 3;
        }
    }

    // ---- 09 whole picture → Adopt (E19, E26)
    fn ie_whole_picture(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut Intake) {
        let Some(pr) = it.proposal.clone() else { return };
        // the reveal runs on one clock; any click finishes it (M07, E19)
        let key = format!("whole_t:{}", it.id);
        let t0: f64 = self.temp.entry(key.clone()).or_insert_with(|| self.now.to_string()).parse().unwrap_or(self.now);
        let skipped = self.temp.contains_key(&format!("whole_skip:{}", it.id)) || self.shot.is_some();
        if !skipped && ui.input(|i| i.pointer.any_click()) {
            self.temp.insert(format!("whole_skip:{}", it.id), "1".into());
        }
        let el = if skipped { 1.0e6 } else { ((self.now - t0) * 1000.0) as f32 };
        let reduced = super::motion::reduced();
        let f = |dur: f32, delay: f32| super::motion::frame_at(el, super::motion::timing(dur, delay, super::motion::EASE), reduced);
        if el < 3800.0 && !reduced {
            ui.ctx().request_repaint();
        }
        let head_top = ui.cursor().min;
        let full_w = ui.available_width();
        ui.label(RichText::new("Your roadmap").font(fs(size::LABEL)).color(p.orange));
        ui.label(RichText::new("Here's the whole picture.").font(fserif(size::STEP_TITLE + 4.0)).color(p.text));
        // the project name, at the right of the title
        let fw = space::S40 * 6.5;
        let name_rect = egui::Rect::from_min_size(egui::pos2(head_top.x + full_w - fw - space::S40 * 2.4, head_top.y + space::S18), egui::vec2(fw + space::S40 * 2.4, space::S40 + space::S4));
        let mut nc = ui.new_child(egui::UiBuilder::new().max_rect(name_rect).layout(Layout::right_to_left(Align::Center)));
        if let Some(x) = it.proposal.as_mut() {
            text_field(&mut nc, p, egui::Id::new(("project-name", &it.id)), &mut x.name, "Project name", false, fw);
        }
        nc.add_space(space::S10);
        nc.label(RichText::new("Project name").size(size::ROW_SMALL).color(p.dim));
        if !it.last_error.is_empty() {
            // ST4 for step 10: the build failed and was rolled back
            ui.add_space(space::S10);
            error_panel(ui, p, &it.last_error, "Nothing you wrote is lost. Adopt again, or edit the full proposal first.");
        }
        ui.add_space(space::S16);
        let w = (ui.available_width() - space::S16 * 2.0) / 3.0;
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = space::S16;
            for (c, phase) in ["Now", "Next", "Later"].iter().enumerate() {
                ui.vertical(|ui| {
                    ui.set_width(w);
                    super::motion::fade_in(ui, f(300.0, 200.0 + 150.0 * c as f32), egui::vec2(0.0, space::S6), |ui| {
                        lane_head(ui, p, phase, c == 0);
                    });
                    let mine: Vec<(usize, &Item)> = pr.roadmap.iter().enumerate().filter(|(_, r)| r.phase == *phase || (r.phase.is_empty() && c == 0)).collect();
                    super::motion::fade_in(ui, f(500.0, 1400.0 + 600.0 * c as f32), egui::vec2(0.0, space::S10), |ui| {
                        if mine.is_empty() {
                            ui.label(RichText::new("Nothing yet").size(size::ROW_SMALL).color(p.dim));
                        }
                        for (n, (ix, r)) in mine.iter().enumerate() {
                            egui::Frame::none().fill(p.card).stroke(Stroke::new(stroke::HAIR, if c == 0 && n == 0 { tok::AMBER } else { p.line })).rounding(egui::Rounding::same(radius::CARD)).inner_margin(egui::Margin::same(space::S16)).show(ui, |ui| {
                                ui.set_width(w - space::S16 * 2.0);
                                ui.add(egui::Label::new(RichText::new(&r.title).font(fs(size::ROW + 1.0)).color(p.text)).wrap());
                                ui.add_space(space::S6);
                                for t in pr.tasks.iter().filter(|t| t.parent == Some(*ix)) {
                                    ui.horizontal_top(|ui| {
                                        let (b, _) = ui.allocate_exact_size(egui::vec2(space::S14, space::S14), Sense::hover());
                                        ui.painter().rect_stroke(b.shrink(1.0), radius::CHIP / 2.0, Stroke::new(stroke::HAIR, p.faint));
                                        ui.add(egui::Label::new(RichText::new(&t.title).size(size::ROW_SMALL).color(p.text)).wrap());
                                    });
                                    ui.add_space(space::S4);
                                }
                                ui.add_space(space::S4);
                                basis_tag2(ui, p, &r.basis);
                            });
                            ui.add_space(space::S10);
                        }
                    });
                });
            }
        });
        let ok = !pr.name.trim().is_empty();
        self.pin_to_foot(ui, space::BUTTON_H_LARGE + space::S40);
        let (mut back, mut adopt, mut editor) = (false, false, false);
        super::motion::fade_in(ui, f(480.0, 3300.0), egui::vec2(0.0, space::S10), |ui| {
            action_bar(ui, p, |ui| {
                back = big_button(ui, p, ButtonKind::Secondary, "Back", true).clicked();
                adopt = big_button(ui, p, ButtonKind::Primary, "Adopt, create this project  →", ok).clicked();
                ui.add(egui::Label::new(RichText::new(if it.new_workspace { "Adopt creates a new workspace with this as its first project. You can undo it." } else { "Adopt creates the project with its roadmap, tasks, validation and pages. You can undo it." }).size(size::ROW_SMALL).color(p.dim)).wrap());
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    editor = link(ui, p, "Edit the full proposal →").on_hover_text("H0 and hypotheses, experiments, uncertainties, knowledge pages").clicked();
                });
            });
        });
        if back {
            it.ending = 2;
        } else if adopt && ok {
            it.last_error.clear();
            self.temp.insert(format!("bridge_t:{}", it.id), self.now.to_string());
            self.adopt_begin(it);
        } else if editor {
            it.editor = true;
        }
    }
}
