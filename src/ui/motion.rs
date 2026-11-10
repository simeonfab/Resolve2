//! The motion helper (tickets E16, B-E18, C-E26). egui 0.29 has no easing, timelines or
//! transforms for widgets, so every animated moment in the app goes through here:
//!
//! - cubic-bezier curves (the brief's e-spring, e-out, e-std, e-in, ease),
//! - timings with delay and stagger (delay + index × stagger, in reading order),
//! - keyframe tracks (one clock driving many values, like the bridge and the reveal),
//! - value tweens for widths and heights (sidebar 248 ↔ 64, rail 308 ↔ 72, a card growing),
//! - origin-aware scale (menus grow from their anchor, dialogs from their centre),
//! - loops (the mic pulse, the waiting ring, the hero glow),
//! - the app-wide reduced-motion flag: every move becomes a fade of 120 ms or less and loops
//!   hold still.
//!
//! While anything moves, the helper asks egui to repaint. Times are milliseconds; the clock is
//! egui's `input.time` (seconds).

use eframe::egui::{self, emath::TSTransform, Context, Id, LayerId, Pos2, Rect, Vec2};
use std::sync::atomic::{AtomicBool, Ordering};

static REDUCED: AtomicBool = AtomicBool::new(false);

/// Turn reduced motion on or off for the whole app.
pub fn set_reduced(on: bool) {
    REDUCED.store(on, Ordering::Relaxed);
}

pub fn reduced() -> bool {
    REDUCED.load(Ordering::Relaxed)
}

/// With reduced motion, a move is a fade no longer than this.
pub const REDUCED_FADE_MS: f32 = 120.0;

/// A cubic-bezier easing curve, as CSS writes it: cubic-bezier(x1, y1, x2, y2).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Curve(pub f32, pub f32, pub f32, pub f32);

/// Arrivals, expansions, sheets, panels.
pub const E_SPRING: Curve = Curve(0.34, 1.56, 0.64, 1.0);
/// Reveals, restores, settles.
pub const E_OUT: Curve = Curve(0.22, 0.61, 0.36, 1.0);
/// Dims, presses, colour changes.
pub const E_STD: Curve = Curve(0.4, 0.0, 0.2, 1.0);
/// Exits, collapses.
pub const E_IN: Curve = Curve(0.55, 0.0, 0.85, 0.35);
/// Only where the brief notes it.
pub const EASE: Curve = Curve(0.25, 0.1, 0.25, 1.0);
pub const LINEAR: Curve = Curve(0.0, 0.0, 1.0, 1.0);

/// (name, curve) for the gallery.
pub const CURVES: [(&str, Curve); 5] = [("e-spring", E_SPRING), ("e-out", E_OUT), ("e-std", E_STD), ("e-in", E_IN), ("ease", EASE)];

impl Curve {
    fn bez(a: f32, b: f32, t: f32) -> f32 {
        // one coordinate of a cubic bezier from (0,0) to (1,1) with control values a and b
        let u = 1.0 - t;
        3.0 * u * u * t * a + 3.0 * u * t * t * b + t * t * t
    }
    fn bez_dx(a: f32, b: f32, t: f32) -> f32 {
        let u = 1.0 - t;
        3.0 * u * u * a + 6.0 * u * t * (b - a) + 3.0 * t * t * (1.0 - b)
    }
    /// The eased value at progress `x` (0..1). May go past 1 for e-spring (that's the overshoot).
    pub fn at(self, x: f32) -> f32 {
        let x = x.clamp(0.0, 1.0);
        if x <= 0.0 || x >= 1.0 {
            return x;
        }
        let Curve(x1, y1, x2, y2) = self;
        // solve bez_x(t) = x: Newton, then bisection if Newton stalls
        let mut t = x;
        for _ in 0..8 {
            let err = Self::bez(x1, x2, t) - x;
            if err.abs() < 1e-5 {
                return Self::bez(y1, y2, t);
            }
            let d = Self::bez_dx(x1, x2, t);
            if d.abs() < 1e-6 {
                break;
            }
            t = (t - err / d).clamp(0.0, 1.0);
        }
        let (mut lo, mut hi) = (0.0f32, 1.0f32);
        t = x;
        for _ in 0..30 {
            let v = Self::bez(x1, x2, t);
            if (v - x).abs() < 1e-5 {
                break;
            }
            if v < x {
                lo = t;
            } else {
                hi = t;
            }
            t = (lo + hi) / 2.0;
        }
        Self::bez(y1, y2, t)
    }
}

/// One timed move: duration and delay in ms, and its curve.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Timing {
    pub ms: f32,
    pub delay: f32,
    pub curve: Curve,
}

pub const fn timing(ms: f32, delay: f32, curve: Curve) -> Timing {
    Timing { ms, delay, curve }
}

impl Timing {
    /// The same move for item `index` of a list: delay + index × stagger.
    pub fn stagger(self, index: usize, stagger_ms: f32) -> Timing {
        Timing { delay: self.delay + index as f32 * stagger_ms, ..self }
    }
    /// When this move ends (delay + duration), in ms.
    pub fn end(self) -> f32 {
        self.delay + self.ms
    }
}

/// Where a move is at one moment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
    /// Eased progress for position, size and scale (0 → 1; e-spring overshoots).
    /// With reduced motion this is always 1: things appear in place.
    pub t: f32,
    /// Opacity progress (0 → 1). With reduced motion, a fade of 120 ms or less.
    pub alpha: f32,
    /// Has it finished?
    pub done: bool,
}

impl Frame {
    /// A value moving from `a` to `b`.
    pub fn lerp(self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.t
    }
    pub fn lerp_vec(self, a: Vec2, b: Vec2) -> Vec2 {
        a + (b - a) * self.t
    }
}

/// Progress of a move that has been running for `elapsed_ms`. Pure: no egui, so it can be tested.
pub fn frame_at(elapsed_ms: f32, tm: Timing, reduced: bool) -> Frame {
    let e = elapsed_ms - tm.delay;
    if reduced {
        let fade = tm.ms.min(REDUCED_FADE_MS).max(1.0);
        let a = (e / fade).clamp(0.0, 1.0);
        return Frame { t: 1.0, alpha: a, done: e >= fade };
    }
    if tm.ms <= 0.0 {
        return Frame { t: 1.0, alpha: 1.0, done: true };
    }
    let raw = (e / tm.ms).clamp(0.0, 1.0);
    Frame { t: tm.curve.at(raw), alpha: raw, done: raw >= 1.0 }
}

fn now(ctx: &Context) -> f64 {
    ctx.input(|i| i.time)
}

/// When the animation `id` started (set on first use).
pub fn started(ctx: &Context, id: Id) -> f64 {
    let t = now(ctx);
    ctx.data_mut(|d| *d.get_temp_mut_or_insert_with(id.with("motion-start"), || t))
}

/// Start (or restart) the animation `id` now.
pub fn restart(ctx: &Context, id: Id) {
    let t = now(ctx);
    ctx.data_mut(|d| d.insert_temp(id.with("motion-start"), t));
}

/// Milliseconds since `id` started.
pub fn elapsed_ms(ctx: &Context, id: Id) -> f32 {
    ((now(ctx) - started(ctx, id)) * 1000.0) as f32
}

/// Where the move `id` is now. Asks for a repaint until it has finished.
pub fn play(ctx: &Context, id: Id, tm: Timing) -> Frame {
    let f = frame_at(elapsed_ms(ctx, id), tm, reduced());
    if !f.done {
        ctx.request_repaint();
    }
    f
}

/// A tween toward `target`: when the target changes, the value moves there from wherever it
/// was. Used for widths and heights (sidebar collapse, rail collapse, a card growing). With
/// reduced motion the value jumps (the fade is drawn by whoever owns the content).
pub fn tween(ctx: &Context, id: Id, target: f32, tm: Timing) -> f32 {
    let key = id.with("tween");
    let t = now(ctx);
    let (from, to, start) = ctx.data_mut(|d| *d.get_temp_mut_or_insert_with(key, || (target, target, t - 10.0)));
    let current = {
        let el = ((t - start) * 1000.0) as f32;
        frame_at(el, Timing { delay: 0.0, ..tm }, false).lerp(from, to)
    };
    if (to - target).abs() > f32::EPSILON {
        // new target: start from where we are now
        ctx.data_mut(|d| d.insert_temp(key, (current, target, t)));
        if reduced() {
            ctx.data_mut(|d| d.insert_temp(key, (target, target, t - 10.0)));
            return target;
        }
        ctx.request_repaint();
        return current;
    }
    if reduced() {
        return to;
    }
    let el = ((t - start) * 1000.0) as f32;
    let f = frame_at(el, Timing { delay: 0.0, ..tm }, false);
    if !f.done {
        ctx.request_repaint();
    }
    f.lerp(from, to)
}

/// A keyframe on a track: at `at_ms` the value is `v`; the segment that ends here uses `curve`.
#[derive(Clone, Copy, Debug)]
pub struct Key {
    pub at_ms: f32,
    pub v: f32,
    pub curve: Curve,
}

pub const fn key(at_ms: f32, v: f32, curve: Curve) -> Key {
    Key { at_ms, v, curve }
}

/// The value of a keyframe track at `ms` (holds the first value before the first key and the
/// last after the last). With reduced motion, the track jumps to its last value.
pub fn track(keys: &[Key], ms: f32, reduced: bool) -> f32 {
    let Some(first) = keys.first() else { return 0.0 };
    let last = keys[keys.len() - 1];
    if reduced || ms >= last.at_ms {
        return last.v;
    }
    if ms <= first.at_ms {
        return first.v;
    }
    for w in keys.windows(2) {
        let (a, b) = (w[0], w[1]);
        if ms >= a.at_ms && ms <= b.at_ms {
            let span = (b.at_ms - a.at_ms).max(1.0);
            let t = b.curve.at((ms - a.at_ms) / span);
            return a.v + (b.v - a.v) * t;
        }
    }
    last.v
}

/// Phase (0..1) of a repeating loop of `period_ms`. Holds still (0) with reduced motion.
pub fn loop_phase(ctx: &Context, period_ms: f32) -> f32 {
    if reduced() {
        return 0.0;
    }
    ctx.request_repaint();
    let ms = (now(ctx) * 1000.0) as f32;
    (ms % period_ms) / period_ms
}

/// Scale a rect about an origin point (a menu grows from its anchor, a dialog from its centre).
pub fn scale_about(rect: Rect, origin: Pos2, s: f32) -> Rect {
    Rect::from_min_max(origin + (rect.min - origin) * s, origin + (rect.max - origin) * s)
}

/// Move and scale a whole layer (an Area, a dialog, a sheet) about an origin. egui scales
/// everything drawn in the layer, text included.
pub fn transform_layer(ctx: &Context, layer: LayerId, origin: Pos2, scale: f32, offset: Vec2) {
    // p' = origin + (p - origin) * s + offset = p*s + (origin*(1-s) + offset)
    let translation = origin.to_vec2() * (1.0 - scale) + offset;
    ctx.set_transform_layer(layer, TSTransform::new(translation, scale));
}

/// An entrance used all over the app: fade in while moving from `from` (an offset) and scaling
/// from `scale_from`, about `origin`. Returns (offset, scale, opacity) for this frame.
pub fn entrance(f: Frame, from: Vec2, scale_from: f32) -> (Vec2, f32, f32) {
    (from * (1.0 - f.t), scale_from + (1.0 - scale_from) * f.t, f.alpha)
}

/// Draw `add` faded and shifted by an entrance (`offset` is where it starts; it settles at 0).
/// The space it takes in the layout never moves, so nothing around it jumps. Scale needs a
/// layer (see `transform_layer`).
pub fn fade_in<R>(ui: &mut egui::Ui, f: Frame, offset: Vec2, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let (off, _, alpha) = entrance(f, offset, 1.0);
    let start = ui.cursor().min;
    let avail = ui.available_size_before_wrap();
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(Rect::from_min_size(start + off, avail)).layout(*ui.layout()));
    child.multiply_opacity(alpha.clamp(0.0, 1.0));
    let r = add(&mut child);
    let used = child.min_rect();
    ui.advance_cursor_after_rect(Rect::from_min_size(start, used.size()));
    r
}

/// The same as `fade_in`, in two halves, for long blocks: draw into the returned child, then call
/// `end_move`. `keep` (0..1) is how much of the block's height stays in the layout (1 normally;
/// less while a row collapses away).
pub fn begin_move(ui: &mut egui::Ui, f: Frame, offset: Vec2) -> (egui::Ui, Pos2) {
    let (off, _, alpha) = entrance(f, offset, 1.0);
    let start = ui.cursor().min;
    let avail = ui.available_size_before_wrap();
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(Rect::from_min_size(start + off, avail)).layout(*ui.layout()));
    child.multiply_opacity(alpha.clamp(0.0, 1.0));
    (child, start)
}

pub fn end_move(ui: &mut egui::Ui, child: &egui::Ui, start: Pos2, keep: f32) {
    let size = child.min_rect().size();
    ui.advance_cursor_after_rect(Rect::from_min_size(start, Vec2::new(size.x, size.y * keep.clamp(0.0, 1.0))));
}

/// A finished move (nothing to animate).
pub const SETTLED: Frame = Frame { t: 1.0, alpha: 1.0, done: true };

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn curves_hit_their_ends() {
        for (_, c) in CURVES {
            assert!((c.at(0.0)).abs() < 1e-4);
            assert!((c.at(1.0) - 1.0).abs() < 1e-4);
        }
    }
}
