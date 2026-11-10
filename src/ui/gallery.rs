//! The component gallery (`--view gallery`): every token, component, toast and motion sample,
//! in light or dark and with reduced motion on or off. It is the Phase 0 proof that the
//! foundations work, and the reference every later phase builds from. No colour or size here
//! that isn't a token.

use super::motion::{self, frame_at, key, timing, Key, CURVES, E_IN, E_OUT, E_SPRING, E_STD, LINEAR};
use super::theme::*;
use super::widgets::*;
use super::*;
use eframe::egui::{self, Align2, RichText, Rounding, Sense, Shape, Stroke};

const STATES: [&str; 4] = ["Full", "Empty", "Loading", "Error"];

fn section(ui: &mut egui::Ui, p: &Pal, eyebrow: &str, title: &str) {
    ui.add_space(space::S32);
    ui.label(RichText::new(eyebrow).font(fs(size::LABEL)).color(p.eyebrow));
    ui.label(RichText::new(title).font(fserif(size::STEP_TITLE)).color(p.text));
    ui.add_space(space::S12);
}

fn now_ms(ui: &egui::Ui) -> f32 {
    (ui.ctx().input(|i| i.time) * 1000.0) as f32
}

impl App {
    pub(super) fn gallery_view(&mut self, ui: &mut egui::Ui, p: &Pal) {
        let ctx = ui.ctx().clone();
        // in a screenshot, show a live toast (with Undo) so its place and look are on record
        if self.shot.is_some() && self.frame == 60 {
            self.undo_toasts.push(("Task completed: Call the garage".into(), self.now + 4.0, "gallery-demo".into()));
        }
        let mut area = egui::ScrollArea::vertical().auto_shrink([false, false]).id_salt("gallery");
        if self.frame < 5 {
            if let Some(s) = self.temp.get("scroll").and_then(|s| s.parse::<f32>().ok()) {
                area = area.vertical_scroll_offset(s);
            }
        }
        area.show(ui, |ui| {
            centered(ui, space::CAP_DEFAULT, |ui| {
                ui.add_space(space::WORK_TOP);
                ui.label(RichText::new("Phase 0 · Foundations").font(fs(size::LABEL)).color(p.eyebrow));
                ui.label(RichText::new("Component gallery").font(fserif(size::DISPLAY)).color(p.text));
                wrap_label(ui, RichText::new("Every token, component, toast and motion sample the redesign builds on. Switch light and dark, and reduced motion, to check each one.").size(size::BODY).color(p.dim));
                ui.add_space(space::S16);
                // controls
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new("Appearance").font(fs(size::SMALL)).color(p.dim));
                    if let Some(i) = segmented(ui, p, egui::Id::new("g-theme"), &["Light", "Dark"], usize::from(p.dark)) {
                        self.reg.settings.dark = i == 1;
                    }
                    ui.add_space(space::S16);
                    ui.label(RichText::new("Reduced motion").font(fs(size::SMALL)).color(p.dim));
                    if let Some(i) = segmented(ui, p, egui::Id::new("g-reduced"), &["Off", "On"], usize::from(motion::reduced())) {
                        motion::set_reduced(i == 1);
                    }
                    ui.add_space(space::S16);
                    ui.label(RichText::new("State").font(fs(size::SMALL)).color(p.dim));
                    let cur = self.view_state as usize;
                    if let Some(i) = segmented(ui, p, egui::Id::new("g-state"), &STATES, cur) {
                        self.view_state = [ViewState::Full, ViewState::Empty, ViewState::Loading, ViewState::Error][i];
                    }
                    ui.add_space(space::S16);
                    if button(ui, p, ButtonKind::Secondary, "Replay motion", true, true).clicked() {
                        for k in ["g-stagger", "g-entrance"] {
                            motion::restart(&ctx, egui::Id::new(k));
                        }
                    }
                });

                self.gallery_states(ui, p);
                gallery_colours(ui, p);
                gallery_type(ui, p);
                gallery_space(ui, p);
                gallery_buttons(ui, p);
                gallery_tags(ui, p);
                self.gallery_inputs(ui, p);
                gallery_cards(ui, p);
                self.gallery_toasts(ui, p);
                gallery_motion(ui, p);
                gallery_glow(ui, p);
                ui.add_space(space::S40 * 2.0);
            });
        });
    }

    /// The four states every view draws, side by side, and the one chosen with --state.
    fn gallery_states(&mut self, ui: &mut egui::Ui, p: &Pal) {
        section(ui, p, "Rules, item 3", "Four states");
        let chosen = self.view_state;
        wrap_label(ui, RichText::new(format!("Every view draws all four. This screenshot was taken in the {} state, outlined below.", STATES[chosen as usize].to_lowercase())).size(size::SMALL).color(p.dim));
        ui.add_space(space::S10);
        ui.columns(4, |cols| {
            for (i, ui) in cols.iter_mut().enumerate() {
                let on = i == chosen as usize;
                let at = ui.painter().add(Shape::Noop);
                let top = ui.cursor().min;
                ui.label(RichText::new(STATES[i]).font(fs(size::LABEL)).color(if on { p.accent_text } else { p.dim }));
                ui.add_space(space::S6);
                match i {
                    0 => {
                        card_lift(ui, p, |ui| {
                            ui.label(RichText::new("Café flat-fee bookkeeping").font(fs(size::BODY)).color(p.text));
                            wrap_label(ui, RichText::new("Next: reconstruct 12 months of hours per café.").size(size::SMALL).color(p.dim));
                            ui.add_space(space::S6);
                            ui.horizontal_wrapped(|ui| {
                                status_pill(ui, "Unresolved");
                                tag(ui, "2 now", p.text, p.surface_raised);
                            });
                        });
                    }
                    1 => {
                        let _ = empty_block(ui, p, "No projects yet", "Start one and Resolve builds the plan with you.", "Start a project");
                    }
                    2 => skeleton_card(ui, p, 3, "Loading your projects"),
                    _ => {
                        let _ = error_block(ui, p, "Couldn't load your projects.", "Open them from Search");
                    }
                }
                if on {
                    let r = egui::Rect::from_min_max(top - egui::vec2(space::S6, space::S6), egui::pos2(top.x + ui.available_width() + space::S6, ui.cursor().min.y + space::S6));
                    ui.painter().set(at, Shape::rect_stroke(r, radius::CARD, Stroke::new(stroke::MARK, p.accent)));
                }
            }
        });
    }

    fn gallery_inputs(&mut self, ui: &mut egui::Ui, p: &Pal) {
        section(ui, p, "Inputs and voice", "Fields, focus ring, mic");
        let fid = egui::Id::new("g-field-focused");
        // the focused example shows the ring in screenshots
        if self.frame < 3 {
            ui.ctx().memory_mut(|m| m.request_focus(fid));
        }
        ui.columns(2, |cols| {
            let w = cols[0].available_width();
            let mut a = self.temp.remove("g-a").unwrap_or_default();
            cols[0].label(RichText::new("Text field").font(fs(size::LABEL)).color(p.dim));
            text_field(&mut cols[0], p, egui::Id::new("g-field-a"), &mut a, "Your first name", false, w);
            cols[0].add_space(space::S12);
            cols[0].label(RichText::new("Focused (ring 3 px amber .22, border amber, 160 e-std)").font(fs(size::LABEL)).color(p.dim));
            let mut b = self.temp.remove("g-b").unwrap_or_else(|| "I run a bookkeeping practice".into());
            text_field(&mut cols[0], p, fid, &mut b, "", false, w);
            self.temp.insert("g-a".into(), a);
            self.temp.insert("g-b".into(), b);
            let w2 = cols[1].available_width();
            cols[1].label(RichText::new("Multi-line with mic").font(fs(size::LABEL)).color(p.dim));
            let mut c = self.temp.remove("g-c").unwrap_or_default();
            cols[1].horizontal_top(|ui| {
                text_field(ui, p, egui::Id::new("g-field-c"), &mut c, "Tell Resolve what's going on", true, w2 - space::MIC - space::S10);
                // the real mic: focuses this field and opens Windows voice typing (Win+H)
                mic_for(ui, p, egui::Id::new("g-field-c"), false);
            });
            self.temp.insert("g-c".into(), c);
            cols[1].add_space(space::S12);
            cols[1].label(RichText::new("Mic states: idle, recording (pulse 1200 loop), working").font(fs(size::LABEL)).color(p.dim));
            cols[1].horizontal(|ui| {
                mic(ui, p, MicState::Idle, false);
                ui.add_space(space::S16);
                mic(ui, p, MicState::Recording, false);
                ui.add_space(space::S16);
                mic(ui, p, MicState::Working, false);
                ui.add_space(space::S24);
                // on dark (Ask bar, intake rail)
                egui::Frame::none().fill(tok::INK).rounding(Rounding::same(radius::PILL)).inner_margin(egui::Margin::symmetric(space::S12, space::S6)).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        mic(ui, p, MicState::Idle, true);
                        mic(ui, p, MicState::Recording, true);
                        mic(ui, p, MicState::Working, true);
                    });
                });
            });
        });
    }

    fn gallery_toasts(&mut self, ui: &mut egui::Ui, p: &Pal) {
        section(ui, p, "B-E10", "Toasts, one at a time");
        wrap_label(ui, RichText::new("A new toast replaces the one showing in 160 ms. With Undo: 4 s and an amber timer bar. Plain: 2.6 s. They sit 10 px above the Ask bar, centred on the work area.").size(size::SMALL).color(p.dim));
        ui.add_space(space::S10);
        ui.horizontal_wrapped(|ui| {
            if button(ui, p, ButtonKind::Secondary, "Show a plain toast", false, true).clicked() {
                self.toast("Saved to Knowledge base");
            }
            if button(ui, p, ButtonKind::Secondary, "Show a toast with Undo", false, true).clicked() {
                self.undo_toasts.push(("Task completed: Call the garage".into(), self.now + 4.0, "gallery-demo".into()));
            }
            if button(ui, p, ButtonKind::Secondary, "Show three quickly", false, true).clicked() {
                self.toast("First");
                self.toast("Second");
                self.toast("Third replaces them");
            }
        });
        ui.add_space(space::S12);
        ui.horizontal_wrapped(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("Plain").font(fs(size::LABEL)).color(p.dim));
                paint_toast(ui, "Saved to Knowledge base", false, 0.0);
            });
            ui.add_space(space::S24);
            ui.vertical(|ui| {
                ui.label(RichText::new("With Undo (timer at 60%)").font(fs(size::LABEL)).color(p.dim));
                paint_toast(ui, "Task completed: Call the garage", true, 0.6);
            });
        });
    }
}

fn gallery_colours(ui: &mut egui::Ui, p: &Pal) {
    section(ui, p, "E18 · Tokens", "Colour");
    let cols_n = 6;
    for row in tok::ALL.chunks(cols_n) {
        ui.columns(cols_n, |cols| {
            for (i, (name, c, hex)) in row.iter().enumerate() {
                let ui = &mut cols[i];
                let w = ui.available_width();
                let (r, _) = ui.allocate_exact_size(egui::vec2(w, space::S40 + space::S16), Sense::hover());
                ui.painter().rect(r, radius::CHIP, *c, Stroke::new(stroke::HAIR, p.line));
                let light = c.r() as f32 * 0.299 + c.g() as f32 * 0.587 + c.b() as f32 * 0.114 > 150.0;
                ui.painter().text(r.left_bottom() + egui::vec2(space::S8, -space::S6), Align2::LEFT_BOTTOM, *hex, fp(size::TAG), if light { tok::INK } else { tok::TEXT_ON_DARK });
                ui.label(RichText::new(*name).font(fs(size::SMALL)).color(p.text));
            }
        });
        ui.add_space(space::S8);
    }
    ui.add_space(space::S8);
    ui.label(RichText::new("Contrast pairs").font(fs(size::LABEL)).color(p.dim));
    ui.horizontal_wrapped(|ui| {
        tag(ui, "amber-text on amber-light", tok::AMBER_TEXT, tok::AMBER_LIGHT);
        tag(ui, "green-text on green-tint", tok::GREEN_TEXT, tok::GREEN_TINT);
        tag(ui, "terracotta on terracotta-tint", tok::TERRACOTTA, tok::TERRACOTTA_TINT);
        tag(ui, "terracotta-on-dark on ink", tok::TERRACOTTA_ON_DARK, tok::INK);
        tag(ui, "text-on-dark-muted on ink", tok::TEXT_ON_DARK_MUTED, tok::INK);
        tag(ui, "ink on amber", tok::INK, tok::AMBER);
    });
}

fn gallery_type(ui: &mut egui::Ui, p: &Pal) {
    section(ui, p, "E18 · Type", "DM Serif Display and Inter");
    let rows: [(&str, egui::FontId, &str); 12] = [
        ("hero · DM Serif 52 / 1.05", fserif(size::HERO), "Hi Simeon."),
        ("display · DM Serif 44 / 1.1", fserif(size::DISPLAY), "Welcome in"),
        ("title · DM Serif 30 / 1.15", fserif(size::STEP_TITLE), "Here's what Resolve understood."),
        ("question · DM Serif 29 / 1.22", fserif(size::QUESTION), "Who is better off if this works?"),
        ("card-serif · DM Serif 24 / 1.25", fserif(size::CARD_SERIF), "Reconstruct 12 months of hours"),
        ("body · Inter 400 14 / 1.5", fp(size::BODY), "Reconstruct the real hours per café across a full year."),
        ("answer · Inter 400 15 / 1.55", fp(size::ANSWER), "I run a small bookkeeping practice for cafés."),
        ("nav · Inter 500 13.5", fm(size::NAV), "Knowledge base"),
        ("button · Inter 600 14", fs(size::BUTTON), "Use this plan"),
        ("label · Inter 600 13", fs(size::LABEL), "WHAT MATTERS NOW"),
        ("small · Inter 400 12.5", fp(size::SMALL), "Saved on this PC · works offline"),
        ("tag · Inter 600 11", fs(size::TAG), "ASSUMPTION · 3"),
    ];
    for (label, font, sample) in rows {
        ui.horizontal(|ui| {
            ui.add_sized([space::S40 * 6.0, space::S24], egui::Label::new(RichText::new(label).size(size::SMALL).color(p.dim)));
            ui.label(RichText::new(sample).font(font).color(p.text));
        });
        ui.add_space(space::S4);
    }
}

fn gallery_space(ui: &mut egui::Ui, p: &Pal) {
    section(ui, p, "E18 · Spacing, radii, shadows", "Spacing, radii and shadows");
    ui.horizontal_wrapped(|ui| {
        for s in [space::S4, space::S6, space::S8, space::S10, space::S12, space::S14, space::S16, space::S18, space::S22, space::S24, space::S26, space::S30, space::S32, space::S40] {
            ui.vertical(|ui| {
                let (r, _) = ui.allocate_exact_size(egui::vec2(s, s), Sense::hover());
                ui.painter().rect_filled(r, radius::GATE.min(s / 4.0), p.accent);
                ui.label(RichText::new(format!("{s}")).size(size::TAG).color(p.dim));
            });
        }
    });
    ui.add_space(space::S12);
    ui.horizontal_wrapped(|ui| {
        for (name, r) in [("chip 7", radius::CHIP), ("small 8", radius::SMALL_BUTTON), ("button 10", radius::BUTTON), ("card 13", radius::CARD), ("toast 14", radius::TOAST), ("dialog 16", radius::DIALOG), ("sheet 20", radius::SHEET), ("pill 999", radius::PILL)] {
            ui.vertical(|ui| {
                let (rect, _) = ui.allocate_exact_size(egui::vec2(space::S40 * 2.0, space::S40 + space::S8), Sense::hover());
                ui.painter().rect(rect, r, p.card, Stroke::new(stroke::HAIR, p.line));
                ui.label(RichText::new(name).size(size::TAG).color(p.dim));
            });
            ui.add_space(space::S8);
        }
    });
    ui.add_space(space::S24);
    ui.horizontal_wrapped(|ui| {
        let shadows: [(&str, Vec<egui::epaint::Shadow>); 5] = [
            ("card-lift", shadow::CARD_LIFT.to_vec()),
            ("dialog", vec![shadow::DIALOG]),
            ("sheet", vec![shadow::SHEET]),
            ("side panel", vec![shadow::SIDE_PANEL]),
            ("Ask bar", vec![shadow::ASK_BAR]),
        ];
        for (name, sh) in shadows {
            ui.vertical(|ui| {
                ui.add_space(space::S24);
                let (rect, _) = ui.allocate_exact_size(egui::vec2(space::S40 * 4.0, space::S40 * 2.0), Sense::hover());
                paint_shadows(ui.painter(), rect, radius::CARD, &sh);
                ui.painter().rect(rect, radius::CARD, p.card, Stroke::new(stroke::HAIR, p.line));
                ui.painter().text(rect.center(), Align2::CENTER_CENTER, name, fs(size::SMALL), p.text);
                ui.add_space(space::S30);
            });
            ui.add_space(space::S30);
        }
    });
    ui.horizontal_wrapped(|ui| {
        for (name, a) in [("backdrop · panel .18", shadow::BACKDROP_PANEL), ("sheet .32", shadow::BACKDROP_SHEET), ("dialog .38", shadow::BACKDROP_DIALOG)] {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(space::S40 * 4.0, space::S40), Sense::hover());
            ui.painter().rect_filled(rect, radius::CHIP, p.card);
            ui.painter().rect_filled(rect, radius::CHIP, tok::ink_a(a));
            ui.painter().text(rect.center(), Align2::CENTER_CENTER, name, fs(size::SMALL), tok::TEXT_ON_DARK);
            ui.add_space(space::S12);
        }
    });
}

fn gallery_buttons(ui: &mut egui::Ui, p: &Pal) {
    section(ui, p, "Components", "Buttons");
    wrap_label(ui, RichText::new("Press any button: it scales to .975 in 240 ms e-std about its centre. Tab to one to see the focus ring.").size(size::SMALL).color(p.dim));
    ui.add_space(space::S10);
    for (small, label) in [(false, "Regular"), (true, "Small")] {
        ui.horizontal_wrapped(|ui| {
            ui.add_sized([space::S40 * 2.0, space::S24], egui::Label::new(RichText::new(label).size(size::SMALL).color(p.dim)));
            for (kind, name) in BUTTON_KINDS {
                let text = match kind {
                    ButtonKind::Primary => "Use this plan",
                    ButtonKind::Dark => "Start this",
                    ButtonKind::Secondary => "Not quite",
                    ButtonKind::Ghost => "See and edit everything",
                    ButtonKind::Danger => "Delete project",
                    ButtonKind::OnDark => "Plan my day",
                };
                button(ui, p, kind, text, small, true).on_hover_text(name);
                ui.add_space(space::S8);
            }
            button(ui, p, ButtonKind::Primary, "Continue (disabled)", small, false);
        });
        ui.add_space(space::S10);
    }
}

fn gallery_tags(ui: &mut egui::Ui, p: &Pal) {
    section(ui, p, "Components", "Pills, tags, chips and the segmented control");
    ui.label(RichText::new("Status pills").font(fs(size::LABEL)).color(p.dim));
    ui.horizontal_wrapped(|ui| {
        for s in ["Unresolved", "Supported", "Weakened", "Refuted", "Planned"] {
            status_pill(ui, s);
        }
        ui.add_space(space::S16);
        gate_chip(ui, "H1 result", false);
        gate_chip(ui, "H1 result", true);
    });
    ui.add_space(space::S10);
    ui.label(RichText::new("Knowledge base kinds").font(fs(size::LABEL)).color(p.dim));
    ui.horizontal_wrapped(|ui| {
        for k in ["Page", "File", "Experiment", "Research", "Evidence", "Hypothesis", "Assumption", "Open question", "Cut"] {
            kind_tag(ui, k);
        }
    });
    ui.add_space(space::S10);
    ui.label(RichText::new("Filter chips").font(fs(size::LABEL)).color(p.dim));
    ui.horizontal_wrapped(|ui| {
        filter_chip(ui, p, "All", Some(24), true);
        filter_chip(ui, p, "Pages", Some(9), false);
        filter_chip(ui, p, "Evidence", Some(4), false);
        filter_chip(ui, p, "Open questions", Some(3), false);
    });
    ui.add_space(space::S10);
    ui.label(RichText::new("Segmented control (thumb slides 320 e-spring)").font(fs(size::LABEL)).color(p.dim));
    let id = egui::Id::new("g-seg");
    let cur: usize = ui.ctx().data_mut(|d| *d.get_temp_mut_or_insert_with(id, || 0usize));
    if let Some(i) = segmented(ui, p, id, &["List", "Board"], cur) {
        ui.ctx().data_mut(|d| d.insert_temp(id, i));
    }
    let id2 = egui::Id::new("g-seg2");
    let cur2: usize = ui.ctx().data_mut(|d| *d.get_temp_mut_or_insert_with(id2, || 0usize));
    if let Some(i) = segmented(ui, p, id2, &["By phase", "By theme"], cur2) {
        ui.ctx().data_mut(|d| d.insert_temp(id2, i));
    }
}

fn gallery_cards(ui: &mut egui::Ui, p: &Pal) {
    section(ui, p, "Components and E17", "Cards and the hero card");
    ui.columns(2, |cols| {
        card_lift(&mut cols[0], p, |ui| {
            ui.label(RichText::new("WHAT MATTERS NOW").font(fs(size::LABEL)).color(p.eyebrow));
            ui.add_space(space::S6);
            ui.label(RichText::new("Card on light").font(fserif(size::CARD_SERIF)).color(p.text));
            wrap_label(ui, RichText::new("White, border, radius 13, 16 padding, card-lift shadow.").size(size::BODY).color(p.dim));
        });
        hero_card(&mut cols[1], p, |ui| {
            ui.label(RichText::new("DO THIS NEXT").font(fs(size::LABEL)).color(tok::TERRACOTTA_ON_DARK));
            ui.add_space(space::S6);
            ui.label(RichText::new("Reconstruct 12 months of hours per café").font(fserif(size::CARD_SERIF)).color(tok::TEXT_ON_DARK));
            wrap_label(ui, RichText::new("The hero card: ink, radius 16, an amber glow drawn as a gradient mesh that breathes on a 3200 ms loop.").size(size::BODY).color(tok::TEXT_ON_DARK_SOFT));
            ui.add_space(space::S10);
            button(ui, p, ButtonKind::Primary, "Start this", false, true);
        });
    });
}

fn gallery_motion(ui: &mut egui::Ui, p: &Pal) {
    section(ui, p, "E16 · B-E18 · Motion", "Curves, tweens, scale, stagger, keyframes");
    let reduced = motion::reduced();
    wrap_label(ui, RichText::new(if reduced { "Reduced motion is ON: every move is a fade of 120 ms or less and loops hold still. The dots sit at their end points." } else { "Each dot runs its curve for 1000 ms, then holds. e-spring overshoots past the end and settles." }).size(size::SMALL).color(if reduced { p.orange } else { p.dim }));
    ui.add_space(space::S10);
    let ms = now_ms(ui) % 1800.0;
    if !reduced {
        ui.ctx().request_repaint();
    }
    for (name, c) in CURVES {
        ui.horizontal(|ui| {
            ui.add_sized([space::S40 * 2.0, space::S18], egui::Label::new(RichText::new(name).font(fs(size::SMALL)).color(p.text)));
            let w = ui.available_width().min(space::S40 * 14.0);
            let (rect, _) = ui.allocate_exact_size(egui::vec2(w, space::S18), Sense::hover());
            ui.painter().rect_filled(egui::Rect::from_center_size(rect.center(), egui::vec2(rect.width(), stroke::MARK)), stroke::HAIR, p.track);
            let f = frame_at(ms, timing(1000.0, 0.0, c), reduced);
            let x = rect.left() + space::S8 + (rect.width() - space::S16) * f.t;
            ui.painter().circle_filled(egui::pos2(x, rect.center().y), space::S6, p.accent.gamma_multiply(f.alpha.max(if reduced { f.alpha } else { 1.0 })));
            // the curve's shape, small
            let (g, _) = ui.allocate_exact_size(egui::vec2(space::S40 + space::S8, space::S18), Sense::hover());
            let pts: Vec<egui::Pos2> = (0..=24).map(|i| {
                let x = i as f32 / 24.0;
                egui::pos2(g.left() + g.width() * x, g.bottom() - g.height() * 0.8 * c.at(x))
            }).collect();
            ui.painter().add(Shape::line(pts, Stroke::new(stroke::ICON, p.dim)));
        });
        ui.add_space(space::S4);
    }
    ui.add_space(space::S16);
    ui.columns(3, |cols| {
        // width tween: the sidebar 248 ↔ 64, 320 e-out, flipping every 2 s
        {
            let ui = &mut cols[0];
            ui.label(RichText::new("Width tween · sidebar 248 ↔ 64 (320 e-out)").font(fs(size::LABEL)).color(p.dim));
            let collapsed = (now_ms(ui) / 2000.0) as i64 % 2 == 1;
            let w = motion::tween(ui.ctx(), egui::Id::new("g-width"), if collapsed { space::SIDEBAR_COLLAPSED } else { space::SIDEBAR }, timing(320.0, 0.0, E_OUT));
            let (rect, _) = ui.allocate_exact_size(egui::vec2(space::SIDEBAR, space::S40 * 2.0), Sense::hover());
            ui.painter().rect_filled(egui::Rect::from_min_size(rect.min, egui::vec2(w, rect.height())), radius::CARD, tok::NAV_BG);
            ui.painter().text(rect.min + egui::vec2(space::S12, space::S12), Align2::LEFT_TOP, format!("{w:.0}"), fs(size::SMALL), tok::TEXT_ON_DARK);
            if !motion::reduced() {
                ui.ctx().request_repaint();
            }
        }
        // origin-aware scale: a menu growing from its anchor (S5: 300 e-spring, scale .9, y −6)
        {
            let ui = &mut cols[1];
            ui.label(RichText::new("Origin-aware scale · a menu from its anchor (300 e-spring)").font(fs(size::LABEL)).color(p.dim));
            let (area, _) = ui.allocate_exact_size(egui::vec2(space::S40 * 6.0, space::S40 * 3.0), Sense::hover());
            let anchor = area.left_top() + egui::vec2(space::S12, space::S12);
            ui.painter().circle_filled(anchor, stroke::DOT, p.orange);
            let f = frame_at(now_ms(ui) % 1600.0, timing(300.0, 0.0, E_SPRING), motion::reduced());
            let s = 0.9 + 0.1 * f.t;
            let menu = egui::Rect::from_min_size(anchor + egui::vec2(0.0, space::S8), egui::vec2(space::S40 * 4.0, space::S40 * 2.0));
            let r = motion::scale_about(menu, anchor, s).translate(egui::vec2(0.0, -6.0 * (1.0 - f.t)));
            paint_shadows(ui.painter(), r, radius::CARD, &shadow::CARD_LIFT);
            ui.painter().rect(r, radius::CARD, p.card.gamma_multiply(f.alpha.max(0.2)), Stroke::new(stroke::HAIR, p.line));
            for i in 0..3 {
                let row = egui::Rect::from_min_size(r.min + egui::vec2(space::S12, space::S12 + i as f32 * space::S18 * s), egui::vec2((r.width() - space::S24) * 0.8, space::S8));
                ui.painter().rect_filled(row, radius::GATE / 2.0, p.track.gamma_multiply(f.alpha));
            }
        }
        // stagger: rows 300 e-out, delay index × 60 (A3); press Replay motion to run again
        {
            let ui = &mut cols[2];
            ui.label(RichText::new("Stagger · rows 300 e-out, 60 apart").font(fs(size::LABEL)).color(p.dim));
            let base = timing(300.0, 0.0, E_OUT);
            let el = motion::elapsed_ms(ui.ctx(), egui::Id::new("g-stagger"));
            for i in 0..5 {
                let f = frame_at(el, base.stagger(i, 60.0), motion::reduced());
                if !f.done {
                    ui.ctx().request_repaint();
                }
                motion::fade_in(ui, f, egui::vec2(0.0, 10.0), |ui| {
                    egui::Frame::none().fill(p.card).stroke(Stroke::new(stroke::HAIR, p.line)).rounding(Rounding::same(radius::SMALL_BUTTON)).inner_margin(egui::Margin::symmetric(space::S10, space::S6)).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.label(RichText::new(format!("Row {}", i + 1)).size(size::SMALL).color(p.text));
                    });
                });
                ui.add_space(space::S4);
            }
        }
    });
    ui.add_space(space::S16);
    // keyframes: the bridge glow opacity (B0–B4): 0–23% .35, 30% .85, 72% .85, 100% .35
    ui.label(RichText::new("Keyframe track · the bridge glow, one 9200 ms clock").font(fs(size::LABEL)).color(p.dim));
    let keys: [Key; 5] = [key(0.0, 0.35, LINEAR), key(0.23 * 9200.0, 0.35, LINEAR), key(0.30 * 9200.0, 0.85, E_STD), key(0.72 * 9200.0, 0.85, LINEAR), key(9200.0, 0.35, E_IN)];
    let w = ui.available_width().min(space::S40 * 18.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(w, space::S40 + space::S8), Sense::hover());
    ui.painter().rect_filled(rect, radius::CHIP, p.surface_raised);
    let pts: Vec<egui::Pos2> = (0..=100).map(|i| {
        let ms = 9200.0 * i as f32 / 100.0;
        egui::pos2(rect.left() + rect.width() * i as f32 / 100.0, rect.bottom() - rect.height() * motion::track(&keys, ms, false))
    }).collect();
    ui.painter().add(Shape::line(pts, Stroke::new(stroke::LINE, p.accent)));
    let ms = now_ms(ui) % 9200.0;
    let v = motion::track(&keys, ms, motion::reduced());
    let x = if motion::reduced() { rect.right() } else { rect.left() + rect.width() * ms / 9200.0 };
    ui.painter().circle_filled(egui::pos2(x, rect.bottom() - rect.height() * v), stroke::DOT, p.orange);
}

fn gallery_glow(ui: &mut egui::Ui, p: &Pal) {
    section(ui, p, "E17 · B-E20", "Glow and bloom (gradient mesh)");
    wrap_label(ui, RichText::new("egui has no gradients or blur, so the hero glow and the reveal bloom are meshes with per-vertex colour. The bloom grows .4 → 6 → 11 during the reveal.").size(size::SMALL).color(p.dim));
    ui.add_space(space::S12);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), space::S40 * 5.0), Sense::hover());
    ui.painter().rect_filled(rect, radius::HERO, tok::INK);
    let painter = ui.painter().with_clip_rect(rect);
    for (i, s) in [0.4_f32, 2.0, 6.0].iter().enumerate() {
        let c = egui::pos2(rect.left() + rect.width() * (0.18 + 0.27 * i as f32), rect.center().y);
        bloom(&painter, c, space::S16, *s, 0.9);
        painter.text(egui::pos2(c.x, rect.bottom() - space::S12), Align2::CENTER_BOTTOM, format!("bloom × {s}"), fs(size::SMALL), tok::TEXT_ON_DARK_SOFT);
    }
    // animated: the reveal's bloom .4 → 6 → 11 over 30–58% of a 6500 clock (looping here)
    let ms = (ui.ctx().input(|i| i.time) * 1000.0) as f32 % 6500.0;
    let keys = [key(0.0, 0.4, LINEAR), key(0.30 * 6500.0, 0.4, LINEAR), key(0.44 * 6500.0, 6.0, E_OUT), key(0.58 * 6500.0, 11.0, E_OUT), key(6500.0, 11.0, LINEAR)];
    let s = motion::track(&keys, ms, motion::reduced());
    let c = egui::pos2(rect.right() - rect.width() * 0.08, rect.center().y);
    bloom(&painter, c, space::S8, s, 0.8);
    painter.text(egui::pos2(c.x, rect.bottom() - space::S12), Align2::CENTER_BOTTOM, "reveal, live", fs(size::SMALL), tok::TEXT_ON_DARK_SOFT);
    if !motion::reduced() {
        ui.ctx().request_repaint();
    }
}
