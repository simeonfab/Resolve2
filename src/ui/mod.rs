pub mod theme;
pub mod widgets;
pub mod motion;
pub mod toast;
mod gallery;
pub mod shell;
pub mod calendar;
mod cockpit;
pub mod connect;
mod import_ui;
mod intake;
mod intake_ending;
mod intake_review;
mod intake_steps;
mod knowledge;
pub mod nav;
pub mod resolve;
pub mod sections;
mod team;
pub mod testlab;
pub mod dashboard;
pub mod plan;
mod help;
mod assistant;
mod databases;
mod pages;
pub mod overview;
pub mod roadmap;
pub(crate) mod settings;
pub mod tasks;

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
    /// Old link to a project's Resolve cockpit: opens Overview for that project.
    Resolve(Id),
    /// All projects.
    Projects,
    /// A product-management or collaboration section (see nav.rs), for the picked project.
    Sec(nav::Section),
    /// Test Intake home (only in the sandbox process).
    TestLab,
    /// "Connect your AI" (first launch, full screen).
    ConnectAi,
    /// Every token, component, toast and motion sample (Phase 0 gallery).
    Gallery,
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
    /// When the dialog opened (NaN until its first frame), for its entrance.
    pub opened: f64,
    /// The last save failed: the text stays and the dialog says so.
    pub error: bool,
    /// The "New project: Name" chip was dismissed (×): saving won't create that project.
    pub new_project_off: bool,
}

pub struct Palette {
    pub query: String,
    pub sel: usize,
    pub focus: bool,
    /// When it opened (0 = this frame), for its entrance.
    pub opened: f64,
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
    /// The next three weeks as a list, by day (B-E12).
    Agenda,
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
    /// the frame the picture was asked for (the window waits for it before closing)
    pub requested_at: u64,
}

pub struct StartOpts {
    pub screenshot: Option<PathBuf>,
    pub workspace: Option<String>,
    pub view: Option<String>,
    /// Test Intake mode: sandbox data folder, no sidebar, no MCP/phone server.
    pub testlab: bool,
    /// --state full|empty|loading|error (screenshots).
    pub state: Option<String>,
    /// --theme light|dark (screenshots; not saved).
    pub theme: Option<String>,
    /// --reduced-motion (screenshots; not saved).
    pub reduced_motion: bool,
    /// --scroll PX: start scrolled down (screenshots of long views).
    pub scroll: Option<f32>,
    /// --collapsed: the sidebar collapsed (screenshots; not saved).
    pub collapsed: bool,
    /// --motion-at <ms>: the intake's step, card, heard and question motions held at that moment
    /// (screenshots of a frame mid-motion).
    pub motion_at: Option<f32>,
    /// --open ask[:question]|search[:query]|setup: open the Ask sheet (optionally asking a real
    /// question first), the search palette, or show an unfinished sample intake (screenshots).
    pub open: Option<String>,
    /// --project <name>: pick that project in the sidebar (screenshots).
    pub project: Option<String>,
    /// --provider / --model: the AI for a screenshot run (in memory, never saved).
    pub provider: Option<String>,
    pub model: Option<String>,
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
    /// The project picked in the sidebar (None = All projects).
    pub current_project: Option<Id>,
    pub connect: connect::ConnectUi,
    pub import: import_ui::ImportUi,
    /// One toast at a time (B-E10).
    pub toaster: toast::Toaster,
    /// --state full|empty|loading|error: which state views draw (screenshots of the four states).
    pub view_state: ViewState,
    /// Never write to disk (screenshots that swap in sample or empty data).
    pub no_save: bool,
    /// The sidebar collapsing or expanding: (when, collapsing?).
    pub sidebar_anim: Option<(f64, bool)>,
    /// The Ask bar and sheet.
    pub ask: shell::AskUi,
    /// The search palette while it closes.
    pub search_fx: Option<shell::SearchFx>,
    /// Plan my day (B-E21).
    pub plan: plan::PlanUi,
    /// The screen shown last frame, and when it was entered (for screen entrances).
    pub view_seen: Option<View>,
    pub view_since: f64,
    /// The intake pane's height this frame (steps pin their action bar to its foot).
    pub intake_pane_h: f32,
    /// The intake step that is leaving (M02.1), with the direction: drawn fading out for 240 ms.
    pub intake_ghost: Option<(crate::resolve::intake::Intake, f32)>,
    /// True while the leaving step is drawn (it must not start AI work or take focus).
    pub intake_ghosting: bool,
    /// --motion-at (see Opts).
    pub motion_at: Option<f32>,
}

/// The four states every view draws (Rules, item 3).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ViewState {
    #[default]
    Full,
    Empty,
    Loading,
    Error,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, opts: StartOpts) -> Self {
        install_fonts(&cc.egui_ctx);
        let reg = Registry::load();
        let meta = opts
            .workspace
            .as_deref()
            .and_then(|w| reg.find(w).cloned())
            // Test Intake keeps all runs in the sandbox's "Personal" workspace
            .or_else(|| if opts.testlab { reg.find("Personal").cloned() } else { None })
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
            Some("filters") => View::Filters,
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
            shot: opts.screenshot.map(|path| Shot { path, requested: false, requested_at: 0 }),
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
            current_project: None,
            connect: connect::ConnectUi::default(),
            import: import_ui::ImportUi::default(),
            toaster: toast::Toaster::default(),
            view_state: ViewState::default(),
            no_save: false,
            sidebar_anim: None,
            ask: shell::AskUi::default(),
            search_fx: None,
            plan: plan::PlanUi::default(),
            view_seen: None,
            view_since: 0.0,
            intake_pane_h: 0.0,
            intake_ghost: None,
            intake_ghosting: false,
            motion_at: opts.motion_at,
        };
        app.page_ui.ai_open = open_ai;
        // reduced motion: the saved setting, or --reduced-motion for a screenshot
        motion::set_reduced(app.reg.settings.reduced_motion || opts.reduced_motion);
        // --theme light|dark: in memory only (the saved choice is untouched)
        match opts.theme.as_deref() {
            Some("light") => app.reg.settings.dark = false,
            Some("dark") => app.reg.settings.dark = true,
            _ => {}
        }
        // --state full|empty|loading|error: which of the four states views draw. Full and empty
        // also swap in sample or empty data, in memory only (never saved).
        app.view_state = match opts.state.as_deref() {
            Some("empty") => ViewState::Empty,
            Some("loading") => ViewState::Loading,
            Some("error") => ViewState::Error,
            _ => ViewState::Full,
        };
        if app.shot.is_some() {
            // sample or empty data for a screenshot must never reach the disk
            app.no_save = matches!(opts.state.as_deref(), Some("full") | Some("empty") | Some("loading") | Some("error"));
            match opts.state.as_deref() {
                Some("full") | Some("loading") | Some("error") => {
                    // the design canvases' sample (café flat-fee bookkeeping), so screens compare like for like
                    let mut d = WorkspaceData::default();
                    crate::seed::seed_canvas(&mut d);
                    app.store.data = d;
                }
                Some("empty") => {
                    let mut d = WorkspaceData::default();
                    crate::seed::seed_empty(&mut d);
                    app.store.data = d;
                }
                _ => {}
            }
        }
        // --collapsed: in memory only
        if opts.collapsed {
            app.store.settings.sidebar_collapsed = true;
        }
        // calendar-agenda | calendar-week | calendar-month: the calendar in that mode
        if let Some(m) = opts.view.as_deref().and_then(|v| v.strip_prefix("calendar-")) {
            app.view = View::Calendar;
            app.cal.mode = match m {
                "agenda" => CalMode::Agenda,
                "week" => CalMode::Week,
                "day" => CalMode::Day,
                _ => CalMode::Month,
            };
        }
        if opts.view.as_deref() == Some("gallery") {
            app.view = View::Gallery;
        }
        if let Some(s) = opts.scroll {
            app.temp.insert("scroll".into(), s.to_string());
        }
        // --view <section>[@p]: open a section; "@p" picks the most recently guided project
        if let Some(v) = opts.view.as_deref() {
            let (name, pick) = match v.split_once('@') {
                Some((n, _)) => (n, true),
                None => (v, false),
            };
            // knowledge-<n>: a Knowledge tab
            let name = match name.strip_prefix("knowledge-").and_then(|n| n.parse::<usize>().ok()) {
                Some(n) => {
                    app.resolve_ui.knowledge_tab = n;
                    "knowledge"
                }
                None => name,
            };
            let sec = match name {
                "overview" => Some(nav::Section::Overview),
                "roadmap" => Some(nav::Section::Roadmap),
                "frameworks" => Some(nav::Section::Frameworks),
                "knowledge" => Some(nav::Section::Knowledge),
                "team" => Some(nav::Section::AiTeam),
                "handoffs" => Some(nav::Section::Handoffs),
                _ => None,
            };
            if pick {
                app.current_project = app.store.data.contexts.iter().filter(|c| app.store.data.project(&c.project_id).is_some()).max_by_key(|c| c.created).map(|c| c.project_id.clone());
            }
            // --state empty --project <name>: the canvases' empty state is a project made but not
            // set up (no tasks, no roadmap), so add a bare one, in memory only
            if let (ViewState::Empty, Some(name)) = (app.view_state, opts.project.as_deref()) {
                if app.shot.is_some() && !app.store.data.projects.iter().any(|p| p.name.eq_ignore_ascii_case(name)) {
                    let pr = Project { id: new_id(), name: name.to_string(), color: crate::ui::theme::PICK_COLOURS[2], sections: vec![], page_id: None, archived: false, favorite: false, order: 1, updated: now_ms() };
                    let pid = pr.id.clone();
                    app.store.data.projects.push(pr);
                    if let Some(mut ctx) = crate::resolve::ProjectContext::adopt(&app.store.data, &pid) {
                        ctx.outcome = crate::seed::canvas_outcome(name);
                        app.store.data.contexts.push(ctx);
                    }
                }
            }
            // --project <name>: that project (screenshots of a particular one)
            if let Some(pid) = opts.project.as_deref().and_then(|n| app.store.data.projects.iter().find(|p| p.name.eq_ignore_ascii_case(n))).map(|p| p.id.clone()) {
                app.current_project = Some(pid);
            }
            if let Some(s) = sec {
                app.view = View::Sec(s);
            } else if let Some(n) = name.strip_prefix("help-").and_then(|n| n.parse::<usize>().ok()) {
                app.temp.insert("help_section".into(), n.to_string());
                app.view = View::Help;
            } else if let Some(n) = name.strip_prefix("settings-").and_then(|n| n.parse::<usize>().ok()) {
                app.settings_ui.tab = n;
                app.view = View::Settings;
            } else if name == "projects" {
                app.view = View::Projects;
            } else if name == "tasks-board" {
                if let Some(pid) = app.current_project.clone() {
                    app.tasks_ui.board.insert(pid.clone());
                    app.view = View::Project(pid);
                }
            } else if name == "tasks" {
                app.view = app.current_project.clone().map(View::Project).unwrap_or(View::Today);
            }
        }
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
            match opts.view.as_deref() {
                // a fresh copy: Connect your AI first, then the intake
                None if !app.reg.settings.ai_setup_done => app.view = View::ConnectAi,
                None => app.open_intake_if_new(),
                Some("connect") => app.view = View::ConnectAi,
                // --view intake-<stage>[-<ending step>]: show the newest intake at that stage
                // (screenshots / checks). In memory only; nothing is saved.
                Some(v) if v.starts_with("intake") => {
                    let rest = v.strip_prefix("intake-").unwrap_or("");
                    let (stage, step) = match rest.rsplit_once('-') {
                        Some((s, n)) if n.parse::<u8>().is_ok() => (s.to_string(), n.parse::<u8>().ok()),
                        _ => (rest.to_string(), None),
                    };
                    // screenshots use the canvases' café intake (in memory only)
                    let sample = if app.shot.is_some() { Some(crate::seed::intake_sample(&mut app.store.data)) } else { None };
                    // empty: a new intake, one question asked and nothing heard yet
                    if let (Some(sid), ViewState::Empty) = (&sample, app.view_state) {
                        if let Some(i) = app.store.data.intakes.iter_mut().find(|i| &i.id == sid) {
                            i.qas.truncate(1);
                            i.qas[0].answer = String::new();
                            i.qas[0].classified = String::new();
                            i.heard.clear();
                            i.understanding = None;
                            i.proposal = None;
                        }
                    }
                    let pick = sample.clone().or_else(|| app.store.data.intakes.iter().filter(|i| stage.is_empty() || i.stage == stage).max_by_key(|i| i.updated).map(|i| i.id.clone())).or_else(|| app.store.data.intakes.iter().max_by_key(|i| i.updated).map(|i| i.id.clone()));
                    if let Some(id) = pick {
                        if let Some(i) = app.store.data.intakes.iter_mut().find(|i| i.id == id) {
                            if !stage.is_empty() {
                                i.stage = stage.clone();
                            }
                            if let Some(n) = step {
                                i.ending = n;
                            }
                            if stage == "questions" {
                                i.cursor = i.qas.len().saturating_sub(1);
                            }
                        }
                        // the reveal shows the project Adopt made (in memory, like the rest)
                        if stage == "reveal" && sample.is_some() {
                            let _ = crate::resolve::journey::adopt_all(&mut app.store.data, &id);
                        }
                        app.view = View::Intake(id.clone());
                        app.intake_shot_state(&id, opts.open.as_deref());
                    }
                }
                _ => {}
            }
        }
        // --open ask | search[:query] (screenshots)
        // --provider / --model: the AI for a screenshot run, in memory only
        if app.shot.is_some() {
            if let Some(pv) = &opts.provider {
                app.reg.settings.ai_provider = pv.clone();
            }
            if let Some(m) = &opts.model {
                app.reg.settings.ai_model = m.clone();
            }
        }
        match opts.open.as_deref() {
            Some("ask") => app.page_ui.ai_open = true,
            Some("plan") => app.open_plan_day(),
            // add:<text>: Add task open with that typed (screenshots)
            Some(o) if o.starts_with("add:") => {
                app.open_quick(QuickKind::Task);
                if let Some(q) = app.quick.as_mut() {
                    q.text = o["add:".len()..].to_string();
                }
            }
            // task:<title>: the task editor open on that task (screenshots)
            Some(o) if o.starts_with("task:") => {
                app.open_task = app.store.data.tasks.iter().find(|t| t.title == o["task:".len()..]).map(|t| t.id.clone());
            }
            // event:<title>: the event editor open on that event (screenshots)
            Some(o) if o.starts_with("event:") => {
                app.event_edit = app.store.data.events.iter().find(|e| e.title == o["event:".len()..]).map(calendar::draft_from);
            }
            Some("try") => {
                app.temp.insert("show_try".into(), "1".into());
            }
            // the Roadmap's whole picture, its By theme list, or one theme open (screenshots)
            Some("whole") => {
                app.temp.insert("rm_whole".into(), "1".into());
            }
            Some("by-theme") => app.resolve_ui.roadmap_by_theme = true,
            Some(o) if o.starts_with("theme:") => {
                if let Some(r) = app.store.data.records.iter().find(|r| r.kind == crate::resolve::Kind::Roadmap && r.title == o["theme:".len()..]) {
                    app.temp.insert("rm_open".into(), r.id.clone());
                }
            }
            // ask:<question>: a real question to the AI, read-only, answered before the screenshot
            Some(o) if o.starts_with("ask:") => {
                app.page_ui.ai_open = true;
                app.page_ui.ai_can_act = false;
                let q = o["ask:".len()..].to_string();
                app.ask_send(&q);
            }
            // setup: an unfinished sample intake in memory (screenshots of "Continue setting up…")
            Some("setup") if app.no_save => {
                let mut i = crate::resolve::intake::Intake::new();
                i.name = "Sam".into();
                i.stage = "confirm".into();
                i.input = "Sample answers for a screenshot.".into();
                app.store.data.intakes.push(i);
            }
            Some(o) if o.starts_with("search") => app.palette = Some(Palette { query: o.strip_prefix("search").unwrap_or("").trim_start_matches(':').to_string(), sel: 0, focus: true, opened: 0.0 }),
            _ => {}
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
        let s: String = s.into();
        if s.starts_with('⚠') || s.starts_with("Couldn't") {
            crate::ai::set_last_error(&s);
        }
        self.toasts.push((s, self.now + 4.0));
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
        if self.no_save {
            self.dirty = false;
            return;
        }
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
        self.quick = Some(QuickAdd { text: String::new(), kind, project, section: String::new(), due, parent: None, page: None, focus: true, opened: f64::NAN, error: false, new_project_off: false });
    }

    /// Create a task from quick-capture text. Returns the new id.
    /// A #Name with no matching project goes nowhere new: the task keeps `project` (B-E11).
    pub fn add_task_text(&mut self, text: &str, project: Option<Id>, section: &str, default_due: Option<NaiveDate>, parent: Option<Id>, page: Option<Id>) -> Option<Id> {
        self.add_task_text_with(text, project, section, default_due, parent, page, false)
    }

    /// As `add_task_text`; `create_project` makes the #Name project when none matches (only Add
    /// task does this, when its "New project" chip is still showing at save).
    pub fn add_task_text_with(&mut self, text: &str, project: Option<Id>, section: &str, default_due: Option<NaiveDate>, parent: Option<Id>, page: Option<Id>, create_project: bool) -> Option<Id> {
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
                // only when asked (Add task's "New project" chip); never quietly
                None if create_project => {
                    let pr = Project { id: new_id(), name: pname.clone(), color: PICK_COLOURS[self.store.data.projects.len() % PICK_COLOURS.len()], sections: vec![], page_id: None, archived: false, favorite: false, order: self.store.data.projects.len() as i64 + 1, updated: now_ms() };
                    let id = pr.id.clone();
                    self.store.data.projects.push(pr);
                    Some(id)
                }
                None => t.project_id.clone(),
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
        let labels: Vec<String> = cs.split(',').filter_map(|id| self.store.data.history.iter().find(|c| c.id == id).map(|c| c.label.clone())).collect();
        let (mut n, mut skipped, mut err) = (0, 0, None);
        for id in cs.split(',').rev() {
            // kept as an "undo" change set, so Redo can bring it back
            match self.store.data.undo_recorded(id) {
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
            _ => self.toast(undo_message(&labels, skipped)),
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
                pg.title = format!("{} ({})", t.name, today().format("%-d %b %Y"));
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
                alerts.push(format!("⏰ {}, due {}", t.title, fmt_time(tm)));
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
        // Ctrl+Z / Ctrl+Y undo and redo workspace changes, but only when no text box has the
        // keyboard (there they edit the text, as usual)
        let typing = ctx.memory(|m| m.focused().is_some());
        let (undo, redo) = if typing || matches!(self.view, View::Intake(_)) {
            (false, false)
        } else {
            ctx.input_mut(|i| (i.consume_key(Modifiers::COMMAND, Key::Z), i.consume_key(Modifiers::COMMAND, Key::Y)))
        };
        if undo {
            self.do_undo();
        }
        if redo {
            self.do_redo();
        }
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
            self.palette = Some(Palette { query: String::new(), sel: 0, focus: true, opened: 0.0 });
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
        let backslash = ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::Backslash));
        if backslash && !matches!(self.view, View::Intake(_) | View::ConnectAi | View::Gallery) {
            self.toggle_sidebar();
        }
        if esc {
            if self.plan.open {
                self.plan.open = false;
            } else if self.page_ui.ai_open {
                self.page_ui.ai_open = false;
            } else if self.palette.is_some() {
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
        // Add task (B-E11, Pack B): a 640 dialog over a dimmed screen. Recognised words are
        // highlighted as you type (dates, times and repeats amber, priority terracotta, #project
        // green); "Resolve understood" shows them as plain chips; Goes to picks the project.
        let screen = ctx.screen_rect();
        if !q.opened.is_finite() {
            q.opened = self.now;
        }
        let el = ((self.now - q.opened) * 1000.0) as f32;
        let dim = motion::frame_at(el, moves::DIALOG_DIM, motion::reduced());
        egui::Area::new(egui::Id::new("quick-dim")).order(egui::Order::Foreground).fixed_pos(screen.min).show(ctx, |ui| {
            if backdrop(ui, screen, shadow::BACKDROP_DIALOG * dim.alpha) {
                keep = false;
            }
        });
        let f = motion::frame_at(el, moves::DIALOG_IN, motion::reduced());
        if !f.done {
            ctx.request_repaint();
        }
        let origin = egui::pos2(screen.center().x, screen.top() + space::QUICK_TOP);
        let area_id = egui::Id::new("quick-add");
        let (title, hint, action) = match q.kind {
            QuickKind::Task => ("Add task", "e.g. Call the garage tomorrow 9am", "Add task"),
            QuickKind::Event => ("Add event", "e.g. Lunch with Sam tomorrow 1pm for 90m", "Add event"),
            QuickKind::Page => ("New page", "Page title", "Create page"),
        };
        let today_ = today();
        // "Did you mean …?" clicked: (the typed #Name, the existing project)
        let mut did_mean: Option<(String, String)> = None;
        egui::Area::new(area_id).order(egui::Order::Foreground).fixed_pos(origin).pivot(Align2::CENTER_TOP).show(ctx, |ui| {
            ui.multiply_opacity(f.alpha.clamp(0.0, 1.0));
            dialog_frame(p).show(ui, |ui| {
                ui.set_width(space::QUICK_W);
                egui::Frame::none().inner_margin(egui::Margin { left: space::S18 + 2.0, right: space::S18 + 2.0, top: space::S18, bottom: space::S14 }).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(title).font(fs(size::SECTION)).color(p.text));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(RichText::new("Type it the way you'd say it").size(size::LABEL).color(p.dim));
                        });
                    });
                    ui.add_space(space::S10);
                    // the field, with recognised words tinted
                    let quick_id = egui::Id::new("quick-add-text");
                    let focused = ui.memory(|m| m.has_focus(quick_id));
                    let mut r = None;
                    egui::Frame::none().fill(p.field).stroke(Stroke::new(stroke::HAIR, if focused { tok::AMBER } else { p.line })).rounding(egui::Rounding::same(radius::FIELD + 2.0)).inner_margin(egui::Margin { left: space::S16, right: space::S10, top: space::S10, bottom: space::S10 }).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let task_kind = q.kind != QuickKind::Page;
                            let pal = p.clone();
                            let mut layouter = move |ui: &egui::Ui, text: &str, wrap: f32| {
                                let mut job = egui::text::LayoutJob::default();
                                let plain = egui::TextFormat { font_id: fp(size::QUICK), color: pal.text, ..Default::default() };
                                let mut at = 0;
                                if task_kind {
                                    for (rg, k) in crate::nlp::recognised(text, today_) {
                                        if rg.start > at {
                                            job.append(&text[at..rg.start], 0.0, plain.clone());
                                        }
                                        let bg = match k {
                                            crate::nlp::Recognised::Priority => pal.orange_tint,
                                            crate::nlp::Recognised::Project => pal.green_tint,
                                            _ => pal.accent_soft,
                                        };
                                        job.append(&text[rg.clone()], 0.0, egui::TextFormat { background: bg, ..plain.clone() });
                                        at = rg.end;
                                    }
                                }
                                job.append(&text[at.min(text.len())..], 0.0, plain);
                                job.wrap.max_width = wrap;
                                ui.fonts(|fo| fo.layout_job(job))
                            };
                            r = Some(ui.add(egui::TextEdit::singleline(&mut q.text).id(quick_id).hint_text(RichText::new(hint).color(p.faint)).frame(false).desired_width(ui.available_width() - space::MIC - space::S10).layouter(&mut layouter)));
                            mic_for(ui, p, quick_id, p.dark);
                        });
                    });
                    let r = r.unwrap();
                    if q.focus {
                        r.request_focus();
                        q.focus = false;
                    }
                    if r.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                        submit = true;
                    }
                    if q.kind != QuickKind::Page && !q.text.trim().is_empty() {
                        let c = crate::nlp::parse(&q.text, today_);
                        ui.add_space(space::S12);
                        ui.label(RichText::new("Resolve understood").font(fs(size::LABEL)).color(p.orange));
                        ui.add_space(space::S6);
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing.x = space::S8;
                            understood_chip(ui, p, if q.kind == QuickKind::Task { "Task" } else { "Event" }, &c.title, None);
                            if c.due.is_none() {
                                if let Some(d) = q.due {
                                    understood_chip(ui, p, "", &fmt_date_short(d), Some(crate::nlp::Recognised::When));
                                }
                            }
                            for (s, k) in c.plain_chips() {
                                if k != crate::nlp::Recognised::Project {
                                    understood_chip(ui, p, "", &s, Some(k));
                                    continue;
                                }
                                // #Name: an existing project, or a new one only if the chip stays (B-E11)
                                let name = c.project.clone().unwrap_or_default();
                                if self.store.data.project_by_name(&name).is_some() || name.eq_ignore_ascii_case("inbox") {
                                    understood_chip(ui, p, "", &s, Some(k));
                                    continue;
                                }
                                if !q.new_project_off {
                                    understood_chip(ui, p, "", &format!("New project: {name}"), Some(crate::nlp::Recognised::When));
                                    if icon_button(ui, Icon::Close, true, false).on_hover_text("Don't create this project").clicked() {
                                        q.new_project_off = true;
                                    }
                                }
                                let names: Vec<String> = self.store.data.projects.iter().filter(|x| !x.archived).map(|x| x.name.clone()).collect();
                                if let Some(close) = crate::nlp::closest_project(&name, &names) {
                                    if understood_chip(ui, p, "", &format!("Did you mean {close}?"), Some(crate::nlp::Recognised::Project)).on_hover_text("Use this project instead").clicked() {
                                        did_mean = Some((name.clone(), close.clone()));
                                    }
                                }
                            }
                            if q.parent.is_some() {
                                understood_chip(ui, p, "", "Subtask", Some(crate::nlp::Recognised::Label));
                            }
                        });
                        ui.add_space(space::S8);
                        ui.label(RichText::new("Click a chip to change it. Backspace on a highlighted word turns it back into plain text.").size(size::LABEL).color(p.dim));
                    }
                    if q.error {
                        ui.add_space(space::S8);
                        ui.label(RichText::new("Couldn't save the task. Your text is still here. Press Enter to try again.").size(size::LABEL).color(p.orange));
                    }
                });
                // footer: Goes to, Cancel, Add
                egui::Frame::none().fill(p.bg).rounding(egui::Rounding { nw: 0.0, ne: 0.0, sw: radius::DIALOG, se: radius::DIALOG }).inner_margin(egui::Margin::symmetric(space::S18 + 2.0, space::S14)).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        if q.kind == QuickKind::Task {
                            ui.label(RichText::new("Goes to").size(size::LABEL).color(p.dim));
                            let named = crate::nlp::parse(&q.text, today_).project;
                            let fallback = if q.project.is_some() { self.store.data.project_name(&q.project) } else { "Captured".to_string() };
                            let label = match named {
                                Some(n) if self.store.data.project_by_name(&n).is_some() => n,
                                Some(n) if !q.new_project_off && !n.eq_ignore_ascii_case("inbox") => format!("New project: {n}"),
                                _ => fallback,
                            };
                            egui::ComboBox::from_id_salt("qa-project").selected_text(RichText::new(label).font(fs(size::LABEL))).width(space::S40 * 4.5).show_ui(ui, |ui| {
                                ui.selectable_value(&mut q.project, None, "Captured");
                                for pr in self.store.data.projects.iter().filter(|x| !x.archived) {
                                    ui.selectable_value(&mut q.project, Some(pr.id.clone()), pr.name.clone());
                                }
                            });
                        }
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if button(ui, p, ButtonKind::Dark, &format!("{action}  Enter"), false, !q.text.trim().is_empty()).clicked() {
                                submit = true;
                            }
                            if button(ui, p, ButtonKind::Secondary, "Cancel  Esc", false, true).clicked() {
                                keep = false;
                            }
                        });
                    });
                });
            });
        });
        motion::transform_layer(ctx, egui::LayerId::new(egui::Order::Foreground, area_id), origin, moves::DIALOG_SCALE + (1.0 - moves::DIALOG_SCALE) * f.t, egui::vec2(0.0, -moves::DIALOG_RISE * (1.0 - f.t)));
        // "Did you mean …?": the task goes to that project and the #Name word leaves the text
        if let Some((typed, project)) = did_mean {
            q.text = crate::nlp::without_tag(&q.text, &typed);
            q.project = self.store.data.project_by_name(&project).map(|x| x.id.clone());
            q.new_project_off = false;
            q.focus = true;
        }
        if submit && !q.text.trim().is_empty() {
            match q.kind {
                QuickKind::Task => {
                    let title = crate::nlp::parse(&q.text, today()).title;
                    // a #Name with no project is made only if its "New project" chip is still showing
                    let create = !q.new_project_off;
                    if self.add_task_text_with(&q.text, q.project.clone(), &q.section, q.due, q.parent.clone(), q.page.clone(), create).is_some() {
                        q.new_project_off = false;
                        self.toast(format!("Added: {title}"));
                        q.error = false;
                    } else {
                        // keep the text, say what went wrong
                        q.error = true;
                        q.focus = true;
                        self.quick = Some(q);
                        return;
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

    fn handle_screenshot(&mut self, ctx: &egui::Context) {
        let Some(shot) = self.shot.as_mut() else { return };
        ctx.request_repaint();
        // a real question asked for the screenshot: wait for its answer (at most 3 minutes)
        let waiting = (self.resolve_ui.chat_job.is_some() || self.plan.job.is_some()) && self.now < 180.0;
        let Some(shot) = self.shot.as_mut() else { return };
        if self.frame >= 120 && !shot.requested && !waiting {
            shot.requested = true;
            shot.requested_at = self.frame as u64;
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
        // asked for the picture: give it time to arrive before closing
        let gave_up = if shot.requested { self.frame as u64 > shot.requested_at + 600 } else { self.frame > 900 && !waiting };
        if saved || gave_up {
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
            View::Resolve(id) => {
                // old links to the cockpit open the project's Overview
                self.current_project = Some(id);
                self.view = View::Sec(nav::Section::Overview);
            }
            View::Projects => self.projects_view(ui, p),
            View::Sec(s) => self.section_view(ui, p, s),
            View::TestLab => self.testlab_view(ui, p),
            View::ConnectAi => self.connect_view(ui, p),
            View::Gallery => self.gallery_view(ui, p),
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
        if self.view_seen.as_ref() != Some(&self.view) {
            self.view_seen = Some(self.view.clone());
            self.view_since = self.now;
        }
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
        self.poll_team(ctx);
        self.poll_plan(ctx);
        self.poll_import(ctx);
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
        let full_screen = self.testlab || matches!(self.view, View::Intake(_) | View::ConnectAi | View::Gallery);
        if !full_screen {
            self.nav_sidebar(ctx, &p);
        }
        if let Some(tid) = self.open_task.clone().filter(|_| !full_screen) {
            if self.store.data.task(&tid).is_some() {
                self.task_panel(ctx, &p, &tid);
            } else {
                self.open_task = None;
            }
        }
        // the Ask Resolve bar floats in a strip kept clear under the screen (not on AI Team,
        // which has its own chat box)
        let bar = !full_screen && nav::shows_ask_bar(self.nav_target_of(&self.view));
        let mut strip: Option<egui::Rect> = None;
        if bar {
            egui::TopBottomPanel::bottom("ask-strip").exact_height(space::ASK_STRIP).show_separator_line(false).frame(egui::Frame::none().fill(p.bg)).show(ctx, |ui| {
                strip = Some(ui.max_rect());
            });
        }
        egui::CentralPanel::default().frame(egui::Frame::none().fill(p.bg)).show(ctx, |ui| {
            // toasts, the Ask bar and the sheet centre on the work area
            let work = ui.max_rect();
            self.toaster.work = Some(strip.map_or(work, |s| work.union(s)));
            self.central(ui, &p);
        });
        if let Some(s) = strip {
            self.ask_bar_ui(ctx, &p, s);
        }
        if !full_screen {
            self.ask_sheet(ctx, &p);
            self.plan_sheet(ctx, &p);
        }
        self.quick_window(ctx, &p);
        self.search_palette(ctx, &p);
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

/// The toast after Undo (B-E17): names what was undone.
pub fn undo_message(labels: &[String], skipped: usize) -> String {
    let what = match labels {
        [] => String::new(),
        [one] => format!(": {one}"),
        [first, rest @ ..] => format!(": {first} and {} more", rest.len()),
    };
    if skipped > 0 {
        format!("Undone{what} ({skipped} edited since and left as they are)")
    } else {
        format!("Undone{what}")
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

/// The proposal's freshness key (the confirmed cards), for sample data.
pub fn intake_key(i: &crate::resolve::intake::Intake) -> String {
    intake_steps::und_key(i)
}

impl App {
    /// Screenshots of the intake's states (`--view intake-<step> --open …`), on the sample in
    /// memory: wait (ST1), wait:<seconds> (ST3), handoff (ST2), error (ST4), resume (ST5),
    /// edit (04b), pull (02c), template (02b), limit (12: the real budget stop runs).
    fn intake_shot_state(&mut self, id: &str, open: Option<&str>) {
        use crate::resolve::journey;
        let Some(open) = open else { return };
        let id = id.to_string();
        let now = self.now;
        let Some(i) = self.store.data.intakes.iter_mut().find(|i| i.id == id) else { return };
        match open {
            "wait" => {
                // the next question being chosen: the answer noted, coverage checked
                i.qas.truncate(6);
                journey::note_progress(&id, "sorted");
                journey::note_progress(&id, "asked");
                let i2 = id.clone();
                self.spawn_job_kind("Thinking about what to ask next…", &id, "next", move || {
                    std::thread::sleep(std::time::Duration::from_secs(600));
                    resolve::JobOut::Answer(i2, journey::AnswerOut::default())
                });
                self.resolve_ui.busy_since = now - 4.0;
            }
            o if o.starts_with("wait:") => {
                let secs: f64 = o[5..].parse().unwrap_or(38.0);
                i.proposal = None;
                i.proposed_from = String::new();
                journey::note_progress(&id, "first-part");
                journey::note_progress(&id, "second-part");
                let i2 = id.clone();
                self.spawn_job("Writing your plan and your validation side by side…", move || {
                    std::thread::sleep(std::time::Duration::from_secs(600));
                    resolve::JobOut::Proposed(i2, None, String::new(), Err(String::new()), vec![])
                });
                self.resolve_ui.busy_since = now - secs;
            }
            "handoff" => {
                // the 15 s hand-off: the next question from Resolve's own list
                i.qas[6].answer = "I could take on about four more cafés before I run out of hours".into();
                let mut q = journey::offline_next(i).unwrap_or_default();
                q.why = format!("Fixed question set: covers \"{}\".", journey::target_label(&q.target));
                i.qas.push(q);
                i.cursor = i.qas.len() - 1;
                i.last_error = "The AI took longer than 15 s, so this question comes from Resolve's own list.".into();
                self.resolve_ui.show_why = true;
            }
            "error" => {
                i.understanding = None;
                i.last_error = "The AI's answer didn't come back in the right shape, even after one retry (The model didn't return JSON). Try again, or pick a stronger model.".into();
            }
            "resume" => {
                self.temp.insert("force_resume".into(), "1".into());
            }
            // mid-motion frames (with --motion-at): M02 the page change Confirm → Next move
            "step" => {
                let mut old = i.clone();
                old.stage = "confirm".into();
                self.temp.insert(format!("step_key:{id}"), "proposal:0".into());
                self.temp.insert(format!("step_t:{id}"), "1".into());
                self.temp.insert(format!("step_dir:{id}"), "1".into());
                self.intake_ghost = Some((old, 1.0));
            }
            // M04.7: the last two heard items have just landed
            "land" => {
                self.temp.insert(format!("heard_open:{id}"), "1".into());
                let n = i.heard.len();
                for (k, h) in i.heard.iter().enumerate() {
                    let v = if k + 2 >= n { format!("1|{}", k + 2 - n) } else { "0|0".into() };
                    self.temp.insert(format!("heard_seen:{}", h.id), v);
                }
            }
            // M04.5: the AI's question arrives after a 6 s wait
            "arrive" => {
                let cur = i.qas.len().saturating_sub(1);
                self.temp.insert(format!("q_shown:{id}"), i.qas[cur].id.clone());
                self.temp.insert(format!("q_t:{id}"), "1".into());
                self.temp.insert(format!("q_out:{id}"), "6".into());
            }
            "edit" => i.editing = true,
            "pull" => {
                i.input = String::new();
                self.resolve_ui.show_import = true;
            }
            "template" => {
                let filled: Vec<&str> = crate::resolve::intake::EXAMPLE_INPUT.lines().take(2).collect();
                let mut s = filled.join("\n");
                for h in &widgets::INTAKE_TEMPLATE[2..] {
                    s.push_str(&format!("\n{h}: "));
                }
                i.input = s;
            }
            "blank" => {
                i.input = String::new();
            }
            "limit" => {
                // no proposal yet: the app asks for one, and the budget stop answers
                i.proposal = None;
                i.proposed_from = String::new();
            }
            _ => {}
        }
    }
}
