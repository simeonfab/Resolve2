//! The Ask sheet (A3, opened by the Ask Resolve bar or the sidebar's Ask AI): the AI Team's General assistant, in the same engine and the same
//! conversation as on the AI Team screen (one conversation per project, or for the whole
//! workspace). Its placeholder and suggestions change with the screen you're on (Ask Resolve,
//! from nav.rs). With "Let AI make changes" on, it acts in the workspace through the same tools
//! Claude/ChatGPT use over MCP, and every change can be undone.

use super::theme::*;
use super::widgets::*;
use super::*;
use crate::resolve::team::GENERAL;
use super::motion::{self, frame_at};
use eframe::egui::{self, RichText};

impl App {
    pub(super) fn ai_config(&self) -> crate::ai::AiConfig {
        let s = &self.reg.settings;
        let key = match s.ai_provider.as_str() {
            "anthropic" => s.anthropic_key.clone(),
            "openai-cloud" => s.openai_key.clone(),
            _ => String::new(),
        };
        crate::ai::AiConfig { provider: s.ai_provider.clone(), url: s.ai_url.clone(), model: s.ai_model.clone(), key }
    }

    pub fn provider_label(&self) -> String {
        match self.reg.settings.ai_provider.as_str() {
            "claude-code" => "Claude: your subscription (via Claude Code, no extra cost)".to_string(),
            "codex-cli" => "ChatGPT: your subscription (via Codex, no extra cost)".to_string(),
            "anthropic" => format!("Claude API · {}", self.reg.settings.ai_model),
            "openai-cloud" => format!("ChatGPT (OpenAI) · {}", self.reg.settings.ai_model),
            "openai" => format!("Local (OpenAI-compatible) · {}", self.reg.settings.ai_model),
            _ => format!("Ollama (local) · {}", self.reg.settings.ai_model),
        }
    }

    /// Send a question to the General assistant from the bar or the sheet, with the open page
    /// as context. Says so if another answer is still running.
    pub(super) fn ask_send(&mut self, q: &str) {
        let page = match &self.view {
            View::Page(id) => self.store.data.page(id).cloned(),
            _ => None,
        };
        let extra = match &page {
            Some(pg) => format!("The user has this page open (id {}):\n---\n{}\n---", pg.id, blocks_to_markdown(pg, &self.store.data)),
            None => String::new(),
        };
        if self.team_send(GENERAL, q, &extra, "Answer") {
            self.page_ui.ai_prompt.clear();
        } else {
            self.toast("Another answer is still running. Try again when it's done.");
        }
    }

    /// The Ask sheet: rises from the bottom edge of the work area, 720 wide, 82% of the window
    /// high, over a .32 backdrop. Esc, × or a click on the backdrop closes it.
    pub fn ask_sheet(&mut self, ctx: &egui::Context, p: &Pal) {
        let open = self.page_ui.ai_open;
        if open != self.ask.was_open {
            self.ask.was_open = open;
            self.ask.moved_at = self.now;
            self.ask.focus = open;
        }
        let el = ((self.now - self.ask.moved_at) * 1000.0) as f32;
        let reduced = motion::reduced();
        // (backdrop opacity, how far down the sheet is (0 = in place), sheet opacity, live?)
        let (dim, down, alpha, live) = if open {
            let d = frame_at(el, moves::SHEET_DIM, reduced);
            let s = frame_at(el, moves::SHEET_IN, reduced);
            if !(d.done && s.done) {
                ctx.request_repaint();
            }
            (d.alpha, 1.0 - s.t, if reduced { s.alpha } else { 1.0 }, true)
        } else {
            let s = frame_at(el, moves::SHEET_OUT, reduced);
            let d = frame_at(el, moves::SHEET_DIM_OUT, reduced);
            if s.done && d.done {
                return;
            }
            ctx.request_repaint();
            (1.0 - d.alpha, s.t, if reduced { 1.0 - s.alpha } else { 1.0 }, false)
        };
        let screen = ctx.screen_rect();
        let work = self.toaster.work.filter(|r| r.is_positive()).unwrap_or(screen);
        let w = space::ASK_SHEET_W.min(work.width() - space::S24 * 2.0);
        let h = (screen.height() * space::ASK_SHEET_HEIGHT).min(work.height());
        let top = work.bottom() - h + h * down.max(0.0);
        let rect = egui::Rect::from_min_size(egui::pos2(work.center().x - w / 2.0, top), egui::vec2(w, h));
        let mut close = false;
        egui::Area::new(egui::Id::new("ask-dim")).order(egui::Order::Foreground).fixed_pos(screen.min).interactable(live).show(ctx, |ui| {
            close = backdrop(ui, screen, shadow::BACKDROP_SHEET * dim.clamp(0.0, 1.0));
        });
        let sheet_id = egui::Id::new("ask-sheet");
        egui::Area::new(sheet_id).order(egui::Order::Foreground).fixed_pos(rect.min).interactable(live).constrain(false).show(ctx, |ui| {
            ui.multiply_opacity(alpha.clamp(0.0, 1.0));
            let frame = sheet_frame(p);
            let m = frame.inner_margin;
            frame.show(ui, |ui| {
                ui.set_width(w - m.left - m.right);
                ui.set_height(h - m.top - m.bottom);
                if self.ask_sheet_body(ui, p, el, live) {
                    close = true;
                }
            });
        });
        ctx.move_to_top(egui::LayerId::new(egui::Order::Foreground, sheet_id));
        if close && live {
            self.page_ui.ai_open = false;
        }
    }

    /// The sheet's rows (300 e-out, stagger 60): header, suggestions, the conversation, then the
    /// box. Returns true when × is pressed.
    fn ask_sheet_body(&mut self, ui: &mut egui::Ui, p: &Pal, el: f32, live: bool) -> bool {
        let pid = match &self.view {
            View::Page(id) => Some(id.clone()),
            _ => None,
        };
        let page = pid.as_deref().and_then(|id| self.store.data.page(id)).cloned();
        let page_md = page.as_ref().map(|pg| blocks_to_markdown(pg, &self.store.data)).unwrap_or_default();
        let ask_copy = self.ask_copy();
        let scope = self.current_project.clone();
        let cid = self.conversation_for(scope.as_deref(), GENERAL);
        let busy = self.resolve_ui.chat_job.is_some();
        let conv = self.store.data.conversations.iter().find(|c| c.id == cid).cloned().unwrap_or_default();
        let last_reply: Option<String> = conv.messages.last().filter(|m| m.role != "user" && !m.error).map(|m| m.text.clone());
        let last_error = conv.messages.last().map_or(false, |m| m.error);
        let last_question: Option<String> = conv.messages.iter().rev().find(|m| m.role == "user").map(|m| m.text.clone());
        let state = match self.view_state {
            ViewState::Full if busy => ViewState::Loading,
            ViewState::Full if last_error => ViewState::Error,
            ViewState::Full if conv.messages.is_empty() => ViewState::Empty,
            s => s,
        };
        let row = |i: usize| if live { frame_at(el, moves::SHEET_ROWS.stagger(i, moves::SHEET_ROWS_STAGGER), motion::reduced()) } else { frame_at(f32::MAX, moves::SHEET_ROWS, false) };
        let rise = egui::vec2(0.0, space::S8);
        let mut ask: Option<(String, String)> = None;
        let mut insert = false;
        let mut make_tasks = false;
        let mut close = false;
        // header
        motion::fade_in(ui, row(0), rise, |ui| {
            ui.horizontal(|ui| {
                icon_widget(ui, Icon::Spark, tok::AMBER);
                ui.add_space(space::S8);
                ui.label(RichText::new("Ask Resolve").font(fserif(size::CARD_SERIF)).color(p.text));
                ui.add_space(space::S8);
                ui.label(RichText::new("General assistant").size(size::SMALL).color(p.dim));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if icon_button(ui, Icon::Close, true, p.dark).on_hover_text("Close (Esc)").clicked() {
                        close = true;
                    }
                    if icon_button(ui, Icon::Settings, true, p.dark).on_hover_text("Choose Claude, ChatGPT or a local model").clicked() {
                        self.settings_ui.tab = 0;
                        self.go(View::Settings);
                        close = true;
                    }
                });
            });
            let working_on = match (&page, scope.as_deref().and_then(|x| self.store.data.project(x))) {
                (Some(pg), _) => format!("Working on: {} {}", pg.icon, if pg.title.is_empty() { "Untitled" } else { &pg.title }),
                (None, Some(pr)) => format!("Working on: {}", pr.name),
                _ => "Working on: your whole workspace".into(),
            };
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new(working_on).size(size::SMALL).color(p.dim));
                ui.label(RichText::new("·").size(size::SMALL).color(p.faint));
                ui.label(RichText::new(self.provider_label()).size(size::SMALL).color(p.faint));
            });
            ui.checkbox(&mut self.page_ui.ai_can_act, RichText::new("Let AI make changes (create tasks, edit pages, add rows…)").size(size::SMALL));
        });
        ui.add_space(space::S8);
        // suggestions for this screen, the page actions, and the workspace actions
        motion::fade_in(ui, row(1), rise, |ui| {
            ui.horizontal_wrapped(|ui| {
                for s in ask_copy.suggestions {
                    if ui.add_enabled_ui(!busy, |ui| filter_chip(ui, p, s, None, false)).inner.clicked() && !busy {
                        ask = Some(("Answer".into(), s.to_string()));
                    }
                }
                if page.is_some() {
                    for (label, kind, req) in [
                        ("Summarise", "Summary", "Summarise the current page in 3-6 bullet points."),
                        ("Continue writing", "Continue", "Continue writing the current page in the same style, and add what you write to the end of the page."),
                        ("Improve writing", "Rewrite", "Rewrite the current page to be clearer and more concise, keeping its structure, and save the result to the page."),
                        ("Turn into tasks", "Actions", "Find the concrete action items in the current page and create a task for each one, linked to the page, with dates where the page implies them. Then list what you created."),
                    ] {
                        if filter_chip(ui, p, label, None, false).clicked() && !busy {
                            ask = Some((kind.into(), req.into()));
                        }
                    }
                }
                if filter_chip(ui, p, "Organise my to-dos", None, false).on_hover_text("Moves the to-dos written in your pages into projects as tasks with dates, linked back to the pages").clicked() && !busy {
                    ask = Some(("Organise".into(), crate::ai::ORGANISE_PROMPT.into()));
                }
                if filter_chip(ui, p, "Weekly review", None, false).clicked() && !busy {
                    ask = Some(("Review".into(), "Write a short weekly review from what I completed this week, what's overdue and what's coming up, and save it as a new page called 'Weekly review' with today's date.".into()));
                }
            });
        });
        ui.add_space(space::S10);
        hline(ui, p);
        // the conversation, in its four states
        let input_h = space::FIELD_H * 2.0 + space::BUTTON_H + space::S40;
        let conv_h = (ui.available_height() - input_h).max(space::S40 * 2.0);
        motion::fade_in(ui, row(2), rise, |ui| {
            ui.allocate_ui(egui::vec2(ui.available_width(), conv_h), |ui| {
                ui.set_min_height(conv_h);
                match state {
                    ViewState::Empty => {
                        ui.add_space(space::S24);
                        wrap_label(ui, RichText::new("Nothing asked yet.").font(fserif(size::CARD_SERIF)).color(p.text));
                        wrap_label(ui, RichText::new("Ask about what you're looking at, or pick a suggestion above.").size(size::BODY).color(p.dim));
                    }
                    ViewState::Loading if !busy => {
                        // screenshots of the loading state: the layout's shape, and what is happening
                        ui.add_space(space::S12);
                        skeleton_card(ui, p, 3, "The General assistant is thinking…");
                    }
                    ViewState::Error if !last_error => {
                        ui.add_space(space::S12);
                        let a = error_block(ui, p, "The AI didn't answer. Nothing you wrote is lost.", "Change AI settings");
                        if a.manual {
                            self.settings_ui.tab = 0;
                            self.go(View::Settings);
                        }
                    }
                    _ => {
                        self.chat_messages(ui, p, &cid, conv_h);
                    }
                }
            });
        });
        if state == ViewState::Error && last_error {
            let a = error_block(ui, p, "The AI didn't answer. Nothing you wrote is lost.", "Change AI settings");
            if a.retry {
                if let Some(q) = &last_question {
                    ask = Some(("Answer".into(), q.clone()));
                }
            }
            if a.manual {
                self.settings_ui.tab = 0;
                self.go(View::Settings);
                close = true;
            }
        }
        // after an answer: use it
        if last_reply.is_some() && state == ViewState::Full {
            ui.horizontal_wrapped(|ui| {
                if page.is_some() && button(ui, p, ButtonKind::Ghost, "Insert into page", true, true).clicked() {
                    insert = true;
                }
                if button(ui, p, ButtonKind::Ghost, "Make tasks from list", true, true).clicked() {
                    make_tasks = true;
                }
                if button(ui, p, ButtonKind::Ghost, "Copy", true, true).clicked() {
                    ui.ctx().copy_text(last_reply.clone().unwrap_or_default());
                    self.toast("Copied");
                }
            });
        }
        // the box
        motion::fade_in(ui, row(3), rise, |ui| {
            let ask_id = egui::Id::new("ai-ask-box");
            ui.horizontal_top(|ui| {
                let w = ui.available_width() - space::MIC - space::S10;
                let r = text_field(ui, p, ask_id, &mut self.page_ui.ai_prompt, ask_copy.placeholder, true, w);
                if live && self.ask.focus {
                    r.request_focus();
                    self.ask.focus = false;
                }
                ui.add_space(space::S10);
                mic_for(ui, p, ask_id, p.dark);
            });
            let enter = ui.memory(|m| m.has_focus(ask_id)) && ui.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.command);
            ui.horizontal(|ui| {
                let can = !busy && !self.page_ui.ai_prompt.trim().is_empty();
                if (button(ui, p, ButtonKind::Primary, "Ask", false, can).clicked() || enter) && can {
                    ask = Some(("Answer".into(), self.page_ui.ai_prompt.clone()));
                }
                ui.label(RichText::new("Ctrl+Enter").size(size::TINY).color(p.faint));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(RichText::new("The same conversation is on the AI Team screen.").size(size::TINY).color(p.faint));
                });
            });
        });
        if let Some((label, request)) = ask {
            let extra = match &page {
                Some(pg) => format!("The user has this page open (id {}):\n---\n{page_md}\n---", pg.id),
                None => String::new(),
            };
            if self.team_send(GENERAL, &request, &extra, &label) {
                self.page_ui.ai_prompt.clear();
            } else {
                self.toast("Another answer is still running. Try again when it's done.");
            }
        }
        let out = last_reply.unwrap_or_default();
        if insert {
            if let Some(id) = &pid {
                let (_, blocks) = markdown_to_blocks(&out);
                if let Some(pg) = self.store.data.page_mut(id) {
                    if pg.blocks.len() == 1 && pg.blocks[0].text.is_empty() {
                        pg.blocks.clear();
                    }
                    pg.blocks.extend(blocks);
                    pg.updated = now_ms();
                }
                self.touch();
            }
        }
        if make_tasks {
            let lines: Vec<String> = out.lines().map(|l| l.trim()).filter_map(|l| l.strip_prefix("- [ ] ").or_else(|| l.strip_prefix("- ")).or_else(|| l.strip_prefix("* "))).filter(|l| !l.starts_with('✓') && !l.starts_with('✗')).map(|s| s.to_string()).collect();
            let n = lines.len();
            for l in lines {
                self.add_task_text(&l, None, "", None, None, pid.clone());
            }
            self.toast(format!("Created {n} tasks"));
        }
        close
    }
}