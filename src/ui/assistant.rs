//! The ✨ AI panel: Notion-AI-style help anywhere in the app, powered by your own Claude or
//! ChatGPT API key, or a free local model. With "Let AI make changes" on, it acts in the
//! workspace through the same tools Claude/ChatGPT use over MCP.

use super::theme::*;
use super::widgets::*;
use super::*;
use eframe::egui::{self, RichText, Stroke};

impl App {
    fn ai_config(&self) -> crate::ai::AiConfig {
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
            "claude-code" => "Claude — your subscription (via Claude Code, no extra cost)".to_string(),
            "codex-cli" => "ChatGPT — your subscription (via Codex, no extra cost)".to_string(),
            "anthropic" => format!("Claude API · {}", self.reg.settings.ai_model),
            "openai-cloud" => format!("ChatGPT (OpenAI) · {}", self.reg.settings.ai_model),
            "openai" => format!("Local (OpenAI-compatible) · {}", self.reg.settings.ai_model),
            _ => format!("Ollama (local) · {}", self.reg.settings.ai_model),
        }
    }

    pub fn ai_panel(&mut self, ctx: &egui::Context, p: &Pal, pid: Option<String>) {
        if let Some(job) = &self.page_ui.ai_job {
            if let Some(res) = job.poll() {
                self.page_ui.ai_out = match res {
                    Ok(s) => s,
                    Err(e) => format!("⚠ {e}"),
                };
                self.page_ui.ai_job = None;
                // the assistant may have changed things: pick them up now, and offer Undo for all of them
                self.store.reload_if_changed();
                let since = self.page_ui.ai_started_ms;
                let ids: Vec<String> = self.store.data.history.iter().filter(|c| c.created >= since && !c.undone && c.label.starts_with("AI:")).map(|c| c.id.clone()).collect();
                if !ids.is_empty() {
                    let n = ids.len();
                    self.undo_toasts.push((format!("The assistant made {n} change(s)"), self.now + 10.0, ids.join(",")));
                }
            } else {
                ctx.request_repaint_after(std::time::Duration::from_millis(200));
            }
        }
        let page = pid.as_deref().and_then(|id| self.store.data.page(id)).cloned();
        let page_md = page.as_ref().map(|pg| blocks_to_markdown(pg, &self.store.data)).unwrap_or_default();
        let mut ask: Option<(String, String)> = None;
        let mut insert = false;
        let mut make_tasks = false;
        egui::SidePanel::right("ai-panel").default_width(380.0).width_range(320.0..=600.0).frame(egui::Frame::none().fill(p.card).inner_margin(egui::Margin::same(14.0)).stroke(Stroke::new(1.0, p.line))).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("✨ Ask AI").font(fb(16.0)));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ghost_button(ui, "✕").clicked() {
                        self.page_ui.ai_open = false;
                    }
                    if ghost_button(ui, RichText::new("⚙").color(p.dim)).on_hover_text("Choose Claude, ChatGPT or a local model").clicked() {
                        self.settings_ui.tab = 4;
                        self.go(View::Settings);
                    }
                });
            });
            ui.label(RichText::new(self.provider_label()).size(11.5).color(p.dim));
            ui.checkbox(&mut self.page_ui.ai_can_act, "Let AI make changes (create tasks, edit pages, add rows…)");
            ui.add_space(6.0);
            ui.label(RichText::new(match &page {
                Some(pg) => format!("Working on: {} {}", pg.icon, if pg.title.is_empty() { "Untitled" } else { &pg.title }),
                None => "Working on: your whole workspace".into(),
            }).size(12.0).color(p.faint));
            ui.horizontal_wrapped(|ui| {
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
                if ui.button("☀ Plan my day").clicked() {
                    ask = Some(("Plan".into(), "Look at today's tasks, overdue tasks and today's events, and suggest a realistic order for my day. Point out clashes.".into()));
                }
                if ui.button("⏰ What's overdue?").clicked() {
                    ask = Some(("Overdue".into(), "List my overdue tasks and suggest new dates this week; don't change anything yet.".into()));
                }
                if ui.button("🗓 Weekly review").clicked() {
                    ask = Some(("Review".into(), "Write a short weekly review from what I completed this week, what's overdue and what's coming up, and save it as a new page called 'Weekly review' with today's date.".into()));
                }
            });
            ui.add_space(6.0);
            let ask_id = egui::Id::new("ai-ask-box");
            let r = ui.add(egui::TextEdit::multiline(&mut self.page_ui.ai_prompt).id(ask_id).hint_text("Ask or tell it to do something — e.g. \"Add a follow-up with Ada on Friday 10am and link it to the Northwind deal\". Press 🎤 to speak.").desired_rows(3).desired_width(f32::INFINITY));
            if self.temp.remove("ai_mic").is_some() {
                r.request_focus();
            }
            let enter = r.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.command);
            ui.horizontal(|ui| {
                if (primary_button(ui, p, "Ask").clicked() || enter) && !self.page_ui.ai_prompt.trim().is_empty() {
                    ask = Some(("Answer".into(), self.page_ui.ai_prompt.clone()));
                }
                if mic_button(ui, p) {
                    ui.memory_mut(|m| m.request_focus(ask_id));
                    self.temp.insert("ai_mic".into(), "1".into());
                }
                ui.label(RichText::new("Ctrl+Enter").size(11.0).color(p.faint));
                if self.page_ui.ai_job.is_some() {
                    ui.spinner();
                    ui.label(RichText::new("Working…").color(p.dim));
                }
            });
            ui.add_space(8.0);
            if !self.page_ui.ai_out.is_empty() {
                egui::Frame::none().fill(p.bg).rounding(8.0).inner_margin(egui::Margin::same(10.0)).show(ui, |ui| {
                    egui::ScrollArea::vertical().max_height((ui.available_height() - 70.0).max(120.0)).show(ui, |ui| {
                        ui.add(egui::Label::new(RichText::new(&self.page_ui.ai_out).size(13.5)).wrap().selectable(true));
                    });
                });
                ui.horizontal_wrapped(|ui| {
                    if page.is_some() && ui.button("⤓ Insert into page").clicked() {
                        insert = true;
                    }
                    if ui.button("✔ Make tasks from list").clicked() {
                        make_tasks = true;
                    }
                    if ui.button("📋 Copy").clicked() {
                        ui.ctx().output_mut(|o| o.copied_text = self.page_ui.ai_out.clone());
                    }
                });
            }
            ui.add_space(6.0);
            ui.label(RichText::new("Prefer chatting in the Claude or ChatGPT app itself? Connect them in Settings → AI & MCP and they get the same powers.").size(11.0).color(p.faint));
        });
        if let Some((label, request)) = ask {
            let mut prompt = format!("Today is {}.\n", today().format("%A %-d %B %Y"));
            if let Some(pg) = &page {
                prompt.push_str(&format!("The user has this page open (id {}):\n---\n{page_md}\n---\n", pg.id));
            }
            let hits = crate::search::search(&self.store.data, &request, 6);
            if !hits.is_empty() {
                prompt.push_str("Possibly related items:\n");
                for h in hits {
                    prompt.push_str(&format!("- [{}] {} (id {}) — {}\n", h.kind.label(), h.title, h.id, h.snippet));
                }
            }
            prompt.push_str(&format!("\nRequest: {request}"));
            self.save_now();
            let cfg = self.ai_config();
            let can_act = self.page_ui.ai_can_act;
            let tool_capable = matches!(cfg.provider.as_str(), "anthropic" | "openai-cloud" | "claude-code" | "codex-cli") || can_act;
            self.page_ui.ai_started_ms = now_ms();
            self.page_ui.ai_job = Some(if tool_capable {
                crate::ai::ask_agent(cfg, crate::ai::AGENT_SYSTEM, &prompt, self.store.meta.clone(), can_act, &label)
            } else {
                crate::ai::ask(&cfg.provider, &cfg.url, &cfg.model, crate::ai::SYSTEM, &prompt, &label)
            });
            self.page_ui.ai_out.clear();
            self.page_ui.ai_for_page = pid.clone();
        }
        if insert {
            if let Some(id) = &pid {
                let (_, blocks) = markdown_to_blocks(&self.page_ui.ai_out);
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
            let lines: Vec<String> = self.page_ui.ai_out.lines().map(|l| l.trim()).filter_map(|l| l.strip_prefix("- [ ] ").or_else(|| l.strip_prefix("- ")).or_else(|| l.strip_prefix("* "))).filter(|l| !l.starts_with('✓') && !l.starts_with('✗')).map(|s| s.to_string()).collect();
            let n = lines.len();
            for l in lines {
                self.add_task_text(&l, None, "", None, None, pid.clone());
            }
            self.toast(format!("Created {n} tasks"));
        }
    }
}
