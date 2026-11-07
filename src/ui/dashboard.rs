use super::theme::*;
use super::widgets::*;
use super::*;
use chrono::{Datelike, Local, Timelike};
use eframe::egui::{self, Color32, RichText, Sense};

impl App {
    pub fn dashboard(&mut self, ui: &mut egui::Ui, p: &Pal) {
        self.top_bar(ui, p, |app, ui| {
            if ui.button("📆 Calendar").clicked() {
                app.go(View::Calendar);
            }
            if ui.button("📄 New page").clicked() {
                app.new_page(None);
            }
        });
        let t0 = today();
        let hour = Local::now().hour();
        let greet = match hour {
            5..=11 => "Good morning",
            12..=17 => "Good afternoon",
            _ => "Good evening",
        };
        let d = &self.store.data;
        let open: Vec<&Task> = d.tasks.iter().filter(|t| !t.completed).collect();
        let overdue = open.iter().filter(|t| t.is_overdue()).count();
        let due_today = open.iter().filter(|t| t.due == Some(t0)).count();
        let week_start = t0 - chrono::Duration::days(t0.weekday().num_days_from_monday() as i64);
        let ws_ms = week_start.and_hms_opt(0, 0, 0).unwrap().and_local_timezone(Local).single().map(|x| x.timestamp_millis()).unwrap_or(0);
        let done_week = d.tasks.iter().filter(|t| t.completed && t.completed_at.map_or(false, |c| c >= ws_ms)).count();
        let mut todays_events: Vec<(NaiveDateTime, Event)> = d.events.iter().filter(|e| d.calendar(&e.calendar_id).map_or(true, |c| c.visible)).filter_map(|e| e.occurrence_on(t0).map(|s| (s, e.clone()))).collect();
        todays_events.sort_by_key(|x| (!x.1.all_day, x.0));
        let n_events = todays_events.len();

        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            centered(ui, 1080.0, |ui| {
                ui.add_space(14.0);
                ui.label(RichText::new(greet).font(fb(30.0)));
                ui.label(RichText::new(format!(
                    "{} · {} due today · {} overdue · {} events",
                    t0.format("%A %-d %B"),
                    due_today,
                    overdue,
                    n_events
                ))
                .size(14.0)
                .color(p.dim));
                ui.add_space(14.0);
                if self.store.meta.name == crate::store::SAMPLE_NAME {
                    let mut go_personal = false;
                    egui::Frame::none().fill(p.callout).rounding(8.0).inner_margin(egui::Margin::symmetric(14.0, 10.0)).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("🧪 You're looking at sample data, kept apart from your own. Your real workspace is \"Personal\".").size(13.5));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if ui.button("Open Personal").clicked() {
                                    go_personal = true;
                                }
                            });
                        });
                    });
                    if go_personal {
                        if let Some(w) = self.reg.find("Personal").map(|w| w.id.clone()) {
                            self.switch_workspace(&w);
                            return;
                        }
                    }
                    ui.add_space(12.0);
                }

                // quick capture bar
                let mut submit = false;
                egui::Frame::none().fill(p.card).stroke(egui::Stroke::new(1.0, p.line)).rounding(10.0).inner_margin(egui::Margin::symmetric(14.0, 8.0)).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("⊕").size(20.0).color(p.accent_text));
                        let text = self.temp.entry("dash_capture".into()).or_default();
                        let r = ui.add(egui::TextEdit::singleline(text).hint_text("Quick capture, for example: Call the garage tomorrow 9am").frame(false).font(fp(15.0)).desired_width(ui.available_width() - 110.0));
                        if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            submit = true;
                        }
                        if primary_button(ui, p, "Add task").clicked() {
                            submit = true;
                        }
                    });
                    let text = self.temp.get("dash_capture").cloned().unwrap_or_default();
                    if !text.trim().is_empty() {
                        let c = crate::nlp::parse(&text, today());
                        ui.horizontal_wrapped(|ui| {
                            for s in c.chips() {
                                chip(ui, &s, p.blue);
                            }
                        });
                    }
                });
                if submit {
                    let text = self.temp.get("dash_capture").cloned().unwrap_or_default();
                    if self.add_task_text(&text, None, "", None, None, None).is_some() {
                        self.toast("Task added");
                        self.temp.insert("dash_capture".into(), String::new());
                    }
                }
                ui.add_space(14.0);

                // Resolve: each guided project's pinned next move
                let rows: Vec<(Id, String, String, usize, usize)> = self
                    .store
                    .data
                    .contexts
                    .iter()
                    .filter(|c| self.store.data.project(&c.project_id).map_or(false, |p| !p.archived))
                    .map(|c| {
                        let a = crate::resolve::assess::assess(&self.store.data, c);
                        let next = a.win.as_ref().and_then(|w| self.store.data.task(w)).map(|t| t.title.clone()).or_else(|| a.next.first().map(|x| x.0.clone())).unwrap_or_else(|| "Plan the next piece of work".into());
                        (c.project_id.clone(), self.store.data.project(&c.project_id).map(|p| p.name.clone()).unwrap_or_default(), next, a.done, a.total)
                    })
                    .collect();
                let mut open_resolve: Option<Id> = None;
                let mut new_intake = false;
                let mut test_intake = false;
                egui::Frame::none().fill(p.callout).rounding(10.0).inner_margin(egui::Margin::symmetric(16.0, 12.0)).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("✦ Resolve").font(fs(15.0)).color(p.accent_text));
                        ui.label(RichText::new("your guided projects and their next move").size(12.5).color(p.dim));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if primary_button(ui, p, "New project").on_hover_text("Opens the intake").clicked() {
                                new_intake = true;
                            }
                            if ui.button("Test Intake").on_hover_text("Try the intake in a sandbox. Nothing touches your workspaces.").clicked() {
                                test_intake = true;
                            }
                        });
                    });
                    if rows.is_empty() {
                        ui.label(RichText::new("Arrive with an idea, a problem or a messy project. Resolve turns it into a connected project with a clear first move.").size(13.0));
                    }
                    for (pid, name, next, done, total) in &rows {
                        ui.add_space(4.0);
                        if wrap_link(ui, RichText::new(format!("{name}  ·  {done}/{total}")).font(fs(14.0))).clicked() {
                            open_resolve = Some(pid.clone());
                        }
                        wrap_label(ui, RichText::new(format!("→ {next}")).size(13.0).color(p.dim));
                    }
                });
                if new_intake {
                    self.start_intake();
                    return;
                }
                if test_intake {
                    self.launch_testlab();
                }
                if let Some(pid) = open_resolve {
                    self.select_project(Some(pid));
                    self.go(View::Sec(nav::Section::Overview));
                    return;
                }
                ui.add_space(14.0);

                // stat cards
                let d = &self.store.data;
                let stats = [
                    ("Due today", due_today.to_string(), p.green, View::Today),
                    ("Overdue", overdue.to_string(), p.accent, View::Today),
                    ("Done this week", done_week.to_string(), p.purple, View::Filters),
                    ("Events today", n_events.to_string(), p.blue, View::Calendar),
                ];
                let mut go_to: Option<View> = None;
                ui.columns(4, |cols| {
                    for (i, (label, value, col, v)) in stats.iter().enumerate() {
                        let ui = &mut cols[i];
                        let resp = egui::Frame::none().fill(p.card).stroke(egui::Stroke::new(1.0, p.line)).rounding(10.0).inner_margin(egui::Margin::symmetric(16.0, 12.0)).show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.horizontal(|ui| {
                                let (r, _) = ui.allocate_exact_size(egui::vec2(4.0, 40.0), Sense::hover());
                                ui.painter().rect_filled(r, 2.0, *col);
                                ui.vertical(|ui| {
                                    ui.label(RichText::new(value).font(fb(24.0)).color(*col));
                                    ui.label(RichText::new(*label).size(12.5).color(p.dim));
                                });
                            });
                        });
                        if ui.interact(resp.response.rect, ui.id().with(("stat", i)), Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                            go_to = Some(v.clone());
                        }
                    }
                });
                ui.add_space(14.0);

                let mut toggles: Vec<Id> = vec![];
                let mut open_task: Option<Id> = None;
                let mut open_page: Option<Id> = None;
                let mut open_event: Option<Event> = None;
                let mut reschedule = false;
                ui.columns(2, |cols| {
                    // ---------- left: tasks
                    {
                        let ui = &mut cols[0];
                        card(ui, p, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Today").font(fb(17.0)));
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if overdue > 0 && ghost_button(ui, RichText::new("Reschedule overdue → today").size(12.0).color(p.accent_text)).clicked() {
                                        reschedule = true;
                                    }
                                });
                            });
                            let mut list: Vec<&Task> = d.tasks.iter().filter(|t| !t.completed && t.parent_id.is_none() && t.due.map_or(false, |x| x <= t0)).collect();
                            list.sort_by_key(|t| (t.due, t.due_time, t.priority));
                            if list.is_empty() {
                                ui.add_space(10.0);
                                ui.label(RichText::new("🎉 Nothing due today. Enjoy it!").color(p.dim));
                                ui.add_space(10.0);
                            }
                            for t in list.iter().take(9) {
                                let r = task_row(ui, p, d, t, true, 0.0);
                                if r.toggled {
                                    toggles.push(t.id.clone());
                                }
                                if r.opened {
                                    open_task = Some(t.id.clone());
                                }
                            }
                            if list.len() > 9 {
                                if ghost_button(ui, RichText::new(format!("+ {} more in Today", list.len() - 9)).color(p.accent_text)).clicked() {
                                    go_to = Some(View::Today);
                                }
                            }
                        });
                        ui.add_space(12.0);
                        // Captured: tasks not in a project yet (what used to be the Inbox)
                        card(ui, p, |ui| {
                            let mut cap: Vec<&Task> = d.tasks.iter().filter(|t| !t.completed && t.project_id.is_none() && t.parent_id.is_none()).collect();
                            cap.sort_by_key(|t| -t.created);
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Captured").font(fb(17.0)));
                                ui.label(RichText::new(format!("{} not in a project yet", cap.len())).size(12.0).color(p.dim));
                            });
                            if cap.is_empty() {
                                wrap_label(ui, RichText::new("Nothing waiting. Use the capture bar above or Add task.").color(p.dim));
                            }
                            for t in cap.iter().take(6) {
                                let r = task_row(ui, p, d, t, false, 0.0);
                                if r.toggled {
                                    toggles.push(t.id.clone());
                                }
                                if r.opened {
                                    open_task = Some(t.id.clone());
                                }
                            }
                            if cap.len() > 6 && ghost_button(ui, RichText::new(format!("+ {} more in Tasks → Captured", cap.len() - 6)).color(p.accent_text)).clicked() {
                                go_to = Some(View::Inbox);
                            }
                        });
                        ui.add_space(12.0);
                        card(ui, p, |ui| {
                            ui.label(RichText::new("Next 7 days").font(fb(17.0)));
                            ui.add_space(4.0);
                            for i in 1..=7 {
                                let day = t0 + chrono::Duration::days(i);
                                let tasks: Vec<&Task> = d.tasks.iter().filter(|t| !t.completed && t.parent_id.is_none() && t.due == Some(day)).collect();
                                let evs: Vec<&Event> = d.events.iter().filter(|e| d.calendar(&e.calendar_id).map_or(true, |c| c.visible) && e.occurrence_on(day).is_some() && e.recurrence != Recurrence::Weekdays && e.recurrence != Recurrence::Daily).collect();
                                if tasks.is_empty() && evs.is_empty() {
                                    continue;
                                }
                                ui.add_space(4.0);
                                ui.label(RichText::new(format!("{} · {}", day.format("%a %-d %b"), fmt_date_short(day))).font(fs(13.0)).color(p.dim));
                                for e in evs.iter().take(3) {
                                    let col = d.calendar(&e.calendar_id).map(|c| rgb(c.color)).unwrap_or(p.blue);
                                    ui.horizontal(|ui| {
                                        let (r, _) = ui.allocate_exact_size(egui::vec2(3.0, 16.0), Sense::hover());
                                        ui.painter().rect_filled(r, 1.0, col);
                                        let time = if e.all_day { "All day".to_string() } else { fmt_time(e.start.time()) };
                                        if wrap_link(ui, RichText::new(format!("{time}  {}", e.title)).size(13.0)).clicked() {
                                            open_event = Some((*e).clone());
                                        }
                                    });
                                }
                                for t in tasks.iter().take(4) {
                                    ui.horizontal(|ui| {
                                        let (r, resp) = ui.allocate_exact_size(egui::vec2(18.0, 18.0), Sense::click());
                                        paint_circle(ui, r.center(), p, t.priority, false, resp.hovered());
                                        if resp.clicked() {
                                            toggles.push(t.id.clone());
                                        }
                                        if wrap_link(ui, RichText::new(&t.title).size(13.0)).clicked() {
                                            open_task = Some(t.id.clone());
                                        }
                                    });
                                }
                            }
                        });
                    }
                    // ---------- right: schedule, pages, projects
                    {
                        let ui = &mut cols[1];
                        card(ui, p, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Schedule").font(fb(17.0)));
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.label(RichText::new(t0.format("%a %-d %b").to_string()).size(12.0).color(p.dim));
                                });
                            });
                            ui.add_space(4.0);
                            let now = now_local();
                            let mut items: Vec<(NaiveDateTime, String, Color32, bool, Option<Event>, Option<Id>)> = todays_events.iter().map(|(s, e)| (*s, e.title.clone(), d.calendar(&e.calendar_id).map(|c| rgb(c.color)).unwrap_or(p.blue), e.all_day, Some(e.clone()), None)).collect();
                            for t in d.tasks.iter().filter(|t| !t.completed && t.due == Some(t0) && t.due_time.is_some()) {
                                items.push((t0.and_time(t.due_time.unwrap()), format!("○ {}", t.title), prio_color(t.priority, p), false, None, Some(t.id.clone())));
                            }
                            items.sort_by_key(|x| (!x.3, x.0));
                            if items.is_empty() {
                                ui.label(RichText::new("No events today").color(p.dim));
                            }
                            let mut now_drawn = false;
                            for (start, title, col, all_day, ev, tid) in items {
                                if !now_drawn && !all_day && start > now {
                                    now_drawn = true;
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(fmt_time(now.time())).size(11.0).color(p.accent_text));
                                        let (r, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 8.0), Sense::hover());
                                        ui.painter().circle_filled(r.left_center(), 4.0, p.accent);
                                        ui.painter().line_segment([r.left_center(), r.right_center()], egui::Stroke::new(1.5, p.accent));
                                    });
                                }
                                let past = !all_day && start + chrono::Duration::minutes(30) < now;
                                let resp = ui.horizontal(|ui| {
                                    let time = if all_day { "All day".to_string() } else { fmt_time(start.time()) };
                                    ui.add_sized([52.0, 20.0], egui::Label::new(RichText::new(time).size(12.5).color(p.dim)));
                                    let (r, _) = ui.allocate_exact_size(egui::vec2(4.0, 22.0), Sense::hover());
                                    ui.painter().rect_filled(r, 2.0, if past { col.linear_multiply(0.4) } else { col });
                                    ui.add(egui::Label::new(RichText::new(&title).size(14.0).color(if past { p.dim } else { p.text })).wrap().sense(Sense::click()))
                                });
                                if resp.inner.clicked() {
                                    if let Some(e) = ev {
                                        open_event = Some(e);
                                    } else if let Some(t) = tid {
                                        open_task = Some(t);
                                    }
                                }
                            }
                        });
                        ui.add_space(12.0);
                        card(ui, p, |ui| {
                            ui.label(RichText::new("Recent pages").font(fb(17.0)));
                            ui.add_space(4.0);
                            let mut pages: Vec<&Page> = d.pages.iter().collect();
                            pages.sort_by_key(|x| -x.updated);
                            for pg in pages.iter().take(5) {
                                let title = if pg.title.is_empty() { "Untitled" } else { &pg.title };
                                let when = chrono::DateTime::from_timestamp_millis(pg.updated).map(|x| x.with_timezone(&Local).format("%-d %b").to_string()).unwrap_or_default();
                                if wrap_link(ui, RichText::new(format!("{}  {}   ", if pg.icon.is_empty() { "📄" } else { &pg.icon }, title)).size(14.0)).clicked() {
                                    open_page = Some(pg.id.clone());
                                }
                                ui.label(RichText::new(when).size(11.0).color(p.faint));
                                ui.add_space(2.0);
                            }
                        });
                        ui.add_space(12.0);
                        card(ui, p, |ui| {
                            ui.label(RichText::new("Projects").font(fb(17.0)));
                            ui.add_space(4.0);
                            for pr in d.projects.iter().filter(|x| !x.archived) {
                                let all: Vec<&Task> = d.tasks.iter().filter(|t| t.project_id.as_deref() == Some(&pr.id)).collect();
                                let done = all.iter().filter(|t| t.completed).count();
                                let frac = if all.is_empty() { 0.0 } else { done as f32 / all.len() as f32 };
                                let resp = ui.vertical(|ui| {
                                    wrap_label(ui, RichText::new(format!("# {}", pr.name)).size(14.0));
                                    ui.horizontal(|ui| {
                                        let (r, _) = ui.allocate_exact_size(egui::vec2((ui.available_width() - 50.0).max(40.0), 8.0), Sense::hover());
                                        ui.painter().rect_filled(r, 4.0, p.hover);
                                        let mut f = r;
                                        f.set_width(r.width() * frac);
                                        ui.painter().rect_filled(f, 4.0, rgb(pr.color));
                                        ui.label(RichText::new(format!("{done}/{}", all.len())).size(12.0).color(p.dim));
                                    });
                                });
                                if ui.interact(resp.response.rect, ui.id().with(("proj", &pr.id)), Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                    go_to = Some(View::Resolve(pr.id.clone()));
                                }
                            }
                        });
                    }
                });
                ui.add_space(30.0);
                for id in toggles {
                    self.toggle_task(&id);
                }
                if reschedule {
                    let t = today();
                    for task in self.store.data.tasks.iter_mut().filter(|x| x.is_overdue()) {
                        task.due = Some(t);
                        task.updated = now_ms();
                    }
                    self.touch();
                    self.toast("Overdue tasks moved to today");
                }
                if let Some(id) = open_task {
                    self.open_task = Some(id);
                }
                if let Some(id) = open_page {
                    self.go(View::Page(id));
                }
                if let Some(e) = open_event {
                    self.event_edit = Some(calendar::draft_from(&e));
                }
                if let Some(v) = go_to {
                    self.go(v);
                }
            });
        });
    }
}
