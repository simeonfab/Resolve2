//! Overview (C-E1 to C-E3, Pack C "Overview"): the project's header with Weekly review and ⋯,
//! the dark DO THIS NEXT card beside the progress card, What matters now, Resolve suggests and
//! Project context, then a small Tasks board. Every line comes from the project's own data
//! (`overview_model`, self-tested); nothing is written by a model unless you run an action.

use super::nav::Section;
use super::resolve::JobOut;
use super::theme::*;
use super::widgets::*;
use super::*;
use crate::resolve::{assess, roles, Kind, Record};
use eframe::egui::{self, RichText};

/// What a Resolve suggests row opens.
#[derive(Clone, Debug, PartialEq)]
pub enum Suggest {
    /// Open this task.
    Task(Id),
    /// Run this ResolveOS action (roles::ACTIONS).
    Action(&'static str),
}

/// Everything Overview shows, worked out from the workspace.
#[derive(Clone, Debug, Default)]
pub struct OverviewModel {
    pub name: String,
    pub outcome: String,
    /// The next move: the intake's first move, else the pinned task.
    pub move_title: String,
    pub move_task: Option<Id>,
    /// "Tests H1, monthly workload is predictable. Its result opens the gate on <theme>." when
    /// the data links them, else the intake's reason for the move, else empty.
    pub move_line: String,
    pub done: usize,
    pub total: usize,
    pub now: usize,
    pub next: usize,
    pub later: usize,
    pub hyp_resolved: usize,
    pub hyp_total: usize,
    pub exp_planned: usize,
    pub open_questions: usize,
    /// (line, urgent?, what it opens)
    pub matters: Vec<(String, bool, Option<Id>)>,
    pub suggests: Vec<(String, Suggest)>,
    pub context: String,
    pub brief: Option<Id>,
    pub plan: Option<Id>,
    /// (lane, open count, the first task titles with their ids)
    pub board: Vec<(&'static str, usize, Vec<(Id, String)>)>,
    /// Anything to plan from (a roadmap or tasks)?
    pub has_plan: bool,
}

/// "H1: Monthly workload is predictable" → ("H1", "monthly workload is predictable").
fn hyp_parts(title: &str) -> (String, String) {
    match title.split_once(": ") {
        Some((id, rest)) if id.starts_with('H') && id.len() <= 4 => {
            let mut c = rest.chars();
            let low = c.next().map(|f| f.to_lowercase().collect::<String>() + c.as_str()).unwrap_or_default();
            (id.to_string(), low.trim_end_matches('.').to_string())
        }
        _ => (String::new(), title.trim_end_matches('.').to_string()),
    }
}

pub fn overview_model(d: &WorkspaceData, pid: &str) -> OverviewModel {
    let mut m = OverviewModel::default();
    let Some(pr) = d.project(pid) else { return m };
    m.name = pr.name.clone();
    let ctx = d.context_for(pid).cloned();
    if let Some(c) = &ctx {
        m.outcome = if c.outcome.trim().is_empty() { c.understanding.goal.clone() } else { c.outcome.clone() };
    }
    let a = ctx.as_ref().map(|c| assess::assess(d, c)).unwrap_or_default();
    m.move_title = super::sections::project_next_move(d, pid);
    m.move_task = a.win.clone().or_else(|| ctx.as_ref().and_then(|c| c.win_task_id.clone()));
    if m.move_title.is_empty() {
        m.move_title = m.move_task.as_ref().and_then(|t| d.task(t)).map(|t| t.title.clone()).unwrap_or_default();
    }
    // the hypothesis the move tests: an experiment that links the move's task; the gate it opens:
    // a theme gated by that hypothesis. Only what the data says; otherwise the intake's reason.
    let recs: Vec<&Record> = d.records.iter().filter(|r| r.project_id.as_deref() == Some(pid)).collect();
    let hyp = m.move_task.as_ref().and_then(|t| recs.iter().find(|r| r.kind == Kind::Experiment && r.links.iter().any(|l| l == t))).and_then(|e| e.parent_id.as_ref()).and_then(|h| recs.iter().find(|r| &r.id == h && r.kind == Kind::Hypothesis));
    if let Some(h) = hyp {
        let (id, stmt) = hyp_parts(&h.title);
        m.move_line = if id.is_empty() { format!("Tests {stmt}.") } else { format!("Tests {id}, {stmt}.") };
        if let Some(g) = recs.iter().find(|r| r.kind == Kind::Roadmap && r.field("gate") == h.id && r.status != "Dropped") {
            m.move_line.push_str(&format!(" Its result opens the gate on {}.", g.title));
        }
    } else {
        m.move_line = d.intakes.iter().find(|i| i.project_id.as_deref() == Some(pid)).and_then(|i| i.proposal.as_ref()).map(|p| p.why.trim().to_string()).unwrap_or_default();
    }
    m.done = a.done;
    m.total = a.total;
    let lane = |l: &str| recs.iter().filter(|r| r.kind == Kind::Roadmap && r.status != "Dropped" && super::sections::lane_of(r) == l).count();
    (m.now, m.next, m.later) = (lane("Now"), lane("Next"), lane("Later"));
    let hyps: Vec<&&Record> = recs.iter().filter(|r| r.kind == Kind::Hypothesis && r.field("type") != "H0").collect();
    m.hyp_total = hyps.len();
    m.hyp_resolved = hyps.iter().filter(|r| r.status != "Unresolved").count();
    m.exp_planned = recs.iter().filter(|r| r.kind == Kind::Experiment && r.status == "Planned").count();
    m.open_questions = recs.iter().filter(|r| r.kind == Kind::Question && r.is_open()).count();
    // what matters now: what is waiting on someone, then the assessment's own order
    let tasks: Vec<&Task> = d.tasks.iter().filter(|t| t.project_id.as_deref() == Some(pid) && t.parent_id.is_none()).collect();
    let now_ms = crate::model::now_ms();
    for t in tasks.iter().filter(|t| !t.completed && t.waiting_since.is_some()) {
        let days = ((now_ms - t.waiting_since.unwrap_or(now_ms)).max(0) / (24 * 3600 * 1000)) as i64;
        let when = match days {
            0 => "since today".to_string(),
            1 => "for 1 day".to_string(),
            n => format!("for {n} days"),
        };
        m.matters.push((format!("{} has been waiting {when}.", t.title), true, Some(t.id.clone())));
    }
    for (reason, target) in &a.next {
        if reason.starts_with("Do this next") {
            continue;
        }
        let urgent = reason.starts_with("Unblock") || reason.starts_with("Overdue");
        m.matters.push((format!("{reason}."), urgent, target.clone()));
    }
    m.matters.truncate(3);
    // Resolve suggests: each row opens an action, worked out from the project
    let themes: Vec<&&Record> = recs.iter().filter(|r| r.kind == Kind::Roadmap && r.status != "Dropped").collect();
    if let Some(t) = themes.iter().find(|r| super::sections::lane_of(r) == "Now" && !r.links.iter().any(|l| d.task(l).is_some())) {
        m.suggests.push((format!("Turn {} into tasks", t.title), Suggest::Action("prepare-build")));
    }
    if let Some(h) = hyps.iter().find(|h| h.is_open() && !recs.iter().any(|e| e.kind == Kind::Experiment && e.parent_id.as_deref() == Some(&h.id))) {
        let (id, stmt) = hyp_parts(&h.title);
        m.suggests.push((format!("Design a test for {}", if id.is_empty() { stmt } else { id }), Suggest::Action("validation-plan")));
    }
    for t in tasks.iter().filter(|t| !t.completed && t.waiting_since.is_some()) {
        m.suggests.push((format!("Chase {}", t.title), Suggest::Task(t.id.clone())));
    }
    if m.suggests.len() < 3 && recs.iter().any(|r| r.field("basis") == "assumption" || r.kind == Kind::Assumption) {
        m.suggests.push(("Challenge the assumptions this rests on".into(), Suggest::Action("challenge")));
    }
    m.suggests.truncate(3);
    // context: where it is, in the person's own words from the intake
    if let Some(c) = &ctx {
        m.context = [&c.understanding.state, &c.understanding.project, &c.notes].iter().map(|s| s.trim().to_string()).find(|s| !s.is_empty()).unwrap_or_default();
        m.brief = c.page_id.clone().or_else(|| pr.page_id.clone()).filter(|x| d.page(x).is_some());
    }
    if let Some(b) = &m.brief {
        m.plan = d.pages.iter().find(|p| p.parent.as_deref() == Some(b.as_str()) && p.title.to_lowercase().contains("plan")).map(|p| p.id.clone());
    }
    for l in super::sections::LANES {
        let open: Vec<&&Task> = tasks.iter().filter(|t| !t.completed && t.section == l).collect();
        m.board.push((l, open.len(), open.iter().take(3).map(|t| (t.id.clone(), t.title.clone())).collect()));
    }
    m.has_plan = !tasks.is_empty() || !themes.is_empty();
    m
}

impl App {
    /// The redesigned Overview. Returns false when there is no project to show (the caller
    /// shows the project picker).
    pub(super) fn overview_page(&mut self, ui: &mut egui::Ui, p: &Pal) -> bool {
        let Some(pid) = self.current_project.clone() else { return false };
        let Some(ctx) = self.ensure_context(&pid) else { return false };
        // keep the pinned move stable (the ResolvePM rule): re-pick only when it's done
        let a = assess::assess(&self.store.data, &ctx);
        if a.win != ctx.win_task_id {
            if let Some(c) = self.store.data.context_for_mut(&pid) {
                c.win_task_id = a.win.clone();
                c.updated = now_ms();
            }
            self.touch();
        }
        let m = overview_model(&self.store.data, &pid);
        let state = match self.view_state {
            ViewState::Full if !m.has_plan => ViewState::Empty,
            s => s,
        };
        // C-E3: an action run from here that failed shows in the next-move card; the rest loads
        let failed = self.temp.get("overview_action").cloned().filter(|_| !self.resolve_ui.error.is_empty() && self.resolve_ui.rx.is_none());
        let busy = self.resolve_ui.rx.is_some();
        let mut act: Option<&'static str> = None;
        let mut open_task: Option<Id> = None;
        let mut goto: Option<View> = None;
        let mut review = false;
        let mut edit_ctx = false;
        let el = ((self.now - self.view_since) * 1000.0) as f32;
        let reduced = super::motion::reduced();
        egui::ScrollArea::vertical().auto_shrink([false, false]).id_salt(("overview", &pid)).show(ui, |ui| {
            egui::Frame::none().inner_margin(egui::Margin { left: space::WORK_SIDE, right: space::WORK_SIDE, top: space::WORK_TOP, bottom: space::S40 * 2.0 }).show(ui, |ui| {
                capped(ui, space::CAP_DEFAULT, |ui| {
                    // header
                    ui.label(RichText::new("Overview").font(fs(size::ROW_SMALL)).color(p.orange));
                    screen_head(ui, p, &m.name, &m.outcome, |ui| {
                        ui.spacing_mut().item_spacing.x = space::S10;
                        let more = button(ui, p, ButtonKind::Secondary, "⋯", false, true);
                        let menu_id = egui::Id::new(("overview-menu", &pid));
                        if more.clicked() {
                            ui.memory_mut(|mm| mm.toggle_popup(menu_id));
                        }
                        egui::popup_below_widget(ui, menu_id, &more, egui::PopupCloseBehavior::CloseOnClick, |ui| {
                            ui.set_min_width(space::S40 * 6.0);
                            for a in roles::ACTIONS.iter().filter(|a| a.id != "handoff") {
                                if ui.add_enabled(!busy || a.deterministic, egui::Button::new(a.label)).on_hover_text(a.hint).clicked() {
                                    act = Some(a.id);
                                    ui.close_menu();
                                }
                            }
                            if ui.button("Write a handoff").clicked() {
                                goto = Some(View::Sec(Section::Handoffs));
                                ui.close_menu();
                            }
                            ui.separator();
                            if ui.button("Edit the project context").clicked() {
                                edit_ctx = true;
                                ui.close_menu();
                            }
                        });
                        if button(ui, p, ButtonKind::Secondary, "Weekly review", false, !busy).on_hover_text("Resolve writes a short review of your week and saves it as a page").clicked() {
                            review = true;
                        }
                    });
                    ui.add_space(space::S18);
                    match state {
                        ViewState::Loading => {
                            ui.horizontal_top(|ui| {
                                let w = ui.available_width();
                                ui.allocate_ui(egui::vec2(w * 0.6, space::HOME_SKELETON_HERO), |ui| skeleton_card(ui, p, 3, ""));
                                ui.add_space(space::S10);
                                ui.vertical(|ui| skeleton_card(ui, p, 3, ""));
                            });
                            ui.add_space(space::S10);
                            skeleton_card(ui, p, 2, "");
                            ui.label(RichText::new("Reading the project…").size(size::SMALL).color(p.dim));
                            return;
                        }
                        ViewState::Empty => {
                            ui.add_space(space::S40 * 2.0);
                            if empty_center(ui, p, Icon::Overview, "This project has no plan yet.", "Finish setting it up and Resolve fills this page with your next move, your roadmap and your first week.", "Continue setting up →") {
                                self.temp.insert("overview_setup".into(), "1".into());
                            }
                            ui.vertical_centered(|ui| {
                                if button(ui, p, ButtonKind::Ghost, "Add a task", false, true).clicked() {
                                    self.temp.insert("overview_add".into(), "1".into());
                                }
                            });
                            return;
                        }
                        _ => {}
                    }
                    let error = state == ViewState::Error || failed.is_some();
                    // row 1: the next move beside the progress
                    let gap = space::S10 + space::S6;
                    let w = ui.available_width();
                    let w_hero = (w - gap) * 0.62;
                    let row1 = egui::Id::new(("ov-row1", &pid));
                    let h1: f32 = ui.data(|d| d.get_temp(row1)).unwrap_or(0.0);
                    let mut hs = [0.0f32; 2];
                    ui.horizontal_top(|ui| {
                        ui.spacing_mut().item_spacing.x = gap;
                        let f0 = super::motion::frame_at(el, moves::CARDS.stagger(0, moves::CARDS_STAGGER), reduced);
                        ui.allocate_ui_with_layout(egui::vec2(w_hero, 0.0), Layout::top_down(Align::Min), |ui| {
                            ui.set_width(w_hero);
                            let top = ui.cursor().min.y;
                            super::motion::fade_in(ui, f0, egui::vec2(0.0, space::S18), |ui| {
                                hero_card(ui, p, |ui| {
                                    ui.set_min_height((h1 - space::HERO_PAD_Y * 2.0 - space::S4).max(0.0));
                                    if error {
                                        ui.label(RichText::new("Resolve couldn't work out your next move.").font(fserif(size::CARD_SERIF)).color(tok::TEXT_ON_DARK));
                                        ui.add_space(space::S6);
                                        ui.add(egui::Label::new(RichText::new("Your roadmap, tasks and pages are safe and open as normal. Only the suggestion failed. Try again, or pick the next move yourself from the roadmap.").size(size::ROW).color(tok::TEXT_ON_DARK_SOFT)).wrap());
                                        if !self.resolve_ui.error.is_empty() {
                                            ui.add(egui::Label::new(RichText::new(format!("It said: {}", self.resolve_ui.error)).size(size::SMALL).color(tok::TEXT_ON_DARK_MUTED)).wrap());
                                        }
                                        ui.add_space(space::S12);
                                        ui.horizontal(|ui| {
                                            if button(ui, p, ButtonKind::Primary, "Try again", false, true).clicked() {
                                                act = failed.as_deref().and_then(|f| roles::ACTIONS.iter().find(|a| a.id == f)).map(|a| a.id);
                                                self.resolve_ui.error.clear();
                                            }
                                            if button(ui, p, ButtonKind::OnDark, "Open the roadmap", false, true).clicked() {
                                                goto = Some(View::Sec(Section::Roadmap));
                                            }
                                        });
                                    } else {
                                        ui.label(RichText::new("DO THIS NEXT").font(fs(size::SMALL)).color(tok::AMBER));
                                        ui.add_space(space::S6);
                                        ui.add(egui::Label::new(RichText::new(if m.move_title.is_empty() { "Nothing open. Plan the next piece of work." } else { m.move_title.as_str() }).font(fserif(size::CARD_SERIF)).color(tok::TEXT_ON_DARK)).wrap());
                                        if !m.move_line.is_empty() {
                                            ui.add_space(space::S6);
                                            ui.add(egui::Label::new(RichText::new(&m.move_line).size(size::ROW).color(tok::TEXT_ON_DARK_SOFT)).wrap());
                                        }
                                        ui.add_space(space::S14);
                                        ui.horizontal(|ui| {
                                            if button(ui, p, ButtonKind::Primary, "Start this", false, m.move_task.is_some()).clicked() {
                                                open_task = m.move_task.clone();
                                            }
                                            if button(ui, p, ButtonKind::OnDark, "See it on the roadmap", false, true).clicked() {
                                                goto = Some(View::Sec(Section::Roadmap));
                                            }
                                        });
                                    }
                                });
                            });
                            hs[0] = ui.min_rect().max.y - top;
                        });
                        let f1 = super::motion::frame_at(el, moves::CARDS.stagger(1, moves::CARDS_STAGGER), reduced);
                        ui.allocate_ui_with_layout(egui::vec2(w - w_hero - gap, 0.0), Layout::top_down(Align::Min), |ui| {
                            ui.set_width(w - w_hero - gap);
                            let top = ui.cursor().min.y;
                            super::motion::fade_in(ui, f1, egui::vec2(0.0, space::S18), |ui| {
                                white_card(ui, p, false, |ui| {
                                    ui.set_min_height((h1 - space::S14 * 2.0 - stroke::HAIR * 2.0).max(0.0));
                                    ui.spacing_mut().item_spacing.y = space::S12;
                                    let pct = if m.total == 0 { 0 } else { (m.done * 100 + m.total / 2) / m.total };
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(format!("{pct}%")).font(fserif(size::HERO_TITLE_WIDE)).color(p.text));
                                        ui.label(RichText::new(format!("{} of {} tasks done", m.done, m.total)).size(size::ROW).color(p.dim));
                                    });
                                    progress_line(ui, p, if m.total == 0 { 0.0 } else { m.done as f32 / m.total as f32 }, ui.available_width());
                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().item_spacing.x = space::S6;
                                        count_pill(ui, p, &format!("{} Now", m.now), 0);
                                        count_pill(ui, p, &format!("{} Next", m.next), 1);
                                        count_pill(ui, p, &format!("{} Later", m.later), 1);
                                    });
                                    let w2 = ui.available_width();
                                    let (r, _) = ui.allocate_exact_size(egui::vec2(w2, stroke::HAIR), egui::Sense::hover());
                                    ui.painter().rect_filled(r, 0.0, p.divider);
                                    let mut job = egui::text::LayoutJob::default();
                                    let f = |font: egui::FontId, c| egui::text::TextFormat { font_id: font, color: c, ..Default::default() };
                                    job.append("Hypotheses: ", 0.0, f(fp(size::ROW_SMALL), p.dim));
                                    job.append(&format!("{} of {} resolved", m.hyp_resolved, m.hyp_total), 0.0, f(fs(size::ROW_SMALL), p.text));
                                    job.append(&format!(" · {} experiment{} planned · {} open question{}", m.exp_planned, if m.exp_planned == 1 { "" } else { "s" }, m.open_questions, if m.open_questions == 1 { "" } else { "s" }), 0.0, f(fp(size::ROW_SMALL), p.dim));
                                    job.wrap.max_width = w2;
                                    ui.label(job);
                                });
                            });
                            hs[1] = ui.min_rect().max.y - top;
                        });
                    });
                    let nat = hs[0].max(hs[1]);
                    if (nat - h1).abs() > 0.5 {
                        ui.data_mut(|d| d.insert_temp(row1, nat));
                        ui.ctx().request_repaint();
                    }
                    ui.add_space(gap);
                    // row 2: what matters now, Resolve suggests, project context
                    let w3 = (w - gap * 2.0) / 3.0;
                    let row2 = egui::Id::new(("ov-row2", &pid));
                    let h2: f32 = ui.data(|d| d.get_temp(row2)).unwrap_or(0.0);
                    let mut hs2 = [0.0f32; 3];
                    ui.horizontal_top(|ui| {
                        ui.spacing_mut().item_spacing.x = gap;
                        for k in 0..3 {
                            let fk = super::motion::frame_at(el, moves::CARDS.stagger(2 + k, moves::CARDS_STAGGER), reduced);
                            ui.allocate_ui_with_layout(egui::vec2(w3, 0.0), Layout::top_down(Align::Min), |ui| {
                                ui.set_width(w3);
                                let top = ui.cursor().min.y;
                                super::motion::fade_in(ui, fk, egui::vec2(0.0, space::S18), |ui| {
                                    white_card(ui, p, false, |ui| {
                                        ui.set_min_height((h2 - space::S14 * 2.0 - stroke::HAIR * 2.0).max(0.0));
                                        match k {
                                            0 => {
                                                ui.label(RichText::new("What matters now").font(fs(size::ROW + 0.5)).color(p.text));
                                                if m.matters.is_empty() {
                                                    ui.label(RichText::new("Nothing pressing.").size(size::ROW_SMALL).color(p.dim));
                                                }
                                                for (line, urgent, target) in &m.matters {
                                                    let r = ui.horizontal_top(|ui| {
                                                        let (d, _) = ui.allocate_exact_size(egui::vec2(space::S10, size::ROW * 1.4), egui::Sense::hover());
                                                        ui.painter().circle_filled(d.center(), space::S4 - 0.5, if *urgent { p.orange } else { tok::AMBER });
                                                        ui.add(egui::Label::new(RichText::new(line).size(size::ROW_SMALL).color(p.text)).wrap().sense(egui::Sense::click()))
                                                    });
                                                    if r.inner.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                                        if let Some(t) = target {
                                                            if self.store.data.task(t).is_some() {
                                                                open_task = Some(t.clone());
                                                            } else {
                                                                self.resolve_ui.sel_record = Some(t.clone());
                                                                goto = Some(View::Sec(Section::Frameworks));
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            1 => {
                                                ui.label(RichText::new("Resolve suggests").font(fs(size::ROW + 0.5)).color(p.text));
                                                if busy {
                                                    ui.horizontal(|ui| {
                                                        spinner_ring(ui, ui.cursor().min + egui::vec2(space::S8, space::S8), space::S6);
                                                        ui.add_space(space::S18);
                                                        ui.label(RichText::new(&self.resolve_ui.busy).size(size::ROW_SMALL).color(p.dim));
                                                    });
                                                } else if m.suggests.is_empty() {
                                                    ui.label(RichText::new("Nothing to suggest right now.").size(size::ROW_SMALL).color(p.dim));
                                                }
                                                for (label, s) in &m.suggests {
                                                    if suggest_row(ui, p, label).clicked() && !busy {
                                                        match s {
                                                            Suggest::Task(t) => open_task = Some(t.clone()),
                                                            Suggest::Action(a) => act = Some(a),
                                                        }
                                                    }
                                                }
                                            }
                                            _ => {
                                                ui.label(RichText::new("Project context").font(fs(size::ROW + 0.5)).color(p.text));
                                                ui.add(egui::Label::new(RichText::new(if m.context.is_empty() { "Nothing written about this project yet." } else { m.context.as_str() }).size(size::ROW_SMALL).color(if m.context.is_empty() { p.dim } else { p.text })).wrap());
                                                ui.add_space(space::S6);
                                                ui.horizontal_top(|ui| {
                                                    let bw = (ui.available_width() - space::S8) / 2.0;
                                                    ui.spacing_mut().item_spacing.x = space::S8;
                                                    for (label, pg) in [("Project brief", m.brief.clone()), ("Plan page", m.plan.clone())] {
                                                        ui.allocate_ui_with_layout(egui::vec2(bw, space::BUTTON_H), Layout::top_down_justified(Align::Center), |ui| {
                                                            if wide_button(ui, p, ButtonKind::Secondary, label, pg.is_some()).clicked() {
                                                                if let Some(pg) = pg {
                                                                    goto = Some(View::Page(pg));
                                                                }
                                                            }
                                                        });
                                                    }
                                                });
                                            }
                                        }
                                    });
                                });
                                hs2[k] = ui.min_rect().max.y - top;
                            });
                        }
                    });
                    let nat2 = hs2.iter().cloned().fold(0.0, f32::max);
                    if (nat2 - h2).abs() > 0.5 {
                        ui.data_mut(|d| d.insert_temp(row2, nat2));
                        ui.ctx().request_repaint();
                    }
                    // AI proposals from an action land here, under the cards
                    self.proposals_panel(ui, p, &pid);
                    ui.add_space(gap);
                    // row 3: a small Tasks board
                    let f5 = super::motion::frame_at(el, moves::CARDS.stagger(5, moves::CARDS_STAGGER), reduced);
                    super::motion::fade_in(ui, f5, egui::vec2(0.0, space::S18), |ui| {
                        white_card(ui, p, false, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Tasks").font(fs(size::ROW + 0.5)).color(p.text));
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if link(ui, p, "Open Tasks →").clicked() {
                                        goto = Some(View::Project(pid.clone()));
                                    }
                                });
                            });
                            let cw = (ui.available_width() - space::S10 * 2.0) / 3.0;
                            ui.horizontal_top(|ui| {
                                ui.spacing_mut().item_spacing.x = space::S10;
                                for (lane, n, cards) in &m.board {
                                    ui.allocate_ui_with_layout(egui::vec2(cw, 0.0), Layout::top_down(Align::Min), |ui| {
                                        mini_column(ui, p, cw, &format!("{lane} · {n}"), cards, &mut open_task);
                                    });
                                }
                            });
                        });
                    });
                    if self.temp.contains_key("overview_ctx_edit") {
                        ui.add_space(gap);
                        self.context_editor(ui, p, &pid);
                    }
                });
            });
        });
        if el < moves::CARDS.stagger(6, moves::CARDS_STAGGER).end() {
            ui.ctx().request_repaint();
        }
        if edit_ctx {
            self.temp.insert("overview_ctx_edit".into(), "1".into());
        }
        if self.temp.remove("overview_add").is_some() {
            self.open_quick(QuickKind::Task);
        }
        if self.temp.remove("overview_setup").is_some() {
            // the unfinished intake (or a new one)
            self.start_intake();
        }
        if review {
            self.page_ui.ai_open = true;
            self.ask_send("Write a short weekly review from what I completed this week, what's overdue and what's coming up, and save it as a new page called 'Weekly review' with today's date.");
        }
        if let Some(t) = open_task {
            self.open_task = Some(t);
        }
        if let Some(id) = act {
            self.temp.insert("overview_action".into(), id.to_string());
            self.resolve_ui.error.clear();
            self.run_overview_action(&pid, &ctx, id);
        }
        if let Some(v) = goto {
            self.go(v);
        }
        true
    }

    /// Run a ResolveOS action from Overview: What matters now is worked out here; the others ask
    /// the AI for proposals you accept or not.
    pub(super) fn run_overview_action(&mut self, pid: &str, ctx: &crate::resolve::ProjectContext, id: &'static str) {
        match id {
            "what-matters" => {
                let text = assess::summary_text(&self.store.data, ctx);
                let mut g = Record::new(Kind::Guidance, Some(pid.to_string()), "What matters now");
                g.body = text;
                g.fields.insert("action".into(), "what-matters".into());
                g.fields.insert("provider".into(), "deterministic".into());
                self.undoable("Saved What matters now to the project's history", "edit", move |d| d.records.push(g));
            }
            "handoff" => self.go(View::Sec(Section::Handoffs)),
            _ => {
                let label = roles::ACTIONS.iter().find(|a| a.id == id).map(|a| a.label).unwrap_or("Working");
                let cfg = crate::ai::current_config();
                let (c, d, pid2, action) = (ctx.clone(), self.store.data.clone(), pid.to_string(), id.to_string());
                self.spawn_job(&format!("{label}…"), move || {
                    let r = roles::propose(&cfg, &action, &c, &d);
                    JobOut::Proposals(pid2, action, r)
                });
            }
        }
    }

    /// The project context, editable (the ⋯ menu's "Edit the project context"). Every role,
    /// assessment and handoff reads from it.
    fn context_editor(&mut self, ui: &mut egui::Ui, p: &Pal, pid: &str) {
        let Some(ctx) = self.store.data.context_for(pid).cloned() else { return };
        let mut c = ctx.clone();
        let mut close = false;
        white_card(ui, p, false, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Project context").font(fs(size::ROW + 0.5)).color(p.text));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if button(ui, p, ButtonKind::Secondary, "Done", true, true).clicked() {
                        close = true;
                    }
                });
            });
            ui.label(RichText::new("Carried over from the intake. Every role, assessment and handoff reads from it.").size(size::SMALL).color(p.dim));
            let fields: [(&str, &mut String); 6] = [
                ("Project", &mut c.understanding.project),
                ("What you want", &mut c.understanding.goal),
                ("Where it is", &mut c.understanding.state),
                ("Biggest uncertainty", &mut c.understanding.uncertainty),
                ("The outcome: who is better off, and how you'd know", &mut c.outcome),
                ("Notes", &mut c.notes),
            ];
            for (label, v) in fields {
                ui.label(RichText::new(label).font(fs(size::SMALL)).color(p.orange));
                ui.add(egui::TextEdit::multiline(v).desired_rows(1).desired_width(f32::INFINITY));
            }
            ui.checkbox(&mut c.outcome_confirmed, "The outcome is confirmed (it names who is better off and a change you could see)");
        });
        if serde_json::to_string(&c).ok() != serde_json::to_string(&ctx).ok() {
            c.updated = now_ms();
            if let Some(x) = self.store.data.contexts.iter_mut().find(|x| x.id == c.id) {
                *x = c;
            }
            self.touch();
        }
        if close {
            self.temp.remove("overview_ctx_edit");
        }
    }
}
