pub mod theme;
pub mod widgets;
mod calendar;
mod cockpit;
mod intake;
mod intake_review;
mod intake_steps;
pub mod resolve;
pub mod testlab;
mod dashboard;
mod help;
mod assistant;
mod databases;
mod pages;
pub(crate) mod settings;
mod tasks;

use crate::model::*;
use crate::store::{Registry, Store};
use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime, Timelike};
use eframe::egui::{self, Align, Align2, Color32, Key, Layout, Modifiers, RichText, Sense, Stroke};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use theme::*;
use widgets::*;

#[derive(Clone, PartialEq, Debug)]
pub enum View {
    Dashboard,
    Inbox,
    Today,
    Upcoming,
    Filters,
    Project(Id),
    Calendar,
    Page(Id),
    Database(Id),
    Settings,
    Help,
    /// A Resolve intake session.
    Intake(Id),
    /// A project's Resolve cockpit (ResolveOS runtime surface).
    Resolve(Id),
    /// Test Intake home (only in the sandbox process).
    TestLab,
}

#[derive(Clone, Copy, PartialEq)]
pub enum QuickKind {
    Task,
    Event,
    Page,
}

pub struct QuickAdd {
    pub text: String,
    pub kind: QuickKind,
    pub project: Option<Id>,
    pub section: String,
    pub due: Option<NaiveDate>,
    pub parent: Option<Id>,
    pub page: Option<Id>,
    pub focus: bool,
}

pub struct Palette {
    pub query: String,
    pub sel: usize,
    pub focus: bool,
}

#[derive(Clone)]
pub struct EventDraft {
    pub id: Option<Id>,
    pub title: String,
    pub calendar_id: Id,
    pub date: NaiveDate,
    pub end_date: NaiveDate,
    pub start: Option<NaiveTime>,
    pub end: Option<NaiveTime>,
    pub all_day: bool,
    pub location: String,
    pub notes: String,
    pub recurrence: Recurrence,
    pub page_id: Option<Id>,
    pub reminder_min: Option<i64>,
    pub focus: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub enum CalMode {
    Day,
    Week,
    Month,
}

pub struct CalState {
    pub mode: CalMode,
    pub anchor: NaiveDate,
    pub mini: NaiveDate,
    pub show_tasks: bool,
    pub drag: Option<(Id, f32, f32)>,
    pub new_cal: Option<String>,
}

#[derive(Default)]
pub struct TasksUi {
    pub adding: Option<String>,
    pub add_text: String,
    pub show_completed: bool,
    pub board: HashSet<String>,
    pub new_section: Option<String>,
    pub sub_text: String,
    pub label: Option<String>,
    pub rename_project: bool,
}

#[derive(Default)]
pub struct PageUi {
    pub focus: Option<(Id, usize, usize)>,
    pub emoji_open: bool,
    pub ai_open: bool,
    pub ai_can_act: bool,
    pub ai_prompt: String,
    pub ai_out: String,
    pub ai_job: Option<crate::ai::AiJob>,
    pub ai_for_page: Option<Id>,
    /// When the current assistant request started (to find the changes it made, for Undo).
    pub ai_started_ms: i64,
    pub menu_block: Option<usize>,
}

#[derive(Default)]
pub struct DbUi {
    pub filter: String,
    pub highlight: Option<Id>,
    pub new_option: String,
}

#[derive(Default)]
pub struct SettingsUi {
    pub tab: usize,
    pub message: String,
    pub models_rx: Option<std::sync::mpsc::Receiver<Result<Vec<String>, String>>>,
    pub models: Vec<String>,
    pub confirm_restore: Option<PathBuf>,
    pub confirm_remove_ws: bool,
}

pub struct Shot {
    pub path: PathBuf,
    pub requested: bool,
}

pub struct StartOpts {
    pub screenshot: Option<PathBuf>,
    pub workspace: Option<String>,
    pub view: Option<String>,
    /// Test Intake mode: sandbox data folder, no sidebar, no MCP/phone server.
    pub testlab: bool,
}

pub struct App {
    pub reg: Registry,
    pub store: Store,
    pub view: View,
    pub back: Vec<View>,
    pub dirty: bool,
    pub last_edit: f64,
    pub toasts: Vec<(String, f64)>,
    pub open_task: Option<Id>,
    pub event_edit: Option<EventDraft>,
    pub quick: Option<QuickAdd>,
    pub palette: Option<Palette>,
    pub cal: CalState,
    pub tasks_ui: TasksUi,
    pub page_ui: PageUi,
    pub db_ui: DbUi,
    pub settings_ui: SettingsUi,
    pub server: Option<crate::server::ServerHandle>,
    pub server_error: Option<String>,
    pub shot: Option<Shot>,
    pub frame: u64,
    pub fired: HashSet<String>,
    pub last_poll: f64,
    pub last_sync: f64,
    pub last_remind: f64,
    pub theme_applied: Option<bool>,
    pub expanded: HashSet<Id>,
    pub new_ws: Option<String>,
    pub now: f64,
    pub sync_status: String,
    pub temp: HashMap<String, String>,
    pub sub_tx: std::sync::mpsc::Sender<(Id, Result<String, String>)>,
    pub sub_rx: std::sync::mpsc::Receiver<(Id, Result<String, String>)>,
    pub last_sub_check: f64,
    pub hub_rx: Option<std::sync::mpsc::Receiver<Result<WorkspaceData, String>>>,
    pub last_hub: f64,
    pub hub_status: String,
    pub resolve_ui: resolve::ResolveUi,
    /// (message, expires, change-set id): toasts with an Undo button.
    pub undo_toasts: Vec<(String, f64, Id)>,
    /// Running as Test Intake (sandbox data folder).
    pub testlab: bool,
    pub lab: testlab::LabUi,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, opts: StartOpts) -> Self {
        install_fonts(&cc.egui_ctx);
        let reg = Registry::load();
        let meta = opts
            .workspace
            .as_deref()
            .and_then(|w| reg.find(w).cloned())
            .or_else(|| reg.settings.last_workspace.as_deref().and_then(|w| reg.find(w).cloned()))
            .unwrap_or_else(|| reg.workspaces[0].clone());
        let mut store = Store::open(meta);
        store.auto_backup_if_due();
        let t = today();
        let open_ai = opts.view.as_deref() == Some("ai");
        let view = match opts.view.as_deref() {
            Some("inbox") => View::Inbox,
            Some("today") => View::Today,
            Some("upcoming") => View::Upcoming,
            Some("calendar") => View::Calendar,
            Some("settings") => View::Settings,
            Some("help") => View::Help,
            Some("resolve") => store.data.contexts.iter().max_by_key(|c| c.updated).map(|c| View::Resolve(c.project_id.clone())).unwrap_or(View::Dashboard),
            Some("intake") => store.data.intakes.iter().max_by_key(|i| i.updated).map(|i| View::Intake(i.id.clone())).unwrap_or(View::Dashboard),
            Some("page") => store.data.pages.iter().find(|p| p.title == "Weekly Review").or(store.data.pages.first()).map(|p| View::Page(p.id.clone())).unwrap_or(View::Dashboard),
            Some("database") => store.data.databases.first().map(|d| View::Database(d.id.clone())).unwrap_or(View::Dashboard),
            _ => View::Dashboard,
        };
        let (sub_tx, sub_rx) = std::sync::mpsc::channel();
        let mut app = App {
            sub_tx,
            sub_rx,
            last_sub_check: -10000.0,
            hub_rx: None,
            last_hub: -1000.0,
            hub_status: String::new(),
            reg,
            store,
            view,
            back: vec![],
            dirty: false,
            last_edit: 0.0,
            toasts: vec![],
            open_task: None,
            event_edit: None,
            quick: None,
            palette: None,
            cal: CalState { mode: CalMode::Week, anchor: t, mini: t, show_tasks: true, drag: None, new_cal: None },
            tasks_ui: TasksUi::default(),
            page_ui: PageUi { ai_can_act: true, ..Default::default() },
            db_ui: DbUi::default(),
            settings_ui: SettingsUi::default(),
            server: None,
            server_error: None,
            shot: opts.screenshot.map(|path| Shot { path, requested: false }),
            frame: 0,
            fired: HashSet::new(),
            last_poll: 0.0,
            last_sync: -1000.0,
            last_remind: 0.0,
            theme_applied: None,
            expanded: HashSet::new(),
            new_ws: None,
            now: 0.0,
            sync_status: String::new(),
            temp: HashMap::new(),
            resolve_ui: resolve::ResolveUi::default(),
            undo_toasts: vec![],
            testlab: opts.testlab,
            lab: testlab::LabUi::default(),
        };
        app.page_ui.ai_open = open_ai;
        if app.testlab {
            // --view intake-<stage> opens the newest run at that stage (screenshots / checks)
            app.view = match opts.view.as_deref() {
                Some(v) if v.starts_with("intake") => {
                    let stage = v.strip_prefix("intake-").unwrap_or("");
                    app.store.data.intakes.iter().filter(|i| stage.is_empty() || i.stage == stage).max_by_key(|i| i.updated).map(|i| View::Intake(i.id.clone())).unwrap_or(View::TestLab)
                }
                _ => View::TestLab,
            };
        } else {
            app.ensure_server();
            if opts.view.is_none() {
                app.open_intake_if_new();
            }
        }
        app
    }

    pub fn pal(&self) -> Pal {
        pal(self.reg.settings.dark)
    }

    pub fn touch(&mut self) {
        self.dirty = true;
        self.last_edit = self.now;
    }

    pub fn toast(&mut self, s: impl Into<String>) {
        self.toasts.push((s.into(), self.now + 4.0));
    }

    pub fn go(&mut self, v: View) {
        if self.view != v {
            self.back.push(self.view.clone());
            if self.back.len() > 50 {
                self.back.remove(0);
            }
            self.view = v;
            self.tasks_ui.adding = None;
            self.page_ui.focus = None;
            self.tasks_ui.rename_project = false;
        }
    }

    pub fn save_now(&mut self) {
        match self.store.save() {
            Ok(()) => self.dirty = false,
            Err(e) => {
                self.dirty = false;
                self.toast(format!("Couldn't save: {e}"));
            }
        }
    }

    pub fn switch_workspace(&mut self, id: &str) {
        if self.store.meta.id == id {
            return;
        }
        if self.dirty {
            self.save_now();
        }
        let Some(meta) = self.reg.find(id).cloned() else { return };
        if let Some(s) = self.server.take() {
            s.stop();
        }
        self.store = Store::open(meta);
        self.store.auto_backup_if_due();
        self.reg.settings.last_workspace = Some(id.to_string());
        self.reg.save();
        self.view = View::Dashboard;
        self.back.clear();
        self.open_intake_if_new();
        self.open_task = None;
        self.event_edit = None;
        self.page_ui = PageUi { ai_can_act: true, ..Default::default() };
        self.last_sync = -1000.0;
        self.ensure_server();
        let name = self.store.meta.name.clone();
        self.toast(format!("Switched to {name}"));
    }

    /// Sync with the always-on hub every minute (or now, when `force`), off the UI thread.
    pub fn hub_tick(&mut self, force: bool) {
        if let Some(rx) = &self.hub_rx {
            match rx.try_recv() {
                Ok(Ok(merged)) => {
                    self.store.data.merge(&merged);
                    self.store.settings.last_hub_sync = now_ms();
                    self.store.save_settings();
                    self.touch();
                    self.hub_status = format!("hub synced {}", chrono::Local::now().format("%H:%M"));
                    self.hub_rx = None;
                }
                Ok(Err(e)) => {
                    self.hub_status = format!("hub: {e}");
                    self.hub_rx = None;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
                Err(_) => self.hub_rx = None,
            }
        }
        let url = self.store.settings.hub_url.trim().to_string();
        if url.is_empty() || self.hub_rx.is_some() || self.shot.is_some() || (!force && self.now - self.last_hub < 60.0) {
            return;
        }
        self.last_hub = self.now;
        let token = self.store.settings.hub_token.clone();
        let data = self.store.data.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(crate::store::hub_exchange(&url, &token, &data));
        });
        self.hub_rx = Some(rx);
        self.hub_status = "hub: syncing…".into();
    }

    pub fn ensure_server(&mut self) {
        let want = self.store.settings.server_enabled;
        let port = self.store.settings.server_port;
        let matches = self.server.as_ref().map_or(false, |s| s.port == port && s.workspace_id == self.store.meta.id);
        if want && !matches {
            if let Some(s) = self.server.take() {
                s.stop();
            }
            match crate::server::start(self.store.meta.clone(), port) {
                Ok(h) => {
                    self.server = Some(h);
                    self.server_error = None;
                }
                Err(e) => self.server_error = Some(e),
            }
        } else if !want {
            if let Some(s) = self.server.take() {
                s.stop();
            }
        }
    }

    pub fn open_quick(&mut self, kind: QuickKind) {
        let (project, due) = match &self.view {
            View::Project(p) => (Some(p.clone()), None),
            View::Today => (None, Some(today())),
            View::Calendar => (None, Some(self.cal.anchor)),
            _ => (None, None),
        };
        self.quick = Some(QuickAdd { text: String::new(), kind, project, section: String::new(), due, parent: None, page: None, focus: true });
    }

    /// Create a task from quick-capture text. Returns the new id.
    pub fn add_task_text(&mut self, text: &str, project: Option<Id>, section: &str, default_due: Option<NaiveDate>, parent: Option<Id>, page: Option<Id>) -> Option<Id> {
        let c = crate::nlp::parse(text, today());
        if c.title.trim().is_empty() {
            return None;
        }
        let mut t = Task::new(c.title.clone());
        t.priority = c.priority.unwrap_or(4);
        t.due = c.due.or(default_due);
        t.due_time = c.time;
        t.recurrence = c.recurrence;
        t.labels = c.labels.clone();
        t.reminder_min = c.reminder_min.or(if c.time.is_some() { Some(0) } else { None });
        t.duration_min = c.duration_min;
        t.section = c.section.clone().unwrap_or_else(|| section.to_string());
        t.project_id = project;
        t.page_id = page;
        if let Some(pname) = &c.project {
            t.project_id = match self.store.data.project_by_name(pname) {
                Some(p) => Some(p.id.clone()),
                None if pname.eq_ignore_ascii_case("inbox") => None,
                None => {
                    let palette: [[u8; 3]; 5] = [[219, 76, 63], [41, 171, 135], [20, 170, 245], [175, 56, 235], [255, 153, 51]];
                    let pr = Project { id: new_id(), name: pname.clone(), color: palette[self.store.data.projects.len() % 5], sections: vec![], page_id: None, archived: false, favorite: false, order: self.store.data.projects.len() as i64 + 1, updated: now_ms() };
                    let id = pr.id.clone();
                    self.store.data.projects.push(pr);
                    Some(id)
                }
            };
        }
        if let Some(pid) = &parent {
            if let Some(par) = self.store.data.task(pid) {
                t.project_id = par.project_id.clone();
                t.section = par.section.clone();
            }
            t.parent_id = parent.clone();
        }
        let id = t.id.clone();
        self.store.data.tasks.push(t);
        self.touch();
        Some(id)
    }

    pub fn toggle_task(&mut self, id: &str) {
        let Some(t) = self.store.data.task(id).cloned() else { return };
        let id = id.to_string();
        if t.completed {
            self.store.data.record_change(&format!("Reopened \"{}\"", t.title), "edit", |d| {
                if let Some(t) = d.task_mut(&id) {
                    t.completed = false;
                    t.completed_at = None;
                    t.updated = now_ms();
                }
            });
            self.touch();
            return;
        }
        let mut rolled = false;
        let mut next = None;
        let cs = self.store.data.record_change(&format!("Completed \"{}\"", t.title), "complete", |d| {
            if let Some(t) = d.task_mut(&id) {
                rolled = t.complete();
                next = t.due;
            }
        });
        self.touch();
        let msg = if rolled { format!("↻ \"{}\" moved to {}", t.title, next.map(fmt_date_short).unwrap_or_default()) } else { format!("✔ Completed \"{}\"", t.title) };
        match cs {
            Some(cs) => self.undo_toasts.push((msg, self.now + 6.0, cs)),
            None => self.toast(msg),
        }
    }

    /// Run a change that can be undone (deletes are soft: restorable from Settings → Recently deleted),
    /// and show a toast with Undo.
    pub fn undoable(&mut self, label: &str, kind: &str, f: impl FnOnce(&mut WorkspaceData)) {
        let cs = self.store.data.record_change(label, kind, f);
        self.touch();
        if let Some(cs) = cs {
            self.undo_toasts.push((label.to_string(), self.now + 6.0, cs));
        }
    }

    /// Undo one change set, or several (comma-separated ids, undone newest first).
    pub fn undo(&mut self, cs: &str) {
        let (mut n, mut skipped, mut err) = (0, 0, None);
        for id in cs.split(',').rev() {
            match self.store.data.undo_change(id) {
                Ok((a, b)) => {
                    n += a;
                    skipped += b;
                }
                Err(e) => err = Some(e),
            }
        }
        self.touch();
        match err {
            Some(e) if n == 0 => self.toast(e),
            _ => self.toast(if skipped > 0 { format!("Undone ({n} restored; {skipped} edited since and left as they are)") } else { "Undone".to_string() }),
        }
        self.undo_toasts.retain(|t| t.2 != cs);
    }

    pub fn new_page(&mut self, parent: Option<Id>) -> Id {
        let mut p = Page::new("");
        p.parent = parent.clone();
        let id = p.id.clone();
        self.store.data.pages.push(p);
        if let Some(par) = parent {
            self.expanded.insert(par);
        }
        self.touch();
        self.go(View::Page(id.clone()));
        self.page_ui.focus = None;
        self.temp.insert("focus_title".into(), id.clone());
        id
    }

    pub fn page_from_builtin(&mut self, i: usize) {
        let t = &crate::templates::PAGE_TEMPLATES[i];
        let (_, blocks) = markdown_to_blocks(t.markdown);
        let id = self.new_page(None);
        if let Some(pg) = self.store.data.page_mut(&id) {
            pg.icon = t.icon.to_string();
            pg.blocks = blocks;
            if t.name == "Meeting notes" || t.name == "Daily journal" {
                pg.title = format!("{} — {}", t.name, today().format("%-d %b %Y"));
            }
        }
    }

    pub fn page_from_template(&mut self, tid: &str) {
        let Some(tpl) = self.store.data.page(tid).cloned() else { return };
        let id = self.new_page(None);
        if let Some(pg) = self.store.data.page_mut(&id) {
            pg.icon = tpl.icon.clone();
            pg.title = tpl.title.trim_end_matches(" (template)").to_string();
            pg.blocks = tpl.blocks.iter().map(|b| Block { id: new_id(), ..b.clone() }).collect();
        }
        self.temp.remove("focus_title");
    }

    pub fn new_database(&mut self) -> Id {
        let name = Column { id: new_id(), name: "Name".into(), kind: ColKind::Text, options: vec![], width: 220.0, target: String::new() };
        let status = Column { id: new_id(), name: "Status".into(), kind: ColKind::Select, options: vec!["Not started".into(), "In progress".into(), "Done".into()], width: 140.0, target: String::new() };
        let date = Column { id: new_id(), name: "Date".into(), kind: ColKind::Date, options: vec![], width: 130.0, target: String::new() };
        let gb = Some(status.id.clone());
        let db = Database { id: new_id(), name: "Untitled database".into(), icon: "▦".into(), columns: vec![name, status, date], rows: vec![], view: DbView::Table, group_by: gb, sort_by: None, filters: vec![], template: false, updated: now_ms() };
        let id = db.id.clone();
        self.store.data.databases.push(db);
        self.touch();
        self.go(View::Database(id.clone()));
        id
    }

    fn check_reminders(&mut self, ctx: &egui::Context) {
        if self.now - self.last_remind < 1.0 {
            return;
        }
        self.last_remind = self.now;
        let now = now_local();
        let mut alerts = vec![];
        for t in self.store.data.tasks.iter_mut() {
            if t.completed {
                continue;
            }
            let (Some(d), Some(tm), Some(m)) = (t.due, t.due_time, t.reminder_min) else { continue };
            let due = d.and_time(tm);
            let fire = due - Duration::minutes(m);
            let key = format!("{d}{tm}");
            if t.reminded_for != key && now >= fire && now - fire < Duration::hours(6) {
                t.reminded_for = key;
                t.updated = now_ms();
                alerts.push(format!("⏰ {} — due {}", t.title, fmt_time(tm)));
            }
        }
        for e in &self.store.data.events {
            let Some(m) = e.reminder_min else { continue };
            if e.all_day {
                continue;
            }
            let Some(start) = e.occurrence_on(now.date()) else { continue };
            let fire = start - Duration::minutes(m);
            let key = format!("{}{}", e.id, start);
            if !self.fired.contains(&key) && now >= fire && now < start + Duration::minutes(5) {
                self.fired.insert(key);
                alerts.push(format!("📅 {} at {}", e.title, fmt_time(start.time())));
            }
        }
        if !alerts.is_empty() {
            for a in alerts {
                self.toasts.push((a, self.now + 12.0));
            }
            self.touch();
            ctx.send_viewport_cmd(egui::ViewportCommand::RequestUserAttention(egui::UserAttentionType::Informational));
        }
    }

    fn shortcuts(&mut self, ctx: &egui::Context) {
        let (k, q, n, esc, e) = ctx.input_mut(|i| {
            (
                i.consume_key(Modifiers::COMMAND, Key::K),
                i.consume_key(Modifiers::COMMAND, Key::Q),
                i.consume_key(Modifiers::COMMAND, Key::N),
                i.key_pressed(Key::Escape),
                i.consume_key(Modifiers::COMMAND, Key::E),
            )
        });
        if k {
            self.palette = Some(Palette { query: String::new(), sel: 0, focus: true });
            self.quick = None;
        }
        if q {
            self.open_quick(QuickKind::Task);
            self.palette = None;
        }
        if e {
            self.open_quick(QuickKind::Event);
            self.palette = None;
        }
        if n {
            self.new_page(None);
        }
        if esc {
            if self.palette.is_some() {
                self.palette = None;
            } else if self.quick.is_some() {
                self.quick = None;
            } else if self.event_edit.is_some() {
                self.event_edit = None;
            } else if self.open_task.is_some() && ctx.memory(|m| m.focused().is_none()) {
                self.open_task = None;
            }
        }
    }

    fn sidebar(&mut self, ctx: &egui::Context, p: &Pal) {
        let d = &self.store.data;
        let inbox_n = d.tasks.iter().filter(|t| !t.completed && t.project_id.is_none() && t.parent_id.is_none()).count();
        let today_n = d.tasks.iter().filter(|t| !t.completed && t.due.map_or(false, |x| x <= today())).count();
        egui::SidePanel::left("sidebar")
            .resizable(true)
            .default_width(268.0)
            .width_range(220.0..=380.0)
            .frame(egui::Frame::none().fill(p.side).inner_margin(egui::Margin { left: 10.0, right: 10.0, top: 12.0, bottom: 8.0 }))
            .show(ctx, |ui| {
                // workspace switcher
                ui.horizontal(|ui| {
                    let meta = self.store.meta.clone();
                    avatar(ui, &meta.name.chars().next().unwrap_or('W').to_string(), rgb(meta.color), 26.0);
                    let mut switch_to: Option<Id> = None;
                    ui.menu_button(RichText::new(format!("{} ⏷", meta.name)).font(fs(14.0)), |ui| {
                        ui.set_min_width(220.0);
                        ui.label(RichText::new("Workspaces — each is private").size(12.0).color(p.dim));
                        for w in self.reg.workspaces.clone() {
                            let sel = w.id == meta.id;
                            if ui.selectable_label(sel, format!("●  {}", w.name)).clicked() {
                                switch_to = Some(w.id.clone());
                                ui.close_menu();
                            }
                        }
                        ui.separator();
                        if ui.button("+ New workspace…").clicked() {
                            self.new_ws = Some(String::new());
                            ui.close_menu();
                        }
                        if ui.button("⚙ Settings").clicked() {
                            self.go(View::Settings);
                            ui.close_menu();
                        }
                    });
                    if let Some(id) = switch_to {
                        self.switch_workspace(&id);
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let icon = if self.reg.settings.dark { "☀" } else { "🌙" };
                        if ghost_button(ui, RichText::new(icon).size(16.0).color(p.dim)).on_hover_text("Switch light / dark").clicked() {
                            self.reg.settings.dark = !self.reg.settings.dark;
                            self.reg.save();
                        }
                        if ghost_button(ui, RichText::new("⏰").size(15.0).color(p.dim)).on_hover_text("Reminders and upcoming").clicked() {
                            self.go(View::Upcoming);
                        }
                    });
                });
                ui.add_space(12.0);
                // add task
                let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 34.0), Sense::click());
                if resp.hovered() {
                    ui.painter().rect_filled(rect, 6.0, p.hover);
                }
                ui.painter().circle_filled(rect.left_center() + egui::vec2(18.0, 0.0), 10.0, p.accent);
                ui.painter().text(rect.left_center() + egui::vec2(18.0, -1.0), Align2::CENTER_CENTER, "+", fb(16.0), p.on_accent);
                ui.painter().text(rect.left_center() + egui::vec2(36.0, 0.0), Align2::LEFT_CENTER, "Add task", fs(14.5), p.accent);
                ui.painter().text(rect.right_center() - egui::vec2(10.0, 0.0), Align2::RIGHT_CENTER, "Ctrl+Q", fp(11.0), p.faint);
                if resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                    self.open_quick(QuickKind::Task);
                }
                ui.add_space(4.0);
                if nav_item(ui, p, "✨", "Ask AI", None, self.page_ui.ai_open, None).on_hover_text("Claude, ChatGPT or a local model — can read and change your workspace").clicked() {
                    self.page_ui.ai_open = !self.page_ui.ai_open;
                }
                if nav_item(ui, p, "🔍", "Search", None, false, None).on_hover_text("Ctrl+K").clicked() {
                    self.palette = Some(Palette { query: String::new(), sel: 0, focus: true });
                }
                let v = self.view.clone();
                if nav_item(ui, p, "🏠", "Home", None, v == View::Dashboard, None).clicked() {
                    self.go(View::Dashboard);
                }
                if nav_item(ui, p, "📥", "Inbox", Some(inbox_n), v == View::Inbox, None).clicked() {
                    self.go(View::Inbox);
                }
                if nav_item(ui, p, "📅", "Today", Some(today_n), v == View::Today, None).clicked() {
                    self.go(View::Today);
                }
                if nav_item(ui, p, "🗓", "Upcoming", None, v == View::Upcoming, None).clicked() {
                    self.go(View::Upcoming);
                }
                if nav_item(ui, p, "🏷", "Filters & Labels", None, v == View::Filters, None).clicked() {
                    self.go(View::Filters);
                }
                if nav_item(ui, p, "📆", "Calendar", None, v == View::Calendar, None).clicked() {
                    self.go(View::Calendar);
                }
                egui::ScrollArea::vertical().auto_shrink([false, false]).max_height(ui.available_height() - 104.0).show(ui, |ui| {
                    // Resolve: guided intake and unfinished sessions
                    ui.horizontal(|ui| {
                        section_label(ui, p, "Resolve");
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ghost_button(ui, RichText::new("+").size(17.0).color(p.dim)).on_hover_text("Bring in an idea, problem or messy project").clicked() {
                                self.start_intake();
                            }
                        });
                    });
                    let intakes = resolve::open_intakes(&self.store.data);
                    if intakes.is_empty() {
                        if nav_item(ui, p, "✦", "Start with Resolve", None, false, Some(p.accent)).on_hover_text("Turn an idea, problem or messy project into a clear first move").clicked() {
                            self.start_intake();
                        }
                    }
                    for (id, title, step) in intakes.into_iter().take(4) {
                        let label = format!("{} · step {}/4", title, (step as usize + 1).min(4));
                        if nav_item(ui, p, "◌", &label, None, v == View::Intake(id.clone()), Some(p.accent)).on_hover_text("Unfinished — pick up where you left off").clicked() {
                            self.go(View::Intake(id));
                        }
                    }
                    // projects
                    ui.horizontal(|ui| {
                        section_label(ui, p, "My Projects");
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ghost_button(ui, RichText::new("+").size(17.0).color(p.dim)).on_hover_text("New project — opens the intake").clicked() {
                                self.start_intake();
                            }
                        });
                    });
                    let mut projects: Vec<Project> = self.store.data.projects.iter().filter(|x| !x.archived).cloned().collect();
                    projects.sort_by_key(|x| x.order);
                    for pr in projects {
                        let n = self.store.data.tasks.iter().filter(|t| !t.completed && t.project_id.as_deref() == Some(&pr.id) && t.parent_id.is_none()).count();
                        let resolved = self.store.data.context_for(&pr.id).is_some();
                        let sel = v == View::Project(pr.id.clone()) || v == View::Resolve(pr.id.clone());
                        if nav_item(ui, p, if resolved { "✦" } else { "#" }, &pr.name, Some(n), sel, Some(rgb(pr.color))).clicked() {
                            if resolved {
                                self.go(View::Resolve(pr.id.clone()));
                            } else {
                                self.go(View::Project(pr.id.clone()));
                            }
                        }
                    }
                    // pages
                    let favs: Vec<(Id, String, String)> = self.store.data.pages.iter().filter(|x| x.favorite && !x.template).map(|x| (x.id.clone(), x.icon.clone(), x.title.clone())).collect();
                    if !favs.is_empty() {
                        section_label(ui, p, "Favorites");
                        for (id, icon, title) in favs {
                            let t = if title.is_empty() { "Untitled".to_string() } else { title };
                            if nav_item(ui, p, if icon.is_empty() { "📄" } else { &icon }, &t, None, v == View::Page(id.clone()), Some(p.text)).clicked() {
                                self.go(View::Page(id));
                            }
                        }
                    }
                    ui.horizontal(|ui| {
                        section_label(ui, p, "Pages");
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.menu_button(RichText::new("+").size(17.0).color(p.dim), |ui| {
                                if ui.button("📄 Blank page   (Ctrl+N)").clicked() {
                                    self.new_page(None);
                                    ui.close_menu();
                                }
                                ui.separator();
                                ui.label(RichText::new("From a template").size(11.5).color(p.dim));
                                for (i, t) in crate::templates::PAGE_TEMPLATES.iter().enumerate() {
                                    if ui.button(format!("{}  {}", t.icon, t.name)).clicked() {
                                        self.page_from_builtin(i);
                                        ui.close_menu();
                                    }
                                }
                                let mine: Vec<(Id, String, String)> = self.store.data.pages.iter().filter(|x| x.template).map(|x| (x.id.clone(), x.icon.clone(), x.title.clone())).collect();
                                if !mine.is_empty() {
                                    ui.separator();
                                    ui.label(RichText::new("Your templates").size(11.5).color(p.dim));
                                    for (id, icon, title) in mine {
                                        ui.horizontal(|ui| {
                                            if ui.button(format!("{icon}  {title}")).clicked() {
                                                self.page_from_template(&id);
                                                ui.close_menu();
                                            }
                                            if ui.small_button("edit").clicked() {
                                                self.go(View::Page(id.clone()));
                                                ui.close_menu();
                                            }
                                        });
                                    }
                                }
                            });
                        });
                    });
                    self.page_tree(ui, p, None, 0);
                    // databases
                    ui.horizontal(|ui| {
                        section_label(ui, p, "Databases");
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.menu_button(RichText::new("+").size(17.0).color(p.dim), |ui| {
                                if ui.button("▦ Blank database").clicked() {
                                    self.new_database();
                                    ui.close_menu();
                                }
                                ui.separator();
                                ui.label(RichText::new("Templates").size(11.5).color(p.dim));
                                if ui.button("🤝 CRM — companies, contacts, opportunities (linked)").clicked() {
                                    self.create_db_from_template("crm");
                                    ui.close_menu();
                                }
                                if ui.button("📊 Project tracker (linked to projects & tasks)").clicked() {
                                    self.create_db_from_template("projects");
                                    ui.close_menu();
                                }
                                if ui.button("🗓 Content calendar").clicked() {
                                    self.create_db_from_template("content");
                                    ui.close_menu();
                                }
                                let mine: Vec<(Id, String)> = self.store.data.databases.iter().filter(|x| x.template).map(|x| (x.id.clone(), x.name.clone())).collect();
                                for (id, name) in mine {
                                    if ui.button(format!("⧉ {name}")).clicked() {
                                        self.create_db_from_template(&id);
                                        ui.close_menu();
                                    }
                                }
                            });
                        });
                    });
                    let dbs: Vec<(Id, String, String)> = self.store.data.databases.iter().filter(|x| !x.template).map(|x| (x.id.clone(), x.icon.clone(), x.name.clone())).collect();
                    for (id, icon, name) in dbs {
                        if nav_item(ui, p, if icon.is_empty() { "▦" } else { &icon }, &name, None, v == View::Database(id.clone()), Some(p.text)).clicked() {
                            self.go(View::Database(id));
                        }
                    }
                });
                ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                    let status = if let Some(e) = &self.store.last_error {
                        format!("⚠ {e}")
                    } else if self.dirty {
                        "Saving…".into()
                    } else if !self.hub_status.is_empty() {
                        format!("✓ Saved on this PC · {}", self.hub_status)
                    } else if !self.sync_status.is_empty() {
                        format!("✓ Saved on this PC · {}", self.sync_status)
                    } else {
                        "✓ Saved on this PC · works offline".into()
                    };
                    ui.label(RichText::new(status).size(11.0).color(p.faint));
                    if nav_item(ui, p, "⚙", "Settings, sync & AI", None, v == View::Settings, None).clicked() {
                        self.go(View::Settings);
                    }
                    if nav_item(ui, p, "❓", "Help & tutorial", None, v == View::Help, None).clicked() {
                        self.go(View::Help);
                    }
                });
            });
    }

    fn page_tree(&mut self, ui: &mut egui::Ui, p: &Pal, parent: Option<&str>, depth: usize) {
        if depth > 8 {
            return;
        }
        // pages that belong to database rows live under their database, templates under the + menu
        let mut kids: Vec<(Id, String, String, i64)> = self
            .store
            .data
            .pages
            .iter()
            .filter(|x| x.parent.as_deref() == parent && !x.template)
            .filter(|x| depth > 0 || self.store.data.row_for_page(&x.id).is_none())
            .map(|x| (x.id.clone(), x.icon.clone(), x.title.clone(), x.created))
            .collect();
        kids.sort_by_key(|k| k.3);
        for (id, icon, title, _) in kids {
            let has_kids = self.store.data.pages.iter().any(|x| x.parent.as_deref() == Some(&id));
            let open = self.expanded.contains(&id);
            let sel = self.view == View::Page(id.clone());
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.add_space(depth as f32 * 14.0);
                let arrow = if !has_kids { " " } else if open { "⏷" } else { "⏵" };
                if ui.add(egui::Button::new(RichText::new(arrow).size(10.0).color(p.faint)).frame(false).min_size(egui::vec2(14.0, 30.0))).clicked() && has_kids {
                    if open {
                        self.expanded.remove(&id);
                    } else {
                        self.expanded.insert(id.clone());
                    }
                }
                let t = if title.is_empty() { "Untitled".to_string() } else { title.clone() };
                if nav_item(ui, p, if icon.is_empty() { "📄" } else { &icon }, &t, None, sel, Some(p.text)).clicked() {
                    self.go(View::Page(id.clone()));
                }
            });
            if open {
                self.page_tree(ui, p, Some(&id), depth + 1);
            }
        }
    }

    fn quick_window(&mut self, ctx: &egui::Context, p: &Pal) {
        let Some(mut q) = self.quick.take() else { return };
        let mut keep = true;
        let mut submit = false;
        egui::Window::new("quick-add")
            .title_bar(false)
            .resizable(false)
            .anchor(Align2::CENTER_TOP, [0.0, 110.0])
            .fixed_size([600.0, 0.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    for (k, label) in [(QuickKind::Task, "Task"), (QuickKind::Event, "Event"), (QuickKind::Page, "Page")] {
                        if pill(ui, p, label, q.kind == k).clicked() {
                            q.kind = k;
                            q.focus = true;
                        }
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(RichText::new("Enter to save · Esc to close").size(11.0).color(p.faint));
                    });
                });
                ui.add_space(6.0);
                let hint = match q.kind {
                    QuickKind::Task => "e.g. Renew passport next fri 10am p1 #Home remind 1h",
                    QuickKind::Event => "e.g. Lunch with Sam tomorrow 1pm for 90m",
                    QuickKind::Page => "Page title",
                };
                let mut r = None;
                ui.horizontal(|ui| {
                    r = Some(ui.add(egui::TextEdit::singleline(&mut q.text).hint_text(hint).font(fp(17.0)).desired_width(ui.available_width() - 30.0).frame(false).margin(egui::vec2(4.0, 8.0))));
                    if mic_button(ui, p) {
                        q.focus = true;
                    }
                });
                let r = r.unwrap();
                if q.focus {
                    r.request_focus();
                    q.focus = false;
                }
                if r.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                    submit = true;
                }
                ui.separator();
                if q.kind != QuickKind::Page {
                    let c = crate::nlp::parse(&q.text, today());
                    ui.horizontal_wrapped(|ui| {
                        if c.due.is_none() {
                            if let Some(d) = q.due {
                                chip(ui, &format!("📅 {}", fmt_date_short(d)), p.green);
                            }
                        }
                        for s in c.chips() {
                            chip(ui, &s, p.blue);
                        }
                        if q.parent.is_some() {
                            chip(ui, "⑂ subtask", p.purple);
                        }
                    });
                }
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    if q.kind == QuickKind::Task {
                        let label = self.store.data.project_name(&q.project);
                        egui::ComboBox::from_id_salt("qa-project").selected_text(format!("# {label}")).width(180.0).show_ui(ui, |ui| {
                            ui.selectable_value(&mut q.project, None, "📥 Inbox");
                            for pr in &self.store.data.projects {
                                ui.selectable_value(&mut q.project, Some(pr.id.clone()), format!("# {}", pr.name));
                            }
                        });
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let label = match q.kind {
                            QuickKind::Task => "Add task",
                            QuickKind::Event => "Add event",
                            QuickKind::Page => "Create page",
                        };
                        if primary_button(ui, p, label).clicked() {
                            submit = true;
                        }
                        if ui.button("Cancel").clicked() {
                            keep = false;
                        }
                    });
                });
            });
        if submit && !q.text.trim().is_empty() {
            match q.kind {
                QuickKind::Task => {
                    if self.add_task_text(&q.text, q.project.clone(), &q.section, q.due, q.parent.clone(), q.page.clone()).is_some() {
                        self.toast("Task added");
                    }
                }
                QuickKind::Event => {
                    let c = crate::nlp::parse(&q.text, today());
                    let date = c.due.or(q.due).unwrap_or_else(today);
                    let start_t = c.time.unwrap_or_else(|| NaiveTime::from_hms_opt((now_local().hour() + 1).min(23), 0, 0).unwrap());
                    let start = date.and_time(start_t);
                    let cal = self.store.data.calendars.iter().find(|c| c.visible).or(self.store.data.calendars.first()).map(|c| c.id.clone());
                    if let Some(cal) = cal {
                        let e = Event {
                            id: new_id(),
                            calendar_id: cal,
                            title: c.title.clone(),
                            start,
                            end: start + Duration::minutes(c.duration_min.unwrap_or(60) as i64),
                            all_day: c.time.is_none() && c.due.is_some() && c.duration_min.is_none(),
                            location: String::new(),
                            notes: String::new(),
                            recurrence: c.recurrence,
                            page_id: q.page.clone(),
                            reminder_min: Some(c.reminder_min.unwrap_or(10)),
                            updated: now_ms(),
                        };
                        self.store.data.events.push(e);
                        self.touch();
                        self.cal.anchor = date;
                        self.toast(format!("Event added on {}", fmt_date_short(date)));
                    } else {
                        self.toast("Create a calendar first");
                    }
                }
                QuickKind::Page => {
                    let id = self.new_page(None);
                    if let Some(pg) = self.store.data.page_mut(&id) {
                        pg.title = q.text.trim().to_string();
                    }
                    self.temp.remove("focus_title");
                }
            }
            q.text.clear();
            q.focus = true;
            // keep the window for tasks so several can be captured in a row
            if q.kind != QuickKind::Task {
                keep = false;
            }
        }
        if keep {
            self.quick = Some(q);
        }
    }

    fn palette_window(&mut self, ctx: &egui::Context, p: &Pal) {
        let Some(mut pal_state) = self.palette.take() else { return };
        let mut keep = true;
        let mut chosen: Option<usize> = None;
        let hits = crate::search::search(&self.store.data, &pal_state.query, 30);
        // commands when nothing typed
        let cmds: Vec<(&str, &str)> = vec![
            ("🏠", "Home dashboard"),
            ("📥", "Inbox"),
            ("📅", "Today"),
            ("📆", "Calendar"),
            ("📄", "New page"),
            ("▦", "New database"),
            ("⚙", "Settings"),
            ("🌓", "Toggle dark mode"),
        ];
        let n_items = if pal_state.query.trim().is_empty() { cmds.len() } else { hits.len() + 2 };
        let (down, up, enter) = ctx.input_mut(|i| (i.consume_key(Modifiers::NONE, Key::ArrowDown), i.consume_key(Modifiers::NONE, Key::ArrowUp), i.key_pressed(Key::Enter)));
        if down {
            pal_state.sel = (pal_state.sel + 1).min(n_items.saturating_sub(1));
        }
        if up {
            pal_state.sel = pal_state.sel.saturating_sub(1);
        }
        egui::Window::new("palette")
            .title_bar(false)
            .resizable(false)
            .anchor(Align2::CENTER_TOP, [0.0, 90.0])
            .fixed_size([640.0, 0.0])
            .show(ctx, |ui| {
                let r = ui.add(egui::TextEdit::singleline(&mut pal_state.query).hint_text("🔍  Search pages, tasks, events, databases…   (page: task: event: db:)").font(fp(17.0)).desired_width(f32::INFINITY).frame(false).margin(egui::vec2(4.0, 8.0)));
                if pal_state.focus {
                    r.request_focus();
                    pal_state.focus = false;
                }
                if r.changed() {
                    pal_state.sel = 0;
                }
                if enter {
                    chosen = Some(pal_state.sel);
                }
                ui.separator();
                egui::ScrollArea::vertical().max_height(440.0).show(ui, |ui| {
                    if pal_state.query.trim().is_empty() {
                        ui.label(RichText::new("Jump to").size(12.0).color(p.dim));
                        for (i, (icon, label)) in cmds.iter().enumerate() {
                            let sel = i == pal_state.sel;
                            if ui.add(egui::SelectableLabel::new(sel, format!("{icon}   {label}"))).clicked() {
                                chosen = Some(i);
                            }
                        }
                    } else {
                        if hits.is_empty() {
                            ui.label(RichText::new("No matches").color(p.dim));
                        }
                        for (i, h) in hits.iter().enumerate() {
                            let sel = i == pal_state.sel;
                            let resp = ui.add(egui::SelectableLabel::new(sel, RichText::new(format!("{}  {}", h.kind.icon(), h.title)).font(fp(14.5))));
                            ui.label(RichText::new(format!("      {} · {}", h.kind.label(), h.snippet)).size(12.0).color(p.dim));
                            if resp.clicked() {
                                chosen = Some(i);
                            }
                        }
                        ui.separator();
                        let base = hits.len();
                        if ui.add(egui::SelectableLabel::new(pal_state.sel == base, format!("+ Add task \"{}\"", pal_state.query.trim()))).clicked() {
                            chosen = Some(base);
                        }
                        if ui.add(egui::SelectableLabel::new(pal_state.sel == base + 1, format!("+ New page \"{}\"", pal_state.query.trim()))).clicked() {
                            chosen = Some(base + 1);
                        }
                    }
                });
            });
        if let Some(i) = chosen {
            keep = false;
            if pal_state.query.trim().is_empty() {
                match i {
                    0 => self.go(View::Dashboard),
                    1 => self.go(View::Inbox),
                    2 => self.go(View::Today),
                    3 => self.go(View::Calendar),
                    4 => {
                        self.new_page(None);
                    }
                    5 => {
                        self.new_database();
                    }
                    6 => self.go(View::Settings),
                    _ => {
                        self.reg.settings.dark = !self.reg.settings.dark;
                        self.reg.save();
                    }
                }
            } else if i < hits.len() {
                self.open_hit(&hits[i]);
            } else if i == hits.len() {
                let q = pal_state.query.clone();
                self.add_task_text(&q, None, "", None, None, None);
                self.toast("Task added to Inbox");
            } else {
                let id = self.new_page(None);
                if let Some(pg) = self.store.data.page_mut(&id) {
                    pg.title = pal_state.query.trim().to_string();
                }
                self.temp.remove("focus_title");
            }
        }
        if keep {
            self.palette = Some(pal_state);
        }
    }

    pub fn open_hit(&mut self, h: &crate::search::Hit) {
        use crate::search::HitKind;
        match h.kind {
            HitKind::Page => self.go(View::Page(h.id.clone())),
            HitKind::Task => {
                let pid = self.store.data.task(&h.id).and_then(|t| t.project_id.clone());
                self.go(match pid {
                    Some(p) => View::Project(p),
                    None => View::Inbox,
                });
                self.open_task = Some(h.id.clone());
            }
            HitKind::Event => {
                if let Some(e) = self.store.data.events.iter().find(|e| e.id == h.id).cloned() {
                    self.cal.anchor = e.start.date();
                    self.go(View::Calendar);
                    self.event_edit = Some(calendar::draft_from(&e));
                }
            }
            HitKind::Project => self.go(View::Project(h.id.clone())),
            HitKind::Row => {
                self.db_ui.highlight = h.row_id.clone();
                self.go(View::Database(h.id.clone()));
            }
        }
    }

    fn new_ws_window(&mut self, ctx: &egui::Context, p: &Pal) {
        let Some(mut name) = self.new_ws.take() else { return };
        let mut keep = true;
        egui::Window::new("New workspace").collapsible(false).resizable(false).anchor(Align2::CENTER_TOP, [0.0, 140.0]).show(ctx, |ui| {
            ui.label(RichText::new("A workspace is completely separate: its own pages, tasks, calendars, backups, sync file and AI access.").color(p.dim));
            ui.add_space(6.0);
            let r = ui.add(egui::TextEdit::singleline(&mut name).hint_text("e.g. Work, Side business").desired_width(320.0));
            r.request_focus();
            ui.horizontal(|ui| {
                if (primary_button(ui, p, "Create").clicked() || (r.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)))) && !name.trim().is_empty() {
                    let colors: [[u8; 3]; 5] = [[63, 81, 181], [41, 171, 135], [255, 153, 51], [175, 56, 235], [20, 170, 245]];
                    let c = colors[self.reg.workspaces.len() % 5];
                    let meta = self.reg.create(name.trim(), c);
                    self.switch_workspace(&meta.id);
                    keep = false;
                }
                if ui.button("Cancel").clicked() {
                    keep = false;
                }
            });
        });
        if keep {
            self.new_ws = Some(name);
        }
    }

    fn toasts_ui(&mut self, ctx: &egui::Context, p: &Pal) {
        let now = self.now;
        self.toasts.retain(|t| t.1 > now);
        self.undo_toasts.retain(|t| t.1 > now);
        if self.toasts.is_empty() && self.undo_toasts.is_empty() {
            return;
        }
        let mut undo: Option<Id> = None;
        egui::Area::new(egui::Id::new("toasts")).anchor(Align2::LEFT_BOTTOM, [290.0, -20.0]).order(egui::Order::Tooltip).show(ctx, |ui| {
            for (msg, until, cs) in self.undo_toasts.iter().rev().take(3) {
                egui::Frame::none().fill(p.toast_bg).rounding(8.0).inner_margin(egui::Margin::symmetric(14.0, 10.0)).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(msg).color(p.toast_text).size(13.5));
                        if ui.add(egui::Button::new(RichText::new("Undo").color(p.accent).strong()).frame(false)).clicked() {
                            undo = Some(cs.clone());
                        }
                    });
                    // depletion bar: time left to undo
                    let frac = ((until - now) / 6.0).clamp(0.0, 1.0) as f32;
                    let r = ui.available_rect_before_wrap();
                    let bar = egui::Rect::from_min_size(egui::pos2(r.left(), r.top() + 2.0), egui::vec2(r.width().max(120.0) * frac, 2.0));
                    ui.painter().rect_filled(bar, 1.0, p.accent);
                    ui.add_space(4.0);
                });
                ui.add_space(6.0);
            }
            for (msg, _) in self.toasts.iter().rev().take(4) {
                egui::Frame::none().fill(p.toast_bg).rounding(8.0).inner_margin(egui::Margin::symmetric(14.0, 10.0)).show(ui, |ui| {
                    ui.label(RichText::new(msg).color(p.toast_text).size(13.5));
                });
                ui.add_space(6.0);
            }
        });
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
        if let Some(cs) = undo {
            self.undo(&cs);
        }
    }

    fn handle_screenshot(&mut self, ctx: &egui::Context) {
        let Some(shot) = self.shot.as_mut() else { return };
        ctx.request_repaint();
        if self.frame >= 120 && !shot.requested {
            shot.requested = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot);
        }
        let mut saved = false;
        ctx.input(|i| {
            for e in &i.raw.events {
                if let egui::Event::Screenshot { image, .. } = e {
                    let [w, h] = image.size;
                    let mut bytes = Vec::with_capacity(w * h * 4);
                    for px in &image.pixels {
                        bytes.extend_from_slice(&px.to_array());
                    }
                    if let Some(img) = image::RgbaImage::from_raw(w as u32, h as u32, bytes) {
                        if let Some(parent) = shot.path.parent() {
                            let _ = std::fs::create_dir_all(parent);
                        }
                        match img.save(&shot.path) {
                            Ok(()) => eprintln!("Saved screenshot to {}", shot.path.display()),
                            Err(e) => eprintln!("Couldn't save screenshot: {e}"),
                        }
                    }
                    saved = true;
                }
            }
        });
        if saved || self.frame > 900 {
            if self.dirty {
                self.save_now();
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    fn central(&mut self, ui: &mut egui::Ui, p: &Pal) {
        // top bar: back + view-specific actions live inside each view
        match self.view.clone() {
            View::Dashboard => self.dashboard(ui, p),
            View::Inbox | View::Today | View::Upcoming | View::Filters | View::Project(_) => self.tasks_view(ui, p),
            View::Calendar => self.calendar_view(ui, p),
            View::Page(id) => {
                if self.store.data.page(&id).is_some() {
                    self.page_view(ui, p, &id)
                } else {
                    self.view = View::Dashboard;
                }
            }
            View::Database(id) => {
                if self.store.data.database(&id).is_some() {
                    self.database_view(ui, p, &id)
                } else {
                    self.view = View::Dashboard;
                }
            }
            View::Settings => self.settings_view(ui, p),
            View::Help => self.help_view(ui, p),
            View::Intake(id) => self.intake_view(ui, p, &id),
            View::Resolve(id) => self.resolve_view(ui, p, &id),
            View::TestLab => self.testlab_view(ui, p),
        }
    }

    pub fn top_bar(&mut self, ui: &mut egui::Ui, p: &Pal, add_right: impl FnOnce(&mut Self, &mut egui::Ui)) {
        ui.horizontal(|ui| {
            ui.set_height(30.0);
            ui.add_space(12.0);
            let can_back = !self.back.is_empty();
            if ui.add_enabled(can_back, egui::Button::new(RichText::new("⏴").size(14.0).color(p.dim)).frame(false)).on_hover_text("Back").clicked() {
                if let Some(v) = self.back.pop() {
                    self.view = v;
                }
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add_space(12.0);
                add_right(self, ui);
            });
        });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.frame += 1;
        self.now = ctx.input(|i| i.time);
        let p = self.pal();
        if self.theme_applied != Some(p.dark) {
            apply(ctx, &p);
            self.theme_applied = Some(p.dark);
        }
        if self.now - self.last_poll > 1.5 {
            self.last_poll = self.now;
            if self.store.reload_if_changed() {
                // edits arrived from an AI assistant, the phone page or sync
                ctx.request_repaint();
            }
        }
        if !self.store.settings.sync_dir.trim().is_empty() && self.now - self.last_sync > 60.0 && self.shot.is_none() {
            self.last_sync = self.now;
            match self.store.sync() {
                Ok(_) => self.sync_status = format!("synced {}", chrono::Local::now().format("%H:%M")),
                Err(e) => self.sync_status = format!("sync: {e}"),
            }
        }
        self.resolve_poll(ctx);
        self.check_reminders(ctx);
        self.shortcuts(ctx);
        self.hub_tick(false);
        // ticking a page to-do ticks its task, and the other way round
        if self.store.data.sync_todo_links() {
            self.touch();
        }
        // external calendar subscriptions: refresh every 30 minutes, apply downloads as they arrive
        if self.now - self.last_sub_check > 1800.0 && self.shot.is_none() {
            self.last_sub_check = self.now;
            let ids: Vec<Id> = self.store.data.calendars.iter().filter(|c| !c.source_url.is_empty()).map(|c| c.id.clone()).collect();
            for id in ids {
                self.refresh_subscription(&id);
            }
            if self.settings_ui.message == "Fetching the calendar…" {
                self.settings_ui.message.clear();
            }
        }
        while let Ok((cal, res)) = self.sub_rx.try_recv() {
            let name = self.store.data.calendar(&cal).map(|c| c.name.clone()).unwrap_or_default();
            match res {
                Ok(text) => {
                    let n = crate::import::apply_subscription(&mut self.store.data, &cal, &text);
                    self.touch();
                    self.settings_ui.message = format!("{name}: {n} events up to date");
                }
                Err(e) => self.settings_ui.message = format!("⚠ {name}: {e}"),
            }
        }

        // the intake (and the Test Intake lab) are full screen: no sidebar, no app chrome
        let full_screen = self.testlab || matches!(self.view, View::Intake(_));
        if !full_screen {
            self.sidebar(ctx, &p);
        }
        if let Some(tid) = self.open_task.clone().filter(|_| !full_screen) {
            if self.store.data.task(&tid).is_some() {
                self.task_panel(ctx, &p, &tid);
            } else {
                self.open_task = None;
            }
        }
        if self.page_ui.ai_open && !full_screen {
            let pid = match &self.view {
                View::Page(id) => Some(id.clone()),
                _ => None,
            };
            self.ai_panel(ctx, &p, pid);
        }
        egui::CentralPanel::default().frame(egui::Frame::none().fill(p.bg)).show(ctx, |ui| {
            self.central(ui, &p);
        });
        self.quick_window(ctx, &p);
        self.palette_window(ctx, &p);
        self.event_window(ctx, &p);
        self.new_ws_window(ctx, &p);
        self.toasts_ui(ctx, &p);

        if self.dirty && self.now - self.last_edit > 0.6 {
            self.save_now();
        }
        if ctx.input(|i| i.viewport().close_requested()) && self.dirty {
            self.save_now();
        }
        self.handle_screenshot(ctx);
        ctx.request_repaint_after(std::time::Duration::from_millis(1000));
    }
}

/// Helper for views: a heading in the Todoist style.
pub fn view_title(ui: &mut egui::Ui, p: &Pal, title: &str, sub: Option<&str>) {
    ui.add_space(18.0);
    ui.label(RichText::new(title).font(fb(26.0)).color(p.text));
    if let Some(s) = sub {
        ui.label(RichText::new(s).size(13.0).color(p.dim));
    }
    ui.add_space(10.0);
}

pub fn hline(ui: &mut egui::Ui, p: &Pal) {
    let r = ui.available_rect_before_wrap();
    ui.painter().line_segment([egui::pos2(r.left(), r.top()), egui::pos2(r.right(), r.top())], Stroke::new(1.0, p.line));
    ui.add_space(1.0);
}

#[allow(dead_code)]
pub fn dt_label(dt: NaiveDateTime) -> String {
    dt.format("%a %-d %b %H:%M").to_string()
}
