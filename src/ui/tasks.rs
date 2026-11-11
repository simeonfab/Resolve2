use super::theme::*;
use super::widgets::*;
use super::*;
use chrono::Datelike;
use eframe::egui::{self, Color32, RichText, Sense, Stroke};

enum TAct {
    Toggle(Id),
    Open(Id),
    Move(Id, String),
    /// A Dependency record waited on has arrived (Met).
    WaitDone(Id),
    ShowDone,
    Organise,
    /// A card dropped on a Board column (a section, or "Done").
    BoardMove(Id, String),
    /// "+ Add" under a Board column.
    AddIn(String),
}

/// What the one-project task list shows, worked out from the workspace (B-E8). Self-checked.
pub struct ProjectTasks {
    /// (task, its theme) in Now: the project's Now section (or no section)
    pub now: Vec<(Id, String)>,
    /// (task or dependency record, title, how long it has waited, theme, is it a task?)
    pub waiting: Vec<(Id, String, String, String, bool)>,
    /// (task, theme, "Next" or "Later")
    pub upcoming: Vec<(Id, String, String)>,
    /// Other sections the person made: (section, tasks)
    pub other: Vec<(String, Vec<(Id, String)>)>,
    pub done_week: usize,
    pub open: usize,
}

pub fn project_tasks_model(d: &WorkspaceData, pid: &str, real_ms: i64, now: NaiveDateTime) -> ProjectTasks {
    use crate::resolve::records::Kind;
    let theme_of = |tid: &str| d.records.iter().find(|r| r.kind == Kind::Roadmap && r.project_id.as_deref() == Some(pid) && r.links.iter().any(|l| l == tid)).map(|r| r.title.clone()).unwrap_or_default();
    let mut tasks: Vec<&Task> = d.tasks.iter().filter(|t| t.project_id.as_deref() == Some(pid) && t.parent_id.is_none()).collect();
    tasks.sort_by_key(|t| t.order);
    let open: Vec<&&Task> = tasks.iter().filter(|t| !t.completed).collect();
    let monday = now.date() - chrono::Duration::days(now.date().weekday().num_days_from_monday() as i64);
    let monday_ms = real_ms - (now - monday.and_time(NaiveTime::MIN)).num_milliseconds();
    let mut m = ProjectTasks { now: vec![], waiting: vec![], upcoming: vec![], other: vec![], done_week: tasks.iter().filter(|t| t.completed && t.completed_at.map_or(false, |c| c >= monday_ms)).count(), open: open.len() };
    let age = |since: i64| {
        let days = ((real_ms - since).max(0) / (24 * 3600 * 1000)) as usize;
        if days == 0 { "Waiting · since today".to_string() } else if days == 1 { "Waiting · 1 day".to_string() } else { format!("Waiting · {days} days") }
    };
    for t in &open {
        if let Some(since) = t.waiting_since {
            m.waiting.push((t.id.clone(), t.title.clone(), age(since), theme_of(&t.id), true));
            continue;
        }
        match t.section.as_str() {
            "" | "Now" => m.now.push((t.id.clone(), theme_of(&t.id))),
            "Next" | "Later" => m.upcoming.push((t.id.clone(), theme_of(&t.id), t.section.clone())),
            s => match m.other.iter_mut().find(|o| o.0 == s) {
                Some(o) => o.1.push((t.id.clone(), theme_of(&t.id))),
                None => m.other.push((s.to_string(), vec![(t.id.clone(), theme_of(&t.id))])),
            },
        }
    }
    m.upcoming.sort_by_key(|u| if u.2 == "Next" { 0 } else { 1 });
    for r in d.records.iter().filter(|r| r.kind == Kind::Dependency && r.project_id.as_deref() == Some(pid) && r.status == "Waiting") {
        let theme = d.records.iter().find(|x| x.kind == Kind::Roadmap && x.project_id.as_deref() == Some(pid) && x.links.contains(&r.id)).map(|x| x.title.clone()).unwrap_or_default();
        m.waiting.push((r.id.clone(), r.title.clone(), age(r.created), theme, false));
    }
    m
}

/// What All projects shows (B-E8, Pack B "Tasks: all projects"), worked out from the workspace.
/// Self-checked.
pub struct AllTasks {
    /// open top-level tasks with no project, oldest first
    pub captured: Vec<Id>,
    /// open tasks whose date has passed
    pub overdue: Vec<Id>,
    /// open tasks due today, by project in sidebar order, then "Not in a project" (None)
    pub today: Vec<(Option<Id>, Vec<Id>)>,
    /// open tasks with a later date, by day
    pub upcoming: Vec<(NaiveDate, Vec<Id>)>,
}

impl AllTasks {
    pub fn today_count(&self) -> usize {
        self.today.iter().map(|g| g.1.len()).sum()
    }
    pub fn upcoming_count(&self) -> usize {
        self.upcoming.iter().map(|g| g.1.len()).sum()
    }
}

pub fn all_tasks_model(d: &WorkspaceData, t0: NaiveDate) -> AllTasks {
    let key = |t: &Task| (t.due_time.is_none(), t.due_time, t.priority, t.order);
    let mut open: Vec<&Task> = d.tasks.iter().filter(|t| !t.completed).collect();
    open.sort_by_key(|t| key(t));
    let captured = {
        let mut v: Vec<&Task> = open.iter().copied().filter(|t| t.project_id.is_none() && t.parent_id.is_none()).collect();
        v.sort_by_key(|t| t.order);
        v.into_iter().map(|t| t.id.clone()).collect()
    };
    let mut overdue: Vec<&Task> = open.iter().copied().filter(|t| t.due.map_or(false, |x| x < t0)).collect();
    overdue.sort_by_key(|t| t.due);
    let mut today: Vec<(Option<Id>, Vec<Id>)> = vec![];
    for pr in d.projects.iter().filter(|x| !x.archived) {
        let v: Vec<Id> = open.iter().filter(|t| t.due == Some(t0) && t.project_id.as_deref() == Some(pr.id.as_str())).map(|t| t.id.clone()).collect();
        if !v.is_empty() {
            today.push((Some(pr.id.clone()), v));
        }
    }
    let loose: Vec<Id> = open.iter().filter(|t| t.due == Some(t0) && t.project_id.as_ref().map_or(true, |p| d.project(p).map_or(true, |x| x.archived))).map(|t| t.id.clone()).collect();
    if !loose.is_empty() {
        today.push((None, loose));
    }
    let mut days: Vec<NaiveDate> = open.iter().filter_map(|t| t.due).filter(|x| *x > t0).collect();
    days.sort();
    days.dedup();
    let upcoming = days.into_iter().map(|day| (day, open.iter().filter(|t| t.due == Some(day)).map(|t| t.id.clone()).collect())).collect();
    AllTasks { captured, overdue: overdue.into_iter().map(|t| t.id.clone()).collect(), today, upcoming }
}

/// The when line on All projects: "Today 16:00 · every month", "Due today", "Fri 13 Nov".
/// Under a day heading the date is left out ("09:30", or nothing).
pub fn all_when(t: &Task, t0: NaiveDate, with_date: bool) -> (String, bool) {
    let time = t.due_time.map(|x| x.format("%H:%M").to_string());
    let (mut s, urgent) = match (t.due, &time) {
        (Some(d), Some(tm)) if d == t0 => (format!("Today {tm}"), true),
        (Some(d), Some(tm)) if d > t0 && with_date => (format!("{} {tm}", d.format("%a %-d %b")), false),
        (Some(d), Some(tm)) if d > t0 => (tm.clone(), false),
        (Some(d), None) if d > t0 && !with_date => (String::new(), false),
        _ => when_label(t, t0),
    };
    if t.recurrence.is_some() {
        let r = t.recurrence.label().to_lowercase();
        s = if s.is_empty() { r } else { format!("{s} · {r}") };
    }
    (s, urgent)
}

/// The Board (B-E8, Pack B "Tasks: one project, board"): Now, Next, Later, any sections of your
/// own, and Done, on surface-sunk columns; cards drag between columns (a move or a completion,
/// both undoable); "+ Add" per column.
fn draw_board(ui: &mut egui::Ui, p: &Pal, d: &WorkspaceData, pid: &str, t0: NaiveDate, monday_ms: i64, acts: &mut Vec<TAct>) {
    use crate::resolve::records::Kind;
    let theme_of = |tid: &str| d.records.iter().find(|r| r.kind == Kind::Roadmap && r.project_id.as_deref() == Some(pid) && r.links.iter().any(|l| l == tid)).map(|r| r.title.clone()).unwrap_or_default();
    let mut tasks: Vec<&Task> = d.tasks.iter().filter(|t| t.project_id.as_deref() == Some(pid) && t.parent_id.is_none()).collect();
    tasks.sort_by_key(|t| t.order);
    let mut cols: Vec<(String, Vec<&Task>)> = vec![("Now".into(), vec![]), ("Next".into(), vec![]), ("Later".into(), vec![])];
    let mut done: Vec<&Task> = vec![];
    for t in tasks {
        if t.completed {
            if t.completed_at.map_or(false, |c| c >= monday_ms) {
                done.push(t);
            }
            continue;
        }
        let sec = if t.section.is_empty() { "Now" } else { t.section.as_str() };
        match cols.iter_mut().find(|c| c.0 == sec) {
            Some(c) => c.1.push(t),
            None => cols.push((sec.to_string(), vec![t])),
        }
    }
    cols.push(("Done".into(), done));
    let gap = space::S12;
    let n = cols.len() as f32;
    let w = ((ui.available_width().min(space::CAP_TASKS) - gap * (n - 1.0)) / n).max(space::S40 * 5.0);
    // finite sizes: inside the page's scroll area the available height is unlimited
    let h = (ui.ctx().screen_rect().height() - space::BOARD_TOP_OFFSET).max(space::S40 * 8.0);
    egui::ScrollArea::horizontal().id_salt(("board", pid)).auto_shrink([false, true]).show(ui, |ui| {
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            for (name, items) in &cols {
                let frame = egui::Frame::none().fill(p.surface_sunk).rounding(egui::Rounding::same(radius::CARD)).inner_margin(egui::Margin::symmetric(space::S10, space::S12));
                let (_, dropped) = ui.allocate_ui_with_layout(egui::vec2(w, h), Layout::top_down(Align::Min), |ui| {
                    ui.set_width(w);
                    ui.dnd_drop_zone::<String, ()>(frame, |ui| {
                        ui.set_width(w - space::S10 * 2.0);
                        ui.set_min_height(h - space::S12 * 2.0);
                        ui.spacing_mut().item_spacing.y = space::S8;
                        ui.horizontal(|ui| {
                            ui.add_space(space::S4);
                            ui.label(RichText::new(name).font(fs(size::ROW_SMALL)).color(p.text));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.add_space(space::S4);
                                ui.label(RichText::new(items.len().to_string()).size(size::LABEL).color(p.dim));
                            });
                        });
                        for t in items {
                            let when = if t.completed {
                                t.completed_at.and_then(crate::model::local_at).map(|x| format!("Done {}", x.format("%A"))).unwrap_or_else(|| "Done".into())
                            } else {
                                when_label(t, t0).0
                            };
                            let (clicked, _) = board_card(ui, p, &t.id, &t.title, &theme_of(&t.id), &when, t.completed);
                            if clicked {
                                acts.push(TAct::Open(t.id.clone()));
                            }
                        }
                        if name != "Done" && ui.add(egui::Label::new(RichText::new("+ Add").font(fs(size::LABEL)).color(p.dim)).sense(Sense::click())).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                            acts.push(TAct::AddIn(name.clone()));
                        }
                    })
                })
                .inner;
                if let Some(id) = dropped {
                    acts.push(TAct::BoardMove((*id).clone(), name.clone()));
                }
            }
        });
    });
}

/// A task row's text, owned (the drawing borrows it as a `TaskLine`).
struct TaskLineOwned {
    title: String,
    notes: String,
    when: String,
    urgent: bool,
    theme: String,
    basis: String,
    done: bool,
}

impl TaskLineOwned {
    fn from(l: (String, bool, String, String, String), t: &Task) -> Self {
        TaskLineOwned { title: l.0, urgent: l.1, when: l.2, theme: l.3, basis: l.4, notes: t.notes.lines().next().unwrap_or("").to_string(), done: t.completed }
    }
    fn view(&self) -> TaskLine<'_> {
        TaskLine { title: &self.title, notes: &self.notes, when: &self.when, urgent: self.urgent, theme: &self.theme, basis: &self.basis, done: self.done }
    }
}

/// "Due today", "Overdue · 3 Nov", "Fri 13 Nov", or the phase for undated upcoming work.
fn when_label(t: &Task, t0: NaiveDate) -> (String, bool) {
    match t.due {
        Some(d) if d < t0 => (format!("Overdue · {}", d.format("%-d %b")), true),
        Some(d) if d == t0 => ("Due today".to_string(), true),
        Some(d) => (d.format("%a %-d %b").to_string(), false),
        None => (String::new(), false),
    }
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
                let r = ui.add(egui::TextEdit::singleline(&mut self.tasks_ui.add_text).hint_text("Task name, for example: Pay rent every month 9am p1 @bills").frame(false).font(fs(14.5)).desired_width(f32::INFINITY));
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

    /// All tasks, filtered: what used to be Inbox, Today, Upcoming and Filters & Labels in the
    /// sidebar. Picking a project in the sidebar shows that project's tasks (and board) instead.
    fn task_filters(&mut self, ui: &mut egui::Ui, p: &Pal) {
        let d = &self.store.data;
        let captured = d.tasks.iter().filter(|t| !t.completed && t.project_id.is_none() && t.parent_id.is_none()).count();
        let today_n = d.tasks.iter().filter(|t| !t.completed && t.due.map_or(false, |x| x <= today())).count();
        let mut go: Option<View> = None;
        ui.add_space(12.0);
        ui.horizontal_wrapped(|ui| {
            for (label, n, v) in [("📥 Captured", Some(captured), View::Inbox), ("📅 Today", Some(today_n), View::Today), ("🗓 Upcoming", None, View::Upcoming), ("🏷 Filters & labels", None, View::Filters)] {
                let text = match n {
                    Some(n) if n > 0 => format!("{label} {n}"),
                    _ => label.to_string(),
                };
                if pill(ui, p, &text, self.view == v).clicked() {
                    go = Some(v);
                }
            }
        });
        if let Some(v) = go {
            self.view = v;
        }
    }

    fn sort_ids(&self, mut v: Vec<&Task>) -> Vec<Id> {
        v.sort_by_key(|t| (t.completed, t.order));
        v.into_iter().map(|t| t.id.clone()).collect()
    }

    /// One project's tasks (B-E8, Pack B "Tasks: one project, list"): the project as eyebrow,
    /// List / Board, Organise, Add task, this week's progress and View completed, then Now,
    /// Waiting on and Upcoming from roadmap; empty, loading and error as drawn.
    fn project_tasks(&mut self, ui: &mut egui::Ui, p: &Pal, pid: &str, acts: &mut Vec<TAct>) {
        let Some(pr) = self.store.data.project(pid).cloned() else { return };
        let t0 = today();
        let careers = self.store.data.mode == Mode::Careers;
        let mut template: Option<usize> = None;
        let m = project_tasks_model(&self.store.data, pid, now_ms(), now_local());
        let state = match self.view_state {
            ViewState::Full if m.open == 0 && m.done_week == 0 && m.waiting.is_empty() => ViewState::Empty,
            s => s,
        };
        let show_done = self.tasks_ui.show_completed;
        let board_on = self.tasks_ui.board.contains(pid);
        let mut board: Option<bool> = None;
        let mut add = false;
        let mut project_act: Option<&str> = None;
        let mut colour: Option<[u8; 3]> = None;
        let mut open_roadmap = false;
        let el = ((self.now - self.view_since) * 1000.0) as f32;
        let reduced = super::motion::reduced();
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            egui::Frame::none().inner_margin(egui::Margin { left: space::WORK_SIDE, right: space::WORK_SIDE, top: space::WORK_TOP, bottom: space::S24 }).show(ui, |ui| {
                capped(ui, space::CAP_TASKS, |ui| {
                    ui.label(RichText::new(&pr.name).font(fs(size::ROW_SMALL)).color(p.orange));
                    screen_head(ui, p, "Tasks", "", |ui| {
                        // the project's own actions: its document, colour, archive, delete
                        ui.menu_button(RichText::new("⋯").font(fs(size::SECTION)).color(p.dim), |ui| {
                            let doc = pr.page_id.clone().filter(|x| self.store.data.page(x).is_some());
                            if ui.button(if doc.is_some() { "Open the project document" } else { "Create a project document" }).clicked() {
                                project_act = Some("doc");
                                ui.close_menu();
                            }
                            ui.label(RichText::new("Colour").size(size::SMALL).color(p.dim));
                            ui.horizontal(|ui| {
                                for c in PICK_COLOURS {
                                    let (r, resp) = ui.allocate_exact_size(egui::vec2(space::S18, space::S18), Sense::click());
                                    ui.painter().circle_filled(r.center(), space::S8, rgb(c));
                                    if resp.clicked() {
                                        colour = Some(c);
                                    }
                                }
                            });
                            ui.separator();
                            if ui.button("Archive project").clicked() {
                                project_act = Some("archive");
                                ui.close_menu();
                            }
                            if ui.button(RichText::new("Delete project and its tasks").color(p.orange)).clicked() {
                                project_act = Some("delete");
                                ui.close_menu();
                            }
                        });
                        let r = button(ui, p, ButtonKind::Dark, "+ Add task", false, true);
                        if careers {
                            // Careers: Blank task, then the templates (Careers brief, "Tasks")
                            let menu = egui::Id::new(("careers-add-task", pid));
                            if r.clicked() {
                                ui.memory_mut(|mm| mm.toggle_popup(menu));
                            }
                            egui::popup_below_widget(ui, menu, &r, egui::PopupCloseBehavior::CloseOnClick, |ui| {
                                ui.set_min_width(space::S40 * 7.0);
                                if ui.button("Blank task").clicked() {
                                    add = true;
                                }
                                template_menu(ui, p, &mut template);
                            });
                        } else if r.clicked() {
                            add = true;
                        }
                        if button(ui, p, ButtonKind::Secondary, "✦ Organise", false, true).on_hover_text("Moves the to-dos in your pages into tasks, with dates").clicked() {
                            acts.push(TAct::Organise);
                        }
                        if let Some(i) = segmented(ui, p, egui::Id::new(("tasks-layout", pid)), &["List", "Board"], board_on as usize) {
                            board = Some(i == 1);
                        }
                    });
                    ui.add_space(space::S10);
                    ui.horizontal(|ui| {
                        let total = m.open + m.done_week;
                        progress_line(ui, p, if total == 0 { 0.0 } else { m.done_week as f32 / total as f32 }, space::S40 * 9.0);
                        ui.add_space(space::S6);
                        ui.label(RichText::new(format!("{} of {total} done this week", m.done_week)).size(size::LABEL).color(p.dim));
                        if ui.add(egui::Label::new(RichText::new(if show_done { "Hide completed ›" } else { "View completed ›" }).font(fs(size::LABEL)).color(p.orange)).sense(Sense::click())).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                            acts.push(TAct::ShowDone);
                        }
                    });
                    ui.add_space(space::S18);
                    match state {
                        ViewState::Loading => {
                            for _ in 0..3 {
                                skeleton_card(ui, p, 2, "");
                                ui.add_space(space::S12);
                            }
                            ui.label(RichText::new("Loading tasks…").size(size::SMALL).color(p.dim));
                        }
                        ViewState::Error => {
                            ui.add_space(space::S40);
                            let a = error_center(ui, p, "Resolve couldn't load these tasks.", "Nothing is lost. Your tasks are saved on this PC.", "Open the roadmap");
                            if a.retry {
                                self.store.reload_if_changed();
                            }
                            if a.manual {
                                open_roadmap = true;
                            }
                        }
                        ViewState::Empty => {
                            ui.add_space(space::S40);
                            if empty_center(ui, p, Icon::Tasks, "No tasks in this project yet.", "Add the first one, or open the roadmap and turn a theme into tasks.", "Add task") {
                                add = true;
                            }
                            ui.vertical_centered(|ui| {
                                if button(ui, p, ButtonKind::Ghost, "Open the roadmap", false, true).clicked() {
                                    open_roadmap = true;
                                }
                            });
                        }
                        ViewState::Full if board_on => {
                            let now = now_local();
                            let monday = t0 - chrono::Duration::days(t0.weekday().num_days_from_monday() as i64);
                            let monday_ms = now_ms() - (now - monday.and_time(NaiveTime::MIN)).num_milliseconds();
                            draw_board(ui, p, &self.store.data, pid, t0, monday_ms, acts);
                        }
                        ViewState::Full => {
                            let d = &self.store.data;
                            let line = |t: &Task, theme: &str, phase: &str| -> (String, bool, String, String, String) {
                                let (mut when, urgent) = when_label(t, t0);
                                if when.is_empty() {
                                    when = phase.to_string();
                                }
                                let basis = t.labels.iter().find(|l| matches!(l.as_str(), "evidence" | "assumption" | "uncertain")).cloned().unwrap_or_default();
                                (t.title.clone(), urgent, when, theme.to_string(), basis)
                            };
                            let mut n = 0;
                            let mut group = |ui: &mut egui::Ui, dot: Color32, title: &str, note: &str, rows: Vec<(Id, TaskLineOwned)>, acts: &mut Vec<TAct>, waiting: bool| {
                                if rows.is_empty() {
                                    return;
                                }
                                let f = super::motion::frame_at(el, moves::CARDS.stagger(n, moves::CARDS_STAGGER), reduced);
                                n += 1;
                                super::motion::fade_in(ui, f, egui::vec2(0.0, space::S18), |ui| {
                                    group_head(ui, p, dot, title, note);
                                    ui.add_space(space::S8);
                                    list_box(ui, p, |ui| {
                                        for (i, (id, l)) in rows.iter().enumerate() {
                                            let (tog, open) = task_line(ui, p, &l.view(), i == 0);
                                            // a waiting row is a task, or a project dependency (a record)
                                            let is_task = !waiting || d.task(id).is_some();
                                            if tog {
                                                acts.push(if is_task { TAct::Toggle(id.clone()) } else { TAct::WaitDone(id.clone()) });
                                            }
                                            if open && is_task {
                                                acts.push(TAct::Open(id.clone()));
                                            }
                                        }
                                    });
                                    ui.add_space(space::S22);
                                });
                            };
                            let now_rows: Vec<(Id, TaskLineOwned)> = m.now.iter().filter_map(|(id, th)| d.task(id).map(|t| (id.clone(), TaskLineOwned::from(line(t, th, ""), t)))).collect();
                            group(ui, tok::AMBER, "Now", &format!("{} active", now_rows.len()), now_rows, acts, false);
                            let wait_rows: Vec<(Id, TaskLineOwned)> = m.waiting.iter().map(|(id, title, age, th, _)| (id.clone(), TaskLineOwned { title: title.clone(), notes: String::new(), when: age.clone(), urgent: false, theme: th.clone(), basis: String::new(), done: false })).collect();
                            let nw = wait_rows.len();
                            group(ui, p.orange, if careers { "Waiting to hear back" } else { "Waiting on" }, &format!("{nw} stalled"), wait_rows, acts, true);
                            let up_rows: Vec<(Id, TaskLineOwned)> = m.upcoming.iter().filter_map(|(id, th, ph)| d.task(id).map(|t| (id.clone(), TaskLineOwned::from(line(t, th, ph), t)))).collect();
                            let nu = up_rows.len();
                            // Careers has no roadmap: plain Upcoming
                            group(ui, p.faint, if careers { "Upcoming" } else { "Upcoming from roadmap" }, &format!("{nu} queued"), up_rows, acts, false);
                            for (sec, rows) in &m.other {
                                let rows: Vec<(Id, TaskLineOwned)> = rows.iter().filter_map(|(id, th)| d.task(id).map(|t| (id.clone(), TaskLineOwned::from(line(t, th, ""), t)))).collect();
                                let k = rows.len();
                                group(ui, p.dim, sec, &format!("{k} open"), rows, acts, false);
                            }
                            if show_done {
                                let mut done: Vec<&Task> = d.tasks.iter().filter(|t| t.project_id.as_deref() == Some(pid) && t.parent_id.is_none() && t.completed).collect();
                                done.sort_by_key(|t| std::cmp::Reverse(t.completed_at));
                                let rows: Vec<(Id, TaskLineOwned)> = done.iter().map(|t| (t.id.clone(), TaskLineOwned::from(line(t, "", ""), t))).collect();
                                let k = rows.len();
                                group(ui, p.green, "Completed", &format!("{k} done"), rows, acts, false);
                            }
                        }
                    }
                });
            });
        });
        if el < moves::CARDS.stagger(4, moves::CARDS_STAGGER).end() {
            ui.ctx().request_repaint();
        }
        match board {
            Some(true) => {
                self.tasks_ui.board.insert(pid.to_string());
            }
            Some(false) => {
                self.tasks_ui.board.remove(pid);
            }
            None => {}
        }
        if let Some(c) = colour {
            let id = pid.to_string();
            self.undoable("Changed a project's colour", "edit", move |d| {
                if let Some(x) = d.projects.iter_mut().find(|x| x.id == id) {
                    x.color = c;
                    x.updated = now_ms();
                }
            });
        }
        match project_act {
            Some("doc") => match pr.page_id.clone().filter(|x| self.store.data.page(x).is_some()) {
                Some(pg) => self.go(View::Page(pg)),
                None => {
                    let mut pg = Page::new(format!("{}: Project brief", pr.name));
                    pg.blocks = vec![Block::new(BlockKind::H2, "Goal"), Block::new(BlockKind::Paragraph, ""), Block::new(BlockKind::H2, "Notes"), Block::new(BlockKind::Paragraph, "")];
                    let pgid = pg.id.clone();
                    self.store.data.pages.push(pg);
                    if let Some(x) = self.store.data.projects.iter_mut().find(|x| x.id == pid) {
                        x.page_id = Some(pgid.clone());
                        x.updated = now_ms();
                    }
                    self.touch();
                    self.go(View::Page(pgid));
                }
            },
            Some("archive") => {
                let id = pid.to_string();
                self.undoable(&format!("Archived \"{}\"", pr.name), "edit", move |d| {
                    if let Some(x) = d.projects.iter_mut().find(|x| x.id == id) {
                        x.archived = true;
                        x.updated = now_ms();
                    }
                });
                self.current_project = None;
                self.go(View::Projects);
            }
            Some("delete") => {
                let id = pid.to_string();
                self.undoable(&format!("Deleted project \"{}\"", pr.name), "delete", move |d| d.delete_project(&id));
                self.current_project = None;
                self.go(View::Projects);
            }
            _ => {}
        }
        if add {
            self.open_quick(QuickKind::Task);
        }
        if let Some(k) = template {
            let (title, days) = crate::resolve::careers::TASK_TEMPLATES[k];
            let mut t = Task::new(title.to_string());
            t.project_id = Some(pid.to_string());
            t.section = "Now".into();
            t.due = days.and_then(|n| today().checked_add_days(chrono::Days::new(n)));
            t.labels = vec!["template".into()];
            // record_id links it to its Pipeline role once Pipeline exists (Step 4)
            t.order = now_ms();
            self.undoable(&format!("Added \"{title}\""), "add", move |d| d.tasks.push(t));
        }
        if open_roadmap {
            self.go(View::Sec(super::nav::Section::Roadmap));
        }
    }

    /// All projects (B-E8, Pack B "Tasks: all projects"): Captured, Today (by project, then not in
    /// a project), Upcoming (by day) and Filters and labels, as tab chips with counts.
    fn all_projects(&mut self, ui: &mut egui::Ui, p: &Pal, acts: &mut Vec<TAct>) {
        use crate::resolve::records::Kind;
        let t0 = today();
        let view = self.view.clone();
        let m = all_tasks_model(&self.store.data, t0);
        let mut go: Option<View> = None;
        let mut add = false;
        let mut to_today: Vec<Id> = vec![];
        let mut filter: Option<String> = None;
        let cur_filter = self.tasks_ui.label.clone().unwrap_or_else(|| "f:p1".into());
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            egui::Frame::none().inner_margin(egui::Margin { left: space::WORK_SIDE, right: space::WORK_SIDE, top: space::WORK_TOP, bottom: space::S24 }).show(ui, |ui| {
                capped(ui, space::CAP_TASKS, |ui| {
                    let d = &self.store.data;
                    ui.label(RichText::new("All projects").font(fs(size::ROW_SMALL)).color(p.orange));
                    screen_head(ui, p, "Tasks", "", |ui| {
                        if button(ui, p, ButtonKind::Dark, "+ Add task", false, true).clicked() {
                            add = true;
                        }
                        if button(ui, p, ButtonKind::Secondary, "✦ Organise", false, true).on_hover_text("Moves the to-dos in your pages into tasks, with dates").clicked() {
                            acts.push(TAct::Organise);
                        }
                    });
                    ui.add_space(space::S10);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = space::S6;
                        for (label, n, v) in [("Captured", Some(m.captured.len()), View::Inbox), ("Today", Some(m.today_count()), View::Today), ("Upcoming", Some(m.upcoming_count()), View::Upcoming), ("Filters and labels", None, View::Filters)] {
                            if tab_chip(ui, p, label, n, view == v).clicked() {
                                go = Some(v);
                            }
                        }
                    });
                    ui.add_space(space::S22);
                    let theme_of = |tid: &str| d.records.iter().find(|r| r.kind == Kind::Roadmap && r.links.iter().any(|l| l == tid)).map(|r| r.title.clone()).unwrap_or_default();
                    let row = |t: &Task, with_date: bool, project_too: bool| -> TaskLineOwned {
                        let (when, urgent) = all_when(t, t0, with_date);
                        let mut theme = theme_of(&t.id);
                        if let Some(pn) = t.project_id.as_ref().filter(|_| project_too).and_then(|x| d.project(x)).map(|x| x.name.clone()) {
                            {
                                theme = if theme.is_empty() { pn.to_string() } else { format!("{pn} · {theme}") };
                            }
                        }
                        let basis = t.labels.iter().find(|l| matches!(l.as_str(), "evidence" | "assumption" | "uncertain")).cloned().unwrap_or_default();
                        TaskLineOwned { title: t.title.clone(), notes: String::new(), when, urgent, theme, basis, done: t.completed }
                    };
                    let rows_box = |ui: &mut egui::Ui, ids: &[Id], with_date: bool, project_too: bool, acts: &mut Vec<TAct>| {
                        list_box(ui, p, |ui| {
                            for (i, id) in ids.iter().enumerate() {
                                let Some(t) = d.task(id) else { continue };
                                let l = row(t, with_date, project_too);
                                let (tog, open) = task_line(ui, p, &l.view(), i == 0);
                                if tog {
                                    acts.push(TAct::Toggle(id.clone()));
                                }
                                if open {
                                    acts.push(TAct::Open(id.clone()));
                                }
                            }
                        });
                        ui.add_space(space::S18);
                    };
                    let empty = |ui: &mut egui::Ui, title: &str, body: &str| -> bool {
                        ui.add_space(space::S40);
                        empty_center(ui, p, Icon::Tasks, title, body, "Add task")
                    };
                    match self.view_state {
                        ViewState::Loading => {
                            for _ in 0..3 {
                                skeleton_card(ui, p, 2, "");
                                ui.add_space(space::S12);
                            }
                            ui.label(RichText::new("Loading tasks…").size(size::SMALL).color(p.dim));
                        }
                        ViewState::Error => {
                            ui.add_space(space::S40);
                            let a = error_center(ui, p, "Resolve couldn't load your tasks.", "Nothing is lost. Your tasks are saved on this PC.", "Add a task");
                            if a.retry {
                                self.store.reload_if_changed();
                            }
                            add |= a.manual;
                        }
                        _ => match &view {
                            View::Inbox => {
                                if m.captured.is_empty() || self.view_state == ViewState::Empty {
                                    add |= empty(ui, "Nothing captured.", "Tasks you add without a project wait here until you give them one.");
                                } else {
                                    group_head(ui, p, p.faint, "Not in a project yet", &m.captured.len().to_string());
                                    ui.add_space(space::S8);
                                    rows_box(ui, &m.captured, true, false, acts);
                                }
                            }
                            View::Upcoming => {
                                if m.upcoming.is_empty() || self.view_state == ViewState::Empty {
                                    add |= empty(ui, "Nothing scheduled ahead.", "Tasks with a later date show here, by day.");
                                } else {
                                    for (day, ids) in &m.upcoming {
                                        let name = if *day == t0 + chrono::Duration::days(1) { format!("Tomorrow · {}", day.format("%a %-d %b")) } else { day.format("%A %-d %B").to_string() };
                                        group_head(ui, p, p.faint, &name, &ids.len().to_string());
                                        ui.add_space(space::S8);
                                        rows_box(ui, ids, false, true, acts);
                                    }
                                }
                            }
                            View::Filters => {
                                let mut labels: Vec<String> = vec![];
                                for t in &d.tasks {
                                    for l in &t.labels {
                                        if !labels.contains(l) {
                                            labels.push(l.clone());
                                        }
                                    }
                                }
                                labels.sort();
                                ui.label(RichText::new("Filters").font(fs(size::CARD_HEAD)).color(p.dim));
                                ui.add_space(space::S6);
                                ui.horizontal_wrapped(|ui| {
                                    ui.spacing_mut().item_spacing = egui::vec2(space::S6, space::S6);
                                    for (k, l) in [("f:p1", "Priority 1"), ("f:p2", "Priority 2"), ("f:overdue", "Overdue"), ("f:nodate", "No date"), ("f:recurring", "Repeating"), ("f:waiting", "Waiting on someone"), ("f:linked", "Linked to a page"), ("f:done", "Completed")] {
                                        if tab_chip(ui, p, l, None, cur_filter == k).clicked() {
                                            filter = Some(k.to_string());
                                        }
                                    }
                                });
                                ui.add_space(space::S14);
                                ui.label(RichText::new("Labels").font(fs(size::CARD_HEAD)).color(p.dim));
                                ui.add_space(space::S6);
                                ui.horizontal_wrapped(|ui| {
                                    ui.spacing_mut().item_spacing = egui::vec2(space::S6, space::S6);
                                    if labels.is_empty() {
                                        ui.label(RichText::new("Add labels to tasks with @name, for example \"Call bank @phone\".").size(size::ROW_SMALL).color(p.dim));
                                    }
                                    for l in &labels {
                                        let k = format!("l:{l}");
                                        if tab_chip(ui, p, &format!("@{l}"), None, cur_filter == k).clicked() {
                                            filter = Some(k);
                                        }
                                    }
                                });
                                ui.add_space(space::S22);
                                let mut v: Vec<&Task> = d
                                    .tasks
                                    .iter()
                                    .filter(|t| match cur_filter.as_str() {
                                        "f:p1" => !t.completed && t.priority == 1,
                                        "f:p2" => !t.completed && t.priority == 2,
                                        "f:overdue" => t.is_overdue(),
                                        "f:nodate" => !t.completed && t.due.is_none() && t.parent_id.is_none(),
                                        "f:recurring" => !t.completed && t.recurrence.is_some(),
                                        "f:waiting" => !t.completed && t.waiting_since.is_some(),
                                        "f:linked" => !t.completed && t.page_id.is_some(),
                                        "f:done" => t.completed,
                                        k => k.strip_prefix("l:").map_or(false, |l| !t.completed && t.labels.iter().any(|x| x == l)),
                                    })
                                    .collect();
                                if cur_filter == "f:done" {
                                    v.sort_by_key(|t| std::cmp::Reverse(t.completed_at));
                                } else {
                                    v.sort_by_key(|t| (t.due.unwrap_or(NaiveDate::MAX), t.priority));
                                }
                                let ids: Vec<Id> = v.iter().map(|t| t.id.clone()).collect();
                                if ids.is_empty() {
                                    ui.label(RichText::new("No tasks match this filter.").size(size::ROW).color(p.dim));
                                } else {
                                    group_head(ui, p, p.faint, "Matching", &ids.len().to_string());
                                    ui.add_space(space::S8);
                                    rows_box(ui, &ids, true, true, acts);
                                }
                            }
                            _ => {
                                if (m.today.is_empty() && m.overdue.is_empty()) || self.view_state == ViewState::Empty {
                                    add |= empty(ui, "Nothing due today.", "Tasks with today's date from every project show here. Captured items wait in Captured.");
                                } else {
                                    if !m.overdue.is_empty() {
                                        ui.horizontal(|ui| {
                                            group_head(ui, p, p.orange, "Overdue", &m.overdue.len().to_string());
                                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                                if button(ui, p, ButtonKind::Ghost, "Move all to today", true, true).clicked() {
                                                    to_today = m.overdue.clone();
                                                }
                                            });
                                        });
                                        ui.add_space(space::S8);
                                        rows_box(ui, &m.overdue, true, true, acts);
                                    }
                                    for (pid, ids) in &m.today {
                                        let (dot, name) = match pid.as_ref().and_then(|x| d.project(x)) {
                                            Some(pr) => (rgb(pr.color), pr.name.clone()),
                                            None => (p.faint, "Not in a project".to_string()),
                                        };
                                        group_head(ui, p, dot, &name, &ids.len().to_string());
                                        ui.add_space(space::S8);
                                        rows_box(ui, ids, true, false, acts);
                                    }
                                }
                            }
                        },
                    }
                });
            });
        });
        if let Some(v) = go {
            self.view = v;
        }
        if let Some(f) = filter {
            self.tasks_ui.label = Some(f);
        }
        if !to_today.is_empty() {
            let n = to_today.len();
            self.undoable(&format!("Moved {n} overdue task{} to today", if n == 1 { "" } else { "s" }), "move", move |d| {
                for id in &to_today {
                    if let Some(t) = d.task_mut(id) {
                        t.due = Some(t0);
                        t.updated = now_ms();
                    }
                }
            });
        }
        if add {
            self.open_quick(QuickKind::Task);
        }
    }

    pub fn tasks_view(&mut self, ui: &mut egui::Ui, p: &Pal) {
        let view = self.view.clone();
        if let View::Project(pid) = &view {
            let mut acts: Vec<TAct> = vec![];
            self.project_tasks(ui, p, &pid.clone(), &mut acts);
            self.apply_task_acts(acts);
            return;
        }
        let mut acts: Vec<TAct> = vec![];
        self.all_projects(ui, p, &mut acts);
        self.apply_task_acts(acts);
    }

    fn apply_task_acts(&mut self, acts: Vec<TAct>) {
        for a in acts {
            match a {
                TAct::WaitDone(rid) => {
                    // a dependency that has arrived: Met, undoable from its toast
                    let title = self.store.data.records.iter().find(|r| r.id == rid).map(|r| r.title.clone()).unwrap_or_default();
                    self.undoable(&format!("Arrived: \"{title}\""), "complete", move |d| {
                        if let Some(r) = d.records.iter_mut().find(|r| r.id == rid) {
                            r.status = "Met".into();
                            r.updated = now_ms();
                        }
                    });
                }
                TAct::ShowDone => self.tasks_ui.show_completed = !self.tasks_ui.show_completed,
                TAct::BoardMove(id, col) => {
                    let Some(t) = self.store.data.task(&id).cloned() else { continue };
                    if col == "Done" {
                        if !t.completed {
                            self.toggle_task(&id);
                        }
                    } else {
                        let sec = col.clone();
                        if t.completed || t.section != sec {
                            let label = if t.completed { format!("Reopened \"{}\" in {sec}", t.title) } else { format!("Moved \"{}\" to {sec}", t.title) };
                            self.undoable(&label, "move", move |d| {
                                if let Some(x) = d.task_mut(&id) {
                                    x.completed = false;
                                    x.completed_at = None;
                                    x.section = sec;
                                    x.updated = now_ms();
                                    x.order = now_ms();
                                }
                            });
                        }
                    }
                }
                TAct::AddIn(sec) => {
                    self.open_quick(QuickKind::Task);
                    if let Some(q) = self.quick.as_mut() {
                        q.section = sec;
                    }
                }
                TAct::Organise => {
                    self.page_ui.ai_open = true;
                    self.ask_send(crate::ai::ORGANISE_PROMPT);
                }
                TAct::Toggle(id) => self.toggle_task(&id),
                TAct::Open(id) => self.open_task = Some(id),
                TAct::Move(id, sec) => {
                    // a move can be undone from its toast (rule 5)
                    let title = self.store.data.task(&id).map(|t| t.title.clone()).unwrap_or_default();
                    let label = format!("Moved \"{title}\" to {}", if sec.is_empty() { "No section" } else { sec.as_str() });
                    self.undoable(&label, "move", move |d| {
                        if let Some(t) = d.task_mut(&id) {
                            t.section = sec.clone();
                            t.updated = now_ms();
                            t.order = now_ms();
                        }
                        let subs: Vec<Id> = d.subtasks(&id).iter().map(|s| s.id.clone()).collect();
                        for s in subs {
                            if let Some(t) = d.task_mut(&s) {
                                t.section = sec.clone();
                                t.updated = now_ms();
                            }
                        }
                    });
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
                    let frame = egui::Frame::none().fill(p.surface_sunk).rounding(10.0).inner_margin(egui::Margin::same(10.0));
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
            self.quick = Some(QuickAdd { text: String::new(), kind: QuickKind::Task, project: pid.clone(), section: sec, due: None, parent: None, page: None, focus: true, opened: f64::NAN, error: false, new_project_off: false });
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
        let mut set_waiting: Option<bool> = None;
        let mut set_theme: Option<Option<Id>> = None;
        // the rest of the screen dims behind the editor; a click there closes it
        let screen = ctx.screen_rect();
        let dim = egui::Rect::from_min_max(screen.min, egui::pos2(screen.right() - space::TASK_PANEL, screen.bottom()));
        egui::Area::new(egui::Id::new("task-dim")).order(egui::Order::Middle).fixed_pos(dim.min).show(ctx, |ui| {
            if backdrop(ui, dim, shadow::BACKDROP_PANEL) {
                close = true;
            }
        });
        // the task editor (B-E9): a 440 push panel from the right; edits save as you type
        egui::SidePanel::right("task-panel").exact_width(space::TASK_PANEL).resizable(false).frame(egui::Frame::none().fill(p.card).inner_margin(egui::Margin::same(space::S18)).stroke(Stroke::new(stroke::HAIR, p.line))).show(ctx, |ui| {
            let d = &self.store.data;
            ui.horizontal(|ui| {
                if icon_button(ui, Icon::Expand, true, false).on_hover_text("Close (Esc)").clicked() {
                    close = true;
                }
                ui.add_space(space::S6);
                let project = if t.project_id.is_some() { d.project_name(&t.project_id) } else { "Captured".to_string() };
                let crumb = if t.section.is_empty() { project } else { format!("{project} · {}", t.section) };
                ui.label(RichText::new(crumb).size(size::LABEL).color(p.dim));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.menu_button(RichText::new("⋯").font(fs(size::SECTION)).color(p.dim), |ui| {
                        let waiting = t.waiting_since.is_some();
                        if ui.button(if waiting { "No longer waiting" } else { "Waiting on someone else" }).clicked() {
                            set_waiting = Some(!waiting);
                            ui.close_menu();
                        }
                        if ui.button(RichText::new("Delete task").color(p.orange)).clicked() {
                            delete = true;
                            ui.close_menu();
                        }
                    });
                });
            });
            ui.add_space(space::S8);
            // the footer first, so the fields above scroll instead of covering it
            egui::TopBottomPanel::bottom("task-panel-foot").frame(egui::Frame::none().inner_margin(egui::Margin::symmetric(0.0, space::S12))).show_separator_line(true).show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    if button(ui, p, ButtonKind::Dark, "Save", false, true).on_hover_text("Changes are already saved; this closes the editor").clicked() {
                        close = true;
                    }
                    if button(ui, p, ButtonKind::Secondary, if t.completed { "Reopen" } else { "Mark complete" }, false, true).clicked() {
                        toggle = true;
                        close = true;
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if button(ui, p, ButtonKind::Ghost, "Delete", false, true).clicked() {
                            delete = true;
                        }
                    });
                });
            });
            if let Some(par) = t.parent_id.as_deref().and_then(|x| d.task(x)) {
                if ghost_button(ui, RichText::new(format!("↖ Subtask of: {}", par.title)).size(12.0).color(p.blue)).clicked() {
                    open_sub = Some(par.id.clone());
                }
            }
            ui.add_space(6.0);
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                // the fields as drawn (Pack B "Tasks: editor"): Title; Theme and End date; Priority
                // and Labels; Notes with the mic. Everything else is under "More".
                field_visuals(ui, p);
                ui.spacing_mut().item_spacing.y = space::S6;
                let w = ui.available_width();
                field_label(ui, p, "Title");
                text_field(ui, p, egui::Id::new(("task-title", &t.id)), &mut t.title, "Task title", false, w);
                ui.add_space(space::S8);
                let themes: Vec<(Id, String)> = d.records.iter().filter(|r| r.kind == crate::resolve::records::Kind::Roadmap && r.project_id.is_some() && r.project_id == t.project_id).map(|r| (r.id.clone(), r.title.clone())).collect();
                let cur_theme = d.records.iter().find(|r| r.kind == crate::resolve::records::Kind::Roadmap && r.links.iter().any(|l| l == &t.id)).map(|r| r.id.clone());
                let half = (w - space::S12) / 2.0;
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = space::S12;
                    ui.allocate_ui_with_layout(egui::vec2(half, space::S40 + space::S24), Layout::top_down(Align::Min), |ui| {
                        ui.set_width(half);
                        field_label(ui, p, "Theme");
                        let name = cur_theme.as_ref().and_then(|id| themes.iter().find(|x| &x.0 == id)).map(|x| x.1.clone()).unwrap_or_else(|| if themes.is_empty() { "No themes".into() } else { "No theme".into() });
                        ui.add_enabled_ui(!themes.is_empty(), |ui| {
                            egui::ComboBox::from_id_salt("tp-theme").selected_text(name).width(half).wrap().show_ui(ui, |ui| {
                                if ui.selectable_label(cur_theme.is_none(), "No theme").clicked() {
                                    set_theme = Some(None);
                                }
                                for (id, title) in &themes {
                                    if ui.selectable_label(cur_theme.as_ref() == Some(id), title).clicked() {
                                        set_theme = Some(Some(id.clone()));
                                    }
                                }
                            });
                        });
                    });
                    ui.allocate_ui_with_layout(egui::vec2(half, space::S40 + space::S24), Layout::top_down(Align::Min), |ui| {
                        ui.set_width(half);
                        field_label(ui, p, "End date");
                        ui.horizontal(|ui| match t.due {
                            Some(mut date) => {
                                if ui.add(egui_extras::DatePickerButton::new(&mut date).id_salt("tp-date").format("%a %-d %b").show_icon(false).calendar_week(false)).changed() {
                                    t.due = Some(date);
                                }
                                if ghost_button(ui, RichText::new("✕").color(p.dim)).on_hover_text("Remove the date").clicked() {
                                    t.due = None;
                                    t.due_time = None;
                                }
                            }
                            None => {
                                for (l, n) in [("Today", 0), ("Tomorrow", 1), ("Next week", 7)] {
                                    if ui.button(RichText::new(l).size(size::SMALL)).clicked() {
                                        t.due = Some(today() + chrono::Duration::days(n));
                                    }
                                }
                            }
                        });
                    });
                });
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = space::S12;
                    ui.allocate_ui_with_layout(egui::vec2(half, space::S40 + space::S24), Layout::top_down(Align::Min), |ui| {
                        ui.set_width(half);
                        field_label(ui, p, "Priority");
                        egui::ComboBox::from_id_salt("tp-prio").selected_text(format!("P{}", t.priority)).width(half).show_ui(ui, |ui| {
                            for pr in 1..=4u8 {
                                ui.selectable_value(&mut t.priority, pr, format!("P{pr} · {}", priority_label(pr)));
                            }
                        });
                    });
                    ui.allocate_ui_with_layout(egui::vec2(half, space::S40 + space::S24), Layout::top_down(Align::Min), |ui| {
                        ui.set_width(half);
                        field_label(ui, p, "Labels");
                        egui::Frame::none().fill(p.card).stroke(Stroke::new(stroke::HAIR, p.line)).rounding(egui::Rounding::same(radius::FIELD)).inner_margin(egui::Margin::symmetric(space::S10, space::S6)).show(ui, |ui| {
                            ui.set_width(half - space::S10 * 2.0);
                            ui.horizontal_wrapped(|ui| {
                                ui.spacing_mut().item_spacing.x = space::S4;
                                let mut remove: Option<usize> = None;
                                for (i, l) in t.labels.iter().enumerate() {
                                    if label_chip(ui, p, l).clicked() {
                                        remove = Some(i);
                                    }
                                }
                                if let Some(i) = remove {
                                    t.labels.remove(i);
                                }
                                let key = format!("labels-{}", t.id);
                                let mut lt = self.temp.get(&key).cloned().unwrap_or_default();
                                let r = ui.add(egui::TextEdit::singleline(&mut lt).hint_text("+ label").frame(false).desired_width(space::S40 * 2.0));
                                if r.changed() {
                                    self.temp.insert(key.clone(), lt.clone());
                                }
                                if r.lost_focus() {
                                    for s in lt.split(',').map(|s| s.trim().trim_start_matches('@').to_string()).filter(|s| !s.is_empty()) {
                                        if !t.labels.contains(&s) {
                                            t.labels.push(s);
                                        }
                                    }
                                    self.temp.remove(&key);
                                }
                            });
                        });
                    });
                });
                ui.add_space(space::S4);
                field_label(ui, p, "Notes");
                let notes_id = egui::Id::new(("task-notes", &t.id));
                egui::Frame::none().fill(p.card).stroke(Stroke::new(stroke::HAIR, p.line)).rounding(egui::Rounding::same(radius::FIELD)).inner_margin(egui::Margin::symmetric(space::S12, space::S10)).show(ui, |ui| {
                    ui.set_width(w - space::S12 * 2.0);
                    ui.add(egui::TextEdit::multiline(&mut t.notes).id(notes_id).hint_text("Notes").frame(false).font(fp(size::ROW)).desired_rows(6).desired_width(f32::INFINITY));
                    ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                        mic_for(ui, p, notes_id, p.dark);
                    });
                });
                ui.add_space(space::S10);
                let subs = d.subtasks(&t.id);
                egui::CollapsingHeader::new(RichText::new("More: project, time, repeat, reminder, document, sub-tasks").size(size::SMALL).color(p.dim)).id_salt("tp-more").default_open(!subs.is_empty()).show(ui, |ui| {
                    egui::Grid::new("task-props").num_columns(2).spacing([space::S12, space::S10]).show(ui, |ui| {
                        ui.label(RichText::new("Project").color(p.dim));
                        egui::ComboBox::from_id_salt("tp-project").selected_text(if t.project_id.is_some() { d.project_name(&t.project_id) } else { "Captured".to_string() }).width(space::S40 * 5.0).show_ui(ui, |ui| {
                            ui.selectable_value(&mut t.project_id, None, "Captured");
                            for pr in &d.projects {
                                ui.selectable_value(&mut t.project_id, Some(pr.id.clone()), pr.name.clone());
                            }
                        });
                        ui.end_row();
                        let sections: Vec<String> = t.project_id.as_deref().and_then(|x| d.project(x)).map(|x| x.sections.clone()).unwrap_or_default();
                        if !sections.is_empty() {
                            ui.label(RichText::new("Section").color(p.dim));
                            egui::ComboBox::from_id_salt("tp-section").selected_text(if t.section.is_empty() { "None".to_string() } else { t.section.clone() }).width(space::S40 * 5.0).show_ui(ui, |ui| {
                                ui.selectable_value(&mut t.section, String::new(), "None");
                                for s in sections {
                                    ui.selectable_value(&mut t.section, s.clone(), s);
                                }
                            });
                            ui.end_row();
                        }
                        if t.due.is_some() {
                            ui.label(RichText::new("Time").color(p.dim));
                            ui.horizontal(|ui| {
                                time_field(ui, egui::Id::new(("tp-time", &t.id)), &mut t.due_time, space::S40 + space::S18);
                                ui.label(RichText::new("for").color(p.dim));
                                let mut dur = t.duration_min.unwrap_or(0) as i32;
                                if ui.add(egui::DragValue::new(&mut dur).range(0..=720).speed(5).suffix(" min")).changed() {
                                    t.duration_min = if dur > 0 { Some(dur as u32) } else { None };
                                }
                            });
                            ui.end_row();
                            ui.label(RichText::new("Repeat").color(p.dim));
                            egui::ComboBox::from_id_salt("tp-rec").selected_text(t.recurrence.label()).width(space::S40 * 5.0).show_ui(ui, |ui| {
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
                                egui::ComboBox::from_id_salt("tp-rem").selected_text(if t.due_time.is_some() { rl(t.reminder_min) } else { "Set a time first".into() }).width(space::S40 * 5.0).show_ui(ui, |ui| {
                                    for m in [None, Some(0), Some(10), Some(30), Some(60), Some(120), Some(1440)] {
                                        ui.selectable_value(&mut t.reminder_min, m, rl(m));
                                    }
                                });
                            });
                            ui.end_row();
                        }
                        ui.label(RichText::new("Document").color(p.dim));
                        ui.horizontal(|ui| {
                            let current = t.page_id.as_deref().and_then(|x| d.page(x)).map(|x| if x.title.is_empty() { "Untitled".to_string() } else { x.title.clone() }).unwrap_or_else(|| "None".into());
                            egui::ComboBox::from_id_salt("tp-page").selected_text(current).width(space::S40 * 4.0).show_ui(ui, |ui| {
                                ui.selectable_value(&mut t.page_id, None, "None");
                                for pg in &d.pages {
                                    ui.selectable_value(&mut t.page_id, Some(pg.id.clone()), if pg.title.is_empty() { "Untitled".to_string() } else { pg.title.clone() });
                                }
                            });
                            if let Some(pg) = &t.page_id {
                                if button(ui, p, ButtonKind::Ghost, "Open", true, true).clicked() {
                                    open_page = Some(pg.clone());
                                }
                            } else if button(ui, p, ButtonKind::Ghost, "+ New doc", true, true).on_hover_text("Create a page linked to this task").clicked() {
                                new_doc = true;
                            }
                        });
                        ui.end_row();
                    });
                    ui.add_space(space::S10);
                    ui.label(RichText::new(format!("Sub-tasks {}/{}", subs.iter().filter(|s| s.completed).count(), subs.len())).font(fs(size::SMALL)).color(p.dim));
                    for s in subs {
                        ui.horizontal(|ui| {
                            let (r, resp) = ui.allocate_exact_size(egui::vec2(space::S18 + 2.0, space::S18 + 2.0), Sense::click());
                            paint_circle(ui, r.center(), p, s.priority, s.completed, resp.hovered());
                            if resp.clicked() {
                                toggle_sub = Some(s.id.clone());
                            }
                            let mut rt = RichText::new(&s.title).size(size::ROW_SMALL);
                            if s.completed {
                                rt = rt.strikethrough().color(p.dim);
                            }
                            if ui.add(egui::Label::new(rt).sense(Sense::click())).clicked() {
                                open_sub = Some(s.id.clone());
                            }
                        });
                    }
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("+").color(p.orange));
                        let r = ui.add(egui::TextEdit::singleline(&mut self.tasks_ui.sub_text).hint_text("Add sub-task (Enter)").frame(false).desired_width(f32::INFINITY));
                        if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) && !self.tasks_ui.sub_text.trim().is_empty() {
                            add_sub = Some(std::mem::take(&mut self.tasks_ui.sub_text));
                            r.request_focus();
                        }
                    });
                    ui.add_space(space::S10);
                    if t.due.is_some() {
                        ui.label(RichText::new("Shows on your calendar on its due date.").size(size::SMALL).color(p.dim));
                    }
                    let created = chrono::DateTime::from_timestamp_millis(t.created).map(|x| x.with_timezone(&chrono::Local).format("%-d %b %Y %H:%M").to_string()).unwrap_or_default();
                    ui.label(RichText::new(format!("Created {created}")).size(size::TINY).color(p.faint));
                });
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
            if moved_project {
                // a move can be undone from its toast (rule 5)
                let where_to = match proj.as_deref().and_then(|p| self.store.data.project(p)) {
                    Some(pr) if !sec.is_empty() => format!("{} · {sec}", pr.name),
                    Some(pr) => pr.name.clone(),
                    None => "Captured".to_string(),
                };
                let label = format!("Moved \"{}\" to {where_to}", t.title);
                let (tid2, t2) = (tid.to_string(), t.clone());
                self.undoable(&label, "move", move |d| {
                    if let Some(x) = d.task_mut(&tid2) {
                        *x = t2;
                    }
                    let subs: Vec<Id> = d.subtasks(&tid2).iter().map(|s| s.id.clone()).collect();
                    for s in subs {
                        if let Some(x) = d.task_mut(&s) {
                            x.project_id = proj.clone();
                            x.section = sec.clone();
                            x.updated = now_ms();
                        }
                    }
                });
            } else {
                *self.store.data.task_mut(tid).unwrap() = t.clone();
                self.touch();
            }
        }
        if toggle {
            self.toggle_task(tid);
        }
        if let Some(theme) = set_theme {
            let (id, title) = (tid.to_string(), t.title.clone());
            let name = theme.as_ref().and_then(|x| self.store.data.records.iter().find(|r| &r.id == x)).map(|r| r.title.clone()).unwrap_or_else(|| "no theme".into());
            self.undoable(&format!("Moved \"{title}\" to {name}"), "move", move |d| {
                for r in d.records.iter_mut().filter(|r| r.kind == crate::resolve::records::Kind::Roadmap) {
                    if r.links.iter().any(|l| l == &id) && Some(&r.id) != theme.as_ref() {
                        r.links.retain(|l| l != &id);
                        r.updated = now_ms();
                    }
                    if Some(&r.id) == theme.as_ref() && !r.links.contains(&id) {
                        r.links.push(id.clone());
                        r.updated = now_ms();
                    }
                }
            });
        }
        if let Some(on) = set_waiting {
            let (id, title) = (tid.to_string(), t.title.clone());
            let label = if on { format!("Waiting on \"{title}\"") } else { format!("No longer waiting on \"{title}\"") };
            self.undoable(&label, "edit", move |d| {
                if let Some(x) = d.task_mut(&id) {
                    x.waiting_since = if on { Some(now_ms()) } else { None };
                    x.updated = now_ms();
                }
            });
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
            // (the Undo toast from `undoable` names it; a second toast would replace it)
            let _ = title;
            self.open_task = None;
            self.touch();
        }
        if close {
            self.open_task = None;
        }
    }
}
