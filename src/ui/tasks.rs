use super::theme::*;
use super::widgets::*;
use super::*;
use chrono::Datelike;
use eframe::egui::{self, Color32, RichText, Sense, Stroke};

enum TAct {
    Toggle(Id),
    Open(Id),
    Move(Id, String),
}

#[derive(Clone, Default)]
struct AddDefaults {
    project: Option<Id>,
    section: String,
    due: Option<NaiveDate>,
}

impl App {
    fn render_tasks(&self, ui: &mut egui::Ui, p: &Pal, ids: &[Id], show_project: bool, nested: bool, acts: &mut Vec<TAct>) {
        let d = &self.store.data;
        for id in ids {
            let Some(t) = d.task(id) else { continue };
            let r = task_row(ui, p, d, t, show_project, 0.0);
            if r.toggled {
                acts.push(TAct::Toggle(t.id.clone()));
            }
            if r.opened {
                acts.push(TAct::Open(t.id.clone()));
            }
            if nested {
                self.render_subtasks(ui, p, &t.id, 1, acts);
            }
        }
    }

    fn render_subtasks(&self, ui: &mut egui::Ui, p: &Pal, parent: &str, depth: usize, acts: &mut Vec<TAct>) {
        if depth > 4 {
            return;
        }
        let d = &self.store.data;
        for s in d.subtasks(parent) {
            if s.completed && !self.tasks_ui.show_completed {
                continue;
            }
            let r = task_row(ui, p, d, s, false, 28.0 * depth as f32);
            if r.toggled {
                acts.push(TAct::Toggle(s.id.clone()));
            }
            if r.opened {
                acts.push(TAct::Open(s.id.clone()));
            }
            self.render_subtasks(ui, p, &s.id, depth + 1, acts);
        }
    }

    fn add_inline(&mut self, ui: &mut egui::Ui, p: &Pal, key: &str, def: AddDefaults) {
        if self.tasks_ui.adding.as_deref() == Some(key) {
            let mut submit = false;
            let mut cancel = false;
            egui::Frame::none().stroke(Stroke::new(1.0, p.line)).rounding(8.0).inner_margin(egui::Margin::same(10.0)).show(ui, |ui| {
                ui.set_width(ui.available_width());
                let r = ui.add(egui::TextEdit::singleline(&mut self.tasks_ui.add_text).hint_text("Task name — try: Pay rent every month 9am p1 @bills").frame(false).font(fs(14.5)).desired_width(f32::INFINITY));
                if !r.has_focus() && self.temp.remove("focus_add").is_some() {
                    r.request_focus();
                }
                if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    submit = true;
                }
                if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    cancel = true;
                }
                let c = crate::nlp::parse(&self.tasks_ui.add_text, today());
                ui.horizontal_wrapped(|ui| {
                    if c.due.is_none() {
                        if let Some(dd) = def.due {
                            chip(ui, &format!("📅 {}", fmt_date_short(dd)), p.green);
                        }
                    }
                    for s in c.chips() {
                        chip(ui, &s, p.blue);
                    }
                    if !def.section.is_empty() {
                        chip(ui, &format!("§ {}", def.section), p.dim);
                    }
                });
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("# {}", self.store.data.project_name(&def.project))).size(12.0).color(p.dim));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if primary_button(ui, p, "Add task").clicked() {
                            submit = true;
                        }
                        if ui.button("Cancel").clicked() {
                            cancel = true;
                        }
                    });
                });
            });
            if submit {
                let text = self.tasks_ui.add_text.clone();
                if self.add_task_text(&text, def.project.clone(), &def.section, def.due, None, None).is_some() {
                    self.tasks_ui.add_text.clear();
                }
                self.temp.insert("focus_add".into(), "1".into());
            }
            if cancel {
                self.tasks_ui.adding = None;
            }
        } else if add_row_button(ui, p, "Add task").clicked() {
            self.tasks_ui.adding = Some(key.to_string());
            self.tasks_ui.add_text.clear();
            self.temp.insert("focus_add".into(), "1".into());
        }
    }

    fn sort_ids(&self, mut v: Vec<&Task>) -> Vec<Id> {
        v.sort_by_key(|t| (t.completed, t.order));
        v.into_iter().map(|t| t.id.clone()).collect()
    }

    pub fn tasks_view(&mut self, ui: &mut egui::Ui, p: &Pal) {
        let view = self.view.clone();
        let board_key = match &view {
            View::Project(id) => id.clone(),
            View::Inbox => "inbox".into(),
            _ => String::new(),
        };
        let is_board = !board_key.is_empty() && self.tasks_ui.board.contains(&board_key);
        self.top_bar(ui, p, |app, ui| {
            ui.menu_button(RichText::new("☰ Display").size(13.0), |ui| {
                ui.checkbox(&mut app.tasks_ui.show_completed, "Show completed tasks");
                if !board_key.is_empty() {
                    ui.separator();
                    ui.label(RichText::new("Layout").size(12.0).color(p.dim));
                    let mut b = app.tasks_ui.board.contains(&board_key);
                    ui.horizontal(|ui| {
                        if ui.selectable_label(!b, "☰ List").clicked() {
                            b = false;
                        }
                        if ui.selectable_label(b, "▥ Board").clicked() {
                            b = true;
                        }
                    });
                    if b {
                        app.tasks_ui.board.insert(board_key.clone());
                    } else {
                        app.tasks_ui.board.remove(&board_key);
                    }
                }
            });
            if let View::Project(pid) = &view {
                let pid = pid.clone();
                let has_ctx = app.store.data.context_for(&pid).is_some();
                if ui.add(egui::Button::new(RichText::new(if has_ctx { "✦ Resolve" } else { "✦ Open in Resolve" }).color(p.on_accent)).fill(p.accent)).on_hover_text("Guidance, decisions, hypotheses and next move for this project").clicked() {
                    app.resolve_ui.tab = 0;
                    app.go(View::Resolve(pid.clone()));
                }
                let doc = app.store.data.project(&pid).and_then(|x| x.page_id.clone()).filter(|x| app.store.data.page(x).is_some());
                if ui.button(if doc.is_some() { "📄 Project doc" } else { "📄 Create project doc" }).clicked() {
                    match doc {
                        Some(pg) => app.go(View::Page(pg)),
                        None => {
                            let name = app.store.data.project(&pid).map(|x| x.name.clone()).unwrap_or_default();
                            let mut pg = Page::new(format!("{name} — Project brief"));
                            pg.icon = "📁".into();
                            pg.blocks = vec![Block::new(BlockKind::H2, "Goal"), Block::new(BlockKind::Paragraph, ""), Block::new(BlockKind::H2, "Notes"), Block::new(BlockKind::Paragraph, "")];
                            let pgid = pg.id.clone();
                            app.store.data.pages.push(pg);
                            if let Some(pr) = app.store.data.projects.iter_mut().find(|x| x.id == pid) {
                                pr.page_id = Some(pgid.clone());
                                pr.updated = now_ms();
                            }
                            app.touch();
                            app.go(View::Page(pgid));
                        }
                    }
                }
                ui.menu_button(RichText::new("⋯").size(16.0), |ui| {
                    if ui.button("✏ Rename").clicked() {
                        app.tasks_ui.rename_project = true;
                        ui.close_menu();
                    }
                    ui.label(RichText::new("Colour").size(12.0).color(p.dim));
                    ui.horizontal(|ui| {
                        for c in [[219u8, 76, 63], [255, 153, 51], [246, 191, 38], [41, 171, 135], [20, 170, 245], [63, 81, 181], [175, 56, 235], [128, 128, 128]] {
                            let (r, resp) = ui.allocate_exact_size(egui::vec2(18.0, 18.0), Sense::click());
                            ui.painter().circle_filled(r.center(), 8.0, rgb(c));
                            if resp.clicked() {
                                if let Some(pr) = app.store.data.projects.iter_mut().find(|x| x.id == pid) {
                                    pr.color = c;
                                    pr.updated = now_ms();
                                }
                                app.touch();
                            }
                        }
                    });
                    ui.separator();
                    if ui.button("🗄 Archive project").clicked() {
                        if let Some(pr) = app.store.data.projects.iter_mut().find(|x| x.id == pid) {
                            pr.archived = true;
                            pr.updated = now_ms();
                        }
                        app.touch();
                        app.go(View::Inbox);
                        ui.close_menu();
                    }
                    if ui.button(RichText::new("🗑 Delete project and its tasks").color(p.accent)).clicked() {
                        { let name = app.store.data.project(&pid).map(|p| p.name.clone()).unwrap_or_default(); let pp = pid.clone(); app.undoable(&format!("Deleted project \"{name}\""), "delete", |d| d.delete_project(&pp)); }
                        app.touch();
                        app.view = View::Inbox;
                        ui.close_menu();
                    }
                });
            }
        });
        let mut acts: Vec<TAct> = vec![];
        let t0 = today();
        let show_done = self.tasks_ui.show_completed;
        let max_w = if is_board { 4000.0 } else { 820.0 };
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            centered(ui, max_w, |ui| match view.clone() {
                View::Inbox => {
                    let n = self.store.data.tasks.iter().filter(|t| !t.completed && t.project_id.is_none() && t.parent_id.is_none()).count();
                    view_title(ui, p, "Inbox", Some(&format!("{n} tasks")));
                    if is_board {
                        self.board(ui, p, None, &mut acts);
                    } else {
                        let ids = self.sort_ids(self.store.data.tasks.iter().filter(|t| t.project_id.is_none() && t.parent_id.is_none() && (show_done || !t.completed)).collect());
                        self.render_tasks(ui, p, &ids, false, true, &mut acts);
                        self.add_inline(ui, p, "inbox", AddDefaults::default());
                    }
                }
                View::Today => {
                    let d = &self.store.data;
                    let n = d.tasks.iter().filter(|t| !t.completed && t.due.map_or(false, |x| x <= t0)).count();
                    view_title(ui, p, "Today", Some(&format!("{n} tasks")));
                    let mut overdue: Vec<&Task> = d.tasks.iter().filter(|t| t.is_overdue() && t.due.map_or(false, |x| x < t0)).collect();
                    overdue.sort_by_key(|t| (t.due, t.priority));
                    if !overdue.is_empty() {
                        let mut resched = false;
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Overdue").font(fb(14.0)));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if ghost_button(ui, RichText::new("Reschedule").color(p.accent).size(13.0)).clicked() {
                                    resched = true;
                                }
                            });
                        });
                        hline(ui, p);
                        let ids: Vec<Id> = overdue.iter().map(|t| t.id.clone()).collect();
                        self.render_tasks(ui, p, &ids, true, false, &mut acts);
                        ui.add_space(14.0);
                        if resched {
                            for t in self.store.data.tasks.iter_mut().filter(|x| x.is_overdue() && x.due.map_or(false, |dd| dd < t0)) {
                                t.due = Some(t0);
                                t.updated = now_ms();
                            }
                            self.touch();
                        }
                    }
                    ui.label(RichText::new(format!("{} · Today · {}", t0.format("%-d %b"), t0.format("%A"))).font(fb(14.0)));
                    hline(ui, p);
                    let d = &self.store.data;
                    let mut today_tasks: Vec<&Task> = d.tasks.iter().filter(|t| t.due == Some(t0) && (show_done || !t.completed)).collect();
                    today_tasks.sort_by_key(|t| (t.completed, t.due_time.is_none(), t.due_time, t.priority));
                    let ids: Vec<Id> = today_tasks.iter().map(|t| t.id.clone()).collect();
                    self.render_tasks(ui, p, &ids, true, false, &mut acts);
                    self.add_inline(ui, p, "today", AddDefaults { due: Some(t0), ..Default::default() });
                }
                View::Upcoming => {
                    view_title(ui, p, "Upcoming", Some(&t0.format("%B %Y").to_string()));
                    // week strip
                    ui.horizontal(|ui| {
                        for i in 0..7 {
                            let day = t0 + chrono::Duration::days(i);
                            let n = self.store.data.tasks.iter().filter(|t| !t.completed && t.due == Some(day)).count();
                            let txt = format!("{}\n{}{}", day.format("%a"), day.day(), if n > 0 { format!(" · {n}") } else { String::new() });
                            ui.add(egui::Button::new(RichText::new(txt).size(12.5).color(if i == 0 { p.accent } else { p.text })).min_size(egui::vec2(88.0, 40.0)).fill(if i == 0 { p.accent_soft } else { Color32::TRANSPARENT }));
                        }
                    });
                    ui.add_space(10.0);
                    let d = &self.store.data;
                    let overdue: Vec<Id> = d.tasks.iter().filter(|t| t.is_overdue() && t.due.map_or(false, |x| x < t0)).map(|t| t.id.clone()).collect();
                    if !overdue.is_empty() {
                        ui.label(RichText::new("Overdue").font(fb(14.0)).color(p.accent));
                        hline(ui, p);
                        self.render_tasks(ui, p, &overdue, true, false, &mut acts);
                        ui.add_space(12.0);
                    }
                    for i in 0..21 {
                        let day = t0 + chrono::Duration::days(i);
                        let label = match i {
                            0 => format!("{} · Today · {}", day.format("%-d %b"), day.format("%A")),
                            1 => format!("{} · Tomorrow · {}", day.format("%-d %b"), day.format("%A")),
                            _ => format!("{} · {}", day.format("%-d %b"), day.format("%A")),
                        };
                        ui.label(RichText::new(label).font(fb(14.0)));
                        hline(ui, p);
                        let d = &self.store.data;
                        let mut v: Vec<&Task> = d.tasks.iter().filter(|t| t.due == Some(day) && (show_done || !t.completed)).collect();
                        v.sort_by_key(|t| (t.completed, t.due_time.is_none(), t.due_time, t.priority));
                        let ids: Vec<Id> = v.iter().map(|t| t.id.clone()).collect();
                        self.render_tasks(ui, p, &ids, true, false, &mut acts);
                        // events of the day for context
                        let evs: Vec<(NaiveDateTime, String, Color32)> = d.events.iter().filter(|e| d.calendar(&e.calendar_id).map_or(true, |c| c.visible)).filter_map(|e| e.occurrence_on(day).map(|s| (s, e.title.clone(), d.calendar(&e.calendar_id).map(|c| rgb(c.color)).unwrap_or(p.blue)))).collect();
                        if !evs.is_empty() {
                            ui.horizontal_wrapped(|ui| {
                                for (s, t, c) in evs.iter().take(6) {
                                    chip(ui, &format!("{} {t}", fmt_time(s.time())), *c);
                                }
                            });
                        }
                        self.add_inline(ui, p, &format!("up-{day}"), AddDefaults { due: Some(day), ..Default::default() });
                        ui.add_space(10.0);
                    }
                }
                View::Filters => {
                    view_title(ui, p, "Filters & Labels", None);
                    let mut labels: Vec<String> = vec![];
                    for t in &self.store.data.tasks {
                        for l in &t.labels {
                            if !labels.contains(l) {
                                labels.push(l.clone());
                            }
                        }
                    }
                    labels.sort();
                    let filters = [("f:p1", "⚑ Priority 1"), ("f:p2", "⚑ Priority 2"), ("f:overdue", "⏰ Overdue"), ("f:nodate", "∅ No date"), ("f:recurring", "↻ Recurring"), ("f:linked", "📄 Linked to a page"), ("f:done", "✔ Completed")];
                    ui.label(RichText::new("Filters").font(fb(15.0)));
                    ui.horizontal_wrapped(|ui| {
                        for (k, l) in filters {
                            let sel = self.tasks_ui.label.as_deref() == Some(k);
                            if pill(ui, p, l, sel).clicked() {
                                self.tasks_ui.label = Some(k.to_string());
                            }
                        }
                    });
                    ui.add_space(8.0);
                    ui.label(RichText::new("Labels").font(fb(15.0)));
                    ui.horizontal_wrapped(|ui| {
                        if labels.is_empty() {
                            ui.label(RichText::new("Add labels to tasks with @name, e.g. \"Call bank @phone\".").color(p.dim));
                        }
                        for l in &labels {
                            let k = format!("l:{l}");
                            let sel = self.tasks_ui.label.as_deref() == Some(k.as_str());
                            if pill(ui, p, &format!("@{l}"), sel).clicked() {
                                self.tasks_ui.label = Some(k);
                            }
                        }
                    });
                    ui.add_space(12.0);
                    let key = self.tasks_ui.label.clone().unwrap_or_else(|| "f:p1".into());
                    let d = &self.store.data;
                    let mut v: Vec<&Task> = d
                        .tasks
                        .iter()
                        .filter(|t| match key.as_str() {
                            "f:p1" => !t.completed && t.priority == 1,
                            "f:p2" => !t.completed && t.priority == 2,
                            "f:overdue" => t.is_overdue(),
                            "f:nodate" => !t.completed && t.due.is_none() && t.parent_id.is_none(),
                            "f:recurring" => !t.completed && t.recurrence.is_some(),
                            "f:linked" => !t.completed && t.page_id.is_some(),
                            "f:done" => t.completed,
                            k => k.strip_prefix("l:").map_or(false, |l| !t.completed && t.labels.iter().any(|x| x == l)),
                        })
                        .collect();
                    if key == "f:done" {
                        v.sort_by_key(|t| -t.completed_at.unwrap_or(0));
                    } else {
                        v.sort_by_key(|t| (t.due.unwrap_or(NaiveDate::MAX), t.priority));
                    }
                    let ids: Vec<Id> = v.iter().map(|t| t.id.clone()).collect();
                    if ids.is_empty() {
                        ui.label(RichText::new("No tasks match").color(p.dim));
                    }
                    self.render_tasks(ui, p, &ids, true, false, &mut acts);
                }
                View::Project(pid) => {
                    let Some(pr) = self.store.data.project(&pid).cloned() else { return };
                    ui.add_space(18.0);
                    if self.tasks_ui.rename_project {
                        let mut name = pr.name.clone();
                        let r = ui.add(egui::TextEdit::singleline(&mut name).font(fb(26.0)).desired_width(f32::INFINITY));
                        if r.changed() {
                            if let Some(x) = self.store.data.projects.iter_mut().find(|x| x.id == pid) {
                                x.name = name;
                                x.updated = now_ms();
                            }
                            self.touch();
                        }
                        if !r.has_focus() && self.temp.remove("proj_focus").is_none() {
                            r.request_focus();
                            self.temp.insert("proj_focus".into(), "1".into());
                        }
                        if r.lost_focus() {
                            self.tasks_ui.rename_project = false;
                            self.temp.remove("proj_focus");
                        }
                    } else {
                        let resp = ui.add(egui::Label::new(RichText::new(&pr.name).font(fb(26.0))).sense(Sense::click())).on_hover_text("Click to rename");
                        if resp.clicked() {
                            self.tasks_ui.rename_project = true;
                        }
                    }
                    let open = self.store.data.tasks.iter().filter(|t| t.project_id.as_deref() == Some(&pid) && !t.completed).count();
                    ui.label(RichText::new(format!("{open} open tasks")).size(13.0).color(p.dim));
                    ui.add_space(10.0);
                    if is_board {
                        self.board(ui, p, Some(pid.clone()), &mut acts);
                        return;
                    }
                    let d = &self.store.data;
                    let ids = self.sort_ids(d.tasks.iter().filter(|t| t.project_id.as_deref() == Some(&pid) && t.parent_id.is_none() && t.section.is_empty() && (show_done || !t.completed)).collect());
                    self.render_tasks(ui, p, &ids, false, true, &mut acts);
                    self.add_inline(ui, p, &format!("p-{pid}-"), AddDefaults { project: Some(pid.clone()), ..Default::default() });
                    for (si, sec) in pr.sections.iter().enumerate() {
                        ui.add_space(16.0);
                        let mut delete = false;
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(sec).font(fb(15.0)));
                            let n = self.store.data.tasks.iter().filter(|t| t.project_id.as_deref() == Some(&pid) && &t.section == sec && !t.completed && t.parent_id.is_none()).count();
                            ui.label(RichText::new(n.to_string()).size(12.0).color(p.dim));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.menu_button(RichText::new("⋯").color(p.dim), |ui| {
                                    if ui.button("🗑 Delete section (tasks move up)").clicked() {
                                        delete = true;
                                        ui.close_menu();
                                    }
                                });
                            });
                        });
                        hline(ui, p);
                        if delete {
                            let sname = sec.clone();
                            for t in self.store.data.tasks.iter_mut().filter(|t| t.project_id.as_deref() == Some(&pid) && t.section == sname) {
                                t.section.clear();
                                t.updated = now_ms();
                            }
                            if let Some(x) = self.store.data.projects.iter_mut().find(|x| x.id == pid) {
                                x.sections.remove(si);
                                x.updated = now_ms();
                            }
                            self.touch();
                            return;
                        }
                        let d = &self.store.data;
                        let ids = self.sort_ids(d.tasks.iter().filter(|t| t.project_id.as_deref() == Some(&pid) && t.parent_id.is_none() && &t.section == sec && (show_done || !t.completed)).collect());
                        self.render_tasks(ui, p, &ids, false, true, &mut acts);
                        self.add_inline(ui, p, &format!("p-{pid}-{sec}"), AddDefaults { project: Some(pid.clone()), section: sec.clone(), ..Default::default() });
                    }
                    ui.add_space(16.0);
                    self.add_section_ui(ui, p, &pid);
                }
                _ => {}
            });
            ui.add_space(40.0);
        });
        for a in acts {
            match a {
                TAct::Toggle(id) => self.toggle_task(&id),
                TAct::Open(id) => self.open_task = Some(id),
                TAct::Move(id, sec) => {
                    if let Some(t) = self.store.data.task_mut(&id) {
                        t.section = sec;
                        t.updated = now_ms();
                        t.order = now_ms();
                    }
                    let subs: Vec<Id> = self.store.data.subtasks(&id).iter().map(|s| s.id.clone()).collect();
                    let sec = self.store.data.task(&id).map(|t| t.section.clone()).unwrap_or_default();
                    for s in subs {
                        if let Some(t) = self.store.data.task_mut(&s) {
                            t.section = sec.clone();
                            t.updated = now_ms();
                        }
                    }
                    self.touch();
                }
            }
        }
    }

    fn add_section_ui(&mut self, ui: &mut egui::Ui, p: &Pal, pid: &str) {
        if let Some(mut name) = self.tasks_ui.new_section.take() {
            let mut keep = true;
            ui.horizontal(|ui| {
                let r = ui.add(egui::TextEdit::singleline(&mut name).hint_text("Section name").desired_width(260.0));
                r.request_focus();
                let enter = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                if (primary_button(ui, p, "Add section").clicked() || enter) && !name.trim().is_empty() {
                    if let Some(x) = self.store.data.projects.iter_mut().find(|x| x.id == pid) {
                        x.sections.push(name.trim().to_string());
                        x.updated = now_ms();
                    }
                    self.touch();
                    keep = false;
                }
                if ui.button("Cancel").clicked() {
                    keep = false;
                }
            });
            if keep {
                self.tasks_ui.new_section = Some(name);
            }
        } else {
            let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 28.0), Sense::click());
            let col = if resp.hovered() { p.accent } else { p.faint };
            ui.painter().line_segment([rect.left_center(), rect.center() - egui::vec2(60.0, 0.0)], Stroke::new(1.0, col));
            ui.painter().line_segment([rect.center() + egui::vec2(60.0, 0.0), rect.right_center()], Stroke::new(1.0, col));
            ui.painter().text(rect.center(), Align2::CENTER_CENTER, "Add section", fs(13.0), col);
            if resp.clicked() {
                self.tasks_ui.new_section = Some(String::new());
            }
        }
    }

    fn board(&mut self, ui: &mut egui::Ui, p: &Pal, pid: Option<Id>, acts: &mut Vec<TAct>) {
        let sections: Vec<String> = pid.as_ref().and_then(|x| self.store.data.project(x)).map(|x| x.sections.clone()).unwrap_or_default();
        let mut cols = vec![String::new()];
        cols.extend(sections);
        let show_done = self.tasks_ui.show_completed;
        let mut add_to: Option<String> = None;
        egui::ScrollArea::horizontal().id_salt("board").auto_shrink([false, true]).show(ui, |ui| {
            ui.horizontal_top(|ui| {
                for sec in &cols {
                    let frame = egui::Frame::none().fill(if p.dark { Color32::from_rgb(36, 36, 36) } else { Color32::from_rgb(248, 247, 245) }).rounding(10.0).inner_margin(egui::Margin::same(10.0));
                    let (_, dropped) = ui.dnd_drop_zone::<Id, ()>(frame, |ui| {
                        ui.set_width(270.0);
                        ui.set_min_height(300.0);
                        let d = &self.store.data;
                        let tasks: Vec<&Task> = {
                            let mut v: Vec<&Task> = d.tasks.iter().filter(|t| t.project_id == pid && t.parent_id.is_none() && &t.section == sec && (show_done || !t.completed)).collect();
                            v.sort_by_key(|t| (t.completed, t.order));
                            v
                        };
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(if sec.is_empty() { "(No section)" } else { sec }).font(fb(14.0)));
                            ui.label(RichText::new(tasks.len().to_string()).size(12.0).color(p.dim));
                        });
                        ui.add_space(4.0);
                        for t in tasks {
                            let id = egui::Id::new(("card", &t.id));
                            ui.dnd_drag_source(id, t.id.clone(), |ui| {
                                egui::Frame::none().fill(p.card).stroke(Stroke::new(1.0, p.line)).rounding(8.0).inner_margin(egui::Margin::same(10.0)).show(ui, |ui| {
                                    ui.set_width(248.0);
                                    ui.horizontal_top(|ui| {
                                        let (r, resp) = ui.allocate_exact_size(egui::vec2(20.0, 20.0), Sense::click());
                                        paint_circle(ui, r.center(), p, t.priority, t.completed, resp.hovered());
                                        if resp.clicked() {
                                            acts.push(TAct::Toggle(t.id.clone()));
                                        }
                                        ui.vertical(|ui| {
                                            if ui.add(egui::Label::new(RichText::new(&t.title).size(14.0)).wrap().sense(Sense::click())).clicked() {
                                                acts.push(TAct::Open(t.id.clone()));
                                            }
                                            if let Some((s, c)) = due_style(t, p) {
                                                ui.label(RichText::new(format!("🗓 {s}{}", if t.recurrence.is_some() { " ↻" } else { "" })).size(12.0).color(c));
                                            }
                                            let subs = d.subtasks(&t.id);
                                            if !subs.is_empty() {
                                                ui.label(RichText::new(format!("⑂ {}/{}", subs.iter().filter(|s| s.completed).count(), subs.len())).size(12.0).color(p.dim));
                                            }
                                        });
                                    });
                                });
                            });
                            ui.add_space(6.0);
                        }
                        if add_row_button(ui, p, "Add task").clicked() {
                            add_to = Some(sec.clone());
                        }
                    });
                    if let Some(tid) = dropped {
                        acts.push(TAct::Move((*tid).clone(), sec.clone()));
                    }
                    ui.add_space(8.0);
                }
                if let Some(pid) = &pid {
                    ui.vertical(|ui| {
                        ui.set_width(220.0);
                        self.add_section_ui(ui, p, pid);
                    });
                }
            });
        });
        if let Some(sec) = add_to {
            self.quick = Some(QuickAdd { text: String::new(), kind: QuickKind::Task, project: pid.clone(), section: sec, due: None, parent: None, page: None, focus: true });
        }
    }

    pub fn task_panel(&mut self, ctx: &egui::Context, p: &Pal, tid: &str) {
        let Some(mut t) = self.store.data.task(tid).cloned() else { return };
        let before = serde_json::to_string(&t).unwrap_or_default();
        let mut close = false;
        let mut delete = false;
        let mut toggle = false;
        let mut open_page: Option<Id> = None;
        let mut new_doc = false;
        let mut open_sub: Option<Id> = None;
        let mut toggle_sub: Option<Id> = None;
        let mut add_sub: Option<String> = None;
        egui::SidePanel::right("task-panel").default_width(400.0).width_range(340.0..=560.0).frame(egui::Frame::none().fill(p.card).inner_margin(egui::Margin::same(16.0)).stroke(Stroke::new(1.0, p.line))).show(ctx, |ui| {
            let d = &self.store.data;
            ui.horizontal(|ui| {
                let crumb = if t.section.is_empty() { format!("# {}", d.project_name(&t.project_id)) } else { format!("# {} / {}", d.project_name(&t.project_id), t.section) };
                ui.label(RichText::new(crumb).size(12.5).color(p.dim));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ghost_button(ui, RichText::new("✕").size(15.0)).on_hover_text("Close (Esc)").clicked() {
                        close = true;
                    }
                    if ghost_button(ui, RichText::new("🗑").size(14.0).color(p.dim)).on_hover_text("Delete task").clicked() {
                        delete = true;
                    }
                });
            });
            if let Some(par) = t.parent_id.as_deref().and_then(|x| d.task(x)) {
                if ghost_button(ui, RichText::new(format!("↖ Subtask of: {}", par.title)).size(12.0).color(p.blue)).clicked() {
                    open_sub = Some(par.id.clone());
                }
            }
            ui.add_space(6.0);
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    let (r, resp) = ui.allocate_exact_size(egui::vec2(24.0, 26.0), Sense::click());
                    paint_circle(ui, r.center(), p, t.priority, t.completed, resp.hovered());
                    if resp.clicked() {
                        toggle = true;
                    }
                    ui.add(egui::TextEdit::multiline(&mut t.title).font(fs(18.0)).frame(false).desired_rows(1).desired_width(f32::INFINITY));
                });
                ui.horizontal_top(|ui| {
                    ui.add_space(30.0);
                    ui.add(egui::TextEdit::multiline(&mut t.notes).hint_text("☰ Description").frame(false).desired_rows(2).desired_width(f32::INFINITY));
                });
                ui.add_space(8.0);
                // subtasks
                let subs = d.subtasks(&t.id);
                ui.label(RichText::new(format!("Sub-tasks {}/{}", subs.iter().filter(|s| s.completed).count(), subs.len())).font(fs(13.0)).color(p.dim));
                for s in subs {
                    ui.horizontal(|ui| {
                        let (r, resp) = ui.allocate_exact_size(egui::vec2(20.0, 20.0), Sense::click());
                        paint_circle(ui, r.center(), p, s.priority, s.completed, resp.hovered());
                        if resp.clicked() {
                            toggle_sub = Some(s.id.clone());
                        }
                        let mut rt = RichText::new(&s.title).size(13.5);
                        if s.completed {
                            rt = rt.strikethrough().color(p.dim);
                        }
                        if ui.add(egui::Label::new(rt).sense(Sense::click())).clicked() {
                            open_sub = Some(s.id.clone());
                        }
                    });
                }
                ui.horizontal(|ui| {
                    ui.label(RichText::new("+").color(p.accent));
                    let r = ui.add(egui::TextEdit::singleline(&mut self.tasks_ui.sub_text).hint_text("Add sub-task (Enter)").frame(false).desired_width(f32::INFINITY));
                    if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) && !self.tasks_ui.sub_text.trim().is_empty() {
                        add_sub = Some(std::mem::take(&mut self.tasks_ui.sub_text));
                        r.request_focus();
                    }
                });
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(6.0);
                egui::Grid::new("task-props").num_columns(2).spacing([12.0, 10.0]).show(ui, |ui| {
                    ui.label(RichText::new("Project").color(p.dim));
                    egui::ComboBox::from_id_salt("tp-project").selected_text(format!("# {}", d.project_name(&t.project_id))).width(200.0).show_ui(ui, |ui| {
                        ui.selectable_value(&mut t.project_id, None, "📥 Inbox");
                        for pr in &d.projects {
                            ui.selectable_value(&mut t.project_id, Some(pr.id.clone()), format!("# {}", pr.name));
                        }
                    });
                    ui.end_row();
                    let sections: Vec<String> = t.project_id.as_deref().and_then(|x| d.project(x)).map(|x| x.sections.clone()).unwrap_or_default();
                    if !sections.is_empty() {
                        ui.label(RichText::new("Section").color(p.dim));
                        egui::ComboBox::from_id_salt("tp-section").selected_text(if t.section.is_empty() { "(none)".to_string() } else { t.section.clone() }).width(200.0).show_ui(ui, |ui| {
                            ui.selectable_value(&mut t.section, String::new(), "(none)");
                            for s in sections {
                                ui.selectable_value(&mut t.section, s.clone(), s);
                            }
                        });
                        ui.end_row();
                    }
                    ui.label(RichText::new("Date").color(p.dim));
                    ui.horizontal(|ui| match t.due {
                        Some(mut date) => {
                            if ui.add(egui_extras::DatePickerButton::new(&mut date).id_salt("tp-date").calendar_week(false)).changed() {
                                t.due = Some(date);
                            }
                            if ghost_button(ui, RichText::new("✕").color(p.dim)).on_hover_text("Remove date").clicked() {
                                t.due = None;
                                t.due_time = None;
                            }
                        }
                        None => {
                            for (l, n) in [("Today", 0), ("Tomorrow", 1), ("Next week", 7)] {
                                if ui.small_button(l).clicked() {
                                    t.due = Some(today() + chrono::Duration::days(n));
                                }
                            }
                        }
                    });
                    ui.end_row();
                    if t.due.is_some() {
                        ui.label(RichText::new("Time").color(p.dim));
                        ui.horizontal(|ui| {
                            time_field(ui, egui::Id::new(("tp-time", &t.id)), &mut t.due_time, 60.0);
                            ui.label(RichText::new("for").color(p.dim));
                            let mut dur = t.duration_min.unwrap_or(0) as i32;
                            if ui.add(egui::DragValue::new(&mut dur).range(0..=720).speed(5).suffix(" min")).changed() {
                                t.duration_min = if dur > 0 { Some(dur as u32) } else { None };
                            }
                        });
                        ui.end_row();
                        ui.label(RichText::new("Repeat").color(p.dim));
                        egui::ComboBox::from_id_salt("tp-rec").selected_text(t.recurrence.label()).width(200.0).show_ui(ui, |ui| {
                            for r in Recurrence::CHOICES {
                                ui.selectable_value(&mut t.recurrence, r, r.label());
                            }
                            ui.selectable_value(&mut t.recurrence, Recurrence::EveryDays(2), "Every 2 days");
                            ui.selectable_value(&mut t.recurrence, Recurrence::EveryWeeks(2), "Every 2 weeks");
                        });
                        ui.end_row();
                        ui.label(RichText::new("Reminder").color(p.dim));
                        let rl = |m: Option<i64>| match m {
                            None => "None".to_string(),
                            Some(0) => "At due time".to_string(),
                            Some(m) if m < 60 => format!("{m} min before"),
                            Some(m) if m < 1440 => format!("{} h before", m / 60),
                            Some(m) => format!("{} day before", m / 1440),
                        };
                        ui.add_enabled_ui(t.due_time.is_some(), |ui| {
                            egui::ComboBox::from_id_salt("tp-rem").selected_text(if t.due_time.is_some() { rl(t.reminder_min) } else { "Set a time first".into() }).width(200.0).show_ui(ui, |ui| {
                                for m in [None, Some(0), Some(10), Some(30), Some(60), Some(120), Some(1440)] {
                                    ui.selectable_value(&mut t.reminder_min, m, rl(m));
                                }
                            });
                        });
                        ui.end_row();
                    }
                    ui.label(RichText::new("Priority").color(p.dim));
                    ui.horizontal(|ui| {
                        for pr in 1..=4u8 {
                            let sel = t.priority == pr;
                            let col = prio_color(pr, p);
                            if ui.add(egui::Button::new(RichText::new(format!("⚑ P{pr}")).color(col).size(13.0)).fill(if sel { col.linear_multiply(0.2) } else { Color32::TRANSPARENT }).stroke(Stroke::new(1.0, if sel { col } else { p.line }))).on_hover_text(priority_label(pr)).clicked() {
                                t.priority = pr;
                            }
                        }
                    });
                    ui.end_row();
                    ui.label(RichText::new("Labels").color(p.dim));
                    let key = format!("labels-{}", t.id);
                    let mut lt = self.temp.get(&key).cloned().unwrap_or_else(|| t.labels.join(", "));
                    let r = ui.add(egui::TextEdit::singleline(&mut lt).hint_text("home, phone").desired_width(200.0));
                    if r.changed() {
                        self.temp.insert(key.clone(), lt.clone());
                    }
                    if r.lost_focus() {
                        t.labels = lt.split(',').map(|s| s.trim().trim_start_matches('@').to_string()).filter(|s| !s.is_empty()).collect();
                        self.temp.remove(&key);
                    }
                    ui.end_row();
                    ui.label(RichText::new("Document").color(p.dim));
                    ui.horizontal(|ui| {
                        let current = t.page_id.as_deref().and_then(|x| d.page(x)).map(|x| format!("{} {}", x.icon, x.title)).unwrap_or_else(|| "None".into());
                        egui::ComboBox::from_id_salt("tp-page").selected_text(current).width(170.0).show_ui(ui, |ui| {
                            ui.selectable_value(&mut t.page_id, None, "None");
                            for pg in &d.pages {
                                ui.selectable_value(&mut t.page_id, Some(pg.id.clone()), format!("{} {}", pg.icon, if pg.title.is_empty() { "Untitled" } else { &pg.title }));
                            }
                        });
                        if let Some(pg) = &t.page_id {
                            if ui.small_button("Open").clicked() {
                                open_page = Some(pg.clone());
                            }
                        } else if ui.small_button("+ New doc").on_hover_text("Create a page linked to this task").clicked() {
                            new_doc = true;
                        }
                    });
                    ui.end_row();
                });
                ui.add_space(12.0);
                if t.due.is_some() {
                    ui.label(RichText::new("📆 Shows on your calendar on its due date.").size(12.0).color(p.dim));
                }
                let created = chrono::DateTime::from_timestamp_millis(t.created).map(|x| x.with_timezone(&chrono::Local).format("%-d %b %Y %H:%M").to_string()).unwrap_or_default();
                ui.label(RichText::new(format!("Created {created}")).size(11.0).color(p.faint));
            });
        });
        let after = serde_json::to_string(&t).unwrap_or_default();
        if after != before {
            if t.due.is_none() {
                t.due_time = None;
            }
            if t.due_time.is_none() && t.reminder_min.is_some() && t.due.is_none() {
                t.reminder_min = None;
            }
            // moving a task moves its subtasks too
            let moved_project = self.store.data.task(tid).map(|o| o.project_id != t.project_id || o.section != t.section).unwrap_or(false);
            t.updated = now_ms();
            let (proj, sec) = (t.project_id.clone(), t.section.clone());
            *self.store.data.task_mut(tid).unwrap() = t.clone();
            if moved_project {
                let subs: Vec<Id> = self.store.data.subtasks(tid).iter().map(|s| s.id.clone()).collect();
                for s in subs {
                    if let Some(x) = self.store.data.task_mut(&s) {
                        x.project_id = proj.clone();
                        x.section = sec.clone();
                        x.updated = now_ms();
                    }
                }
            }
            self.touch();
        }
        if toggle {
            self.toggle_task(tid);
        }
        if let Some(s) = toggle_sub {
            self.toggle_task(&s);
        }
        if let Some(text) = add_sub {
            self.add_task_text(&text, None, "", None, Some(tid.to_string()), None);
        }
        if new_doc {
            let mut pg = Page::new(t.title.clone());
            pg.icon = "📝".into();
            let id = pg.id.clone();
            self.store.data.pages.push(pg);
            if let Some(x) = self.store.data.task_mut(tid) {
                x.page_id = Some(id.clone());
                x.updated = now_ms();
            }
            self.touch();
            self.go(View::Page(id));
        }
        if let Some(pg) = open_page {
            self.go(View::Page(pg));
        }
        if let Some(s) = open_sub {
            self.open_task = Some(s);
        }
        if delete {
            let title = t.title.clone();
            { let t2 = tid.to_string(); let name = self.store.data.task(&t2).map(|t| t.title.clone()).unwrap_or_default(); self.undoable(&format!("Deleted task \"{name}\""), "delete", |d| d.delete_task(&t2)); }
            self.open_task = None;
            self.touch();
            self.toast(format!("Deleted \"{title}\""));
        }
        if close {
            self.open_task = None;
        }
    }
}
