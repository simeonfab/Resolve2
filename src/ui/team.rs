//! Collaboration: the AI Team (one engine shared with Ask AI) and Handoffs.
//!
//! Ported from Control Centre (build-me-a-single-productivity/app/src/ui/sections.rs:
//! `ai_team_view`, `team_view`, `conversation_for`, `poll_ai_jobs` and the Handoffs list).
//! Changed for Resolve 2: conversations live in Resolve 2's WorkspaceData; the per-role AI picker
//! and role editor were not ported (every role uses the AI chosen in Settings); handoff briefs
//! come from Resolve 2's generator (`roles::handoff_brief`) and are kept as Handoff records.

use super::theme::*;
use super::widgets::*;
use super::*;
use crate::resolve::team::{self, GENERAL, TEAM};
use crate::resolve::{roles, Kind, Record};
use eframe::egui::{self, RichText, Sense, Stroke};

impl App {
    pub(super) fn conversation_for(&mut self, pid: Option<&str>, agent: &str) -> Id {
        if let Some(c) = self.store.data.conversations.iter().filter(|c| c.agent == agent && c.project_id.as_deref() == pid).max_by_key(|c| c.updated) {
            return c.id.clone();
        }
        let c = Conversation { id: new_id(), project_id: pid.map(|s| s.to_string()), agent: agent.to_string(), title: team::specialist(agent).name.to_string(), messages: vec![], updated: now_ms() };
        let id = c.id.clone();
        self.store.data.conversations.push(c);
        id
    }

    /// Who may change the workspace: Ask AI's checkbox for the General assistant, the safe
    /// default for the provider for the specialists.
    pub(super) fn role_can_change(&self, agent: &str) -> bool {
        if agent == GENERAL {
            self.page_ui.ai_can_act
        } else {
            team::may_change(&self.ai_config().provider)
        }
    }

    /// Send a message to a role (Ask AI sends to the General assistant). `extra` is added to the
    /// request (e.g. the open page). Returns false if another answer is still running.
    pub(super) fn team_send(&mut self, agent: &str, msg: &str, extra: &str, label: &str) -> bool {
        if self.resolve_ui.chat_job.is_some() || msg.trim().is_empty() {
            return false;
        }
        let scope = self.current_project.clone();
        let cid = self.conversation_for(scope.as_deref(), agent);
        let conv = self.store.data.conversations.iter().find(|c| c.id == cid).cloned().unwrap_or_default();
        let history: String = conv
            .messages
            .iter()
            .rev()
            .take(10)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .map(|m| format!("{}: {}", if m.role == "user" { "User".to_string() } else { team::specialist(&m.role).name.to_string() }, m.text))
            .collect::<Vec<_>>()
            .join("\n");
        if let Some(c) = self.store.data.conversations.iter_mut().find(|c| c.id == cid) {
            c.messages.push(ChatMessage { role: "user".into(), text: msg.trim().to_string(), at: now_ms(), ..Default::default() });
            c.updated = now_ms();
        }
        self.save_now();
        let system = team::role_system(&self.store.data, agent, scope.as_deref());
        let mut prompt = format!("Today is {}.\n", today().format("%A %-d %B %Y"));
        if !extra.is_empty() {
            prompt.push_str(extra);
            prompt.push('\n');
        }
        let hits = crate::search::search(&self.store.data, msg, 6);
        if !hits.is_empty() {
            prompt.push_str("Possibly related items:\n");
            for h in hits {
                prompt.push_str(&format!("- [{}] {} (id {}) — {}\n", h.kind.label(), h.title, h.id, h.snippet));
            }
        }
        prompt.push_str(&format!("\nConversation so far:\n{history}\n\nUser: {}", msg.trim()));
        let cfg = self.ai_config();
        let can_change = self.role_can_change(agent);
        let tool_capable = matches!(cfg.provider.as_str(), "anthropic" | "openai-cloud" | "claude-code" | "codex-cli") || can_change;
        self.resolve_ui.chat_started_ms = now_ms();
        let job = if tool_capable { crate::ai::ask_agent(cfg, &system, &prompt, self.store.meta.clone(), can_change, label) } else { crate::ai::ask(&cfg.provider, &cfg.url, &cfg.model, &system, &prompt, label) };
        self.resolve_ui.chat_job = Some((cid, job));
        self.resolve_ui.chat_started = self.now;
        self.resolve_ui.chat_via = self.provider_label();
        true
    }

    /// Collect a finished answer every frame, whichever screen is open.
    pub(super) fn poll_team(&mut self, ctx: &egui::Context) {
        let finished = match &self.resolve_ui.chat_job {
            Some((cid, job)) => job.poll().map(|res| (cid.clone(), res)),
            None => None,
        };
        if let Some((cid, res)) = finished {
            let (text, error) = match res {
                Ok(t) if t.trim().is_empty() => ("The AI finished without saying anything. Try asking again, or pick a different AI in Settings.".to_string(), true),
                Ok(t) => (t, false),
                Err(e) => (e, true),
            };
            // the role may have changed things through its tools: pick them up, offer Undo
            self.store.reload_if_changed();
            let since = self.resolve_ui.chat_started_ms;
            let ids: Vec<String> = self.store.data.history.iter().filter(|c| c.created >= since && !c.undone && c.label.starts_with("AI:")).map(|c| c.id.clone()).collect();
            if !ids.is_empty() {
                let n = ids.len();
                self.undo_toasts.push((format!("The AI made {n} change(s)"), self.now + 10.0, ids.join(",")));
            }
            let via = std::mem::take(&mut self.resolve_ui.chat_via);
            if let Some(c) = self.store.data.conversations.iter_mut().find(|c| c.id == cid) {
                let agent = c.agent.clone();
                c.messages.push(ChatMessage { role: agent, text, at: now_ms(), via, error });
                c.updated = now_ms();
            }
            self.resolve_ui.chat_job = None;
            self.touch();
        }
        if self.resolve_ui.chat_job.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(250));
        }
    }

    /// The messages of a conversation, with the "thinking…" line while an answer is running.
    pub(super) fn chat_messages(&mut self, ui: &mut egui::Ui, p: &Pal, cid: &str, max_h: f32) {
        let conv = self.store.data.conversations.iter().find(|c| c.id == cid).cloned().unwrap_or_default();
        let busy_here = self.resolve_ui.chat_job.as_ref().map_or(false, |(c, _)| c == cid);
        let mut stop = false;
        egui::ScrollArea::vertical().id_salt(("chat", cid)).max_height(max_h).stick_to_bottom(true).show(ui, |ui| {
            if conv.messages.is_empty() {
                wrap_label(ui, RichText::new(format!("Ask a question or give an instruction. The {} sees this project's context, roadmap, tasks, hypotheses, evidence and decisions.", team::specialist(&conv.agent).name)).size(size::SMALL).color(p.faint));
            }
            for m in &conv.messages {
                let mine = m.role == "user";
                let err = m.error;
                let fill = if err { p.accent_soft } else if mine { p.hover } else { p.bg };
                egui::Frame::none().fill(fill).stroke(if err { Stroke::new(1.0, p.orange) } else { Stroke::NONE }).rounding(space::RADIUS_SMALL).inner_margin(egui::Margin::same(8.0)).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.vertical(|ui| {
                        let author = if mine { "You".to_string() } else { team::specialist(&m.role).name.to_string() };
                        ui.label(RichText::new(if err { format!("{author} · couldn't answer") } else { author }).size(size::TINY).color(if err { p.orange } else { p.dim }));
                        ui.add(egui::Label::new(RichText::new(&m.text).size(size::BODY)).wrap().selectable(true));
                        if !m.via.is_empty() {
                            ui.label(RichText::new(&m.via).size(size::TINY).color(p.faint));
                        }
                    });
                });
                ui.add_space(4.0);
            }
            if busy_here {
                let secs = (self.now - self.resolve_ui.chat_started).max(0.0) as u64;
                ui.horizontal_wrapped(|ui| {
                    ui.spinner();
                    ui.label(RichText::new(format!("{} is thinking… {secs} s", team::specialist(&conv.agent).name)).color(p.dim));
                    if ui.small_button("Stop waiting").clicked() {
                        stop = true;
                    }
                });
            }
        });
        if stop {
            if let Some(c) = self.store.data.conversations.iter_mut().find(|c| c.id == cid) {
                c.messages.push(ChatMessage { role: c.agent.clone(), text: "Stopped waiting for this answer. If it finishes, changes it made will still appear.".into(), at: now_ms(), via: String::new(), error: true });
                c.updated = now_ms();
            }
            self.resolve_ui.chat_job = None;
            self.touch();
        }
    }

    pub(super) fn team_view(&mut self, ui: &mut egui::Ui, p: &Pal) {
        if self.resolve_ui.agent.is_empty() || !TEAM.iter().any(|r| r.id == self.resolve_ui.agent) {
            self.resolve_ui.agent = "product-director".into();
        }
        ui.horizontal_wrapped(|ui| {
            for sp in TEAM.iter() {
                let on = self.resolve_ui.agent == sp.id;
                let r = egui::Frame::none().fill(if on { p.accent_soft } else { p.card }).stroke(Stroke::new(1.0, if on { p.accent } else { p.line })).rounding(space::RADIUS_SMALL).inner_margin(egui::Margin::symmetric(10.0, 6.0)).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        avatar(ui, sp.avatar, if on { p.accent } else { p.purple }, 24.0);
                        ui.vertical(|ui| {
                            ui.label(RichText::new(sp.name).font(fs(13.0)));
                            ui.label(RichText::new(sp.tag).size(size::TINY).color(p.dim));
                        });
                    });
                });
                if ui.interact(r.response.rect, ui.id().with(("sp", sp.id)), Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                    self.resolve_ui.agent = sp.id.to_string();
                }
            }
        });
        ui.add_space(8.0);
        let agent = self.resolve_ui.agent.clone();
        let sp = team::specialist(&agent);
        let scope = self.current_project.clone();
        let cid = self.conversation_for(scope.as_deref(), &agent);
        let scope_name = scope.as_deref().and_then(|x| self.store.data.project(x)).map(|x| x.name.clone()).unwrap_or_else(|| "all projects".into());
        let can_change = self.role_can_change(&agent);
        let has_msgs = self.store.data.conversations.iter().find(|c| c.id == cid).map_or(false, |c| !c.messages.is_empty());
        let busy_any = self.resolve_ui.chat_job.is_some();
        let busy_here = self.resolve_ui.chat_job.as_ref().map_or(false, |(c, _)| c == &cid);
        let mut send = false;
        let mut clear = false;
        let input_id = egui::Id::new("team-chat-input");
        let via = self.provider_label();
        card(ui, p, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new(format!("{} · {scope_name}", sp.name)).font(fs(13.5)).color(p.dim));
                if agent == GENERAL {
                    ui.label(RichText::new("· same conversation as Ask AI").size(size::TINY).color(p.faint));
                }
                if has_msgs && ghost_button(ui, RichText::new("New conversation").size(size::SMALL).color(p.dim)).on_hover_text("Start fresh. The previous conversation is kept in the workspace.").clicked() {
                    clear = true;
                }
            });
            let power = if can_change { "can make changes in your workspace (you can undo them)" } else { "read only: it can look things up but not change anything" };
            wrap_label(ui, RichText::new(format!("Answers with {via} · {power}")).size(size::TINY).color(p.faint));
            ui.add_space(4.0);
            self.chat_messages(ui, p, &cid, 380.0);
            // Enter sends, Shift+Enter starts a new line
            let focused = ui.memory(|m| m.has_focus(input_id));
            let enter = focused
                && ui.input_mut(|i| {
                    let hit = i.events.iter().any(|e| matches!(e, egui::Event::Key { key: egui::Key::Enter, pressed: true, modifiers, .. } if !modifiers.shift));
                    if hit {
                        i.events.retain(|e| !matches!(e, egui::Event::Key { key: egui::Key::Enter, pressed: true, modifiers, .. } if !modifiers.shift));
                    }
                    hit
                });
            if enter {
                send = true;
            }
            ui.horizontal(|ui| {
                let hint = format!("Message the {} (Enter to send, Shift+Enter for a new line)", sp.name);
                ui.add(egui::TextEdit::multiline(&mut self.resolve_ui.chat_input).id(input_id).hint_text(hint).desired_rows(2).desired_width(ui.available_width() - 80.0));
                if ui.add_enabled(!busy_any, egui::Button::new(RichText::new("Send").color(p.on_accent)).fill(p.accent).min_size(egui::vec2(0.0, 30.0))).on_disabled_hover_text("Waiting for the current answer").clicked() {
                    send = true;
                }
            });
            if busy_any && !busy_here {
                ui.label(RichText::new("Another role is still answering; send when it's done.").size(size::TINY).color(p.faint));
            }
        });
        if clear {
            let c = Conversation { id: new_id(), project_id: scope.clone(), agent: agent.clone(), title: sp.name.to_string(), messages: vec![], updated: now_ms() + 1 };
            self.store.data.conversations.push(c);
            self.touch();
        }
        if send && !busy_any {
            let msg = std::mem::take(&mut self.resolve_ui.chat_input);
            self.team_send(&agent, &msg, "", "team");
        }
    }

    // ------------------------------------------------------------ Handoffs

    pub(super) fn handoffs_view(&mut self, ui: &mut egui::Ui, p: &Pal) {
        let scope = self.current_project.clone();
        let mut make = false;
        let mut list: Vec<Record> = self.store.data.records.iter().filter(|r| r.kind == Kind::Handoff && scope.as_ref().map_or(true, |s| r.project_id.as_ref() == Some(s))).cloned().collect();
        list.sort_by_key(|r| std::cmp::Reverse(r.created));
        card(ui, p, |ui| {
            ui.label(RichText::new("New handoff").font(fs(size::SECTION)));
            if scope.is_some() {
                wrap_label(ui, RichText::new("Written from the project's real state: outcome, decisions, open questions, the cut list and the next tasks. Claude and Codex can also read and update this workspace over MCP (Settings → AI & MCP).").size(size::TINY).color(p.faint));
                ui.horizontal_wrapped(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.resolve_ui.handoff_for).hint_text("For (Claude Code, Codex, Sam…)").desired_width(200.0));
                    ui.add(egui::TextEdit::singleline(&mut self.resolve_ui.handoff_focus).hint_text("What they should do").desired_width(340.0));
                    if primary_button(ui, p, "Create brief").clicked() {
                        make = true;
                    }
                });
            } else {
                ui.label(RichText::new("Pick a project in the sidebar to write a handoff from its context.").size(size::SMALL).color(p.dim));
            }
        });
        // the live brief (copy it, or save it as a page), as it was on the old Handoff tab
        if let Some(pid) = scope.clone() {
            if let Some(ctx) = self.ensure_context(&pid) {
                ui.add_space(6.0);
                egui::CollapsingHeader::new(RichText::new("Current brief (always up to date)").size(size::SMALL).color(p.dim)).id_salt(("live-brief", &pid)).show(ui, |ui| {
                    self.cockpit_handoff(ui, p, &pid, &ctx);
                });
            }
        }
        ui.add_space(8.0);
        if list.is_empty() {
            ui.label(RichText::new("No saved handoffs yet.").size(size::SMALL).color(p.faint));
        }
        let mut copy: Option<String> = None;
        let mut pick: Option<Id> = None;
        for r in &list {
            let pname = r.project_id.as_deref().and_then(|x| self.store.data.project(x)).map(|x| x.name.clone()).unwrap_or_default();
            let is_sel = self.resolve_ui.sel_record.as_deref() == Some(r.id.as_str());
            egui::Frame::none().fill(p.card).stroke(Stroke::new(1.0, if is_sel { p.accent } else { p.line })).rounding(space::RADIUS).inner_margin(egui::Margin::symmetric(12.0, 8.0)).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal_wrapped(|ui| {
                    chip(ui, &r.status, if r.status == "Done" { p.green } else { p.dim });
                    if wrap_link(ui, RichText::new(&r.title).font(fs(size::BODY))).clicked() {
                        pick = Some(r.id.clone());
                    }
                    if scope.is_none() {
                        ui.label(RichText::new(&pname).size(size::TINY).color(p.faint));
                    }
                    let when = chrono::DateTime::from_timestamp_millis(r.created).map(|x| x.with_timezone(&chrono::Local).format("%-d %b %H:%M").to_string()).unwrap_or_default();
                    ui.label(RichText::new(when).size(size::TINY).color(p.faint));
                    if ui.small_button("Copy brief").clicked() {
                        copy = Some(r.body.clone());
                    }
                });
            });
            ui.add_space(4.0);
            if is_sel {
                if let Some(pid) = r.project_id.clone() {
                    let id = r.id.clone();
                    self.record_editor(ui, p, &pid, &id);
                }
                egui::Frame::none().fill(p.code).rounding(space::RADIUS_SMALL).inner_margin(12.0).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.add(egui::Label::new(RichText::new(&r.body).monospace().size(size::SMALL)).wrap().selectable(true));
                });
                ui.add_space(8.0);
            }
        }
        if let Some(b) = copy {
            ui.ctx().copy_text(b);
            self.toast("Handoff brief copied");
        }
        if let Some(id) = pick {
            self.resolve_ui.sel_record = if self.resolve_ui.sel_record.as_deref() == Some(id.as_str()) { None } else { Some(id) };
        }
        if make {
            if let Some(pid) = scope {
                let Some(ctx) = self.ensure_context(&pid) else { return };
                let to = std::mem::take(&mut self.resolve_ui.handoff_for).trim().to_string();
                let focus = std::mem::take(&mut self.resolve_ui.handoff_focus).trim().to_string();
                let for_whom = if to.is_empty() { String::new() } else { format!(" for {to}") };
                let mut body = format!("# Handoff{for_whom}\n\n");
                if !focus.is_empty() {
                    body.push_str(&format!("## What to do\n{focus}\n\n"));
                }
                body.push_str(&roles::handoff_brief(&self.store.data, &ctx));
                let mut r = Record::new(Kind::Handoff, Some(pid.clone()), format!("Handoff{for_whom}"));
                r.body = body;
                r.fields.insert("to".into(), to);
                r.fields.insert("focus".into(), focus);
                self.resolve_ui.sel_record = Some(r.id.clone());
                self.store.data.records.push(r);
                self.touch();
            }
        }
    }
}
