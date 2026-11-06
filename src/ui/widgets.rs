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

/// A 🎤 button; returns true when pressed (caller focuses its text box).
pub fn mic_button(ui: &mut Ui, p: &Pal) -> bool {
    let r = ui.add(egui::Button::new(RichText::new("🎤").size(15.0).color(p.accent)).frame(false)).on_hover_text("Speak instead of typing (Windows voice typing, Win+H)");
    if r.clicked() {
        start_voice_typing();
        true
    } else {
        false
    }
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
