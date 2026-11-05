//! The intake journey screens: full screen, one question at a time (Typeform pattern), then
//! confirm understanding → proposal review → Adopt → bridge → roadmap reveal → the project.
//! Logic lives in `resolve::journey`; this file is layout and input only. Colours, sizes and
//! spacing come from `theme.rs`.

use super::resolve::JobOut;
use super::theme::*;
use super::widgets::*;
use super::*;
use crate::resolve::intake::{self, Intake};
use crate::resolve::journey::{self, Heard, Item};

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
            it.stage = if it.input.trim().is_empty() { "bring".into() } else { "questions".into() };
            if it.name.is_empty() {
                it.stage = "name".into();
            }
        }
        let before = serde_json::to_string(&it).unwrap_or_default();
        let busy = self.resolve_ui.rx.is_some();

        // ---- top strip: mark, progress, Exit / Restart (+ test mode)
        let stage_ix = ["name", "bring", "questions", "confirm", "proposal", "bridge", "reveal"].iter().position(|s| *s == it.stage).unwrap_or(0);
        let pct = match it.stage.as_str() {
            "questions" => 0.30 + 0.35 * it.coverage(),
            _ => [0.08, 0.2, 0.3, 0.7, 0.85, 0.95, 1.0][stage_ix],
        };
        let mut exit = false;
        let mut restart = false;
        egui::Frame::none().fill(p.ink).inner_margin(egui::Margin::symmetric(20.0, 12.0)).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                let (r, _) = ui.allocate_exact_size(egui::vec2(24.0, 24.0), Sense::hover());
                ui.painter().rect_filled(r, space::RADIUS_SMALL, p.accent);
                ui.painter().text(r.center(), Align2::CENTER_CENTER, "R", fserif(15.0), p.on_accent);
                ui.label(RichText::new("Resolve").font(fs(size::SECTION)).color(p.on_ink));
                ui.add_space(14.0);
                let (bar, _) = ui.allocate_exact_size(egui::vec2(320.0, 4.0), Sense::hover());
                ui.painter().rect_filled(bar, 2.0, p.on_ink_soft.linear_multiply(0.25));
                ui.painter().rect_filled(egui::Rect::from_min_size(bar.min, egui::vec2(bar.width() * pct as f32, 4.0)), 2.0, p.accent);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.add(egui::Button::new(RichText::new("Exit").size(size::SMALL).color(p.on_ink_soft)).frame(false)).on_hover_text("Leave the intake — your answers are saved (Esc)").clicked() {
                        exit = true;
                    }
                    if ui.add(egui::Button::new(RichText::new("↻ Restart").size(size::SMALL).color(p.on_ink_soft)).frame(false)).on_hover_text("Start this intake again (keeps your name)").clicked() {
                        restart = true;
                    }
                    if self.testlab {
                        if ui.add(egui::Button::new(RichText::new(if self.resolve_ui.inspector { "Hide inspector" } else { "Inspector" }).size(size::SMALL).color(p.on_ink_soft)).frame(false)).clicked() {
                            self.resolve_ui.inspector = !self.resolve_ui.inspector;
                        }
                    }
                });
            });
        });
        if self.testlab {
            egui::Frame::none().fill(p.banner_bg).inner_margin(egui::Margin::symmetric(20.0, 6.0)).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(RichText::new(format!("TEST MODE — sandbox data in {} · nothing touches your real workspaces · provider: {}", crate::store::root().display(), if it.provider.is_empty() { "app default".to_string() } else { it.provider.replace('|', " / ") })).size(size::SMALL).color(p.banner_text).strong());
            });
        }
        if !busy && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Escape)) {
            exit = true;
        }

        // ---- body
        let side = it.stage == "questions";
        egui::ScrollArea::vertical().auto_shrink([false, false]).id_salt(("intake-body", &it.id)).show(ui, |ui| {
            ui.add_space(36.0);
            let avail = ui.available_width();
            let panel = if side { space::SIDE_PANEL } else { 0.0 };
            let col = space::COLUMN.min(avail - panel - 96.0).max(360.0);
            let pad = ((avail - col - panel - if side { 40.0 } else { 0.0 }) / 2.0).max(24.0);
            ui.horizontal_top(|ui| {
                ui.add_space(pad);
                ui.vertical(|ui| {
                    ui.set_width(col);
                    if busy {
                        card(ui, p, |ui| {
                            ui.horizontal(|ui| {
                                ui.spinner();
                                ui.label(RichText::new(&self.resolve_ui.busy).font(fs(size::SECTION)));
                            });
                            let cfg = journey::cfg_for(&it);
                            ui.label(RichText::new(match cfg { Some(c) => format!("Using {} · {}", c.provider, if c.model.is_empty() { "default model".into() } else { c.model }), None => "No AI — fixed question set".into() }).size(size::TINY).color(p.faint));
                        });
                        return;
                    }
                    if !it.last_error.is_empty() && it.stage != "bridge" {
                        egui::Frame::none().fill(p.callout).rounding(space::RADIUS_SMALL).inner_margin(10.0).show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.label(RichText::new(&it.last_error).size(size::SMALL).color(p.dim));
                        });
                        ui.add_space(12.0);
                    }
                    match it.stage.as_str() {
                        "name" => self.ij_name(ui, p, &mut it),
                        "bring" => self.ij_bring(ui, p, &mut it),
                        "questions" => self.ij_question(ui, p, &mut it),
                        "confirm" => self.ij_confirm(ui, p, &mut it),
                        "proposal" => self.ij_proposal(ui, p, &mut it),
                        "bridge" => self.ij_bridge(ui, p, &mut it),
                        _ => self.ij_reveal(ui, p, &mut it),
                    }
                });
                if side {
                    ui.add_space(40.0);
                    ui.vertical(|ui| {
                        ui.set_width(space::SIDE_PANEL);
                        self.ij_heard(ui, p, &mut it);
                        if self.testlab && self.resolve_ui.inspector {
                            ui.add_space(12.0);
                            self.ij_inspector(ui, p, &mut it);
                        }
                    });
                }
            });
            ui.add_space(40.0);
        });

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
                *x = Intake { id: x.id.clone(), name, provider, scenario, stage: "bring".into(), created: x.created, ..fresh };
                x.greeted = true;
            }
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
}
