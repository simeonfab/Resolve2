//! Plan my day (B-E21, B-E22, B-E23): a sheet over Home, 760 wide and 88% high. The AI is asked
//! for a structured plan (time, item, reason, what won't fit) and the reply is matched back to
//! the real tasks and events; if it fails, Home says so in its own card (B-E6) and the sheet lists
//! today by time. The data and parsing are pure functions, self-checked.

use super::motion::{self, frame_at};
use super::theme::*;
use super::widgets::*;
use super::*;
use chrono::Datelike;
use eframe::egui::{self, RichText};

#[derive(Clone, Debug, PartialEq)]
pub struct PlanRow {
    /// "09:30 – 11:00", or "16:00" for a fixed time
    pub time: String,
    pub item: String,
    pub why: String,
    /// "Task", "Event" or "Overdue"
    pub tag: String,
    pub task: Option<Id>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Plan {
    pub rows: Vec<PlanRow>,
    /// (item, why, task)
    pub wont_fit: Vec<(String, String, Option<Id>)>,
    pub clashes: usize,
}

/// The Plan my day sheet's state.
#[derive(Default)]
pub struct PlanUi {
    /// The sheet is open.
    pub open: bool,
    /// The last attempt failed: what went wrong (Home shows it in its own card, B-E6).
    pub error: Option<String>,
    pub job: Option<crate::ai::AiJob>,
    pub started: f64,
    pub plan: Option<Plan>,
    /// When the plan was made (the app's clock), for "From your agenda at 09:12".
    pub made_at: Option<NaiveTime>,
    /// The change box (B-E23): a constraint for the next plan.
    pub change: String,
    pub was_open: bool,
    pub moved_at: f64,
}

/// Today as the plan sees it: (time label, title, tag, task) for events, tasks due today and
/// overdue tasks, plus (events, tasks, overdue) counts.
pub fn plan_agenda(d: &WorkspaceData, now: NaiveDateTime) -> (Vec<(String, String, String, Option<Id>)>, (usize, usize, usize)) {
    let t0 = now.date();
    let mut v: Vec<(NaiveTime, String, String, String, Option<Id>)> = vec![];
    for e in d.events.iter().filter(|e| d.calendar(&e.calendar_id).map_or(true, |c| c.visible)) {
        if let Some(s) = e.occurrence_on(t0) {
            let time = if e.all_day { "All day".to_string() } else { fmt_time(s.time()) };
            v.push((if e.all_day { NaiveTime::MIN } else { s.time() }, time, e.title.clone(), "Event".into(), None));
        }
    }
    let n_events = v.len();
    let mut n_tasks = 0;
    let mut n_over = 0;
    for t in d.tasks.iter().filter(|t| !t.completed && t.parent_id.is_none()) {
        match t.due {
            Some(x) if x == t0 => {
                n_tasks += 1;
                let (sort, time) = match t.due_time {
                    Some(tm) => (tm, fmt_time(tm)),
                    None => (NaiveTime::from_hms_opt(23, 58, 0).unwrap(), "No time".to_string()),
                };
                v.push((sort, time, t.title.clone(), "Task".into(), Some(t.id.clone())));
            }
            Some(x) if x < t0 => {
                n_over += 1;
                v.push((NaiveTime::from_hms_opt(23, 59, 0).unwrap(), "Overdue".into(), t.title.clone(), "Overdue".into(), Some(t.id.clone())));
            }
            _ => {}
        }
    }
    v.sort_by_key(|x| x.0);
    (v.into_iter().map(|x| (x.1, x.2, x.3, x.4)).collect(), (n_events, n_tasks + n_over, n_over))
}

/// The request: today's real items, and the structured answer wanted.
pub fn plan_prompt(d: &WorkspaceData, now: NaiveDateTime, change: &str) -> String {
    let (items, _) = plan_agenda(d, now);
    let mut s = format!("It is {} at {}. Plan the rest of today in order.\n\nToday's items:\n", now.format("%A %-d %B"), now.format("%H:%M"));
    for (time, title, tag, task) in &items {
        let project = task.as_deref().and_then(|t| d.task(t)).and_then(|t| t.project_id.clone()).and_then(|p| d.project(&p).map(|p| p.name.clone())).unwrap_or_default();
        s.push_str(&format!("- [{tag}] {title} ({time}){}\n", if project.is_empty() { String::new() } else { format!(", project: {project}") }));
    }
    if !change.trim().is_empty() {
        s.push_str(&format!("\nThe person asks: {}\n", change.trim()));
    }
    s.push_str("\nKeep events at their times. Put the most important task first while the day is clear. Leave out what will not fit and say why.\nReply with JSON only, no other text:\n{\"rows\":[{\"start\":\"09:30\",\"end\":\"11:00\",\"item\":\"exact title from the list\",\"why\":\"one short sentence\"}],\"wont_fit\":[{\"item\":\"exact title\",\"why\":\"one short sentence\"}],\"clashes\":0}");
    s
}

pub const PLAN_SYSTEM: &str = "You plan one person's day from their own calendar and task list. Use only the items given, with their exact titles. Never invent tasks, events or times outside the day. Answer with JSON only.";

/// Read the AI's reply and match each item back to a real task or event. Items not in today's
/// list are dropped (never shown as if they were the person's).
pub fn parse_plan(text: &str, d: &WorkspaceData, now: NaiveDateTime) -> Result<Plan, String> {
    let v = crate::ai::parse_json_object(text)?;
    let (items, _) = plan_agenda(d, now);
    let find = |name: &str| items.iter().find(|i| i.1.eq_ignore_ascii_case(name.trim()));
    let mut plan = Plan::default();
    for r in v.get("rows").and_then(|x| x.as_array()).cloned().unwrap_or_default() {
        let item = r.get("item").and_then(|x| x.as_str()).unwrap_or("").to_string();
        let Some(known) = find(&item) else { continue };
        let start = r.get("start").and_then(|x| x.as_str()).unwrap_or("").trim().to_string();
        let end = r.get("end").and_then(|x| x.as_str()).unwrap_or("").trim().to_string();
        let time = if end.is_empty() || end == start { start.clone() } else { format!("{start} – {end}") };
        plan.rows.push(PlanRow { time, item: known.1.clone(), why: r.get("why").and_then(|x| x.as_str()).unwrap_or("").trim().to_string(), tag: known.2.clone(), task: known.3.clone() });
    }
    for r in v.get("wont_fit").and_then(|x| x.as_array()).cloned().unwrap_or_default() {
        let item = r.get("item").and_then(|x| x.as_str()).unwrap_or("").to_string();
        if let Some(known) = find(&item) {
            plan.wont_fit.push((known.1.clone(), r.get("why").and_then(|x| x.as_str()).unwrap_or("").trim().to_string(), known.3.clone()));
        }
    }
    plan.clashes = v.get("clashes").and_then(|x| x.as_u64()).unwrap_or(0) as usize;
    if plan.rows.is_empty() {
        return Err("The AI's plan didn't name any of today's items.".into());
    }
    Ok(plan)
}

/// The next Monday after `d` (Move to Monday, B-E23).
pub fn next_monday(d: NaiveDate) -> NaiveDate {
    let n = 7 - d.weekday().num_days_from_monday() as i64;
    d + chrono::Duration::days(n)
}

impl App {
    /// Open Plan my day, asking the AI if there is no plan yet.
    pub fn open_plan_day(&mut self) {
        self.plan.open = true;
        if self.plan.plan.is_none() && self.plan.job.is_none() {
            self.start_plan();
        }
    }

    fn start_plan(&mut self) {
        let cfg = self.ai_config();
        let prompt = plan_prompt(&self.store.data, now_local(), &self.plan.change);
        self.plan.job = Some(crate::ai::ask(&cfg.provider, &cfg.url, &cfg.model, PLAN_SYSTEM, &prompt, "Plan my day"));
        self.plan.started = self.now;
        self.plan.error = None;
    }

    /// Collect a finished plan, whichever screen is open.
    pub(super) fn poll_plan(&mut self, ctx: &egui::Context) {
        let done = self.plan.job.as_ref().and_then(|j| j.poll());
        if let Some(res) = done {
            self.plan.job = None;
            match res.and_then(|t| parse_plan(&t, &self.store.data, now_local())) {
                Ok(p) => {
                    self.plan.plan = Some(p);
                    self.plan.made_at = Some(now_local().time());
                    self.plan.error = None;
                }
                Err(e) => {
                    self.plan.plan = None;
                    self.plan.error = Some("The AI didn't answer.".into());
                    crate::ai::set_last_error(&format!("Plan my day: {e}"));
                }
            }
        }
        if self.plan.job.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(250));
        }
    }

    /// The sheet: rises from the bottom (460 e-spring), 760 wide, 88% high, over a .32 backdrop.
    pub(super) fn plan_sheet(&mut self, ctx: &egui::Context, p: &Pal) {
        let open = self.plan.open;
        if open != self.plan.was_open {
            self.plan.was_open = open;
            self.plan.moved_at = self.now;
        }
        let el = ((self.now - self.plan.moved_at) * 1000.0) as f32;
        let reduced = motion::reduced();
        let (dim, down, live) = if open {
            let d = frame_at(el, moves::SHEET_DIM, reduced);
            let s = frame_at(el, moves::SHEET_IN, reduced);
            if !(d.done && s.done) {
                ctx.request_repaint();
            }
            (d.alpha, 1.0 - s.t, true)
        } else {
            if self.plan.moved_at == 0.0 {
                return;
            }
            let s = frame_at(el, moves::SHEET_OUT, reduced);
            if s.done {
                return;
            }
            ctx.request_repaint();
            (1.0 - s.alpha, s.t, false)
        };
        let screen = ctx.screen_rect();
        let work = self.toaster.work.filter(|r| r.is_positive()).unwrap_or(screen);
        let w = space::PLAN_SHEET_W.min(work.width() - space::S24 * 2.0);
        let h = (screen.height() * space::PLAN_SHEET_HEIGHT).min(work.height());
        let rect = egui::Rect::from_min_size(egui::pos2(work.center().x - w / 2.0, work.bottom() - h + h * down.max(0.0)), egui::vec2(w, h));
        let mut close = false;
        egui::Area::new(egui::Id::new("plan-dim")).order(egui::Order::Foreground).fixed_pos(screen.min).interactable(live).show(ctx, |ui| {
            close = backdrop(ui, screen, shadow::BACKDROP_SHEET * dim.clamp(0.0, 1.0));
        });
        let id = egui::Id::new("plan-sheet");
        egui::Area::new(id).order(egui::Order::Foreground).fixed_pos(rect.min).interactable(live).constrain(false).show(ctx, |ui| {
            let frame = sheet_frame(p);
            let m = frame.inner_margin;
            frame.show(ui, |ui| {
                ui.set_width(w - m.left - m.right);
                ui.set_height(h - m.top - m.bottom);
                if self.plan_body(ui, p, el, live) {
                    close = true;
                }
            });
        });
        ctx.move_to_top(egui::LayerId::new(egui::Order::Foreground, id));
        if close && live {
            self.plan.open = false;
        }
    }

    fn plan_body(&mut self, ui: &mut egui::Ui, p: &Pal, el: f32, live: bool) -> bool {
        let now = now_local();
        let (items, (n_ev, n_tasks, n_over)) = plan_agenda(&self.store.data, now);
        let state = match self.view_state {
            ViewState::Full if self.plan.job.is_some() => ViewState::Loading,
            ViewState::Full if self.plan.error.is_some() => ViewState::Error,
            ViewState::Full if items.is_empty() => ViewState::Empty,
            s => s,
        };
        let mut close = false;
        let mut again = false;
        let mut move_monday: Option<Id> = None;
        let mut upcoming = false;
        let rise = egui::vec2(0.0, space::S8);
        let row = |i: usize| if live { frame_at(el, moves::SHEET_ROWS.stagger(i, moves::SHEET_ROWS_STAGGER), motion::reduced()) } else { frame_at(f32::MAX, moves::SHEET_ROWS, false) };
        // header
        ui.horizontal(|ui| {
            icon_widget(ui, Icon::Spark, tok::AMBER);
            ui.add_space(space::S8);
            ui.vertical(|ui| {
                ui.label(RichText::new(format!("Your plan for {}", now.format("%A"))).font(fserif(size::CARD_SERIF + 2.0)).color(p.text));
                let sub = match (state, self.plan.made_at) {
                    (ViewState::Full, Some(at)) => {
                        let clash = self.plan.plan.as_ref().map_or(0, |x| x.clashes);
                        let mut s = format!("From your agenda at {} · {} · {}{}", fmt_time(at), plural(n_ev, "event", "events"), plural(n_tasks, "task", "tasks"), if n_over > 0 { format!(", {n_over} overdue") } else { String::new() });
                        if clash > 0 {
                            s.push_str(&format!(" · {} found", plural(clash, "clash", "clashes")));
                        }
                        s.push_str(&format!(" · planned by {}", self.provider_label()));
                        s
                    }
                    _ => now.format("%A %-d %B").to_string(),
                };
                ui.label(RichText::new(sub).size(size::LABEL).color(p.dim));
            });
            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                if icon_button(ui, Icon::Close, true, p.dark).on_hover_text("Close (Esc)").clicked() {
                    close = true;
                }
            });
        });
        ui.add_space(space::S14);
        let foot_h = space::BUTTON_H * 2.0 + space::FIELD_H + space::S40 + space::S12;
        let body_h = (ui.available_height() - foot_h).max(space::S40 * 3.0);
        egui::ScrollArea::vertical().max_height(body_h).auto_shrink([false, false]).show(ui, |ui| match state {
            ViewState::Loading => {
                ui.label(RichText::new(format!("Reading today's {} and {}, overdue included…", plural(n_ev, "event", "events"), plural(n_tasks, "task", "tasks"))).size(size::ROW_SMALL).color(p.dim));
                ui.add_space(space::S10);
                for _ in 0..4 {
                    skeleton(ui, p, ui.available_width(), space::S40 + space::S10);
                    ui.add_space(space::S8);
                }
            }
            ViewState::Empty => {
                ui.add_space(space::S24);
                ui.label(RichText::new("Nothing to plan today.").font(fserif(size::CARD_SERIF)).color(p.text));
                ui.add_space(space::S10);
                ui.horizontal(|ui| {
                    if button(ui, p, ButtonKind::Secondary, "Show Upcoming", false, true).clicked() {
                        upcoming = true;
                    }
                    if button(ui, p, ButtonKind::Ghost, "Close", false, true).clicked() {
                        close = true;
                    }
                });
            }
            ViewState::Error => {
                white_card(ui, p, false, |ui| {
                    ui.spacing_mut().item_spacing.y = space::S6;
                    ui.label(RichText::new("The AI didn't answer, so there's no suggested order.").font(fs(size::ROW)).color(p.text));
                    ui.label(RichText::new("Here is today by time. Nothing else was lost.").size(size::LABEL).color(p.dim));
                    ui.add_space(space::S6);
                    for (time, title, _, _) in &items {
                        ui.horizontal(|ui| {
                            ui.add_sized([space::PLAN_TIME_W, size::ROW + space::S6], egui::Label::new(RichText::new(time).size(size::LABEL).color(p.dim)));
                            ui.label(RichText::new(title).size(size::ROW).color(p.text));
                        });
                    }
                    ui.add_space(space::S6);
                    if button(ui, p, ButtonKind::Dark, "Try again", false, true).clicked() {
                        again = true;
                    }
                });
            }
            ViewState::Full => {
                let plan = self.plan.plan.clone().unwrap_or_default();
                motion::fade_in(ui, row(0), rise, |ui| {
                    list_box(ui, p, |ui| {
                        for (i, r) in plan.rows.iter().enumerate() {
                            plan_row(ui, p, &r.time, &r.item, &r.why, &r.tag, i == 0);
                        }
                    });
                });
                for (k, (item, why, task)) in plan.wont_fit.iter().enumerate() {
                    ui.add_space(space::S12);
                    motion::fade_in(ui, row(k + 1), rise, |ui| {
                        egui::Frame::none().fill(p.orange_tint).rounding(egui::Rounding::same(radius::CARD)).inner_margin(egui::Margin::symmetric(space::S16, space::S12)).show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(RichText::new("Won't fit today.").font(fs(size::ROW_SMALL)).color(p.orange));
                                    ui.add(egui::Label::new(RichText::new(format!("{item}. {why}")).size(size::LABEL).color(p.text)).wrap());
                                });
                                if let Some(t) = task {
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        if button(ui, p, ButtonKind::Secondary, "Move to Monday", true, true).clicked() {
                                            move_monday = Some(t.clone());
                                        }
                                    });
                                }
                            });
                        });
                    });
                }
            }
        });
        // footer: Use this plan, Plan again, the change box
        ui.add_space(space::S12);
        let decided = false; // B-E22: where a planned start time is kept is Simeon's decision (see the Phase 2 report)
        let full = state == ViewState::Full;
        let r = button(ui, p, ButtonKind::Primary, "Use this plan", false, full && decided);
        if !decided {
            r.on_hover_text("Waiting for a decision on where a planned start time is kept (B-E22). Nothing is written yet.");
        }
        ui.add_space(space::S8);
        ui.horizontal(|ui| {
            if button(ui, p, ButtonKind::Secondary, "Plan again", false, self.plan.job.is_none()).clicked() {
                again = true;
            }
            let cid = egui::Id::new("plan-change");
            let w = ui.available_width() - space::MIC - space::S10;
            let r = text_field(ui, p, cid, &mut self.plan.change, "Change something: I can't start before 10", false, w);
            if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) && !self.plan.change.trim().is_empty() {
                again = true;
            }
            mic_for(ui, p, cid, p.dark);
        });
        ui.label(RichText::new("Use this plan gives each task today a start time. Nothing moves in your calendar.").size(size::SMALL).color(p.dim));
        if again && self.plan.job.is_none() {
            self.start_plan();
        }
        if let Some(t) = move_monday {
            let to = next_monday(now.date());
            let title = self.store.data.task(&t).map(|x| x.title.clone()).unwrap_or_default();
            self.undoable(&format!("Moved \"{title}\" to Monday"), "move", move |d| {
                if let Some(x) = d.task_mut(&t) {
                    x.due = Some(to);
                    x.updated = now_ms();
                }
            });
            if let Some(pl) = self.plan.plan.as_mut() {
                pl.wont_fit.retain(|w| w.0 != title);
            }
        }
        if upcoming {
            self.plan.open = false;
            self.go(View::Upcoming);
        }
        close
    }
}

fn plural(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}
