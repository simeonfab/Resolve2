use super::theme::*;
use crate::model::*;
use eframe::egui::{self, Align, Align2, Color32, Layout, Pos2, Rect, Response, RichText, Rounding, Sense, Shape, Stroke, Ui, Vec2};

pub fn card<R>(ui: &mut Ui, p: &Pal, add: impl FnOnce(&mut Ui) -> R) -> R {
    egui::Frame::none()
        .fill(p.card)
        .stroke(Stroke::new(1.0, p.line))
        .rounding(Rounding::same(10.0))
        .inner_margin(egui::Margin::same(16.0))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            // a plain top-down layout: text wraps inside the card instead of being stretched
            ui.vertical(|ui| add(ui)).inner
        })
        .inner
}

/// A label that always wraps within the space it's given (never pushes past its column).
pub fn wrap_label(ui: &mut Ui, text: RichText) -> Response {
    ui.add(egui::Label::new(text).wrap())
}

/// A clickable wrapped label.
pub fn wrap_link(ui: &mut Ui, text: RichText) -> Response {
    ui.add(egui::Label::new(text).wrap().sense(Sense::click())).on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// The redesigned screens' column: the full width inside the work area's padding, capped at
/// `cap` and centred (Rules, item 6).
pub fn capped<R>(ui: &mut Ui, cap: f32, add: impl FnOnce(&mut Ui) -> R) -> R {
    let avail = ui.available_width();
    let w = cap.min(avail);
    ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
        ui.add_space(((avail - w) / 2.0).max(0.0));
        ui.vertical(|ui| {
            ui.set_width(w);
            ui.set_max_width(w);
            add(ui)
        })
        .inner
    })
    .inner
}

/// Lay out content in a centred column no wider than `max_w`.
pub fn centered<R>(ui: &mut Ui, max_w: f32, add: impl FnOnce(&mut Ui) -> R) -> R {
    let avail = ui.available_width();
    let w = max_w.min(avail - 48.0).max(240.0);
    let pad = ((avail - w) / 2.0).max(0.0);
    ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
        ui.add_space(pad);
        ui.vertical(|ui| {
            ui.set_width(w);
            ui.set_max_width(w);
            add(ui)
        })
        .inner
    })
    .inner
}

pub fn nav_item(ui: &mut Ui, p: &Pal, icon: &str, label: &str, count: Option<usize>, selected: bool, icon_color: Option<Color32>) -> Response {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 32.0), Sense::click());
    let hovered = resp.hovered();
    if selected {
        ui.painter().rect_filled(rect, 6.0, p.accent_soft);
    } else if hovered {
        ui.painter().rect_filled(rect, 6.0, p.hover);
    }
    let fg = if selected { p.accent } else { p.text };
    ui.painter().text(rect.left_center() + egui::vec2(18.0, 0.0), Align2::CENTER_CENTER, icon, fp(15.0), if selected { p.accent } else { icon_color.unwrap_or(p.dim) });
    let label_rect_w = rect.width() - 70.0;
    let galley = ui.painter().layout(label.to_string(), fp(14.0), fg, f32::INFINITY);
    let mut text = label.to_string();
    if galley.size().x > label_rect_w {
        let mut t: String = label.chars().collect();
        while !t.is_empty() && ui.painter().layout_no_wrap(format!("{t}…"), fp(14.0), fg).size().x > label_rect_w {
            t.pop();
        }
        text = format!("{t}…");
    }
    ui.painter().text(rect.left_center() + egui::vec2(36.0, 0.0), Align2::LEFT_CENTER, text, fp(14.0), fg);
    if let Some(c) = count {
        if c > 0 {
            ui.painter().text(rect.right_center() - egui::vec2(12.0, 0.0), Align2::RIGHT_CENTER, c.to_string(), fp(12.0), if selected { p.accent } else { p.dim });
        }
    }
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

pub fn section_label(ui: &mut Ui, p: &Pal, text: &str) -> Response {
    ui.add_space(10.0);
    let r = ui.add(egui::Label::new(RichText::new(text).font(fs(13.0)).color(p.dim)).sense(Sense::click()));
    ui.add_space(2.0);
    r
}

/// Draw Todoist's priority circle. `hover` shows the check mark preview.
pub fn paint_circle(ui: &Ui, center: Pos2, p: &Pal, prio: u8, done: bool, hover: bool) {
    let c = prio_color(prio, p);
    let r = 9.0;
    let painter = ui.painter();
    if done {
        painter.circle_filled(center, r, c);
        tick(painter, center, Color32::WHITE);
    } else {
        if prio < 4 {
            painter.circle_filled(center, r, c.linear_multiply(0.18));
        }
        painter.circle_stroke(center, r, Stroke::new(if prio < 4 { 2.0 } else { 1.3 }, c));
        if hover {
            tick(painter, center, c);
        }
    }
}

fn tick(painter: &egui::Painter, c: Pos2, col: Color32) {
    painter.line_segment([c + egui::vec2(-4.0, 0.0), c + egui::vec2(-1.0, 3.0)], Stroke::new(1.6, col));
    painter.line_segment([c + egui::vec2(-1.0, 3.0), c + egui::vec2(4.5, -3.0)], Stroke::new(1.6, col));
}

pub fn due_style(t: &Task, p: &Pal) -> Option<(String, Color32)> {
    let d = t.due?;
    let mut s = fmt_date_short(d);
    if let Some(tm) = t.due_time {
        s.push(' ');
        s.push_str(&fmt_time(tm));
    }
    let diff = (d - today()).num_days();
    let col = if t.is_overdue() {
        p.accent
    } else if diff == 0 {
        p.green
    } else if diff == 1 {
        p.orange
    } else if diff < 7 {
        p.purple
    } else {
        p.dim
    };
    Some((s, col))
}

pub struct RowOut {
    pub toggled: bool,
    pub opened: bool,
}

pub fn task_row(ui: &mut Ui, p: &Pal, d: &WorkspaceData, t: &Task, show_project: bool, indent: f32) -> RowOut {
    let mut out = RowOut { toggled: false, opened: false };
    let id = ui.make_persistent_id(("task-row", &t.id));
    let bg = ui.painter().add(Shape::Noop);
    let top = ui.cursor().min;
    let width = ui.available_width();
    let mut circle_center = Pos2::ZERO;
    ui.add_space(8.0);
    ui.horizontal_top(|ui| {
        ui.add_space(indent);
        let (crect, _) = ui.allocate_exact_size(egui::vec2(22.0, 20.0), Sense::hover());
        circle_center = crect.center();
        ui.vertical(|ui| {
            ui.set_width(width - indent - 40.0);
            let title_col = if t.completed { p.dim } else { p.text };
            let mut rt = RichText::new(&t.title).font(fp(14.0)).color(title_col);
            if t.completed {
                rt = rt.strikethrough();
            }
            ui.add(egui::Label::new(rt).wrap());
            if !t.notes.is_empty() {
                let first = t.notes.lines().next().unwrap_or("");
                ui.label(RichText::new(first).size(12.0).color(p.dim));
            }
            let subs = d.subtasks(&t.id);
            let has_meta = t.due.is_some() || t.page_id.is_some() || !t.labels.is_empty() || !subs.is_empty() || show_project || t.reminder_min.is_some();
            if has_meta {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 10.0;
                    if let Some((s, col)) = due_style(t, p) {
                        let rec = if t.recurrence.is_some() { " ↻" } else { "" };
                        ui.label(RichText::new(format!("🗓 {s}{rec}")).size(12.0).color(col));
                    }
                    if t.reminder_min.is_some() && t.due_time.is_some() {
                        ui.label(RichText::new("⏰").size(11.0).color(p.dim));
                    }
                    if !subs.is_empty() {
                        let done = subs.iter().filter(|s| s.completed).count();
                        ui.label(RichText::new(format!("⑂ {done}/{}", subs.len())).size(12.0).color(p.dim));
                    }
                    if let Some(pg) = t.page_id.as_deref().and_then(|x| d.page(x)) {
                        ui.label(RichText::new(format!("📄 {}", pg.title)).size(12.0).color(p.blue));
                    }
                    for l in &t.labels {
                        ui.label(RichText::new(format!("@{l}")).size(12.0).color(p.orange));
                    }
                    if show_project {
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let pc = rgb(d.project_color(&t.project_id));
                            ui.label(RichText::new(format!("{} #", d.project_name(&t.project_id))).size(12.0).color(p.dim));
                            let _ = pc;
                        });
                    }
                });
            }
        });
    });
    ui.add_space(8.0);
    let bottom = ui.cursor().min.y;
    let rect = Rect::from_min_max(Pos2::new(top.x, top.y), Pos2::new(top.x + width, bottom));
    let resp = ui.interact(rect, id, Sense::click());
    let pointer = ui.ctx().pointer_hover_pos();
    let over_circle = pointer.map_or(false, |pp| pp.distance(circle_center) < 12.0);
    if resp.hovered() {
        ui.painter().set(bg, Shape::rect_filled(rect.expand2(egui::vec2(6.0, 0.0)), 6.0, p.hover.linear_multiply(0.5)));
    }
    paint_circle(ui, circle_center, p, t.priority, t.completed, resp.hovered() && over_circle);
    ui.painter().line_segment([Pos2::new(rect.left() + indent, rect.bottom()), Pos2::new(rect.right(), rect.bottom())], Stroke::new(1.0, p.line));
    if resp.clicked() {
        if over_circle {
            out.toggled = true;
        } else {
            out.opened = true;
        }
    }
    resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    out
}

pub fn add_row_button(ui: &mut Ui, p: &Pal, label: &str) -> Response {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width().min(400.0), 32.0), Sense::click());
    let col = if resp.hovered() { p.accent } else { p.dim };
    if resp.hovered() {
        ui.painter().circle_filled(rect.left_center() + egui::vec2(9.0, 0.0), 9.0, p.accent);
        ui.painter().text(rect.left_center() + egui::vec2(9.0, 0.0), Align2::CENTER_CENTER, "+", fb(15.0), p.on_accent);
    } else {
        ui.painter().text(rect.left_center() + egui::vec2(9.0, 0.0), Align2::CENTER_CENTER, "+", fp(18.0), p.accent);
    }
    ui.painter().text(rect.left_center() + egui::vec2(28.0, 0.0), Align2::LEFT_CENTER, label, fp(14.0), col);
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

pub fn pill(ui: &mut Ui, p: &Pal, text: &str, selected: bool) -> Response {
    let fill = if selected { p.accent } else { Color32::TRANSPARENT };
    let fg = if selected { p.on_accent } else { p.text };
    ui.add(egui::Button::new(RichText::new(text).color(fg).size(13.0)).fill(fill).stroke(Stroke::new(1.0, if selected { p.accent } else { p.line })).rounding(Rounding::same(14.0)))
}

pub fn ghost_button(ui: &mut Ui, text: impl Into<egui::WidgetText>) -> Response {
    ui.add(egui::Button::new(text).frame(false))
}

pub fn primary_button(ui: &mut Ui, p: &Pal, text: &str) -> Response {
    ui.add(egui::Button::new(RichText::new(text).color(p.on_accent).font(fs(14.0))).fill(p.accent).rounding(Rounding::same(6.0)).min_size(Vec2::new(0.0, 30.0)))
}

pub fn chip(ui: &mut Ui, text: &str, col: Color32) -> Response {
    let galley = ui.painter().layout_no_wrap(text.to_string(), fp(12.0), col);
    let size = galley.size() + egui::vec2(14.0, 6.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    ui.painter().rect(rect, 10.0, col.linear_multiply(0.14), Stroke::new(1.0, col.linear_multiply(0.5)));
    ui.painter().galley(rect.min + egui::vec2(7.0, 3.0), galley, col);
    resp
}

pub fn avatar(ui: &mut Ui, letter: &str, color: Color32, size: f32) -> Response {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(size, size), Sense::click());
    ui.painter().circle_filled(rect.center(), size / 2.0, color);
    ui.painter().text(rect.center(), Align2::CENTER_CENTER, letter, fb(size * 0.5), Color32::WHITE);
    resp
}

/// Start Windows' built-in voice typing (the same as pressing Windows+H). Whatever you say is
/// typed into the text box that has focus. Free, no setup; Windows asks once to allow the microphone.
pub fn start_voice_typing() {
    let script = r#"Add-Type -Namespace W -Name K -MemberDefinition '[DllImport("user32.dll")] public static extern void keybd_event(byte b, byte s, uint f, System.UIntPtr e);'; Start-Sleep -Milliseconds 250; [W.K]::keybd_event(0x5B,0,0,[UIntPtr]::Zero); [W.K]::keybd_event(0x48,0,0,[UIntPtr]::Zero); [W.K]::keybd_event(0x48,0,2,[UIntPtr]::Zero); [W.K]::keybd_event(0x5B,0,2,[UIntPtr]::Zero)"#;
    let _ = std::process::Command::new("powershell").args(["-NoProfile", "-NonInteractive", "-WindowStyle", "Hidden", "-Command", script]).spawn();
}

/// The mic for one text field (E2, voice by Windows voice typing as Simeon chose on 7 Nov 2026).
/// Click: the field gets focus and Windows voice typing (Win+H) opens, so speech is typed into it.
/// The button shows Recording from then until the field loses focus or the mic is clicked again
/// (which presses Win+H again, closing voice typing). Windows does not tell apps whether its
/// voice typing panel is open, so this state follows the click and the focus, not the panel.
pub fn mic_for(ui: &mut Ui, p: &Pal, field: egui::Id, on_dark: bool) -> Response {
    let key = field.with("mic-on");
    let now = ui.input(|i| i.time);
    let mut since: Option<f64> = ui.data(|d| d.get_temp(key)).flatten();
    // the click itself takes focus from the field for a frame, so allow a moment before reading focus
    if let Some(t0) = since {
        if now - t0 > 0.6 && !ui.memory(|m| m.has_focus(field)) {
            since = None;
        }
    }
    let state = if since.is_some() { MicState::Recording } else { MicState::Idle };
    let r = mic(ui, p, state, on_dark);
    if r.clicked() {
        start_voice_typing();
        since = if since.is_some() { None } else { Some(now) };
        if since.is_some() {
            ui.memory_mut(|m| m.request_focus(field));
        }
    }
    if since.is_some() {
        ui.ctx().request_repaint_after(std::time::Duration::from_millis(250));
    }
    ui.data_mut(|d| d.insert_temp(key, since));
    r
}

/// HH:MM text field. Returns true when the time changed.
pub fn time_field(ui: &mut Ui, id: egui::Id, value: &mut Option<chrono::NaiveTime>, width: f32) -> bool {
    let mut text: String = ui.data_mut(|d| d.get_temp::<String>(id)).unwrap_or_else(|| value.map(fmt_time).unwrap_or_default());
    let r = ui.add(egui::TextEdit::singleline(&mut text).desired_width(width).hint_text("HH:MM"));
    let mut changed = false;
    if r.changed() {
        ui.data_mut(|d| d.insert_temp(id, text.clone()));
    }
    if r.lost_focus() {
        let parsed = if text.trim().is_empty() { None } else { chrono::NaiveTime::parse_from_str(text.trim(), "%H:%M").ok().or_else(|| crate::nlp::parse_time(&text)) };
        if parsed != *value && (parsed.is_some() || text.trim().is_empty()) {
            *value = parsed;
            changed = true;
        }
        ui.data_mut(|d| d.remove::<String>(id));
    }
    changed
}

// ======================================================================================
// Redesign components (Phase 0). Every later screen builds from these; all colours and sizes
// come from theme.rs, all movement from motion.rs.
// ======================================================================================

use super::motion::{self, timing, E_OUT, E_SPRING, E_STD};
use egui::epaint::{Mesh, Shadow};

/// Paint layered soft shadows behind a rect (card-lift, dialog, sheet, side panel, Ask bar).
pub fn paint_shadows(painter: &egui::Painter, rect: Rect, rounding: f32, shadows: &[Shadow]) {
    for s in shadows {
        painter.add(s.as_shape(rect, Rounding::same(rounding)));
    }
}

/// A radial glow as a gradient mesh (tickets E17, B-E20): `color` at the centre fading to
/// nothing at `radius`. egui has no CSS gradients or blur; a mesh with per-vertex colour gives
/// the same soft light. `strength` is the centre opacity (0..1).
pub fn glow(painter: &egui::Painter, center: Pos2, radius: f32, color: Color32, strength: f32) {
    if radius <= 0.5 || strength <= 0.0 {
        return;
    }
    const RINGS: usize = 14;
    const SEGS: usize = 64;
    let mut mesh = Mesh::default();
    let at = |k: f32| -> Color32 {
        // smooth falloff (gaussian-like), premultiplied by the colour's own alpha
        let a = strength * (-(k * k) * 3.2).exp() * (1.0 - k).max(0.0).powf(0.6);
        color.gamma_multiply(a.clamp(0.0, 1.0))
    };
    mesh.colored_vertex(center, at(0.0));
    for r in 1..=RINGS {
        let k = r as f32 / RINGS as f32;
        let c = at(k);
        for s in 0..SEGS {
            let ang = s as f32 / SEGS as f32 * std::f32::consts::TAU;
            mesh.colored_vertex(center + egui::vec2(ang.cos(), ang.sin()) * radius * k, c);
        }
    }
    let ring = |r: usize, s: usize| -> u32 { (1 + (r - 1) * SEGS + (s % SEGS)) as u32 };
    for s in 0..SEGS {
        mesh.add_triangle(0, ring(1, s), ring(1, s + 1));
    }
    for r in 1..RINGS {
        for s in 0..SEGS {
            let (a, b, c, d) = (ring(r, s), ring(r, s + 1), ring(r + 1, s), ring(r + 1, s + 1));
            mesh.add_triangle(a, b, c);
            mesh.add_triangle(b, d, c);
        }
    }
    painter.add(egui::Shape::mesh(mesh));
}

/// A radial glow drawn inside a rounded rect only (the hero card's light): a grid mesh over the
/// rect whose corner points are pulled onto the rounded edge, coloured by the same falloff.
pub fn glow_in_rect(rect: Rect, rounding: f32, center: Pos2, radius: f32, color: Color32, strength: f32) -> Shape {
    const NX: usize = 36;
    const NY: usize = 24;
    let rr = rounding.min(rect.width() / 2.0).min(rect.height() / 2.0);
    let inside = |q: Pos2| -> Pos2 {
        // pull points in the corner squares onto the corner arc
        let cx = q.x.clamp(rect.left() + rr, rect.right() - rr);
        let cy = q.y.clamp(rect.top() + rr, rect.bottom() - rr);
        let d = q - egui::pos2(cx, cy);
        if d.length() > rr {
            egui::pos2(cx, cy) + d.normalized() * rr
        } else {
            q
        }
    };
    let mut mesh = Mesh::default();
    for j in 0..=NY {
        for i in 0..=NX {
            let q = egui::pos2(rect.left() + rect.width() * i as f32 / NX as f32, rect.top() + rect.height() * j as f32 / NY as f32);
            let q = inside(q);
            let k = ((q - center).length() / radius.max(1.0)).min(1.0);
            let a = strength * (-(k * k) * 3.2).exp() * (1.0 - k).max(0.0).powf(0.6);
            mesh.colored_vertex(q, color.gamma_multiply(a.clamp(0.0, 1.0)));
        }
    }
    let at = |i: usize, j: usize| (j * (NX + 1) + i) as u32;
    for j in 0..NY {
        for i in 0..NX {
            mesh.add_triangle(at(i, j), at(i + 1, j), at(i, j + 1));
            mesh.add_triangle(at(i + 1, j), at(i + 1, j + 1), at(i, j + 1));
        }
    }
    Shape::mesh(mesh)
}

/// The reveal's amber bloom: a glow whose size follows `scale` (the brief's .4 → 6 → 11).
pub fn bloom(painter: &egui::Painter, center: Pos2, base_radius: f32, scale: f32, strength: f32) {
    glow(painter, center, base_radius * scale, tok::AMBER, strength);
}

/// Card on light: white, border, radius 13, card-lift shadow, 16 padding.
pub fn card_lift<R>(ui: &mut Ui, p: &Pal, add: impl FnOnce(&mut Ui) -> R) -> R {
    let shadow_at = ui.painter().add(Shape::Noop);
    let inner = egui::Frame::none().fill(p.card).stroke(Stroke::new(1.0, p.line)).rounding(Rounding::same(radius::CARD)).inner_margin(egui::Margin::same(space::S16)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.vertical(|ui| add(ui)).inner
    });
    if !p.dark {
        let shapes: Vec<Shape> = shadow::CARD_LIFT.iter().map(|s| Shape::Rect(s.as_shape(inner.response.rect, Rounding::same(radius::CARD)))).collect();
        ui.painter().set(shadow_at, Shape::Vec(shapes));
    }
    inner.inner
}

/// The dark hero card (one per screen): ink, radius 16, an amber glow in the corner that
/// breathes on a 3200 ms loop (holds still with reduced motion).
pub fn hero_card<R>(ui: &mut Ui, p: &Pal, add: impl FnOnce(&mut Ui) -> R) -> R {
    let mut glow_at = None;
    let edge = if p.dark { Stroke::new(stroke::HAIR, tok::white_a(0.08)) } else { Stroke::NONE };
    let inner = egui::Frame::none().fill(tok::INK).stroke(edge).rounding(Rounding::same(radius::HERO)).inner_margin(egui::Margin { left: space::HERO_PAD_X, right: space::HERO_PAD_X, top: space::HERO_PAD_Y, bottom: space::HERO_PAD_Y + 2.0 }).show(ui, |ui| {
        // reserved inside the frame, so the glow sits above the fill and below the content
        glow_at = Some(ui.painter().add(Shape::Noop));
        ui.set_width(ui.available_width());
        ui.visuals_mut().override_text_color = Some(tok::TEXT_ON_DARK);
        ui.vertical(|ui| add(ui)).inner
    });
    let rect = inner.response.rect;
    let phase = motion::loop_phase(ui.ctx(), 3200.0);
    let breathe = 0.85 + 0.15 * (0.5 - 0.5 * (phase * std::f32::consts::TAU).cos());
    let _ = p;
    if let Some(at) = glow_at {
        ui.painter().set(at, glow_in_rect(rect, radius::HERO, rect.right_top() + egui::vec2(-48.0, 36.0), rect.height().max(160.0) * 1.2, tok::AMBER, 0.32 * breathe));
    }
    inner.inner
}

/// Button kinds from the canvases.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ButtonKind {
    /// Amber fill, ink text (the CTA).
    Primary,
    /// Ink fill, text-on-dark (primary dark buttons).
    Dark,
    /// White fill, border, ink text.
    Secondary,
    /// No fill: a text action in terracotta (links) or ink.
    Ghost,
    /// Terracotta text on terracotta-tint.
    Danger,
    /// On the ink hero: no fill, a faint light edge, text-on-dark ("Plan my day").
    OnDark,
}

pub const BUTTON_KINDS: [(ButtonKind, &str); 5] = [(ButtonKind::Primary, "Primary"), (ButtonKind::Dark, "Dark"), (ButtonKind::Secondary, "Secondary"), (ButtonKind::Ghost, "Ghost"), (ButtonKind::Danger, "Danger")];

fn button_colours(p: &Pal, kind: ButtonKind, enabled: bool, hovered: bool) -> (Color32, Color32, Stroke) {
    if !enabled {
        return (if kind == ButtonKind::Ghost { Color32::TRANSPARENT } else { p.track }, p.faint, Stroke::NONE);
    }
    match kind {
        ButtonKind::Primary => (if hovered { p.accent.gamma_multiply(0.92) } else { p.accent }, p.on_accent, Stroke::NONE),
        ButtonKind::Dark => (if hovered { tok::NAV_BG } else { tok::INK }, tok::TEXT_ON_DARK, Stroke::NONE),
        ButtonKind::Secondary => (if hovered { p.surface_raised } else { p.card }, p.text, Stroke::new(1.0, p.line)),
        ButtonKind::Ghost => (if hovered { p.surface_raised } else { Color32::TRANSPARENT }, p.orange, Stroke::NONE),
        ButtonKind::Danger => (p.orange_tint, p.orange, Stroke::NONE),
        ButtonKind::OnDark => (if hovered { tok::NAV_BG } else { Color32::TRANSPARENT }, tok::TEXT_ON_DARK, Stroke::new(stroke::HAIR, tok::white_a(0.2))),
    }
}

/// A button: Inter 600, radius 10 (8 small), height 36 (28 small). Presses scale to .975 in
/// 240 ms e-std about the centre; disabled uses the track colour.
pub fn button(ui: &mut Ui, p: &Pal, kind: ButtonKind, text: &str, small: bool, enabled: bool) -> Response {
    let font = fs(if small { size::SMALL } else { size::BUTTON });
    let galley = ui.painter().layout_no_wrap(text.to_string(), font.clone(), p.text);
    let pad = if small { 10.0 } else { 16.0 };
    let h = if small { space::BUTTON_H_SMALL } else { space::BUTTON_H };
    let size = egui::vec2(galley.size().x + pad * 2.0, h);
    let (rect, resp) = ui.allocate_exact_size(size, if enabled { Sense::click() } else { Sense::hover() });
    let id = resp.id;
    let down = enabled && resp.is_pointer_button_down_on();
    let s = motion::tween(ui.ctx(), id.with("press"), if down { 0.975 } else { 1.0 }, timing(240.0, 0.0, E_STD));
    let r = motion::scale_about(rect, rect.center(), s);
    let (fill, fg, stroke) = button_colours(p, kind, enabled, resp.hovered());
    let rad = if small { radius::SMALL_BUTTON } else { radius::BUTTON };
    ui.painter().rect(r, rad, fill, stroke);
    if resp.has_focus() {
        focus_ring(ui.painter(), r, rad, 1.0);
    }
    ui.painter().text(r.center(), Align2::CENTER_CENTER, text, font, fg);
    if enabled {
        resp.on_hover_cursor(egui::CursorIcon::PointingHand)
    } else {
        resp
    }
}

/// The focus ring: 3 px amber at .22 outside the edge, plus an amber border. `t` fades it in.
pub fn focus_ring(painter: &egui::Painter, rect: Rect, rounding: f32, t: f32) {
    let t = t.clamp(0.0, 1.0);
    if t <= 0.0 {
        return;
    }
    painter.rect_stroke(rect.expand(space::FOCUS_RING / 2.0 + 0.5), rounding + space::FOCUS_RING / 2.0, Stroke::new(space::FOCUS_RING * t, tok::amber_a(shadow::FOCUS_ALPHA * t)));
    painter.rect_stroke(rect, rounding, Stroke::new(1.0, tok::AMBER.gamma_multiply(t)));
}

/// A text field: card fill, border, radius 10, Inter 15 answer text; on focus the border turns
/// amber and the ring grows in 160 ms e-std (M01.2).
pub fn text_field(ui: &mut Ui, p: &Pal, id: egui::Id, text: &mut String, hint: &str, multiline: bool, width: f32) -> Response {
    let rows = if multiline { 3 } else { 1 };
    let frame_rect_at = ui.painter().add(Shape::Noop);
    let ring_at = ui.painter().add(Shape::Noop);
    let inner = egui::Frame::none().inner_margin(egui::Margin::symmetric(12.0, 9.0)).show(ui, |ui| {
        let te = if multiline { egui::TextEdit::multiline(text) } else { egui::TextEdit::singleline(text) };
        ui.add(te.id(id).frame(false).hint_text(RichText::new(hint).color(p.faint)).font(fp(size::ANSWER)).desired_rows(rows).desired_width(width - 24.0).text_color(p.text))
    });
    let resp = inner.inner;
    let rect = inner.response.rect;
    let f = motion::tween(ui.ctx(), id.with("focus"), if resp.has_focus() { 1.0 } else { 0.0 }, timing(160.0, 0.0, E_STD));
    ui.painter().set(frame_rect_at, Shape::rect_filled(rect, radius::FIELD, p.field));
    let ring = {
        let mut v = vec![Shape::rect_stroke(rect, radius::FIELD, Stroke::new(1.0, p.line))];
        if f > 0.0 {
            v.push(Shape::rect_stroke(rect.expand(space::FOCUS_RING / 2.0 + 0.5), radius::FIELD + space::FOCUS_RING / 2.0, Stroke::new(space::FOCUS_RING * f, tok::amber_a(shadow::FOCUS_ALPHA * f))));
            v.push(Shape::rect_stroke(rect, radius::FIELD, Stroke::new(1.0, tok::AMBER.gamma_multiply(f))));
        }
        Shape::Vec(v)
    };
    ui.painter().set(ring_at, ring);
    resp
}

/// A tag or pill (radius 999): text on a fill, Inter 600 11.
pub fn tag(ui: &mut Ui, text: &str, fg: Color32, bg: Color32) -> Response {
    let galley = ui.painter().layout_no_wrap(text.to_string(), fs(size::TAG), fg);
    let size = galley.size() + egui::vec2(16.0, 6.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    ui.painter().rect_filled(rect, radius::PILL, bg);
    ui.painter().galley(rect.min + egui::vec2(8.0, 3.0), galley, fg);
    resp
}

/// The status pills (Unresolved, Supported, Weakened, Refuted, Planned).
pub fn status_pill(ui: &mut Ui, status: &str) -> Response {
    let (fg, bg) = status_pair(status);
    tag(ui, status, fg, bg)
}

/// A Knowledge base card-kind chip.
pub fn kind_tag(ui: &mut Ui, kind: &str) -> Response {
    let (fg, bg) = kind_pair(kind);
    tag(ui, kind, fg, bg)
}

/// A roadmap gate chip: "◇ H1 result" (ink-soft on surface, radius 6) or "✓ Gate open".
pub fn gate_chip(ui: &mut Ui, text: &str, open: bool) -> Response {
    let (fg, bg) = if open { status_pair("Supported") } else { (tok::INK_SOFT, tok::SURFACE) };
    let label = if open { "✓ Gate open".to_string() } else { format!("◇ {text}") };
    let galley = ui.painter().layout_no_wrap(label, fs(size::TAG), fg);
    let size = galley.size() + egui::vec2(14.0, 6.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    ui.painter().rect_filled(rect, radius::GATE, bg);
    ui.painter().galley(rect.min + egui::vec2(7.0, 3.0), galley, fg);
    resp
}

/// A filter chip (radius 7) with an optional count.
pub fn filter_chip(ui: &mut Ui, p: &Pal, text: &str, count: Option<usize>, selected: bool) -> Response {
    let label = match count {
        Some(n) => format!("{text}  {n}"),
        None => text.to_string(),
    };
    let fg = if selected { tok::TEXT_ON_DARK } else { p.text };
    let galley = ui.painter().layout_no_wrap(label, fs(size::SMALL), fg);
    let size = galley.size() + egui::vec2(20.0, 10.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let (fill, stroke) = if selected { (tok::INK, Stroke::NONE) } else if resp.hovered() { (p.surface_raised, Stroke::new(1.0, p.line)) } else { (p.card, Stroke::new(1.0, p.line)) };
    ui.painter().rect(rect, radius::CHIP, fill, stroke);
    ui.painter().galley(rect.min + egui::vec2(10.0, 5.0), galley, fg);
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A segmented control (List ⇄ Board, Phase ⇄ Theme): track, white thumb that slides 320 ms
/// e-spring. Returns the newly picked index.
pub fn segmented(ui: &mut Ui, p: &Pal, id: egui::Id, options: &[&str], selected: usize) -> Option<usize> {
    let font = fs(size::SMALL);
    let widths: Vec<f32> = options.iter().map(|o| ui.painter().layout_no_wrap(o.to_string(), font.clone(), p.text).size().x + 24.0).collect();
    let total: f32 = widths.iter().sum::<f32>() + 6.0;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(total, 32.0), Sense::hover());
    ui.painter().rect_filled(rect, radius::SMALL_BUTTON + 1.0, p.track);
    let mut x = rect.left() + 3.0;
    let mut lefts = vec![];
    for w in &widths {
        lefts.push(x);
        x += w;
    }
    let tl = motion::tween(ui.ctx(), id.with("thumb-x"), lefts.get(selected).copied().unwrap_or(rect.left() + 3.0) - rect.left(), timing(320.0, 0.0, E_SPRING));
    let tw = motion::tween(ui.ctx(), id.with("thumb-w"), widths.get(selected).copied().unwrap_or(40.0), timing(320.0, 0.0, E_SPRING));
    let thumb = Rect::from_min_size(egui::pos2(rect.left() + tl, rect.top() + 3.0), egui::vec2(tw, rect.height() - 6.0));
    ui.painter().rect(thumb, radius::SMALL_BUTTON - 1.0, p.card, Stroke::NONE);
    let mut picked = None;
    for (i, o) in options.iter().enumerate() {
        let r = Rect::from_min_size(egui::pos2(lefts[i], rect.top()), egui::vec2(widths[i], rect.height()));
        let resp = ui.interact(r, id.with(("seg", i)), Sense::click());
        if resp.clicked() && i != selected {
            picked = Some(i);
        }
        let col = if i == selected { p.text } else { p.dim };
        ui.painter().text(r.center(), Align2::CENTER_CENTER, *o, font.clone(), col);
    }
    picked
}

/// A skeleton bar (loading state): a track-coloured block whose brightness pulses on a 1600 ms
/// loop (holds still with reduced motion).
pub fn skeleton(ui: &mut Ui, p: &Pal, width: f32, height: f32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), Sense::hover());
    let ph = motion::loop_phase(ui.ctx(), 1600.0);
    let k = 0.75 + 0.25 * (0.5 - 0.5 * (ph * std::f32::consts::TAU).cos());
    ui.painter().rect_filled(rect, radius::SMALL_BUTTON, p.track.gamma_multiply(k));
}

/// A loading card in the shape of a content card: a title bar, `lines` text bars, and one line
/// saying what is happening.
pub fn skeleton_card(ui: &mut Ui, p: &Pal, lines: usize, doing: &str) {
    card_lift(ui, p, |ui| {
        let w = ui.available_width();
        skeleton(ui, p, w * 0.45, 16.0);
        ui.add_space(space::S10);
        for i in 0..lines {
            skeleton(ui, p, if i + 1 == lines { w * 0.6 } else { w }, 10.0);
            ui.add_space(space::S8);
        }
        ui.add_space(space::S4);
        ui.horizontal(|ui| {
            ui.spinner();
            wrap_label(ui, RichText::new(doing).size(size::SMALL).color(p.dim));
        });
    });
}

/// The empty state: what goes here, and one action. Returns true when the action is pressed.
pub fn empty_block(ui: &mut Ui, p: &Pal, title: &str, line: &str, action: &str) -> bool {
    let mut go = false;
    card_lift(ui, p, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(space::S12);
            ui.label(RichText::new(title).font(fserif(size::CARD_SERIF)).color(p.text));
            ui.add_space(space::S6);
            wrap_label(ui, RichText::new(line).size(size::BODY).color(p.dim));
            ui.add_space(space::S14);
            go = button(ui, p, ButtonKind::Primary, action, false, true).clicked();
            ui.add_space(space::S12);
        });
    });
    go
}

/// What the error block's buttons did.
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct ErrorAction {
    pub retry: bool,
    pub manual: bool,
}

/// The error state: what failed, that nothing else was lost, Try again (focused) and a manual
/// path. Terracotta on terracotta-tint, inside its own card so the rest of the screen loads.
pub fn error_block(ui: &mut Ui, p: &Pal, what_failed: &str, manual: &str) -> ErrorAction {
    let mut out = ErrorAction::default();
    egui::Frame::none().fill(p.orange_tint).rounding(Rounding::same(radius::CARD)).inner_margin(egui::Margin::same(space::S16)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.vertical(|ui| {
            ui.label(RichText::new(what_failed).font(fs(size::BODY)).color(p.orange));
            wrap_label(ui, RichText::new("Nothing else was lost.").size(size::SMALL).color(p.dim));
            ui.add_space(space::S10);
            ui.horizontal_wrapped(|ui| {
                let r = button(ui, p, ButtonKind::Dark, "Try again", true, true);
                if !ui.ctx().memory(|m| m.focused().is_some()) {
                    r.request_focus();
                }
                out.retry = r.clicked();
                out.manual = button(ui, p, ButtonKind::Ghost, manual, true, true).clicked();
            });
        });
    });
    out
}

/// The mic button's states (E2). Speech to text is wired up separately once the approach is
/// approved; the component only draws.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum MicState {
    #[default]
    Idle,
    /// Recording: terracotta, with a pulse ring (0 → 8 px, fading) on a 1200 ms loop.
    Recording,
    /// Turning speech into text.
    Working,
}

/// The 34 px round mic button. Press scales to .975 (240 e-std).
pub fn mic(ui: &mut Ui, p: &Pal, state: MicState, on_dark: bool) -> Response {
    let d = space::MIC;
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(d, d), Sense::click());
    let s = motion::tween(ui.ctx(), resp.id.with("press"), if resp.is_pointer_button_down_on() { 0.975 } else { 1.0 }, timing(240.0, 0.0, E_STD));
    let c = rect.center();
    let r = d / 2.0 * s;
    let painter = ui.painter();
    let (fill, fg, edge) = match state {
        MicState::Recording => (if on_dark { tok::TERRACOTTA_ON_DARK } else { tok::TERRACOTTA }, tok::TEXT_ON_DARK, None),
        MicState::Working => (if on_dark { tok::PANE } else { p.surface_raised }, if on_dark { tok::TEXT_ON_DARK_SOFT } else { p.dim }, Some(if on_dark { tok::white_a(0.12) } else { p.line })),
        MicState::Idle => {
            let f = if resp.hovered() { if on_dark { tok::white_a(0.10) } else { p.surface_raised } } else if on_dark { tok::white_a(0.06) } else { p.card };
            (f, if on_dark { tok::TEXT_ON_DARK_SOFT } else { p.dim }, Some(if on_dark { tok::white_a(0.12) } else { p.line }))
        }
    };
    if state == MicState::Recording {
        // pulse: a ring growing 0 → 8 px while fading (1200 ms loop; still with reduced motion)
        let ph = motion::loop_phase(ui.ctx(), 1200.0);
        let grow: f32 = 8.0 * motion::E_OUT.at(ph);
        let ring_col = if on_dark { tok::TERRACOTTA_ON_DARK } else { tok::TERRACOTTA };
        let alpha = if motion::reduced() { 0.35 } else { 0.45 * (1.0 - ph) };
        painter.circle_stroke(c, r + grow.max(if motion::reduced() { 4.0 } else { 0.0 }), Stroke::new(2.0, ring_col.gamma_multiply(alpha)));
    }
    painter.circle_filled(c, r, fill);
    if let Some(e) = edge {
        painter.circle_stroke(c, r, Stroke::new(1.0, e));
    }
    if state == MicState::Working {
        let ph = motion::loop_phase(ui.ctx(), 900.0);
        let a0 = ph * std::f32::consts::TAU;
        let pts: Vec<Pos2> = (0..=20).map(|i| a0 + i as f32 / 20.0 * 4.0).map(|a| c + egui::vec2(a.cos(), a.sin()) * 7.0).collect();
        painter.add(Shape::line(pts, Stroke::new(1.6, fg)));
    } else {
        // the mic glyph: a capsule, a cradle and a stem (stroke 1.4)
        let st = Stroke::new(1.5, fg);
        let cap = Rect::from_center_size(c + egui::vec2(0.0, -2.5), egui::vec2(6.0, 10.0));
        painter.rect_stroke(cap, 3.0, st);
        let cradle: Vec<Pos2> = (0..=12).map(|i| {
            let a = std::f32::consts::PI * (i as f32 / 12.0);
            c + egui::vec2(-a.cos() * 6.0, a.sin() * 5.5 - 1.0)
        }).collect();
        painter.add(Shape::line(cradle, st));
        painter.line_segment([c + egui::vec2(0.0, 4.5), c + egui::vec2(0.0, 7.5)], st);
    }
    let tip = match state {
        MicState::Idle => "Speak instead of typing",
        MicState::Recording => "Recording. Click to stop",
        MicState::Working => "Turning speech into text",
    };
    resp.on_hover_text(tip).on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// One toast (B-E10): ink, radius 14, 460 wide, text on dark; with Undo, a 3 px amber timer bar
/// across the bottom shows the time left (`timer` 1 → 0). Returns true when Undo is pressed.
pub fn paint_toast(ui: &mut Ui, text: &str, undo: bool, timer: f32) -> bool {
    let mut pressed = false;
    let w = space::TOAST_W;
    let shadow_at = ui.painter().add(Shape::Noop);
    // on dark, ink barely differs from the work area: a faint edge keeps the toast's shape
    let edge = if ui.visuals().dark_mode { Stroke::new(stroke::HAIR, tok::white_a(0.10)) } else { Stroke::NONE };
    let inner = egui::Frame::none().fill(tok::INK).stroke(edge).rounding(Rounding::same(radius::TOAST)).inner_margin(egui::Margin::symmetric(space::S16, space::S12)).show(ui, |ui| {
        ui.set_width(w - space::S16 * 2.0);
        ui.horizontal(|ui| {
            let tw = if undo { w - space::S16 * 2.0 - 70.0 } else { w - space::S16 * 2.0 };
            ui.scope(|ui| {
                ui.set_max_width(tw);
                ui.add(egui::Label::new(RichText::new(text).size(size::BODY).color(tok::TEXT_ON_DARK)).wrap());
            });
            if undo {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let g = ui.painter().layout_no_wrap("Undo".into(), fs(size::BODY), tok::AMBER);
                    let (r, resp) = ui.allocate_exact_size(g.size() + egui::vec2(12.0, 8.0), Sense::click());
                    if resp.hovered() {
                        ui.painter().rect_filled(r, radius::SMALL_BUTTON, tok::white_a(0.08));
                    }
                    ui.painter().galley(r.min + egui::vec2(6.0, 4.0), g, tok::AMBER);
                    pressed = resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked();
                });
            }
        });
    });
    let rect = inner.response.rect;
    ui.painter().set(shadow_at, Shape::Rect(shadow::ASK_BAR.as_shape(rect, Rounding::same(radius::TOAST))));
    if undo {
        let t = timer.clamp(0.0, 1.0);
        let bar = Rect::from_min_size(egui::pos2(rect.left() + radius::TOAST / 2.0, rect.bottom() - space::TOAST_TIMER), egui::vec2((rect.width() - radius::TOAST) * t, space::TOAST_TIMER));
        ui.painter().rect_filled(bar, space::TOAST_TIMER / 2.0, tok::AMBER);
    }
    pressed
}

/// Make the entrance helper available to views with the brief's common timings.
pub mod moves {
    use super::motion::{timing, Timing, E_IN, E_OUT, E_SPRING, E_STD};
    /// Cards arriving: 460 e-spring at 120, stagger 70 (y 18, scale .98).
    pub const CARDS: Timing = timing(460.0, 120.0, E_SPRING);
    pub const CARDS_STAGGER: f32 = 70.0;
    /// A header settling: 340 e-out (y −8 → 0).
    pub const HEADER: Timing = timing(340.0, 0.0, E_OUT);
    /// Toasts: in 360 e-spring (y 20), out 260 e-in (y 16), swap 160 e-in.
    pub const TOAST_IN: Timing = timing(360.0, 0.0, E_SPRING);
    pub const TOAST_OUT: Timing = timing(260.0, 0.0, E_IN);
    pub const TOAST_SWAP: Timing = timing(160.0, 0.0, E_IN);
    /// Toast lifetimes: with Undo 4 s (timer bar), plain 2.6 s.
    pub const TOAST_UNDO_MS: f32 = 4000.0;
    pub const TOAST_PLAIN_MS: f32 = 2600.0;
    /// Section switch: out 240 e-in (x −24), in 400 e-spring at 90 (x 28).
    pub const SECTION_OUT: Timing = timing(240.0, 0.0, E_IN);
    pub const SECTION_IN: Timing = timing(400.0, 90.0, E_SPRING);
    /// Dims and colour changes.
    pub const DIM: Timing = timing(240.0, 0.0, E_STD);
    /// Sidebar collapse: labels fade 120 e-std, then width 248 ↔ 64 in 320 e-out.
    pub const SIDEBAR_WIDTH: Timing = timing(320.0, 120.0, E_OUT);
    /// Theme expands: height 360 e-spring.
    pub const EXPAND: Timing = timing(360.0, 0.0, E_SPRING);
    /// Sidebar labels fade 120 e-std (before the width when collapsing, after it when expanding).
    pub const SIDEBAR_LABELS: Timing = timing(120.0, 0.0, E_STD);
    /// Ask bar rises 480 e-spring (y 30), at 460 on Home and 760 on Tasks.
    pub const ASK_BAR: Timing = timing(480.0, 460.0, E_SPRING);
    pub const ASK_BAR_TASKS_DELAY: f32 = 760.0;
    pub const ASK_BAR_RISE: f32 = 30.0;
    /// The Try panel: 300 e-spring from the bottom centre (scale .96 → 1).
    pub const TRY: Timing = timing(300.0, 0.0, E_SPRING);
    pub const TRY_SCALE: f32 = 0.96;
    /// Ask sheet: dim 240 e-std; sheet 460 e-spring at 40 (y 100% → 0); rows 300 e-out, stagger
    /// 60; close 300 e-in, the dim leaving at 80.
    pub const SHEET_DIM: Timing = timing(240.0, 0.0, E_STD);
    pub const SHEET_IN: Timing = timing(460.0, 40.0, E_SPRING);
    pub const SHEET_ROWS: Timing = timing(300.0, 40.0, E_OUT);
    pub const SHEET_ROWS_STAGGER: f32 = 60.0;
    pub const SHEET_OUT: Timing = timing(300.0, 0.0, E_IN);
    pub const SHEET_DIM_OUT: Timing = timing(240.0, 80.0, E_STD);
    /// Dialogs (the search palette): dim 180 e-std, dialog 300 e-spring at 40 (scale .96, y −6),
    /// close 200 e-in. Search results swap with no motion.
    pub const DIALOG_DIM: Timing = timing(180.0, 0.0, E_STD);
    pub const DIALOG_IN: Timing = timing(300.0, 40.0, E_SPRING);
    pub const DIALOG_OUT: Timing = timing(200.0, 0.0, E_IN);
    pub const DIALOG_SCALE: f32 = 0.96;
    pub const DIALOG_RISE: f32 = 6.0;
}

// ======================================================================================
// Shell components (Phase 1): sidebar items and icons, the Ask bar, sheets, dialogs and the
// search palette's rows. Views place them; everything they draw comes from theme.rs.
// ======================================================================================

/// Line icons for the shell: 16 px, stroke 1.4.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Icon {
    Home,
    Projects,
    Calendar,
    Overview,
    Roadmap,
    Tasks,
    Frameworks,
    Knowledge,
    Team,
    Handoffs,
    Search,
    Settings,
    Help,
    Plus,
    Spark,
    Undo,
    Redo,
    Collapse,
    Expand,
    Close,
    Down,
    Resume,
}

/// Draw one line icon centred on `c`.
pub fn paint_icon(painter: &egui::Painter, c: Pos2, icon: Icon, col: Color32) {
    let st = Stroke::new(stroke::ICON, col);
    let v = |x: f32, y: f32| c + egui::vec2(x, y);
    let star = |r: f32, k: f32| -> Vec<Pos2> {
        (0..8).map(|i| {
            let a = std::f32::consts::FRAC_PI_4 * i as f32 - std::f32::consts::FRAC_PI_2;
            let rr = if i % 2 == 0 { r } else { r * k };
            c + egui::vec2(a.cos(), a.sin()) * rr
        }).collect()
    };
    match icon {
        Icon::Home => {
            painter.add(Shape::line(vec![v(-6.5, -0.5), v(0.0, -6.5), v(6.5, -0.5)], st));
            painter.add(Shape::line(vec![v(-5.0, -2.0), v(-5.0, 6.5), v(5.0, 6.5), v(5.0, -2.0)], st));
            painter.add(Shape::line(vec![v(-1.8, 6.5), v(-1.8, 2.5), v(1.8, 2.5), v(1.8, 6.5)], st));
        }
        Icon::Projects => {
            for (x, y) in [(-6.5, -6.5), (0.8, -6.5), (-6.5, 0.8), (0.8, 0.8)] {
                painter.rect_stroke(Rect::from_min_size(v(x, y), egui::vec2(5.7, 5.7)), 1.5, st);
            }
        }
        Icon::Calendar => {
            painter.rect_stroke(Rect::from_min_max(v(-6.5, -5.0), v(6.5, 6.5)), 2.0, st);
            painter.line_segment([v(-6.5, -1.2), v(6.5, -1.2)], st);
            painter.line_segment([v(-3.0, -7.0), v(-3.0, -3.5)], st);
            painter.line_segment([v(3.0, -7.0), v(3.0, -3.5)], st);
        }
        Icon::Overview => {
            painter.circle_stroke(c, 6.5, st);
            painter.circle_stroke(c, 3.0, st);
            painter.circle_filled(c, 1.0, col);
        }
        Icon::Roadmap => {
            for (x0, x1, y) in [(-6.5, 2.5, -4.5), (-3.5, 6.5, 0.0), (-6.5, 0.5, 4.5)] {
                painter.line_segment([v(x0, y), v(x1, y)], st);
                painter.circle_filled(v(x0, y), 1.3, col);
            }
        }
        Icon::Tasks => {
            painter.rect_stroke(Rect::from_center_size(c, egui::vec2(13.0, 13.0)), 3.0, st);
            painter.add(Shape::line(vec![v(-3.2, 0.2), v(-0.8, 2.6), v(3.6, -2.2)], st));
        }
        Icon::Frameworks => {
            painter.circle_stroke(v(0.0, -4.5), 2.2, st);
            painter.circle_stroke(v(-4.5, 4.5), 2.2, st);
            painter.circle_stroke(v(4.5, 4.5), 2.2, st);
            painter.line_segment([v(-1.2, -2.6), v(-3.4, 2.5)], st);
            painter.line_segment([v(1.2, -2.6), v(3.4, 2.5)], st);
        }
        Icon::Knowledge => {
            painter.add(Shape::line(vec![v(0.0, -4.5), v(-2.5, -6.0), v(-6.5, -6.0), v(-6.5, 5.0), v(-2.5, 5.0), v(0.0, 6.5)], st));
            painter.add(Shape::line(vec![v(0.0, -4.5), v(2.5, -6.0), v(6.5, -6.0), v(6.5, 5.0), v(2.5, 5.0), v(0.0, 6.5)], st));
            painter.line_segment([v(0.0, -4.5), v(0.0, 6.5)], st);
        }
        Icon::Team | Icon::Spark => {
            painter.add(Shape::closed_line(star(7.0, 0.32), st));
        }
        Icon::Handoffs => {
            painter.line_segment([v(-6.5, -2.8), v(6.0, -2.8)], st);
            painter.add(Shape::line(vec![v(3.5, -5.3), v(6.0, -2.8), v(3.5, -0.3)], st));
            painter.line_segment([v(6.5, 2.8), v(-6.0, 2.8)], st);
            painter.add(Shape::line(vec![v(-3.5, 0.3), v(-6.0, 2.8), v(-3.5, 5.3)], st));
        }
        Icon::Search => {
            painter.circle_stroke(v(-1.2, -1.2), 5.0, st);
            painter.line_segment([v(2.6, 2.6), v(6.5, 6.5)], st);
        }
        Icon::Settings => {
            painter.circle_stroke(c, 4.6, st);
            painter.circle_stroke(c, 1.8, st);
            for i in 0..8 {
                let a = std::f32::consts::FRAC_PI_4 * i as f32;
                let d = egui::vec2(a.cos(), a.sin());
                painter.line_segment([c + d * 4.6, c + d * 6.8], st);
            }
        }
        Icon::Help => {
            painter.circle_stroke(c, 6.8, st);
            let q: Vec<Pos2> = (0..=10).map(|i| {
                let a = std::f32::consts::PI * (1.0 + i as f32 / 10.0 * 1.25);
                v(0.0, -2.0) + egui::vec2(a.cos(), a.sin()) * 2.6
            }).collect();
            let end = *q.last().unwrap();
            painter.add(Shape::line(q, st));
            painter.line_segment([end, v(0.0, 2.0)], st);
            painter.circle_filled(v(0.0, 4.4), 0.9, col);
        }
        Icon::Plus => {
            painter.line_segment([v(-5.5, 0.0), v(5.5, 0.0)], st);
            painter.line_segment([v(0.0, -5.5), v(0.0, 5.5)], st);
        }
        Icon::Undo | Icon::Redo => {
            let m = if icon == Icon::Undo { 1.0 } else { -1.0 };
            let arc: Vec<Pos2> = (0..=12).map(|i| {
                let a = -std::f32::consts::PI * i as f32 / 12.0;
                v(m * 5.0 * a.cos(), 5.0 * a.sin() + 2.5)
            }).collect();
            painter.add(Shape::line(arc, st));
            painter.add(Shape::line(vec![v(-m * 7.5, 0.0), v(-m * 5.0, 2.8), v(-m * 2.5, 0.0)], st));
        }
        Icon::Collapse | Icon::Expand => {
            let m = if icon == Icon::Collapse { 1.0 } else { -1.0 };
            for dx in [-2.2, 2.2] {
                painter.add(Shape::line(vec![v(m * (dx + 2.2), -4.5), v(m * (dx - 2.2), 0.0), v(m * (dx + 2.2), 4.5)], st));
            }
        }
        Icon::Close => {
            painter.line_segment([v(-5.0, -5.0), v(5.0, 5.0)], st);
            painter.line_segment([v(5.0, -5.0), v(-5.0, 5.0)], st);
        }
        Icon::Down => {
            painter.add(Shape::line(vec![v(-3.5, -1.5), v(0.0, 2.0), v(3.5, -1.5)], st));
        }
        Icon::Resume => {
            painter.circle_stroke(c, 6.8, st);
            painter.add(Shape::closed_line(vec![v(-1.8, -3.5), v(3.6, 0.0), v(-1.8, 3.5)], st));
        }
    }
}

/// A 16 px icon as a widget (allocates its square).
pub fn icon_widget(ui: &mut Ui, icon: Icon, col: Color32) -> Response {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(space::NAV_ICON, space::NAV_ICON), Sense::hover());
    paint_icon(ui.painter(), rect.center(), icon, col);
    resp
}

/// A small round icon button (undo, redo, collapse, close). `on_dark` for the sidebar.
pub fn icon_button(ui: &mut Ui, icon: Icon, enabled: bool, on_dark: bool) -> Response {
    let d = space::NAV_ICON + space::S10;
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(d, d), if enabled { Sense::click() } else { Sense::hover() });
    let (fg, hover) = if on_dark { (tok::TEXT_ON_DARK_SOFT, tok::PANE_HOVER) } else { (tok::INK_SOFT, tok::SURFACE_SUNK) };
    if enabled && resp.hovered() {
        ui.painter().rect_filled(rect, radius::SMALL_BUTTON, hover);
    }
    let col = if enabled { fg } else { fg.gamma_multiply(0.35) };
    paint_icon(ui.painter(), rect.center(), icon, col);
    if enabled { resp.on_hover_cursor(egui::CursorIcon::PointingHand) } else { resp }
}

/// The amber workspace tile with the workspace's initial.
pub fn ws_tile(painter: &egui::Painter, rect: Rect, initial: &str) {
    painter.rect_filled(rect, radius::TILE, tok::AMBER);
    painter.text(rect.center(), Align2::CENTER_CENTER, initial, fb(size::LABEL), tok::INK);
}

/// One sidebar item: 32 high, radius 8, icon 16 at 20 from the left, label Inter 13.5 in
/// text-on-dark-soft. Active: amber .16 fill (300 e-std), label text-on-dark, icon amber.
/// `labels` (1 → 0) fades the label and count when the sidebar collapses; collapsed items are
/// 40 wide, so the icon is centred without moving.
pub fn side_item(ui: &mut Ui, icon: Icon, label: &str, right: Option<String>, active: bool, labels: f32) -> Response {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), space::NAV_ITEM_H), Sense::click());
    let tint = motion::tween(ui.ctx(), resp.id.with("active"), if active { 1.0 } else { 0.0 }, timing(300.0, 0.0, E_STD));
    let painter = ui.painter();
    if resp.hovered() && !active {
        painter.rect_filled(rect, radius::NAV_ITEM, tok::PANE_RAISED);
    }
    if tint > 0.001 {
        painter.rect_filled(rect, radius::NAV_ITEM, tok::AMBER_16_ON_NAV.gamma_multiply(tint));
    }
    let icon_col = if active { tok::AMBER } else { tok::TEXT_ON_DARK_SOFT };
    paint_icon(painter, rect.left_center() + egui::vec2(space::NAV_ICON_X, 0.0), icon, icon_col);
    if labels > 0.01 {
        let fg = if active { tok::TEXT_ON_DARK } else { tok::TEXT_ON_DARK_SOFT };
        // the right-hand note: a count (Tasks) or a shortcut (Search)
        let right_g = right.map(|r| painter.layout_no_wrap(r, fp(size::SMALL), tok::TEXT_ON_DARK_MUTED.gamma_multiply(labels)));
        let right_w = right_g.as_ref().map_or(space::S8, |g| g.size().x + space::S18);
        let room = rect.width() - space::NAV_LABEL_X - right_w;
        let text = fit_text(painter, label, fm(size::NAV), room);
        painter.text(rect.left_center() + egui::vec2(space::NAV_LABEL_X, 0.0), Align2::LEFT_CENTER, text, fm(size::NAV), fg.gamma_multiply(labels));
        if let Some(g) = right_g {
            painter.galley(egui::pos2(rect.right() - space::S10 - g.size().x, rect.center().y - g.size().y / 2.0), g, tok::TEXT_ON_DARK_MUTED);
        }
    }
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Shorten `text` with "…" to fit `width`.
pub fn fit_text(painter: &egui::Painter, text: &str, font: egui::FontId, width: f32) -> String {
    if painter.layout_no_wrap(text.to_string(), font.clone(), Color32::WHITE).size().x <= width {
        return text.to_string();
    }
    let mut t: String = text.to_string();
    while !t.is_empty() && painter.layout_no_wrap(format!("{t}…"), font.clone(), Color32::WHITE).size().x > width {
        t.pop();
    }
    format!("{}…", t.trim_end())
}

/// A sidebar group label (Inter 600 11, letter spacing .04em, text-on-dark-muted). When the
/// sidebar collapses the label fades and a thin divider takes its place.
pub fn side_group(ui: &mut Ui, label: &str, labels: f32) {
    ui.add_space(space::S12);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), size::GROUP + space::S8), Sense::hover());
    let painter = ui.painter();
    if labels > 0.01 {
        let mut job = egui::text::LayoutJob::default();
        job.append(&label.to_uppercase(), 0.0, egui::TextFormat { font_id: fs(size::GROUP), color: tok::TEXT_ON_DARK_MUTED.gamma_multiply(labels), extra_letter_spacing: size::GROUP_SPACING, ..Default::default() });
        let g = painter.layout_job(job);
        painter.galley(egui::pos2(rect.left() + space::NAV_PAD, rect.center().y - g.size().y / 2.0), g, tok::TEXT_ON_DARK_MUTED);
    }
    if labels < 0.99 {
        let y = rect.center().y;
        painter.line_segment([egui::pos2(rect.left() + space::S8, y), egui::pos2(rect.right() - space::S8, y)], Stroke::new(stroke::HAIR, tok::white_a(0.08 * (1.0 - labels))));
    }
}

/// A full-width divider in the sidebar (white .08).
pub fn side_divider(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), space::S10), Sense::hover());
    ui.painter().line_segment([rect.left_center(), rect.right_center()], Stroke::new(stroke::HAIR, tok::white_a(0.08)));
}

/// The tooltip beside a collapsed sidebar item (S4): after 400 ms of hover, its name slides in
/// from x −4 over 120 ms e-out.
pub fn side_tip(ui: &Ui, resp: &Response, text: &str) {
    let ctx = ui.ctx();
    let id = resp.id.with("side-tip");
    if !resp.hovered() {
        ctx.data_mut(|d| d.remove::<f64>(id));
        return;
    }
    let now = ui.input(|i| i.time);
    let since = ctx.data_mut(|d| *d.get_temp_mut_or_insert_with(id, || now));
    let f = motion::frame_at(((now - since) * 1000.0) as f32, timing(120.0, 400.0, E_OUT), motion::reduced());
    if !f.done {
        ctx.request_repaint();
    }
    if f.alpha <= 0.0 {
        return;
    }
    let x = resp.rect.right() + space::NAV_TIP_GAP - 4.0 * (1.0 - f.t);
    egui::Area::new(id).order(egui::Order::Tooltip).fixed_pos(egui::pos2(x, resp.rect.center().y)).pivot(Align2::LEFT_CENTER).interactable(false).show(ctx, |ui| {
        ui.multiply_opacity(f.alpha);
        egui::Frame::none().fill(tok::INK).stroke(Stroke::new(stroke::HAIR, tok::white_a(0.10))).rounding(Rounding::same(radius::SMALL_BUTTON)).inner_margin(egui::Margin::symmetric(space::S10, space::S6)).show(ui, |ui| {
            ui.add(egui::Label::new(RichText::new(text).font(fp(size::LABEL)).color(tok::TEXT_ON_DARK)).extend());
        });
    });
}

/// The sidebar's two buttons: Add task (amber) and Ask AI (outlined, ✦ amber). Press scales to
/// .975. With `labels` at 0 (collapsed) only the icon shows.
pub fn side_button(ui: &mut Ui, primary: bool, icon: Icon, label: &str, labels: f32, on: bool) -> Response {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), space::NAV_BUTTON_H), Sense::click());
    let s = motion::tween(ui.ctx(), resp.id.with("press"), if resp.is_pointer_button_down_on() { 0.975 } else { 1.0 }, timing(240.0, 0.0, E_STD));
    let r = motion::scale_about(rect, rect.center(), s);
    let painter = ui.painter();
    let (fg, icon_col) = if primary {
        painter.rect_filled(r, radius::BUTTON, if resp.hovered() { tok::AMBER.gamma_multiply(0.9) } else { tok::AMBER });
        (tok::INK, tok::INK)
    } else {
        let fill = if on { tok::AMBER_16_ON_NAV } else if resp.hovered() { tok::PANE_RAISED } else { Color32::TRANSPARENT };
        painter.rect(r, radius::BUTTON, fill, Stroke::new(stroke::HAIR, tok::white_a(0.18)));
        (tok::TEXT_ON_DARK, tok::AMBER)
    };
    if labels > 0.01 {
        let g = painter.layout_no_wrap(label.to_string(), fs(size::LABEL), fg.gamma_multiply(labels));
        let total = space::NAV_ICON + space::S6 + g.size().x;
        let x0 = r.center().x - total / 2.0;
        paint_icon(painter, egui::pos2(x0 + space::NAV_ICON / 2.0, r.center().y), icon, icon_col);
        painter.galley(egui::pos2(x0 + space::NAV_ICON + space::S6, r.center().y - g.size().y / 2.0), g, fg);
    } else {
        paint_icon(painter, r.center(), icon, icon_col);
    }
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// "Continue setting up…" (B-E2): an amber-tinted card in the sidebar footer, shown only while
/// an intake is unfinished. Collapsed, it is the resume icon with an amber dot.
pub fn setup_card(ui: &mut Ui, title: &str, line: &str, labels: f32) -> Response {
    let h = if labels > 0.01 { space::NAV_ITEM_H + space::S18 } else { space::NAV_ITEM_H };
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), h), Sense::click());
    let painter = ui.painter();
    painter.rect_filled(rect, radius::BUTTON, if resp.hovered() { tok::AMBER_16_ON_NAV } else { tok::AMBER_16_ON_NAV.gamma_multiply(0.75) });
    let ic = egui::pos2(rect.left() + space::NAV_ICON_X, rect.center().y);
    paint_icon(painter, ic, Icon::Resume, tok::AMBER);
    if labels > 0.01 {
        painter.text(egui::pos2(rect.left() + space::NAV_LABEL_X, rect.center().y - space::S8), Align2::LEFT_CENTER, title, fs(size::LABEL), tok::TEXT_ON_DARK.gamma_multiply(labels));
        let line = fit_text(painter, line, fp(size::TINY), rect.width() - space::NAV_LABEL_X - space::S8);
        painter.text(egui::pos2(rect.left() + space::NAV_LABEL_X, rect.center().y + space::S8), Align2::LEFT_CENTER, line, fp(size::TINY), tok::TEXT_ON_DARK_SOFT.gamma_multiply(labels));
    } else {
        painter.circle_filled(ic + egui::vec2(space::S8, -space::S8), stroke::DOT / 2.0 + 1.0, tok::AMBER);
    }
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// The save status at the foot of the sidebar: a dot (green when all is well, terracotta on an
/// error) and the line. Collapsed: the dot only.
pub fn side_status(ui: &mut Ui, text: &str, ok: bool, labels: f32) -> Response {
    let w = ui.available_width();
    let text_w = w - space::NAV_LABEL_X + space::S10;
    let g = (labels > 0.01).then(|| ui.painter().layout(text.to_string(), fp(size::TINY), tok::TEXT_ON_DARK_MUTED.gamma_multiply(labels), text_w));
    let h = g.as_ref().map_or(space::S18, |g| g.size().y.max(space::S18));
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, h), Sense::hover());
    let dot = egui::pos2(rect.left() + space::NAV_ICON_X, rect.top() + space::S8 + 1.0);
    ui.painter().circle_filled(dot, stroke::DOT / 2.0 + 0.5, if ok { tok::GREEN } else { tok::TERRACOTTA_ON_DARK });
    if let Some(g) = g {
        ui.painter().galley(egui::pos2(rect.left() + space::NAV_ICON_X + space::S10, rect.top()), g, tok::TEXT_ON_DARK_MUTED);
    }
    resp
}

/// The project picker's row (dot + name + ▾) on the dark sidebar. Collapsed: the dot alone.
pub fn picker_row(ui: &mut Ui, dot: Color32, name: &str, labels: f32) -> Response {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), space::NAV_ITEM_H), Sense::click());
    let painter = ui.painter();
    painter.rect(rect, radius::NAV_ITEM, if resp.hovered() { tok::PANE_HOVER } else { tok::PANE_RAISED }, Stroke::new(stroke::HAIR, tok::PANE_TRACK));
    painter.circle_filled(rect.left_center() + egui::vec2(space::NAV_ICON_X, 0.0), stroke::DOT, dot);
    if labels > 0.01 {
        let room = rect.width() - space::NAV_LABEL_X - space::S24;
        let text = fit_text(painter, name, fs(size::LABEL), room);
        painter.text(rect.left_center() + egui::vec2(space::NAV_LABEL_X - space::S8, 0.0), Align2::LEFT_CENTER, text, fs(size::LABEL), tok::TEXT_ON_DARK.gamma_multiply(labels));
        paint_icon(painter, rect.right_center() - egui::vec2(space::S14, 0.0), Icon::Down, tok::TEXT_ON_DARK_SOFT.gamma_multiply(labels));
    }
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// What the Ask bar did this frame.
pub struct AskBarOut {
    /// Enter pressed (with or without text).
    pub submit: bool,
    pub focused: bool,
}

/// The Ask Resolve bar (B-E3): a 560 × 48 ink pill, ✦ in amber on the left, the screen's
/// placeholder, a 34 px mic on the right. `rect` is where it sits this frame.
pub fn ask_bar(ui: &mut Ui, p: &Pal, rect: Rect, id: egui::Id, text: &mut String, placeholder: &str) -> AskBarOut {
    let painter = ui.painter().clone();
    painter.add(Shape::Rect(shadow::ASK_BAR.as_shape(rect, Rounding::same(radius::PILL))));
    let edge = if ui.visuals().dark_mode { Stroke::new(stroke::HAIR, tok::white_a(0.10)) } else { Stroke::NONE };
    painter.rect(rect, radius::PILL, tok::INK, edge);
    let mut out = AskBarOut { submit: false, focused: false };
    let inner = rect.shrink2(egui::vec2(space::S16, 0.0));
    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(inner).layout(Layout::left_to_right(Align::Center)), |ui| {
        icon_widget(ui, Icon::Spark, tok::AMBER);
        ui.add_space(space::S10);
        let w = ui.available_width() - space::MIC - space::S10;
        let r = ui.add(
            egui::TextEdit::singleline(text)
                .id(id)
                .frame(false)
                .hint_text(RichText::new(placeholder).color(tok::TEXT_ON_DARK_MUTED))
                .text_color(tok::TEXT_ON_DARK)
                .font(fp(size::BODY))
                .desired_width(w),
        );
        out.focused = r.has_focus();
        out.submit = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        ui.add_space(space::S10 - space::S16 / 2.0);
        mic_for(ui, p, id, true);
    });
    out
}

/// The "Try" panel above the Ask bar: the screen's suggestions. Returns the one clicked.
pub fn try_panel(ui: &mut Ui, p: &Pal, suggestions: &[&str]) -> Option<usize> {
    let mut picked = None;
    egui::Frame::none().fill(p.card).stroke(Stroke::new(stroke::HAIR, p.line)).rounding(Rounding::same(radius::CARD)).shadow(shadow::CARD_LIFT[1]).inner_margin(egui::Margin::same(space::S12)).show(ui, |ui| {
        ui.set_width(space::ASK_W - space::S12 * 2.0);
        ui.label(RichText::new("Try").font(fs(size::GROUP)).color(p.faint));
        ui.add_space(space::S4);
        for (i, s) in suggestions.iter().enumerate() {
            let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), space::NAV_ITEM_H), Sense::click());
            if resp.hovered() {
                ui.painter().rect_filled(rect, radius::NAV_ITEM, p.hover);
            }
            paint_icon(ui.painter(), rect.left_center() + egui::vec2(space::S14, 0.0), Icon::Spark, p.accent_text);
            ui.painter().text(rect.left_center() + egui::vec2(space::S30, 0.0), Align2::LEFT_CENTER, *s, fp(size::BODY), p.text);
            if resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                picked = Some(i);
            }
        }
    });
    picked
}

/// Paint a backdrop over `rect` at `alpha` (sheet .32, dialog .38). Returns true when clicked.
pub fn backdrop(ui: &mut Ui, rect: Rect, alpha: f32) -> bool {
    let resp = ui.allocate_rect(rect, Sense::click());
    ui.painter().rect_filled(rect, 0.0, tok::ink_a(alpha));
    resp.clicked()
}

/// A sheet rising from the bottom (A3): surface-raised, radius 20 on the top corners only.
pub fn sheet_frame(p: &Pal) -> egui::Frame {
    let r = radius::SHEET;
    egui::Frame::none().fill(p.surface_raised).rounding(Rounding { nw: r, ne: r, sw: 0.0, se: 0.0 }).shadow(shadow::SHEET).inner_margin(egui::Margin { left: space::S24, right: space::S24, top: space::S18, bottom: space::S16 })
}

/// A centred dialog: card, radius 16, dialog shadow.
pub fn dialog_frame(p: &Pal) -> egui::Frame {
    let edge = if p.dark { Stroke::new(stroke::HAIR, tok::white_a(0.10)) } else { Stroke::NONE };
    egui::Frame::none().fill(p.card).stroke(edge).rounding(Rounding::same(radius::DIALOG)).shadow(shadow::DIALOG)
}

/// A key hint ("Esc", "Enter", "↑ ↓").
pub fn kbd(ui: &mut Ui, p: &Pal, key: &str) -> Response {
    let g = ui.painter().layout_no_wrap(key.to_string(), fp(size::TINY), p.dim);
    let (rect, resp) = ui.allocate_exact_size(g.size() + egui::vec2(space::S10, space::S4), Sense::hover());
    ui.painter().rect(rect, radius::GATE, p.surface_sunk, Stroke::new(stroke::HAIR, p.line));
    ui.painter().galley(rect.min + egui::vec2(space::S10 / 2.0, space::S4 / 2.0), g, p.dim);
    resp
}

/// An uppercase group heading in a list (search results, Try).
pub fn list_heading(ui: &mut Ui, p: &Pal, text: &str) {
    let mut job = egui::text::LayoutJob::default();
    job.append(&text.to_uppercase(), 0.0, egui::TextFormat { font_id: fs(size::GROUP), color: p.faint, extra_letter_spacing: size::GROUP_SPACING, ..Default::default() });
    ui.add_space(space::S8);
    ui.label(job);
    ui.add_space(space::S4);
}

/// One search result: title, then the kind and a snippet. Selected rows take the amber tint.
pub fn search_row(ui: &mut Ui, p: &Pal, title: &str, meta: &str, selected: bool) -> Response {
    let w = ui.available_width();
    let h = size::BODY + size::SMALL + space::S26;
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, h), Sense::click());
    let painter = ui.painter();
    if selected {
        painter.rect_filled(rect, radius::NAV_ITEM, p.accent_soft);
    } else if resp.hovered() {
        painter.rect_filled(rect, radius::NAV_ITEM, p.hover);
    }
    let x = rect.left() + space::S12;
    let room = w - space::S24;
    painter.text(egui::pos2(x, rect.top() + space::S8), Align2::LEFT_TOP, fit_text(painter, title, fp(size::BODY), room), fp(size::BODY), p.text);
    painter.text(egui::pos2(x, rect.bottom() - space::S8), Align2::LEFT_BOTTOM, fit_text(painter, meta, fp(size::SMALL), room), fp(size::SMALL), p.dim);
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

// ======================================================================================
// Daily-screen components (Phase 2, Pack B): Home's cards and rows.
// ======================================================================================

/// A plain white card: border, radius 13, padding 14 / 16. Dashed (an empty card) draws its
/// edge in short strokes.
pub fn white_card<R>(ui: &mut Ui, p: &Pal, dashed: bool, add: impl FnOnce(&mut Ui) -> R) -> R {
    let edge = if dashed { Stroke::NONE } else { Stroke::new(stroke::HAIR, p.line) };
    let inner = egui::Frame::none().fill(p.card).stroke(edge).rounding(Rounding::same(radius::CARD)).inner_margin(egui::Margin::symmetric(space::S16, space::S14)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        // rows inside a card space themselves; the column's gaps stay outside
        ui.spacing_mut().item_spacing.y = space::NAV_GAP;
        ui.vertical(|ui| add(ui)).inner
    });
    if dashed {
        let r = inner.response.rect.shrink(0.5);
        let col = if p.dark { p.line } else { tok::DASHED_EDGE };
        let pts = [r.left_top(), r.right_top(), r.right_bottom(), r.left_bottom(), r.left_top()];
        for w in pts.windows(2) {
            ui.painter().extend(Shape::dashed_line(&[w[0], w[1]], Stroke::new(stroke::HAIR, col), space::S6, space::S4));
        }
    }
    inner.inner
}

/// A card's head: an uppercase label (12, 700) and, on the right, a note or a link. Returns
/// true when the link is clicked.
pub fn card_head(ui: &mut Ui, p: &Pal, label: &str, right: Option<(&str, Color32)>, link: bool) -> bool {
    let mut clicked = false;
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).font(fb(size::CARD_HEAD)).color(p.dim));
        if let Some((text, col)) = right {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let r = ui.add(egui::Label::new(RichText::new(text).font(fs(size::CARD_HEAD)).color(col)).sense(if link { Sense::click() } else { Sense::hover() }));
                if link {
                    clicked = r.on_hover_cursor(egui::CursorIcon::PointingHand).clicked();
                }
            });
        }
    });
    clicked
}

/// The mark at the start of an agenda row: an event (rounded square) or a task (circle).
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Mark {
    Event(Color32),
    Task(Color32),
}

/// One agenda row: time (52 wide), the 16 px mark, title (14, 500) and a meta line (12). The
/// first row has no line above it. Returns (row clicked, mark clicked).
pub fn agenda_row(ui: &mut Ui, p: &Pal, time: &str, mark: Mark, title: &str, meta: &str, first: bool) -> (Response, Response) {
    if !first {
        let r = ui.available_rect_before_wrap();
        ui.painter().line_segment([r.left_top(), r.right_top()], Stroke::new(stroke::HAIR, p.divider));
    }
    let w = ui.available_width();
    let text_w = w - space::AGENDA_TIME_W - space::MARK - space::S14 * 2.0;
    let title_g = ui.painter().layout(title.to_string(), fm(size::ROW), p.text, text_w);
    let meta_g = ui.painter().layout(meta.to_string(), fp(size::SMALL), p.dim, text_w);
    let h = title_g.size().y + space::S4 / 2.0 + meta_g.size().y + space::S10 * 2.0;
    let (rect, row) = ui.allocate_exact_size(egui::vec2(w, h), Sense::click());
    let top = rect.top() + space::S10;
    let painter = ui.painter();
    painter.text(egui::pos2(rect.left(), top + 1.0), Align2::LEFT_TOP, time, fp(size::SMALL), p.dim);
    let mrect = Rect::from_min_size(egui::pos2(rect.left() + space::AGENDA_TIME_W + space::S14, top + 1.0), egui::vec2(space::MARK, space::MARK));
    let mark_resp = ui.interact(mrect, row.id.with("mark"), Sense::click());
    let painter = ui.painter();
    match mark {
        Mark::Event(c) => {
            painter.rect_stroke(mrect.shrink(0.75), space::S4, Stroke::new(stroke::ICON + 0.1, c));
        }
        Mark::Task(c) => {
            painter.circle_stroke(mrect.center(), space::MARK / 2.0 - 0.75, Stroke::new(stroke::ICON + 0.1, c));
            if mark_resp.hovered() {
                painter.circle_filled(mrect.center(), space::MARK / 2.0 - 3.0, c.gamma_multiply(0.4));
            }
        }
    }
    let tx = mrect.right() + space::S14;
    painter.galley(egui::pos2(tx, top), title_g.clone(), p.text);
    painter.galley(egui::pos2(tx, top + title_g.size().y + space::S4 / 2.0), meta_g, p.dim);
    (row.on_hover_cursor(egui::CursorIcon::PointingHand), mark_resp.on_hover_cursor(egui::CursorIcon::PointingHand))
}

/// "Since Monday": a done item with a green tick.
pub fn win_row(ui: &mut Ui, p: &Pal, text: &str) {
    ui.add_space(space::S4);
    ui.horizontal(|ui| {
        let (r, _) = ui.allocate_exact_size(egui::vec2(space::MARK, space::MARK), Sense::hover());
        ui.painter().circle_filled(r.center(), space::MARK / 2.0, p.green_tint);
        ui.painter().add(Shape::line(vec![r.center() + egui::vec2(-3.2, 0.2), r.center() + egui::vec2(-0.9, 2.5), r.center() + egui::vec2(3.3, -2.0)], Stroke::new(stroke::ICON, p.green_text)));
        ui.add_space(space::S10 - ui.spacing().item_spacing.x);
        ui.add(egui::Label::new(RichText::new(text).size(size::ROW_SMALL).color(p.text)).wrap());
    });
}

/// "Waiting on": the item and how long it has waited, in a terracotta pill.
pub fn stalled_row(ui: &mut Ui, p: &Pal, text: &str, age: &str) -> Response {
    ui.add_space(space::S8);
    ui.horizontal_top(|ui| {
        let w = ui.available_width();
        let pill = ui.painter().layout_no_wrap(age.to_string(), fs(size::TINY), p.orange);
        let text_w = w - pill.size().x - space::S16 - space::S10;
        let r = ui.scope(|ui| {
            ui.set_width(text_w);
            ui.add(egui::Label::new(RichText::new(text).size(size::ROW_SMALL).color(p.text)).wrap().sense(Sense::click()))
        });
        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            let (pr, _) = ui.allocate_exact_size(pill.size() + egui::vec2(space::S16, space::S4), Sense::hover());
            ui.painter().rect_filled(pr, radius::PILL, p.orange_tint);
            ui.painter().galley(pr.min + egui::vec2(space::S8, space::S4 / 2.0), pill, p.orange);
        });
        r.inner
    })
    .inner
}

/// "Projects": name, done count, and an amber 5 px bar.
pub fn progress_row(ui: &mut Ui, p: &Pal, name: &str, count: &str, frac: f32) -> Response {
    ui.add_space(space::S10);
    let w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, size::ROW_SMALL + space::S6 + space::BAR_H + space::S4), Sense::click());
    let painter = ui.painter();
    painter.text(rect.left_top(), Align2::LEFT_TOP, fit_text(painter, name, fm(size::ROW_SMALL), w - space::S40 * 2.0), fm(size::ROW_SMALL), p.text);
    painter.text(rect.right_top(), Align2::RIGHT_TOP, count, fp(size::ROW_SMALL), p.dim);
    let bar = Rect::from_min_size(egui::pos2(rect.left(), rect.top() + size::ROW_SMALL + space::S6 + 2.0), egui::vec2(w, space::BAR_H));
    painter.rect_filled(bar, space::BAR_H / 2.0, p.divider);
    painter.rect_filled(Rect::from_min_size(bar.min, egui::vec2(w * frac.clamp(0.0, 1.0), space::BAR_H)), space::BAR_H / 2.0, tok::AMBER);
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// An amber-light strip or card with an action on the right ("2 captured, not in a project yet
/// · Sort →", "Continue setting up · Pick up →"). Returns the click.
pub fn amber_card(ui: &mut Ui, p: &Pal, lead: Option<&str>, title: &str, line: Option<&str>, action: &str) -> Response {
    let (fill, edge) = if p.dark { (p.callout, p.line) } else { (tok::AMBER_LIGHT, tok::AMBER_EDGE) };
    let inner = egui::Frame::none().fill(fill).stroke(Stroke::new(stroke::HAIR, edge)).rounding(Rounding::same(radius::BUTTON + 2.0)).inner_margin(egui::Margin::symmetric(space::S14, space::S12)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            if let Some(l) = lead {
                ui.label(RichText::new(l).font(fb(size::ROW_SMALL)).color(p.text));
            } else {
                let (r, _) = ui.allocate_exact_size(egui::vec2(space::S10, space::S10), Sense::hover());
                ui.painter().circle_filled(r.center(), space::S10 / 2.0, tok::AMBER);
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(RichText::new(action).font(fs(size::ROW_SMALL)).color(p.orange));
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new(title).font(if line.is_some() { fs(size::HERO_BODY) } else { fp(size::ROW_SMALL) }).color(p.text));
                        if let Some(l) = line {
                            ui.add(egui::Label::new(RichText::new(l).size(size::LABEL).color(p.dim)).wrap());
                        }
                    });
                });
            });
        });
    });
    ui.interact(inner.response.rect, inner.response.id.with("amber-card"), Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// The hero's top line: "Your highest-value move" in amber and the project as an outlined pill.
pub fn hero_eyebrow(ui: &mut Ui, label: &str, pill: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).font(fs(size::CARD_HEAD)).color(tok::AMBER));
        ui.add_space(space::S10 - ui.spacing().item_spacing.x);
        let g = ui.painter().layout_no_wrap(pill.to_string(), fp(size::TINY), tok::TEXT_ON_DARK_SOFT);
        let (r, _) = ui.allocate_exact_size(g.size() + egui::vec2(space::S18, space::S4), Sense::hover());
        ui.painter().rect_stroke(r, radius::PILL, Stroke::new(stroke::HAIR, tok::white_a(0.16)));
        ui.painter().galley(r.min + egui::vec2(space::S18 / 2.0, space::S4 / 2.0), g, tok::TEXT_ON_DARK_SOFT);
    });
}

/// One project on the Projects screen (B-E7).
pub struct ProjectCardView<'a> {
    pub name: &'a str,
    pub dot: Color32,
    pub outcome: &'a str,
    pub next: &'a str,
    pub frac: f32,
    pub done: &'a str,
    /// (Now, Next, Later, open questions) as their labels: "2 Now", "8 open questions".
    pub pills: [&'a str; 4],
    pub updated: &'a str,
}

/// A small pill: ink for Now, sunk for Next and Later, terracotta tint for open questions.
pub fn count_pill(ui: &mut Ui, p: &Pal, text: &str, kind: usize) {
    let (fill, fg) = match kind {
        0 => (tok::INK, tok::TEXT_ON_DARK),
        3 => (p.orange_tint, p.orange),
        _ => (p.bg, p.text),
    };
    let g = ui.painter().layout_no_wrap(text.to_string(), fs(size::TINY), fg);
    let (r, _) = ui.allocate_exact_size(g.size() + egui::vec2(space::S18, space::S6), Sense::hover());
    ui.painter().rect_filled(r, radius::PILL, fill);
    ui.painter().galley(r.min + egui::vec2(space::S18 / 2.0, space::S6 / 2.0), g, fg);
}

/// A project card: dot, name (16, 600), ⋯, outcome, "Next move · …", progress, the phase pills and
/// when it was last updated. Returns (card clicked, ⋯ clicked).
pub fn project_card(ui: &mut Ui, p: &Pal, c: &ProjectCardView, menu_open: bool) -> (Response, Response) {
    let mut menu = None;
    let shadow_at = ui.painter().add(Shape::Noop);
    let inner = egui::Frame::none().fill(p.card).stroke(Stroke::new(stroke::HAIR, p.line)).rounding(Rounding::same(radius::CARD)).inner_margin(egui::Margin::symmetric(space::S18 + 2.0, space::S18)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.spacing_mut().item_spacing.y = space::S10;
        ui.horizontal(|ui| {
            let (r, _) = ui.allocate_exact_size(egui::vec2(space::S10, space::S10), Sense::hover());
            ui.painter().circle_filled(r.center(), space::S10 / 2.0, c.dot);
            ui.label(RichText::new(c.name).font(fs(size::SECTION + 1.0)).color(p.text));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let (r, resp) = ui.allocate_exact_size(egui::vec2(space::S26 + 2.0, space::S26 + 2.0), Sense::click());
                if menu_open || resp.hovered() {
                    ui.painter().rect_stroke(r, radius::CHIP, Stroke::new(stroke::HAIR, p.line));
                }
                ui.painter().text(r.center(), Align2::CENTER_CENTER, "⋯", fs(size::BODY), p.dim);
                menu = Some(resp.on_hover_text("More actions").on_hover_cursor(egui::CursorIcon::PointingHand));
            });
        });
        if !c.outcome.is_empty() {
            ui.add(egui::Label::new(RichText::new(c.outcome).size(size::ROW_SMALL).color(p.dim)).wrap());
        }
        if !c.next.is_empty() {
            egui::Frame::none().fill(p.surface_raised).rounding(Rounding::same(radius::SMALL_BUTTON + 1.0)).inner_margin(egui::Margin::symmetric(space::S12, space::S8)).show(ui, |ui| {
                ui.set_width(ui.available_width());
                let mut job = egui::text::LayoutJob::default();
                job.wrap.max_width = ui.available_width();
                job.append("Next move · ", 0.0, egui::TextFormat { font_id: fs(size::LABEL), color: p.orange, ..Default::default() });
                job.append(c.next, 0.0, egui::TextFormat { font_id: fp(size::LABEL), color: p.text, ..Default::default() });
                ui.label(job);
            });
        }
        ui.horizontal(|ui| {
            let g = ui.painter().layout_no_wrap(c.done.to_string(), fp(size::SMALL), p.dim);
            let w = ui.available_width() - g.size().x - space::S10;
            let (r, _) = ui.allocate_exact_size(egui::vec2(w, space::BAR_H), Sense::hover());
            ui.painter().rect_filled(r, space::BAR_H / 2.0, p.divider);
            ui.painter().rect_filled(Rect::from_min_size(r.min, egui::vec2(w * c.frac.clamp(0.0, 1.0), space::BAR_H)), space::BAR_H / 2.0, tok::AMBER);
            ui.label(RichText::new(c.done).size(size::SMALL).color(p.dim));
        });
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = space::S6;
            for (i, t) in c.pills.iter().enumerate() {
                if !t.is_empty() {
                    count_pill(ui, p, t, i);
                }
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(RichText::new(c.updated).size(size::SMALL).color(p.dim));
            });
        });
    });
    let rect = inner.response.rect;
    let card = ui.interact(rect, inner.response.id.with("project-card"), Sense::click());
    if card.hovered() && !p.dark {
        let shapes: Vec<Shape> = shadow::CARD_LIFT.iter().map(|s| Shape::Rect(s.as_shape(rect, Rounding::same(radius::CARD)))).collect();
        ui.painter().set(shadow_at, Shape::Vec(shapes));
    }
    (card.on_hover_cursor(egui::CursorIcon::PointingHand), menu.unwrap())
}

/// The dashed "+ New project" card at the end of a grid.
pub fn dashed_add_card(ui: &mut Ui, p: &Pal, title: &str, line: &str, min_h: f32) -> Response {
    let w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, min_h), Sense::click());
    let r = rect.shrink(0.75);
    let col = if p.dark { p.line } else { tok::DASHED_EDGE };
    let pts = [r.left_top(), r.right_top(), r.right_bottom(), r.left_bottom(), r.left_top()];
    for seg in pts.windows(2) {
        ui.painter().extend(Shape::dashed_line(&[seg[0], seg[1]], Stroke::new(stroke::HAIR + 0.5, col), space::S6, space::S4));
    }
    if resp.hovered() {
        ui.painter().rect_filled(r, radius::CARD, p.surface_raised);
    }
    let c = rect.center();
    ui.painter().circle_filled(c - egui::vec2(0.0, space::S30), space::S40 / 2.0, if p.dark { p.callout } else { tok::AMBER_LIGHT });
    ui.painter().text(c - egui::vec2(0.0, space::S30), Align2::CENTER_CENTER, "+", fp(size::TITLE - 2.0), p.orange);
    ui.painter().text(c + egui::vec2(0.0, space::S6), Align2::CENTER_CENTER, title, fs(size::HERO_BODY), p.text);
    ui.painter().text(c + egui::vec2(0.0, space::S30), Align2::CENTER_CENTER, line, fp(size::LABEL), p.dim);
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A centred empty state: an icon tile, a serif title, a line and one action. Returns the click.
pub fn empty_center(ui: &mut Ui, p: &Pal, icon: Icon, title: &str, body: &str, action: &str) -> bool {
    let mut clicked = false;
    ui.vertical_centered(|ui| {
        ui.set_max_width(space::SEARCH_W - space::S40 * 5.5);
        let (r, _) = ui.allocate_exact_size(egui::vec2(space::S40 + space::S16, space::S40 + space::S16), Sense::hover());
        ui.painter().rect(r, radius::HERO, if p.dark { p.callout } else { tok::AMBER_LIGHT }, Stroke::new(stroke::HAIR, if p.dark { p.line } else { tok::AMBER_EDGE }));
        paint_icon(ui.painter(), r.center(), icon, p.orange);
        ui.add_space(space::S8);
        ui.label(RichText::new(title).font(fserif(size::HERO_TITLE)).color(p.text));
        ui.add(egui::Label::new(RichText::new(body).size(size::HERO_BODY).color(p.dim)).wrap());
        ui.add_space(space::S8);
        clicked = button(ui, p, ButtonKind::Primary, action, false, true).clicked();
    });
    clicked
}

/// A centred error card: what failed (serif 24), that nothing else was lost, Try again and a
/// manual path.
pub fn error_center(ui: &mut Ui, p: &Pal, title: &str, body: &str, manual: &str) -> ErrorAction {
    let mut out = ErrorAction { retry: false, manual: false };
    ui.vertical_centered(|ui| {
        ui.set_max_width(space::SEARCH_W - space::S40 * 4.0);
        white_card(ui, p, false, |ui| {
            ui.spacing_mut().item_spacing.y = space::S8;
            ui.label(RichText::new(title).font(fserif(size::CARD_SERIF)).color(p.text));
            ui.add(egui::Label::new(RichText::new(body).size(size::ROW).color(p.dim)).wrap());
            ui.add_space(space::S8);
            ui.horizontal(|ui| {
                out.retry = button(ui, p, ButtonKind::Dark, "Try again", false, true).clicked();
                out.manual = button(ui, p, ButtonKind::Secondary, manual, false, true).clicked();
            });
        });
    });
    out
}

/// A screen's title (DM Serif 32) with its subline, and room on the right for its main button.
pub fn screen_head(ui: &mut Ui, p: &Pal, title: &str, sub: &str, add_right: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new(title).font(fserif(size::GREETING)).color(p.text));
            if !sub.is_empty() {
                ui.label(RichText::new(sub).size(size::ROW).color(p.dim));
            }
        });
        ui.with_layout(Layout::right_to_left(Align::Max), add_right);
    });
}

/// One row of the redesigned task list (B-E8).
pub struct TaskLine<'a> {
    pub title: &'a str,
    /// The first line of the notes (may be empty).
    pub notes: &'a str,
    /// "Due today", "Fri 13 Nov", "Waiting · 3 days", "Next".
    pub when: &'a str,
    /// The due date has passed or is today (drawn in terracotta).
    pub urgent: bool,
    /// The roadmap theme (terracotta, 600).
    pub theme: &'a str,
    /// "evidence", "assumption" or "uncertain" (a tag), or empty.
    pub basis: &'a str,
    pub done: bool,
}

/// A group heading in the task list: a dot, the name (14.5, 600) and a note ("2 active").
pub fn group_head(ui: &mut Ui, p: &Pal, dot: Color32, title: &str, note: &str) {
    ui.horizontal(|ui| {
        let (r, _) = ui.allocate_exact_size(egui::vec2(space::S8, space::S8), Sense::hover());
        ui.painter().circle_filled(r.center(), space::S8 / 2.0, dot);
        ui.label(RichText::new(title).font(fs(size::HERO_BODY)).color(p.text));
        ui.label(RichText::new(note).size(size::LABEL).color(p.dim));
    });
}

/// The white box a group's rows sit in (rows are separated by dividers).
pub fn list_box<R>(ui: &mut Ui, p: &Pal, add: impl FnOnce(&mut Ui) -> R) -> R {
    egui::Frame::none().fill(p.card).stroke(Stroke::new(stroke::HAIR, p.line)).rounding(Rounding::same(radius::CARD)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.spacing_mut().item_spacing.y = 0.0;
        add(ui)
    })
    .inner
}

/// A tag for the basis of an item: evidence (green), assumption (amber), uncertain (terracotta).
pub fn basis_tag(ui: &mut Ui, p: &Pal, basis: &str) {
    let (fg, bg) = match basis {
        "evidence" => (p.green_text, p.green_tint),
        "uncertain" => (p.orange, p.orange_tint),
        _ => (p.accent_text, p.accent_soft),
    };
    let g = ui.painter().layout_no_wrap(basis.to_string(), fs(size::TINY), fg);
    let (r, _) = ui.allocate_exact_size(g.size() + egui::vec2(space::S16, space::S4), Sense::hover());
    ui.painter().rect_filled(r, radius::PILL, bg);
    ui.painter().galley(r.min + egui::vec2(space::S8, space::S4 / 2.0), g, fg);
}

/// One task row: the round check (20), the title (14.5), the notes' first line (12.5) and a meta
/// line (when, theme, basis). Returns (toggled, opened).
pub fn task_line(ui: &mut Ui, p: &Pal, t: &TaskLine, first: bool) -> (bool, bool) {
    let w = ui.available_width();
    let top = ui.cursor().min;
    let bg_at = ui.painter().add(Shape::Noop);
    let mut toggled = false;
    let inner = egui::Frame::none().inner_margin(egui::Margin { left: space::S14, right: space::S16, top: space::S12, bottom: space::S12 }).show(ui, |ui| {
        ui.set_width(w - space::S14 - space::S16);
        ui.horizontal_top(|ui| {
            let d = space::MARK + space::S4;
            let (cr, cresp) = ui.allocate_exact_size(egui::vec2(d, d), Sense::click());
            let col = if t.done { p.green } else { p.faint };
            ui.painter().circle_stroke(cr.center(), d / 2.0 - 1.0, Stroke::new(stroke::ICON, col));
            if t.done || cresp.hovered() {
                ui.painter().add(Shape::line(vec![cr.center() + egui::vec2(-4.0, 0.3), cr.center() + egui::vec2(-1.2, 3.0), cr.center() + egui::vec2(4.2, -2.6)], Stroke::new(stroke::ICON, if t.done { p.green } else { p.faint })));
            }
            toggled = cresp.on_hover_text(if t.done { "Reopen" } else { "Complete" }).on_hover_cursor(egui::CursorIcon::PointingHand).clicked();
            ui.add_space(space::S10);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = space::S4 / 2.0;
                let mut title = RichText::new(t.title).size(size::HERO_BODY).color(if t.done { p.dim } else { p.text });
                if t.done {
                    title = title.strikethrough();
                }
                ui.add(egui::Label::new(title).wrap());
                if !t.notes.is_empty() {
                    ui.add(egui::Label::new(RichText::new(t.notes).size(size::SMALL).color(p.dim)).truncate());
                }
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = space::S8;
                    if !t.when.is_empty() {
                        ui.label(RichText::new(t.when).font(fs(size::SMALL - 0.5)).color(if t.urgent { p.orange } else { p.dim }));
                    }
                    if !t.theme.is_empty() {
                        ui.label(RichText::new(t.theme).font(fs(size::SMALL - 0.5)).color(p.orange));
                    }
                    if !t.basis.is_empty() {
                        basis_tag(ui, p, t.basis);
                    }
                });
            });
        });
    });
    let rect = Rect::from_min_size(top, egui::vec2(w, inner.response.rect.height()));
    let resp = ui.interact(rect, inner.response.id.with("task-line"), Sense::click());
    if resp.hovered() {
        ui.painter().set(bg_at, Shape::rect_filled(rect.shrink(stroke::HAIR), radius::CARD - 1.0, p.surface_raised));
    }
    if !first {
        ui.painter().line_segment([rect.left_top(), rect.right_top()], Stroke::new(stroke::HAIR, p.divider));
    }
    (toggled, resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() && !toggled)
}

/// A thin progress bar with its label to the right.
pub fn progress_line(ui: &mut Ui, p: &Pal, frac: f32, width: f32) {
    let (r, _) = ui.allocate_exact_size(egui::vec2(width, space::BAR_H), Sense::hover());
    ui.painter().rect_filled(r, space::BAR_H / 2.0, p.track);
    ui.painter().rect_filled(Rect::from_min_size(r.min, egui::vec2(width * frac.clamp(0.0, 1.0), space::BAR_H)), space::BAR_H / 2.0, tok::AMBER);
}

/// A "Resolve understood" chip in Add task (no emoji): the task itself (white, "Task <b>title</b>"),
/// or a recognised part tinted by what it is.
pub fn understood_chip(ui: &mut Ui, p: &Pal, lead: &str, text: &str, kind: Option<crate::nlp::Recognised>) {
    use crate::nlp::Recognised;
    let (bg, fg, edge) = match kind {
        None => (p.card, p.text, p.line),
        Some(Recognised::Priority) => (p.orange_tint, p.orange, p.orange_tint),
        Some(Recognised::Project) => (p.green_tint, p.green_text, p.green_tint),
        Some(_) => (p.accent_soft, p.text, if p.dark { p.line } else { tok::AMBER_EDGE }),
    };
    let mut job = egui::text::LayoutJob::default();
    if !lead.is_empty() {
        job.append(&format!("{lead} "), 0.0, egui::TextFormat { font_id: fp(size::LABEL), color: p.dim, ..Default::default() });
    }
    job.append(text, 0.0, egui::TextFormat { font_id: fs(size::LABEL), color: fg, ..Default::default() });
    let g = ui.painter().layout_job(job);
    let (r, _) = ui.allocate_exact_size(g.size() + egui::vec2(space::S22, space::S14), Sense::hover());
    ui.painter().rect(r, radius::SMALL_BUTTON, bg, Stroke::new(stroke::HAIR, edge));
    ui.painter().galley(r.min + egui::vec2(space::S22 / 2.0, space::S14 / 2.0), g, fg);
}

/// One row of Plan my day: the time (92 wide), a 4 px bar (amber task, terracotta event or
/// overdue), the item and why it sits there, and its tag. The first row is bold on a warm white.
pub fn plan_row(ui: &mut Ui, p: &Pal, time: &str, item: &str, why: &str, tag: &str, first: bool) {
    let w = ui.available_width();
    let top = ui.cursor().min;
    let bg_at = ui.painter().add(Shape::Noop);
    let (bar, tag_bg, tag_fg) = match tag {
        "Event" => (p.orange, p.orange_tint, p.orange),
        "Overdue" => (p.orange, p.orange_tint, p.orange),
        _ => (tok::AMBER, p.accent_soft, p.accent_text),
    };
    let inner = egui::Frame::none().inner_margin(egui::Margin::symmetric(space::S16, space::S12)).show(ui, |ui| {
        ui.set_width(w - space::S16 * 2.0);
        ui.horizontal_top(|ui| {
            ui.add_sized([space::PLAN_TIME_W, size::ROW + space::S4], egui::Label::new(RichText::new(time).size(size::LABEL).color(p.dim)));
            let (r, _) = ui.allocate_exact_size(egui::vec2(stroke::DOT, space::S40 - space::S4), Sense::hover());
            ui.painter().rect_filled(r, stroke::DOT / 2.0, bar);
            ui.add_space(space::S6);
            let tag_g = ui.painter().layout_no_wrap(tag.to_string(), fs(size::TINY), tag_fg);
            let text_w = ui.available_width() - tag_g.size().x - space::S30;
            ui.vertical(|ui| {
                ui.set_width(text_w);
                ui.add(egui::Label::new(RichText::new(item).font(if first { fs(size::ROW) } else { fm(size::ROW) }).color(p.text)).wrap());
                if !why.is_empty() {
                    ui.add(egui::Label::new(RichText::new(why).size(size::SMALL).color(p.dim)).wrap());
                }
            });
            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                let (r, _) = ui.allocate_exact_size(tag_g.size() + egui::vec2(space::S16, space::S4), Sense::hover());
                ui.painter().rect_filled(r, radius::PILL, tag_bg);
                ui.painter().galley(r.min + egui::vec2(space::S8, space::S4 / 2.0), tag_g, tag_fg);
            });
        });
    });
    let rect = Rect::from_min_size(top, egui::vec2(w, inner.response.rect.height()));
    if first {
        ui.painter().set(bg_at, Shape::rect_filled(rect.shrink(stroke::HAIR), radius::CARD - 1.0, if p.dark { p.surface_raised } else { tok::AMBER_LIGHT }));
    } else {
        ui.painter().line_segment([rect.left_top(), rect.right_top()], Stroke::new(stroke::HAIR, p.divider));
    }
}

/// A line led by a small dot ("Focus theme · …" on the hero).
pub fn dot_line(ui: &mut Ui, text: &str, dot: Color32, col: Color32) {
    ui.horizontal(|ui| {
        let (r, _) = ui.allocate_exact_size(egui::vec2(space::S8, space::S8), Sense::hover());
        ui.painter().circle_filled(r.center(), 3.5, dot);
        ui.label(RichText::new(text).size(size::SMALL).color(col));
    });
}