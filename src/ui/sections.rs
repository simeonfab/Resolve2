//! Product-management sections: the dispatcher, Projects, Overview and Roadmap. Every section
//! reads the same project data, scoped to the project picked in the sidebar or to all projects.
//! (Frameworks and Knowledge are in knowledge.rs; AI Team and Handoffs in team.rs.)
//!
//! Ported from Control Centre (build-me-a-single-productivity/app/src/ui/sections.rs):
//! `section_view` and `projects_view`. Changed for Resolve 2: Overview reuses Resolve 2's own
//! project screen (cockpit.rs); the Roadmap is built on Resolve 2's roadmap records
//! (Kind::Roadmap), milestones, dependencies, hypotheses (validation gates) and project sections.

use super::nav::Section;
use super::theme::*;
use super::widgets::*;
use super::*;
use crate::resolve::{Kind, ProjectContext, Record};
use eframe::egui::{self, RichText, Sense, Stroke};

pub(super) const LANES: [&str; 3] = ["Now", "Next", "Later"];

pub(super) fn lane_of(r: &Record) -> &str {
    match r.field("phase") {
        "Next" => "Next",
        "Later" => "Later",
        _ => "Now",
    }
}

/// What a project card on the Projects screen shows (B-E7).
#[derive(Clone, Debug, Default)]
pub struct ProjectCardModel {
    pub id: Id,
    pub name: String,
    pub color: [u8; 3],
    pub outcome: String,
    pub next: String,
    pub frac: f32,
    /// "1 of 5 tasks"
    pub done: String,
    /// "2 Now", "2 Next", "2 Later", "8 open questions" (empty when zero)
    pub now: String,
    pub next_n: String,
    pub later: String,
    pub open: String,
    /// "Updated today", "Updated yesterday", "Updated Tuesday", "Updated 12 Oct"
    pub updated: String,
}

/// A project's next move: the intake's first move, else its pinned task, else what the
/// assessment says comes next.
pub fn project_next_move(d: &WorkspaceData, pid: &str) -> String {
    let first = d.intakes.iter().find(|i| i.project_id.as_deref() == Some(pid)).and_then(|i| i.proposal.as_ref()).map(|p| p.next_move.trim().to_string()).filter(|s| !s.is_empty());
    first
        .or_else(|| {
            let c = d.context_for(pid)?;
            let a = crate::resolve::assess::assess(d, c);
            a.win.as_ref().and_then(|w| d.task(w)).filter(|t| !t.completed).map(|t| t.title.clone()).or_else(|| a.next.first().map(|x| x.0.clone()))
        })
        .unwrap_or_default()
}

pub fn project_card_model(d: &WorkspaceData, pid: &str, real_ms: i64, now: NaiveDateTime) -> ProjectCardModel {
    let Some(pr) = d.project(pid) else { return ProjectCardModel::default() };
    let tasks: Vec<&Task> = d.tasks.iter().filter(|t| t.project_id.as_deref() == Some(pid) && t.parent_id.is_none()).collect();
    let done = tasks.iter().filter(|t| t.completed).count();
    let lane = |l: &str| d.records.iter().filter(|r| r.kind == Kind::Roadmap && r.project_id.as_deref() == Some(pid) && r.status != "Dropped" && lane_of(r) == l).count();
    let open = d.records.iter().filter(|r| r.project_id.as_deref() == Some(pid) && matches!(r.kind, Kind::Question | Kind::Hypothesis) && r.is_open()).count();
    let pill = |n: usize, word: &str| if n == 0 { String::new() } else { format!("{n} {word}") };
    let latest = tasks.iter().map(|t| t.updated).chain(d.records.iter().filter(|r| r.project_id.as_deref() == Some(pid)).map(|r| r.updated)).chain(d.context_for(pid).map(|c| c.updated)).chain([pr.updated]).max().unwrap_or(pr.updated);
    let days = ((real_ms - latest).max(0) / (24 * 3600 * 1000)) as i64;
    let updated = match days {
        0 => "Updated today".to_string(),
        1 => "Updated yesterday".to_string(),
        2..=6 => format!("Updated {}", (now.date() - chrono::Duration::days(days)).format("%A")),
        _ => format!("Updated {}", (now.date() - chrono::Duration::days(days)).format("%-d %b")),
    };
    ProjectCardModel {
        id: pid.to_string(),
        name: pr.name.clone(),
        color: pr.color,
        outcome: d.context_for(pid).map(|c| if c.outcome.is_empty() { c.understanding.goal.clone() } else { c.outcome.clone() }).unwrap_or_default(),
        next: project_next_move(d, pid),
        frac: if tasks.is_empty() { 0.0 } else { done as f32 / tasks.len() as f32 },
        done: format!("{done} of {} tasks", tasks.len()),
        now: pill(lane("Now"), "Now"),
        next_n: pill(lane("Next"), "Next"),
        later: pill(lane("Later"), "Later"),
        open: if open == 0 { String::new() } else if open == 1 { "1 open question".into() } else { format!("{open} open questions") },
        updated,
    }
}

/// Roadmap edits collected while drawing, applied afterwards.
enum RmAct {
    Move(Id, &'static str),
    Gate(Id, Option<Id>),
    After(Id, Option<Id>),
    Status(Id, &'static str),
    OpenTask(Id),
}

impl App {
    /// The project's context; existing projects get one on first visit (fields start empty —
    /// nothing is guessed).
    pub(super) fn ensure_context(&mut self, pid: &str) -> Option<ProjectContext> {
        if self.store.data.context_for(pid).is_none() {
            let c = ProjectContext::adopt(&self.store.data, pid)?;
            self.store.data.contexts.push(c);
            self.touch();
        }
        self.store.data.context_for(pid).cloned()
    }

    pub(super) fn scope_ids(&self) -> Vec<Id> {
        match &self.current_project {
            Some(p) => vec![p.clone()],
            None => {
                let mut ps: Vec<&Project> = self.store.data.projects.iter().filter(|x| !x.archived).collect();
                ps.sort_by_key(|x| x.order);
                ps.into_iter().map(|x| x.id.clone()).collect()
            }
        }
    }

    pub(super) fn section_view(&mut self, ui: &mut egui::Ui, p: &Pal, s: Section) {
        // the redesigned sections (Phase 4) draw their own header, no top bar
        if s == Section::Overview && self.current_project.is_some() && self.overview_page(ui, p) {
            return;
        }
        if s == Section::Roadmap && self.current_project.is_some() && self.roadmap_page(ui, p) {
            return;
        }
        self.top_bar(ui, p, |_, _| {});
        let wide = matches!(s, Section::Roadmap);
        egui::ScrollArea::vertical().id_salt(("sec", s as u8)).auto_shrink([false, false]).show(ui, |ui| {
            centered(ui, if wide { 1400.0 } else { 1080.0 }, |ui| {
                let name = match s {
                    Section::Overview => "Overview",
                    Section::Roadmap => "Roadmap",
                    Section::Frameworks => "Frameworks",
                    Section::Knowledge => "Knowledge",
                    Section::AiTeam => "AI Team",
                    Section::Handoffs => "Handoffs",
                };
                if self.current_project.is_some() {
                    self.project_header(ui, p, name);
                } else {
                    let sub = match s {
                        Section::Roadmap | Section::Frameworks => "All projects. Pick a project in the sidebar to focus on one.",
                        Section::AiTeam => "Product specialists that read and update the real project context. Ask AI is the General assistant.",
                        Section::Handoffs => "Self-contained briefs for Claude, Codex or a person, written from the project's real context.",
                        _ => "Pick a project in the sidebar to focus on it.",
                    };
                    let all = if matches!(s, Section::Roadmap | Section::Frameworks | Section::Knowledge) { " (all projects)" } else { "" };
                    view_title(ui, p, &format!("{name}{all}"), Some(sub));
                }
                ui.add_space(8.0);
                match s {
                    Section::Overview => self.overview_view(ui, p),
                    Section::Roadmap => self.roadmap_view(ui, p),
                    Section::Frameworks => self.frameworks_view(ui, p),
                    Section::Knowledge => self.knowledge_view(ui, p),
                    Section::AiTeam => self.team_view(ui, p),
                    Section::Handoffs => self.handoffs_view(ui, p),
                }
                ui.add_space(40.0);
            });
        });
    }

    /// "Pick a project" prompt plus clickable project cards.
    pub(super) fn pick_project(&mut self, ui: &mut egui::Ui, p: &Pal, why: &str) {
        wrap_label(ui, RichText::new(why).size(size::BODY).color(p.dim));
        ui.add_space(8.0);
        self.project_cards(ui, p);
    }

    // ------------------------------------------------------------ Projects

    /// Projects (B-E7, Pack B): cards in two columns, three at 1920 or with the sidebar collapsed
    /// (B-E19), a dashed New project card last; empty, loading and error as drawn.
    pub(super) fn projects_view(&mut self, ui: &mut egui::Ui, p: &Pal) {
        let mut ids: Vec<(Id, i64)> = self.store.data.projects.iter().filter(|x| !x.archived).map(|x| (x.id.clone(), x.order)).collect();
        ids.sort_by_key(|x| x.1);
        let cards: Vec<ProjectCardModel> = ids.iter().map(|(id, _)| project_card_model(&self.store.data, id, now_ms(), now_local())).collect();
        let state = match self.view_state {
            ViewState::Full if cards.is_empty() => ViewState::Empty,
            s => s,
        };
        let wide = ui.ctx().screen_rect().width() > space::WIDE_FROM;
        let cols = if wide || self.store.settings.sidebar_collapsed { 3 } else { 2 };
        let mut act: Option<(usize, Id)> = None;
        let mut new_project = false;
        let el = ((self.now - self.view_since) * 1000.0) as f32;
        let reduced = super::motion::reduced();
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            egui::Frame::none().inner_margin(egui::Margin { left: space::WORK_SIDE, right: space::WORK_SIDE, top: space::WORK_TOP, bottom: space::S24 }).show(ui, |ui| {
                capped(ui, space::CAP_DEFAULT, |ui| {
                    screen_head(ui, p, "Projects", "Each project has its own overview, roadmap, tasks, frameworks and knowledge.", |ui| {
                        if button(ui, p, ButtonKind::Primary, "+ New project", false, true).clicked() {
                            new_project = true;
                        }
                    });
                    ui.add_space(space::S24);
                    match state {
                        ViewState::Empty => {
                            ui.add_space(space::S40 * 1.5);
                            if empty_center(ui, p, Icon::Projects, "No projects yet.", "Start one and Resolve turns it into a first move and a roadmap. Tasks you capture without a project stay in Tasks, Captured.", "New project →") {
                                new_project = true;
                            }
                        }
                        ViewState::Error => {
                            ui.add_space(space::S40 * 1.5);
                            let a = error_center(ui, p, "Resolve couldn't read your projects.", "Your data is still on this PC. This is usually a file another app has open. Try again in a moment.", "Open Help");
                            if a.retry {
                                self.store.reload_if_changed();
                            }
                            if a.manual {
                                self.go(View::Help);
                            }
                        }
                        ViewState::Loading => {
                            ui.columns(cols, |c| {
                                for col in c.iter_mut() {
                                    skeleton_card(col, p, 4, "");
                                }
                            });
                        }
                        ViewState::Full => {
                            let gap = space::S16;
                            let w = (ui.available_width() - gap * (cols as f32 - 1.0)) / cols as f32;
                            let n = cards.len() + 1;
                            for row in 0..(n + cols - 1) / cols {
                                ui.horizontal_top(|ui| {
                                    ui.spacing_mut().item_spacing.x = gap;
                                    for c in 0..cols {
                                        let i = row * cols + c;
                                        if i >= n {
                                            break;
                                        }
                                        let f = super::motion::frame_at(el, moves::CARDS.stagger(i, moves::CARDS_STAGGER), reduced);
                                        ui.allocate_ui_with_layout(egui::vec2(w, 0.0), Layout::top_down(Align::Min), |ui| {
                                            ui.set_width(w);
                                            ui.set_max_width(w);
                                            super::motion::fade_in(ui, f, egui::vec2(0.0, space::S18), |ui| {
                                                if i == cards.len() {
                                                    if dashed_add_card(ui, p, "New project", "Opens the intake. About ten minutes.", space::S40 * 5.5).clicked() {
                                                        new_project = true;
                                                    }
                                                    return;
                                                }
                                                let m = &cards[i];
                                                let pills = [m.now.as_str(), m.next_n.as_str(), m.later.as_str(), m.open.as_str()];
                                                let view = ProjectCardView { name: &m.name, dot: rgb(m.color), outcome: &m.outcome, next: &m.next, frac: m.frac, done: &m.done, pills, updated: &m.updated };
                                                let popup = egui::Id::new(("project-menu", &m.id));
                                                let open = ui.memory(|mem| mem.is_popup_open(popup));
                                                let (card, menu) = project_card(ui, p, &view, open);
                                                if menu.clicked() {
                                                    ui.memory_mut(|mem| mem.toggle_popup(popup));
                                                } else if card.clicked() {
                                                    act = Some((0, m.id.clone()));
                                                }
                                                egui::popup_below_widget(ui, popup, &menu, egui::PopupCloseBehavior::CloseOnClick, |ui| {
                                                    ui.set_min_width(space::S40 * 5.0);
                                                    for (k, label) in ["Open overview", "Rename", "Edit outcome"].iter().enumerate() {
                                                        if ui.selectable_label(false, RichText::new(*label).size(size::ROW_SMALL)).clicked() {
                                                            act = Some((k, m.id.clone()));
                                                        }
                                                    }
                                                });
                                            });
                                        });
                                    }
                                });
                                ui.add_space(gap);
                            }
                        }
                    }
                    self.project_edit_window(ui.ctx(), p);
                // archived projects stay reachable
                let archived: Vec<(Id, String)> = self.store.data.projects.iter().filter(|x| x.archived).map(|x| (x.id.clone(), x.name.clone())).collect();
                if !archived.is_empty() {
                    ui.add_space(12.0);
                    egui::CollapsingHeader::new(RichText::new(format!("Archived ({})", archived.len())).size(size::SMALL).color(p.dim)).id_salt("archived").show(ui, |ui| {
                        for (id, name) in archived {
                            ui.horizontal_wrapped(|ui| {
                                ui.label(RichText::new(&name).color(p.dim));
                                if ui.small_button("Restore").clicked() {
                                    if let Some(x) = self.store.data.projects.iter_mut().find(|x| x.id == id) {
                                        x.archived = false;
                                        x.updated = now_ms();
                                    }
                                    self.touch();
                                }
                            });
                        }
                    });
                }
                ui.add_space(space::S40);
                });
            });
        });
        if el < moves::CARDS.stagger(cards.len() + 1, moves::CARDS_STAGGER).end() {
            ui.ctx().request_repaint();
        }
        if new_project {
            self.start_intake();
        }
        match act {
            Some((0, id)) => {
                self.select_project(Some(id));
                self.go(View::Sec(Section::Overview));
            }
            Some((1, id)) => {
                let name = self.store.data.project(&id).map(|x| x.name.clone()).unwrap_or_default();
                self.temp.insert("project_edit".into(), format!("name\u{1f}{id}\u{1f}{name}"));
            }
            Some((_, id)) => {
                let outcome = self.store.data.context_for(&id).map(|c| c.outcome.clone()).unwrap_or_default();
                self.temp.insert("project_edit".into(), format!("outcome\u{1f}{id}\u{1f}{outcome}"));
            }
            None => {}
        }
    }

    /// Rename a project, or edit its outcome (from a card's ⋯ menu). Saving can be undone.
    fn project_edit_window(&mut self, ctx: &egui::Context, p: &Pal) {
        let Some(raw) = self.temp.get("project_edit").cloned() else { return };
        let mut parts = raw.splitn(3, '\u{1f}');
        let (field, id, mut text) = (parts.next().unwrap_or("").to_string(), parts.next().unwrap_or("").to_string(), parts.next().unwrap_or("").to_string());
        let mut close = false;
        let mut save = false;
        let title = if field == "name" { "Rename project" } else { "Edit outcome" };
        egui::Window::new(title).collapsible(false).resizable(false).anchor(egui::Align2::CENTER_TOP, [0.0, space::DIALOG_TOP]).frame(dialog_frame(p).inner_margin(egui::Margin::same(space::S18))).show(ctx, |ui| {
            ui.set_width(space::SEARCH_W - space::S40 * 4.0);
            let id_field = egui::Id::new("project-edit-field");
            let r = text_field(ui, p, id_field, &mut text, if field == "name" { "Project name" } else { "Who is better off, and how you'd know" }, field != "name", ui.available_width());
            r.request_focus();
            ui.add_space(space::S10);
            ui.horizontal(|ui| {
                save = button(ui, p, ButtonKind::Primary, "Save", false, !text.trim().is_empty()).clicked() || (field == "name" && ui.input(|i| i.key_pressed(egui::Key::Enter)) && !text.trim().is_empty());
                close = button(ui, p, ButtonKind::Secondary, "Cancel", false, true).clicked() || ui.input(|i| i.key_pressed(egui::Key::Escape));
            });
        });
        if save {
            let v = text.trim().to_string();
            let (pid, f) = (id.clone(), field.clone());
            let label = if f == "name" { format!("Renamed a project to \"{v}\"") } else { "Edited a project's outcome".to_string() };
            self.undoable(&label, "edit", move |d| {
                if f == "name" {
                    if let Some(x) = d.projects.iter_mut().find(|x| x.id == pid) {
                        x.name = v;
                        x.updated = now_ms();
                    }
                } else if let Some(c) = d.context_for_mut(&pid) {
                    c.outcome = v;
                    c.updated = now_ms();
                }
            });
            close = true;
        }
        if close {
            self.temp.remove("project_edit");
        } else {
            self.temp.insert("project_edit".into(), format!("{field}\u{1f}{id}\u{1f}{text}"));
        }
    }

    fn project_cards(&mut self, ui: &mut egui::Ui, p: &Pal) {
        let mut open: Option<Id> = None;
        let d = &self.store.data;
        let mut ps: Vec<&Project> = d.projects.iter().filter(|x| !x.archived).collect();
        ps.sort_by_key(|x| x.order);
        if ps.is_empty() {
            ui.label(RichText::new("No projects yet. Press New project to start one with the intake.").color(p.dim));
        }
        for pr in ps {
            let tasks: Vec<&Task> = d.tasks.iter().filter(|t| t.project_id.as_deref() == Some(pr.id.as_str()) && t.parent_id.is_none()).collect();
            let done = tasks.iter().filter(|t| t.completed).count();
            let lanes: Vec<usize> = LANES.iter().map(|l| d.records.iter().filter(|r| r.kind == Kind::Roadmap && r.project_id.as_deref() == Some(pr.id.as_str()) && r.status != "Dropped" && lane_of(r) == *l).count()).collect();
            let unc = d.records.iter().filter(|r| r.project_id.as_deref() == Some(pr.id.as_str()) && matches!(r.kind, Kind::Question | Kind::Hypothesis) && r.is_open()).count();
            let line = d.context_for(&pr.id).map(|c| if !c.outcome.is_empty() { c.outcome.clone() } else { c.understanding.goal.clone() }).unwrap_or_default();
            let resp = egui::Frame::none().fill(p.card).stroke(Stroke::new(1.0, p.line)).rounding(space::RADIUS).inner_margin(egui::Margin::symmetric(14.0, 10.0)).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.vertical(|ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new("●").color(rgb(pr.color)).size(16.0));
                        ui.label(RichText::new(&pr.name).font(fs(size::SECTION)));
                    });
                    if !line.is_empty() {
                        wrap_label(ui, RichText::new(&line).size(size::SMALL).color(p.dim));
                    }
                    ui.horizontal_wrapped(|ui| {
                        let frac = if tasks.is_empty() { 0.0 } else { done as f32 / tasks.len() as f32 };
                        let (r, _) = ui.allocate_exact_size(egui::vec2(90.0, 6.0), Sense::hover());
                        ui.painter().rect_filled(r, 3.0, p.hover);
                        let mut f = r;
                        f.set_width(r.width() * frac);
                        ui.painter().rect_filled(f, 3.0, rgb(pr.color));
                        ui.label(RichText::new(format!("{done}/{} tasks · {} now · {} next · {} later", tasks.len(), lanes[0], lanes[1], lanes[2])).size(size::SMALL).color(p.dim));
                        if unc > 0 {
                            chip(ui, &format!("{unc} open questions"), p.orange);
                        }
                    });
                });
            });
            if ui.interact(resp.response.rect, ui.id().with(("proj-row", &pr.id)), Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                open = Some(pr.id.clone());
            }
            ui.add_space(6.0);
        }
        if let Some(pid) = open {
            self.select_project(Some(pid));
            self.go(View::Sec(Section::Overview));
        }
    }

    // ------------------------------------------------------------ Overview

    fn overview_view(&mut self, ui: &mut egui::Ui, p: &Pal) {
        let Some(pid) = self.current_project.clone() else {
            self.pick_project(ui, p, "Choose a project to see its overview: the next move, what matters now and the context from the intake.");
            return;
        };
        let Some(ctx) = self.ensure_context(&pid) else { return };
        // the old "Plan page" and "Tasks board" buttons
        let brief = ctx.page_id.clone().or_else(|| self.store.data.project(&pid).and_then(|x| x.page_id.clone())).filter(|x| self.store.data.page(x).is_some());
        ui.horizontal_wrapped(|ui| {
            if let Some(pg) = brief.clone() {
                if ui.button("📄 Plan page").on_hover_text("The project brief (also in Knowledge)").clicked() {
                    self.go(View::Page(pg));
                }
            }
            if ui.button("☰ Tasks board").on_hover_text("This project's tasks as a board (Tasks)").clicked() {
                self.tasks_ui.board.insert(pid.clone());
                self.go(View::Project(pid.clone()));
            }
            if self.resolve_ui.rx.is_some() {
                ui.spinner();
                ui.label(RichText::new(&self.resolve_ui.busy).size(size::SMALL).color(p.dim));
            }
        });
        if !self.resolve_ui.error.is_empty() {
            wrap_label(ui, RichText::new(format!("⚠ {}", self.resolve_ui.error)).size(size::SMALL).color(p.orange));
        }
        ui.add_space(8.0);
        self.cockpit_overview(ui, p, &pid, &ctx);
    }

    // ------------------------------------------------------------ Roadmap

    fn roadmap_view(&mut self, ui: &mut egui::Ui, p: &Pal) {
        let scope = self.scope_ids();
        if scope.is_empty() {
            self.pick_project(ui, p, "No projects yet.");
            return;
        }
        ui.horizontal_wrapped(|ui| {
            if pill(ui, p, "By phase", !self.resolve_ui.roadmap_by_theme).clicked() {
                self.resolve_ui.roadmap_by_theme = false;
            }
            if pill(ui, p, "By theme", self.resolve_ui.roadmap_by_theme).clicked() {
                self.resolve_ui.roadmap_by_theme = true;
            }
            ui.label(RichText::new("Order comes from sequence, not dates: each theme follows the one before it.").size(size::TINY).color(p.faint));
        });
        // add a theme / milestone (one project at a time)
        if let Some(pid) = self.current_project.clone() {
            let (mut add_theme, mut add_ms) = (false, false);
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                if self.resolve_ui.new_theme_lane.is_empty() {
                    self.resolve_ui.new_theme_lane = "Now".into();
                }
                ui.add(egui::TextEdit::singleline(&mut self.resolve_ui.new_theme).hint_text("New theme…").desired_width(260.0));
                for l in LANES {
                    if pill(ui, p, l, self.resolve_ui.new_theme_lane == l).clicked() {
                        self.resolve_ui.new_theme_lane = l.into();
                    }
                }
                add_theme = ui.add_enabled(!self.resolve_ui.new_theme.trim().is_empty(), egui::Button::new("+ Theme")).clicked();
                ui.add_space(12.0);
                ui.add(egui::TextEdit::singleline(&mut self.resolve_ui.new_milestone).hint_text("New milestone…").desired_width(220.0));
                add_ms = ui.add_enabled(!self.resolve_ui.new_milestone.trim().is_empty(), egui::Button::new("+ Milestone")).clicked();
            });
            if add_theme {
                let mut r = Record::new(Kind::Roadmap, Some(pid.clone()), self.resolve_ui.new_theme.trim());
                r.fields.insert("phase".into(), self.resolve_ui.new_theme_lane.clone());
                self.store.data.records.push(r);
                self.resolve_ui.new_theme.clear();
                self.touch();
            }
            if add_ms {
                let r = Record::new(Kind::Milestone, Some(pid.clone()), self.resolve_ui.new_milestone.trim());
                self.store.data.records.push(r);
                self.resolve_ui.new_milestone.clear();
                self.touch();
            }
        }
        ui.add_space(8.0);
        let mut acts: Vec<RmAct> = vec![];
        let multi = scope.len() > 1;
        if self.resolve_ui.roadmap_by_theme {
            for pid in &scope {
                self.roadmap_by_theme(ui, p, pid, multi, &mut acts);
            }
        } else {
            ui.columns(3, |cols| {
                for (c, lane) in LANES.iter().enumerate() {
                    let ui = &mut cols[c];
                    ui.label(RichText::new(*lane).font(fs(size::SECTION)).color(if c == 0 { p.accent } else { p.dim }));
                    ui.add_space(4.0);
                    for pid in &scope {
                        self.roadmap_lane(ui, p, pid, lane, multi, &mut acts);
                    }
                }
            });
        }
        ui.add_space(14.0);
        self.roadmap_extras(ui, p, &scope, multi, &mut acts);
        for a in acts {
            match a {
                RmAct::OpenTask(t) => self.open_task = Some(t),
                RmAct::Move(id, lane) => {
                    if let Some(r) = self.store.data.record_mut(&id) {
                        r.fields.insert("phase".into(), lane.into());
                        r.updated = now_ms();
                    }
                    self.touch();
                }
                RmAct::Gate(id, h) => {
                    if let Some(r) = self.store.data.record_mut(&id) {
                        let _ = r.set_field("gate", h.as_deref().unwrap_or(""));
                    }
                    self.touch();
                }
                RmAct::After(id, o) => {
                    if let Some(r) = self.store.data.record_mut(&id) {
                        let _ = r.set_field("after", o.as_deref().unwrap_or(""));
                    }
                    self.touch();
                }
                RmAct::Status(id, s) => {
                    if let Some(r) = self.store.data.record_mut(&id) {
                        let _ = r.set_status(s);
                    }
                    self.touch();
                }
            }
        }
    }

    fn themes_of(&self, pid: &str) -> Vec<Record> {
        let mut v: Vec<Record> = self.store.data.records.iter().filter(|r| r.kind == Kind::Roadmap && r.project_id.as_deref() == Some(pid) && r.status != "Dropped").cloned().collect();
        v.sort_by_key(|r| (LANES.iter().position(|l| *l == lane_of(r)).unwrap_or(0), r.created));
        v
    }

    /// One theme card: sequence, title, basis, tasks done, dependency and validation gate.
    fn theme_card(&mut self, ui: &mut egui::Ui, p: &Pal, r: &Record, seq: usize, all: &[Record], acts: &mut Vec<RmAct>, show_tasks: bool) {
        let d = &self.store.data;
        let tasks: Vec<&Task> = r.links.iter().filter_map(|l| d.task(l)).collect();
        let done = tasks.iter().filter(|t| t.completed).count();
        let gate = d.record(r.field("gate")).cloned();
        let after = all.iter().find(|x| x.id == r.field("after")).map(|x| x.title.clone());
        let hyps: Vec<(Id, String)> = d.records.iter().filter(|x| x.kind == Kind::Hypothesis && x.project_id == r.project_id).map(|x| (x.id.clone(), x.title.clone())).collect();
        let lane = lane_of(r).to_string();
        egui::Frame::none().fill(p.card).stroke(Stroke::new(1.0, if lane == "Now" { p.accent } else { p.line })).rounding(space::RADIUS).inner_margin(egui::Margin::same(12.0)).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.vertical(|ui| {
                ui.label(RichText::new(format!("Step {seq}")).size(size::TINY).color(p.faint));
                wrap_label(ui, RichText::new(&r.title).font(fs(size::BODY)));
                if !r.body.is_empty() {
                    wrap_label(ui, RichText::new(&r.body).size(size::SMALL).color(p.dim));
                }
                ui.horizontal_wrapped(|ui| {
                    let basis = r.field("basis");
                    if !basis.is_empty() {
                        let col = match basis {
                            "evidence" => p.green,
                            "assumption" => p.accent,
                            _ => p.orange,
                        };
                        chip(ui, basis, col);
                    }
                    if r.status != "Planned" {
                        chip(ui, &r.status, if r.status == "Done" { p.green } else { p.blue });
                    }
                    if !tasks.is_empty() {
                        ui.label(RichText::new(format!("{done}/{} tasks", tasks.len())).size(size::TINY).color(p.dim));
                    }
                });
                if let Some(a) = &after {
                    wrap_label(ui, RichText::new(format!("⛓ After: {a}")).size(size::TINY).color(p.dim));
                }
                if let Some(g) = &gate {
                    let open = g.status == "Supported";
                    wrap_label(ui, RichText::new(format!("{} Gate: {} ({})", if open { "✔" } else { "⏸" }, g.title, g.status)).size(size::TINY).color(if open { p.green } else { p.orange }))
                        .on_hover_text(if open { "The hypothesis is supported, so this theme can go ahead." } else { "This theme waits until the hypothesis is supported." });
                }
                if show_tasks {
                    for t in &tasks {
                        if wrap_link(ui, RichText::new(format!("{} {}", if t.completed { "☑" } else { "☐" }, t.title)).size(size::SMALL)).clicked() {
                            acts.push(RmAct::OpenTask(t.id.clone()));
                        }
                    }
                }
                ui.horizontal_wrapped(|ui| {
                    let i = LANES.iter().position(|l| *l == lane).unwrap_or(0);
                    if i > 0 && ui.small_button(format!("← {}", LANES[i - 1])).clicked() {
                        acts.push(RmAct::Move(r.id.clone(), LANES[i - 1]));
                    }
                    if i < 2 && ui.small_button(format!("{} →", LANES[i + 1])).clicked() {
                        acts.push(RmAct::Move(r.id.clone(), LANES[i + 1]));
                    }
                    ui.menu_button(RichText::new("⋯").size(size::SMALL), |ui| {
                        ui.label(RichText::new("Comes after").size(size::TINY).color(p.dim));
                        if ui.selectable_label(after.is_none(), "Nothing (can start any time)").clicked() {
                            acts.push(RmAct::After(r.id.clone(), None));
                            ui.close_menu();
                        }
                        for o in all.iter().filter(|o| o.id != r.id) {
                            if ui.selectable_label(r.field("after") == o.id, &o.title).clicked() {
                                acts.push(RmAct::After(r.id.clone(), Some(o.id.clone())));
                                ui.close_menu();
                            }
                        }
                        ui.separator();
                        ui.label(RichText::new("Validation gate").size(size::TINY).color(p.dim));
                        if ui.selectable_label(gate.is_none(), "No gate").clicked() {
                            acts.push(RmAct::Gate(r.id.clone(), None));
                            ui.close_menu();
                        }
                        for (hid, ht) in &hyps {
                            if ui.selectable_label(r.field("gate") == hid, ht).clicked() {
                                acts.push(RmAct::Gate(r.id.clone(), Some(hid.clone())));
                                ui.close_menu();
                            }
                        }
                        ui.separator();
                        for s in ["Planned", "Active", "Done", "Dropped"] {
                            if ui.selectable_label(r.status == s, s).clicked() {
                                acts.push(RmAct::Status(r.id.clone(), s));
                                ui.close_menu();
                            }
                        }
                    });
                });
            });
        });
        ui.add_space(6.0);
    }

    fn roadmap_lane(&mut self, ui: &mut egui::Ui, p: &Pal, pid: &str, lane: &str, multi: bool, acts: &mut Vec<RmAct>) {
        let all = self.themes_of(pid);
        let items: Vec<(usize, Record)> = all.iter().enumerate().filter(|(_, r)| lane_of(r) == lane).map(|(i, r)| (i + 1, r.clone())).collect();
        // tasks in this phase that no theme covers (the project's Now / Next / Later sections)
        let linked: Vec<Id> = all.iter().flat_map(|r| r.links.clone()).collect();
        let loose: Vec<(Id, String)> = self.store.data.tasks.iter().filter(|t| t.project_id.as_deref() == Some(pid) && !t.completed && t.parent_id.is_none() && t.section == lane && !linked.contains(&t.id)).map(|t| (t.id.clone(), t.title.clone())).collect();
        if items.is_empty() && loose.is_empty() {
            return;
        }
        if multi {
            let name = self.store.data.project(pid).map(|x| (x.name.clone(), x.color)).unwrap_or_default();
            wrap_label(ui, RichText::new(format!("● {}", name.0)).size(size::SMALL).color(rgb(name.1)));
        }
        for (seq, r) in &items {
            self.theme_card(ui, p, r, *seq, &all, acts, false);
        }
        if !loose.is_empty() {
            ui.label(RichText::new(format!("Other {lane} tasks")).size(size::TINY).color(p.faint));
            for (id, t) in loose.iter().take(6) {
                if wrap_link(ui, RichText::new(format!("☐ {t}")).size(size::SMALL).color(p.dim)).clicked() {
                    acts.push(RmAct::OpenTask(id.clone()));
                }
            }
            if loose.len() > 6 {
                ui.label(RichText::new(format!("+ {} more in Tasks", loose.len() - 6)).size(size::TINY).color(p.faint));
            }
        }
        ui.add_space(8.0);
    }

    fn roadmap_by_theme(&mut self, ui: &mut egui::Ui, p: &Pal, pid: &str, multi: bool, acts: &mut Vec<RmAct>) {
        let all = self.themes_of(pid);
        if all.is_empty() {
            return;
        }
        if multi {
            let name = self.store.data.project(pid).map(|x| (x.name.clone(), x.color)).unwrap_or_default();
            ui.label(RichText::new(format!("● {}", name.0)).font(fs(size::SECTION)).color(rgb(name.1)));
        }
        for (i, r) in all.iter().enumerate() {
            ui.label(RichText::new(lane_of(r)).size(size::TINY).color(p.faint));
            self.theme_card(ui, p, r, i + 1, &all, acts, true);
        }
        ui.add_space(10.0);
    }

    /// Milestones, dependencies and what's coming up (the old Upcoming list feeds the roadmap).
    fn roadmap_extras(&mut self, ui: &mut egui::Ui, p: &Pal, scope: &[Id], multi: bool, acts: &mut Vec<RmAct>) {
        let d = &self.store.data;
        let in_scope = |r: &Record| r.project_id.as_ref().map_or(false, |x| scope.contains(x));
        let ms: Vec<Record> = d.records.iter().filter(|r| r.kind == Kind::Milestone && in_scope(r)).cloned().collect();
        let deps: Vec<Record> = d.records.iter().filter(|r| matches!(r.kind, Kind::Dependency | Kind::Blocker) && in_scope(r) && r.is_open()).cloned().collect();
        let t0 = today();
        let mut soon: Vec<&Task> = d.tasks.iter().filter(|t| !t.completed && t.project_id.as_ref().map_or(false, |x| scope.contains(x)) && t.due.map_or(false, |x| x <= t0 + chrono::Duration::days(14))).collect();
        soon.sort_by_key(|t| (t.due, t.priority));
        let pname = |id: &Option<Id>| id.as_deref().and_then(|x| d.project(x)).map(|x| x.name.clone()).unwrap_or_default();
        ui.columns(3, |cols| {
            card(&mut cols[0], p, |ui| {
                ui.label(RichText::new("⚑ Milestones").font(fs(size::SECTION)));
                if ms.is_empty() {
                    wrap_label(ui, RichText::new("None yet. Add one above (pick a project first).").size(size::SMALL).color(p.faint));
                }
                for (i, m) in ms.iter().enumerate() {
                    let proj = if multi { format!(" · {}", pname(&m.project_id)) } else { String::new() };
                    wrap_label(ui, RichText::new(format!("{}. {}{proj}", i + 1, m.title)).size(size::SMALL));
                    let crit = m.field("criteria");
                    if !crit.is_empty() {
                        wrap_label(ui, RichText::new(format!("Done when: {crit}")).size(size::TINY).color(p.dim));
                    }
                    if m.status != "Planned" {
                        chip(ui, &m.status, if m.status == "Reached" { p.green } else { p.orange });
                    }
                }
            });
            card(&mut cols[1], p, |ui| {
                ui.label(RichText::new("⛓ Dependencies & blockers").font(fs(size::SECTION)));
                if deps.is_empty() {
                    wrap_label(ui, RichText::new("Nothing waiting on anyone. Theme order (\"Comes after\") is set from each card's ⋯ menu.").size(size::SMALL).color(p.faint));
                }
                for r in &deps {
                    wrap_label(ui, RichText::new(format!("{} {} ({})", r.kind.icon(), r.title, r.status)).size(size::SMALL));
                }
            });
            card(&mut cols[2], p, |ui| {
                ui.label(RichText::new("Coming up").font(fs(size::SECTION)));
                ui.label(RichText::new("Dated tasks in the next 14 days").size(size::TINY).color(p.faint));
                if soon.is_empty() {
                    wrap_label(ui, RichText::new("Nothing dated soon.").size(size::SMALL).color(p.faint));
                }
                for t in soon.iter().take(10) {
                    let when = t.due.map(fmt_date_short).unwrap_or_default();
                    if wrap_link(ui, RichText::new(format!("{when} · {}", t.title)).size(size::SMALL)).clicked() {
                        acts.push(RmAct::OpenTask(t.id.clone()));
                    }
                }
            });
        });
    }
}
