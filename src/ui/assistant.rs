//! The ✨ Ask AI panel: the AI Team's General assistant, in the same engine and the same
//! conversation as on the AI Team screen (one conversation per project, or for the whole
//! workspace). Its placeholder and suggestions change with the screen you're on (Ask Resolve,
//! from nav.rs). With "Let AI make changes" on, it acts in the workspace through the same tools
//! Claude/ChatGPT use over MCP, and every change can be undone.

use super::theme::*;
use super::widgets::*;
use super::*;
use crate::resolve::team::GENERAL;
use eframe::egui::{self, RichText, Stroke};

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

    pub fn ai_panel(&mut self, ctx: &egui::Context, p: &Pal, pid: Option<String>) {
        let page = pid.as_deref().and_then(|id| self.store.data.page(id)).cloned();
        let page_md = page.as_ref().map(|pg| blocks_to_markdown(pg, &self.store.data)).unwrap_or_default();
        let ask_copy = self.ask_copy();
        let scope = self.current_project.clone();
        let cid = self.conversation_for(scope.as_deref(), GENERAL);
        let busy = self.resolve_ui.chat_job.is_some();
        let last_reply: Option<String> = self.store.data.conversations.iter().find(|c| c.id == cid).and_then(|c| c.messages.last()).filter(|m| m.role != "user" && !m.error).map(|m| m.text.clone());
        let mut ask: Option<(String, String)> = None;
        let mut insert = false;
        let mut make_tasks = false;
        egui::SidePanel::right("ai-panel").default_width(380.0).width_range(320.0..=600.0).frame(egui::Frame::none().fill(p.card).inner_margin(egui::Margin::same(14.0)).stroke(Stroke::new(1.0, p.line))).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("✨ Ask AI").font(fb(16.0)));
                ui.label(RichText::new("General assistant").size(size::SMALL).color(p.dim));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ghost_button(ui, "✕").clicked() {
                        self.page_ui.ai_open = false;
                    }
                    if ghost_button(ui, RichText::new("⚙").color(p.dim)).on_hover_text("Choose Claude, ChatGPT or a local model").clicked() {
                        self.settings_ui.tab = 0;
                        self.go(View::Settings);
                    }
                });
            });
            wrap_label(ui, RichText::new(self.provider_label()).size(size::TINY).color(p.dim));
            ui.checkbox(&mut self.page_ui.ai_can_act, "Let AI make changes (create tasks, edit pages, add rows…)");
            let working_on = match (&page, scope.as_deref().and_then(|x| self.store.data.project(x))) {
                (Some(pg), _) => format!("Working on: {} {}", pg.icon, if pg.title.is_empty() { "Untitled" } else { &pg.title }),
                (None, Some(pr)) => format!("Working on: {}", pr.name),
                _ => "Working on: your whole workspace".into(),
            };
            wrap_label(ui, RichText::new(working_on).size(size::SMALL).color(p.faint));
            ui.add_space(4.0);
            // contextual suggestions (Ask Resolve), plus the page actions when a page is open
            ui.horizontal_wrapped(|ui| {
                for s in ask_copy.suggestions {
                    if ui.add_enabled(!busy, egui::Button::new(RichText::new(*s).size(size::SMALL))).clicked() {
                        ask = Some(("Answer".into(), s.to_string()));
                    }
                }
                if page.is_some() {
                    if ui.button("📝 Summarise").clicked() {
                        ask = Some(("Summary".into(), "Summarise the current page in 3-6 bullet points.".into()));
                    }
                    if ui.button("✍ Continue writing").clicked() {
                        ask = Some(("Continue".into(), "Continue writing the current page in the same style, and add what you write to the end of the page.".into()));
                    }
                    if ui.button("✨ Improve writing").clicked() {
                        ask = Some(("Rewrite".into(), "Rewrite the current page to be clearer and more concise, keeping its structure, and save the result to the page.".into()));
                    }
                    if ui.button("✔ Turn into tasks").clicked() {
                        ask = Some(("Actions".into(), "Find the concrete action items in the current page and create a task for each one, linked to the page, with dates where the page implies them. Then list what you created.".into()));
                    }
                }
                if ui.button("🗂 Organise my to-dos").on_hover_text("Moves the to-dos written in your pages into projects as tasks with dates, linked back to the pages").clicked() {
                    ask = Some(("Organise".into(), crate::ai::ORGANISE_PROMPT.into()));
                }
                if ui.button("🗓 Weekly review").clicked() {
                    ask = Some(("Review".into(), "Write a short weekly review from what I completed this week, what's overdue and what's coming up, and save it as a new page called 'Weekly review' with today's date.".into()));
                }
            });
            ui.add_space(6.0);
            let ask_id = egui::Id::new("ai-ask-box");
            let r = ui.add(egui::TextEdit::multiline(&mut self.page_ui.ai_prompt).id(ask_id).hint_text(format!("{} (press 🎤 to speak)", ask_copy.placeholder)).desired_rows(3).desired_width(f32::INFINITY));
            let enter = r.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.command);
            ui.horizontal(|ui| {
                if (ui.add_enabled(!busy, egui::Button::new(RichText::new("Ask").color(p.on_accent)).fill(p.accent)).clicked() || enter) && !self.page_ui.ai_prompt.trim().is_empty() {
                    ask = Some(("Answer".into(), self.page_ui.ai_prompt.clone()));
                }
                mic_for(ui, p, ask_id, p.dark);
                ui.label(RichText::new("Ctrl+Enter").size(size::TINY).color(p.faint));
            });
            ui.add_space(8.0);
            let h = (ui.available_height() - 80.0).max(120.0);
            self.chat_messages(ui, p, &cid, h);
            if last_reply.is_some() {
                ui.horizontal_wrapped(|ui| {
                    if page.is_some() && ui.button("⤓ Insert into page").clicked() {
                        insert = true;
                    }
                    if ui.button("✔ Make tasks from list").clicked() {
                        make_tasks = true;
                    }
                    if ui.button("📋 Copy").clicked() {
                        ui.ctx().copy_text(last_reply.clone().unwrap_or_default());
                    }
                });
            }
            wrap_label(ui, RichText::new("This is the AI Team's General assistant: the same conversation is on the AI Team screen.").size(size::TINY).color(p.faint));
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
    }
}
