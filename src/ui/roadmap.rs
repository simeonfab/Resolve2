//! Roadmap (C-E4 to C-E8, Pack C "Roadmap"): Now, Next and Later lanes of theme cards (tag, gate
//! chip, progress; open in place on the height spring), By theme, the Whole picture, phase moves
//! that go through history (F6 when they land; a move that can't be made changes nothing and
//! says so), and the right rail (Milestones, Dependencies and blockers, Coming up). Every line
//! comes from the project's records and tasks (`roadmap_model`, self-tested).

use super::motion::{self, Frame};
use super::nav::Section;
use super::sections::{lane_of, LANES};
use super::theme::*;
use super::widgets::*;
use super::*;
use crate::resolve::{Kind, Record};
use eframe::egui::{self, RichText};

/// What each lane means (the canvas's notes beside the phase).
const LANE_NOTES: [&str; 3] = ["this month", "after the gates", "parked"];

/// One theme as the Roadmap draws it.
#[derive(Clone, Debug, Default)]
pub struct ThemeView {
    pub id: Id,
    pub title: String,
    pub lane: &'static str,
    pub basis: String,
    pub detail: String,
    pub status: String,
    /// The theme it comes after (a record id, or empty).
    pub after: String,
    /// (hypothesis id, "H1 result", open?) when the theme waits on a hypothesis (C-E5).
    pub gate: Option<(Id, String, bool)>,
    /// The hypothesis the theme rests on (its gate, else the hypothesis an experiment on one of
    /// its tasks tests): its row on the whole picture.
    pub hyp: Option<Id>,
    /// (task id, title, done)
    pub tasks: Vec<(Id, String, bool)>,
    /// "Waiting on: <task> · 3 days" for a task of the theme that waits on someone.
    pub waiting: Option<String>,
    /// The experiment that tests it: (record id, "E0").
    pub exp: Option<(Id, String)>,
}

impl ThemeView {
    pub fn done(&self) -> usize {
        self.tasks.iter().filter(|t| t.2).count()
    }
}

#[derive(Clone, Debug, Default)]
pub struct RoadmapModel {
    pub project: String,
    pub themes: Vec<ThemeView>,
    /// The project's hypotheses, H0 left out: (id, "H1", "Monthly workload is predictable").
    pub hyps: Vec<(Id, String, String)>,
    /// (title, "Fri 20 Nov", reached)
    pub milestones: Vec<(String, String, bool)>,
    /// What is held up by someone else (drawn in terracotta): "<task> → <theme>".
    pub blocked: Vec<String>,
    /// What waits on what: "Price to absorb spikes waits on the E0 result."
    pub waits: Vec<String>,
    /// (task, "Fri 13 Nov" or "after the gate", task id)
    pub coming: Vec<(String, String, Option<Id>)>,
}

/// "H1: Monthly workload is predictable" → ("H1", "Monthly workload is predictable").
pub fn hyp_label(title: &str) -> (String, String) {
    match title.split_once(": ") {
        Some((id, rest)) if id.starts_with('H') && id.len() <= 4 => (id.to_string(), rest.trim().to_string()),
        _ => (String::new(), title.trim().to_string()),
    }
}

fn short_date(s: &str) -> String {
    chrono::NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d").map(|d| d.format("%a %-d %b").to_string()).unwrap_or_else(|_| s.trim().to_string())
}

pub fn roadmap_model(d: &WorkspaceData, pid: &str, today: chrono::NaiveDate, now_ms: i64) -> RoadmapModel {
    let mut m = RoadmapModel::default();
    let Some(pr) = d.project(pid) else { return m };
    m.project = pr.name.clone();
    let recs: Vec<&Record> = d.records.iter().filter(|r| r.project_id.as_deref() == Some(pid)).collect();
    let exps: Vec<&&Record> = recs.iter().filter(|r| r.kind == Kind::Experiment).collect();
    let e_label = |id: &str| exps.iter().position(|e| e.id == id).map(|i| format!("E{i}")).unwrap_or_default();
    let is_hyp = |id: &str| recs.iter().find(|r| r.id == id && r.kind == Kind::Hypothesis).copied();
    for h in recs.iter().filter(|r| r.kind == Kind::Hypothesis && r.field("type") != "H0") {
        let (l, s) = hyp_label(&h.title);
        m.hyps.push((h.id.clone(), l, s));
    }
    let mut themes: Vec<&&Record> = recs.iter().filter(|r| r.kind == Kind::Roadmap && r.status != "Dropped").collect();
    themes.sort_by_key(|r| LANES.iter().position(|l| *l == lane_of(r)).unwrap_or(0));
    for r in themes {
        let lane = LANES.iter().copied().find(|l| *l == lane_of(r)).unwrap_or("Now");
        let tasks: Vec<&Task> = r.links.iter().filter_map(|l| d.task(l)).collect();
        let gate = is_hyp(r.field("gate")).map(|h| {
            let (l, _) = hyp_label(&h.title);
            (h.id.clone(), if l.is_empty() { "Gate".to_string() } else { format!("{l} result") }, h.status == "Supported")
        });
        // the experiment that tests one of its tasks, else one that tests its gate
        let exp = exps.iter().find(|e| e.links.iter().any(|l| tasks.iter().any(|t| &t.id == l))).or_else(|| gate.as_ref().and_then(|g| exps.iter().find(|e| e.parent_id.as_deref() == Some(g.0.as_str()))));
        let hyp = gate.as_ref().map(|g| g.0.clone()).or_else(|| exp.and_then(|e| e.parent_id.clone()).filter(|h| m.hyps.iter().any(|x| &x.0 == h)));
        let waiting = tasks.iter().find(|t| !t.completed && t.waiting_since.is_some()).map(|t| {
            let days = ((now_ms - t.waiting_since.unwrap_or(now_ms)).max(0) / (24 * 3600 * 1000)) as i64;
            let when = match days {
                0 => "since today".to_string(),
                1 => "1 day".to_string(),
                n => format!("{n} days"),
            };
            format!("Waiting on: {} · {when}", t.title)
        });
        m.themes.push(ThemeView {
            id: r.id.clone(),
            title: r.title.clone(),
            lane,
            basis: r.field("basis").to_string(),
            detail: if r.body.trim().is_empty() { r.field("detail").trim().to_string() } else { r.body.trim().to_string() },
            status: r.status.clone(),
            after: r.field("after").to_string(),
            gate,
            hyp,
            tasks: tasks.iter().map(|t| (t.id.clone(), t.title.clone(), t.completed)).collect(),
            waiting,
            exp: exp.map(|e| (e.id.clone(), e_label(&e.id))),
        });
    }
    // milestones by date (undated last)
    let mut ms: Vec<&&Record> = recs.iter().filter(|r| r.kind == Kind::Milestone && r.status != "Missed").collect();
    ms.sort_by_key(|r| {
        let d = chrono::NaiveDate::parse_from_str(r.field("date").trim(), "%Y-%m-%d").ok();
        (d.is_none(), d)
    });
    m.milestones = ms.iter().map(|r| (r.title.clone(), short_date(r.field("date")), r.status == "Reached")).collect();
    // dependencies and blockers
    for t in m.themes.iter() {
        for (tid, _, done) in &t.tasks {
            if let Some(task) = d.task(tid).filter(|x| !*done && x.waiting_since.is_some()) {
                m.blocked.push(format!("{} → {}", task.title, t.title));
            }
        }
    }
    for r in recs.iter().filter(|r| matches!(r.kind, Kind::Dependency | Kind::Blocker) && r.is_open()) {
        m.blocked.push(format!("{} ({})", r.title, r.status.to_lowercase()));
    }
    for t in &m.themes {
        if let Some((hid, label, false)) = &t.gate {
            let on = exps.iter().find(|e| e.parent_id.as_deref() == Some(hid.as_str())).map(|e| e_label(&e.id)).unwrap_or_else(|| label.trim_end_matches(" result").to_string());
            m.waits.push(format!("{} waits on the {on} result.", t.title));
        }
        if let Some(o) = m.themes.iter().find(|o| o.id == t.after && o.status != "Done") {
            m.waits.push(format!("{} comes after {}.", t.title, o.title));
        }
    }
    // coming up: dated tasks in the next two weeks, then the first task behind each closed gate
    let mut soon: Vec<&Task> = d.tasks.iter().filter(|t| t.project_id.as_deref() == Some(pid) && !t.completed && t.parent_id.is_none() && t.due.map_or(false, |x| x > today && x <= today + chrono::Duration::days(14))).collect();
    soon.sort_by_key(|t| (t.due, t.priority));
    m.coming = soon.iter().map(|t| (t.title.clone(), t.due.map(|x| x.format("%a %-d %b").to_string()).unwrap_or_default(), Some(t.id.clone()))).collect();
    for t in m.themes.iter().filter(|t| matches!(t.gate, Some((_, _, false)))) {
        if let Some((id, title, _)) = t.tasks.iter().find(|x| !x.2) {
            m.coming.push((title.clone(), "after the gate".into(), Some(id.clone())));
        }
    }
    m.coming.truncate(4);
    m
}

/// Move a theme to another phase. A theme that isn't there any more changes nothing and says so
/// (C-E7: the rollback is that nothing was written). Callers wrap it in history.
pub fn move_theme(d: &mut WorkspaceData, id: &str, lane: &str) -> Result<String, String> {
    let Some(r) = d.record_mut(id).filter(|r| r.kind == Kind::Roadmap) else {
        return Err("That theme isn't on the roadmap any more.".into());
    };
    if !LANES.contains(&lane) {
        return Err(format!("{lane} isn't a phase."));
    }
    r.fields.insert("phase".into(), lane.into());
    r.updated = now_ms();
    Ok(r.title.clone())
}

/// What a click on the Roadmap asks for; applied after drawing.
enum Act {
    Toggle(Id),
    Move(Id, &'static str),
    Gate(Id, Option<Id>),
    After(Id, Option<Id>),
    Status(Id, &'static str),
    OpenTask(Id),
    CheckTask(Id),
    AddTask(Id),
    OpenExp(Id),
    Retry,
    Dismiss,
    OpenTheme(Id),
}

/// Lists the card menus need.
struct Lists {
    themes: Vec<(Id, String)>,
    hyps: Vec<(Id, String)>,
}

fn next_lane(lane: &str) -> &'static str {
    match lane {
        "Now" => "Next",
        "Next" => "Later",
        _ => "Next",
    }
}

/// The open part of a theme: what it involves, its tasks, what it waits on, and its buttons.
fn theme_body(ui: &mut egui::Ui, p: &Pal, t: &ThemeView, lists: &Lists, acts: &mut Vec<Act>, divider: bool) {
    ui.spacing_mut().item_spacing.y = space::S8;
    if divider {
        let w = ui.available_width();
        let (r, _) = ui.allocate_exact_size(egui::vec2(w, stroke::HAIR), egui::Sense::hover());
        ui.painter().rect_filled(r, 0.0, p.divider);
    }
    if !t.detail.is_empty() {
        ui.add(egui::Label::new(RichText::new(&t.detail).size(size::ROW_SMALL - 0.5).color(p.dim)).wrap());
    }
    if t.tasks.is_empty() {
        ui.label(RichText::new("No tasks yet.").size(size::ROW_SMALL - 0.5).color(p.dim));
    }
    for (id, title, done) in &t.tasks {
        if theme_task(ui, p, title, *done).clicked() {
            acts.push(Act::CheckTask(id.clone()));
        }
    }
    if let Some(w) = &t.waiting {
        wait_box(ui, p, w);
    }
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(space::S8, space::S8);
        if button(ui, p, ButtonKind::Secondary, "+ Task", true, true).clicked() {
            acts.push(Act::AddTask(t.id.clone()));
        }
        let to = next_lane(t.lane);
        if button(ui, p, ButtonKind::Secondary, &format!("Move to {to}"), true, true).clicked() {
            acts.push(Act::Move(t.id.clone(), to));
        }
        if let Some((eid, label)) = &t.exp {
            if button(ui, p, ButtonKind::Secondary, &format!("Open {label}"), true, true).clicked() {
                acts.push(Act::OpenExp(eid.clone()));
            }
        }
        let more = button(ui, p, ButtonKind::Ghost, "⋯", true, true).on_hover_text("Phase, order, gate and status");
        let menu = ui.make_persistent_id(("rm-more", &t.id));
        if more.clicked() {
            ui.memory_mut(|mm| mm.toggle_popup(menu));
        }
        egui::popup_below_widget(ui, menu, &more, egui::PopupCloseBehavior::CloseOnClickOutside, |ui| {
            ui.set_min_width(space::S40 * 6.0);
            ui.label(RichText::new("Move to").size(size::TINY).color(p.dim));
            for l in LANES.iter().filter(|l| **l != t.lane) {
                if ui.button(*l).clicked() {
                    acts.push(Act::Move(t.id.clone(), l));
                    ui.memory_mut(|mm| mm.close_popup());
                }
            }
            ui.separator();
            ui.label(RichText::new("Comes after").size(size::TINY).color(p.dim));
            if ui.selectable_label(t.after.is_empty(), "Nothing (can start any time)").clicked() {
                acts.push(Act::After(t.id.clone(), None));
                ui.memory_mut(|mm| mm.close_popup());
            }
            for (oid, ot) in lists.themes.iter().filter(|o| o.0 != t.id) {
                if ui.selectable_label(&t.after == oid, ot).clicked() {
                    acts.push(Act::After(t.id.clone(), Some(oid.clone())));
                    ui.memory_mut(|mm| mm.close_popup());
                }
            }
            ui.separator();
            ui.label(RichText::new("Validation gate").size(size::TINY).color(p.dim));
            if ui.selectable_label(t.gate.is_none(), "No gate").clicked() {
                acts.push(Act::Gate(t.id.clone(), None));
                ui.memory_mut(|mm| mm.close_popup());
            }
            for (hid, ht) in &lists.hyps {
                if ui.selectable_label(t.gate.as_ref().map_or(false, |g| &g.0 == hid), ht).clicked() {
                    acts.push(Act::Gate(t.id.clone(), Some(hid.clone())));
                    ui.memory_mut(|mm| mm.close_popup());
                }
            }
            ui.separator();
            ui.label(RichText::new("Status").size(size::TINY).color(p.dim));
            for s in ["Planned", "Active", "Done", "Dropped"] {
                if ui.selectable_label(t.status == s, s).clicked() {
                    acts.push(Act::Status(t.id.clone(), s));
                    ui.memory_mut(|mm| mm.close_popup());
                }
            }
        });
    });
}

/// A theme card in a lane (C-E4).
fn theme_card(ui: &mut egui::Ui, p: &Pal, t: &ThemeView, open: bool, land: Frame, lists: &Lists, acts: &mut Vec<Act>) {
    motion::fade_in(ui, land, egui::vec2(0.0, -moves::LAND_DROP), |ui| {
        theme_frame(ui, p, open, |ui| {
            ui.horizontal_top(|ui| {
                let tw = ui.available_width() - space::RM_TOGGLE - ui.spacing().item_spacing.x;
                let title = ui
                    .allocate_ui_with_layout(egui::vec2(tw, 0.0), Layout::top_down(Align::Min), |ui| {
                        ui.set_width(tw);
                        ui.add(egui::Label::new(RichText::new(&t.title).font(fs(size::ROW_SMALL)).color(p.text)).wrap().sense(egui::Sense::click()))
                    })
                    .inner;
                if title.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() | chevron_button(ui, p, open).clicked() {
                    acts.push(Act::Toggle(t.id.clone()));
                }
            });
            if !t.basis.is_empty() || t.gate.is_some() {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(space::S6, space::S6);
                    if !t.basis.is_empty() {
                        basis_chip(ui, p, &t.basis);
                    }
                    if let Some((_, g, o)) = &t.gate {
                        gate_chip(ui, g, *o).on_hover_text(if *o { "The hypothesis is supported, so this theme can go ahead." } else { "This theme waits for that result." });
                    }
                });
            }
            ui.horizontal(|ui| {
                let count = format!("{} of {}", t.done(), t.tasks.len());
                let cw = ui.painter().layout_no_wrap(count.clone(), fp(size::SMALL), p.dim).size().x;
                let w = ui.available_width() - cw - space::S10 - ui.spacing().item_spacing.x;
                progress_line(ui, p, if t.tasks.is_empty() { 0.0 } else { t.done() as f32 / t.tasks.len() as f32 }, w);
                ui.add_space(space::S10 - ui.spacing().item_spacing.x);
                ui.label(RichText::new(count).size(size::SMALL).color(p.dim));
            });
            motion::height_spring(ui, egui::Id::new(("rm-open", &t.id)), open, moves::EXPAND, moves::COLLAPSE, |ui| theme_body(ui, p, t, lists, acts, true));
        });
    });
}

impl App {
    /// The redesigned Roadmap for one project. Returns false with no project picked (the caller
    /// shows every project's roadmap).
    pub(super) fn roadmap_page(&mut self, ui: &mut egui::Ui, p: &Pal) -> bool {
        let Some(pid) = self.current_project.clone() else { return false };
        let m = roadmap_model(&self.store.data, &pid, today(), now_ms());
        let state = match self.view_state {
            ViewState::Full if m.themes.is_empty() => ViewState::Empty,
            s => s,
        };
        // C-E7: a move that couldn't be made: (theme id, title, where it still is, where it was going)
        let mut failed: Option<(Id, String, String, String)> = self.temp.get("rm_fail").and_then(|s| {
            let v: Vec<&str> = s.split('\t').collect();
            (v.len() == 4).then(|| (v[0].to_string(), v[1].to_string(), v[2].to_string(), v[3].to_string()))
        });
        if state == ViewState::Error && failed.is_none() {
            // the error state's screenshot: the canvas's failed move
            failed = m.themes.iter().find(|t| t.lane == "Next").map(|t| (t.id.clone(), t.title.clone(), "Next".to_string(), "Later".to_string()));
        }
        let open_id = self.temp.get("rm_open").cloned();
        if let Some(id) = self.temp.remove("rm_land_new") {
            motion::restart(ui.ctx(), egui::Id::new(("rm-land", &id)));
        }
        let landed: Option<(Id, Frame)> = self.temp.get("rm_landed").cloned().map(|id| {
            let f = motion::play(ui.ctx(), egui::Id::new(("rm-land", &id)), moves::LAND);
            (id, f)
        });
        if landed.as_ref().map_or(false, |l| l.1.done) {
            self.temp.remove("rm_landed");
        }
        let lists = Lists { themes: m.themes.iter().map(|t| (t.id.clone(), t.title.clone())).collect(), hyps: m.hyps.iter().map(|h| (h.0.clone(), if h.1.is_empty() { h.2.clone() } else { format!("{}: {}", h.1, h.2) })).collect() };
        let mut acts: Vec<Act> = vec![];
        let mut whole = false;
        let mut add_open = false;
        let mut draft = false;
        let by_theme = self.resolve_ui.roadmap_by_theme;
        let mut set_by_theme: Option<bool> = None;
        let screen = ui.ctx().screen_rect();
        egui::ScrollArea::vertical().auto_shrink([false, false]).id_salt(("roadmap", &pid)).show(ui, |ui| {
            egui::Frame::none().inner_margin(egui::Margin { left: space::WORK_SIDE_WIDE, right: space::WORK_SIDE_WIDE, top: space::WORK_TOP, bottom: 0.0 }).show(ui, |ui| {
                capped(ui, space::CAP_ROADMAP, |ui| {
                    eyebrow_head(ui, p, &m.project, "Roadmap", |ui| {
                        ui.spacing_mut().item_spacing.x = space::S10;
                        if button(ui, p, ButtonKind::Primary, "+ Add a theme", false, true).clicked() {
                            add_open = true;
                        }
                        if button(ui, p, ButtonKind::Secondary, "Whole picture", false, state == ViewState::Full).clicked() {
                            whole = true;
                        }
                        if let Some(i) = segmented(ui, p, egui::Id::new("rm-view"), &["By phase", "By theme"], by_theme as usize) {
                            set_by_theme = Some(i == 1);
                        }
                    });
                    ui.add_space(space::S18);
                    if add_open || self.temp.contains_key("rm_add") {
                        self.temp.insert("rm_add".into(), "1".into());
                        self.add_theme_card(ui, p, &pid);
                        ui.add_space(space::RM_LANE_GAP);
                    }
                    let w = ui.available_width();
                    let lw = ((w - space::RM_RAIL - space::RM_RAIL_GAP - space::RM_LANE_GAP * 2.0) / 3.0).floor();
                    let lane_h = (screen.bottom() - ui.cursor().min.y).max(space::S40 * 6.0);
                    match state {
                        ViewState::Loading => {
                            ui.horizontal_top(|ui| {
                                ui.spacing_mut().item_spacing.x = 0.0;
                                for _ in 0..3 {
                                    ui.allocate_ui_with_layout(egui::vec2(lw, lane_h), Layout::top_down(Align::Min), |ui| lane_box(ui, p, lw, lane_h, "", "", |_| {}));
                                    ui.add_space(space::RM_LANE_GAP);
                                }
                                ui.add_space(space::RM_RAIL_GAP - space::RM_LANE_GAP);
                                ui.allocate_ui_with_layout(egui::vec2(space::RM_RAIL, 0.0), Layout::top_down(Align::Min), |ui| skeleton_card(ui, p, 6, "Reading the roadmap…"));
                            });
                            return;
                        }
                        ViewState::Empty => {
                            ui.add_space(space::S40 * 2.0);
                            let (a, b) = empty_center_two(ui, p, Icon::Roadmap, "No roadmap yet.", "A roadmap is a few themes in Now, Next and Later. Ask Resolve to draft one from your brief, or add the first theme yourself.", "Draft it from my brief", "Add a theme");
                            draft = a;
                            add_open |= b;
                            return;
                        }
                        _ => {}
                    }
                    if let Some((_, title, from, _)) = &failed {
                        ui.add_space(space::S12);
                        let a = error_center(ui, p, "That move didn't save.", &format!("{title} is still in {from}. Nothing else changed. Try the move again."), "Back to the roadmap");
                        if a.retry {
                            acts.push(Act::Retry);
                        }
                        if a.manual {
                            acts.push(Act::Dismiss);
                        }
                        return;
                    }
                    let sx = ui.spacing().item_spacing.x;
                    ui.horizontal_top(|ui| {
                        // explicit gaps between the columns; the columns keep the normal spacing
                        ui.spacing_mut().item_spacing.x = 0.0;
                        if by_theme {
                            let lw3 = lw * 3.0 + space::RM_LANE_GAP * 2.0;
                            ui.allocate_ui_with_layout(egui::vec2(lw3, 0.0), Layout::top_down(Align::Min), |ui| {
                                ui.spacing_mut().item_spacing.x = sx;
                                ui.set_width(lw3);
                                list_box(ui, p, |ui| {
                                    for (i, t) in m.themes.iter().enumerate() {
                                        let sub = if t.tasks.is_empty() { "No tasks yet".to_string() } else { t.tasks.iter().map(|x| if x.2 { format!("{} ✓", x.1) } else { x.1.clone() }).collect::<Vec<_>>().join(" · ") };
                                        let open = open_id.as_deref() == Some(t.id.as_str());
                                        let gate = t.gate.as_ref().map(|g| (g.1.as_str(), g.2));
                                        if theme_row(ui, p, egui::Id::new(("rm-row", &t.id)), t.lane, &t.title, &t.basis, gate, &sub, &format!("{} of {}", t.done(), t.tasks.len()), i == 0).clicked() {
                                            acts.push(Act::Toggle(t.id.clone()));
                                        }
                                        motion::height_spring(ui, egui::Id::new(("rm-row-open", &t.id)), open, moves::EXPAND, moves::COLLAPSE, |ui| {
                                            egui::Frame::none().inner_margin(egui::Margin { left: space::S16 + space::S40 + space::S16 + space::S14, right: space::S16, top: 0.0, bottom: space::S14 }).show(ui, |ui| {
                                                ui.set_width(ui.available_width());
                                                theme_body(ui, p, t, &lists, &mut acts, false);
                                            });
                                        });
                                    }
                                });
                            });
                        } else {
                            for (k, lane) in LANES.iter().enumerate() {
                                ui.allocate_ui_with_layout(egui::vec2(lw, lane_h), Layout::top_down(Align::Min), |ui| {
                                    ui.spacing_mut().item_spacing.x = sx;
                                    lane_box(ui, p, lw, lane_h, lane, LANE_NOTES[k], |ui| {
                                        for t in m.themes.iter().filter(|t| t.lane == *lane) {
                                            let open = open_id.as_deref() == Some(t.id.as_str());
                                            let land = landed.as_ref().filter(|l| l.0 == t.id).map(|l| l.1).unwrap_or(motion::SETTLED);
                                            theme_card(ui, p, t, open, land, &lists, &mut acts);
                                        }
                                    });
                                });
                                if k < 2 {
                                    ui.add_space(space::RM_LANE_GAP);
                                }
                            }
                        }
                        ui.add_space(space::RM_RAIL_GAP);
                        ui.allocate_ui_with_layout(egui::vec2(space::RM_RAIL, 0.0), Layout::top_down(Align::Min), |ui| {
                            ui.set_width(space::RM_RAIL);
                            ui.spacing_mut().item_spacing = egui::vec2(sx, space::RM_LANE_GAP);
                            self.roadmap_rail(ui, p, &m, &mut acts);
                        });
                    });
                });
            });
        });
        if let Some(b) = set_by_theme {
            self.resolve_ui.roadmap_by_theme = b;
        }
        if add_open {
            self.temp.insert("rm_add".into(), "1".into());
        }
        if whole {
            self.temp.insert("rm_whole".into(), "1".into());
            motion::restart(ui.ctx(), egui::Id::new("rm-whole"));
        }
        if self.temp.contains_key("rm_whole") && state == ViewState::Full {
            if let Some(a) = self.whole_picture(ui.ctx(), p, &m) {
                acts.push(a);
            }
        }
        if draft {
            self.page_ui.ai_open = true;
            self.ask_send(&format!("Draft a roadmap for the project \"{}\" from its project brief: three to six themes, each placed in Now, Next or Later, added as roadmap records with the phase set. Then tell me what you added.", m.project));
        }
        self.apply_roadmap(acts, &pid, failed);
        true
    }

    /// The right rail (C-E8): Milestones, Dependencies and blockers, Coming up.
    fn roadmap_rail(&mut self, ui: &mut egui::Ui, p: &Pal, m: &RoadmapModel, acts: &mut Vec<Act>) {
        white_card(ui, p, false, |ui| {
            ui.spacing_mut().item_spacing.y = space::S8;
            rail_title(ui, p, "Milestones");
            if m.milestones.is_empty() {
                ui.label(RichText::new("No milestones yet. Add one with + Add a theme.").size(size::ROW_SMALL - 0.5).color(p.dim));
            }
            for (t, d, reached) in &m.milestones {
                milestone_row(ui, p, t, d, *reached);
            }
        });
        white_card(ui, p, false, |ui| {
            ui.spacing_mut().item_spacing.y = space::S8;
            rail_title(ui, p, "Dependencies and blockers");
            if m.blocked.is_empty() && m.waits.is_empty() {
                ui.label(RichText::new("Nothing is waiting on anything.").size(size::ROW_SMALL - 0.5).color(p.dim));
            }
            for b in &m.blocked {
                wait_box(ui, p, b);
            }
            for w in &m.waits {
                ui.add(egui::Label::new(RichText::new(w).size(size::ROW_SMALL - 0.5).color(p.dim)).wrap());
            }
        });
        white_card(ui, p, false, |ui| {
            ui.spacing_mut().item_spacing.y = space::S8;
            rail_title(ui, p, "Coming up");
            if m.coming.is_empty() {
                ui.label(RichText::new("Nothing dated in the next two weeks.").size(size::ROW_SMALL - 0.5).color(p.dim));
            }
            for (t, when, id) in &m.coming {
                if rail_line(ui, p, t, when).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                    if let Some(id) = id {
                        acts.push(Act::OpenTask(id.clone()));
                    }
                }
            }
        });
    }

    /// "+ Add a theme": a theme in a phase, or a milestone with its date.
    fn add_theme_card(&mut self, ui: &mut egui::Ui, p: &Pal, pid: &str) {
        let mut close = false;
        white_card(ui, p, false, |ui| {
            ui.spacing_mut().item_spacing.y = space::S10;
            ui.horizontal(|ui| {
                ui.label(RichText::new("Add to the roadmap").font(fs(size::ROW)).color(p.text));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if button(ui, p, ButtonKind::Ghost, "Done", true, true).clicked() {
                        close = true;
                    }
                });
            });
            if self.resolve_ui.new_theme_lane.is_empty() {
                self.resolve_ui.new_theme_lane = "Now".into();
            }
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = space::S10;
                let enter = text_field(ui, p, egui::Id::new("rm-new-theme"), &mut self.resolve_ui.new_theme, "A theme, e.g. Price to absorb spikes", false, space::S40 * 8.0).lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                let sel = LANES.iter().position(|l| *l == self.resolve_ui.new_theme_lane).unwrap_or(0);
                if let Some(i) = segmented(ui, p, egui::Id::new("rm-new-lane"), &LANES, sel) {
                    self.resolve_ui.new_theme_lane = LANES[i].into();
                }
                let ok = !self.resolve_ui.new_theme.trim().is_empty();
                if (button(ui, p, ButtonKind::Primary, "Add theme", false, ok).clicked() || enter) && ok {
                    let title = self.resolve_ui.new_theme.trim().to_string();
                    let lane = self.resolve_ui.new_theme_lane.clone();
                    let pid = pid.to_string();
                    let mut r = Record::new(Kind::Roadmap, Some(pid), &title);
                    r.fields.insert("phase".into(), lane.clone());
                    let id = r.id.clone();
                    self.undoable(&format!("Added \"{title}\" to {lane}"), "add", move |d| d.records.push(r));
                    self.resolve_ui.new_theme.clear();
                    self.temp.insert("rm_landed".into(), id.clone());
                    motion::restart(ui.ctx(), egui::Id::new(("rm-land", &id)));
                }
            });
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = space::S10;
                text_field(ui, p, egui::Id::new("rm-new-ms"), &mut self.resolve_ui.new_milestone, "A milestone, e.g. First flat-fee client signed", false, space::S40 * 8.0);
                text_field(ui, p, egui::Id::new("rm-new-ms-date"), self.temp.entry("rm_ms_date".into()).or_default(), "Date, e.g. 2026-12-16", false, space::S40 * 4.0);
                let ok = !self.resolve_ui.new_milestone.trim().is_empty();
                if button(ui, p, ButtonKind::Secondary, "Add milestone", false, ok).clicked() && ok {
                    let title = self.resolve_ui.new_milestone.trim().to_string();
                    let mut r = Record::new(Kind::Milestone, Some(pid.to_string()), &title);
                    let date = self.temp.remove("rm_ms_date").unwrap_or_default();
                    if chrono::NaiveDate::parse_from_str(date.trim(), "%Y-%m-%d").is_ok() {
                        r.fields.insert("date".into(), date.trim().to_string());
                    }
                    self.undoable(&format!("Added the milestone \"{title}\""), "add", move |d| d.records.push(r));
                    self.resolve_ui.new_milestone.clear();
                }
            });
        });
        if close {
            self.temp.remove("rm_add");
        }
    }

    /// The whole picture (C-E6, F5): hypotheses down the side, phases across, each theme in its
    /// cell; a cell opens its theme.
    fn whole_picture(&mut self, ctx: &egui::Context, p: &Pal, m: &RoadmapModel) -> Option<Act> {
        let screen = ctx.screen_rect();
        let reduced = motion::reduced();
        let el = motion::elapsed_ms(ctx, egui::Id::new("rm-whole"));
        let dim = motion::frame_at(el, moves::WHOLE_DIM, reduced);
        let f = motion::frame_at(el, moves::WHOLE_IN, reduced);
        if !f.done || el < moves::WHOLE_CELLS.stagger(m.themes.len() + 3, moves::WHOLE_CELLS_STAGGER).end() {
            ctx.request_repaint();
        }
        let mut close = false;
        let mut pick: Option<Id> = None;
        egui::Area::new(egui::Id::new("rm-whole-dim")).order(egui::Order::Foreground).fixed_pos(screen.min).show(ctx, |ui| {
            close |= backdrop(ui, screen, shadow::BACKDROP_DIALOG * dim.alpha.clamp(0.0, 1.0));
        });
        let w = space::WHOLE_W.min(screen.width() - space::S40 * 2.0);
        let rows: Vec<(Option<Id>, String, String)> = m.hyps.iter().filter(|h| m.themes.iter().any(|t| t.hyp.as_ref() == Some(&h.0))).map(|h| (Some(h.0.clone()), h.1.clone(), h.2.clone())).chain(m.themes.iter().any(|t| t.hyp.is_none()).then(|| (None, String::new(), "Not tied to a hypothesis".to_string()))).collect();
        egui::Area::new(egui::Id::new("rm-whole")).order(egui::Order::Foreground).fixed_pos(egui::pos2(screen.center().x, screen.top() + space::WHOLE_TOP + moves::DIALOG_RISE * (1.0 - f.t))).pivot(Align2::CENTER_TOP).show(ctx, |ui| {
            ui.multiply_opacity(f.alpha.clamp(0.0, 1.0));
            dialog_frame(p).inner_margin(egui::Margin { left: space::S22, right: space::S22, top: space::S22, bottom: space::S18 }).show(ui, |ui| {
                ui.set_width(w - space::S22 * 2.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("The whole picture").font(fserif(size::HERO_TITLE)).color(p.text));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if button(ui, p, ButtonKind::Secondary, "✕", true, true).on_hover_text("Close (Esc)").clicked() {
                            close = true;
                        }
                        ui.label(RichText::new("Each theme, the hypothesis it rests on, and when it runs").size(size::ROW_SMALL - 0.5).color(p.dim));
                    });
                });
                ui.add_space(space::S10);
                let gap = space::S6;
                let cw = ((ui.available_width() - space::WHOLE_LABEL_W - gap * 3.0) / 3.0).floor();
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = gap;
                    ui.add_space(space::WHOLE_LABEL_W);
                    for (k, l) in LANES.iter().enumerate() {
                        whole_head(ui, p, cw, l, k == 0);
                    }
                });
                ui.add_space(gap);
                let mut n = 0usize;
                for (hid, label, stmt) in &rows {
                    ui.horizontal_top(|ui| {
                        ui.spacing_mut().item_spacing.x = gap;
                        ui.allocate_ui_with_layout(egui::vec2(space::WHOLE_LABEL_W, space::WHOLE_CELL_H), Layout::top_down(Align::Min), |ui| {
                            ui.set_width(space::WHOLE_LABEL_W - space::S10);
                            ui.spacing_mut().item_spacing.y = space::S4 / 2.0;
                            ui.add_space(space::S8);
                            if !label.is_empty() {
                                ui.label(RichText::new(label).font(fs(size::SMALL + 0.5)).color(p.text));
                            }
                            ui.add(egui::Label::new(RichText::new(stmt).size(size::SMALL + 0.5).color(p.dim)).wrap());
                        });
                        for (k, l) in LANES.iter().enumerate() {
                            ui.allocate_ui_with_layout(egui::vec2(cw, space::WHOLE_CELL_H), Layout::top_down(Align::Min), |ui| {
                                ui.set_width(cw);
                                ui.spacing_mut().item_spacing.y = gap;
                                for t in m.themes.iter().filter(|t| t.lane == *l && t.hyp == *hid) {
                                    let cf = motion::frame_at(el, moves::WHOLE_CELLS.stagger(n, moves::WHOLE_CELLS_STAGGER), reduced);
                                    n += 1;
                                    let gated = t.gate.as_ref().map_or(false, |g| Some(&g.0) == hid.as_ref());
                                    let r = motion::fade_in(ui, cf, egui::vec2(0.0, space::S8), |ui| whole_cell(ui, p, cw, space::WHOLE_CELL_H, &t.title, gated, k == 0));
                                    if r.clicked() {
                                        pick = Some(t.id.clone());
                                    }
                                }
                            });
                        }
                    });
                    ui.add_space(gap);
                }
                ui.add_space(space::S8);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = space::S16;
                    ui.label(RichText::new("◇ gate: the theme waits for that result").size(size::SMALL).color(p.dim));
                    ui.label(RichText::new("Click a cell to open the theme").size(size::SMALL).color(p.dim));
                });
            });
        });
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            close = true;
        }
        if close || pick.is_some() {
            self.temp.remove("rm_whole");
        }
        pick.map(Act::OpenTheme)
    }

    fn apply_roadmap(&mut self, acts: Vec<Act>, pid: &str, failed: Option<(Id, String, String, String)>) {
        for a in acts {
            match a {
                Act::Toggle(id) => {
                    // one open at a time
                    if self.temp.get("rm_open") == Some(&id) {
                        self.temp.remove("rm_open");
                    } else {
                        self.temp.insert("rm_open".into(), id);
                    }
                }
                Act::OpenTheme(id) => {
                    self.resolve_ui.roadmap_by_theme = false;
                    self.temp.insert("rm_open".into(), id);
                }
                Act::Move(id, lane) => self.roadmap_move(&id, lane),
                Act::Retry => {
                    if let Some((id, _, _, to)) = &failed {
                        self.temp.remove("rm_fail");
                        let to = LANES.iter().copied().find(|l| l == to).unwrap_or("Next");
                        self.roadmap_move(id, to);
                    }
                }
                Act::Dismiss => {
                    self.temp.remove("rm_fail");
                    if self.view_state == ViewState::Error {
                        self.view_state = ViewState::Full;
                    }
                }
                Act::Gate(id, h) => {
                    let title = self.store.data.record(&id).map(|r| r.title.clone()).unwrap_or_default();
                    let label = match &h {
                        Some(_) => format!("Gated \"{title}\""),
                        None => format!("Took the gate off \"{title}\""),
                    };
                    self.undoable(&label, "edit", move |d| {
                        if let Some(r) = d.record_mut(&id) {
                            let _ = r.set_field("gate", h.as_deref().unwrap_or(""));
                        }
                    });
                }
                Act::After(id, o) => {
                    let title = self.store.data.record(&id).map(|r| r.title.clone()).unwrap_or_default();
                    self.undoable(&format!("Changed what \"{title}\" comes after"), "edit", move |d| {
                        if let Some(r) = d.record_mut(&id) {
                            let _ = r.set_field("after", o.as_deref().unwrap_or(""));
                        }
                    });
                }
                Act::Status(id, s) => {
                    let title = self.store.data.record(&id).map(|r| r.title.clone()).unwrap_or_default();
                    self.undoable(&format!("Marked \"{title}\" {}", s.to_lowercase()), "edit", move |d| {
                        if let Some(r) = d.record_mut(&id) {
                            let _ = r.set_status(s);
                        }
                    });
                }
                Act::OpenTask(id) => self.open_task = Some(id),
                Act::CheckTask(id) => self.toggle_task(&id),
                Act::AddTask(theme) => {
                    let Some(r) = self.store.data.record(&theme).cloned() else { continue };
                    let mut t = Task::new("New task");
                    t.project_id = Some(pid.to_string());
                    t.section = lane_of(&r).to_string();
                    let tid = t.id.clone();
                    let th = theme.clone();
                    let link = tid.clone();
                    self.undoable(&format!("Added a task to \"{}\"", r.title), "add", move |d| {
                        d.tasks.push(t);
                        if let Some(r) = d.record_mut(&th) {
                            r.links.push(link);
                        }
                    });
                    self.open_task = Some(tid);
                }
                Act::OpenExp(id) => {
                    self.resolve_ui.sel_record = Some(id);
                    self.go(View::Sec(Section::Frameworks));
                }
            }
        }
    }

    /// C-E7: a phase move goes through history (Undo toast) and lands with F6; one that can't be
    /// made changes nothing and the Roadmap says so.
    fn roadmap_move(&mut self, id: &str, lane: &'static str) {
        let before = self.store.data.record(id).map(|r| (r.title.clone(), lane_of(r).to_string()));
        let mut result: Result<String, String> = Err(String::new());
        let id_s = id.to_string();
        if before.is_some() {
            let label = format!("Moved \"{}\" to {lane}", before.as_ref().map(|b| b.0.as_str()).unwrap_or(""));
            let r = &mut result;
            self.undoable(&label, "edit", |d| *r = move_theme(d, &id_s, lane));
        }
        match result {
            Ok(_) => {
                self.temp.remove("rm_fail");
                self.temp.insert("rm_landed".into(), id.to_string());
                self.temp.insert("rm_land_new".into(), id.to_string());
            }
            Err(_) => {
                let (title, from) = before.unwrap_or_else(|| ("That theme".into(), "its phase".into()));
                self.temp.insert("rm_fail".into(), format!("{id}\t{title}\t{from}\t{lane}"));
            }
        }
    }
}
