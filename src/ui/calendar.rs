use super::theme::*;
use super::widgets::*;
use super::*;
use chrono::{Datelike, Duration, Months};
use eframe::egui::{self, Color32, Pos2, Rect, RichText, Sense, Stroke};

pub fn draft_from(e: &Event) -> EventDraft {
    EventDraft {
        id: Some(e.id.clone()),
        title: e.title.clone(),
        calendar_id: e.calendar_id.clone(),
        date: e.start.date(),
        end_date: if e.all_day { (e.end - Duration::seconds(1)).date().max(e.start.date()) } else { e.end.date() },
        start: if e.all_day { None } else { Some(e.start.time()) },
        end: if e.all_day { None } else { Some(e.end.time()) },
        all_day: e.all_day,
        location: e.location.clone(),
        notes: e.notes.clone(),
        recurrence: e.recurrence,
        page_id: e.page_id.clone(),
        reminder_min: e.reminder_min,
        focus: false,
    }
}

enum CalItem {
    Event(Event, NaiveDateTime),
    Task(Task),
}

struct Placed {
    start: f32,
    end: f32,
    lane: usize,
    lanes: usize,
    item: CalItem,
}

fn layout_day(mut items: Vec<(f32, f32, CalItem)>) -> Vec<Placed> {
    items.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap().then(b.1.partial_cmp(&a.1).unwrap()));
    let mut out: Vec<Placed> = vec![];
    let mut cluster_start = 0;
    let mut cluster_end = -1.0f32;
    let mut lanes_end: Vec<f32> = vec![];
    for (s, e, it) in items {
        if s >= cluster_end && !out.is_empty() {
            let n = lanes_end.len().max(1);
            for p in out[cluster_start..].iter_mut() {
                p.lanes = n;
            }
            cluster_start = out.len();
            lanes_end.clear();
        }
        let lane = match lanes_end.iter().position(|&le| le <= s) {
            Some(i) => {
                lanes_end[i] = e;
                i
            }
            None => {
                lanes_end.push(e);
                lanes_end.len() - 1
            }
        };
        cluster_end = if out.len() == cluster_start { e } else { cluster_end.max(e) };
        out.push(Placed { start: s, end: e, lane, lanes: 1, item: it });
    }
    let n = lanes_end.len().max(1);
    for p in out[cluster_start..].iter_mut() {
        p.lanes = n;
    }
    out
}

fn text_on(c: Color32) -> Color32 {
    let l = 0.299 * c.r() as f32 + 0.587 * c.g() as f32 + 0.114 * c.b() as f32;
    if l > 170.0 {
        tok::INK
    } else {
        Color32::WHITE
    }
}

/// One line of the Agenda.
#[derive(Clone, Debug)]
pub struct AgendaLine {
    /// "10:30–11:00", "Due today", "Due", "16:00"
    pub time: String,
    pub title: String,
    /// The calendar, or the task's project (or "Captured").
    pub right: String,
    pub color: [u8; 3],
    pub event: Option<Id>,
    pub task: Option<Id>,
}

/// The Agenda's days from `from` for `days` days: events on visible calendars and, if shown,
/// open tasks with a date (amber). Days with nothing are left out.
pub fn agenda_days(d: &WorkspaceData, from: NaiveDate, days: i64, show_tasks: bool, today: NaiveDate) -> Vec<(NaiveDate, Vec<AgendaLine>)> {
    let mut out = vec![];
    for i in 0..days {
        let day = from + Duration::days(i);
        let mut lines: Vec<(NaiveTime, AgendaLine)> = vec![];
        for e in d.events.iter().filter(|e| d.calendar(&e.calendar_id).map_or(true, |c| c.visible)) {
            if let Some(s) = e.occurrence_on(day) {
                let cal = d.calendar(&e.calendar_id);
                let time = if e.all_day { "All day".to_string() } else { format!("{}–{}", fmt_time(s.time()), fmt_time((s + (e.end - e.start)).time())) };
                lines.push((if e.all_day { NaiveTime::MIN } else { s.time() }, AgendaLine { time, title: e.title.clone(), right: cal.map(|c| c.name.clone()).unwrap_or_default(), color: cal.map(|c| c.color).unwrap_or(crate::ui::theme::PICK_COLOURS[0]), event: Some(e.id.clone()), task: None }));
            }
        }
        if show_tasks {
            for t in d.tasks.iter().filter(|t| !t.completed && t.parent_id.is_none() && (t.due == Some(day) || t.planned_start.map_or(false, |ps| ps.date() == day))) {
                // a planned start (Plan my day) shows the task at that time; the due date is untouched
                let planned = t.planned_start.filter(|ps| ps.date() == day && t.due_time.is_none()).map(|ps| ps.time());
                let time = match (t.due_time, planned) {
                    (Some(tm), _) => fmt_time(tm),
                    (None, Some(pt)) => format!("Planned {}", fmt_time(pt)),
                    (None, None) if day == today => "Due today".to_string(),
                    (None, None) => "Due".to_string(),
                };
                let right = t.project_id.as_deref().and_then(|p| d.project(p)).map(|p| p.name.clone()).unwrap_or_else(|| "Captured".into());
                lines.push((t.due_time.or(planned).unwrap_or(NaiveTime::from_hms_opt(12, 0, 0).unwrap()), AgendaLine { time, title: t.title.clone(), right, color: crate::ui::theme::PICK_COLOURS[2], event: None, task: Some(t.id.clone()) }));
            }
        }
        if !lines.is_empty() {
            lines.sort_by_key(|l| l.0);
            out.push((day, lines.into_iter().map(|l| l.1).collect()));
        }
    }
    out
}

impl App {
    /// The Agenda view (B-E12): each day with something on it, events and tasks with a date, in
    /// a white box per day; today's heading in terracotta.
    fn agenda_list(&mut self, ui: &mut egui::Ui, p: &Pal) {
        let days = agenda_days(&self.store.data, self.cal.anchor, 21, self.cal.show_tasks, today());
        let mut open_event: Option<Id> = None;
        let mut open_task: Option<Id> = None;
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            capped(ui, space::CAP_DEFAULT, |ui| {
                if days.is_empty() {
                    ui.add_space(space::S40);
                    ui.vertical_centered(|ui| {
                        ui.add(egui::Label::new(RichText::new("Nothing on your calendar yet. Add an event, or give a task a date and it appears here.").size(size::ROW).color(p.dim)).wrap());
                    });
                }
                for (day, lines) in &days {
                    let head = if *day == today() { format!("Today · {}", day.format("%A %-d %B")) } else { day.format("%A %-d %B").to_string() };
                    ui.add_space(space::S10);
                    ui.label(RichText::new(head).font(fs(size::ROW_SMALL)).color(if *day == today() { p.orange } else { p.text }));
                    ui.add_space(space::S6);
                    list_box(ui, p, |ui| {
                        for (i, l) in lines.iter().enumerate() {
                            if agenda_line(ui, p, &l.time, rgb(l.color), l.event.is_some(), &l.title, &l.right, i == 0).clicked() {
                                if let Some(e) = &l.event {
                                    open_event = Some(e.clone());
                                } else if let Some(t) = &l.task {
                                    open_task = Some(t.clone());
                                }
                            }
                        }
                    });
                }
                ui.add_space(space::S24);
            });
        });
        if let Some(id) = open_event {
            if let Some(e) = self.store.data.events.iter().find(|e| e.id == id).cloned() {
                self.event_edit = Some(draft_from(&e));
            }
        }
        if open_task.is_some() {
            self.open_task = open_task;
        }
    }

    fn visible_days(&self) -> Vec<NaiveDate> {
        match self.cal.mode {
            CalMode::Day => vec![self.cal.anchor],
            CalMode::Agenda => (0..21).map(|i| self.cal.anchor + Duration::days(i)).collect(),
            CalMode::Week | CalMode::Month => {
                let mon = self.cal.anchor - Duration::days(self.cal.anchor.weekday().num_days_from_monday() as i64);
                (0..7).map(|i| mon + Duration::days(i)).collect()
            }
        }
    }

    fn new_event_draft(&self, date: NaiveDate, start: Option<NaiveTime>) -> EventDraft {
        let cal = self.store.data.calendars.iter().find(|c| c.visible).or(self.store.data.calendars.first()).map(|c| c.id.clone()).unwrap_or_default();
        EventDraft {
            id: None,
            title: String::new(),
            calendar_id: cal,
            date,
            end_date: date,
            start,
            end: start.map(|s| s + Duration::minutes(60)),
            all_day: start.is_none(),
            location: String::new(),
            notes: String::new(),
            recurrence: Recurrence::None,
            page_id: None,
            reminder_min: Some(10),
            focus: true,
        }
    }

    /// The Calendar (B-E12, Pack B "Calendar"): a serif title with ‹ Today ›, Month / Week /
    /// Agenda and + Event across the top; on the left the calendars as coloured toggles, Tasks with
    /// dates, + New calendar and the "Tasks show here too" note; the grid on the right. Empty,
    /// loading and error as drawn.
    pub fn calendar_view(&mut self, ui: &mut egui::Ui, p: &Pal) {
        let mut new_event = false;
        egui::TopBottomPanel::top("cal-head").show_separator_line(false).frame(egui::Frame::none().fill(p.bg).inner_margin(egui::Margin { left: space::CAL_SIDE, right: space::CAL_SIDE, top: space::S24, bottom: space::S10 })).show_inside(ui, |ui| {
            let step = |mode: CalMode, d: NaiveDate, dir: i64| -> NaiveDate {
                match mode {
                    CalMode::Day => d + Duration::days(dir),
                    CalMode::Week | CalMode::Agenda => d + Duration::days(7 * dir),
                    CalMode::Month => {
                        if dir > 0 {
                            d.checked_add_months(Months::new(1)).unwrap_or(d)
                        } else {
                            d.checked_sub_months(Months::new(1)).unwrap_or(d)
                        }
                    }
                }
            };
            let a = self.cal.anchor;
            let title = match self.cal.mode {
                CalMode::Day => a.format("%A %-d %B").to_string(),
                CalMode::Agenda => "Agenda".to_string(),
                CalMode::Week => {
                    let mon = a - Duration::days(a.weekday().num_days_from_monday() as i64);
                    let sun = mon + Duration::days(6);
                    if mon.month() == sun.month() {
                        format!("{} – {}", mon.day(), sun.format("%-d %B"))
                    } else {
                        format!("{} – {}", mon.format("%-d %b"), sun.format("%-d %b"))
                    }
                }
                CalMode::Month => a.format("%B %Y").to_string(),
            };
            ui.horizontal(|ui| {
                ui.label(RichText::new(title).font(fserif(size::GREETING)).color(p.text));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = space::S10;
                    if button(ui, p, ButtonKind::Dark, "+ Event", false, true).clicked() {
                        new_event = true;
                    }
                    let sel = match self.cal.mode {
                        CalMode::Month => 0,
                        CalMode::Week => 1,
                        CalMode::Agenda => 2,
                        CalMode::Day => usize::MAX,
                    };
                    if let Some(i) = segmented(ui, p, egui::Id::new("cal-mode"), &["Month", "Week", "Agenda"], sel) {
                        self.cal.mode = [CalMode::Month, CalMode::Week, CalMode::Agenda][i];
                    }
                    ui.add_space(space::S6);
                    if button(ui, p, ButtonKind::Secondary, "›", true, true).on_hover_text("Next").clicked() {
                        self.cal.anchor = step(self.cal.mode, self.cal.anchor, 1);
                        self.cal.mini = self.cal.anchor;
                    }
                    if button(ui, p, ButtonKind::Secondary, "Today", true, true).clicked() {
                        self.cal.anchor = today();
                        self.cal.mini = today();
                        self.temp.remove("cal_scrolled");
                    }
                    if button(ui, p, ButtonKind::Secondary, "‹", true, true).on_hover_text("Previous").clicked() {
                        self.cal.anchor = step(self.cal.mode, self.cal.anchor, -1);
                        self.cal.mini = self.cal.anchor;
                    }
                });
            });
        });
        egui::SidePanel::left("cal-left").exact_width(space::CAL_SIDE + space::CAL_LEFT + space::CAL_GAP).resizable(false).show_separator_line(false).frame(egui::Frame::none().fill(p.bg).inner_margin(egui::Margin { left: space::CAL_SIDE, right: space::CAL_GAP, top: space::S4, bottom: space::S10 })).show_inside(ui, |ui| {
            ui.spacing_mut().item_spacing.y = space::S4 / 2.0;
            ui.label(RichText::new("CALENDARS").font(fs(size::CARD_HEAD)).color(p.dim));
            ui.add_space(space::S4);
            let cals = self.store.data.calendars.clone();
            for c in cals {
                let resp = cal_check(ui, p, rgb(c.color), c.visible, &c.name).on_hover_text("Show or hide. Right-click to rename, recolour or delete.");
                if resp.clicked() {
                    if let Some(x) = self.store.data.calendars.iter_mut().find(|x| x.id == c.id) {
                        x.visible = !x.visible;
                        x.updated = now_ms();
                    }
                    self.touch();
                }
                // rename, colour and delete stay on the right-click menu (not drawn on the canvas)
                resp.context_menu(|ui| {
                    let key = format!("calname-{}", c.id);
                    let mut nm = self.temp.get(&key).cloned().unwrap_or_else(|| c.name.clone());
                    if ui.text_edit_singleline(&mut nm).changed() {
                        self.temp.insert(key.clone(), nm.clone());
                        if let Some(x) = self.store.data.calendars.iter_mut().find(|x| x.id == c.id) {
                            x.name = nm;
                            x.updated = now_ms();
                        }
                        self.touch();
                    }
                    ui.horizontal(|ui| {
                        for cc in PICK_COLOURS {
                            let (r, resp) = ui.allocate_exact_size(egui::vec2(space::S18, space::S18), Sense::click());
                            ui.painter().circle_filled(r.center(), space::S8, rgb(cc));
                            if resp.clicked() {
                                if let Some(x) = self.store.data.calendars.iter_mut().find(|x| x.id == c.id) {
                                    x.color = cc;
                                    x.updated = now_ms();
                                }
                                self.touch();
                            }
                        }
                    });
                    if self.store.data.calendars.len() > 1 && ui.button(RichText::new("Delete calendar and its events").color(p.orange)).clicked() {
                        let cid = c.id.clone();
                        self.undoable(&format!("Deleted calendar \"{}\"", c.name), "delete", move |d| d.delete_calendar(&cid));
                        ui.close_menu();
                    }
                });
            }
            if cal_check(ui, p, tok::AMBER, self.cal.show_tasks, "Tasks with dates").clicked() {
                self.cal.show_tasks = !self.cal.show_tasks;
            }
            ui.add_space(space::S6);
            if let Some(mut name) = self.cal.new_cal.take() {
                let mut keep = true;
                let r = ui.add(egui::TextEdit::singleline(&mut name).hint_text("Calendar name").desired_width(f32::INFINITY));
                r.request_focus();
                if r.lost_focus() {
                    if ui.input(|i| i.key_pressed(egui::Key::Enter)) && !name.trim().is_empty() {
                        let c = PICK_COLOURS[self.store.data.calendars.len() % PICK_COLOURS.len()];
                        let cal = Calendar { id: new_id(), name: name.trim().to_string(), color: c, visible: true, updated: now_ms(), ..Default::default() };
                        self.undoable(&format!("Added calendar \"{}\"", cal.name), "edit", move |d| d.calendars.push(cal));
                    }
                    keep = false;
                }
                if keep {
                    self.cal.new_cal = Some(name);
                }
            } else if ui.add(egui::Label::new(RichText::new("+ New calendar").font(fs(size::ROW_SMALL)).color(p.orange)).sense(Sense::click())).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                self.cal.new_cal = Some(String::new());
            }
            ui.add_space(space::S22);
            note_card(ui, p, "Tasks show here too", "Tasks with a date appear as amber rows, on the day they're due.");
        });
        let state = self.view_state;
        egui::CentralPanel::default().frame(egui::Frame::none().fill(p.bg).inner_margin(egui::Margin { left: 0.0, right: space::CAL_SIDE, top: space::S4, bottom: space::S6 })).show_inside(ui, |ui| {
            let area = ui.max_rect();
            match state {
                ViewState::Loading => {
                    ui.add_space(space::S40 * 3.0);
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new("Loading events…").size(size::ROW_SMALL).color(p.dim));
                    });
                    ui.ctx().request_repaint();
                }
                ViewState::Error => {
                    ui.add_space(space::S40 * 2.0);
                    let a = error_center(ui, p, "Resolve couldn't load your events.", "Your calendars are saved on this PC. Tasks with dates still show.", "Open Agenda");
                    if a.retry {
                        self.store.reload_if_changed();
                    }
                    if a.manual {
                        self.cal.mode = CalMode::Agenda;
                    }
                }
                _ => {
                    match self.cal.mode {
                        CalMode::Month => self.month_grid(ui, p),
                        CalMode::Agenda => self.agenda_list(ui, p),
                        _ => self.time_grid(ui, p),
                    }
                    let d = &self.store.data;
                    let nothing = state == ViewState::Empty || (d.events.is_empty() && !d.tasks.iter().any(|t| !t.completed && t.due.is_some()));
                    if nothing && self.cal.mode != CalMode::Agenda {
                        // the empty card sits over the grid (Pack B "Empty · no events this month")
                        let w = space::EVENT_W - space::S40 * 2.0;
                        let r = Rect::from_center_size(area.center() - egui::vec2(0.0, space::S40), egui::vec2(w, space::S40 * 3.0));
                        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(r).layout(Layout::top_down(Align::Center)));
                        card_lift(&mut child, p, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.label(RichText::new("Nothing on your calendar yet.").font(fserif(size::CARD_SERIF)).color(p.text));
                                ui.add(egui::Label::new(RichText::new("Add an event, or give a task a date and it appears here.").size(size::SMALL).color(p.dim)).wrap());
                                ui.add_space(space::S6);
                                if button(ui, p, ButtonKind::Dark, "+ Event", true, true).clicked() {
                                    new_event = true;
                                }
                            });
                        });
                    }
                }
            }
        });
        if new_event {
            let start = NaiveTime::from_hms_opt((now_local().hour() + 1).min(22), 0, 0);
            self.event_edit = Some(self.new_event_draft(self.cal.anchor, start));
        }
    }

    fn mini_month(&mut self, ui: &mut egui::Ui, p: &Pal) {
        let m = self.cal.mini;
        let first = NaiveDate::from_ymd_opt(m.year(), m.month(), 1).unwrap();
        ui.horizontal(|ui| {
            ui.label(RichText::new(first.format("%B %Y").to_string()).font(fs(13.5)));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ghost_button(ui, RichText::new("›").size(16.0)).clicked() {
                    self.cal.mini = first.checked_add_months(Months::new(1)).unwrap_or(first);
                }
                if ghost_button(ui, RichText::new("‹").size(16.0)).clicked() {
                    self.cal.mini = first.checked_sub_months(Months::new(1)).unwrap_or(first);
                }
            });
        });
        let start = first - Duration::days(first.weekday().num_days_from_monday() as i64);
        let cell = 29.0;
        let (rect, _) = ui.allocate_exact_size(egui::vec2(cell * 7.0, cell * 7.0), Sense::hover());
        for (i, l) in ["M", "T", "W", "T", "F", "S", "S"].iter().enumerate() {
            ui.painter().text(rect.min + egui::vec2(cell * (i as f32 + 0.5), cell * 0.5), Align2::CENTER_CENTER, *l, fp(11.0), p.dim);
        }
        let visible = self.visible_days();
        for w in 0..6 {
            for dcol in 0..7 {
                let day = start + Duration::days(w * 7 + dcol);
                let c = rect.min + egui::vec2(cell * (dcol as f32 + 0.5), cell * (w as f32 + 1.5));
                let r = Rect::from_center_size(c, egui::vec2(cell - 2.0, cell - 2.0));
                let resp = ui.interact(r, ui.id().with(("mini", day)), Sense::click());
                let in_month = day.month() == first.month();
                if day == today() {
                    ui.painter().circle_filled(c, 12.0, p.blue);
                } else if visible.contains(&day) && self.cal.mode != CalMode::Month {
                    ui.painter().circle_filled(c, 12.0, p.blue.linear_multiply(0.2));
                } else if resp.hovered() {
                    ui.painter().circle_filled(c, 12.0, p.hover);
                }
                let col = if day == today() { Color32::WHITE } else if in_month { p.text } else { p.faint };
                ui.painter().text(c, Align2::CENTER_CENTER, day.day().to_string(), fp(11.5), col);
                if resp.clicked() {
                    self.cal.anchor = day;
                    self.temp.remove("cal_scrolled");
                }
            }
        }
    }

    fn time_grid(&mut self, ui: &mut egui::Ui, p: &Pal) {
        let days = self.visible_days();
        let gutter = 56.0;
        let hour_h = 48.0;
        let full = ui.available_rect_before_wrap();
        let col_w = ((full.width() - gutter) / days.len() as f32).max(60.0);
        let d = &self.store.data;
        let visible_cal = |cid: &str| d.calendar(cid).map_or(true, |c| c.visible);
        // header with dates
        let (hrect, _) = ui.allocate_exact_size(egui::vec2(full.width(), 66.0), Sense::hover());
        let mut clicked_day: Option<NaiveDate> = None;
        for (i, day) in days.iter().enumerate() {
            let x = hrect.left() + gutter + col_w * i as f32 + col_w / 2.0;
            let is_today = *day == today();
            ui.painter().text(Pos2::new(x, hrect.top() + 12.0), Align2::CENTER_CENTER, day.format("%a").to_string().to_uppercase(), fs(11.0), if is_today { p.blue } else { p.dim });
            let c = Pos2::new(x, hrect.top() + 40.0);
            if is_today {
                ui.painter().circle_filled(c, 19.0, p.blue);
            }
            ui.painter().text(c, Align2::CENTER_CENTER, day.day().to_string(), fp(22.0), if is_today { Color32::WHITE } else { p.text });
            let r = Rect::from_center_size(c, egui::vec2(40.0, 40.0));
            if ui.interact(r, ui.id().with(("hdr", day)), Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                clicked_day = Some(*day);
            }
        }
        ui.painter().text(Pos2::new(hrect.left() + gutter - 8.0, hrect.bottom() - 4.0), Align2::RIGHT_BOTTOM, chrono::Local::now().format("GMT%:z").to_string().replace(":00", ""), fp(10.0), p.faint);
        // all-day strip
        let mut allday: Vec<Vec<(String, Color32, bool, Option<Event>, Option<Id>)>> = vec![vec![]; days.len()];
        for (i, day) in days.iter().enumerate() {
            for e in d.events.iter().filter(|e| e.all_day && visible_cal(&e.calendar_id)) {
                if e.occurrence_on(*day).is_some() {
                    let col = d.calendar(&e.calendar_id).map(|c| rgb(c.color)).unwrap_or(p.blue);
                    allday[i].push((e.title.clone(), col, false, Some(e.clone()), None));
                }
            }
            if self.cal.show_tasks {
                for t in d.tasks.iter().filter(|t| !t.completed && t.due == Some(*day) && t.due_time.is_none() && t.planned_start.map_or(true, |ps| ps.date() != *day)) {
                    allday[i].push((t.title.clone(), prio_color(t.priority, p), true, None, Some(t.id.clone())));
                }
            }
        }
        let rows = allday.iter().map(|v| v.len()).max().unwrap_or(0).min(4);
        let strip_h = (rows as f32 * 22.0 + 6.0).max(10.0);
        let (srect, _) = ui.allocate_exact_size(egui::vec2(full.width(), strip_h), Sense::hover());
        let mut open_event: Option<Event> = None;
        let mut open_task: Option<Id> = None;
        let mut toggle: Option<Id> = None;
        for (i, items) in allday.iter().enumerate() {
            for (j, (title, col, is_task, ev, tid)) in items.iter().take(4).enumerate() {
                let r = Rect::from_min_size(Pos2::new(srect.left() + gutter + col_w * i as f32 + 2.0, srect.top() + 2.0 + j as f32 * 22.0), egui::vec2(col_w - 6.0, 20.0));
                let resp = ui.interact(r, ui.id().with(("ad", i, j)), Sense::click());
                if *is_task {
                    ui.painter().rect(r, 4.0, p.card, Stroke::new(1.0, col.linear_multiply(0.7)));
                    paint_circle(ui, r.left_center() + egui::vec2(11.0, 0.0), p, 4, false, false);
                    ui.painter().with_clip_rect(r).text(r.left_center() + egui::vec2(24.0, 0.0), Align2::LEFT_CENTER, title, fp(12.0), p.text);
                    if resp.clicked() {
                        if ui.ctx().pointer_interact_pos().map_or(false, |pp| pp.x < r.left() + 22.0) {
                            toggle = tid.clone();
                        } else {
                            open_task = tid.clone();
                        }
                    }
                } else {
                    ui.painter().rect_filled(r, 4.0, *col);
                    ui.painter().with_clip_rect(r).text(r.left_center() + egui::vec2(6.0, 0.0), Align2::LEFT_CENTER, title, fs(12.0), text_on(*col));
                    if resp.clicked() {
                        open_event = ev.clone();
                    }
                }
            }
            if items.len() > 4 {
                ui.painter().text(Pos2::new(srect.left() + gutter + col_w * (i as f32 + 1.0) - 6.0, srect.bottom() - 2.0), Align2::RIGHT_BOTTOM, format!("+{}", items.len() - 4), fp(11.0), p.dim);
            }
        }
        ui.painter().line_segment([Pos2::new(srect.left() + gutter, srect.bottom()), Pos2::new(srect.right(), srect.bottom())], Stroke::new(1.0, p.line));

        // hours
        let mut scroll = egui::ScrollArea::vertical().id_salt("cal-hours").auto_shrink([false, false]);
        if !self.temp.contains_key("cal_scrolled") {
            let h = now_local().hour().saturating_sub(2).min(15) as f32;
            scroll = scroll.vertical_scroll_offset(h.max(6.0) * hour_h);
            self.temp.insert("cal_scrolled".into(), "1".into());
        }
        let mut new_at: Option<(NaiveDate, NaiveTime)> = None;
        let mut moved: Option<(String, bool, i64, i64)> = None;
        let drag_state = self.cal.drag.clone();
        let mut new_drag: Option<Option<(Id, f32, f32)>> = None;
        scroll.show(ui, |ui| {
            let (grid, _) = ui.allocate_exact_size(egui::vec2(full.width(), hour_h * 24.0 + 8.0), Sense::hover());
            let top = grid.top() + 4.0;
            let painter = ui.painter().clone();
            for h in 0..24 {
                let y = top + h as f32 * hour_h;
                if h > 0 {
                    painter.text(Pos2::new(grid.left() + gutter - 8.0, y), Align2::RIGHT_CENTER, format!("{h:02}:00"), fp(11.0), p.dim);
                }
                painter.line_segment([Pos2::new(grid.left() + gutter, y), Pos2::new(grid.right(), y)], Stroke::new(1.0, p.line));
            }
            for i in 0..=days.len() {
                let x = grid.left() + gutter + col_w * i as f32;
                painter.line_segment([Pos2::new(x, top), Pos2::new(x, top + hour_h * 24.0)], Stroke::new(1.0, p.line));
            }
            // empty-slot clicks
            for (i, day) in days.iter().enumerate() {
                let col = Rect::from_min_size(Pos2::new(grid.left() + gutter + col_w * i as f32, top), egui::vec2(col_w, hour_h * 24.0));
                let resp = ui.interact(col, ui.id().with(("col", day)), Sense::click());
                if resp.clicked() {
                    if let Some(pp) = resp.interact_pointer_pos() {
                        let mins = (((pp.y - top) / hour_h * 60.0) / 30.0).floor() as i64 * 30;
                        let mins = mins.clamp(0, 23 * 60 + 30);
                        new_at = Some((*day, NaiveTime::from_hms_opt((mins / 60) as u32, (mins % 60) as u32, 0).unwrap()));
                    }
                }
            }
            // items
            for (i, day) in days.iter().enumerate() {
                let mut items: Vec<(f32, f32, CalItem)> = vec![];
                for e in d.events.iter().filter(|e| !e.all_day && visible_cal(&e.calendar_id)) {
                    if let Some(s) = e.occurrence_on(*day) {
                        let start_min = if s.date() == *day { s.time().num_seconds_from_midnight() as f32 / 60.0 } else { 0.0 };
                        let occ_end = if e.recurrence.is_some() { s + e.duration() } else { e.end };
                        let end_min = if occ_end.date() > *day { 24.0 * 60.0 } else { occ_end.time().num_seconds_from_midnight() as f32 / 60.0 };
                        items.push((start_min, end_min.max(start_min + 26.0), CalItem::Event(e.clone(), s)));
                    }
                }
                if self.cal.show_tasks {
                    for t in d.tasks.iter().filter(|t| !t.completed && t.due == Some(*day) && t.due_time.is_some()) {
                        let s = t.due_time.unwrap().num_seconds_from_midnight() as f32 / 60.0;
                        let e = s + t.duration_min.unwrap_or(30) as f32;
                        items.push((s, e.max(s + 20.0).min(24.0 * 60.0), CalItem::Task(t.clone())));
                    }
                    // tasks Plan my day placed: at their planned start (B-E22)
                    for t in d.tasks.iter().filter(|t| !t.completed && t.due_time.is_none() && t.planned_start.map_or(false, |ps| ps.date() == *day)) {
                        let s = t.planned_start.unwrap().time().num_seconds_from_midnight() as f32 / 60.0;
                        let e = s + t.duration_min.unwrap_or(30) as f32;
                        items.push((s, e.max(s + 20.0).min(24.0 * 60.0), CalItem::Task(t.clone())));
                    }
                }
                for pl in layout_day(items) {
                    let lane_w = (col_w - 8.0) / pl.lanes as f32;
                    let x0 = grid.left() + gutter + col_w * i as f32 + 2.0 + lane_w * pl.lane as f32;
                    let y0 = top + pl.start / 60.0 * hour_h + 1.0;
                    let y1 = top + pl.end / 60.0 * hour_h - 1.0;
                    let mut r = Rect::from_min_max(Pos2::new(x0, y0), Pos2::new(x0 + lane_w - 2.0, y1));
                    let (key, is_task) = match &pl.item {
                        CalItem::Event(e, _) => (e.id.clone(), false),
                        CalItem::Task(t) => (t.id.clone(), true),
                    };
                    let id = ui.id().with(("item", &key, day));
                    let resp = ui.interact(r, id, Sense::click_and_drag());
                    if let Some((did, dy, dx)) = &drag_state {
                        if did == &key {
                            r = r.translate(egui::vec2(*dx, *dy));
                        }
                    }
                    if resp.dragged() {
                        let delta = resp.drag_delta();
                        let (dy, dx) = match &drag_state {
                            Some((did, dy, dx)) if did == &key => (*dy + delta.y, *dx + delta.x),
                            _ => (delta.y, delta.x),
                        };
                        new_drag = Some(Some((key.clone(), dy, dx)));
                        ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                    }
                    if resp.drag_stopped() {
                        if let Some((did, dy, dx)) = &drag_state {
                            if did == &key {
                                let mins = ((dy / hour_h * 60.0) / 15.0).round() as i64 * 15;
                                let ddays = (dx / col_w).round() as i64;
                                if mins != 0 || ddays != 0 {
                                    moved = Some((key.clone(), is_task, mins, ddays));
                                }
                            }
                        }
                        new_drag = Some(None);
                    }
                    let height = r.height();
                    match &pl.item {
                        CalItem::Event(e, s) => {
                            let col = d.calendar(&e.calendar_id).map(|c| rgb(c.color)).unwrap_or(p.blue);
                            let past = *s + e.duration() < now_local();
                            let fill = if past { col.linear_multiply(0.75) } else { col };
                            ui.painter().rect(r, 5.0, fill, Stroke::new(1.0, p.bg));
                            let tc = text_on(col);
                            let clip = ui.painter().with_clip_rect(r.shrink(2.0));
                            let time = format!("{} – {}", fmt_time(s.time()), fmt_time((*s + e.duration()).time()));
                            if height < 30.0 {
                                clip.text(r.left_top() + egui::vec2(6.0, 2.0), Align2::LEFT_TOP, format!("{}, {}", e.title, fmt_time(s.time())), fs(11.5), tc);
                            } else {
                                clip.text(r.left_top() + egui::vec2(6.0, 3.0), Align2::LEFT_TOP, &e.title, fs(12.0), tc);
                                clip.text(r.left_top() + egui::vec2(6.0, 18.0), Align2::LEFT_TOP, time, fp(11.0), tc);
                                if !e.location.is_empty() && height > 50.0 {
                                    clip.text(r.left_top() + egui::vec2(6.0, 32.0), Align2::LEFT_TOP, &e.location, fp(11.0), tc);
                                }
                            }
                            if resp.clicked() {
                                open_event = Some(e.clone());
                            }
                            resp.on_hover_text(format!("{}\n{}{}", e.title, if e.recurrence.is_some() { format!("{} · ", e.recurrence.label()) } else { String::new() }, d.calendar(&e.calendar_id).map(|c| c.name.clone()).unwrap_or_default()));
                        }
                        CalItem::Task(t) => {
                            let col = prio_color(t.priority, p);
                            ui.painter().rect(r, 5.0, p.card, Stroke::new(1.2, col));
                            paint_circle(ui, r.left_top() + egui::vec2(12.0, 11.0), p, t.priority, false, false);
                            let clip = ui.painter().with_clip_rect(r.shrink(2.0));
                            clip.text(r.left_top() + egui::vec2(25.0, 3.0), Align2::LEFT_TOP, &t.title, fs(12.0), p.text);
                            if height > 34.0 {
                                clip.text(r.left_top() + egui::vec2(25.0, 18.0), Align2::LEFT_TOP, format!("Task · {}", d.project_name(&t.project_id)), fp(11.0), p.dim);
                            }
                            if resp.clicked() {
                                if resp.interact_pointer_pos().map_or(false, |pp| pp.x < r.left() + 22.0) {
                                    toggle = Some(t.id.clone());
                                } else {
                                    open_task = Some(t.id.clone());
                                }
                            }
                        }
                    }
                }
            }
            // now line
            let now = now_local();
            if let Some(i) = days.iter().position(|x| *x == now.date()) {
                let y = top + now.time().num_seconds_from_midnight() as f32 / 3600.0 * hour_h;
                let x0 = grid.left() + gutter + col_w * i as f32;
                ui.painter().line_segment([Pos2::new(x0, y), Pos2::new(x0 + col_w, y)], Stroke::new(2.0, tok::TERRACOTTA));
                ui.painter().circle_filled(Pos2::new(x0, y), 5.0, tok::TERRACOTTA);
            }
        });
        if let Some(nd) = new_drag {
            self.cal.drag = nd;
        }
        if let Some((id, is_task, mins, ddays)) = moved {
            if is_task {
                if let Some(t) = self.store.data.task_mut(&id) {
                    // a task drawn at its planned start moves its planned start, never its due date or time
                    if let (Some(ps), None) = (t.planned_start, t.due_time) {
                        t.planned_start = Some(ps + Duration::minutes(mins) + Duration::days(ddays));
                        t.updated = now_ms();
                    } else if let (Some(dd), Some(tm)) = (t.due, t.due_time) {
                        let dt = dd.and_time(tm) + Duration::minutes(mins) + Duration::days(ddays);
                        t.due = Some(dt.date());
                        t.due_time = Some(dt.time());
                        t.reminded_for.clear();
                        t.updated = now_ms();
                    }
                }
            } else if let Some(e) = self.store.data.event_mut(&id) {
                let delta = Duration::minutes(mins) + Duration::days(ddays);
                e.start += delta;
                e.end += delta;
                e.updated = now_ms();
            }
            self.touch();
        }
        if let Some((day, t)) = new_at {
            if self.cal.drag.is_none() {
                self.event_edit = Some(self.new_event_draft(day, Some(t)));
            }
        }
        if let Some(e) = open_event {
            self.event_edit = Some(draft_from(&e));
        }
        if let Some(t) = open_task {
            self.open_task = Some(t);
        }
        if let Some(t) = toggle {
            self.toggle_task(&t);
        }
        if let Some(day) = clicked_day {
            self.cal.anchor = day;
            self.cal.mode = CalMode::Day;
        }
    }

    fn month_grid(&mut self, ui: &mut egui::Ui, p: &Pal) {
        let a = self.cal.anchor;
        let first = NaiveDate::from_ymd_opt(a.year(), a.month(), 1).unwrap();
        let start = first - Duration::days(first.weekday().num_days_from_monday() as i64);
        let full = ui.available_rect_before_wrap();
        let cw = full.width() / 7.0;
        let head = 24.0;
        let ch = ((full.height() - head) / 6.0).max(70.0);
        let (rect, _) = ui.allocate_exact_size(egui::vec2(full.width(), head + ch * 6.0), Sense::hover());
        let d = &self.store.data;
        let mut open_event: Option<Event> = None;
        let mut open_task: Option<Id> = None;
        let mut goto_day: Option<NaiveDate> = None;
        let mut new_on: Option<NaiveDate> = None;
        for (i, l) in ["MON", "TUE", "WED", "THU", "FRI", "SAT", "SUN"].iter().enumerate() {
            ui.painter().text(Pos2::new(rect.left() + cw * (i as f32 + 0.5), rect.top() + 12.0), Align2::CENTER_CENTER, *l, fs(11.0), p.dim);
        }
        for w in 0..6 {
            for c in 0..7 {
                let day = start + Duration::days(w * 7 + c);
                let r = Rect::from_min_size(Pos2::new(rect.left() + cw * c as f32, rect.top() + head + ch * w as f32), egui::vec2(cw, ch));
                ui.painter().rect_stroke(r, 0.0, Stroke::new(0.5, p.line));
                let cell = ui.interact(r, ui.id().with(("mcell", day)), Sense::click());
                if cell.double_clicked() {
                    new_on = Some(day);
                } else if cell.clicked() {
                    goto_day = Some(day);
                }
                let in_month = day.month() == first.month();
                let num_c = Pos2::new(r.center().x, r.top() + 14.0);
                if day == today() {
                    ui.painter().circle_filled(num_c, 11.0, p.blue);
                }
                ui.painter().text(num_c, Align2::CENTER_CENTER, day.day().to_string(), fp(12.0), if day == today() { Color32::WHITE } else if in_month { p.text } else { p.faint });
                let mut items: Vec<(Option<NaiveTime>, String, Color32, bool, Option<Event>, Option<Id>)> = vec![];
                for e in d.events.iter().filter(|e| d.calendar(&e.calendar_id).map_or(true, |c| c.visible)) {
                    if let Some(s) = e.occurrence_on(day) {
                        let col = d.calendar(&e.calendar_id).map(|c| rgb(c.color)).unwrap_or(p.blue);
                        items.push((if e.all_day { None } else { Some(s.time()) }, e.title.clone(), col, e.all_day, Some(e.clone()), None));
                    }
                }
                if self.cal.show_tasks {
                    for t in d.tasks.iter().filter(|t| !t.completed && t.due == Some(day)) {
                        items.push((t.due_time, format!("○ {}", t.title), prio_color(t.priority, p), false, None, Some(t.id.clone())));
                    }
                }
                items.sort_by_key(|x| (!x.3, x.0));
                let max = ((ch - 30.0) / 18.0).floor().max(1.0) as usize;
                for (j, (tm, title, col, all_day, ev, tid)) in items.iter().take(max).enumerate() {
                    let ir = Rect::from_min_size(Pos2::new(r.left() + 3.0, r.top() + 28.0 + j as f32 * 18.0), egui::vec2(cw - 6.0, 16.0));
                    let resp = ui.interact(ir, ui.id().with(("mi", day, j)), Sense::click());
                    let clip = ui.painter().with_clip_rect(ir);
                    if *all_day {
                        ui.painter().rect_filled(ir, 3.0, *col);
                        clip.text(ir.left_center() + egui::vec2(4.0, 0.0), Align2::LEFT_CENTER, title, fs(11.0), text_on(*col));
                    } else {
                        if resp.hovered() {
                            ui.painter().rect_filled(ir, 3.0, p.hover);
                        }
                        ui.painter().circle_filled(ir.left_center() + egui::vec2(5.0, 0.0), 3.5, *col);
                        let t = tm.map(|x| format!("{} ", fmt_time(x))).unwrap_or_default();
                        clip.text(ir.left_center() + egui::vec2(12.0, 0.0), Align2::LEFT_CENTER, format!("{t}{title}"), fp(11.0), p.text);
                    }
                    if resp.clicked() {
                        if let Some(e) = ev {
                            open_event = Some(e.clone());
                        } else if let Some(t) = tid {
                            open_task = Some(t.clone());
                        }
                    }
                }
                if items.len() > max {
                    ui.painter().text(Pos2::new(r.left() + 6.0, r.bottom() - 4.0), Align2::LEFT_BOTTOM, format!("{} more", items.len() - max), fp(11.0), p.dim);
                }
            }
        }
        if let Some(e) = open_event {
            self.event_edit = Some(draft_from(&e));
        } else if let Some(t) = open_task {
            self.open_task = Some(t);
        } else if let Some(dd) = new_on {
            self.event_edit = Some(self.new_event_draft(dd, NaiveTime::from_hms_opt(9, 0, 0)));
        } else if let Some(dd) = goto_day {
            self.cal.anchor = dd;
            self.cal.mode = CalMode::Day;
            self.temp.remove("cal_scrolled");
        }
    }

    pub fn event_window(&mut self, ctx: &egui::Context, p: &Pal) {
        let Some(mut dr) = self.event_edit.take() else { return };
        let mut keep = true;
        let mut save = false;
        let mut delete = false;
        let mut open_page: Option<Id> = None;
        let mut new_doc = false;
        // The event editor (B-E12, Pack B "Calendar: event"): a 520 dialog over a dimmed screen;
        // the title in serif with an amber rule, then When, All day, Repeats, Calendar and Notes;
        // Delete, Cancel and Save in the footer. Location, reminder and a linked document stay
        // under "More" (not drawn on the canvas, kept so nothing is lost).
        let screen = ctx.screen_rect();
        egui::Area::new(egui::Id::new("event-dim")).order(egui::Order::Foreground).fixed_pos(screen.min).show(ctx, |ui| {
            if backdrop(ui, screen, shadow::BACKDROP_DIALOG) {
                keep = false;
            }
        });
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            keep = false;
        }
        egui::Area::new(egui::Id::new("event-editor")).order(egui::Order::Foreground).fixed_pos(egui::pos2(screen.center().x, screen.top() + space::S40 * 2.0)).pivot(Align2::CENTER_TOP).show(ctx, |ui| {
            dialog_frame(p).show(ui, |ui| {
                ui.set_width(space::EVENT_W);
                egui::Frame::none().inner_margin(egui::Margin { left: space::S18 + 2.0, right: space::S18 + 2.0, top: space::S22, bottom: space::S18 }).show(ui, |ui| {
                    let r = ui.add(egui::TextEdit::singleline(&mut dr.title).hint_text(RichText::new("Event title").color(p.faint)).font(fserif(size::CARD_SERIF)).text_color(p.text).frame(false).desired_width(f32::INFINITY));
                    if dr.focus {
                        r.request_focus();
                        dr.focus = false;
                    }
                    if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        save = true;
                    }
                    let (lr, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), stroke::ICON), Sense::hover());
                    ui.painter().rect_filled(lr, 0.0, tok::AMBER);
                    ui.add_space(space::S12);
                    let label = |ui: &mut egui::Ui, text: &str| {
                        ui.allocate_ui_with_layout(egui::vec2(space::EVENT_LABEL_W, space::BUTTON_H), Layout::left_to_right(Align::Center), |ui| {
                            ui.set_min_size(egui::vec2(space::EVENT_LABEL_W, space::BUTTON_H));
                            ui.label(RichText::new(text).size(size::ROW).color(p.dim));
                        });
                    };
                    ui.spacing_mut().item_spacing.y = space::S10;
                    ui.spacing_mut().button_padding = egui::vec2(space::S10, space::S6);
                    {
                        // fields read as white boxes with a hairline, as drawn
                        let v = ui.visuals_mut();
                        v.extreme_bg_color = p.card;
                        for w in [&mut v.widgets.inactive, &mut v.widgets.hovered, &mut v.widgets.active, &mut v.widgets.open] {
                            w.weak_bg_fill = p.card;
                            w.bg_fill = p.card;
                            w.bg_stroke = Stroke::new(stroke::HAIR, p.line);
                            w.rounding = egui::Rounding::same(radius::FIELD);
                        }
                        v.widgets.hovered.weak_bg_fill = p.surface_raised;
                    }
                    ui.horizontal(|ui| {
                        label(ui, "When");
                        let mut date = dr.date;
                        if ui.add(egui_extras::DatePickerButton::new(&mut date).id_salt("ev-date").format("%a %-d %b").show_icon(false).calendar_week(false)).changed() {
                            let shift = date - dr.date;
                            dr.date = date;
                            dr.end_date = dr.end_date + shift;
                        }
                        if !dr.all_day {
                            time_field(ui, egui::Id::new("ev-start"), &mut dr.start, space::S40 + space::S12);
                            ui.label(RichText::new("to").size(size::ROW).color(p.dim));
                            time_field(ui, egui::Id::new("ev-end"), &mut dr.end, space::S40 + space::S12);
                        } else {
                            ui.label(RichText::new("until").size(size::ROW).color(p.dim));
                            let mut ed = dr.end_date.max(dr.date);
                            if ui.add(egui_extras::DatePickerButton::new(&mut ed).id_salt("ev-enddate").format("%a %-d %b").show_icon(false).calendar_week(false)).changed() {
                                dr.end_date = ed.max(dr.date);
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        label(ui, "All day");
                        let was = dr.all_day;
                        switch(ui, p, &mut dr.all_day);
                        if was && !dr.all_day && dr.start.is_none() {
                            dr.start = NaiveTime::from_hms_opt(9, 0, 0);
                            dr.end = NaiveTime::from_hms_opt(10, 0, 0);
                        }
                    });
                    ui.horizontal(|ui| {
                        label(ui, "Repeats");
                        egui::ComboBox::from_id_salt("ev-rec").selected_text(dr.recurrence.label()).show_ui(ui, |ui| {
                            for r in Recurrence::CHOICES {
                                ui.selectable_value(&mut dr.recurrence, r, r.label());
                            }
                            ui.selectable_value(&mut dr.recurrence, Recurrence::EveryWeeks(2), "Every 2 weeks");
                        });
                    });
                    ui.horizontal(|ui| {
                        label(ui, "Calendar");
                        let (cname, ccol) = self.store.data.calendar(&dr.calendar_id).map(|c| (c.name.clone(), rgb(c.color))).unwrap_or_default();
                        let (sw, _) = ui.allocate_exact_size(egui::vec2(space::S10, space::S10), Sense::hover());
                        ui.painter().rect_filled(sw, radius::CHIP / 3.0, ccol);
                        egui::ComboBox::from_id_salt("ev-cal").selected_text(RichText::new(cname).color(p.text)).show_ui(ui, |ui| {
                            for c in &self.store.data.calendars {
                                ui.selectable_value(&mut dr.calendar_id, c.id.clone(), RichText::new(format!("■ {}", c.name)).color(rgb(c.color)));
                            }
                        });
                    });
                    ui.horizontal_top(|ui| {
                        label(ui, "Notes");
                        egui::Frame::none().fill(p.field).stroke(Stroke::new(stroke::HAIR, p.line)).rounding(egui::Rounding::same(radius::FIELD)).inner_margin(egui::Margin::symmetric(space::S10, space::S8)).show(ui, |ui| {
                            ui.add(egui::TextEdit::multiline(&mut dr.notes).hint_text("Add notes").frame(false).font(fp(size::ROW)).desired_rows(3).desired_width(f32::INFINITY));
                        });
                    });
                    egui::CollapsingHeader::new(RichText::new("More: location, reminder, document").size(size::SMALL).color(p.dim)).id_salt("ev-more").show(ui, |ui| {
                        ui.horizontal(|ui| {
                            label(ui, "Location");
                            ui.add(egui::TextEdit::singleline(&mut dr.location).hint_text("Add location").desired_width(f32::INFINITY));
                        });
                        ui.horizontal(|ui| {
                            label(ui, "Reminder");
                            let rl = |m: Option<i64>| match m {
                                None => "None".to_string(),
                                Some(0) => "At start".to_string(),
                                Some(m) if m < 60 => format!("{m} min before"),
                                Some(m) => format!("{} h before", m / 60),
                            };
                            egui::ComboBox::from_id_salt("ev-rem").selected_text(rl(dr.reminder_min)).show_ui(ui, |ui| {
                                for m in [None, Some(0), Some(5), Some(10), Some(30), Some(60), Some(120)] {
                                    ui.selectable_value(&mut dr.reminder_min, m, rl(m));
                                }
                            });
                        });
                        ui.horizontal(|ui| {
                            label(ui, "Document");
                            let current = dr.page_id.as_deref().and_then(|x| self.store.data.page(x)).map(|x| x.title.clone()).unwrap_or_else(|| "None".into());
                            egui::ComboBox::from_id_salt("ev-page").selected_text(current).show_ui(ui, |ui| {
                                ui.selectable_value(&mut dr.page_id, None, "None");
                                for pg in &self.store.data.pages {
                                    ui.selectable_value(&mut dr.page_id, Some(pg.id.clone()), pg.title.clone());
                                }
                            });
                            if let Some(pg) = &dr.page_id {
                                if button(ui, p, ButtonKind::Ghost, "Open", true, true).clicked() {
                                    open_page = Some(pg.clone());
                                }
                            } else if button(ui, p, ButtonKind::Ghost, "+ Meeting notes", true, true).clicked() {
                                new_doc = true;
                            }
                        });
                    });
                    if dr.recurrence.is_some() && dr.id.is_some() {
                        ui.label(RichText::new("Changes apply to every event in the series.").size(size::SMALL).color(p.faint));
                    }
                });
                // footer band
                egui::Frame::none().fill(p.bg).rounding(egui::Rounding { nw: 0.0, ne: 0.0, sw: radius::DIALOG, se: radius::DIALOG }).inner_margin(egui::Margin::symmetric(space::S18 + 2.0, space::S14)).show(ui, |ui| {
                    ui.set_width(space::EVENT_W - (space::S18 + 2.0) * 2.0);
                    ui.horizontal(|ui| {
                        if dr.id.is_some() && button(ui, p, ButtonKind::Ghost, "Delete", false, true).clicked() {
                            delete = true;
                        }
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if button(ui, p, ButtonKind::Dark, "Save", false, true).clicked() {
                                save = true;
                            }
                            if button(ui, p, ButtonKind::Secondary, "Cancel", false, true).clicked() {
                                keep = false;
                            }
                        });
                    });
                });
            });
        });
        if new_doc {
            let mut pg = Page::new(if dr.title.is_empty() { "Meeting notes".to_string() } else { format!("Notes: {}", dr.title) });
            pg.icon = "🗒".into();
            pg.blocks = vec![
                Block::new(BlockKind::Paragraph, format!("📅 {}", dr.date.format("%A %-d %B %Y"))),
                Block::new(BlockKind::H2, "Agenda"),
                Block::new(BlockKind::Bullet, ""),
                Block::new(BlockKind::H2, "Action items"),
                Block::new(BlockKind::Todo, ""),
            ];
            dr.page_id = Some(pg.id.clone());
            self.store.data.pages.push(pg);
            self.touch();
        }
        if save {
            if dr.title.trim().is_empty() {
                dr.title = "(No title)".into();
            }
            let (start, end) = if dr.all_day {
                (dr.date.and_hms_opt(0, 0, 0).unwrap(), (dr.end_date.max(dr.date) + Duration::days(1)).and_hms_opt(0, 0, 0).unwrap())
            } else {
                let s = dr.date.and_time(dr.start.unwrap_or(NaiveTime::from_hms_opt(9, 0, 0).unwrap()));
                let mut e = dr.date.and_time(dr.end.unwrap_or(s.time() + Duration::hours(1)));
                if e <= s {
                    e += Duration::days(1);
                }
                (s, e)
            };
            match &dr.id {
                Some(id) => {
                    if let Some(e) = self.store.data.event_mut(id) {
                        e.title = dr.title.clone();
                        e.calendar_id = dr.calendar_id.clone();
                        e.start = start;
                        e.end = end;
                        e.all_day = dr.all_day;
                        e.location = dr.location.clone();
                        e.notes = dr.notes.clone();
                        e.recurrence = dr.recurrence;
                        e.page_id = dr.page_id.clone();
                        e.reminder_min = dr.reminder_min;
                        e.updated = now_ms();
                    }
                }
                None => {
                    self.store.data.events.push(Event {
                        id: new_id(),
                        calendar_id: dr.calendar_id.clone(), record_id: None, source: String::new(),
                        title: dr.title.clone(),
                        start,
                        end,
                        all_day: dr.all_day,
                        location: dr.location.clone(),
                        notes: dr.notes.clone(),
                        recurrence: dr.recurrence,
                        page_id: dr.page_id.clone(),
                        reminder_min: dr.reminder_min,
                        updated: now_ms(),
                    });
                }
            }
            self.touch();
            keep = false;
        }
        if delete {
            if let Some(id) = &dr.id {
                { let eid = id.to_string(); let name = self.store.data.events.iter().find(|e| e.id == eid).map(|e| e.title.clone()).unwrap_or_default(); self.undoable(&format!("Deleted event \"{name}\""), "delete", |d| d.delete_event(&eid)); }
            }
            keep = false;
        }
        if let Some(pg) = open_page {
            keep = false;
            self.go(View::Page(pg));
        }
        if keep {
            self.event_edit = Some(dr);
        }
    }
}
