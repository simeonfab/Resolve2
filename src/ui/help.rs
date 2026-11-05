use super::theme::*;
use super::widgets::*;
use super::*;
use eframe::egui::{self, RichText};

/// The written tutorial, shipped inside the program.
pub const TUTORIAL: &str = include_str!("../../TUTORIAL.md");

impl App {
    pub fn help_view(&mut self, ui: &mut egui::Ui, p: &Pal) {
        self.top_bar(ui, p, |app, ui| {
            if ui.button("Copy as text").clicked() {
                ui.ctx().output_mut(|o| o.copied_text = TUTORIAL.to_string());
                app.toast("Tutorial copied");
            }
        });
        let (title, blocks) = markdown_to_blocks(TUTORIAL);
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            centered(ui, 760.0, |ui| {
                ui.add_space(24.0);
                ui.label(RichText::new("❓").size(44.0));
                ui.label(RichText::new(title.unwrap_or_else(|| "Tutorial".into())).font(fb(36.0)));
                ui.add_space(12.0);
                let mut number = 0;
                for b in &blocks {
                    number = if b.kind == BlockKind::Numbered { number + 1 } else { 0 };
                    let (size, bold) = match b.kind {
                        BlockKind::H1 => (26.0, true),
                        BlockKind::H2 => (20.0, true),
                        BlockKind::H3 => (17.0, true),
                        _ => (15.0, false),
                    };
                    let job = super::pages::inline_job(&b.text, size, bold, p.text, p, false, false, false);
                    match b.kind {
                        BlockKind::H1 | BlockKind::H2 => {
                            ui.add_space(18.0);
                            ui.label(job);
                            ui.add_space(2.0);
                        }
                        BlockKind::H3 => {
                            ui.add_space(10.0);
                            ui.label(job);
                        }
                        BlockKind::Bullet | BlockKind::Todo => {
                            ui.horizontal_top(|ui| {
                                ui.label(RichText::new("  •").size(15.0));
                                ui.add(egui::Label::new(job).wrap());
                            });
                        }
                        BlockKind::Numbered => {
                            ui.horizontal_top(|ui| {
                                ui.label(RichText::new(format!("  {number}.")).size(15.0));
                                ui.add(egui::Label::new(job).wrap());
                            });
                        }
                        BlockKind::Callout | BlockKind::Quote => {
                            egui::Frame::none().fill(p.callout).rounding(6.0).inner_margin(egui::Margin::symmetric(16.0, 12.0)).show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.add(egui::Label::new(job).wrap());
                            });
                        }
                        BlockKind::Code => {
                            egui::Frame::none().fill(p.code).rounding(6.0).inner_margin(egui::Margin::same(10.0)).show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.label(RichText::new(&b.text).monospace());
                            });
                        }
                        BlockKind::Divider => {
                            ui.separator();
                        }
                        _ => {
                            ui.add_space(4.0);
                            ui.add(egui::Label::new(job).wrap());
                        }
                    }
                }
                ui.add_space(60.0);
            });
        });
    }
}
