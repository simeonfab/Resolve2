//! Resolve Intake screens (ported from ResolveOS-intake-frontend public/index.html).

use super::resolve::{amber, JobOut};
use super::theme::*;
use super::widgets::*;
use super::*;
use crate::resolve::intake::{self, Intake, NotYet, Phase, Understanding};

const INK: Color32 = Color32::from_rgb(26, 26, 24);
const ON_INK: Color32 = Color32::from_rgb(251, 248, 242);
const ON_INK_SOFT: Color32 = Color32::from_rgb(201, 196, 183);
const EYEBROW: Color32 = Color32::from_rgb(217, 169, 124);

fn edit_card(ui: &mut egui::Ui, p: &Pal, icon: &str, label: &str, val: &mut String) {
    egui::Frame::none().fill(p.card).stroke(Stroke::new(1.0, p.line)).rounding(10.0).inner_margin(12.0).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(RichText::new(format!("{icon} {label}")).size(12.5).color(p.dim));
        ui.add(egui::TextEdit::multiline(val).desired_rows(2).desired_width(f32::INFINITY).frame(false));
    });
    ui.add_space(6.0);
}

fn cta(p: &Pal, text: &str) -> egui::Button<'static> {
    egui::Button::new(RichText::new(text.to_string()).font(fs(14.0)).color(p.on_accent)).fill(amber()).min_size(egui::vec2(0.0, 30.0))
}

impl App {
    pub fn intake_view(&mut self, ui: &mut egui::Ui, p: &Pal, id: &str) {
        let Some(mut it) = self.get_intake(id) else {
            self.view = View::Dashboard;
            return;
        };
        let before = serde_json::to_string(&it).unwrap_or_default();
        let busy = self.resolve_ui.rx.is_some();
        self.top_bar(ui, p, |app, ui| {
            if ui.button("Start over").on_hover_text("Begin a fresh intake (this one is kept)").clicked() {
                app.start_intake();
            }
            ui.label(RichText::new("Saved as you go — you can close and come back").size(11.5).color(p.faint));
        });
        if self.view != View::Intake(id.to_string()) {
            return;
        }
        let steps = ["Get started", "Bring it in", "Confirm understanding", "Your next move"];
        let cur = (it.step as usize).min(3);
        let pct = [0.25, 0.5, 0.75, 1.0][cur];
        egui::ScrollArea::vertical().auto_shrink([false, false]).id_salt("intake-scroll").show(ui, |ui| {
            centered(ui, 760.0, |ui| {
                ui.add_space(8.0);
                let (r, _) = ui.allocate_exact_size(egui::vec2(ui.available_width().min(360.0), 4.0), Sense::hover());
                ui.painter().rect_filled(r, 2.0, p.line);
                ui.painter().rect_filled(egui::Rect::from_min_size(r.min, egui::vec2(r.width() * pct, 4.0)), 2.0, amber());
                ui.horizontal(|ui| {
                    for (i, s) in steps.iter().enumerate() {
                        let col = if i == cur { p.text } else if i < cur { p.dim } else { p.faint };
                        ui.label(RichText::new(format!("{} {s}", if i < cur { "✓" } else { "·" })).size(12.0).color(col));
                    }
                });
                ui.add_space(18.0);
                if busy {
                    self.loader(ui, p);
                    return;
                }
                if !it.last_error.is_empty() {
                    egui::Frame::none().fill(p.accent_soft).rounding(8.0).inner_margin(12.0).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.label(RichText::new("Something went wrong").font(fs(14.0)).color(p.orange));
                        ui.label(RichText::new(&it.last_error).size(12.5));
                    });
                    ui.add_space(10.0);
                }
                match it.step {
                    0 => self.intake_start(ui, p, &mut it),
                    1 => self.intake_bring(ui, p, &mut it),
                    2 => self.intake_confirm(ui, p, &mut it),
                    3 => self.intake_next(ui, p, &mut it),
                    _ => self.intake_handoff(ui, p, &mut it),
                }
            });
        });
        // a job or the handoff may have replaced the stored intake this frame: only write UI edits back
        let stored = self.get_intake(id).map(|x| serde_json::to_string(&x).unwrap_or_default()).unwrap_or_default();
        let now = serde_json::to_string(&it).unwrap_or_default();
        if now != before && stored == before {
            it.touch();
            self.put_intake(&it);
            self.touch();
        }
    }

    fn intake_start(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut Intake) {
        ui.label(RichText::new("Let's get started").size(13.0).color(p.dim));
        ui.label(RichText::new("What's your name?").font(fserif(30.0)));
        ui.add(egui::TextEdit::singleline(&mut it.name).hint_text("Your first name").font(fp(17.0)).desired_width(320.0));
        ui.add_space(20.0);
        if it.name.trim().is_empty() {
            ui.label(RichText::new("Type your name to continue").size(12.0).color(p.faint));
            return;
        }
        ui.label(RichText::new(format!("Hi {}. Welcome to Resolve.", it.name.trim())).font(fs(16.0)));
        ui.add_space(8.0);
        ui.label(RichText::new("So, what do you need help with?").font(fserif(24.0)));
        ui.add_space(6.0);
        for s in intake::SITUATIONS.iter() {
            let sel = it.situation == s.id;
            let resp = egui::Frame::none().fill(if sel { p.accent_soft } else { p.card }).stroke(Stroke::new(1.0, if sel { amber() } else { p.line })).rounding(10.0).inner_margin(14.0).show(ui, |ui| {
                ui.set_width(ui.available_width().min(640.0));
                ui.horizontal(|ui| {
                    ui.label(RichText::new(s.icon).size(22.0));
                    ui.vertical(|ui| {
                        ui.label(RichText::new(s.title).font(fs(15.0)));
                        ui.label(RichText::new(s.desc).size(12.5).color(p.dim));
                        if !s.specified {
                            ui.label(RichText::new("Uses the messy-project flow for now — questions for this route aren't designed yet").size(11.0).color(p.faint));
                        }
                    });
                });
            });
            if ui.interact(resp.response.rect, ui.id().with(("sit", s.id)), Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                it.situation = s.id.into();
                it.step = 1;
            }
            ui.add_space(6.0);
        }
    }

    fn intake_bring(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut Intake) {
        ui.label(RichText::new("In your own words").size(13.0).color(p.dim));
        ui.label(RichText::new("Tell Resolve what's going on.").font(fserif(30.0)));
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            for (label, text) in intake::STARTERS {
                if chip(ui, label, amber()).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() && it.input.trim().is_empty() {
                    it.input = text.to_string();
                }
            }
        });
        ui.add_space(6.0);
        ui.add(egui::TextEdit::multiline(&mut it.input).hint_text("Describe the project however you like. What exists, what's confusing, what you're trying to decide, what's blocking you. Don't tidy it up.").desired_rows(12).desired_width(f32::INFINITY).font(fp(15.0)));
        if ui.link(RichText::new("Coming from another AI chat? Get a prompt to pull it in").size(12.5)).clicked() {
            self.resolve_ui.show_import = !self.resolve_ui.show_import;
        }
        if self.resolve_ui.show_import {
            card(ui, p, |ui| {
                ui.label(RichText::new(intake::IMPORT_PROMPT).size(12.5));
                if ui.button("Copy prompt").clicked() {
                    ui.ctx().copy_text(intake::IMPORT_PROMPT.into());
                    self.toast("Copied — paste it into that chat, then paste the answer here");
                }
            });
        }
        ui.add_space(10.0);
        let has = !it.input.trim().is_empty();
        ui.horizontal(|ui| {
            if ui.button("Back").clicked() {
                it.step = 0;
            }
            if ui.add_enabled(has, cta(p, "Build my path")).clicked() {
                it.last_error.clear();
                self.put_intake(it);
                let cfg = crate::ai::current_config();
                let (id, input) = (it.id.clone(), it.input.clone());
                self.spawn_job("Reading your project… separating what's decided from what's open", move || JobOut::Understand(id, intake::run_understand(&cfg, &input)));
            }
            if ui.add_enabled(has, egui::Button::new("Fill it in myself")).on_hover_text("No AI: you write the understanding and next move yourself").clicked() {
                it.manual = true;
                it.tools = intake::detect_tools(&it.input);
                if it.understanding.is_none() {
                    it.understanding = Some(Understanding { mentioned_tools: it.tools.clone(), ..Default::default() });
                }
                it.last_error.clear();
                it.step = 2;
            }
        });
        if !has {
            ui.label(RichText::new("Add a little context to continue").size(12.0).color(p.faint));
        }
    }

    fn intake_confirm(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut Intake) {
        ui.label(RichText::new("Here's what Resolve got from that.").font(fserif(30.0)));
        ui.label(RichText::new(if it.manual { "Fill these in — they shape everything that follows." } else { "Correct anything wrong — it shapes everything that follows." }).size(13.0).color(p.dim));
        ui.add_space(8.0);
        let manual = it.manual;
        let mut u = it.understanding.clone().unwrap_or_default();
        ui.columns(2, |c| {
            edit_card(&mut c[0], p, "📁", "Project", &mut u.project);
            edit_card(&mut c[1], p, "🎯", "What you want", &mut u.goal);
        });
        ui.columns(2, |c| {
            edit_card(&mut c[0], p, "📍", "Where it is", &mut u.state);
            edit_card(&mut c[1], p, "❓", "Biggest uncertainty", &mut u.uncertainty);
        });
        if manual {
            edit_card(ui, p, "🌱", "If this works, who is better off, and how would you know?", &mut u.outcome);
        } else if u.outcome_stated {
            edit_card(ui, p, "🌱", "Outcome (you said it)", &mut u.outcome);
        }
        ui.add_space(8.0);
        ui.label(RichText::new("Tools we noticed you mentioned").font(fs(13.5)));
        ui.horizontal_wrapped(|ui| {
            let mut all: Vec<String> = vec![];
            for t in u.mentioned_tools.iter().chain(it.tools.iter()) {
                if !all.iter().any(|x| x.eq_ignore_ascii_case(t)) {
                    all.push(t.clone());
                }
            }
            for t in all {
                let on = it.tools.iter().any(|x| x.eq_ignore_ascii_case(&t));
                if pill(ui, p, &t, on).clicked() {
                    if on {
                        it.tools.retain(|x| !x.eq_ignore_ascii_case(&t));
                    } else {
                        it.tools.push(t.clone());
                    }
                }
            }
            let r = ui.add(egui::TextEdit::singleline(&mut self.resolve_ui.new_tool).hint_text("+ Add").desired_width(90.0));
            if r.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) && !self.resolve_ui.new_tool.trim().is_empty() {
                let t = self.resolve_ui.new_tool.trim().to_string();
                if !it.tools.iter().any(|x| x.eq_ignore_ascii_case(&t)) {
                    it.tools.push(t);
                }
                self.resolve_ui.new_tool.clear();
            }
        });
        it.understanding = Some(u.clone());
        it.sync_gap_answers();
        ui.add_space(8.0);
        if !manual {
            let qs = it.gap_answers.len();
            if qs == 0 {
                ui.label(RichText::new("Resolve has everything it needs to build your path.").size(13.0).color(p.green));
            } else {
                ui.label(RichText::new(if qs == 1 { "One thing Resolve still needs" } else { "A couple of things Resolve still needs" }).font(fs(14.0)));
                let oq = it.outcome_question();
                for g in it.gap_answers.iter_mut() {
                    let required = !oq.is_empty() && g.question == oq;
                    egui::Frame::none().fill(p.card).stroke(Stroke::new(1.0, p.line)).rounding(8.0).inner_margin(12.0).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal_wrapped(|ui| {
                            ui.label(RichText::new(&g.question).font(fs(13.5)));
                            chip(ui, if required { "Required" } else { "Optional" }, if required { p.orange } else { p.dim });
                        });
                        ui.add(egui::TextEdit::multiline(&mut g.answer).hint_text("Type your answer here").desired_rows(2).desired_width(f32::INFINITY));
                    });
                }
                if !it.reask_message.is_empty() {
                    ui.label(RichText::new(&it.reask_message).size(13.0).color(amber()));
                }
            }
        }
        ui.add_space(8.0);
        ui.label(RichText::new("What Resolve is assuming").font(fs(13.5)));
        let mut text = u.assumptions.join("\n");
        if ui.add(egui::TextEdit::multiline(&mut text).hint_text("One assumption per line").desired_rows(2).desired_width(f32::INFINITY)).changed() {
            if let Some(x) = it.understanding.as_mut() {
                x.assumptions = text.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect();
            }
        }
        if u.assumptions.is_empty() && !manual {
            ui.label(RichText::new("No explicit assumptions were needed — the input was specific enough.").size(12.0).color(p.faint));
        }
        ui.add_space(12.0);
        let u = it.understanding.clone().unwrap_or_default();
        let gate_blocked = !manual && it.outcome_gate_active() && it.outcome_answer().is_empty();
        let manual_blocked = manual && (u.project.trim().is_empty() || u.goal.trim().is_empty() || u.state.trim().is_empty() || u.outcome.trim().is_empty());
        let mut go = false;
        ui.horizontal(|ui| {
            if ui.button("Back").clicked() {
                it.step = 1;
            }
            if ui.add_enabled(!gate_blocked && !manual_blocked, cta(p, "This looks right — build my path")).clicked() {
                go = true;
            }
        });
        if go {
            it.last_error.clear();
            if manual {
                it.unverified = true;
                it.outcome_confirmed = false;
                self.put_intake(it);
                self.run_recommend(&it.id);
            } else {
                let answer = it.outcome_answer();
                let needs_judge = it.outcome_gate_active() && !answer.is_empty() && !it.judged.contains(&answer);
                self.put_intake(it);
                if needs_judge {
                    let cfg = crate::ai::current_config();
                    let (id, q) = (it.id.clone(), it.outcome_question());
                    self.spawn_job("Checking your outcome…", move || {
                        let r = intake::run_judge(&cfg, &q, &answer);
                        JobOut::Judge(id, answer, r)
                    });
                } else {
                    self.run_recommend(&it.id);
                }
            }
        }
        if gate_blocked {
            ui.label(RichText::new("Answer the outcome question to continue").size(12.0).color(p.faint));
        }
        if manual_blocked {
            ui.label(RichText::new("Fill in the project, what you want, where it is and the outcome to continue").size(12.0).color(p.faint));
        }
    }

    fn intake_next(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut Intake) {
        let manual = it.manual;
        let mut r = it.recommendation.clone().unwrap_or_default();
        ui.label(RichText::new("Your next move").size(13.0).color(p.dim));
        ui.label(RichText::new("Here's what to do next.").font(fb(26.0)));
        ui.add_space(8.0);
        egui::Frame::none().fill(INK).rounding(12.0).inner_margin(18.0).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(if r.top_card_type == "decision" { "🎯 Decide this first" } else { "🎯 Do this first" }).size(12.5).color(EYEBROW));
            if manual {
                ui.add(egui::TextEdit::multiline(&mut r.recommended_action).hint_text("The single most important thing to do next").desired_rows(2).desired_width(f32::INFINITY));
                ui.add(egui::TextEdit::multiline(&mut r.why).hint_text("Why this, now").desired_rows(2).desired_width(f32::INFINITY));
            } else {
                ui.label(RichText::new(&r.recommended_action).font(fserif(22.0)).color(ON_INK));
                ui.label(RichText::new(&r.why).size(13.0).color(ON_INK_SOFT));
            }
        });
        ui.add_space(8.0);
        card(ui, p, |ui| {
            ui.label(RichText::new("🏁 First milestone").font(fs(13.5)));
            if manual {
                ui.add(egui::TextEdit::singleline(&mut r.milestone).hint_text("What 'done' looks like").desired_width(f32::INFINITY));
            } else {
                ui.label(&r.milestone);
            }
        });
        ui.add_space(8.0);
        ui.label(RichText::new("What happens after this").font(fs(14.0)));
        let labels = ["This week", "Next", "Later"];
        while r.roadmap.len() < 3 {
            r.roadmap.push(Phase::default());
        }
        ui.columns(3, |c| {
            for i in 0..3 {
                card(&mut c[i], p, |ui| {
                    ui.label(RichText::new(labels[i]).size(12.0).color(amber()));
                    if manual {
                        ui.add(egui::TextEdit::multiline(&mut r.roadmap[i].action).hint_text("Action").desired_rows(2).desired_width(f32::INFINITY));
                        ui.add(egui::TextEdit::singleline(&mut r.roadmap[i].output).hint_text("Output").desired_width(f32::INFINITY));
                    } else {
                        ui.label(RichText::new(&r.roadmap[i].action).font(fs(13.5)));
                        ui.label(RichText::new(format!("Output: {}", r.roadmap[i].output)).size(12.0).color(p.dim));
                    }
                });
            }
        });
        ui.add_space(8.0);
        ui.columns(2, |c| {
            card(&mut c[0], p, |ui| {
                ui.label(RichText::new("Top priorities").font(fs(13.5)));
                if manual {
                    let mut t = r.top_priorities.join("\n");
                    if ui.add(egui::TextEdit::multiline(&mut t).hint_text("1–3, one per line").desired_rows(3).desired_width(f32::INFINITY)).changed() {
                        r.top_priorities = t.lines().map(|x| x.to_string()).collect();
                    }
                } else {
                    for x in &r.top_priorities {
                        ui.label(format!("• {x}"));
                    }
                }
            });
            card(&mut c[1], p, |ui| {
                ui.label(RichText::new("Not yet").font(fs(13.5)));
                if manual {
                    let mut t = r.do_not_yet.iter().map(|n| if n.reason.is_empty() { n.item.clone() } else { format!("{} — {}", n.item, n.reason) }).collect::<Vec<_>>().join("\n");
                    if ui.add(egui::TextEdit::multiline(&mut t).hint_text("Thing — reason (1–3 lines)").desired_rows(3).desired_width(f32::INFINITY)).changed() {
                        r.do_not_yet = t
                            .lines()
                            .map(|l| {
                                let mut sp = l.splitn(2, " — ");
                                NotYet { item: sp.next().unwrap_or("").trim().into(), reason: sp.next().unwrap_or("").trim().into() }
                            })
                            .collect();
                    }
                } else {
                    for n in &r.do_not_yet {
                        ui.label(format!("• {} — {}", n.item, n.reason));
                    }
                }
            });
        });
        if manual {
            ui.horizontal(|ui| {
                ui.label("Open decision");
                ui.add(egui::TextEdit::singleline(&mut r.open_decision).hint_text("A decision that still matters (optional)").desired_width(f32::INFINITY));
            });
        } else if !r.open_decision.is_empty() {
            ui.label(RichText::new(format!("⚠ One decision before you move: {}", r.open_decision)).size(13.0).color(amber()));
        }
        it.recommendation = Some(r);
        ui.add_space(12.0);
        let check = intake::handoff_check(it);
        let mut again = false;
        ui.horizontal(|ui| {
            if ui.button("Back").clicked() {
                it.step = 2;
            }
            if ui.add_enabled(check.is_ok(), cta(p, "Continue")).clicked() {
                it.step = 4;
            }
            if !manual && ui.button("Ask again").on_hover_text("Regenerate the recommendation").clicked() {
                again = true;
            }
        });
        if again {
            self.put_intake(it);
            self.run_recommend(&it.id);
        }
        if let Err(e) = check {
            ui.label(RichText::new(format!("Still needed: {e}")).size(12.0).color(p.faint));
        }
    }

    fn intake_handoff(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut Intake) {
        let r = it.recommendation.clone().unwrap_or_default();
        egui::Frame::none().fill(INK).rounding(14.0).inner_margin(20.0).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new("🏆").size(30.0));
            ui.label(RichText::new(format!("You just cleared the fog, {}.", it.name.trim())).font(fserif(26.0)).color(ON_INK));
            let ws = if r.win_summary.trim().is_empty() { "A messy project with no clear direction — turned into a focused path forward. That's the hard part done.".to_string() } else { r.win_summary.clone() };
            ui.label(RichText::new(ws).size(13.5).color(ON_INK_SOFT));
        });
        ui.add_space(10.0);
        card(ui, p, |ui| {
            ui.label(RichText::new(if r.top_card_type == "decision" { "Decide this first" } else { "Do this first" }).size(12.0).color(amber()));
            ui.label(RichText::new(&r.recommended_action).font(fserif(20.0)));
            ui.label(RichText::new("This becomes your first move inside Resolve — pinned and waiting, with everything you just untangled carried across. Pick up exactly where you left off, no re-explaining.").size(12.5).color(p.dim));
        });
        if let Some(pid) = it.project_id.clone().filter(|x| self.store.data.project(x).is_some()) {
            ui.add_space(8.0);
            if primary_button(ui, p, "Open the project").clicked() {
                self.go(View::Resolve(pid));
            }
            return;
        }
        ui.add_space(10.0);
        let ai = !it.manual;
        let (mut cont, mut without) = (false, false);
        ui.horizontal(|ui| {
            if ui.button("Back").clicked() {
                it.step = 3;
            }
            let label = if ai && it.plan.is_none() { "Continue in Resolve (builds the plan first)" } else { "Continue in Resolve" };
            if ui.add(cta(p, label)).clicked() {
                cont = true;
            }
            if ai && it.plan.is_none() && !it.last_error.is_empty() && ui.button("Create without the plan sections").clicked() {
                without = true;
            }
        });
        if cont || without {
            self.put_intake(it);
            if cont && ai && it.plan.is_none() {
                let cfg = crate::ai::current_config();
                let (id, u, rec, g, t) = (it.id.clone(), it.understanding.clone().unwrap_or_default(), r.clone(), it.gap_answers.clone(), it.tools.clone());
                self.spawn_job("Building your project plan… sequencing the work", move || JobOut::Plan(id, intake::run_plan(&cfg, &u, &rec, &g, &t)));
            } else {
                self.finish_intake(&it.id);
            }
        }
    }
}
