//! Toasts, one at a time (ticket B-E10). A new toast replaces the one showing with a 160 ms
//! swap. With Undo it stays 4 s with an amber timer bar; plain toasts stay 2.6 s. In 360 ms
//! e-spring (rising 20 px), out 260 ms e-in (sinking 16 px). Placed 10 px above the Ask bar,
//! centred on the work area. Code anywhere still calls `app.toast(..)` or pushes to
//! `undo_toasts`; those lists are an inbox the toaster drains every frame.

use super::motion::{self, frame_at};
use super::theme::*;
use super::widgets::{moves, paint_toast};
use crate::model::Id;
use eframe::egui::{self, Align2, Rect};

#[derive(Clone, Debug, PartialEq)]
pub struct Toast {
    pub text: String,
    /// The change set Undo reverses (None for a plain toast).
    pub undo: Option<Id>,
    /// When it arrived (app clock, seconds).
    pub born: f64,
}

impl Toast {
    /// How long it stays, in ms.
    pub fn life_ms(&self) -> f32 {
        if self.undo.is_some() {
            moves::TOAST_UNDO_MS
        } else {
            moves::TOAST_PLAIN_MS
        }
    }
}

/// What is on screen: the current toast, and the one leaving (swapped out or timed out).
#[derive(Default)]
pub struct Toaster {
    pub current: Option<Toast>,
    /// (toast, when it started leaving, swapped out by a newer one?)
    pub leaving: Option<(Toast, f64, bool)>,
    /// The work area (to the right of the sidebar), set each frame.
    pub work: Option<Rect>,
}

impl Toaster {
    /// Show a new toast now, replacing whatever is showing (it swaps out in 160 ms).
    pub fn push(&mut self, now: f64, text: String, undo: Option<Id>) {
        if let Some(c) = self.current.take() {
            self.leaving = Some((c, now, true));
        }
        self.current = Some(Toast { text, undo, born: now });
    }

    /// Move timed-out toasts to "leaving" and forget finished ones.
    pub fn tick(&mut self, now: f64) {
        if let Some(c) = &self.current {
            if (now - c.born) * 1000.0 >= c.life_ms() as f64 {
                let c = self.current.take().unwrap();
                self.leaving = Some((c, now, false));
            }
        }
        if let Some((_, since, swap)) = &self.leaving {
            let ms = if *swap { moves::TOAST_SWAP.ms } else { moves::TOAST_OUT.ms };
            if (now - since) * 1000.0 > ms as f64 + 20.0 {
                self.leaving = None;
            }
        }
    }

    pub fn busy(&self) -> bool {
        self.current.is_some() || self.leaving.is_some()
    }
}

impl super::App {
    /// Take new toasts from the inbox (the newest wins; older ones in the same frame are
    /// replaced at once, as the brief asks), then draw.
    pub(super) fn toasts_ui(&mut self, ctx: &egui::Context, _p: &Pal) {
        let now = self.now;
        // newest last: plain toasts and undo toasts pushed this frame, in the order they came
        let mut incoming: Vec<(String, Option<Id>)> = vec![];
        for (msg, _) in self.toasts.drain(..) {
            incoming.push((msg, None));
        }
        for (msg, _, cs) in self.undo_toasts.drain(..) {
            incoming.push((msg, Some(cs)));
        }
        for (msg, cs) in incoming {
            self.toaster.push(now, msg, cs);
        }
        self.toaster.tick(now);
        if !self.toaster.busy() {
            return;
        }
        ctx.request_repaint();
        let work = self.toaster.work.filter(|r| r.is_positive()).unwrap_or_else(|| ctx.screen_rect());
        let bottom = work.bottom() - (space::ASK_BOTTOM + space::ASK_H + space::TOAST_GAP);
        let anchor = egui::pos2(work.center().x, bottom);
        let reduced = motion::reduced();
        let mut undo: Option<Id> = None;
        // the leaving one first (under the new one)
        if let Some((t, since, swap)) = self.toaster.leaving.clone() {
            let tm = if swap { moves::TOAST_SWAP } else { moves::TOAST_OUT };
            let f = frame_at(((now - since) * 1000.0) as f32, tm, reduced);
            let y = if reduced { 0.0 } else { 16.0 * f.t };
            let alpha = 1.0 - f.alpha;
            let left = 1.0 - (((since - t.born) * 1000.0) as f32 / t.life_ms()).clamp(0.0, 1.0);
            show_toast(ctx, egui::Id::new(("toast-leaving", t.born.to_bits())), anchor + egui::vec2(0.0, y), alpha, &t, left);
        }
        if let Some(t) = self.toaster.current.clone() {
            let el = ((now - t.born) * 1000.0) as f32;
            let f = frame_at(el, moves::TOAST_IN, reduced);
            let y = if reduced { 0.0 } else { 20.0 * (1.0 - f.t) };
            let left = 1.0 - (el / t.life_ms()).clamp(0.0, 1.0);
            if show_toast(ctx, egui::Id::new(("toast", t.born.to_bits())), anchor + egui::vec2(0.0, y), f.alpha, &t, left) {
                undo = t.undo.clone();
                // Undo pressed: the toast leaves at once
                let c = self.toaster.current.take().unwrap();
                self.toaster.leaving = Some((c, now, false));
            }
        }
        if let Some(cs) = undo {
            self.undo(&cs);
        }
    }
}

/// Draw one toast with its bottom centre at `anchor`. Returns true when Undo is pressed.
fn show_toast(ctx: &egui::Context, id: egui::Id, anchor: egui::Pos2, alpha: f32, t: &Toast, timer: f32) -> bool {
    let mut pressed = false;
    egui::Area::new(id).order(egui::Order::Tooltip).fixed_pos(egui::pos2(anchor.x - space::TOAST_W / 2.0, anchor.y)).pivot(Align2::LEFT_BOTTOM).interactable(true).show(ctx, |ui| {
        ui.multiply_opacity(alpha.clamp(0.0, 1.0));
        pressed = paint_toast(ui, &t.text, t.undo.is_some(), timer);
    });
    let _ = Rect::NOTHING;
    pressed
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn one_at_a_time() {
        let mut t = Toaster::default();
        t.push(0.0, "a".into(), None);
        t.push(0.1, "b".into(), Some("cs".into()));
        assert_eq!(t.current.as_ref().unwrap().text, "b");
        assert!(t.leaving.as_ref().unwrap().2);
    }
}
