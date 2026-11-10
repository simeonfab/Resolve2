//! Frameworks (C-E9, C-E10, Pack C "Frameworks"): the registry strip (Hypothesis Zero, and a slot
//! for more methods), the tree (H0, each Hx with its negation NHx, experiments with what they
//! hold on), and the node editor beside it. A result goes result → hypothesis status → roadmap
//! gate → toast, in that order, as one undoable change (`apply_result`, self-tested). Saving
//! checks everything first: a save that can't be made writes nothing and the edit stays in the
//! panel.

use super::nav::FRAMEWORKS;
use super::theme::*;
use super::widgets::*;
use super::*;
use crate::resolve::{assess, Kind, Record};
use eframe::egui::{self, RichText};

/// One node of the tree as Frameworks draws it.
#[derive(Clone, Debug, Default)]
pub struct NodeView {
    pub id: Id,
    pub kind: Kind,
    pub depth: usize,
    /// "H0", "H1", "H2.1", "E0", "Ev"
    pub label: String,
    /// The statement without its label.
    pub title: String,
    /// The negation for a hypothesis ("NH1: …"), "Holds if …" for an experiment.
    pub sub: String,
    pub status: String,
    pub root: bool,
}

#[derive(Clone, Debug, Default)]
pub struct FrameworksModel {
    pub project: String,
    pub nodes: Vec<NodeView>,
    /// Hypotheses other than H0, experiments, and hypotheses no longer unresolved.
    pub hyps: usize,
    pub exps: usize,
    pub resolved: usize,
}

/// "H2: Café clients want a flat fee" → ("H2", "Café clients want a flat fee").
pub fn split_label(title: &str) -> (String, String) {
    match title.split_once(": ") {
        Some((l, rest)) if l.starts_with('H') && l.len() <= 6 && l[1..].chars().all(|c| c.is_ascii_digit() || c == '.') => (l.to_string(), rest.to_string()),
        _ => (String::new(), title.to_string()),
    }
}

/// E0, E1… in the order the project's experiments were made.
pub fn exp_label(d: &WorkspaceData, pid: &str, id: &str) -> String {
    d.records.iter().filter(|r| r.kind == Kind::Experiment && r.project_id.as_deref() == Some(pid)).position(|r| r.id == id).map(|i| format!("E{i}")).unwrap_or_else(|| "E".into())
}

pub fn frameworks_model(d: &WorkspaceData, pid: &str) -> FrameworksModel {
    let mut m = FrameworksModel::default();
    let Some(pr) = d.project(pid) else { return m };
    m.project = pr.name.clone();
    let tree = assess::hypothesis_tree(d, pid);
    let mut labels: std::collections::HashMap<Id, String> = Default::default();
    let mut kids: std::collections::HashMap<Id, usize> = Default::default();
    for (depth, id) in tree {
        let Some(r) = d.record(&id) else { continue };
        let root = r.kind == Kind::Hypothesis && r.field("type") == "H0";
        let (own, stmt) = split_label(&r.title);
        let label = match r.kind {
            Kind::Hypothesis if !own.is_empty() => own,
            Kind::Hypothesis => {
                // a sub-hypothesis: its parent's label and its place among its siblings
                let parent = r.parent_id.as_ref().and_then(|p| labels.get(p).cloned()).filter(|l| l != "H0");
                match parent {
                    Some(pl) => {
                        let k = kids.entry(r.parent_id.clone().unwrap_or_default()).or_insert(0);
                        *k += 1;
                        format!("{pl}.{k}")
                    }
                    None => "H".into(),
                }
            }
            Kind::Experiment => exp_label(d, pid, &r.id),
            _ => "Ev".into(),
        };
        labels.insert(r.id.clone(), label.clone());
        let sub = match r.kind {
            Kind::Hypothesis => {
                let n = r.field("negation").trim();
                if n.is_empty() {
                    String::new()
                } else if n.starts_with("NH") || label.is_empty() {
                    n.to_string()
                } else {
                    format!("N{label}: {n}")
                }
            }
            Kind::Experiment => {
                let s = r.field("support_if").trim();
                if s.is_empty() {
                    r.field("prediction").trim().to_string()
                } else {
                    format!("Holds if {s}")
                }
            }
            _ => r.field("source").trim().to_string(),
        };
        if r.kind == Kind::Hypothesis && !root {
            m.hyps += 1;
            if r.status != "Unresolved" {
                m.resolved += 1;
            }
        }
        if r.kind == Kind::Experiment {
            m.exps += 1;
        }
        m.nodes.push(NodeView { id: r.id.clone(), kind: r.kind, depth, label, title: stmt, sub, status: r.status.clone(), root });
    }
    m
}

/// The hypothesis status a result leads to (None: inconclusive, nothing changes).
pub fn result_status(result: &str) -> Option<&'static str> {
    match result.trim().to_lowercase().as_str() {
        "supports" => Some("Supported"),
        "weakens" => Some("Weakened"),
        "refutes" => Some("Refuted"),
        _ => None,
    }
}

/// C-E10: record an experiment's result, then the hypothesis status it leads to, then read the
/// roadmap gates that hypothesis holds; returns the toast. Checks first, so a result that can't
/// be recorded writes nothing.
pub fn apply_result(d: &mut WorkspaceData, exp_id: &str, result: &str) -> Result<String, String> {
    let e = d.record(exp_id).filter(|r| r.kind == Kind::Experiment).cloned().ok_or("That experiment isn't there any more.")?;
    let result = result.trim().to_lowercase();
    if !["supports", "weakens", "refutes", "inconclusive"].contains(&result.as_str()) {
        return Err(format!("\"{result}\" isn't a result: use supports, weakens, refutes or inconclusive."));
    }
    let pid = e.project_id.clone().unwrap_or_default();
    let el = exp_label(d, &pid, exp_id);
    // 1. the result
    if let Some(x) = d.record_mut(exp_id) {
        x.fields.insert("result".into(), result.clone());
        x.status = "Done".into();
        x.updated = now_ms();
    }
    let mut msg = format!("{el} result recorded: {result}.");
    // 2. the hypothesis status
    let Some(h) = e.parent_id.as_ref().and_then(|h| d.record(h)).filter(|h| h.kind == Kind::Hypothesis).cloned() else { return Ok(msg) };
    let (hl, _) = split_label(&h.title);
    let hl = if hl.is_empty() { "The hypothesis".to_string() } else { hl };
    match result_status(&result) {
        Some(s) => {
            if let Some(x) = d.record_mut(&h.id) {
                x.status = s.into();
                x.updated = now_ms();
            }
            msg.push_str(&format!(" {hl} is now {s}."));
        }
        None => msg.push_str(&format!(" {hl} stays {}.", h.status)),
    }
    // 3. the gates it holds
    let now_status = d.record(&h.id).map(|x| x.status.clone()).unwrap_or_default();
    let gates: Vec<String> = d.records.iter().filter(|r| r.kind == Kind::Roadmap && r.field("gate") == h.id && r.status != "Dropped").map(|r| r.title.clone()).collect();
    for g in gates {
        if now_status == "Supported" {
            msg.push_str(&format!(" The gate on {g} is open."));
        } else {
            msg.push_str(&format!(" The gate on {g} stays closed."));
        }
    }
    Ok(msg)
}

/// What the node editor holds until Save (kept in egui's memory, so closing and coming back
/// keeps it; Discard drops it).
#[derive(Clone, Debug, Default)]
pub struct Draft {
    pub id: Id,
    pub statement: String,
    pub negation: String,
    pub status: String,
    pub prediction: String,
    pub holds: String,
    pub fails: String,
    pub result: String,
}

pub fn draft_of(r: &Record) -> Draft {
    Draft {
        id: r.id.clone(),
        statement: split_label(&r.title).1,
        negation: r.field("negation").to_string(),
        status: r.status.clone(),
        prediction: r.field("prediction").to_string(),
        holds: r.field("support_if").to_string(),
        fails: r.field("refute_if").to_string(),
        result: r.field("result").to_string(),
    }
}

/// Save a node from its draft. Everything is checked on a copy first; nothing is written unless
/// all of it can be. Returns the toast.
pub fn save_node(d: &mut WorkspaceData, dr: &Draft) -> Result<String, String> {
    let orig = d.record(&dr.id).cloned().ok_or("It isn't there any more.")?;
    let mut r = orig.clone();
    let (label, _) = split_label(&orig.title);
    r.title = if label.is_empty() { dr.statement.trim().to_string() } else { format!("{label}: {}", dr.statement.trim()) };
    match r.kind {
        Kind::Hypothesis => {
            r.set_field("negation", dr.negation.trim())?;
            r.set_status(&dr.status)?;
        }
        Kind::Experiment => {
            r.set_field("prediction", dr.prediction.trim())?;
            r.set_field("support_if", dr.holds.trim())?;
            r.set_field("refute_if", dr.fails.trim())?;
            r.set_status(&dr.status)?;
        }
        _ => {}
    }
    r.updated = now_ms();
    let pid = orig.project_id.clone().unwrap_or_default();
    let name = match orig.kind {
        Kind::Experiment => exp_label(d, &pid, &orig.id),
        _ if !label.is_empty() => label.clone(),
        _ => orig.kind.label().to_string(),
    };
    if let Some(x) = d.record_mut(&dr.id) {
        *x = r.clone();
    }
    // a new result goes the C-E10 way; a hypothesis set Supported opens its gates
    if r.kind == Kind::Experiment && !dr.result.trim().is_empty() && dr.result.trim().to_lowercase() != orig.field("result").to_lowercase() {
        return apply_result(d, &dr.id, &dr.result);
    }
    let mut msg = format!("Saved {name}.");
    if r.kind == Kind::Hypothesis && r.status != orig.status {
        msg = format!("{name} is now {}.", r.status);
        for g in d.records.iter().filter(|x| x.kind == Kind::Roadmap && x.field("gate") == r.id && x.status != "Dropped") {
            msg.push_str(&format!(" The gate on {} {}.", g.title, if r.status == "Supported" { "is open" } else { "stays closed" }));
        }
    }
    Ok(msg)
}

fn draft_key(id: &str) -> egui::Id {
    egui::Id::new(("fw-draft", id))
}

enum FwAct {
    Select(Id),
    Close,
    Save(Draft),
    Discard(Id),
    AddHyp,
    AddExp(Option<Id>),
    AddSub(Id),
    WriteH0,
    Suggest,
}

impl App {
    /// The redesigned Frameworks for one project. Returns false with no project picked.
    pub(super) fn frameworks_page(&mut self, ui: &mut egui::Ui, p: &Pal) -> bool {
        let Some(pid) = self.current_project.clone() else { return false };
        let m = frameworks_model(&self.store.data, &pid);
        let state = match self.view_state {
            ViewState::Full if m.nodes.is_empty() => ViewState::Empty,
            s => s,
        };
        // a node selected elsewhere (Overview, Roadmap) opens here when it belongs to the tree
        let mut sel = self.resolve_ui.sel_record.clone().filter(|s| m.nodes.iter().any(|n| &n.id == s));
        if state == ViewState::Error && self.temp.get("fw_fail").is_none() {
            if let Some(h2) = m.nodes.iter().find(|n| n.label == "H2").or_else(|| m.nodes.iter().find(|n| n.kind == Kind::Hypothesis && !n.root)) {
                sel = Some(h2.id.clone());
                self.temp.insert("fw_fail".into(), format!("{}\tIt said no.", h2.id));
            }
        }
        let failed: Option<(Id, String)> = self.temp.get("fw_fail").and_then(|s| s.split_once('\t').map(|(a, b)| (a.to_string(), b.to_string())));
        let busy = self.resolve_ui.rx.is_some();
        let mut acts: Vec<FwAct> = vec![];
        egui::ScrollArea::vertical().auto_shrink([false, false]).id_salt(("frameworks", &pid)).show(ui, |ui| {
            egui::Frame::none().inner_margin(egui::Margin { left: space::WORK_SIDE, right: space::WORK_SIDE, top: space::WORK_TOP, bottom: space::S40 * 2.0 }).show(ui, |ui| {
                capped(ui, space::CAP_DEFAULT, |ui| {
                    eyebrow_head(ui, p, &m.project, "Frameworks", |ui| {
                        ui.spacing_mut().item_spacing.x = space::S10;
                        if button(ui, p, ButtonKind::Primary, "+ Experiment", false, state == ViewState::Full).clicked() {
                            acts.push(FwAct::AddExp(sel.clone()));
                        }
                        if button(ui, p, ButtonKind::Secondary, "+ Hypothesis", false, state != ViewState::Loading).clicked() {
                            acts.push(FwAct::AddHyp);
                        }
                    });
                    ui.add_space(space::S12);
                    // the registry: Hypothesis Zero, and a slot for more methods. The one-line
                    // description beside it is parked for a brainstorm, so its slot stays empty.
                    ui.horizontal_top(|ui| {
                        ui.spacing_mut().item_spacing.x = space::S10;
                        for f in FRAMEWORKS {
                            registry_card(ui, p, space::FW_REGISTRY_W, f.name, &format!("{} hypothes{} · {} experiment{} · {} resolved", m.hyps, if m.hyps == 1 { "is" } else { "es" }, m.exps, if m.exps == 1 { "" } else { "s" }, m.resolved), true, false);
                        }
                        registry_card(ui, p, space::FW_REGISTRY_MORE_W, "More methods later", "The registry holds one method today", false, true);
                    });
                    ui.add_space(space::S14);
                    match state {
                        ViewState::Loading => {
                            skeleton(ui, p, ui.available_width(), space::S40);
                            for k in 0..3 {
                                ui.add_space(space::S8);
                                ui.horizontal(|ui| {
                                    ui.add_space(space::FW_INDENT * (1 + k % 2) as f32 - space::S8);
                                    skeleton(ui, p, ui.available_width(), space::S40 + space::S8);
                                });
                            }
                            ui.add_space(space::S8);
                            ui.label(RichText::new("Reading the hypotheses…").size(size::SMALL).color(p.dim));
                            return;
                        }
                        ViewState::Empty => {
                            ui.add_space(space::S40 * 1.5);
                            let (a, b) = empty_center_two(ui, p, Icon::Frameworks, "No hypotheses yet.", "Start with H0: the one belief this whole project rests on. Resolve suggests the ones under it.", "Write H0", "Suggest from my brief");
                            if a {
                                acts.push(FwAct::WriteH0);
                            }
                            if b && !busy {
                                acts.push(FwAct::Suggest);
                            }
                            self.proposals_panel(ui, p, &pid);
                            return;
                        }
                        _ => {}
                    }
                    if let Some((fid, why)) = &failed {
                        let name = m.nodes.iter().find(|n| &n.id == fid).map(|n| n.label.clone()).filter(|l| !l.is_empty()).unwrap_or_else(|| "Your edit".into());
                        ui.add_space(space::S12);
                        let body = format!("Your edit is still in the panel. The roadmap gate kept its old state. Try saving again.{}", if why.is_empty() || why == "It said no." { String::new() } else { format!(" It said: {why}") });
                        let mut out = (false, false);
                        ui.vertical_centered(|ui| {
                            ui.set_max_width(space::SEARCH_W - space::S40 * 4.0);
                            white_card(ui, p, false, |ui| {
                                ui.spacing_mut().item_spacing.y = space::S8;
                                ui.label(RichText::new(format!("{name} didn't save.")).font(fserif(size::CARD_SERIF)).color(p.text));
                                ui.add(egui::Label::new(RichText::new(body).size(size::ROW).color(p.dim)).wrap());
                                ui.add_space(space::S8);
                                ui.horizontal(|ui| {
                                    out.0 = button(ui, p, ButtonKind::Dark, "Save again", false, true).clicked();
                                    out.1 = button(ui, p, ButtonKind::Secondary, "Discard my edit", false, true).clicked();
                                });
                            });
                        });
                        if out.0 {
                            let dr = ui.data(|d| d.get_temp::<Draft>(draft_key(fid))).or_else(|| self.store.data.record(fid).map(draft_of));
                            if let Some(dr) = dr {
                                acts.push(FwAct::Save(dr));
                            }
                        }
                        if out.1 {
                            acts.push(FwAct::Discard(fid.clone()));
                        }
                        return;
                    }
                    self.proposals_panel(ui, p, &pid);
                    let w = ui.available_width();
                    let editor = sel.is_some();
                    let tw = if editor { w - space::FW_EDITOR - space::S16 } else { w };
                    ui.horizontal_top(|ui| {
                        ui.spacing_mut().item_spacing.x = space::S16;
                        ui.allocate_ui_with_layout(egui::vec2(tw, 0.0), Layout::top_down(Align::Min), |ui| {
                            ui.set_width(tw);
                            white_card(ui, p, false, |ui| {
                                ui.spacing_mut().item_spacing.y = space::S8;
                                for n in &m.nodes {
                                    let selected = sel.as_deref() == Some(n.id.as_str());
                                    let style = if n.root {
                                        NodeStyle::Root
                                    } else if n.kind == Kind::Hypothesis {
                                        NodeStyle::Hypothesis
                                    } else {
                                        NodeStyle::Experiment
                                    };
                                    let indent = if n.depth == 0 { 0.0 } else { space::FW_INDENT * n.depth as f32 - space::S8 };
                                    ui.horizontal_top(|ui| {
                                        ui.add_space(indent);
                                        ui.spacing_mut().item_spacing.x = 0.0;
                                        ui.allocate_ui_with_layout(egui::vec2(ui.available_width(), 0.0), Layout::top_down(Align::Min), |ui| {
                                            if tree_row(ui, p, egui::Id::new(("fw-node", &n.id)), style, &n.label, &n.title, &n.sub, &n.status, selected).clicked() {
                                                acts.push(FwAct::Select(n.id.clone()));
                                            }
                                        });
                                    });
                                }
                            });
                        });
                        if let Some(id) = &sel {
                            ui.allocate_ui_with_layout(egui::vec2(space::FW_EDITOR, 0.0), Layout::top_down(Align::Min), |ui| {
                                ui.set_width(space::FW_EDITOR);
                                self.node_editor(ui, p, &pid, id, &m, &mut acts);
                            });
                        }
                    });
                });
            });
        });
        self.apply_frameworks(ui.ctx(), &pid, acts);
        true
    }

    /// The node editor (C-E9): statement, negation and status for a hypothesis; the test, its
    /// prediction, thresholds and result for an experiment; Save writes it all at once.
    fn node_editor(&mut self, ui: &mut egui::Ui, p: &Pal, pid: &str, id: &str, m: &FrameworksModel, acts: &mut Vec<FwAct>) {
        let Some(r) = self.store.data.record(id).cloned() else { return };
        let node = m.nodes.iter().find(|n| n.id == id).cloned().unwrap_or_default();
        let key = draft_key(id);
        let mut dr: Draft = ui.data_mut(|d| d.get_temp_mut_or_insert_with(key, || draft_of(&r)).clone());
        let mut all_fields = self.temp.get("fw_all").map_or(false, |x| x == id);
        white_card(ui, p, false, |ui| {
            ui.spacing_mut().item_spacing.y = space::S8;
            ui.horizontal(|ui| {
                ui.label(RichText::new(&node.label).font(fs(size::ROW_SMALL)).color(p.orange));
                ui.label(RichText::new(if node.root { "Hypothesis Zero" } else { r.kind.label() }).size(size::ROW_SMALL).color(p.dim));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if button(ui, p, ButtonKind::Secondary, "✕", true, true).on_hover_text("Close (your edit stays here until you save or discard it)").clicked() {
                        acts.push(FwAct::Close);
                    }
                });
            });
            let w = ui.available_width();
            match r.kind {
                Kind::Hypothesis => {
                    field_label(ui, p, "We believe");
                    text_field(ui, p, egui::Id::new(("fw-stmt", id)), &mut dr.statement, "The belief, in one sentence", true, w);
                    let nl = if node.label.is_empty() { "NHx".to_string() } else { format!("N{}", node.label) };
                    field_label(ui, p, &format!("It's wrong if ({nl})"));
                    text_field(ui, p, egui::Id::new(("fw-neg", id)), &mut dr.negation, "What would show it's wrong", true, w);
                    field_label(ui, p, "Status");
                    let opts = Kind::Hypothesis.statuses();
                    let sel = opts.iter().position(|s| *s == dr.status).unwrap_or(0);
                    if let Some(i) = segmented(ui, p, egui::Id::new(("fw-status", id)), opts, sel) {
                        dr.status = opts[i].to_string();
                    }
                    // its experiments, with what they predict and hold on
                    for e in self.store.data.records.iter().filter(|x| x.kind == Kind::Experiment && x.parent_id.as_deref() == Some(id)) {
                        let el = exp_label(&self.store.data, pid, &e.id);
                        white_card(ui, p, false, |ui| {
                            ui.spacing_mut().item_spacing.y = space::S4;
                            ui.add(egui::Label::new(RichText::new(format!("{el} · {}", e.title)).font(fs(size::SMALL + 0.5)).color(p.text)).wrap());
                            let pr = e.field("prediction").trim();
                            lead_line(ui, p, "Prediction:", if pr.is_empty() { "not set yet." } else { pr });
                            let th = [e.field("support_if").trim(), e.field("refute_if").trim()].iter().filter(|s| !s.is_empty()).cloned().collect::<Vec<_>>().join(". ");
                            lead_line(ui, p, "Threshold:", if th.is_empty() { "not set yet." } else { &th });
                        });
                    }
                    let gates: Vec<String> = self.store.data.records.iter().filter(|x| x.kind == Kind::Roadmap && x.field("gate") == id && x.status != "Dropped").map(|x| format!("{} ({})", x.title, super::sections::lane_of(x))).collect();
                    for g in gates {
                        soft_note(ui, p, |ui| lead_line(ui, p, "Opens the gate on", &g));
                    }
                }
                Kind::Experiment => {
                    let locked = r.status != "Planned";
                    field_label(ui, p, "We test");
                    text_field(ui, p, egui::Id::new(("fw-stmt", id)), &mut dr.statement, "What you'll do", true, w);
                    ui.add_enabled_ui(!locked, |ui| {
                        field_label(ui, p, "Prediction");
                        text_field(ui, p, egui::Id::new(("fw-pred", id)), &mut dr.prediction, "What you expect to see if the hypothesis holds", true, w);
                        field_label(ui, p, "Holds if");
                        text_field(ui, p, egui::Id::new(("fw-hold", id)), &mut dr.holds, "The threshold that supports it", false, w);
                        field_label(ui, p, "Wrong if");
                        text_field(ui, p, egui::Id::new(("fw-fail", id)), &mut dr.fails, "The threshold that refutes it", false, w);
                    });
                    if locked {
                        ui.label(RichText::new("The prediction and thresholds were fixed when it started.").size(size::SMALL).color(p.dim));
                    }
                    field_label(ui, p, "Status");
                    let opts = Kind::Experiment.statuses();
                    let s = opts.iter().position(|x| *x == dr.status).unwrap_or(0);
                    if let Some(i) = segmented(ui, p, egui::Id::new(("fw-estatus", id)), opts, s) {
                        dr.status = opts[i].to_string();
                    }
                    field_label(ui, p, "Result");
                    let res = ["Supports", "Weakens", "Refutes", "Inconclusive"];
                    let ri = res.iter().position(|x| x.eq_ignore_ascii_case(dr.result.trim())).unwrap_or(res.len());
                    if let Some(i) = segmented(ui, p, egui::Id::new(("fw-result", id)), &res, ri) {
                        dr.result = res[i].to_lowercase();
                    }
                    if let Some(h) = r.parent_id.as_ref().and_then(|h| self.store.data.record(h)) {
                        let t = h.title.clone();
                        soft_note(ui, p, |ui| lead_line(ui, p, "Tests", &t));
                    }
                }
                _ => {
                    ui.label(RichText::new("Edit it with all its fields below.").size(size::SMALL).color(p.dim));
                    all_fields = true;
                }
            }
            ui.add_space(space::S6);
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(space::S8, space::S8);
                if matches!(r.kind, Kind::Hypothesis | Kind::Experiment) && button(ui, p, ButtonKind::Dark, "Save", false, true).clicked() {
                    acts.push(FwAct::Save(dr.clone()));
                }
                if r.kind == Kind::Hypothesis {
                    if button(ui, p, ButtonKind::Secondary, "+ Sub-hypothesis", false, true).clicked() {
                        acts.push(FwAct::AddSub(id.to_string()));
                    }
                    if button(ui, p, ButtonKind::Secondary, "+ Experiment", false, true).clicked() {
                        acts.push(FwAct::AddExp(Some(id.to_string())));
                    }
                }
                if r.kind != Kind::Evidence && link(ui, p, if all_fields { "Fewer fields" } else { "All fields…" }).clicked() {
                    all_fields = !all_fields;
                }
            });
        });
        ui.data_mut(|d| d.insert_temp(key, dr));
        if all_fields {
            self.temp.insert("fw_all".into(), id.to_string());
            ui.add_space(space::S10);
            self.record_editor(ui, p, pid, id);
            // the full editor writes as you type: keep the draft in step with it
            if let Some(r2) = self.store.data.record(id) {
                let fresh = draft_of(r2);
                ui.data_mut(|d| d.insert_temp(key, fresh));
            }
        } else if self.temp.get("fw_all").map_or(false, |x| x == id) {
            self.temp.remove("fw_all");
        }
    }

    fn apply_frameworks(&mut self, ctx: &egui::Context, pid: &str, acts: Vec<FwAct>) {
        for a in acts {
            match a {
                FwAct::Select(id) => {
                    self.resolve_ui.sel_record = if self.resolve_ui.sel_record.as_deref() == Some(id.as_str()) { None } else { Some(id) };
                }
                FwAct::Close => self.resolve_ui.sel_record = None,
                FwAct::Save(dr) => {
                    let mut out: Result<String, String> = Err(String::new());
                    // check on a copy first, so a failure writes nothing and the toast is honest
                    let mut probe = self.store.data.clone();
                    match save_node(&mut probe, &dr) {
                        Ok(label) => {
                            let o = &mut out;
                            self.undoable(&label, "edit", |d| *o = save_node(d, &dr));
                        }
                        Err(e) => out = Err(e),
                    }
                    match out {
                        Ok(_) => {
                            self.temp.remove("fw_fail");
                            ctx.data_mut(|d| d.remove::<Draft>(draft_key(&dr.id)));
                            if self.view_state == ViewState::Error {
                                self.view_state = ViewState::Full;
                            }
                        }
                        Err(e) => {
                            ctx.data_mut(|d| d.insert_temp(draft_key(&dr.id), dr.clone()));
                            self.temp.insert("fw_fail".into(), format!("{}\t{e}", dr.id));
                        }
                    }
                }
                FwAct::Discard(id) => {
                    ctx.data_mut(|d| d.remove::<Draft>(draft_key(&id)));
                    self.temp.remove("fw_fail");
                    if self.view_state == ViewState::Error {
                        self.view_state = ViewState::Full;
                    }
                }
                FwAct::AddHyp => {
                    let d = &self.store.data;
                    let h0 = d.records.iter().find(|r| r.kind == Kind::Hypothesis && r.project_id.as_deref() == Some(pid) && r.field("type") == "H0").map(|r| r.id.clone());
                    let n = d.records.iter().filter(|r| r.kind == Kind::Hypothesis && r.project_id.as_deref() == Some(pid) && r.field("type") != "H0" && r.parent_id == h0).count() + 1;
                    let mut r = Record::new(Kind::Hypothesis, Some(pid.to_string()), format!("H{n}: "));
                    r.parent_id = h0;
                    let id = r.id.clone();
                    self.undoable(&format!("Added H{n}"), "add", move |d| d.records.push(r));
                    self.resolve_ui.sel_record = Some(id);
                }
                FwAct::AddSub(parent) => {
                    let mut r = Record::new(Kind::Hypothesis, Some(pid.to_string()), "");
                    r.parent_id = Some(parent);
                    let id = r.id.clone();
                    self.undoable("Added a sub-hypothesis", "add", move |d| d.records.push(r));
                    self.resolve_ui.sel_record = Some(id);
                }
                FwAct::AddExp(under) => {
                    let d = &self.store.data;
                    let parent = under.filter(|u| d.record(u).map_or(false, |r| r.kind == Kind::Hypothesis)).or_else(|| d.records.iter().find(|r| r.kind == Kind::Hypothesis && r.project_id.as_deref() == Some(pid) && r.field("type") != "H0").map(|r| r.id.clone()));
                    let mut r = Record::new(Kind::Experiment, Some(pid.to_string()), "");
                    r.parent_id = parent;
                    let id = r.id.clone();
                    self.undoable("Added an experiment", "add", move |d| d.records.push(r));
                    self.resolve_ui.sel_record = Some(id);
                }
                FwAct::WriteH0 => {
                    let mut r = Record::new(Kind::Hypothesis, Some(pid.to_string()), "H0: ");
                    r.fields.insert("type".into(), "H0".into());
                    let id = r.id.clone();
                    self.undoable("Added H0", "add", move |d| d.records.push(r));
                    self.resolve_ui.sel_record = Some(id);
                }
                FwAct::Suggest => {
                    if let Some(c) = self.ensure_context(pid) {
                        self.temp.insert("overview_action".into(), "validation-plan".into());
                        self.resolve_ui.error.clear();
                        self.run_overview_action(pid, &c, "validation-plan");
                    }
                }
            }
        }
    }
}
