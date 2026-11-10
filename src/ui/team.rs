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

    /// No role changes the workspace from a chat (Simeon, 10 Nov 2026): every role, whatever
    /// the provider, reads with the tools and proposes; you add each change with a click, and
    /// Undo stays.
    pub(super) fn role_can_change(&self, _agent: &str) -> bool {
        false
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
        // every role proposes, and you confirm (C-E19)
        let system = format!("{system}\n\n{}", team::PROPOSE_RULE);
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
        let mut decide: Option<(usize, usize, bool)> = None;
        egui::ScrollArea::vertical().id_salt(("chat", cid)).max_height(max_h).stick_to_bottom(true).show(ui, |ui| {
            if conv.messages.is_empty() {
                wrap_label(ui, RichText::new(format!("Ask a question or give an instruction. The {} sees this project's context, roadmap, tasks, hypotheses, evidence and decisions.", team::specialist(&conv.agent).name)).size(size::SMALL).color(p.faint));
            }
            for (k, m) in conv.messages.iter().enumerate() {
                let mine = m.role == "user";
                let err = m.error;
                // a role's proposals become cards under its answer
                let (shown, props) = team::split_proposals(&m.text);
                let m = &ChatMessage { text: shown, ..m.clone() };
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
                let w = ui.available_width();
                for (j, pr) in props.iter().enumerate() {
                    ui.add_space(4.0);
                    if let Some(add) = pr_card(ui, p, pr, w) {
                        decide = Some((k, j, add));
                    }
                }
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
        if let Some((k, j, add)) = decide {
            self.decide_proposal(cid, k, j, add);
        }
        if stop {
            if let Some(c) = self.store.data.conversations.iter_mut().find(|c| c.id == cid) {
                c.messages.push(ChatMessage { role: c.agent.clone(), text: "Stopped waiting for this answer.".into(), at: now_ms(), via: String::new(), error: true });
                c.updated = now_ms();
            }
            self.resolve_ui.chat_job = None;
            self.touch();
        }
    }

    /// The redesigned AI Team for one project (C-E19, Pack C "AITeam"). Returns false with no
    /// project picked (the old view stays for all projects).
    pub(super) fn team_page(&mut self, ui: &mut egui::Ui, p: &Pal) -> bool {
        let Some(pid) = self.current_project.clone() else { return false };
        if self.resolve_ui.agent.is_empty() || !TEAM.iter().any(|r| r.id == self.resolve_ui.agent) {
            self.resolve_ui.agent = "validation".into();
        }
        let agent = self.resolve_ui.agent.clone();
        let sp = team::specialist(&agent);
        let name = self.store.data.project(&pid).map(|x| x.name.clone()).unwrap_or_default();
        let cid = self.conversation_for(Some(&pid), &agent);
        let conv = self.store.data.conversations.iter().find(|c| c.id == cid).cloned().unwrap_or_default();
        let earlier: Vec<(Id, String, String)> = {
            let mut v: Vec<&Conversation> = self.store.data.conversations.iter().filter(|c| c.agent == agent && c.project_id.as_deref() == Some(pid.as_str()) && c.id != cid && !c.messages.is_empty()).collect();
            v.sort_by_key(|c| std::cmp::Reverse(c.updated));
            v.into_iter().take(5).map(|c| (c.id.clone(), c.messages.iter().find(|m| m.role == "user").map(|m| m.text.lines().next().unwrap_or("").to_string()).unwrap_or_else(|| c.title.clone()), super::kb::day_label(c.updated, today()))).collect()
        };
        let (n_pages, n_recs) = super::kb::kb_counts(&self.store.data, &pid);
        let reads = format!("Reads your brief, roadmap, H0 tree and {n_recs} record{}", if n_recs == 1 { "" } else { "s" });
        let _ = n_pages;
        let busy_any = self.resolve_ui.chat_job.is_some();
        let busy_here = self.resolve_ui.chat_job.as_ref().map_or(false, |(c, _)| c == &cid);
        let state = match self.view_state {
            ViewState::Full if conv.messages.is_empty() && !busy_here => ViewState::Empty,
            s => s,
        };
        let input_id = egui::Id::new("team-chat-input");
        let mut pick: Option<&'static str> = None;
        let mut open_conv: Option<Id> = None;
        let (mut send, mut new_chat) = (false, false);
        let mut prefill: Option<String> = None;
        let mut decide: Option<(usize, bool)> = None;
        let mut open_settings = false;
        let mut copy_q: Option<String> = None;
        let screen_h = ui.available_height();
        egui::Frame::none().inner_margin(egui::Margin { left: space::WORK_SIDE - space::S8, right: space::WORK_SIDE - space::S8, top: space::S8, bottom: space::S24 }).show(ui, |ui| {
            capped(ui, space::CAP_TEAM, |ui| {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = space::S18;
                    // the roles and earlier chats
                    ui.allocate_ui_with_layout(egui::vec2(space::TEAM_LIST, 0.0), Layout::top_down(Align::Min), |ui| {
                        ui.set_width(space::TEAM_LIST);
                        ui.spacing_mut().item_spacing.y = space::S8;
                        ui.label(RichText::new(&name).font(fs(size::ROW_SMALL)).color(p.orange));
                        ui.label(RichText::new("AI Team").font(fserif(size::GREETING)).color(p.text));
                        ui.add_space(space::S4);
                        for r in TEAM.iter() {
                            let sub = if r.id == GENERAL { "Ask anything · same chat as Ask AI".to_string() } else { r.tag.replace(" & ", " and ") };
                            let glyph = if r.id == GENERAL { "✦" } else { r.avatar };
                            if role_card(ui, p, glyph, r.name, &sub, r.id == agent).clicked() {
                                pick = Some(r.id);
                            }
                        }
                        if !earlier.is_empty() {
                            ui.add_space(space::S10);
                            ui.label(RichText::new(format!("Earlier with the {}", sp.name)).font(fs(size::SMALL)).color(p.text));
                            for (id, t, day) in &earlier {
                                if ui.add(egui::Label::new(RichText::new(format!("{t} · {day}")).size(size::ROW_SMALL).color(p.text)).truncate().sense(Sense::click())).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                    open_conv = Some(id.clone());
                                }
                            }
                        }
                    });
                    // the chat
                    // the gap before the next item isn't taken off available_width yet
                    let w = ui.available_width() - ui.spacing().item_spacing.x - stroke::HAIR * 2.0;
                    let h = (screen_h - space::S8 - space::S24).max(space::S40 * 10.0);
                    egui::Frame::none().fill(p.card).stroke(Stroke::new(stroke::HAIR, p.line)).rounding(egui::Rounding::same(radius::DIALOG)).show(ui, |ui| {
                        ui.set_width(w);
                        ui.set_height(h);
                        ui.vertical(|ui| {
                            // head
                            egui::Frame::none().inner_margin(egui::Margin::symmetric(space::S18, space::S12)).show(ui, |ui| {
                                ui.set_width(w - space::S18 * 2.0);
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(sp.name).font(fs(size::HERO_BODY)).color(p.text));
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        new_chat = button(ui, p, ButtonKind::Secondary, "New chat", true, !conv.messages.is_empty()).on_hover_text("Start fresh. This chat is kept under Earlier").clicked();
                                        ui.label(RichText::new(&reads).size(size::SMALL).color(p.dim));
                                    });
                                });
                            });
                            let (hr, _) = ui.allocate_exact_size(egui::vec2(w, stroke::HAIR), Sense::hover());
                            ui.painter().rect_filled(hr, 0.0, p.divider);
                            // the messages, then the suggestions and the box at the foot
                            let foot = space::S40 * 2.5;
                            let body_h = (ui.available_height() - foot).max(space::S40 * 3.0);
                            ui.allocate_ui_with_layout(egui::vec2(w, body_h), Layout::top_down(Align::Min), |ui| {
                                ui.set_height(body_h);
                                egui::ScrollArea::vertical().id_salt(("team-chat", &cid)).stick_to_bottom(true).auto_shrink([false, false]).show(ui, |ui| {
                                    egui::Frame::none().inner_margin(egui::Margin::same(space::S18)).show(ui, |ui| {
                                        let cw = w - space::S18 * 2.0;
                                        ui.set_width(cw);
                                        ui.spacing_mut().item_spacing.y = space::S12;
                                        if state == ViewState::Loading && conv.messages.is_empty() {
                                            chat_bubble(ui, p, "Two clients said yes in principle. Is that enough to call H2 supported?", true, cw);
                                            thinking_line(ui, p, "Reading H2, E1 and your records…");
                                            return;
                                        }
                                        if state == ViewState::Empty {
                                            ui.add_space(body_h * 0.28);
                                            ui.vertical_centered(|ui| {
                                                ui.set_max_width(space::SEARCH_W - space::S40 * 3.0);
                                                ui.label(RichText::new(format!("Ask the {}", sp.name)).font(fserif(size::CARD_SERIF)).color(p.text));
                                                ui.add(egui::Label::new(RichText::new(format!("{}. It reads this project before it answers, and proposes changes for you to accept.", sp.tag.replace(" & ", " and "))).size(size::ROW_SMALL).color(p.dim)).wrap());
                                            });
                                            return;
                                        }
                                        for (k, m) in conv.messages.iter().enumerate() {
                                            let mine = m.role == "user";
                                            if m.error && crate::ai::is_limit(&m.text) {
                                                let q = conv.messages[..k].iter().rev().find(|x| x.role == "user").map(|x| x.text.clone()).unwrap_or_default();
                                                let (a, b) = budget_card(ui, p, &m.text, cw * 0.82);
                                                open_settings |= a;
                                                if b {
                                                    copy_q = Some(q);
                                                }
                                                continue;
                                            }
                                            let (text, props) = team::split_proposals(&m.text);
                                            if !text.trim().is_empty() {
                                                chat_bubble(ui, p, text.trim(), mine, cw);
                                            }
                                            for (j, pr) in props.iter().enumerate() {
                                                match pr_card(ui, p, pr, cw * 0.82) {
                                                    Some(true) => decide = Some((k * 100 + j, true)),
                                                    Some(false) => decide = Some((k * 100 + j, false)),
                                                    None => {}
                                                }
                                            }
                                        }
                                        if busy_here {
                                            let secs = (self.now - self.resolve_ui.chat_started).max(0.0) as u64;
                                            thinking_line(ui, p, &format!("Reading this project… {secs} s"));
                                        }
                                    });
                                });
                            });
                            egui::Frame::none().inner_margin(egui::Margin { left: space::S18, right: space::S18, top: 0.0, bottom: space::S14 }).show(ui, |ui| {
                                ui.set_width(w - space::S18 * 2.0);
                                ui.spacing_mut().item_spacing.y = space::S10;
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = space::S8;
                                    for s in team::suggestions(&agent) {
                                        if suggestion_chip(ui, p, s).clicked() {
                                            prefill = Some(s.to_string());
                                        }
                                    }
                                });
                                let focused = ui.memory(|m| m.has_focus(input_id));
                                if focused && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)) {
                                    send = true;
                                }
                                let out = chat_box(ui, p, input_id, &mut self.resolve_ui.chat_input, &format!("Ask the {}", sp.name), !busy_any);
                                send |= out;
                            });
                        });
                    });
                });
            });
        });
        if let Some(id) = pick {
            self.resolve_ui.agent = id.to_string();
        }
        if let Some(c) = open_conv {
            if let Some(x) = self.store.data.conversations.iter_mut().find(|x| x.id == c) {
                x.updated = now_ms();
            }
            self.touch();
        }
        if new_chat {
            let c = Conversation { id: new_id(), project_id: Some(pid.clone()), agent: agent.clone(), title: sp.name.to_string(), messages: vec![], updated: now_ms() + 1 };
            self.store.data.conversations.push(c);
            self.touch();
        }
        if let Some(s) = prefill {
            self.resolve_ui.chat_input = s;
            ui.memory_mut(|m| m.request_focus(input_id));
        }
        if send && !busy_any && !self.resolve_ui.chat_input.trim().is_empty() {
            let msg = std::mem::take(&mut self.resolve_ui.chat_input);
            self.team_send(&agent, &msg, "", "team");
        }
        if open_settings {
            self.go(View::Settings);
        }
        if let Some(q) = copy_q {
            ui.ctx().copy_text(q);
            self.toast("Copied your question");
        }
        if let Some((key, add)) = decide {
            self.decide_proposal(&cid, key / 100, key % 100, add);
        }
        true
    }

    /// Add it / Not now on a proposed change: the change goes through history (Add it), and the
    /// message remembers the choice either way.
    pub(super) fn decide_proposal(&mut self, cid: &str, msg: usize, j: usize, add: bool) {
        let Some(conv) = self.store.data.conversations.iter().find(|c| c.id == cid).cloned() else { return };
        let Some(text) = conv.messages.get(msg).map(|m| m.text.clone()) else { return };
        let (_, props) = team::split_proposals(&text);
        let Some(pr) = props.get(j).cloned() else { return };
        if add {
            let pid = conv.project_id.clone();
            let what = pr.clone();
            let parent = if pr.kind == "page" { None } else { pid.as_deref().and_then(|p| team::find_parent(&self.store.data, p, &pr.on)) };
            let label = if pr.kind == "page" { format!("Saved the page \"{}\"", pr.on) } else { format!("Added: {}", pr.text) };
            self.undoable(&label, "add", move |d| team::apply_proposal(d, pid.as_deref(), &what, parent.as_deref()));
        }
        if let Some(c) = self.store.data.conversations.iter_mut().find(|c| c.id == cid) {
            if let Some(m) = c.messages.get_mut(msg) {
                m.text = team::mark_proposal(&m.text, j, if add { "ADDED" } else { "SKIPPED" });
            }
            c.updated = now_ms();
        }
        self.touch();
    }

    pub(super) fn team_view(&mut self, ui: &mut egui::Ui, p: &Pal) {
        if self.resolve_ui.agent.is_empty() || !TEAM.iter().any(|r| r.id == self.resolve_ui.agent) {
            self.resolve_ui.agent = "product-director".into();
        }
        // one row of equal cards: same width and height; the selected one only gets a highlight
        // border (same size, same place, so nothing moves)
        let mut pick: Option<&'static str> = None;
        let current = self.resolve_ui.agent.clone();
        ui.columns(TEAM.len(), |cols| {
            for (i, sp) in TEAM.iter().enumerate() {
                let ui = &mut cols[i];
                let on = current == sp.id;
                let r = egui::Frame::none().fill(p.card).stroke(Stroke::new(if on { 2.0 } else { 1.0 }, if on { p.accent } else { p.line })).rounding(space::RADIUS_SMALL).inner_margin(egui::Margin::same(if on { 9.0 } else { 10.0 })).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.set_height(58.0);
                    ui.horizontal_top(|ui| {
                        avatar(ui, sp.avatar, if on { p.accent } else { p.purple }, 24.0);
                        ui.vertical(|ui| {
                            ui.add(egui::Label::new(RichText::new(sp.name).font(fs(13.0))).truncate());
                            ui.add(egui::Label::new(RichText::new(sp.tag).size(size::TINY).color(p.dim)).wrap());
                        });
                    });
                });
                if ui.interact(r.response.rect, ui.id().with(("sp", sp.id)), Sense::click()).on_hover_text(sp.name).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                    pick = Some(sp.id);
                }
            }
        });
        if let Some(id) = pick {
            self.resolve_ui.agent = id.to_string();
        }
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

    /// The AI's rewrite of a new handoff arrives: it replaces the draft (which stays if it fails).
    pub(super) fn poll_handoff(&mut self, ctx: &egui::Context) {
        let done = match &self.resolve_ui.handoff_job {
            Some((rid, rx)) => rx.try_recv().ok().map(|r| (rid.clone(), r)),
            None => None,
        };
        if let Some((rid, r)) = done {
            let via = self.provider_label();
            if let Some(rec) = self.store.data.record_mut(&rid) {
                match r {
                    Ok(brief) => {
                        rec.body = format!("{brief}\n\n---\nWritten by {via} from your request and the project's context.");
                        rec.fields.insert("writer".into(), via);
                    }
                    Err(e) => {
                        rec.fields.insert("writer".into(), format!("Resolve (the AI couldn't write it: {})", e.chars().take(120).collect::<String>()));
                    }
                }
                rec.updated = now_ms();
            }
            self.resolve_ui.handoff_job = None;
            self.touch();
        }
        if self.resolve_ui.handoff_job.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(300));
        }
    }

    /// Ask the AI to rewrite a handoff's brief (a new one, or Try again).
    fn handoff_rewrite(&mut self, pid: &str, rid: &str) -> bool {
        let Some(ctx) = self.ensure_context(pid) else { return false };
        let Some(r) = self.store.data.record(rid).cloned() else { return false };
        let cfg = self.ai_config();
        if cfg.provider == "offline" || self.resolve_ui.handoff_job.is_some() {
            return false;
        }
        if let Some(x) = self.store.data.record_mut(rid) {
            x.fields.insert("writer".into(), "writing with AI…".into());
        }
        let (d, to, focus) = (self.store.data.clone(), r.field("to").to_string(), r.field("focus").to_string());
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(roles::handoff_ai(&cfg, &d, &ctx, &to, &focus));
        });
        self.resolve_ui.handoff_job = Some((rid.to_string(), rx));
        true
    }

    /// The redesigned Handoffs for one project (C-E20, Pack C "Handoffs"). Returns false with no
    /// project picked.
    pub(super) fn handoffs_page(&mut self, ui: &mut egui::Ui, p: &Pal) -> bool {
        let Some(pid) = self.current_project.clone() else { return false };
        self.poll_handoff(ui.ctx());
        let name = self.store.data.project(&pid).map(|x| x.name.clone()).unwrap_or_default();
        let mut list: Vec<Record> = self.store.data.records.iter().filter(|r| r.kind == Kind::Handoff && r.project_id.as_deref() == Some(pid.as_str())).cloned().collect();
        list.sort_by_key(|r| std::cmp::Reverse(r.created));
        let done_tab = self.temp.get("ho_tab").map_or(false, |x| x == "done");
        let sel = self.resolve_ui.sel_record.clone().filter(|s| list.iter().any(|r| &r.id == s)).or_else(|| list.first().map(|r| r.id.clone()));
        let cur = sel.as_ref().and_then(|s| list.iter().find(|r| &r.id == s)).cloned();
        let state = match self.view_state {
            ViewState::Full if list.is_empty() => ViewState::Empty,
            s => s,
        };
        let for_kind = self.temp.get("ho_for").cloned().unwrap_or_else(|| "A person".into());
        let mut make = false;
        let mut pick: Option<Id> = None;
        let mut set_status: Option<(Id, &'static str)> = None;
        let mut copy: Option<String> = None;
        let mut copy_live = false;
        let mut retry = false;
        let mut plain = false;
        let mut tab: Option<bool> = None;
        let mut for_pick: Option<&'static str> = None;
        let screen_h = ui.available_height();
        egui::Frame::none().inner_margin(egui::Margin { left: space::WORK_SIDE - space::S8, right: space::WORK_SIDE - space::S8, top: space::S8, bottom: space::S24 }).show(ui, |ui| {
            capped(ui, space::CAP_DEFAULT, |ui| {
                ui.label(RichText::new(&name).font(fs(size::ROW_SMALL)).color(p.orange));
                ui.label(RichText::new("Handoffs").font(fserif(size::GREETING)).color(p.text));
                ui.add_space(space::S10);
                let h = (screen_h - space::S40 * 3.0).max(space::S40 * 10.0);
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = space::S16;
                    ui.allocate_ui_with_layout(egui::vec2(space::HO_LEFT, h), Layout::top_down(Align::Min), |ui| {
                        ui.set_width(space::HO_LEFT);
                        ui.spacing_mut().item_spacing.y = space::S12;
                        white_card(ui, p, false, |ui| {
                            ui.spacing_mut().item_spacing.y = space::S8;
                            ui.label(RichText::new("New handoff").font(fs(size::ROW + 0.5)).color(p.text));
                            field_label(ui, p, "For");
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = space::S6;
                                for k in ["Claude Code", "Codex", "A person"] {
                                    if tab_chip(ui, p, k, None, for_kind == k).clicked() {
                                        for_pick = Some(k);
                                    }
                                }
                            });
                            let w = ui.available_width();
                            if for_kind == "A person" {
                                field_label(ui, p, "Who");
                                text_field(ui, p, egui::Id::new("ho-who"), &mut self.resolve_ui.handoff_for, "Their name, and who they are to you", false, w);
                            }
                            field_label(ui, p, "What they should do");
                            let fid = egui::Id::new("ho-what");
                            ui.horizontal_top(|ui| {
                                text_field(ui, p, fid, &mut self.resolve_ui.handoff_focus, "The job, in a sentence or two", true, w - space::MIC - space::S8);
                                mic_for(ui, p, fid, false);
                            });
                            ui.add_space(space::S4);
                            make = wide_button(ui, p, ButtonKind::Primary, "Create brief", !self.resolve_ui.handoff_focus.trim().is_empty()).clicked();
                        });
                        let sent: Vec<&Record> = list.iter().filter(|r| r.status != "Done").collect();
                        let done: Vec<&Record> = list.iter().filter(|r| r.status == "Done").collect();
                        egui::Frame::none().fill(p.card).stroke(Stroke::new(stroke::HAIR, p.line)).rounding(egui::Rounding::same(radius::CARD)).inner_margin(egui::Margin::symmetric(space::S16, space::S12)).show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.set_min_height((ui.available_height() - space::S24).max(space::S40 * 3.0));
                            if let Some(t) = underline_tabs(ui, p, &[&format!("Sent · {}", sent.len()), &format!("Done · {}", done.len())], done_tab as usize) {
                                tab = Some(t == 1);
                            }
                            ui.add_space(space::S6);
                            let rows = if done_tab { &done } else { &sent };
                            if rows.is_empty() {
                                ui.add(egui::Label::new(RichText::new(if done_tab { "Nothing done yet. Briefs you mark Done move here." } else { "No saved handoffs yet. Briefs you mark Sent wait here until they're done." }).size(size::SMALL + 0.5).color(p.dim)).wrap());
                            }
                            for (i, r) in rows.iter().enumerate() {
                                if i > 0 {
                                    let (hr, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), stroke::HAIR), Sense::hover());
                                    ui.painter().rect_filled(hr, 0.0, p.divider);
                                }
                                let who = if r.field("to").is_empty() { "Someone".to_string() } else { r.field("to").to_string() };
                                let when = super::kb::day_label(r.updated, today());
                                let meta = match r.status.as_str() {
                                    "Done" => format!("{who} · done {}", when.to_lowercase()),
                                    "Sent" => format!("{who} · sent {when}"),
                                    _ => format!("{who} · draft"),
                                };
                                let (action, to) = match r.status.as_str() {
                                    "Done" => ("Reopen", "Sent"),
                                    "Sent" => ("Mark done", "Done"),
                                    _ => ("Mark sent", "Sent"),
                                };
                                ui.horizontal(|ui| {
                                    let tw = ui.available_width() - space::S40 * 2.5;
                                    let r2 = ui.allocate_ui_with_layout(egui::vec2(tw, 0.0), Layout::top_down(Align::Min), |ui| {
                                        ui.set_width(tw);
                                        ui.spacing_mut().item_spacing.y = space::S4 / 2.0;
                                        ui.add(egui::Label::new(RichText::new(&r.title).font(fs(size::ROW_SMALL)).color(if sel.as_deref() == Some(r.id.as_str()) { p.accent_text } else { p.text })).truncate());
                                        ui.label(RichText::new(meta).size(size::SMALL).color(p.dim));
                                    });
                                    if ui.interact(r2.response.rect, egui::Id::new(("ho-row", &r.id)), Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                        pick = Some(r.id.clone());
                                    }
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        if button(ui, p, ButtonKind::Secondary, action, true, true).clicked() {
                                            set_status = Some((r.id.clone(), to));
                                        }
                                    });
                                });
                            }
                        });
                    });
                    // the brief
                    let w = ui.available_width() - ui.spacing().item_spacing.x - stroke::HAIR * 2.0;
                    egui::Frame::none().fill(p.card).stroke(Stroke::new(stroke::HAIR, p.line)).rounding(egui::Rounding::same(radius::CARD + 1.0)).show(ui, |ui| {
                        ui.set_width(w);
                        ui.set_height(h);
                        ui.vertical(|ui| {
                            egui::Frame::none().inner_margin(egui::Margin::symmetric(space::S18, space::S12)).show(ui, |ui| {
                                ui.set_width(w - space::S18 * 2.0);
                                ui.horizontal(|ui| {
                                    let who = cur.as_ref().map(|r| r.field("to").to_string()).filter(|x| !x.is_empty());
                                    ui.label(RichText::new(match &who { Some(x) => format!("Brief for {}", x.split(',').next().unwrap_or(x).trim()), None => "Brief".into() }).font(fs(size::HERO_BODY)).color(p.text));
                                    if let Some(r) = cur.as_ref().filter(|_| state == ViewState::Full) {
                                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                            ui.spacing_mut().item_spacing.x = space::S8;
                                            let (action, to) = match r.status.as_str() {
                                                "Done" => ("Reopen", "Sent"),
                                                "Sent" => ("Mark done", "Done"),
                                                _ => ("Mark sent", "Sent"),
                                            };
                                            if button(ui, p, ButtonKind::Dark, action, true, true).clicked() {
                                                set_status = Some((r.id.clone(), to));
                                            }
                                            if button(ui, p, ButtonKind::Secondary, "Copy", true, true).clicked() {
                                                copy = Some(r.body.clone());
                                            }
                                        });
                                    }
                                });
                            });
                            let (hr, _) = ui.allocate_exact_size(egui::vec2(w, stroke::HAIR), Sense::hover());
                            ui.painter().rect_filled(hr, 0.0, p.divider);
                            let foot = space::S40;
                            let body_h = (ui.available_height() - foot).max(space::S40 * 3.0);
                            ui.allocate_ui_with_layout(egui::vec2(w, body_h), Layout::top_down(Align::Min), |ui| {
                                ui.set_height(body_h);
                                egui::ScrollArea::vertical().id_salt(("ho-brief", sel.clone().unwrap_or_default())).auto_shrink([false, false]).show(ui, |ui| {
                                    egui::Frame::none().inner_margin(egui::Margin::same(space::S18)).show(ui, |ui| {
                                        ui.set_width(w - space::S18 * 2.0);
                                        ui.spacing_mut().item_spacing.y = space::S6;
                                        match (state, &cur) {
                                            (ViewState::Empty, _) | (_, None) => {
                                                ui.add_space(body_h * 0.28);
                                                ui.vertical_centered(|ui| {
                                                    ui.set_max_width(space::SEARCH_W - space::S40 * 4.0);
                                                    ui.label(RichText::new("Write a brief for someone else").font(fserif(size::CARD_SERIF)).color(p.text));
                                                    ui.add(egui::Label::new(RichText::new("Say who it's for and what they should do. Resolve adds what exists, why it matters, what's off limits and when it's done.").size(size::ROW_SMALL).color(p.dim)).wrap());
                                                });
                                            }
                                            (_, Some(r)) => {
                                                let writer = r.field("writer").to_string();
                                                if state == ViewState::Loading || writer.starts_with("writing") {
                                                    ui.label(RichText::new("Writing the brief from the project…").size(size::ROW_SMALL).color(p.dim));
                                                    let ww = ui.available_width();
                                                    for k in 0..3 {
                                                        skeleton(ui, p, ww * (0.95 - k as f32 * 0.15), space::S8);
                                                    }
                                                    return;
                                                }
                                                if state == ViewState::Error || writer.starts_with("Resolve (the AI couldn't") {
                                                    let (a, b) = handoff_fallback_card(ui, p);
                                                    retry |= a;
                                                    plain |= b;
                                                    if state == ViewState::Error {
                                                        return;
                                                    }
                                                    ui.add_space(space::S8);
                                                }
                                                for (head, body) in roles::brief_sections(&r.body) {
                                                    if !head.is_empty() {
                                                        ui.add_space(space::S6);
                                                        ui.label(RichText::new(head.to_uppercase()).font(fs(size::SMALL)).color(p.orange));
                                                    }
                                                    ui.add(egui::Label::new(RichText::new(body).size(size::ROW + 0.5).color(p.text)).wrap().selectable(true));
                                                }
                                            }
                                        }
                                    });
                                });
                            });
                            let (fr, _) = ui.allocate_exact_size(egui::vec2(w, foot), Sense::hover());
                            ui.painter().rect_filled(fr, egui::Rounding { nw: 0.0, ne: 0.0, sw: radius::CARD, se: radius::CARD }, p.surface_raised);
                            let mut fu = ui.new_child(egui::UiBuilder::new().max_rect(fr.shrink2(egui::vec2(space::S18, 0.0))).layout(Layout::left_to_right(Align::Center)));
                            fu.label(RichText::new("Current project brief (always up to date) ·").size(size::ROW_SMALL).color(p.dim));
                            copy_live = link(&mut fu, p, "Copy it").clicked();
                        });
                    });
                });
            });
        });
        if let Some(k) = for_pick {
            self.temp.insert("ho_for".into(), k.into());
            if k != "A person" {
                self.resolve_ui.handoff_for.clear();
            }
        }
        if let Some(t) = tab {
            self.temp.insert("ho_tab".into(), if t { "done".into() } else { "sent".into() });
        }
        if let Some(id) = pick {
            self.resolve_ui.sel_record = Some(id);
        }
        if let Some(b) = copy {
            ui.ctx().copy_text(b);
            self.toast("Brief copied");
        }
        if copy_live {
            if let Some(ctx) = self.ensure_context(&pid) {
                ui.ctx().copy_text(roles::handoff_brief(&self.store.data, &ctx));
                self.toast("Current project brief copied");
            }
        }
        if let Some((id, to)) = set_status {
            let title = self.store.data.record(&id).map(|r| r.title.clone()).unwrap_or_default();
            let label = match to {
                "Done" => format!("Marked \"{title}\" done"),
                "Sent" => format!("Marked \"{title}\" sent"),
                _ => format!("Changed \"{title}\""),
            };
            self.undoable(&label, "edit", move |d| {
                if let Some(r) = d.record_mut(&id) {
                    let _ = r.set_status(to);
                }
            });
        }
        if retry {
            if let Some(r) = &cur {
                if !self.handoff_rewrite(&pid, &r.id) {
                    self.toast("No AI is connected, so the plain brief stays.");
                }
            }
            if self.view_state == ViewState::Error {
                self.view_state = ViewState::Full;
            }
        }
        if plain {
            if let Some(r) = &cur {
                if let Some(x) = self.store.data.record_mut(&r.id) {
                    x.fields.insert("writer".into(), "Resolve (plain brief)".into());
                }
                self.touch();
            }
            if self.view_state == ViewState::Error {
                self.view_state = ViewState::Full;
            }
        }
        if make {
            let to = if for_kind == "A person" { std::mem::take(&mut self.resolve_ui.handoff_for).trim().to_string() } else { for_kind.clone() };
            let focus = std::mem::take(&mut self.resolve_ui.handoff_focus).trim().to_string();
            if let Some(ctx) = self.ensure_context(&pid) {
                let body = roles::handoff_for(&self.store.data, &ctx, &to, &focus);
                let mut r = Record::new(Kind::Handoff, Some(pid.clone()), focus.chars().take(60).collect::<String>());
                r.body = body;
                r.fields.insert("to".into(), to);
                r.fields.insert("focus".into(), focus);
                r.fields.insert("writer".into(), "Resolve (no AI connected)".into());
                let id = r.id.clone();
                let label = format!("Wrote a handoff: {}", r.title);
                self.undoable(&label, "add", move |d| d.records.push(r));
                self.resolve_ui.sel_record = Some(id.clone());
                self.temp.insert("ho_tab".into(), "sent".into());
                self.handoff_rewrite(&pid, &id);
            }
        }
        true
    }

    pub(super) fn handoffs_view(&mut self, ui: &mut egui::Ui, p: &Pal) {
        self.poll_handoff(ui.ctx());
        let scope = self.current_project.clone();
        let mut make = false;
        let mut list: Vec<Record> = self.store.data.records.iter().filter(|r| r.kind == Kind::Handoff && scope.as_ref().map_or(true, |s| r.project_id.as_ref() == Some(s))).cloned().collect();
        list.sort_by_key(|r| std::cmp::Reverse(r.created));
        card(ui, p, |ui| {
            ui.label(RichText::new("New handoff").font(fs(size::SECTION)));
            if scope.is_some() {
                wrap_label(ui, RichText::new("Written from the project's real state: outcome, decisions, open questions, the cut list and the next tasks. Claude and Codex can also read and update this workspace over MCP (Settings → Advanced).").size(size::TINY).color(p.faint));
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
                    let writer = r.field("writer");
                    if !writer.is_empty() {
                        if writer.starts_with("writing") {
                            ui.spinner();
                        }
                        ui.label(RichText::new(format!("· {writer}")).size(size::TINY).color(p.faint));
                    }
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
                // written for this request: what you typed + the parts of the project that bear on it
                let body = roles::handoff_for(&self.store.data, &ctx, &to, &focus);
                let title = if focus.is_empty() { format!("Handoff{for_whom}") } else { format!("{}{for_whom}", focus.chars().take(60).collect::<String>()) };
                let mut r = Record::new(Kind::Handoff, Some(pid.clone()), title);
                r.body = body;
                r.fields.insert("to".into(), to.clone());
                r.fields.insert("focus".into(), focus.clone());
                // with an AI connected, it rewrites the brief from the request and the full context
                let cfg = self.ai_config();
                if cfg.provider != "offline" && self.resolve_ui.handoff_job.is_none() {
                    r.fields.insert("writer".into(), "writing with AI…".into());
                    let (d, c, rid) = (self.store.data.clone(), ctx.clone(), r.id.clone());
                    let (tx, rx) = std::sync::mpsc::channel();
                    std::thread::spawn(move || {
                        let _ = tx.send(roles::handoff_ai(&cfg, &d, &c, &to, &focus));
                    });
                    self.resolve_ui.handoff_job = Some((rid, rx));
                } else {
                    r.fields.insert("writer".into(), "Resolve (no AI connected)".into());
                }
                self.resolve_ui.sel_record = Some(r.id.clone());
                self.store.data.records.push(r);
                self.touch();
            }
        }
    }
}
