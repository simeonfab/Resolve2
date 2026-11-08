//! Home (Pack B, tickets B-E4 and B-E6): the greeting, the hero "Your highest-value move" from
//! a project's first move, Today (events and tasks due today), Since Monday, Waiting on,
//! Projects and Captured, in the four states. The data is worked out by `home_model`, a pure
//! function the self-test checks; the drawing only places it.

use super::motion::{self, frame_at};
use super::theme::*;
use super::widgets::*;
use super::*;
use chrono::{Datelike, Local, Timelike};
use eframe::egui::{self, RichText};

#[derive(Clone, Debug, PartialEq)]
pub enum AgendaKind {
    Event,
    /// A task in a project.
    Task,
    /// A task not in a project yet.
    Captured,
}

#[derive(Clone, Debug)]
pub struct AgendaItem {
    /// "10:30", or "Today" for a task with no time.
    pub time: String,
    pub sort: NaiveTime,
    pub title: String,
    pub meta: String,
    pub kind: AgendaKind,
    pub task: Option<Id>,
    pub event: Option<Id>,
}

#[derive(Clone, Debug)]
pub struct HeroMove {
    pub project_id: Id,
    pub project: String,
    pub title: String,
    pub why: String,
    pub theme: String,
    pub task: Option<Id>,
}

#[derive(Clone, Debug, Default)]
pub struct HomeModel {
    pub greeting: String,
    pub subline: String,
    pub hero: Option<HeroMove>,
    pub agenda: Vec<AgendaItem>,
    pub wins: Vec<String>,
    /// (what, how long it has waited)
    pub waiting: Vec<(String, String)>,
    /// (project id, name, done, total)
    pub projects: Vec<(Id, String, usize, usize)>,
    pub captured: usize,
    /// An unfinished intake: (id, the rail step's name).
    pub setup: Option<(Id, &'static str)>,
}

fn plural(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// The rail's step names (1 to 4).
pub const RAIL_STEPS: [&str; 4] = ["Get started", "Bring it in", "Confirm understanding", "Your next move"];

/// Everything Home shows, from the workspace alone (no AI). `now` is the app's clock;
/// `real_ms` the real time, for ages and "since Monday" (edit times are real).
pub fn home_model(d: &WorkspaceData, now: NaiveDateTime, real_ms: i64, pick: Option<&str>) -> HomeModel {
    let t0 = now.date();
    let mut m = HomeModel::default();
    let name = d.intakes.iter().filter(|i| !i.name.trim().is_empty()).max_by_key(|i| i.updated).map(|i| i.name.trim().to_string());
    let part = match now.hour() {
        5..=11 => "Good morning",
        12..=17 => "Good afternoon",
        _ => "Good evening",
    };
    m.greeting = match &name {
        Some(n) => format!("{part}, {n}"),
        None => part.to_string(),
    };

    // today: events on visible calendars, and open tasks due today
    let visible = |e: &Event| d.calendar(&e.calendar_id).map_or(true, |c| c.visible);
    for e in d.events.iter().filter(|e| visible(e)) {
        if let Some(start) = e.occurrence_on(t0) {
            let mins = (e.end - e.start).num_minutes();
            let cal = d.calendar(&e.calendar_id).map(|c| format!(" · {} calendar", c.name)).unwrap_or_default();
            let (time, sort) = if e.all_day { ("All day".to_string(), NaiveTime::MIN) } else { (fmt_time(start.time()), start.time()) };
            let meta = if e.all_day { format!("Event{cal}") } else { format!("Event · {mins} min{cal}") };
            m.agenda.push(AgendaItem { time, sort, title: e.title.clone(), meta, kind: AgendaKind::Event, task: None, event: Some(e.id.clone()) });
        }
    }
    let n_events = m.agenda.len();
    let due: Vec<&Task> = d.tasks.iter().filter(|t| !t.completed && t.parent_id.is_none() && t.due == Some(t0)).collect();
    for t in &due {
        let project = t.project_id.as_deref().and_then(|p| d.project(p));
        let (time, sort) = match t.due_time {
            Some(tm) => (fmt_time(tm), tm),
            // a task for today with no time sits at "now": it's for doing now
            None => ("Today".to_string(), now.time()),
        };
        m.agenda.push(AgendaItem {
            time,
            sort,
            title: t.title.clone(),
            meta: format!("Task · {}", project.map(|p| p.name.clone()).unwrap_or_else(|| "Captured".into())),
            kind: if project.is_some() { AgendaKind::Task } else { AgendaKind::Captured },
            task: Some(t.id.clone()),
            event: None,
        });
    }
    m.agenda.sort_by_key(|a| a.sort);
    m.subline = if due.is_empty() && n_events == 0 {
        format!("{} · nothing scheduled yet", t0.format("%A %-d %B"))
    } else {
        format!("{} · {} due today · {}", t0.format("%A %-d %B"), due.len(), plural(n_events, "event", "events"))
    };

    // the hero: the guided project whose first move is due soonest
    let mut best: Option<(i64, i64, HeroMove)> = None;
    for c in &d.contexts {
        let Some(pr) = d.project(&c.project_id).filter(|p| !p.archived) else { continue };
        if pick.map_or(false, |x| x != pr.id) {
            continue;
        }
        let a = crate::resolve::assess::assess(d, c);
        let win = a.win.as_ref().and_then(|w| d.task(w)).filter(|t| !t.completed);
        let intake = d.intakes.iter().find(|i| i.project_id.as_deref() == Some(pr.id.as_str()));
        let first_move = intake.and_then(|i| i.proposal.as_ref()).map(|p| p.next_move.trim().to_string()).filter(|s| !s.is_empty());
        let title = first_move.or_else(|| win.map(|t| t.title.clone())).or_else(|| a.next.first().map(|x| x.0.clone()));
        let Some(title) = title else { continue };
        let why = Some(c.recommendation.why.trim().to_string()).filter(|s| !s.is_empty()).or_else(|| intake.and_then(|i| i.proposal.as_ref()).map(|p| p.why.trim().to_string())).unwrap_or_default();
        let theme = win
            .and_then(|t| d.records.iter().find(|r| r.kind == crate::resolve::records::Kind::Roadmap && r.project_id.as_deref() == Some(pr.id.as_str()) && r.links.contains(&t.id)))
            .map(|r| r.title.clone())
            .unwrap_or_default();
        let due_key = win.and_then(|t| t.due).map(|x| (x - t0).num_days()).unwrap_or(i64::MAX);
        let hero = HeroMove { project_id: pr.id.clone(), project: pr.name.clone(), title, why, theme, task: win.map(|t| t.id.clone()) };
        if best.as_ref().map_or(true, |(k, o, _)| (due_key, pr.order) < (*k, *o)) {
            best = Some((due_key, pr.order, hero));
        }
    }
    m.hero = best.map(|b| b.2);

    // since Monday: what was finished this week, oldest first
    let monday = t0 - chrono::Duration::days(t0.weekday().num_days_from_monday() as i64);
    let monday_ms = real_ms - (now - monday.and_time(NaiveTime::MIN)).num_milliseconds();
    let mut wins: Vec<&Task> = d.tasks.iter().filter(|t| t.completed && t.completed_at.map_or(false, |c| c >= monday_ms)).collect();
    wins.sort_by_key(|t| t.completed_at);
    m.wins = wins.iter().map(|t| t.title.clone()).collect();

    // waiting on: tasks marked waiting (days since that date), then project dependencies still waiting
    let mut waiting_tasks: Vec<&Task> = d.tasks.iter().filter(|t| !t.completed && t.waiting_since.is_some()).collect();
    waiting_tasks.sort_by_key(|t| t.waiting_since);
    for t in waiting_tasks {
        if t.project_id.as_deref().and_then(|p| d.project(p)).map_or(false, |p| p.archived) {
            continue;
        }
        let days = ((real_ms - t.waiting_since.unwrap_or(real_ms)) / (24 * 3600 * 1000)).max(0) as usize;
        m.waiting.push((t.title.clone(), if days == 0 { "today".to_string() } else { plural(days, "day", "days") }));
    }
    for r in d.records.iter().filter(|r| r.kind == crate::resolve::records::Kind::Dependency && r.status == "Waiting") {
        if r.project_id.as_deref().and_then(|p| d.project(p)).map_or(false, |p| p.archived) {
            continue;
        }
        let days = ((real_ms - r.created) / (24 * 3600 * 1000)).max(0) as usize;
        let age = if days == 0 { "today".to_string() } else { plural(days, "day", "days") };
        m.waiting.push((r.title.clone(), age));
    }

    let mut ps: Vec<&Project> = d.projects.iter().filter(|p| !p.archived).collect();
    ps.sort_by_key(|p| p.order);
    for pr in ps {
        let all: Vec<&Task> = d.tasks.iter().filter(|t| t.project_id.as_deref() == Some(pr.id.as_str()) && t.parent_id.is_none()).collect();
        if all.is_empty() && !d.contexts.iter().any(|c| c.project_id == pr.id) {
            continue;
        }
        m.projects.push((pr.id.clone(), pr.name.clone(), all.iter().filter(|t| t.completed).count(), all.len()));
    }
    m.captured = d.tasks.iter().filter(|t| !t.completed && t.project_id.is_none() && t.parent_id.is_none()).count();
    m.setup = crate::ui::resolve::open_intakes(d).into_iter().next().map(|(id, _, step)| (id, RAIL_STEPS[(step.clamp(1, 4) - 1) as usize]));
    m
}

/// What a click on Home asked for.
enum HomeAct {
    NewProject,
    Resume(Id),
    StartThis(Id, Option<Id>),
    PlanDay,
    OpenCalendar,
    OpenTask(Id),
    Toggle(Id),
    OpenEvent(Id),
    AllProjects,
    OpenProject(Id),
    Captured,
    RetryPlan,
    OpenToday,
    SamplePersonal,
}

impl App {
    pub fn dashboard(&mut self, ui: &mut egui::Ui, p: &Pal) {
        let m = home_model(&self.store.data, now_local(), now_ms(), self.current_project.as_deref());
        let state = match self.view_state {
            ViewState::Full if m.hero.is_none() && m.projects.is_empty() => ViewState::Empty,
            ViewState::Full if self.plan.error.is_some() => ViewState::Error,
            s => s,
        };
        let wide = ui.ctx().screen_rect().width() > space::WIDE_FROM;
        let mut acts: Vec<HomeAct> = vec![];
        let el = ((self.now - self.view_since) * 1000.0) as f32;
        let reduced = motion::reduced();
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            egui::Frame::none().inner_margin(egui::Margin { left: space::WORK_SIDE, right: space::WORK_SIDE, top: space::WORK_TOP, bottom: space::S24 }).show(ui, |ui| {
                let cap = if wide { space::CAP_DEFAULT } else { ui.available_width() };
                capped(ui, cap, |ui| {
                    // header: 340 e-out, y −8 → 0
                    motion::fade_in(ui, frame_at(el, moves::HEADER, reduced), egui::vec2(0.0, -space::S8), |ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(RichText::new(&m.greeting).font(fserif(size::GREETING)).color(p.text));
                                ui.label(RichText::new(&m.subline).size(size::ROW).color(p.dim));
                            });
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if button(ui, p, ButtonKind::Secondary, "+ New project", false, true).clicked() {
                                    acts.push(HomeAct::NewProject);
                                }
                            });
                        });
                    });
                    if self.store.meta.name == crate::store::SAMPLE_NAME {
                        ui.add_space(space::S12);
                        if amber_card(ui, p, None, "You're looking at sample data, kept apart from your own.", Some("Your real workspace is \"Personal\"."), "Open Personal").clicked() {
                            acts.push(HomeAct::SamplePersonal);
                        }
                    }
                    ui.add_space(space::S22);
                    let w = ui.available_width();
                    let aside_w = ((w - space::HOME_GAP) * (1.0 - space::HOME_LEFT_SHARE)).min(space::HOME_ASIDE_MAX);
                    let left_w = w - space::HOME_GAP - aside_w;
                    ui.horizontal_top(|ui| {
                        ui.spacing_mut().item_spacing.x = space::HOME_GAP;
                        let card = |i: usize| frame_at(el, moves::CARDS.stagger(i, moves::CARDS_STAGGER), reduced);
                        let rise = egui::vec2(0.0, space::S18);
                        ui.allocate_ui_with_layout(egui::vec2(left_w, 0.0), Layout::top_down(Align::Min), |ui| {
                            ui.set_width(left_w);
                            ui.spacing_mut().item_spacing.y = space::S16;
                            match state {
                                ViewState::Loading => {
                                    skeleton(ui, p, left_w, space::HOME_SKELETON_HERO);
                                    skeleton_card(ui, p, 3, "");
                                }
                                ViewState::Empty => {
                                    motion::fade_in(ui, card(0), rise, |ui| {
                                        white_card(ui, p, false, |ui| {
                                            ui.label(RichText::new("Start here").font(fs(size::CARD_HEAD)).color(p.orange));
                                            ui.label(RichText::new("Bring in your first project.").font(fserif(size::STEP_TITLE)).color(p.text));
                                            wrap_label(ui, RichText::new("Tell Resolve what's going on, however messy. You'll get one clear first move and a roadmap you can change any time.").size(size::HERO_BODY).color(p.dim));
                                            ui.add_space(space::S10);
                                            if button(ui, p, ButtonKind::Primary, "New project →", false, true).clicked() {
                                                acts.push(HomeAct::NewProject);
                                            }
                                        });
                                    });
                                }
                                ViewState::Error => {
                                    motion::fade_in(ui, card(0), rise, |ui| {
                                        white_card(ui, p, false, |ui| {
                                            ui.label(RichText::new("Resolve couldn't plan your day just now.").font(fserif(size::CARD_SERIF)).color(p.text));
                                            let why = self.plan.error.clone().unwrap_or_else(|| "The AI didn't answer.".into());
                                            wrap_label(ui, RichText::new(format!("{} Your tasks and calendar are below, and your first move is still on the project.", why.trim_end_matches('.').to_string() + ".")).size(size::ROW).color(p.dim));
                                            ui.add_space(space::S10);
                                            ui.horizontal(|ui| {
                                                if button(ui, p, ButtonKind::Dark, "Try again", false, true).clicked() {
                                                    acts.push(HomeAct::RetryPlan);
                                                }
                                                if button(ui, p, ButtonKind::Secondary, "Open today's tasks", false, true).clicked() {
                                                    acts.push(HomeAct::OpenToday);
                                                }
                                            });
                                        });
                                    });
                                }
                                ViewState::Full => {
                                    if let Some(h) = &m.hero {
                                        motion::fade_in(ui, card(0), rise, |ui| {
                                            hero_card(ui, p, |ui| {
                                                ui.spacing_mut().item_spacing.y = space::S10;
                                                hero_eyebrow(ui, "Your highest-value move", &h.project);
                                                ui.add(egui::Label::new(RichText::new(&h.title).font(fserif(if wide { size::HERO_TITLE_WIDE } else { size::HERO_TITLE })).color(tok::TEXT_ON_DARK)).wrap());
                                                if !h.why.is_empty() {
                                                    ui.add(egui::Label::new(RichText::new(&h.why).size(size::HERO_BODY).color(tok::HERO_BODY)).wrap());
                                                }
                                                if !h.theme.is_empty() {
                                                    dot_line(ui, &format!("Focus theme · {}", h.theme), tok::AMBER, tok::HERO_BODY);
                                                }
                                                ui.add_space(space::S6);
                                                ui.horizontal(|ui| {
                                                    if button(ui, p, ButtonKind::Primary, "Start this →", false, true).clicked() {
                                                        acts.push(HomeAct::StartThis(h.project_id.clone(), h.task.clone()));
                                                    }
                                                    if button(ui, p, ButtonKind::OnDark, "✦ Plan my day", false, true).clicked() {
                                                        acts.push(HomeAct::PlanDay);
                                                    }
                                                });
                                            });
                                        });
                                    }
                                }
                            }
                            if let Some((id, step)) = &m.setup {
                                motion::fade_in(ui, card(1), rise, |ui| {
                                    if amber_card(ui, p, None, "Continue setting up", Some(&format!("You stopped at {step}. Everything you wrote is saved.")), "Pick up →").clicked() {
                                        acts.push(HomeAct::Resume(id.clone()));
                                    }
                                });
                            }
                            if matches!(state, ViewState::Full | ViewState::Error) {
                                motion::fade_in(ui, card(2), rise, |ui| {
                                    white_card(ui, p, false, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new("Today").font(fb(size::LABEL)).color(p.text));
                                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                                if ui.add(egui::Label::new(RichText::new("Open calendar →").font(fs(size::SMALL)).color(p.orange)).sense(egui::Sense::click())).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                                    acts.push(HomeAct::OpenCalendar);
                                                }
                                            });
                                        });
                                        if m.agenda.is_empty() {
                                            ui.add_space(space::S8);
                                            ui.label(RichText::new("Nothing scheduled. Events and dated tasks show here.").size(size::ROW_SMALL).color(p.dim));
                                        }
                                        for (i, a) in m.agenda.iter().enumerate() {
                                            let mark = match a.kind {
                                                AgendaKind::Event => Mark::Event(p.orange),
                                                AgendaKind::Task => Mark::Task(tok::AMBER),
                                                AgendaKind::Captured => Mark::Task(p.faint),
                                            };
                                            let (row, mk) = agenda_row(ui, p, &a.time, mark, &a.title, &a.meta, i == 0);
                                            if let Some(t) = &a.task {
                                                if mk.clicked() {
                                                    acts.push(HomeAct::Toggle(t.clone()));
                                                } else if row.clicked() {
                                                    acts.push(HomeAct::OpenTask(t.clone()));
                                                }
                                            } else if let Some(e) = &a.event {
                                                if row.clicked() || mk.clicked() {
                                                    acts.push(HomeAct::OpenEvent(e.clone()));
                                                }
                                            }
                                        }
                                    });
                                });
                            }
                        });
                        // the right column
                        ui.allocate_ui_with_layout(egui::vec2(aside_w, 0.0), Layout::top_down(Align::Min), |ui| {
                            ui.set_width(aside_w);
                            ui.spacing_mut().item_spacing.y = space::S12;
                            match state {
                                ViewState::Loading => {
                                    for _ in 0..3 {
                                        skeleton_card(ui, p, 2, "");
                                    }
                                    ui.label(RichText::new("Loading your day…").size(size::SMALL).color(p.dim));
                                }
                                ViewState::Empty => {
                                    motion::fade_in(ui, card(1), rise, |ui| {
                                        white_card(ui, p, false, |ui| {
                                            card_head(ui, p, "TODAY", None, false);
                                            wrap_label(ui, RichText::new("Nothing scheduled. Events and dated tasks show here.").size(size::ROW_SMALL).color(p.dim));
                                        });
                                    });
                                    motion::fade_in(ui, card(2), rise, |ui| {
                                        white_card(ui, p, true, |ui| {
                                            card_head(ui, p, "CAPTURED", None, false);
                                            wrap_label(ui, RichText::new("Nothing yet. Use Add task or the bar below to note anything on your mind.").size(size::ROW_SMALL).color(p.dim));
                                        });
                                    });
                                }
                                ViewState::Error => {
                                    white_card(ui, p, false, |ui| {
                                        wrap_label(ui, RichText::new("Only the plan failed. Everything else on Home loads from your own data, which is saved on this PC.").size(size::ROW_SMALL).color(p.dim));
                                    });
                                }
                                ViewState::Full => {
                                    let mut n = 1;
                                    if !m.wins.is_empty() {
                                        motion::fade_in(ui, card(n), rise, |ui| {
                                            white_card(ui, p, false, |ui| {
                                                card_head(ui, p, "SINCE MONDAY", Some((&format!("{} done", m.wins.len()), p.green_text)), false);
                                                for w in m.wins.iter().take(5) {
                                                    win_row(ui, p, w);
                                                }
                                            });
                                        });
                                        n += 1;
                                    }
                                    if !m.waiting.is_empty() {
                                        motion::fade_in(ui, card(n), rise, |ui| {
                                            white_card(ui, p, false, |ui| {
                                                card_head(ui, p, "WAITING ON", Some((&format!("{} stalled", m.waiting.len()), p.orange)), false);
                                                for (what, age) in m.waiting.iter().take(4) {
                                                    stalled_row(ui, p, what, age);
                                                }
                                            });
                                        });
                                        n += 1;
                                    }
                                    if !m.projects.is_empty() {
                                        motion::fade_in(ui, card(n), rise, |ui| {
                                            white_card(ui, p, false, |ui| {
                                                if card_head(ui, p, "PROJECTS", Some(("All →", p.orange)), true) {
                                                    acts.push(HomeAct::AllProjects);
                                                }
                                                for (id, name, done, total) in m.projects.iter().take(5) {
                                                    let frac = if *total == 0 { 0.0 } else { *done as f32 / *total as f32 };
                                                    if progress_row(ui, p, name, &format!("{done} of {total}"), frac).clicked() {
                                                        acts.push(HomeAct::OpenProject(id.clone()));
                                                    }
                                                }
                                            });
                                        });
                                        n += 1;
                                    }
                                    if m.captured > 0 {
                                        motion::fade_in(ui, card(n), rise, |ui| {
                                            if amber_card(ui, p, Some(&m.captured.to_string()), "captured, not in a project yet", None, "Sort →").clicked() {
                                                acts.push(HomeAct::Captured);
                                            }
                                        });
                                    }
                                }
                            }
                        });
                    });
                });
            });
        });
        if el < moves::CARDS.stagger(5, moves::CARDS_STAGGER).end() {
            ui.ctx().request_repaint();
        }
        for a in acts {
            match a {
                HomeAct::NewProject => self.start_intake(),
                HomeAct::Resume(id) => self.go(View::Intake(id)),
                HomeAct::StartThis(pid, task) => {
                    self.current_project = Some(pid.clone());
                    self.go(View::Project(pid));
                    self.open_task = task;
                }
                HomeAct::PlanDay => self.open_plan_day(),
                HomeAct::RetryPlan => {
                    self.plan.error = None;
                    self.open_plan_day();
                }
                HomeAct::OpenCalendar => self.go(View::Calendar),
                HomeAct::OpenTask(id) => self.open_task = Some(id),
                HomeAct::Toggle(id) => self.toggle_task(&id),
                HomeAct::OpenEvent(id) => {
                    if let Some(e) = self.store.data.events.iter().find(|e| e.id == id).cloned() {
                        self.event_edit = Some(calendar::draft_from(&e));
                    }
                }
                HomeAct::AllProjects => self.go(View::Projects),
                HomeAct::OpenProject(id) => {
                    self.select_project(Some(id));
                    self.go(View::Sec(nav::Section::Overview));
                }
                HomeAct::Captured => self.go(View::Inbox),
                HomeAct::OpenToday => self.go(View::Today),
                HomeAct::SamplePersonal => {
                    if let Some(w) = self.reg.find("Personal").map(|w| w.id.clone()) {
                        self.switch_workspace(&w);
                    }
                }
            }
        }
        let _ = Local::now();
    }
}
