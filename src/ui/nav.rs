//! Navigation, defined as data. To add, rename or reorder a sidebar item, or change what Ask
//! Resolve suggests on a screen, edit the tables below; the sidebar and the Ask panel are drawn
//! from them. No view code needs to change.
//!
//! Ported from Control Centre (build-me-a-single-productivity/app/src/ui/nav.rs): the NAV /
//! UTILITIES tables, Target routing, the project picker and the sidebar layout. Changed for
//! Resolve 2: Frameworks is its own item (not a Knowledge tab), Evidence & Records lives inside
//! Knowledge, AI Team and Handoffs are separate sections, Undo/Redo use Resolve 2's change-set
//! history, and each item carries its Ask Resolve wording.

use super::theme::*;
use super::widgets::*;
use super::*;
use super::motion;
use eframe::egui::{self, RichText};

/// Product-management and collaboration sections. Each reads the selected project (or all
/// projects when none is selected).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Section {
    Overview,
    Roadmap,
    Frameworks,
    Knowledge,
    AiTeam,
    Handoffs,
    // Careers mode's own sections (Careers brief, "Shell"). Their screens come in later steps.
    Pipeline,
    MyCv,
    Stats,
    Recorder,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Target {
    Home,
    Projects,
    Calendar,
    Sec(Section),
    /// The selected project's tasks, or every task (with Captured / Today / Upcoming / Filters).
    Tasks,
    Search,
    Settings,
    Help,
}

/// What Ask Resolve offers on a screen: the input's placeholder and up to three suggestions.
pub struct AskCopy {
    pub placeholder: &'static str,
    pub suggestions: &'static [&'static str],
}

pub struct NavItem {
    pub label: &'static str,
    pub icon: Icon,
    pub target: Target,
    pub hint: &'static str,
    pub ask: AskCopy,
}

pub struct NavGroup {
    /// Shown in capitals (WORKSPACE, YOUR PROJECT, COLLABORATION).
    pub label: &'static str,
    /// Shows the project picker under the group label.
    pub picker: bool,
    pub items: &'static [NavItem],
}

// Ask Resolve wording, from the build brief (7 Nov 2026, "The shell"). Home, Tasks, Projects and
// Search keep the wording they had; AI Team has no bar (its page has its own chat box).
const ASK_HOME: AskCopy = AskCopy { placeholder: "Ask Resolve, or capture a thought", suggestions: &["What should I focus on right now?", "Why is my most important item blocked?", "Capture a quick note"] };
const ASK_TASKS: AskCopy = AskCopy { placeholder: "Add a task, or ask what's next", suggestions: &["Add a task to Now", "What's waiting on someone else?", "Clear my captured inbox"] };
const ASK_ROADMAP: AskCopy = AskCopy { placeholder: "Ask about your roadmap", suggestions: &["What's missing from this plan?", "Move a theme into Now", "Why is this sequenced this way?"] };
const ASK_OVERVIEW: AskCopy = AskCopy { placeholder: "Ask about this project", suggestions: &["What matters most this week?", "What's still uncertain?", "Summarise where this project is"] };
const ASK_FRAMEWORKS: AskCopy = AskCopy { placeholder: "Ask about your hypotheses and tests", suggestions: &["Which hypothesis is weakest?", "Suggest a cheap test for H0", "What would change my mind?"] };
const ASK_KNOWLEDGE: AskCopy = AskCopy { placeholder: "Find or capture something you know", suggestions: &["What evidence supports H0?", "Summarise the project brief", "Capture a research note"] };
const ASK_TEAM: AskCopy = AskCopy { placeholder: "Ask the General assistant (the team chat is on this page)", suggestions: &[] };
const ASK_HANDOFFS: AskCopy = AskCopy { placeholder: "Ask about a handoff", suggestions: &["What should the next handoff cover?", "Is anything blocking a handoff?"] };
const ASK_PLAIN: AskCopy = AskCopy { placeholder: "Ask Resolve, or capture a thought", suggestions: &["What should I focus on right now?", "Capture a quick note"] };
const ASK_CALENDAR: AskCopy = AskCopy { placeholder: "Ask about your schedule", suggestions: &["Plan my day", "What's overdue?", "Find time for my next task"] };

// Ask Resolve wording in Careers mode (Careers brief, "Ask Resolve wording (Careers)").
const ASK_C_HOME: AskCopy = AskCopy { placeholder: "Add a job, or ask what's next", suggestions: &["Review this job ad", "What should I apply to today?", "Start my daily hunt"] };
const ASK_C_PIPELINE: AskCopy = AskCopy { placeholder: "Ask about your applications", suggestions: &["Who haven't I heard back from?", "What closes this week?", "Move a role to Interview"] };
const ASK_C_CV: AskCopy = AskCopy { placeholder: "Ask about your CV", suggestions: &["What's weakest on my CV?", "Rebuild my CV for this job", "Which proof points need a number?"] };
const ASK_C_STATS: AskCopy = AskCopy { placeholder: "Ask about your numbers", suggestions: &["Where am I dropping off?", "Am I on target this week?", "What should I change?"] };
const ASK_C_TASKS: AskCopy = AskCopy { placeholder: "Add a task, or ask what's next", suggestions: &["Add a follow-up", "What's waiting to hear back?", "Clear my captured inbox"] };
const ASK_C_RECORDER: AskCopy = AskCopy { placeholder: "Ask about your interviews", suggestions: &["What did I struggle with last time?", "Drill me on my weak spots", "Prep me for my next interview"] };
const ASK_C_FRAMEWORKS: AskCopy = AskCopy { placeholder: "Ask about your gates and templates", suggestions: &["Is my target too broad?", "Tighten my gates", "Update my cover letter template"] };
const ASK_C_KNOWLEDGE: AskCopy = AskCopy { placeholder: "Find or capture something about your search", suggestions: &["What proof do I have for this skill?", "Add a proof point", "Which CV did I send here?"] };
const ASK_C_CALENDAR: AskCopy = AskCopy { placeholder: "Ask about your schedule", suggestions: &["Plan my day", "Which interviews are coming up?", "Which closing dates are this week?"] };

/// The sidebar groups in this mode (Careers brief, "Modes"): only the middle group and the Ask
/// wording differ between PM and Careers.
pub fn nav(mode: Mode) -> &'static [NavGroup] {
    match mode {
        Mode::Pm => NAV,
        Mode::Careers => NAV_CAREERS,
    }
}

/// Careers mode's sidebar (Careers brief, "Sidebar"): YOUR SEARCH replaces YOUR PROJECT, with the
/// search picker under it. Icons reuse the shell's set until the Careers screens are built.
pub const NAV_CAREERS: &[NavGroup] = &[
    NavGroup {
        label: "Workspace",
        picker: false,
        items: &[
            NavItem { label: "Home", icon: Icon::Home, target: Target::Home, hint: "Today, what needs a check, your daily hunt", ask: ASK_C_HOME },
            NavItem { label: "Projects", icon: Icon::Projects, target: Target::Projects, hint: "All projects", ask: ASK_PLAIN },
            NavItem { label: "Calendar", icon: Icon::Calendar, target: Target::Calendar, hint: "Interviews, follow-ups and closing dates", ask: ASK_C_CALENDAR },
        ],
    },
    NavGroup {
        label: "Your search",
        picker: true,
        items: &[
            NavItem { label: "Pipeline", icon: Icon::Roadmap, target: Target::Sec(Section::Pipeline), hint: "Every role, by stage", ask: ASK_C_PIPELINE },
            NavItem { label: "My CV", icon: Icon::Doc, target: Target::Sec(Section::MyCv), hint: "Your master CV, CV and cover letter templates, LinkedIn fixes", ask: ASK_C_CV },
            NavItem { label: "Stats", icon: Icon::Overview, target: Target::Sec(Section::Stats), hint: "Where you're dropping off, and against your target", ask: ASK_C_STATS },
            NavItem { label: "Tasks", icon: Icon::Tasks, target: Target::Tasks, hint: "Follow-ups and everything else to do", ask: ASK_C_TASKS },
            NavItem { label: "Interview recorder", icon: Icon::Resume, target: Target::Sec(Section::Recorder), hint: "Record an interview and see how each answer went", ask: ASK_C_RECORDER },
            NavItem { label: "Frameworks", icon: Icon::Frameworks, target: Target::Sec(Section::Frameworks), hint: "Gates, interview prep, offers and the playbook", ask: ASK_C_FRAMEWORKS },
            NavItem { label: "Knowledge base", icon: Icon::Knowledge, target: Target::Sec(Section::Knowledge), hint: "CVs, evidence bank, LinkedIn, job ads, interview notes, offers", ask: ASK_C_KNOWLEDGE },
        ],
    },
    NavGroup {
        label: "Collaboration",
        picker: false,
        items: &[
            NavItem { label: "AI Team", icon: Icon::Team, target: Target::Sec(Section::AiTeam), hint: "Scout, Reviewer, Tailor, Coach and Negotiator", ask: ASK_TEAM },
            NavItem { label: "Handoffs", icon: Icon::Handoffs, target: Target::Sec(Section::Handoffs), hint: "Briefs for Claude, Codex or people", ask: ASK_HANDOFFS },
        ],
    },
];

/// PM mode's sidebar.
pub const NAV: &[NavGroup] = &[
    NavGroup {
        label: "Workspace",
        picker: false,
        items: &[
            NavItem { label: "Home", icon: Icon::Home, target: Target::Home, hint: "Across all projects: captured, today, schedule", ask: ASK_HOME },
            NavItem { label: "Projects", icon: Icon::Projects, target: Target::Projects, hint: "All projects", ask: ASK_PLAIN },
            NavItem { label: "Calendar", icon: Icon::Calendar, target: Target::Calendar, hint: "Events and scheduled tasks", ask: ASK_CALENDAR },
        ],
    },
    NavGroup {
        label: "Your project",
        picker: true,
        items: &[
            NavItem { label: "Overview", icon: Icon::Overview, target: Target::Sec(Section::Overview), hint: "Do this next, what matters now, project context", ask: ASK_OVERVIEW },
            NavItem { label: "Roadmap", icon: Icon::Roadmap, target: Target::Sec(Section::Roadmap), hint: "Now, Next, Later: milestones, dependencies, validation gates", ask: ASK_ROADMAP },
            NavItem { label: "Tasks", icon: Icon::Tasks, target: Target::Tasks, hint: "The project's tasks and board, or all tasks with Captured, Today, Upcoming and filters", ask: ASK_TASKS },
            NavItem { label: "Frameworks", icon: Icon::Frameworks, target: Target::Sec(Section::Frameworks), hint: "Hypothesis Zero", ask: ASK_FRAMEWORKS },
            NavItem { label: "Knowledge base", icon: Icon::Knowledge, target: Target::Sec(Section::Knowledge), hint: "Project brief, documents, research, cut list, Evidence & Records", ask: ASK_KNOWLEDGE },
        ],
    },
    NavGroup {
        label: "Collaboration",
        picker: false,
        items: &[
            NavItem { label: "AI Team", icon: Icon::Team, target: Target::Sec(Section::AiTeam), hint: "Talk to product specialists", ask: ASK_TEAM },
            NavItem { label: "Handoffs", icon: Icon::Handoffs, target: Target::Sec(Section::Handoffs), hint: "Briefs for Claude, Codex or people", ask: ASK_HANDOFFS },
        ],
    },
];

/// The sidebar footer. Search shows its shortcut on the right.
pub const UTILITIES: &[NavItem] = &[
    NavItem { label: "Search", icon: Icon::Search, target: Target::Search, hint: "Ctrl K", ask: ASK_PLAIN },
    NavItem { label: "Settings", icon: Icon::Settings, target: Target::Settings, hint: "", ask: ASK_PLAIN },
    NavItem { label: "Help", icon: Icon::Help, target: Target::Help, hint: "", ask: ASK_PLAIN },
];

/// The screens without the Ask Resolve bar (AI Team has its own chat box, decision C2).
pub fn shows_ask_bar(t: Option<Target>) -> bool {
    t != Some(Target::Sec(Section::AiTeam))
}

/// Sidebar collapse (S3): collapsing, labels fade 120 e-std, then the width goes 248 → 64 in
/// 320 e-out; expanding runs the other way (width first, then labels). Returns (width, label
/// opacity) `elapsed_ms` after the toggle.
pub fn sidebar_frame(elapsed_ms: f32, collapsing: bool, reduced: bool) -> (f32, f32) {
    let (full, small) = (space::SIDEBAR, space::SIDEBAR_COLLAPSED);
    let w_tm = if collapsing { moves::SIDEBAR_WIDTH } else { motion::Timing { delay: 0.0, ..moves::SIDEBAR_WIDTH } };
    // with reduced motion nothing waits: the width jumps and the labels fade at once
    let l_tm = if collapsing || reduced { moves::SIDEBAR_LABELS } else { motion::Timing { delay: moves::SIDEBAR_WIDTH.ms, ..moves::SIDEBAR_LABELS } };
    let w = motion::frame_at(elapsed_ms, w_tm, reduced);
    let l = motion::frame_at(elapsed_ms, l_tm, reduced);
    if collapsing {
        (w.lerp(full, small), if reduced { 1.0 - l.alpha } else { 1.0 - l.t })
    } else {
        (w.lerp(small, full), if reduced { l.alpha } else { l.t })
    }
}
/// Frameworks registry. Each declares its name and what it is; Hypothesis Zero is the first.
/// To add a method, add an entry here and a renderer in `sections::frameworks_view`.
/// `read_by` names who reads it (Careers: "Read by Reviewer"); empty in PM.
pub struct Framework {
    pub id: &'static str,
    pub name: &'static str,
    pub about: &'static str,
    pub read_by: &'static str,
}

/// The Frameworks registry in this mode.
pub fn frameworks(mode: Mode) -> &'static [Framework] {
    match mode {
        Mode::Pm => FRAMEWORKS,
        Mode::Careers => FRAMEWORKS_CAREERS,
    }
}

/// Careers mode's registry, in order (Careers brief, "Frameworks"); it replaces Hypothesis Zero.
/// CV templates and the cover letter template live in My CV only (decided 10 Nov 2026): My CV is
/// the one home for the master CV, CV templates, the cover letter template and LinkedIn fixes.
pub const FRAMEWORKS_CAREERS: &[Framework] = &[
    Framework { id: "gates", name: "Gates", about: "A list of rules, each a Hard gate or a Preference, with why it matters. Set by intake. A hard gate fails the job. A preference only lowers its score.", read_by: "Reviewer" },
    Framework { id: "interview-prep", name: "Interview prep", about: "The Cockpit sheet (running order, key numbers, answers on pay and logistics, likely objections, questions to ask), the 90-day plan, the six-month plan and technical-depth drills.", read_by: "Coach" },
    Framework { id: "offers", name: "Offers and negotiation", about: "Offer comparison, market rate check and the counter-offer plan.", read_by: "Negotiator" },
    Framework { id: "playbook", name: "Improvement playbook", about: "For each weak stage: likely causes, the fix, and which role owns it.", read_by: "Stats" },
    Framework { id: "advice", name: "Advice", about: "Short practical guides.", read_by: "Everyone" },
];

/// PM mode's registry.
pub const FRAMEWORKS: &[Framework] = &[Framework {
    id: "h0",
    read_by: "",
    name: "Hypothesis Zero",
    about: "H0 is the baseline hypothesis. Every hypothesis Hx has its explicit negation NHx; any node can have sub-hypotheses; experiments E0, E1… test any node with predictions and thresholds set in advance. Results update the hypothesis (unresolved, supported, weakened, refuted) and gate roadmap items.",
}];

/// The Settings tabs (C-E21, Pack C "Settings"), in order. `--view settings-<n>` opens one.
pub const SETTINGS_TABS: [&str; 4] = ["AI and MCP", "Sync and phone", "Workspaces", "About"];

/// The Knowledge base tabs in this mode.
pub fn knowledge_tabs(mode: Mode) -> &'static [(&'static str, &'static str)] {
    match mode {
        Mode::Pm => KNOWLEDGE_TABS,
        Mode::Careers => KNOWLEDGE_TABS_CAREERS,
    }
}

/// Careers mode's Knowledge base tabs (Careers brief, "Knowledge base"). "CVs and cover letters"
/// holds the copies sent with each application; the master CV and templates are in My CV.
pub const KNOWLEDGE_TABS_CAREERS: &[(&str, &str)] = &[("CVs and cover letters", "📄"), ("Evidence bank", "◆"), ("LinkedIn", "📄"), ("Job ads", "📄"), ("Interview notes", "📄"), ("Offers", "📄"), ("Advanced: databases", "▦")];

/// PM mode's Knowledge base tabs, as tabs on the Knowledge screen.
pub const KNOWLEDGE_TABS: &[(&str, &str)] = &[("Brief and documents", "📄"), ("Research", "🔎"), ("Cut list", "✂"), ("Evidence and Records", "◆"), ("Advanced: databases", "▦")];

impl App {
    /// The nav item a view belongs to (for highlighting and Ask Resolve wording).
    pub fn nav_target_of(&self, v: &View) -> Option<Target> {
        Some(match v {
            View::Dashboard => Target::Home,
            View::Projects => Target::Projects,
            View::Calendar => Target::Calendar,
            View::Sec(s) => Target::Sec(*s),
            View::Resolve(_) => Target::Sec(Section::Overview),
            View::Project(_) | View::Inbox | View::Today | View::Upcoming | View::Filters => Target::Tasks,
            View::Page(_) | View::Database(_) => Target::Sec(Section::Knowledge),
            View::Settings => Target::Settings,
            View::Help => Target::Help,
            View::Intake(_) | View::TestLab | View::ConnectAi | View::Gallery => return None,
        })
    }

    pub fn target_view(&self, t: Target) -> Option<View> {
        Some(match t {
            Target::Home => View::Dashboard,
            Target::Projects => View::Projects,
            Target::Calendar => View::Calendar,
            Target::Sec(s) => View::Sec(s),
            Target::Tasks => match &self.current_project {
                Some(p) => View::Project(p.clone()),
                None => View::Today,
            },
            Target::Settings => View::Settings,
            Target::Help => View::Help,
            Target::Search => return None,
        })
    }

    pub fn is_active(&self, t: Target) -> bool {
        self.nav_target_of(&self.view) == Some(t)
    }

    pub fn open_target(&mut self, t: Target) {
        if t == Target::Search {
            self.palette = Some(Palette { query: String::new(), sel: 0, focus: true, opened: 0.0 });
            return;
        }
        if let Some(v) = self.target_view(t) {
            self.go(v);
        }
    }

    /// The open workspace's mode: picks every per-mode table.
    pub fn mode(&self) -> Mode {
        self.store.data.mode
    }

    /// What Ask Resolve offers on the current screen.
    pub fn ask_copy(&self) -> &'static AskCopy {
        let t = self.nav_target_of(&self.view);
        nav(self.mode()).iter().flat_map(|g| g.items.iter()).chain(UTILITIES.iter()).find(|i| Some(i.target) == t).map(|i| &i.ask).unwrap_or(&ASK_PLAIN)
    }

    /// Pick which project the product-management sections show.
    pub fn select_project(&mut self, pid: Option<Id>) {
        self.current_project = pid;
        self.resolve_ui.sel_record = None;
        // stay in the same section, now for the new project
        if matches!(self.view, View::Project(_) | View::Inbox | View::Today | View::Upcoming | View::Filters | View::Resolve(_)) {
            self.view = match (&self.current_project, &self.view) {
                (_, View::Resolve(_)) => View::Sec(Section::Overview),
                (Some(p), _) => View::Project(p.clone()),
                (None, _) => View::Today,
            };
        }
    }

    /// Keep the registry's mode for the open workspace in step with its data (the switcher reads it).
    pub fn sync_meta_mode(&mut self) {
        let m = self.store.data.mode;
        if self.store.meta.mode == m {
            return;
        }
        self.store.meta.mode = m;
        let id = self.store.meta.id.clone();
        if let Some(w) = self.reg.workspaces.iter_mut().find(|w| w.id == id) {
            w.mode = m;
        }
        if !self.no_save {
            self.reg.save();
        }
    }

    pub fn do_undo(&mut self) {
        let Some(cs) = self.store.data.undo_target().map(|c| c.id.clone()) else {
            self.toast("Nothing to undo");
            return;
        };
        self.undo(&cs);
    }

    pub fn do_redo(&mut self) {
        match self.store.data.redo_last() {
            Ok((label, _)) => {
                self.touch();
                self.toast(format!("Redone: {label}"));
            }
            Err(e) => self.toast(e),
        }
    }

    /// Collapse or expand the sidebar (« or Ctrl \), remembered per workspace.
    pub fn toggle_sidebar(&mut self) {
        let collapsing = !self.store.settings.sidebar_collapsed;
        self.store.settings.sidebar_collapsed = collapsing;
        if !self.no_save {
            self.store.save_settings();
        }
        self.sidebar_anim = Some((self.now, collapsing));
    }

    pub fn nav_sidebar(&mut self, ctx: &egui::Context, _p: &Pal) {
        // the sidebar is nav-bg in light and dark alike: it draws with the dark palette
        let dp = sidebar_pal();
        let p = &dp;
        let collapsed = self.store.settings.sidebar_collapsed;
        let (width, labels) = match self.sidebar_anim {
            Some((t0, collapsing)) => {
                let (w, l) = sidebar_frame(((self.now - t0) * 1000.0) as f32, collapsing, motion::reduced());
                if (w - if collapsing { space::SIDEBAR_COLLAPSED } else { space::SIDEBAR }).abs() < 0.5 && (l - if collapsing { 0.0 } else { 1.0 }).abs() < 0.01 {
                    self.sidebar_anim = None;
                } else {
                    ctx.request_repaint();
                }
                (w, l)
            }
            None if collapsed => (space::SIDEBAR_COLLAPSED, 0.0),
            None => (space::SIDEBAR, 1.0),
        };
        // icons only once the labels have gone and the width has (nearly) arrived
        let icons_only = labels < 0.01 && width < space::SIDEBAR_COLLAPSED + space::S24;
        egui::SidePanel::left("sidebar")
            .resizable(false)
            .exact_width(width)
            .show_separator_line(false)
            .frame(egui::Frame::none().fill(tok::NAV_BG).inner_margin(egui::Margin { left: space::NAV_PAD, right: space::NAV_PAD, top: space::S12, bottom: space::S8 }))
            .show(ctx, |ui| {
                ui.style_mut().visuals = visuals(p);
                ui.spacing_mut().item_spacing.y = space::NAV_GAP;
                self.sidebar_top(ui, p, labels, icons_only);
                ui.add_space(space::S12);
                // Add task and Ask AI: side by side, or stacked icons when collapsed
                let mut add = false;
                let mut ask = false;
                let ask_on = self.page_ui.ai_open;
                if icons_only {
                    add = side_button(ui, true, Icon::Plus, "Add task", 0.0, false).clicked();
                    ui.add_space(space::S4);
                    let r = side_button(ui, false, Icon::Spark, "Ask AI", 0.0, ask_on);
                    side_tip(ui, &r, "Ask AI");
                    ask = r.clicked();
                } else {
                    ui.columns(2, |cols| {
                        add = side_button(&mut cols[0], true, Icon::Plus, "Add task", labels, false).on_hover_text("Ctrl+Q").clicked();
                        ask = side_button(&mut cols[1], false, Icon::Spark, "Ask AI", labels, ask_on).on_hover_text("The General assistant. Ask or tell it to do something about what you're looking at").clicked();
                    });
                }
                if add {
                    self.open_quick(QuickKind::Task);
                }
                if ask {
                    self.page_ui.ai_open = !self.page_ui.ai_open;
                }
                // the footer gets its own strip at the bottom, so the groups above scroll instead of overlapping it
                egui::TopBottomPanel::bottom("sidebar-bottom").frame(egui::Frame::none()).show_separator_line(false).show_inside(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = space::NAV_GAP;
                    side_divider(ui);
                    // Continue setting up… (B-E2): only while an intake is unfinished
                    if let Some((id, _, step)) = crate::ui::resolve::open_intakes(&self.store.data).into_iter().next() {
                        let line = format!("New project · step {step} of 4");
                        let r = setup_card(ui, "Continue setting up…", &line, labels);
                        if icons_only {
                            side_tip(ui, &r, &format!("Continue setting up… {line}"));
                        }
                        if r.clicked() {
                            self.go(View::Intake(id));
                        }
                        ui.add_space(space::S4);
                    }
                    for item in UTILITIES {
                        let right = (!item.hint.is_empty()).then(|| item.hint.to_string());
                        let r = side_item(ui, item.icon, item.label, right, self.is_active(item.target), labels);
                        if icons_only {
                            side_tip(ui, &r, item.label);
                        }
                        if r.clicked() {
                            self.open_target(item.target);
                        }
                    }
                    ui.add_space(space::S6);
                    let (status, ok) = if let Some(e) = &self.store.last_error {
                        (format!("Couldn't save: {e}"), false)
                    } else if self.dirty {
                        ("Saving…".into(), true)
                    } else if !self.hub_status.is_empty() {
                        (format!("Saved on this PC · {}", self.hub_status), true)
                    } else if !self.sync_status.is_empty() {
                        (format!("Saved on this PC · {}", self.sync_status), true)
                    } else {
                        ("Saved on this PC · works offline".into(), true)
                    };
                    let r = side_status(ui, &status, ok, labels);
                    if icons_only {
                        side_tip(ui, &r, &status);
                    }
                    // which app this is: a preview names its branch and version
                    if let Some(b) = crate::store::PREVIEW {
                        let line = format!("Preview: {b}, {}", env!("CARGO_PKG_VERSION"));
                        let r = preview_note(ui, &line, labels);
                        if icons_only {
                            side_tip(ui, &r, &line);
                        }
                    }
                });
                egui::CentralPanel::default().frame(egui::Frame::none()).show_inside(ui, |ui| {
                    egui::ScrollArea::vertical().auto_shrink([false, false]).scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded).show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = space::NAV_GAP;
                        for g in nav(self.mode()) {
                            side_group(ui, g.label, labels);
                            if g.picker {
                                self.project_picker(ui, p, labels, icons_only);
                                ui.add_space(space::S4);
                            }
                            for item in g.items {
                                let count = match item.target {
                                    Target::Tasks => Some(match &self.current_project {
                                        Some(pid) => self.store.data.tasks.iter().filter(|t| !t.completed && t.project_id.as_deref() == Some(pid.as_str()) && t.parent_id.is_none()).count(),
                                        None => self.store.data.tasks.iter().filter(|t| !t.completed && t.due.map_or(false, |x| x <= today())).count(),
                                    }),
                                    _ => None,
                                };
                                let r = side_item(ui, item.icon, item.label, count.filter(|c| *c > 0).map(|c| c.to_string()), self.is_active(item.target), labels);
                                let r = if icons_only {
                                    side_tip(ui, &r, item.label);
                                    r
                                } else {
                                    r.on_hover_text(item.hint)
                                };
                                if r.clicked() {
                                    self.open_target(item.target);
                                }
                            }
                        }
                    });
                });
            });
    }

    /// Workspace switcher (amber tile, name, ▾), Undo, Redo and collapse; collapsed, the tile
    /// with the expand button under it.
    fn sidebar_top(&mut self, ui: &mut egui::Ui, p: &Pal, labels: f32, icons_only: bool) {
        let meta = self.store.meta.clone();
        let initial = meta.name.chars().next().unwrap_or('W').to_uppercase().to_string();
        let popup = egui::Id::new("workspace-switcher");
        let resp = if icons_only {
            let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), space::NAV_ITEM_H), egui::Sense::click());
            ws_tile(ui.painter(), egui::Rect::from_center_size(rect.center(), egui::vec2(space::NAV_TILE, space::NAV_TILE)), &initial);
            side_tip(ui, &resp, &meta.name);
            resp
        } else {
            let mut resp = None;
            ui.horizontal(|ui| {
                let right_w = (space::NAV_ICON + space::S10) * 3.0 + space::S4 * 2.0;
                let w = (ui.available_width() - right_w).max(space::NAV_TILE);
                let (rect, r) = ui.allocate_exact_size(egui::vec2(w, space::NAV_ITEM_H), egui::Sense::click());
                if r.hovered() {
                    ui.painter().rect_filled(rect, radius::NAV_ITEM, tok::PANE_RAISED);
                }
                let tile = egui::Rect::from_min_size(rect.left_center() - egui::vec2(0.0, space::NAV_TILE / 2.0), egui::vec2(space::NAV_TILE, space::NAV_TILE));
                ws_tile(ui.painter(), tile, &initial);
                // a Careers workspace shows its mode beside the name (Careers brief, "Sidebar")
                let careers = self.store.data.mode == Mode::Careers;
                let chip_w = if careers { space::S40 * 1.6 } else { 0.0 };
                let room = rect.width() - space::NAV_TILE - space::S8 - space::S18 - chip_w;
                let name = fit_text(ui.painter(), &meta.name, fs(size::BODY), room);
                let g = ui.painter().layout_no_wrap(name, fs(size::BODY), tok::TEXT_ON_DARK.gamma_multiply(labels));
                let gx = tile.right() + space::S8;
                let mut gw = g.size().x;
                ui.painter().galley(egui::pos2(gx, rect.center().y - g.size().y / 2.0), g, tok::TEXT_ON_DARK);
                if careers {
                    gw += space::S6 + mode_chip_at(ui.painter(), egui::pos2(gx + gw + space::S6, rect.center().y), Mode::Careers.label(), labels);
                }
                paint_icon(ui.painter(), egui::pos2(gx + gw + space::S10, rect.center().y), Icon::Down, tok::TEXT_ON_DARK_SOFT.gamma_multiply(labels));
                resp = Some(r.on_hover_cursor(egui::CursorIcon::PointingHand));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = space::S4;
                    if icon_button(ui, Icon::Collapse, true, true).on_hover_text("Collapse the sidebar (Ctrl+\\)").clicked() {
                        self.toggle_sidebar();
                    }
                    let redo = self.store.data.redo_target().map(|c| c.label.trim_start_matches("Undo: ").to_string());
                    let redo_tip = redo.as_ref().map(|l| format!("Redo: {l} (Ctrl+Y)")).unwrap_or_else(|| "Nothing to redo".into());
                    if icon_button(ui, Icon::Redo, redo.is_some(), true).on_hover_text(redo_tip).clicked() {
                        self.do_redo();
                    }
                    let undo = self.store.data.undo_target().map(|c| c.label.clone());
                    let undo_tip = undo.as_ref().map(|l| format!("Undo: {l} (Ctrl+Z)")).unwrap_or_else(|| "Nothing to undo".into());
                    if icon_button(ui, Icon::Undo, undo.is_some(), true).on_hover_text(undo_tip).clicked() {
                        self.do_undo();
                    }
                });
            });
            resp.unwrap()
        };
        if resp.clicked() {
            ui.memory_mut(|m| m.toggle_popup(popup));
        }
        let mut switch_to: Option<Id> = None;
        let mut start_new = false;
        egui::popup_below_widget(ui, popup, &resp, egui::PopupCloseBehavior::CloseOnClick, |ui| {
            ui.set_min_width(space::SIDEBAR - space::NAV_PAD * 2.0);
            ui.label(RichText::new("Workspaces (each is private)").size(size::SMALL).color(p.dim));
            // each row shows its mode (the open one reads its own data, the rest the registry)
            let here = self.store.data.mode;
            for w in self.reg.workspaces.clone() {
                let mode = if w.id == meta.id { here } else { w.mode };
                ui.horizontal(|ui| {
                    if ui.selectable_label(w.id == meta.id, format!("●  {}", w.name)).clicked() {
                        switch_to = Some(w.id.clone());
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        mode_tag(ui, p, mode);
                    });
                });
            }
            ui.separator();
            if ui.button("+ Start something new with Resolve").on_hover_text("Runs the intake; Adopt creates a new workspace with your project as its first project").clicked() {
                start_new = true;
            }
        });
        if let Some(id) = switch_to {
            self.switch_workspace(&id);
        }
        if start_new {
            self.start_something_new();
        }
        if icons_only {
            let (rect, r) = ui.allocate_exact_size(egui::vec2(ui.available_width(), space::NAV_ICON + space::S10), egui::Sense::hover());
            let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect).layout(Layout::top_down(Align::Center)));
            let b = icon_button(&mut child, Icon::Expand, true, true);
            side_tip(ui, &b, "Expand the sidebar (Ctrl+\\)");
            if b.clicked() {
                self.toggle_sidebar();
            }
            let _ = r;
        }
    }

    fn project_picker(&mut self, ui: &mut egui::Ui, p: &Pal, labels: f32, icons_only: bool) {
        let cur = self.current_project.as_deref().and_then(|id| self.store.data.project(id)).map(|x| (x.name.clone(), x.color));
        // Careers: the picker chooses the search
        let careers = self.store.data.mode == Mode::Careers;
        let (all, new_one) = if careers { ("All searches", "+ New search (opens the intake)") } else { ("All projects", "+ New project (opens the intake)") };
        let (name, dot) = match &cur {
            Some((n, c)) => (n.clone(), rgb(*c)),
            None => (all.to_string(), tok::TEXT_ON_DARK_MUTED),
        };
        let resp = picker_row(ui, dot, &name, labels);
        if icons_only {
            side_tip(ui, &resp, &name);
        }
        let popup = egui::Id::new("project-picker");
        if resp.clicked() {
            ui.memory_mut(|m| m.toggle_popup(popup));
        }
        let mut pick: Option<Option<Id>> = None;
        let mut new_project = false;
        egui::popup_below_widget(ui, popup, &resp, egui::PopupCloseBehavior::CloseOnClick, |ui| {
            ui.set_min_width(space::SIDEBAR - space::NAV_PAD * 2.0);
            if ui.selectable_label(self.current_project.is_none(), all).clicked() {
                pick = Some(None);
            }
            let mut ps: Vec<&Project> = self.store.data.projects.iter().filter(|x| !x.archived).collect();
            ps.sort_by_key(|x| x.order);
            for pr in ps {
                if ui.selectable_label(self.current_project.as_deref() == Some(pr.id.as_str()), RichText::new(format!("● {}", pr.name)).color(rgb(pr.color))).clicked() {
                    pick = Some(Some(pr.id.clone()));
                }
            }
            ui.separator();
            if ui.selectable_label(false, RichText::new(new_one).color(p.text)).clicked() {
                new_project = true;
            }
        });
        if let Some(pid) = pick {
            self.select_project(pid);
        }
        if new_project {
            self.start_intake();
        }
    }
    /// Project name and a one-line summary, at the top of every project section.
    pub fn project_header(&mut self, ui: &mut egui::Ui, p: &Pal, section: &str) {
        let Some(pid) = self.current_project.clone() else { return };
        let Some(pr) = self.store.data.project(&pid).cloned() else {
            self.current_project = None;
            return;
        };
        ui.add_space(12.0);
        ui.label(RichText::new(section).size(size::SMALL).color(p.eyebrow));
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("●").color(rgb(pr.color)).size(18.0));
            ui.label(RichText::new(&pr.name).font(fserif(size::STEP_TITLE)));
        });
        ui.add_space(4.0);
        hline(ui, p);
    }
}
