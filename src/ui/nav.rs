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
use eframe::egui::{self, Align2, RichText, Sense};

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
    pub icon: &'static str,
    pub target: Target,
    pub hint: &'static str,
    pub ask: AskCopy,
}

pub struct NavGroup {
    pub label: &'static str,
    /// Shows the project picker under the group label.
    pub picker: bool,
    pub items: &'static [NavItem],
}

// Ask Resolve wording. Home, Tasks and Roadmap follow resolve-docs briefs/14-N8-global-bar.md
// (Dashboard, Work, Roadmap). The others follow the same pattern; their wording is a proposal to
// confirm (the brief asks for it to be flagged, not guessed — see docs/reports).
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

pub const NAV: &[NavGroup] = &[
    NavGroup {
        label: "Workspace",
        picker: false,
        items: &[
            NavItem { label: "Home", icon: "🏠", target: Target::Home, hint: "Across all projects: captured, today, schedule", ask: ASK_HOME },
            NavItem { label: "Projects", icon: "▣", target: Target::Projects, hint: "All projects", ask: ASK_PLAIN },
            NavItem { label: "Calendar", icon: "📆", target: Target::Calendar, hint: "Events and scheduled tasks", ask: ASK_CALENDAR },
        ],
    },
    NavGroup {
        label: "Product management",
        picker: true,
        items: &[
            NavItem { label: "Overview", icon: "◎", target: Target::Sec(Section::Overview), hint: "Do this next, what matters now, project context", ask: ASK_OVERVIEW },
            NavItem { label: "Roadmap", icon: "⇶", target: Target::Sec(Section::Roadmap), hint: "Now, Next, Later — milestones, dependencies, validation gates", ask: ASK_ROADMAP },
            NavItem { label: "Tasks", icon: "☑", target: Target::Tasks, hint: "The project's tasks and board, or all tasks with Captured, Today, Upcoming and filters", ask: ASK_TASKS },
            NavItem { label: "Frameworks", icon: "⌬", target: Target::Sec(Section::Frameworks), hint: "Hypothesis Zero", ask: ASK_FRAMEWORKS },
            NavItem { label: "Knowledge", icon: "📚", target: Target::Sec(Section::Knowledge), hint: "Project brief, documents, research, cut list, Evidence & Records", ask: ASK_KNOWLEDGE },
        ],
    },
    NavGroup {
        label: "Collaboration",
        picker: false,
        items: &[
            NavItem { label: "AI Team", icon: "✦", target: Target::Sec(Section::AiTeam), hint: "Talk to product specialists", ask: ASK_TEAM },
            NavItem { label: "Handoffs", icon: "⇄", target: Target::Sec(Section::Handoffs), hint: "Briefs for Claude, Codex or people", ask: ASK_HANDOFFS },
        ],
    },
];

pub const UTILITIES: &[NavItem] = &[
    NavItem { label: "Search", icon: "🔍", target: Target::Search, hint: "Ctrl+K", ask: ASK_PLAIN },
    NavItem { label: "Settings, sync & AI", icon: "⚙", target: Target::Settings, hint: "", ask: ASK_PLAIN },
    NavItem { label: "Help & tutorial", icon: "❓", target: Target::Help, hint: "", ask: ASK_PLAIN },
];

/// Frameworks registry. Each declares its name and what it is; Hypothesis Zero is the first.
/// To add a method, add an entry here and a renderer in `sections::frameworks_view`.
pub struct Framework {
    pub id: &'static str,
    pub name: &'static str,
    pub about: &'static str,
}

pub const FRAMEWORKS: &[Framework] = &[Framework {
    id: "h0",
    name: "Hypothesis Zero",
    about: "H0 is the baseline hypothesis. Every hypothesis Hx has its explicit negation NHx; any node can have sub-hypotheses; experiments E0, E1… test any node with predictions and thresholds set in advance. Results update the hypothesis (unresolved, supported, weakened, refuted) and gate roadmap items.",
}];

/// The parts of Knowledge, as tabs on the Knowledge screen.
pub const KNOWLEDGE_TABS: &[(&str, &str)] = &[("Brief & documents", "📄"), ("Research", "🔎"), ("Cut list", "✂"), ("Evidence & Records", "◆"), ("Advanced: databases", "▦")];

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
            View::Intake(_) | View::TestLab | View::ConnectAi => return None,
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
            self.palette = Some(Palette { query: String::new(), sel: 0, focus: true });
            return;
        }
        if let Some(v) = self.target_view(t) {
            self.go(v);
        }
    }

    /// What Ask Resolve offers on the current screen.
    pub fn ask_copy(&self) -> &'static AskCopy {
        let t = self.nav_target_of(&self.view);
        NAV.iter().flat_map(|g| g.items.iter()).chain(UTILITIES.iter()).find(|i| Some(i.target) == t).map(|i| &i.ask).unwrap_or(&ASK_PLAIN)
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

    pub fn nav_sidebar(&mut self, ctx: &egui::Context, p: &Pal) {
        egui::SidePanel::left("sidebar")
            .resizable(true)
            .default_width(268.0)
            .width_range(220.0..=380.0)
            .frame(egui::Frame::none().fill(p.side).inner_margin(egui::Margin { left: 10.0, right: 10.0, top: 12.0, bottom: 8.0 }))
            .show(ctx, |ui| {
                // workspace switcher, undo / redo, light-dark
                ui.horizontal(|ui| {
                    let meta = self.store.meta.clone();
                    avatar(ui, &meta.name.chars().next().unwrap_or('W').to_string(), rgb(meta.color), 26.0);
                    let mut switch_to: Option<Id> = None;
                    ui.menu_button(RichText::new(format!("{} ⏷", meta.name)).font(fs(14.0)), |ui| {
                        ui.set_min_width(240.0);
                        ui.label(RichText::new("Workspaces — each is private").size(size::SMALL).color(p.dim));
                        for w in self.reg.workspaces.clone() {
                            if ui.selectable_label(w.id == meta.id, format!("●  {}", w.name)).clicked() {
                                switch_to = Some(w.id.clone());
                                ui.close_menu();
                            }
                        }
                        ui.separator();
                        if ui.button("+ Start something new with Resolve").on_hover_text("Runs the intake; Adopt creates a new workspace with your project as its first project").clicked() {
                            self.start_something_new();
                            ui.close_menu();
                        }
                    });
                    if let Some(id) = switch_to {
                        self.switch_workspace(&id);
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let icon = if self.reg.settings.dark { "☀" } else { "🌙" };
                        if ghost_button(ui, RichText::new(icon).size(15.0).color(p.dim)).on_hover_text("Light / dark").clicked() {
                            self.reg.settings.dark = !self.reg.settings.dark;
                            self.reg.save();
                        }
                        let redo = self.store.data.redo_target().map(|c| c.label.trim_start_matches("Undo: ").to_string());
                        let redo_tip = redo.as_ref().map(|l| format!("Redo: {l} (Ctrl+Y)")).unwrap_or_else(|| "Nothing to redo".into());
                        if ui.add_enabled(redo.is_some(), egui::Button::new(RichText::new("↷").size(16.0)).frame(false)).on_hover_text(redo_tip).on_disabled_hover_text("Nothing to redo").clicked() {
                            self.do_redo();
                        }
                        let undo = self.store.data.undo_target().map(|c| c.label.clone());
                        let undo_tip = undo.as_ref().map(|l| format!("Undo: {l} (Ctrl+Z)")).unwrap_or_else(|| "Nothing to undo".into());
                        if ui.add_enabled(undo.is_some(), egui::Button::new(RichText::new("↶").size(16.0)).frame(false)).on_hover_text(undo_tip).on_disabled_hover_text("Nothing to undo").clicked() {
                            self.do_undo();
                        }
                    });
                });
                ui.add_space(10.0);
                // the two global actions, side by side
                ui.columns(2, |cols| {
                    let (rect, resp) = cols[0].allocate_exact_size(egui::vec2(cols[0].available_width(), 34.0), Sense::click());
                    cols[0].painter().rect_filled(rect, space::RADIUS_SMALL, if resp.hovered() { p.accent.linear_multiply(0.85) } else { p.accent });
                    cols[0].painter().text(rect.center(), Align2::CENTER_CENTER, "+  Add task", fs(13.5), p.on_accent);
                    if resp.on_hover_text("Ctrl+Q").on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                        self.open_quick(QuickKind::Task);
                    }
                    let on = self.page_ui.ai_open;
                    let (rect, resp) = cols[1].allocate_exact_size(egui::vec2(cols[1].available_width(), 34.0), Sense::click());
                    cols[1].painter().rect(rect, space::RADIUS_SMALL, if on || resp.hovered() { p.accent_soft } else { p.card }, egui::Stroke::new(1.0, p.accent));
                    cols[1].painter().text(rect.center(), Align2::CENTER_CENTER, "✨  Ask AI", fs(13.5), p.text);
                    if resp.on_hover_text("The General assistant — ask or tell it to do something about what you're looking at").on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                        self.page_ui.ai_open = !self.page_ui.ai_open;
                    }
                });
                ui.add_space(6.0);
                // utilities get their own strip at the bottom, so the list above scrolls instead of overlapping them
                egui::TopBottomPanel::bottom("sidebar-bottom").frame(egui::Frame::none()).show_separator_line(true).show_inside(ui, |ui| {
                    ui.add_space(4.0);
                    for item in UTILITIES {
                        if nav_item(ui, p, item.icon, item.label, None, self.is_active(item.target), None).on_hover_text(item.hint).clicked() {
                            self.open_target(item.target);
                        }
                    }
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
                    wrap_label(ui, RichText::new(status).size(size::TINY).color(p.faint));
                });
                egui::CentralPanel::default().frame(egui::Frame::none()).show_inside(ui, |ui| {
                    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                        for g in NAV {
                            ui.add_space(6.0);
                            ui.label(RichText::new(g.label).font(fs(12.0)).color(p.faint));
                            if g.picker {
                                self.project_picker(ui, p);
                            }
                            for item in g.items {
                                let count = match item.target {
                                    Target::Tasks => Some(match &self.current_project {
                                        Some(pid) => self.store.data.tasks.iter().filter(|t| !t.completed && t.project_id.as_deref() == Some(pid.as_str()) && t.parent_id.is_none()).count(),
                                        None => self.store.data.tasks.iter().filter(|t| !t.completed && t.due.map_or(false, |x| x <= today())).count(),
                                    }),
                                    _ => None,
                                };
                                if nav_item(ui, p, item.icon, item.label, count, self.is_active(item.target), None).on_hover_text(item.hint).clicked() {
                                    self.open_target(item.target);
                                }
                            }
                        }
                        // unfinished intakes stay one click away (they used to sit under "Resolve")
                        let open = crate::ui::resolve::open_intakes(&self.store.data);
                        if !open.is_empty() {
                            ui.add_space(6.0);
                            ui.label(RichText::new("Unfinished intakes").font(fs(12.0)).color(p.faint));
                            for (id, title, step) in open.into_iter().take(4) {
                                let label = format!("{} · step {}/4", title, (step as usize + 1).min(4));
                                if nav_item(ui, p, "◌", &label, None, self.view == View::Intake(id.clone()), Some(p.accent)).on_hover_text("Pick up where you left off").clicked() {
                                    self.go(View::Intake(id));
                                }
                            }
                        }
                    });
                });
            });
    }

    fn project_picker(&mut self, ui: &mut egui::Ui, p: &Pal) {
        let cur = self.current_project.as_deref().and_then(|id| self.store.data.project(id)).map(|x| (x.name.clone(), x.color));
        let label = match &cur {
            Some((n, _)) => format!("● {n}"),
            None => "All projects".to_string(),
        };
        let col = cur.as_ref().map(|c| rgb(c.1)).unwrap_or(p.dim);
        let mut pick: Option<Option<Id>> = None;
        let mut new_project = false;
        egui::ComboBox::from_id_salt("project-context").selected_text(RichText::new(label).color(col).font(fs(13.0))).width(ui.available_width() - 8.0).show_ui(ui, |ui| {
            if ui.selectable_label(self.current_project.is_none(), "All projects").clicked() {
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
            if ui.selectable_label(false, "+ New project (opens the intake)").clicked() {
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
            ui.label(RichText::new(&pr.name).font(fserif(size::HERO)));
        });
        ui.add_space(4.0);
        hline(ui, p);
    }
}
