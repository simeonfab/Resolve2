//! The Resolve project cockpit: the ResolveOS Runtime Surface on top of the existing project.
//! Overview (next move, what matters now, context) · Validation (Hypothesis Zero tree) ·
//! Records · Team (roles) · Handoff.

use super::resolve::{amber, JobOut};
use super::nav::Section;
use super::theme::*;
use super::widgets::*;
use super::*;
use crate::resolve::roles;
use crate::resolve::{assess, Kind, ProjectContext, Record};

const INK: Color32 = Color32::from_rgb(26, 26, 24);
const ON_INK: Color32 = Color32::from_rgb(251, 248, 242);
const ON_INK_SOFT: Color32 = Color32::from_rgb(201, 196, 183);
const EYEBROW: Color32 = Color32::from_rgb(217, 169, 124);

fn status_color(r: &Record, p: &Pal) -> Color32 {
    match r.status.as_str() {
        "Supported" | "Decided" | "Done" | "Closed" | "Answered" | "Resolved" | "Reached" | "Met" | "Holding" => p.green,
        "Refuted" | "Broken" | "Blocking" | "Occurred" | "Missed" | "At risk" => p.orange,
        "Running" | "Weakened" | "Mitigating" => amber(),
        _ => p.dim,
    }
}

impl App {
    pub(super) fn cockpit_overview(&mut self, ui: &mut egui::Ui, p: &Pal, pid: &str, ctx: &ProjectContext) {
        let a = assess::assess(&self.store.data, ctx);
        // keep the pinned win stable (ResolvePM rule) — re-pick only when it's done
        if a.win != ctx.win_task_id {
            if let Some(c) = self.store.data.context_for_mut(pid) {
                c.win_task_id = a.win.clone();
                c.updated = now_ms();
            }
            self.touch();
        }
        let win = a.win.as_ref().and_then(|w| self.store.data.task(w)).cloned();
        egui::Frame::none().fill(INK).rounding(12.0).inner_margin(18.0).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new("Do this next").size(12.5).color(EYEBROW));
            match &win {
                Some(t) => {
                    ui.horizontal(|ui| {
                        let (r, resp) = ui.allocate_exact_size(egui::vec2(26.0, 26.0), Sense::click());
                        ui.painter().circle_stroke(r.center(), 10.0, Stroke::new(2.0, amber()));
                        if resp.on_hover_text("Mark done").clicked() {
                            self.toggle_task(&t.id);
                        }
                        let l = ui.add(egui::Label::new(RichText::new(&t.title).font(fserif(22.0)).color(ON_INK)).sense(Sense::click()));
                        if l.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                            self.open_task = Some(t.id.clone());
                        }
                    });
                    if let Some(d) = t.due {
                        ui.label(RichText::new(format!("Due {}", fmt_date_short(d))).size(12.0).color(ON_INK_SOFT));
                    }
                }
                None => {
                    ui.label(RichText::new("Nothing open — plan the next piece of work.").font(fserif(20.0)).color(ON_INK));
                }
            }
        });
        ui.add_space(10.0);
        let frac = if a.total == 0 { 0.0 } else { a.done as f32 / a.total as f32 };
        ui.horizontal(|ui| {
            let (r, _) = ui.allocate_exact_size(egui::vec2(260.0, 6.0), Sense::hover());
            ui.painter().rect_filled(r, 3.0, p.line);
            ui.painter().rect_filled(egui::Rect::from_min_size(r.min, egui::vec2(r.width() * frac, 6.0)), 3.0, amber());
            ui.label(RichText::new(format!("{} of {} tasks done", a.done, a.total)).size(12.5).color(p.dim));
        });
        ui.add_space(10.0);
        ui.label(RichText::new("ResolveOS").font(fs(14.0)));
        let mut clicked: Option<&'static str> = None;
        ui.horizontal_wrapped(|ui| {
            for act in roles::ACTIONS.iter() {
                let busy = self.resolve_ui.rx.is_some();
                if ui.add_enabled(!busy || act.deterministic, egui::Button::new(RichText::new(act.label).size(13.0))).on_hover_text(act.hint).clicked() {
                    clicked = Some(act.id);
                }
            }
        });
        match clicked {
            Some("what-matters") => {
                let text = assess::summary_text(&self.store.data, ctx);
                let mut g = Record::new(Kind::Guidance, Some(pid.to_string()), "What matters now");
                g.body = text;
                g.fields.insert("action".into(), "what-matters".into());
                g.fields.insert("provider".into(), "deterministic".into());
                self.store.data.records.push(g);
                self.touch();
                self.toast("Saved to the project's history (Knowledge → Evidence & Records)");
            }
            Some("handoff") => self.go(View::Sec(Section::Handoffs)),
            Some(id) => {
                let label = roles::ACTIONS.iter().find(|a| a.id == id).map(|a| a.label).unwrap_or("Working");
                let cfg = crate::ai::current_config();
                let (c, d, pid2, action) = (ctx.clone(), self.store.data.clone(), pid.to_string(), id.to_string());
                self.spawn_job(&format!("{label}…"), move || {
                    let r = roles::propose(&cfg, &action, &c, &d);
                    JobOut::Proposals(pid2, action, r)
                });
            }
            None => {}
        }
        self.proposals_panel(ui, p, pid);
        ui.add_space(10.0);
        let mut goto: Option<Id> = None;
        card(ui, p, |ui| {
            ui.label(RichText::new("What matters now").font(fs(15.0)));
            if a.next.is_empty() {
                ui.label(RichText::new("Nothing pressing.").color(p.dim));
            }
            for (i, (reason, target)) in a.next.iter().take(10).enumerate() {
                let r = ui.add(egui::Label::new(RichText::new(format!("{}. {reason}", i + 1)).size(13.5)).sense(Sense::click()));
                if r.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                    goto = target.clone();
                }
            }
            ui.add_space(4.0);
            ui.label(RichText::new(format!("Decided {} · open decisions {} · uncertain {} · blocked {} · open risks {}", a.decided.len(), a.open_decisions.len(), a.uncertain.len(), a.blocked.len(), a.risks.len())).size(12.0).color(p.faint));
        });
        if let Some(id) = goto {
            if self.store.data.task(&id).is_some() {
                self.open_task = Some(id);
            } else {
                let h0 = self.store.data.record(&id).map_or(false, |r| matches!(r.kind, Kind::Hypothesis | Kind::Experiment));
                self.resolve_ui.knowledge_tab = 3;
                self.go(View::Sec(if h0 { Section::Frameworks } else { Section::Knowledge }));
                self.resolve_ui.sel_record = Some(id);
            }
        }
        ui.add_space(10.0);
        let mut c = ctx.clone();
        card(ui, p, |ui| {
            ui.label(RichText::new("Project context").font(fs(15.0)));
            ui.label(RichText::new("Carried over from intake. Edit it here; every role, assessment and handoff reads from it.").size(12.0).color(p.faint));
            let fields: [(&str, &mut String); 6] = [
                ("Project", &mut c.understanding.project),
                ("What you want", &mut c.understanding.goal),
                ("Where it is", &mut c.understanding.state),
                ("Biggest uncertainty", &mut c.understanding.uncertainty),
                ("Outcome — who is better off, and how would you know?", &mut c.outcome),
                ("Notes", &mut c.notes),
            ];
            for (label, v) in fields {
                ui.label(RichText::new(label).size(12.0).color(p.dim));
                ui.add(egui::TextEdit::multiline(v).desired_rows(1).desired_width(f32::INFINITY));
            }
            ui.checkbox(&mut c.outcome_confirmed, "The outcome is confirmed (names who is better off and an observable change)");
        });
        if serde_json::to_string(&c).ok() != serde_json::to_string(ctx).ok() {
            c.updated = now_ms();
            if let Some(x) = self.store.data.contexts.iter_mut().find(|x| x.id == c.id) {
                *x = c;
            }
            self.touch();
        }
        ui.add_space(10.0);
        ui.columns(2, |cols| {
            card(&mut cols[0], p, |ui| {
                ui.label(RichText::new("Top priorities").font(fs(14.0)));
                for x in ctx.recommendation.top_priorities.iter().filter(|x| !x.trim().is_empty()) {
                    ui.label(format!("• {x}"));
                }
                ui.add_space(6.0);
                ui.label(RichText::new("Not yet").font(fs(14.0)));
                for n in ctx.recommendation.do_not_yet.iter().filter(|n| !n.item.trim().is_empty()) {
                    ui.label(RichText::new(format!("• {} — {}", n.item, n.reason)).size(13.0).color(p.dim));
                }
            });
            card(&mut cols[1], p, |ui| {
                ui.label(RichText::new("Readiness (from intake)").font(fs(14.0)));
                match &ctx.plan {
                    Some(pl) => {
                        for r in &pl.readiness {
                            ui.horizontal_wrapped(|ui| {
                                let col = match r.status.as_str() {
                                    "Ready" => p.green,
                                    "Partially ready" => amber(),
                                    _ => p.orange,
                                };
                                chip(ui, &r.status, col);
                                ui.label(RichText::new(&r.area).font(fs(13.0)));
                            });
                            ui.label(RichText::new(&r.note).size(12.0).color(p.dim));
                        }
                    }
                    None => {
                        ui.label(RichText::new("No readiness assessment yet (the plan step wasn't run).").size(12.5).color(p.faint));
                    }
                }
            });
        });
    }

    fn proposals_panel(&mut self, ui: &mut egui::Ui, p: &Pal, pid: &str) {
        let Some((ppid, action, summary, items, mut accept)) = self.resolve_ui.proposals.clone() else { return };
        if ppid != pid {
            return;
        }
        ui.add_space(8.0);
        let mut done = false;
        let mut apply = false;
        egui::Frame::none().fill(p.callout).rounding(10.0).inner_margin(14.0).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(roles::ACTIONS.iter().find(|a| a.id == action).map(|a| a.label).unwrap_or("Proposals")).font(fs(15.0)));
            ui.label(RichText::new(&summary).size(13.0));
            ui.label(RichText::new("Proposed by AI — nothing is added until you confirm.").size(11.5).color(p.faint));
            for (i, it) in items.iter().enumerate() {
                let mut depth = 0;
                let mut cur = it.parent;
                while let Some(x) = cur {
                    depth += 1;
                    cur = items.get(x).and_then(|y| y.parent);
                    if depth > 6 {
                        break;
                    }
                }
                ui.horizontal(|ui| {
                    ui.add_space(depth as f32 * 18.0);
                    ui.checkbox(&mut accept[i], "");
                    chip(ui, &it.kind, amber());
                    ui.label(RichText::new(&it.title).size(13.5));
                });
                if !it.body.is_empty() {
                    ui.horizontal(|ui| {
                        ui.add_space(depth as f32 * 18.0 + 30.0);
                        ui.label(RichText::new(&it.body).size(12.0).color(p.dim));
                    });
                }
            }
            ui.horizontal(|ui| {
                if primary_button(ui, p, "Add selected").clicked() {
                    apply = true;
                }
                if ui.button("Discard").clicked() {
                    done = true;
                }
            });
        });
        if apply {
            let ids = roles::apply_proposals(&mut self.store.data, pid, &items, &accept, None);
            let n = ids.iter().filter(|x| x.is_some()).count();
            self.touch();
            self.toast(format!("Added {n} item(s) to the project"));
            done = true;
        }
        if done {
            self.resolve_ui.proposals = None;
        } else if let Some(x) = self.resolve_ui.proposals.as_mut() {
            x.4 = accept;
        }
    }

    pub(super) fn cockpit_validation(&mut self, ui: &mut egui::Ui, p: &Pal, pid: &str) {
        ui.label(RichText::new("Hypothesis Zero").font(fs(16.0)));
        ui.label(RichText::new("Any hypothesis can have sub-hypotheses and experiments, in any order. State the negation, keep contradicting evidence, and fix an experiment's thresholds before running it.").size(12.5).color(p.dim));
        let mut add = false;
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.resolve_ui.new_title).hint_text("A claim you want to test…").desired_width(420.0));
            add = ui.add_enabled(!self.resolve_ui.new_title.trim().is_empty(), egui::Button::new("+ Root hypothesis")).clicked();
        });
        if add {
            let r = Record::new(Kind::Hypothesis, Some(pid.to_string()), self.resolve_ui.new_title.trim());
            self.resolve_ui.sel_record = Some(r.id.clone());
            self.store.data.records.push(r);
            self.resolve_ui.new_title.clear();
            self.touch();
        }
        ui.add_space(8.0);
        let tree = assess::hypothesis_tree(&self.store.data, pid);
        let rows: Vec<(usize, Record)> = tree.iter().filter_map(|(d, id)| self.store.data.record(id).cloned().map(|r| (*d, r))).collect();
        let sel = self.resolve_ui.sel_record.clone();
        let mut pick: Option<Id> = None;
        ui.columns(2, |cols| {
            if rows.is_empty() {
                cols[0].label(RichText::new("No hypotheses yet. Add one above, or use “Create validation plan” on the Overview.").color(p.faint));
            }
            for (depth, r) in &rows {
                let is_sel = sel.as_deref() == Some(r.id.as_str());
                cols[0].horizontal(|ui| {
                    ui.add_space(*depth as f32 * 18.0);
                    let col = match r.kind {
                        Kind::Hypothesis => amber(),
                        Kind::Experiment => p.blue,
                        _ => p.green,
                    };
                    chip(ui, r.kind.icon(), col);
                    let lab = ui.add(egui::Label::new(RichText::new(&r.title).size(13.5).color(if is_sel { amber() } else { p.text })).sense(Sense::click()).wrap());
                    if lab.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                        pick = Some(r.id.clone());
                    }
                    ui.label(RichText::new(&r.status).size(11.5).color(status_color(r, p)));
                });
            }
            if let Some(id) = sel.clone() {
                if self.store.data.record(&id).is_some() {
                    self.record_editor(&mut cols[1], p, pid, &id);
                }
            }
        });
        if pick.is_some() {
            self.resolve_ui.sel_record = pick;
        }
    }

    pub(super) fn cockpit_records(&mut self, ui: &mut egui::Ui, p: &Pal, pid: &str) {
        ui.horizontal_wrapped(|ui| {
            if pill(ui, p, "All", self.resolve_ui.kind_filter.is_none()).clicked() {
                self.resolve_ui.kind_filter = None;
            }
            for k in Kind::ALL {
                let n = self.store.data.records.iter().filter(|r| r.kind == k && r.project_id.as_deref() == Some(pid)).count();
                if n > 0 && pill(ui, p, &format!("{} {n}", k.plural()), self.resolve_ui.kind_filter == Some(k)).clicked() {
                    self.resolve_ui.kind_filter = Some(k);
                }
            }
        });
        let mut add = false;
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("new-kind").selected_text(self.resolve_ui.new_kind.label()).show_ui(ui, |ui| {
                for k in Kind::ALL {
                    if !matches!(k, Kind::Guidance | Kind::Handoff) {
                        ui.selectable_value(&mut self.resolve_ui.new_kind, k, k.label());
                    }
                }
            });
            let r = ui.add(egui::TextEdit::singleline(&mut self.resolve_ui.new_title).hint_text("Title…").desired_width(380.0));
            let enter = r.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
            add = ui.button("Add").clicked() || enter;
        });
        if add && !self.resolve_ui.new_title.trim().is_empty() {
            let rec = Record::new(self.resolve_ui.new_kind, Some(pid.to_string()), self.resolve_ui.new_title.trim());
            self.resolve_ui.sel_record = Some(rec.id.clone());
            self.store.data.records.push(rec);
            self.resolve_ui.new_title.clear();
            self.touch();
        }
        ui.add_space(6.0);
        let mut list: Vec<Record> = self.store.data.records.iter().filter(|r| r.project_id.as_deref() == Some(pid) && r.kind != Kind::Handoff).filter(|r| self.resolve_ui.kind_filter.map_or(true, |k| r.kind == k)).cloned().collect();
        list.sort_by_key(|r| (!r.is_open(), r.kind, std::cmp::Reverse(r.updated)));
        let sel = self.resolve_ui.sel_record.clone();
        let mut pick: Option<Id> = None;
        ui.columns(2, |cols| {
            if list.is_empty() {
                cols[0].label(RichText::new("No records yet.").color(p.faint));
            }
            for r in &list {
                let is_sel = sel.as_deref() == Some(r.id.as_str());
                cols[0].horizontal(|ui| {
                    chip(ui, r.kind.label(), p.dim);
                    let lab = ui.add(egui::Label::new(RichText::new(&r.title).size(13.5).color(if is_sel { amber() } else { p.text })).sense(Sense::click()).wrap());
                    if lab.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                        pick = Some(r.id.clone());
                    }
                    ui.label(RichText::new(&r.status).size(11.5).color(status_color(r, p)));
                });
            }
            if let Some(id) = sel.clone() {
                if self.store.data.record(&id).map_or(false, |r| r.project_id.as_deref() == Some(pid)) {
                    self.record_editor(&mut cols[1], p, pid, &id);
                }
            }
        });
        if pick.is_some() {
            self.resolve_ui.sel_record = pick;
        }
    }

    /// Edit any record using its kind's field schema.
    pub(super) fn record_editor(&mut self, ui: &mut egui::Ui, p: &Pal, pid: &str, id: &str) {
        let Some(orig) = self.store.data.record(id).cloned() else { return };
        let mut r = orig.clone();
        let mut delete = false;
        let mut add_child: Option<Kind> = None;
        let mut make_task = false;
        let mut open: Option<Id> = None;
        let mut msg = String::new();
        let names: Vec<(Id, String)> = r
            .links
            .iter()
            .filter_map(|l| {
                let d = &self.store.data;
                d.task(l).map(|t| format!("☐ {}", t.title)).or_else(|| d.page(l).map(|pg| format!("📄 {}", pg.title))).or_else(|| d.record(l).map(|x| format!("{} {}", x.kind.icon(), x.title))).map(|n| (l.clone(), n))
            })
            .collect();
        card(ui, p, |ui| {
            ui.horizontal(|ui| {
                chip(ui, r.kind.label(), amber());
                egui::ComboBox::from_id_salt(("status", id)).selected_text(r.status.clone()).show_ui(ui, |ui| {
                    for s in r.kind.statuses() {
                        if ui.selectable_label(r.status == *s, *s).clicked() {
                            if let Err(e) = r.set_status(s) {
                                msg = e;
                            }
                        }
                    }
                });
            });
            ui.add(egui::TextEdit::multiline(&mut r.title).font(fs(15.0)).desired_rows(1).desired_width(f32::INFINITY));
            ui.add(egui::TextEdit::multiline(&mut r.body).hint_text("Notes").desired_rows(2).desired_width(f32::INFINITY));
            for f in r.kind.fields() {
                let locked = orig.is_locked(f.key);
                ui.label(RichText::new(format!("{}{}", if locked { "🔒 " } else { "" }, f.label)).size(12.0).color(p.dim));
                let mut v = r.field(f.key).to_string();
                let resp = if f.multiline {
                    ui.add_enabled(!locked, egui::TextEdit::multiline(&mut v).desired_rows(1).desired_width(f32::INFINITY))
                } else {
                    ui.add_enabled(!locked, egui::TextEdit::singleline(&mut v).desired_width(f32::INFINITY))
                };
                if resp.changed() {
                    if let Err(e) = r.set_field(f.key, &v) {
                        msg = e;
                    }
                }
            }
            if r.kind == Kind::Experiment && orig.status == "Planned" {
                ui.label(RichText::new("Prediction and thresholds lock when you set it Running.").size(11.5).color(p.faint));
            }
            if !names.is_empty() {
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new("Linked:").size(12.0).color(p.dim));
                    for (l, n) in &names {
                        if chip(ui, n, p.blue).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                            open = Some(l.clone());
                        }
                    }
                });
            }
            ui.horizontal_wrapped(|ui| {
                if r.kind == Kind::Hypothesis {
                    if ui.button("+ Sub-hypothesis").clicked() {
                        add_child = Some(Kind::Hypothesis);
                    }
                    if ui.button("+ Experiment").clicked() {
                        add_child = Some(Kind::Experiment);
                    }
                }
                if matches!(r.kind, Kind::Hypothesis | Kind::Experiment) && ui.button("+ Evidence").clicked() {
                    add_child = Some(Kind::Evidence);
                }
                if ui.button("Make a task").on_hover_text("Create a linked task in this project").clicked() {
                    make_task = true;
                }
                if ui.button(RichText::new("Delete").color(p.orange)).clicked() {
                    delete = true;
                }
            });
            if !msg.is_empty() {
                ui.label(RichText::new(&msg).size(12.0).color(p.orange));
            }
        });
        if let Some(l) = open {
            if self.store.data.task(&l).is_some() {
                self.open_task = Some(l);
            } else if self.store.data.page(&l).is_some() {
                self.go(View::Page(l));
            } else {
                self.resolve_ui.sel_record = Some(l);
            }
        }
        if delete {
            { let rid = id.to_string(); self.undoable(&format!("Deleted {} \"{}\"", r.kind.label().to_lowercase(), r.title), "delete", |d| d.delete_record(&rid)); }
            self.resolve_ui.sel_record = None;
            self.touch();
            return;
        }
        if serde_json::to_string(&r).ok() != serde_json::to_string(&orig).ok() {
            r.updated = now_ms();
            if let Some(x) = self.store.data.record_mut(id) {
                *x = r.clone();
            }
            self.touch();
        }
        if let Some(k) = add_child {
            let title = match k {
                Kind::Hypothesis => "New sub-hypothesis",
                Kind::Experiment => "New experiment",
                _ => "New evidence",
            };
            let mut c = Record::new(k, Some(pid.to_string()), title);
            c.parent_id = Some(id.to_string());
            self.resolve_ui.sel_record = Some(c.id.clone());
            self.store.data.records.push(c);
            self.touch();
        }
        if make_task {
            let mut t = Task::new(match r.kind {
                Kind::Experiment => format!("Run: {}", r.title),
                Kind::Decision => format!("Decide: {}", r.title),
                Kind::Blocker => format!("Unblock: {}", r.title),
                _ => r.title.clone(),
            });
            t.project_id = Some(pid.to_string());
            t.section = "Now".into();
            t.notes = format!("From {} “{}”", r.kind.label().to_lowercase(), r.title);
            let tid = t.id.clone();
            self.store.data.tasks.push(t);
            if let Some(x) = self.store.data.record_mut(id) {
                x.links.push(tid);
                x.updated = now_ms();
            }
            self.touch();
            self.toast("Task created and linked");
        }
    }

    pub(super) fn cockpit_handoff(&mut self, ui: &mut egui::Ui, p: &Pal, pid: &str, ctx: &ProjectContext) {
        let brief = roles::handoff_brief(&self.store.data, ctx);
        ui.label(RichText::new("Hand off to an execution tool").font(fs(16.0)));
        ui.label(RichText::new("A self-contained brief built from the project's real state — paste it into Codex, Claude Code, Cursor or ChatGPT. They can also read and update this workspace directly over MCP (Settings → AI & MCP).").size(12.5).color(p.dim));
        let (mut copy, mut save) = (false, false);
        ui.horizontal(|ui| {
            copy = primary_button(ui, p, "Copy brief").clicked();
            save = ui.button("Save as a page").clicked();
        });
        if copy {
            ui.ctx().copy_text(brief.clone());
            self.toast("Handoff brief copied");
        }
        if save {
            let (title, blocks) = markdown_to_blocks(&brief);
            let mut pg = Page::new(title.unwrap_or_else(|| "Handoff".into()));
            pg.icon = "📦".into();
            pg.blocks = blocks;
            pg.parent = ctx.page_id.clone().filter(|x| self.store.data.page(x).is_some());
            let id = pg.id.clone();
            self.store.data.pages.push(pg);
            if let Some(c) = self.store.data.context_for_mut(pid) {
                c.updated = now_ms();
            }
            self.touch();
            self.go(View::Page(id));
        }
        ui.add_space(6.0);
        egui::Frame::none().fill(p.code).rounding(8.0).inner_margin(12.0).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(brief).monospace().size(12.5));
        });
    }
}
