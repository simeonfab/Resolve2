//! The intake's ending, presented like the original Resolve (reference pictures 05–09):
//! 1 next move · 2 roadmap themes confirmed one card at a time · 3 first week · 4 whole picture → Adopt.
//! The full proposal (H0, Hx/NHx, E0, uncertainties, pages) stays one click away in the editor.

use super::intake_steps::und_key;
use super::theme::*;
use super::widgets::*;
use super::*;
use crate::resolve::intake::Intake;
use crate::resolve::journey::Item;

fn cta(p: &Pal, text: &str) -> egui::Button<'static> {
    egui::Button::new(RichText::new(text.to_string()).font(fs(size::BODY)).color(p.on_ink)).fill(p.ink).rounding(space::RADIUS_SMALL).min_size(egui::vec2(0.0, 34.0))
}

fn eyebrow(ui: &mut egui::Ui, p: &Pal, text: &str) {
    ui.label(RichText::new(text).size(size::SMALL).color(p.eyebrow).strong());
}

fn basis_chip(ui: &mut egui::Ui, p: &Pal, b: &str) {
    let col = match b {
        "evidence" => p.green,
        "assumption" => p.accent,
        _ => p.orange,
    };
    chip(ui, if b.is_empty() { "uncertain" } else { b }, col);
}

impl App {
    pub(super) fn ij_proposal(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut Intake) {
        let key = und_key(it);
        let fresh = it.proposal.is_some() && (it.proposed_from == key || it.proposed_from.is_empty());
        if !fresh {
            let pending = self.resolve_ui.bg_propose.as_ref().map_or(false, |(x, k)| x == &it.id && *k == key);
            if pending {
                self.ij_waiting(ui, p, "Writing your plan and your validation side by side…", "proposal");
            } else if !it.last_error.is_empty() && it.proposal.is_none() {
                if ui.add(cta(p, "Try again")).clicked() {
                    it.last_error.clear();
                    self.ij_spawn_proposal_pub(it, None);
                }
            } else {
                self.ij_spawn_proposal_pub(it, None);
            }
            return;
        }
        if it.editor {
            self.ij_editor(ui, p, it);
            return;
        }
        let mut open_editor = false;
        match it.ending {
            0 => self.ie_next_move(ui, p, it),
            1 => self.ie_themes(ui, p, it),
            2 => self.ie_first_week(ui, p, it),
            _ => self.ie_whole_picture(ui, p, it),
        }
        ui.add_space(16.0);
        ui.horizontal(|ui| {
            if it.ending > 0 && ui.button("Back").clicked() {
                it.ending -= 1;
                if it.ending == 1 {
                    it.theme_ix = 0;
                }
            }
            if ui.add(egui::Label::new(RichText::new("See and edit everything →").size(size::SMALL).color(p.dim)).sense(Sense::click())).on_hover_text("H0 and hypotheses, experiments, uncertainties, knowledge pages").clicked() {
                open_editor = true;
            }
        });
        if open_editor {
            it.editor = true;
        }
    }

    // ---- 1. next move
    fn ie_next_move(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut Intake) {
        let Some(pr) = it.proposal.as_mut() else { return };
        eyebrow(ui, p, "Your next move");
        ui.label(RichText::new("Here's what to do next.").font(fserif(size::HERO)));
        ui.add_space(10.0);
        egui::Frame::none().fill(p.ink).rounding(space::RADIUS).inner_margin(space::CARD_PAD + 4.0).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.vertical(|ui| {
                ui.label(RichText::new("Do this first").size(size::SMALL).color(p.eyebrow).strong());
                if pr.next_move.trim().is_empty() {
                    ui.add(egui::TextEdit::multiline(&mut pr.next_move).hint_text("Your first move — one thing you can do this week").desired_rows(2).desired_width(f32::INFINITY));
                } else {
                    wrap_label(ui, RichText::new(&pr.next_move).font(fserif(size::TITLE)).color(p.on_ink));
                }
                if !pr.why.trim().is_empty() {
                    ui.add_space(6.0);
                    wrap_label(ui, RichText::new(&pr.why).size(size::BODY).color(p.on_ink_soft));
                }
            });
        });
        ui.add_space(14.0);
        let ok = !pr.next_move.trim().is_empty();
        if ui.add_enabled(ok, cta(p, "Continue  →")).clicked() {
            it.ending = 1;
            it.theme_ix = 0;
        }
    }

    // ---- 2. roadmap themes, one card at a time
    fn ie_themes(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut Intake) {
        let Some(pr) = it.proposal.as_mut() else { return };
        let n = pr.roadmap.len();
        eyebrow(ui, p, "Your roadmap");
        ui.label(RichText::new("Does this belong on your roadmap?").font(fserif(size::HERO)));
        if n == 0 {
            ui.label(RichText::new("There are no roadmap themes yet. Add one, or carry on and add them later.").size(size::SMALL).color(p.dim));
            if ui.button("+ Add a theme").clicked() {
                pr.roadmap.push(Item { title: String::new(), phase: "Now".into(), basis: "uncertain".into(), ..Default::default() });
                self.resolve_ui.theme_edit = true;
            }
            if ui.add(cta(p, "Continue  →")).clicked() {
                it.ending = 2;
            }
            return;
        }
        if it.theme_ix >= n {
            it.ending = 2;
            return;
        }
        let k = it.theme_ix;
        ui.label(RichText::new(format!("Theme {} of {n}", k + 1)).size(size::SMALL).color(p.faint));
        ui.add_space(8.0);
        let editing = self.resolve_ui.theme_edit;
        let (mut yes, mut not_quite, mut save, mut drop) = (false, false, false, false);
        {
            let t = &mut pr.roadmap[k];
            egui::Frame::none().fill(p.card).stroke(Stroke::new(1.0, if editing { p.accent } else { p.line })).rounding(space::RADIUS).inner_margin(space::CARD_PAD).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.vertical(|ui| {
                    ui.horizontal_wrapped(|ui| {
                        chip(ui, if t.phase.is_empty() { "Now" } else { &t.phase }, p.accent);
                        basis_chip(ui, p, &t.basis);
                    });
                    if editing {
                        ui.add(egui::TextEdit::singleline(&mut t.title).font(fs(size::SECTION)).desired_width(f32::INFINITY).hint_text("Theme — a few words"));
                        ui.add(egui::TextEdit::multiline(&mut t.detail).desired_rows(2).desired_width(f32::INFINITY).hint_text("What it involves"));
                        ui.horizontal(|ui| {
                            for ph in ["Now", "Next", "Later"] {
                                if pill(ui, p, ph, t.phase == ph).clicked() {
                                    t.phase = ph.into();
                                }
                            }
                        });
                    } else {
                        wrap_label(ui, RichText::new(&t.title).font(fserif(size::TITLE)));
                        if !t.detail.is_empty() {
                            wrap_label(ui, RichText::new(&t.detail).size(size::BODY).color(p.dim));
                        }
                    }
                });
            });
            ui.add_space(12.0);
            ui.horizontal_wrapped(|ui| {
                if editing {
                    save = ui.add_enabled(!t.title.trim().is_empty(), cta(p, "Save and continue")).clicked();
                    drop = ui.button("Leave it off the roadmap").clicked();
                } else {
                    yes = ui.add(cta(p, "Yes, add to my roadmap")).clicked();
                    not_quite = ui.button("Not quite").clicked();
                }
            });
            if yes || save {
                t.confirmed = true;
            }
        }
        if not_quite {
            self.resolve_ui.theme_edit = true;
        } else if yes || save {
            self.resolve_ui.theme_edit = false;
            it.theme_ix += 1;
        } else if drop {
            self.resolve_ui.theme_edit = false;
            pr.roadmap.remove(k);
            // tasks pointing at later themes shift down
            for t in pr.tasks.iter_mut() {
                t.parent = match t.parent {
                    Some(x) if x == k => None,
                    Some(x) if x > k => Some(x - 1),
                    o => o,
                };
            }
        }
    }

    // ---- 3. first week
    fn ie_first_week(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut Intake) {
        let Some(pr) = it.proposal.as_ref() else { return };
        eyebrow(ui, p, "Your first week");
        ui.label(RichText::new("Here's where to begin.").font(fserif(size::HERO)));
        ui.add_space(10.0);
        let now: Vec<&Item> = pr.tasks.iter().filter(|t| t.phase == "Now" || t.phase.is_empty()).collect();
        if now.is_empty() {
            ui.label(RichText::new("No first tasks yet — add them in \"See and edit everything\", or after you adopt.").size(size::SMALL).color(p.dim));
        }
        for (n, t) in now.iter().enumerate() {
            egui::Frame::none().fill(if n == 0 { p.ink } else { p.card }).stroke(Stroke::new(1.0, p.line)).rounding(space::RADIUS).inner_margin(12.0).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.vertical(|ui| {
                    let (fg, soft) = if n == 0 { (p.on_ink, p.on_ink_soft) } else { (p.text, p.dim) };
                    wrap_label(ui, RichText::new(format!("{}. {}", n + 1, t.title)).font(fs(size::SECTION)).color(fg));
                    if !t.detail.is_empty() {
                        wrap_label(ui, RichText::new(&t.detail).size(size::SMALL).color(soft));
                    }
                });
            });
            ui.add_space(6.0);
        }
        ui.add_space(8.0);
        if ui.add(cta(p, "See it as a roadmap  →")).clicked() {
            it.ending = 3;
        }
    }

    // ---- 4. whole picture → Adopt
    fn ie_whole_picture(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut Intake) {
        let Some(pr) = it.proposal.clone() else { return };
        eyebrow(ui, p, "Your roadmap");
        ui.label(RichText::new("Here's the whole picture.").font(fserif(size::HERO)));
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Project name").size(size::SMALL).color(p.dim));
            if let Some(x) = it.proposal.as_mut() {
                ui.add(egui::TextEdit::singleline(&mut x.name).font(fs(size::SECTION)).desired_width(280.0));
            }
        });
        ui.add_space(10.0);
        ui.columns(3, |cols| {
            for (c, phase) in ["Now", "Next", "Later"].iter().enumerate() {
                cols[c].vertical(|ui| {
                    ui.label(RichText::new(*phase).font(fs(size::SECTION)).color(if c == 0 { p.accent } else { p.dim }));
                    let mine: Vec<(usize, &Item)> = pr.roadmap.iter().enumerate().filter(|(_, r)| r.phase == *phase || (r.phase.is_empty() && c == 0)).collect();
                    if mine.is_empty() {
                        egui::Frame::none().stroke(Stroke::new(1.0, p.line)).rounding(space::RADIUS).inner_margin(12.0).show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.label(RichText::new("Nothing yet").size(size::SMALL).color(p.faint));
                        });
                    }
                    for (ix, r) in mine {
                        egui::Frame::none().fill(p.card).stroke(Stroke::new(1.0, if c == 0 { p.accent } else { p.line })).rounding(space::RADIUS).inner_margin(12.0).show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.vertical(|ui| {
                                wrap_label(ui, RichText::new(&r.title).font(fs(size::BODY)));
                                for t in pr.tasks.iter().filter(|t| t.parent == Some(ix)) {
                                    wrap_label(ui, RichText::new(format!("☐ {}", t.title)).size(size::SMALL).color(p.dim));
                                }
                                basis_chip(ui, p, &r.basis);
                            });
                        });
                        ui.add_space(6.0);
                    }
                });
            }
        });
        ui.add_space(14.0);
        let ok = !pr.name.trim().is_empty();
        if ui.add_enabled(ok, cta(p, "Adopt — create this project")).clicked() {
            self.adopt_begin(it);
        }
        ui.label(RichText::new(if it.new_workspace { "Adopt creates a new workspace with this as its first project. You can undo it." } else { "Adopt creates the project with its roadmap, tasks, validation and pages. You can undo it." }).size(size::TINY).color(p.faint));
    }
}
