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
    let inner = egui::Frame::none().fill(tok::INK).stroke(edge).rounding(Rounding::same(radius::HERO)).inner_margin(egui::Margin::same(space::S24)).show(ui, |ui| {
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
}
