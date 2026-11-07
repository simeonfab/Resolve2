//! The shell around every screen (Phase 1): the floating Ask Resolve bar with its Try panel
//! (B-E3) and the search palette (B-E14). The sidebar is in nav.rs; the Ask sheet the bar
//! opens is in assistant.rs. Everything drawn comes from widgets.rs; every move from motion.rs.

use super::motion::{self, frame_at, Timing};
use super::theme::*;
use super::widgets::*;
use super::*;
use crate::search::{Hit, HitKind};
use eframe::egui::{self, Rect};

/// The Ask bar and sheet's state.
pub struct AskUi {
    /// What is typed in the bar.
    pub text: String,
    /// The screen the bar last rose on, and when (it rises again on each new screen).
    pub screen: String,
    pub risen_at: f64,
    /// The Try panel: when it opened, and where it was last drawn (clicks inside keep it open).
    pub try_since: Option<f64>,
    pub try_rect: Option<Rect>,
    /// The sheet: open last frame?, when it last opened or closed, focus its box once.
    pub was_open: bool,
    pub moved_at: f64,
    pub focus: bool,
}

impl Default for AskUi {
    fn default() -> Self {
        AskUi { text: String::new(), screen: String::new(), risen_at: 0.0, try_since: None, try_rect: None, was_open: false, moved_at: f64::NEG_INFINITY, focus: false }
    }
}

/// The search palette while it closes (200 e-in): the last query, and when it closed.
pub struct SearchFx {
    pub query: String,
    pub closed_at: Option<f64>,
}

/// The type prefixes the search understands, in the order Tab steps through them.
pub const SEARCH_TYPES: [(&str, &str); 6] = [("", "All"), ("task:", "Tasks"), ("page:", "Pages"), ("project:", "Projects"), ("event:", "Events"), ("db:", "Databases")];

/// Split a query into its type prefix (index into SEARCH_TYPES) and the words after it.
pub fn split_type(query: &str) -> (usize, String) {
    let q = query.trim_start();
    for (i, (pre, _)) in SEARCH_TYPES.iter().enumerate().skip(1) {
        if q.len() >= pre.len() && q[..pre.len()].eq_ignore_ascii_case(pre) {
            return (i, q[pre.len()..].trim_start().to_string());
        }
    }
    (0, q.to_string())
}

/// The same query searching type `i` (0 = everything).
pub fn with_type(query: &str, i: usize) -> String {
    let (_, words) = split_type(query);
    match SEARCH_TYPES[i % SEARCH_TYPES.len()].0 {
        "" => words,
        pre => format!("{pre} {words}").trim_end().to_string() + if words.is_empty() { " " } else { "" },
    }
}

/// Tab: the next type.
pub fn next_type(query: &str) -> String {
    with_type(query, split_type(query).0 + 1)
}

/// Results grouped by kind, groups in the order their best result ranks. Returns indices into `hits`.
pub fn group_hits(hits: &[Hit]) -> Vec<(HitKind, Vec<usize>)> {
    let mut groups: Vec<(HitKind, Vec<usize>)> = vec![];
    for (i, h) in hits.iter().enumerate() {
        match groups.iter_mut().find(|g| g.0 == h.kind) {
            Some(g) => g.1.push(i),
            None => groups.push((h.kind, vec![i])),
        }
    }
    groups
}

fn kind_heading(k: HitKind) -> &'static str {
    match k {
        HitKind::Task => "Tasks",
        HitKind::Page => "Pages",
        HitKind::Project => "Projects",
        HitKind::Event => "Events",
        HitKind::Row => "Databases",
    }
}

/// What the palette was asked to do this frame.
enum SearchPick {
    Hit(usize),
    AddTask,
    Close,
}

impl App {
    /// The Ask Resolve bar, floating in the strip kept clear under the screen (B-E3). Focusing
    /// it opens the Try panel; Enter (or a suggestion) opens the Ask sheet.
    pub(super) fn ask_bar_ui(&mut self, ctx: &egui::Context, p: &Pal, strip: Rect) {
        if self.page_ui.ai_open || self.ask.was_open {
            // the sheet covers it
            return;
        }
        let target = self.nav_target_of(&self.view);
        let copy = self.ask_copy();
        // rises on each screen: 480 e-spring (y 30), at 460 (760 on Tasks)
        let key = format!("{target:?}");
        if self.ask.screen != key {
            self.ask.screen = key;
            self.ask.risen_at = self.now;
        }
        let delay = if target == Some(nav::Target::Tasks) { moves::ASK_BAR_TASKS_DELAY } else { moves::ASK_BAR.delay };
        let f = frame_at(((self.now - self.ask.risen_at) * 1000.0) as f32, Timing { delay, ..moves::ASK_BAR }, motion::reduced());
        if !f.done {
            ctx.request_repaint();
        }
        let w = space::ASK_W.min(strip.width() - space::S24 * 2.0);
        let y = strip.bottom() - space::ASK_BOTTOM - space::ASK_H + moves::ASK_BAR_RISE * (1.0 - f.t);
        let rect = Rect::from_min_size(egui::pos2(strip.center().x - w / 2.0, y), egui::vec2(w, space::ASK_H));
        let id = egui::Id::new("ask-bar");

        let mut out = None;
        egui::Area::new(id.with("area")).order(egui::Order::Foreground).fixed_pos(rect.min).show(ctx, |ui| {
            ui.multiply_opacity(f.alpha.clamp(0.0, 1.0));
            out = Some(ask_bar(ui, p, rect, id, &mut self.ask.text, copy.placeholder));
        });
        let Some(out) = out else { return };
        // the Try panel: open while the bar has focus, or while the pointer is on the panel
        let on_panel = self.ask.try_rect.map_or(false, |r| ctx.input(|i| i.pointer.interact_pos().map_or(false, |pos| r.contains(pos))));
        // --open try (screenshots) shows it without focus: a screenshot window may not hold focus
        let forced = self.temp.contains_key("show_try");
        let show_try = !copy.suggestions.is_empty() && (out.focused || forced || (self.ask.try_since.is_some() && on_panel));
        if show_try && self.ask.try_since.is_none() {
            self.ask.try_since = Some(self.now);
        } else if !show_try {
            self.ask.try_since = None;
            self.ask.try_rect = None;
        }
        let mut asked: Option<String> = None;
        if let Some(t0) = self.ask.try_since {
            let tf = frame_at(((self.now - t0) * 1000.0) as f32, moves::TRY, motion::reduced());
            if !tf.done {
                ctx.request_repaint();
            }
            let origin = egui::pos2(rect.center().x, rect.top() - space::S8);
            let area_id = id.with("try");
            let mut picked = None;
            let r = egui::Area::new(area_id).order(egui::Order::Foreground).fixed_pos(origin).pivot(egui::Align2::CENTER_BOTTOM).show(ctx, |ui| {
                ui.multiply_opacity(tf.alpha.clamp(0.0, 1.0));
                picked = try_panel(ui, p, copy.suggestions);
            });
            self.ask.try_rect = Some(r.response.rect);
            motion::transform_layer(ctx, egui::LayerId::new(egui::Order::Foreground, area_id), origin, moves::TRY_SCALE + (1.0 - moves::TRY_SCALE) * tf.t, egui::Vec2::ZERO);
            if let Some(i) = picked {
                asked = Some(copy.suggestions[i].to_string());
                self.ask.try_since = None;
                self.ask.try_rect = None;
            }
        }
        if out.submit {
            asked = Some(std::mem::take(&mut self.ask.text));
            self.ask.try_since = None;
        }
        if let Some(q) = asked {
            self.page_ui.ai_open = true;
            if !q.trim().is_empty() {
                self.ask_send(&q);
            }
        }
    }

    /// The search palette (B-E14): 680 wide over the current screen, results grouped by type.
    pub(super) fn search_palette(&mut self, ctx: &egui::Context, p: &Pal) {
        let reduced = motion::reduced();
        let screen = ctx.screen_rect();
        let Some(mut st) = self.palette.take() else {
            // closing: 200 e-in, nothing to click
            if let Some(fx) = &mut self.search_fx {
                let t0 = *fx.closed_at.get_or_insert(self.now);
                let f = frame_at(((self.now - t0) * 1000.0) as f32, moves::DIALOG_OUT, reduced);
                if f.done {
                    self.search_fx = None;
                    return;
                }
                ctx.request_repaint();
                let mut ghost = Palette { query: fx.query.clone(), sel: usize::MAX, focus: false, opened: 0.0 };
                let fade = 1.0 - f.alpha;
                egui::Area::new(egui::Id::new("search-dim")).order(egui::Order::Foreground).fixed_pos(screen.min).interactable(false).show(ctx, |ui| {
                    backdrop(ui, screen, shadow::BACKDROP_DIALOG * fade);
                });
                self.search_dialog(ctx, p, &mut ghost, fade, false);
            }
            return;
        };
        if st.opened <= 0.0 {
            st.opened = self.now;
        }
        let el = ((self.now - st.opened) * 1000.0) as f32;
        let dim = frame_at(el, moves::DIALOG_DIM, reduced);
        if !dim.done {
            ctx.request_repaint();
        }
        let mut close = false;
        egui::Area::new(egui::Id::new("search-dim")).order(egui::Order::Foreground).fixed_pos(screen.min).show(ctx, |ui| {
            close = backdrop(ui, screen, shadow::BACKDROP_DIALOG * dim.alpha.clamp(0.0, 1.0));
        });
        let pick = self.search_dialog(ctx, p, &mut st, 1.0, true);
        self.search_fx = Some(SearchFx { query: st.query.clone(), closed_at: None });
        match pick {
            Some(SearchPick::Hit(i)) => {
                let hits = crate::search::search(&self.store.data, &st.query, 30);
                let order: Vec<usize> = group_hits(&hits).into_iter().flat_map(|g| g.1).collect();
                if let Some(h) = order.get(i).and_then(|&k| hits.get(k)) {
                    self.open_hit(h);
                }
            }
            Some(SearchPick::AddTask) => {
                let (_, words) = split_type(&st.query);
                self.add_task_text(&words, None, "", None, None, None);
                self.toast(format!("Task added: {}", words.trim()));
            }
            Some(SearchPick::Close) => {}
            None if !close => self.palette = Some(st),
            None => {}
        }
    }

    /// The palette dialog itself. `alpha` fades it; `live` is false while it closes.
    fn search_dialog(&mut self, ctx: &egui::Context, p: &Pal, st: &mut Palette, alpha: f32, live: bool) -> Option<SearchPick> {
        let screen = ctx.screen_rect();
        let reduced = motion::reduced();
        let failed = self.view_state == ViewState::Error;
        let hits = if failed { vec![] } else { crate::search::search(&self.store.data, &st.query, 30) };
        let groups = group_hits(&hits);
        let order: Vec<usize> = groups.iter().flat_map(|g| g.1.clone()).collect();
        let (ty, words) = split_type(&st.query);
        let none = !failed && !words.trim().is_empty() && hits.is_empty();
        let n_items = if none { 1 } else { order.len() };
        let mut pick = None;
        if live {
            let (down, up, tab, enter) = ctx.input_mut(|i| (i.consume_key(Modifiers::NONE, Key::ArrowDown), i.consume_key(Modifiers::NONE, Key::ArrowUp), i.consume_key(Modifiers::NONE, Key::Tab), i.key_pressed(Key::Enter)));
            if down {
                st.sel = (st.sel + 1).min(n_items.saturating_sub(1));
            }
            if up {
                st.sel = st.sel.saturating_sub(1);
            }
            if tab {
                st.query = next_type(&st.query);
                st.sel = 0;
                st.focus = true;
            }
            if enter && !failed {
                if none {
                    pick = Some(SearchPick::AddTask);
                } else if n_items > 0 {
                    pick = Some(SearchPick::Hit(st.sel.min(n_items - 1)));
                }
            }
        }
        let f = frame_at(((self.now - st.opened) * 1000.0) as f32, moves::DIALOG_IN, reduced);
        if live && !f.done {
            ctx.request_repaint();
        }
        let w = space::SEARCH_W.min(screen.width() - space::S24 * 2.0);
        let origin = egui::pos2(screen.center().x, screen.top() + space::DIALOG_TOP);
        let area_id = egui::Id::new("search-dialog");
        egui::Area::new(area_id).order(egui::Order::Foreground).fixed_pos(origin).pivot(egui::Align2::CENTER_TOP).interactable(live).show(ctx, |ui| {
            ui.multiply_opacity((alpha * if live { f.alpha } else { 1.0 }).clamp(0.0, 1.0));
            dialog_frame(p).show(ui, |ui| {
                ui.set_width(w);
                ui.spacing_mut().item_spacing.y = space::S4;
                // the input row
                egui::Frame::none().inner_margin(egui::Margin { left: space::S18, right: space::S16, top: space::S14, bottom: space::S10 }).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        icon_widget(ui, Icon::Search, p.dim);
                        ui.add_space(space::S10);
                        let te_w = ui.available_width() - space::S40;
                        let r = ui.add(egui::TextEdit::singleline(&mut st.query).id(egui::Id::new("search-input")).hint_text(RichText::new("Search everything").color(p.faint)).font(fp(size::ANSWER)).frame(false).desired_width(te_w));
                        if live && st.focus {
                            r.request_focus();
                            st.focus = false;
                        }
                        if r.changed() {
                            st.sel = 0;
                        }
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if kbd(ui, p, "Esc").interact(Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                pick = Some(SearchPick::Close);
                            }
                        });
                    });
                    ui.add_space(space::S10);
                    // type chips: Tab steps through them
                    ui.horizontal_wrapped(|ui| {
                        for (i, (_, name)) in SEARCH_TYPES.iter().enumerate() {
                            if filter_chip(ui, p, name, None, ty == i).clicked() {
                                st.query = with_type(&st.query, i);
                                st.sel = 0;
                                st.focus = true;
                            }
                        }
                    });
                });
                hline(ui, p);
                // results (they swap with no motion)
                egui::Frame::none().inner_margin(egui::Margin::symmetric(space::S10, space::S8)).show(ui, |ui| {
                    egui::ScrollArea::vertical().max_height(space::SEARCH_LIST_H).auto_shrink([false, true]).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        let pad = |ui: &mut egui::Ui, add: &mut dyn FnMut(&mut egui::Ui)| {
                            egui::Frame::none().inner_margin(egui::Margin::symmetric(space::S8, space::S10)).show(ui, |ui| add(ui));
                        };
                        if failed {
                            pad(ui, &mut |ui| {
                                wrap_label(ui, RichText::new("Search couldn't finish. Your data is fine. Press Enter to try again.").size(size::BODY).color(p.orange));
                            });
                        } else if words.trim().is_empty() {
                            pad(ui, &mut |ui| {
                                wrap_label(ui, RichText::new("Type to search everything in this workspace.").size(size::BODY).color(p.dim));
                                list_heading(ui, p, "Narrow it down");
                                ui.horizontal_wrapped(|ui| {
                                    for (i, (pre, _)) in SEARCH_TYPES.iter().enumerate().skip(1) {
                                        if filter_chip(ui, p, pre, None, false).clicked() {
                                            st.query = with_type(&st.query, i);
                                            st.focus = true;
                                        }
                                    }
                                });
                            });
                        } else if none {
                            pad(ui, &mut |ui| {
                                wrap_label(ui, RichText::new(format!("Nothing matches \"{}\". Try fewer words, or search one type with task: or page:.", words.trim())).size(size::BODY).color(p.dim));
                            });
                            if search_row(ui, p, &format!("Add \"{}\" as a task", words.trim()), "Task · goes to Captured", st.sel == 0).clicked() {
                                pick = Some(SearchPick::AddTask);
                            }
                        } else {
                            let mut n = 0;
                            for (kind, idx) in &groups {
                                list_heading(ui, p, kind_heading(*kind));
                                for &k in idx {
                                    let h = &hits[k];
                                    let meta = if h.snippet.trim().is_empty() { h.kind.label().to_string() } else { format!("{} · {}", h.kind.label(), h.snippet.trim()) };
                                    let r = search_row(ui, p, &h.title, &meta, st.sel == n);
                                    if live && st.sel == n && (ctx.input(|i| i.key_pressed(Key::ArrowDown) || i.key_pressed(Key::ArrowUp))) {
                                        r.scroll_to_me(None);
                                    }
                                    if r.clicked() {
                                        pick = Some(SearchPick::Hit(n));
                                    }
                                    n += 1;
                                }
                            }
                        }
                    });
                });
                // footer: the keys
                egui::Frame::none().fill(p.surface_sunk).rounding(egui::Rounding { nw: 0.0, ne: 0.0, sw: radius::DIALOG, se: radius::DIALOG }).inner_margin(egui::Margin::symmetric(space::S18, space::S10)).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        for (k, what) in [("↑ ↓", "move"), ("Enter", "open"), ("Tab", "next type"), ("Ctrl K", "from anywhere")] {
                            kbd(ui, p, k);
                            ui.label(RichText::new(what).size(size::TINY).color(p.dim));
                            ui.add_space(space::S10);
                        }
                    });
                });
            });
        });
        if live {
            let (off, s, _) = motion::entrance(f, egui::vec2(0.0, -moves::DIALOG_RISE), moves::DIALOG_SCALE);
            motion::transform_layer(ctx, egui::LayerId::new(egui::Order::Foreground, area_id), origin, s, off);
        }
        pick
    }
}
