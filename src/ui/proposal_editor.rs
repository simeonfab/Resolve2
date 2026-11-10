//! The proposal editor (C-E11, Pack C "ProposalEditor"): the intake's "See and edit everything",
//! full window. A top bar (← Back to the summary, the proposal's name, Saved, Adopt), the section
//! list on the left, and the sections as cards you edit where the text stands. Every edit goes
//! into the intake as you type, so Back keeps it and Adopt uses it. Regenerate and Remove stay
//! on each section.

use super::theme::*;
use super::widgets::*;
use super::*;
use crate::resolve::intake::Intake;
use crate::resolve::journey::{self, Item, Proposal};

/// The sections: (key for scrolling, name in the list, the proposal's "removed" key).
const SECTIONS: [(&str, &str, &str); 8] = [
    ("name", "Name and summary", ""),
    ("outcome", "Outcome and H0", "objective"),
    ("hyps", "Hypotheses", "h0"),
    ("exps", "Experiments", "experiments"),
    ("unc", "Uncertainties", "uncertainties"),
    ("roadmap", "Roadmap", "roadmap"),
    ("week", "First week", "tasks"),
    ("pages", "Pages", "pages"),
];

fn count(pr: &Proposal, key: &str) -> Option<usize> {
    match key {
        "hyps" => Some(pr.hypotheses.len()),
        "exps" => Some(pr.experiments.len()),
        "unc" => Some(pr.uncertainties.len()),
        "roadmap" => Some(pr.roadmap.len()),
        "week" => Some(pr.tasks.len()),
        "pages" => Some(pr.pages.len()),
        _ => None,
    }
}

/// Has the proposal anything to edit beyond a name and an outcome?
pub fn proposal_is_thin(pr: &Proposal) -> bool {
    pr.hypotheses.is_empty() && pr.roadmap.is_empty() && pr.tasks.is_empty() && pr.experiments.is_empty()
}

/// What a card's head asks for.
#[derive(Default)]
struct HeadOut {
    regen: bool,
    toggle: bool,
    add: bool,
}

/// A card's head: the serif title, then on the right a note or an add button, Regenerate and
/// Remove (or Restore).
fn card_head(ui: &mut egui::Ui, p: &Pal, title: &str, note: &str, add: Option<&str>, ai: bool, removable: bool, removed: bool) -> HeadOut {
    let mut out = HeadOut::default();
    ui.horizontal(|ui| {
        ui.label(RichText::new(title).font(fserif(size::CARD_SERIF)).color(if removed { p.faint } else { p.text }));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = space::S8;
            if removable && button(ui, p, ButtonKind::Ghost, if removed { "Restore" } else { "Remove" }, true, true).on_hover_text(if removed { "Put this section back" } else { "Leave this section out: it won't be created" }).clicked() {
                out.toggle = true;
            }
            if ai && !removed && button(ui, p, ButtonKind::Ghost, "↻ Regenerate", true, true).clicked() {
                out.regen = true;
            }
            if let Some(a) = add.filter(|_| !removed) {
                out.add = button(ui, p, ButtonKind::Secondary, a, true, true).clicked();
            }
            if !note.is_empty() {
                ui.label(RichText::new(note).size(size::SMALL).color(p.dim));
            }
        });
    });
    ui.add_space(space::S6);
    out
}

fn removed_line(ui: &mut egui::Ui, p: &Pal) {
    ui.label(RichText::new("Removed. It won't be created.").size(size::ROW_SMALL).color(p.faint));
}

/// The H labels of the hypotheses and which experiments test each.
fn exp_chips(pr: &Proposal, h: usize) -> Vec<String> {
    pr.experiments.iter().enumerate().filter(|(_, e)| e.parent == Some(h)).map(|(i, _)| format!("E{i}")).collect()
}

impl App {
    /// The full-window proposal editor. `it.proposal` is edited in place.
    pub(super) fn proposal_editor_page(&mut self, ui: &mut egui::Ui, p: &Pal, it: &mut Intake) {
        let Some(mut pr) = it.proposal.clone() else {
            it.editor = false;
            return;
        };
        let ai = journey::cfg_for(it).is_some() && !pr.offline;
        let busy = self.resolve_ui.rx.is_some();
        let save_err = self.temp.get("save_error").cloned();
        let state = match self.view_state {
            ViewState::Full if busy => ViewState::Loading,
            ViewState::Full if save_err.is_some() => ViewState::Error,
            ViewState::Full if proposal_is_thin(&pr) => ViewState::Empty,
            s => s,
        };
        let full = ui.max_rect();
        ui.painter().rect_filled(full, 0.0, p.bg);
        // the top bar
        let bar = egui::Rect::from_min_size(full.min, egui::vec2(full.width(), space::PE_BAR));
        let (mut back, mut adopt) = (false, false);
        let name_ok = !pr.name.trim().is_empty();
        editor_bar(ui, p, bar, |ui| {
            back = button(ui, p, ButtonKind::Secondary, "← Back to the summary", false, true).on_hover_text("Your edits are kept").clicked();
            let rest = ui.available_rect_before_wrap();
            let mut job = egui::text::LayoutJob::default();
            job.append("Proposal · ", 0.0, egui::TextFormat { font_id: fp(size::ROW_SMALL), color: p.dim, ..Default::default() });
            job.append(if name_ok { pr.name.trim() } else { "Untitled" }, 0.0, egui::TextFormat { font_id: fs(size::ROW + 0.5), color: p.text, ..Default::default() });
            let g = ui.painter().layout_job(job);
            ui.painter().galley(egui::pos2(full.center().x - g.size().x / 2.0, rest.center().y - g.size().y / 2.0), g, p.text);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = space::S14;
                adopt = button(ui, p, ButtonKind::Primary, "Adopt and create this project", false, name_ok && !busy).on_hover_text("Writes the project, roadmap, tasks, validation and pages. You can undo it.").clicked();
                let saved = if state == ViewState::Error { "Not saved" } else if self.dirty { "Saving…" } else { "Saved" };
                ui.label(RichText::new(saved).size(size::ROW_SMALL).color(if saved == "Not saved" { p.orange } else { p.dim }));
            });
        });
        // the body: the section list, then the cards, capped at 1240 and centred
        let body = egui::Rect::from_min_max(egui::pos2(full.left(), bar.bottom()), full.max);
        let cap = space::CAP_DEFAULT.min(body.width() - space::S40 * 2.0);
        let left = body.center().x - cap / 2.0;
        let list = egui::Rect::from_min_size(egui::pos2(left, body.top() + space::S22), egui::vec2(space::PE_LIST, body.height() - space::S22));
        let mut goto: Option<&'static str> = None;
        let current = self.temp.get("pe_sec").cloned().unwrap_or_else(|| "name".into());
        {
            let mut lu = ui.new_child(egui::UiBuilder::new().max_rect(list).layout(Layout::top_down(Align::Min)));
            lu.spacing_mut().item_spacing.y = space::S4 / 2.0;
            for (key, name, _) in SECTIONS {
                if section_item(&mut lu, p, space::PE_LIST, name, count(&pr, key), current == key).clicked() {
                    goto = Some(key);
                }
            }
            lu.add_space(space::S18);
            lu.allocate_ui_with_layout(egui::vec2(space::PE_LIST - space::S10 * 2.0, 0.0), Layout::top_down(Align::Min), |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(space::S10);
                    ui.add(egui::Label::new(RichText::new("Nothing here is final. You can change any of it once your project is open.").size(size::SMALL + 0.5).color(p.dim)).wrap());
                });
            });
        }
        let cards = egui::Rect::from_min_max(egui::pos2(list.right() + space::PE_GAP, body.top()), egui::pos2(left + cap, body.bottom()));
        let mut cu = ui.new_child(egui::UiBuilder::new().max_rect(cards).layout(Layout::top_down(Align::Min)));
        let mut regen: Option<&'static str> = None;
        let mut toggle: Option<&'static str> = None;
        let (mut more_q, mut blank, mut save_again) = (false, false, false);
        let scroll_to = self.temp.remove("pe_goto");
        let mut seen: Vec<(&'static str, f32)> = vec![];
        egui::ScrollArea::vertical().auto_shrink([false, false]).id_salt(("proposal-editor", &it.id)).show(&mut cu, |ui| {
            ui.add_space(space::S22);
            ui.set_width(cards.width() - space::S8);
            match state {
                ViewState::Loading => {
                    ui.spacing_mut().item_spacing.y = space::S10;
                    let w = ui.available_width();
                    skeleton_card(ui, p, 3, "");
                    skeleton_card(ui, p, 5, "");
                    let _ = w;
                    ui.label(RichText::new(if busy { self.resolve_ui.busy.clone() } else { "Writing the full proposal from your answers…".into() }).size(size::SMALL).color(p.dim));
                    return;
                }
                ViewState::Empty => {
                    ui.add_space(space::S40);
                    let (a, b) = empty_center_two(ui, p, Icon::Overview, "Resolve didn't have enough to propose yet.", "You skipped most of the questions. Answer two or three more, or start the project and fill it in yourself.", "Answer more questions", "Start it blank");
                    more_q = a;
                    blank = b;
                    return;
                }
                ViewState::Error => {
                    ui.add_space(space::S12);
                    let body = format!("Your new wording is still here, but it isn't saved on this PC yet. Save it again before you adopt or close Resolve.{}", save_err.as_ref().map(|e| format!(" It said: {e}")).unwrap_or_default());
                    let a = error_center_as(ui, p, "Your last edit didn't save.", &body, "Save again", "Back to the summary");
                    save_again = a.retry;
                    back |= a.manual;
                    return;
                }
                _ => {}
            }
            ui.spacing_mut().item_spacing.y = space::S14;
            let rm = |pr: &Proposal, k: &str| pr.removed.iter().any(|x| x == k);
            let mut card = |ui: &mut egui::Ui, key: &'static str, add: &mut dyn FnMut(&mut egui::Ui, &mut Proposal)| {
                let top = ui.cursor().min.y;
                let r = egui::Frame::none().fill(p.card).stroke(egui::Stroke::new(stroke::HAIR, p.line)).rounding(egui::Rounding::same(radius::CARD + 1.0)).inner_margin(egui::Margin::symmetric(space::S18, space::S16)).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.spacing_mut().item_spacing.y = space::S8;
                    add(ui, &mut pr);
                });
                if scroll_to.as_deref() == Some(key) {
                    ui.scroll_to_rect(r.response.rect, Some(Align::TOP));
                }
                seen.push((key, top));
            };
            // name and summary
            card(ui, "name", &mut |ui, pr| {
                let h = card_head(ui, p, "Name and summary", "Click any text to edit it", None, ai, false, false);
                if h.regen {
                    regen = Some("summary");
                }
                let w = ui.available_width();
                field_label(ui, p, "Project name");
                text_field(ui, p, egui::Id::new(("pe-name", 0)), &mut pr.name, "Short: name the real thing", false, w);
                if pr.name.split_whitespace().count() > 6 {
                    ui.label(RichText::new("Keep it short: a few words naming the real thing.").size(size::SMALL).color(p.orange));
                }
                field_label(ui, p, "One-line summary");
                text_field(ui, p, egui::Id::new(("pe-summary", 0)), &mut pr.summary, "What this project is, in a line", false, w);
            });
            // outcome and H0
            card(ui, "outcome", &mut |ui, pr| {
                let removed = rm(pr, "objective");
                let h = card_head(ui, p, "Outcome and H0", "", None, ai, true, removed);
                if h.toggle {
                    toggle = Some("objective");
                }
                if h.regen {
                    regen = Some("objective");
                }
                if removed {
                    removed_line(ui, p);
                    return;
                }
                let w = ui.available_width();
                field_label(ui, p, "Objective");
                text_field(ui, p, egui::Id::new(("pe-objective", 0)), &mut pr.objective, "What you are setting out to do", false, w);
                field_label(ui, p, "Outcome");
                text_field(ui, p, egui::Id::new(("pe-outcome", 0)), &mut pr.outcome, "What is true when this works", false, w);
                field_label(ui, p, "H0 · the belief everything rests on");
                text_field(ui, p, egui::Id::new(("pe-h0", 0)), &mut pr.h0.title, "The one belief this project rests on", false, w);
                field_label(ui, p, "It's wrong if (NH0)");
                text_field(ui, p, egui::Id::new(("pe-nh0", 0)), &mut pr.h0.negation, "What would show H0 is wrong", false, w);
            });
            // hypotheses and experiments
            card(ui, "hyps", &mut |ui, pr| {
                let removed = rm(pr, "h0");
                let h = card_head(ui, p, "Hypotheses and experiments", "", Some("+ Hypothesis"), ai, true, removed);
                if h.toggle {
                    toggle = Some("h0");
                }
                if h.regen {
                    regen = Some("h0");
                }
                if h.add {
                    pr.hypotheses.push(Item { basis: "uncertain".into(), ..Default::default() });
                }
                if removed {
                    removed_line(ui, p);
                    return;
                }
                let mut remove: Option<usize> = None;
                for n in 0..pr.hypotheses.len() {
                    let chips = exp_chips(pr, n);
                    let hx = &mut pr.hypotheses[n];
                    edit_row(ui, p, |ui| {
                        ui.horizontal_top(|ui| {
                            ui.spacing_mut().item_spacing.x = space::S12;
                            ui.allocate_ui_with_layout(egui::vec2(space::S22, 0.0), Layout::top_down(Align::Min), |ui| { ui.set_width(space::S22); ui.label(RichText::new(format!("H{}", n + 1)).font(fs(size::SMALL + 0.5)).color(p.orange)); });
                            let cw: f32 = if chips.is_empty() { space::S40 * 2.0 } else { chips.len() as f32 * space::S40 };
                            let tw = ui.available_width() - cw - space::S22 - space::S12 * 2.0;
                            ui.vertical(|ui| {
                                ui.set_width(tw);
                                ui.spacing_mut().item_spacing.y = space::S4;
                                inline_edit(ui, p, egui::Id::new(("pe-hyp", n)), &mut hx.title, fp(size::ROW + 0.5), p.text, tw, "The hypothesis", false);
                                inline_edit(ui, p, egui::Id::new(("pe-neg", n)), &mut hx.negation, fp(size::SMALL + 0.5), p.dim, tw, &format!("NH{}: what would show it's wrong", n + 1), false);
                            });
                            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                                ui.spacing_mut().item_spacing.x = space::S6;
                                if remove_x(ui, p).clicked() {
                                    remove = Some(n);
                                }
                                if chips.is_empty() {
                                    tag(ui, "No test yet", p.dim, p.surface_raised);
                                }
                                for c in chips.iter().rev() {
                                    tag(ui, c, p.dim, p.surface_raised);
                                }
                            });
                        });
                    });
                }
                if let Some(n) = remove {
                    pr.hypotheses.remove(n);
                    // experiments that tested it now test H0; later ones move up one
                    for e in pr.experiments.iter_mut() {
                        e.parent = match e.parent {
                            Some(x) if x == n => None,
                            Some(x) if x > n => Some(x - 1),
                            x => x,
                        };
                    }
                }
            });
            // experiments
            card(ui, "exps", &mut |ui, pr| {
                let removed = rm(pr, "experiments");
                let h = card_head(ui, p, "Experiments", "E0 first", Some("+ Experiment"), ai, true, removed);
                if h.toggle {
                    toggle = Some("experiments");
                }
                if h.regen {
                    regen = Some("experiments");
                }
                if h.add {
                    pr.experiments.push(Item { basis: "uncertain".into(), ..Default::default() });
                }
                if removed {
                    removed_line(ui, p);
                    return;
                }
                let hyp_titles: Vec<String> = pr.hypotheses.iter().enumerate().map(|(i, h)| format!("H{}: {}", i + 1, h.title)).collect();
                let mut remove: Option<usize> = None;
                for (n, e) in pr.experiments.iter_mut().enumerate() {
                    edit_row(ui, p, |ui| {
                        ui.horizontal_top(|ui| {
                            ui.spacing_mut().item_spacing.x = space::S12;
                            ui.allocate_ui_with_layout(egui::vec2(space::S22, 0.0), Layout::top_down(Align::Min), |ui| { ui.set_width(space::S22); ui.label(RichText::new(format!("E{n}")).font(fs(size::SMALL + 0.5)).color(p.dim)); });
                            let tw = ui.available_width() - space::S22 - space::S12;
                            ui.vertical(|ui| {
                                ui.set_width(tw);
                                ui.spacing_mut().item_spacing.y = space::S4;
                                inline_edit(ui, p, egui::Id::new(("pe-exp", n)), &mut e.title, fp(size::ROW + 0.5), p.text, tw, "What you'll do", false);
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("Tests").size(size::SMALL).color(p.dim));
                                    let sel = e.parent.and_then(|x| hyp_titles.get(x)).cloned().unwrap_or_else(|| "H0".into());
                                    egui::ComboBox::from_id_salt(("pe-exp-parent", n)).width(tw - space::S40 * 2.5).truncate().selected_text(RichText::new(sel).size(size::SMALL)).show_ui(ui, |ui| {
                                        ui.selectable_value(&mut e.parent, None, "H0");
                                        for (k, h) in hyp_titles.iter().enumerate() {
                                            ui.selectable_value(&mut e.parent, Some(k), h);
                                        }
                                    });
                                });
                                for (k, label) in [("prediction", "Prediction"), ("support_if", "Holds if"), ("refute_if", "Wrong if")] {
                                    let mut v = e.fields.get(k).cloned().unwrap_or_default();
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(label).font(fs(size::SMALL)).color(p.text));
                                        let w2 = ui.available_width();
                                        if inline_edit(ui, p, egui::Id::new(("pe-exp-f", n, k)), &mut v, fp(size::SMALL + 0.5), p.text, w2, "not set yet", false).changed() {
                                            e.fields.insert(k.to_string(), v.clone());
                                        }
                                    });
                                }
                            });
                            if remove_x(ui, p).clicked() {
                                remove = Some(n);
                            }
                        });
                    });
                }
                if let Some(n) = remove {
                    pr.experiments.remove(n);
                }
            });
            // uncertainties
            card(ui, "unc", &mut |ui, pr| {
                let removed = rm(pr, "uncertainties");
                let n = pr.uncertainties.len();
                let h = card_head(ui, p, "Uncertainties", &format!("{n} · these become open questions"), None, ai, true, removed);
                if h.toggle {
                    toggle = Some("uncertainties");
                }
                if h.regen {
                    regen = Some("uncertainties");
                }
                if removed {
                    removed_line(ui, p);
                    return;
                }
                let mut remove: Option<usize> = None;
                for (k, u) in pr.uncertainties.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = space::S10;
                        let (r, _) = ui.allocate_exact_size(egui::vec2(space::S8, space::S8), egui::Sense::hover());
                        ui.painter().circle_filled(r.center(), space::S4 - 1.0, tok::AMBER);
                        let w = ui.available_width() - space::S22 - space::S10;
                        inline_edit(ui, p, egui::Id::new(("pe-unc", k)), &mut u.title, fp(size::ROW), p.text, w, "Something you don't know yet", false);
                        if remove_x(ui, p).clicked() {
                            remove = Some(k);
                        }
                    });
                }
                if let Some(k) = remove {
                    pr.uncertainties.remove(k);
                }
                if link(ui, p, "+ Uncertainty").clicked() {
                    pr.uncertainties.push(Item { basis: "uncertain".into(), ..Default::default() });
                }
            });
            // roadmap
            card(ui, "roadmap", &mut |ui, pr| {
                let removed = rm(pr, "roadmap");
                let h = card_head(ui, p, "Roadmap", "Now, Next and Later", Some("+ Theme"), ai, true, removed);
                if h.toggle {
                    toggle = Some("roadmap");
                }
                if h.regen {
                    regen = Some("roadmap");
                }
                if h.add {
                    pr.roadmap.push(Item { basis: "uncertain".into(), phase: "Now".into(), ..Default::default() });
                }
                if removed {
                    removed_line(ui, p);
                    return;
                }
                let mut remove: Option<usize> = None;
                for (k, r) in pr.roadmap.iter_mut().enumerate() {
                    edit_row(ui, p, |ui| {
                        ui.horizontal_top(|ui| {
                            ui.spacing_mut().item_spacing.x = space::S10;
                            let phase = if r.phase.is_empty() { "Now".to_string() } else { r.phase.clone() };
                            let sel = super::sections::LANES.iter().position(|l| *l == phase).unwrap_or(0);
                            if let Some(i) = segmented(ui, p, egui::Id::new(("pe-rm-phase", k)), &super::sections::LANES, sel) {
                                r.phase = super::sections::LANES[i].into();
                            }
                            let chip = ui.allocate_ui_with_layout(egui::vec2(space::S40 * 2.0, 0.0), Layout::top_down(Align::Min), |ui| { ui.set_width(space::S40 * 2.0); basis_chip(ui, p, if r.basis.is_empty() { "uncertain" } else { &r.basis }) }).inner;
                            if chip.on_hover_text("What it rests on. Click to change").on_hover_cursor(egui::CursorIcon::PointingHand).interact(egui::Sense::click()).clicked() {
                                r.basis = match r.basis.as_str() {
                                    "evidence" => "assumption".into(),
                                    "assumption" => "uncertain".into(),
                                    _ => "evidence".into(),
                                };
                            }
                            let tw = ui.available_width() - space::S22 - space::S10;
                            ui.vertical(|ui| {
                                ui.set_width(tw);
                                ui.spacing_mut().item_spacing.y = space::S4;
                                inline_edit(ui, p, egui::Id::new(("pe-rm", k)), &mut r.title, fp(size::ROW + 0.5), p.text, tw, "A theme", false);
                                inline_edit(ui, p, egui::Id::new(("pe-rm-d", k)), &mut r.detail, fp(size::SMALL + 0.5), p.dim, tw, "What it involves", true);
                            });
                            if remove_x(ui, p).clicked() {
                                remove = Some(k);
                            }
                        });
                    });
                }
                if let Some(k) = remove {
                    pr.roadmap.remove(k);
                    for t in pr.tasks.iter_mut() {
                        t.parent = match t.parent {
                            Some(x) if x == k => None,
                            Some(x) if x > k => Some(x - 1),
                            x => x,
                        };
                    }
                }
            });
            // the first week's tasks
            card(ui, "week", &mut |ui, pr| {
                let removed = rm(pr, "tasks");
                let h = card_head(ui, p, "First week", "Tasks", Some("+ Task"), ai, true, removed);
                if h.toggle {
                    toggle = Some("tasks");
                }
                if h.regen {
                    regen = Some("tasks");
                }
                if h.add {
                    pr.tasks.push(Item { basis: "uncertain".into(), phase: "Now".into(), ..Default::default() });
                }
                if removed {
                    removed_line(ui, p);
                    return;
                }
                let themes: Vec<String> = pr.roadmap.iter().map(|r| r.title.clone()).collect();
                let mut remove: Option<usize> = None;
                for (k, t) in pr.tasks.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = space::S10;
                        let cw = space::S40 * 5.0;
                        let w = ui.available_width() - cw - space::S22 - space::S10 * 3.0;
                        inline_edit(ui, p, egui::Id::new(("pe-task", k)), &mut t.title, fp(size::ROW), p.text, w, "A task", false);
                        let sel = t.parent.and_then(|x| themes.get(x)).cloned().unwrap_or_else(|| "No theme".into());
                        ui.allocate_ui_with_layout(egui::vec2(cw, space::S26), Layout::left_to_right(Align::Center), |ui| {
                            ui.set_width(cw);
                            egui::ComboBox::from_id_salt(("pe-task-theme", k)).width(cw - space::S40).truncate().selected_text(RichText::new(sel).size(size::SMALL)).show_ui(ui, |ui| {
                                ui.selectable_value(&mut t.parent, None, "No theme");
                                for (j, th) in themes.iter().enumerate() {
                                    ui.selectable_value(&mut t.parent, Some(j), th);
                                }
                            });
                        });
                        if remove_x(ui, p).clicked() {
                            remove = Some(k);
                        }
                    });
                }
                if let Some(k) = remove {
                    pr.tasks.remove(k);
                }
            });
            // pages
            card(ui, "pages", &mut |ui, pr| {
                let removed = rm(pr, "pages");
                let h = card_head(ui, p, "Pages", "The project brief first", None, ai, true, removed);
                if h.toggle {
                    toggle = Some("pages");
                }
                if h.regen {
                    regen = Some("pages");
                }
                if removed {
                    removed_line(ui, p);
                    return;
                }
                for (k, pg) in pr.pages.iter_mut().enumerate() {
                    edit_row(ui, p, |ui| {
                        let w = ui.available_width();
                        ui.spacing_mut().item_spacing.y = space::S4;
                        inline_edit(ui, p, egui::Id::new(("pe-page", k)), &mut pg.title, fs(size::ROW + 0.5), p.text, w, "Page title", false);
                        inline_edit(ui, p, egui::Id::new(("pe-page-d", k)), &mut pg.detail, fp(size::ROW_SMALL), p.dim, w, "What the page says", true);
                    });
                }
            });
            if ai {
                ui.horizontal(|ui| {
                    if button(ui, p, ButtonKind::Ghost, "↻ Regenerate everything", false, !busy).clicked() {
                        regen = Some("all");
                    }
                });
            }
            ui.add_space(space::S40 * 2.0);
        });
        // the list follows the scroll: the last section whose top has passed the bar
        if let Some((k, _)) = seen.iter().rev().find(|(_, y)| *y <= body.top() + space::S40 * 3.0) {
            if goto.is_none() && scroll_to.is_none() {
                self.temp.insert("pe_sec".into(), k.to_string());
            }
        }
        if let Some(k) = goto {
            self.temp.insert("pe_sec".into(), k.into());
            self.temp.insert("pe_goto".into(), k.into());
        }
        if let Some(k) = toggle {
            if pr.removed.iter().any(|x| x == k) {
                pr.removed.retain(|x| x != k);
            } else {
                pr.removed.push(k.to_string());
            }
        }
        it.proposal = Some(pr);
        if save_again {
            self.touch();
            self.save_now();
        } else if back {
            it.editor = false;
        } else if more_q {
            it.editor = false;
            it.stage = "questions".into();
            it.cursor = it.qas.len().saturating_sub(1);
        } else if blank || adopt {
            self.adopt_begin(it);
        } else if let Some(s) = regen {
            self.ij_spawn_proposal_pub(it, Some(s));
        }
    }
}
