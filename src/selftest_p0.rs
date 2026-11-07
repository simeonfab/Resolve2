//! Phase 0 checks (redesign foundations): tokens match the brief, the contrast rules hold, the
//! motion helper's curves, timings, stagger, keyframes and reduced-motion fades behave as
//! written, toasts show one at a time with the right lifetimes, and no view file holds a colour
//! literal.

use crate::selftest::T;
use crate::ui::motion::{self, frame_at, key, timing, CURVES, E_IN, E_OUT, E_SPRING, E_STD, LINEAR};
use crate::ui::theme::{size, tok};
use crate::ui::toast::Toaster;
use crate::ui::widgets::moves;
use eframe::egui::Color32;

fn hex(c: Color32) -> String {
    format!("#{:02X}{:02X}{:02X}", c.r(), c.g(), c.b())
}

/// WCAG relative luminance contrast ratio.
fn contrast(a: Color32, b: Color32) -> f32 {
    let lum = |c: Color32| {
        let f = |v: u8| {
            let s = v as f32 / 255.0;
            if s <= 0.03928 { s / 12.92 } else { ((s + 0.055) / 1.055).powf(2.4) }
        };
        0.2126 * f(c.r()) + 0.7152 * f(c.g()) + 0.0722 * f(c.b())
    };
    let (x, y) = (lum(a), lum(b));
    (x.max(y) + 0.05) / (x.min(y) + 0.05)
}

pub fn run(t: &mut T) {
    println!("\n-- Phase 0: foundations --");

    // tokens: every value in the brief's table, by name
    let wrong: Vec<String> = tok::ALL.iter().filter(|(_, c, h)| hex(*c) != *h).map(|(n, c, h)| format!("{n} is {} not {h}", hex(*c))).collect();
    t.check("the 24 colour tokens match the brief exactly", wrong.is_empty() && tok::ALL.len() == 24, wrong.join("; "));
    t.check("type: HERO 52, QUESTION 30, ANSWER 15 (resized as the brief asks)", size::HERO == 52.0 && size::QUESTION == 30.0 && size::ANSWER == 15.0, format!("{} {} {}", size::HERO, size::QUESTION, size::ANSWER));

    // contrast: the pairs in Tokens, and no amber text on light
    let light = crate::ui::theme::pal(false);
    let dark = crate::ui::theme::pal(true);
    t.check("on light, amber meaning is drawn as amber-text, not amber", light.accent_text == tok::AMBER_TEXT && light.accent_text != tok::AMBER, hex(light.accent_text));
    t.check("on light, green text is #3F5A33 on #E8EFE3", light.green_text == tok::GREEN_TEXT && light.green_tint == tok::GREEN_TINT, hex(light.green_text));
    t.check("on dark, terracotta is #C97A4A and the smallest text #8A8578", dark.orange == tok::TERRACOTTA_ON_DARK && dark.faint == tok::TEXT_ON_DARK_MUTED, format!("{} {}", hex(dark.orange), hex(dark.faint)));
    let pairs = [("amber-text on amber-light", tok::AMBER_TEXT, tok::AMBER_LIGHT), ("green-text on green-tint", tok::GREEN_TEXT, tok::GREEN_TINT), ("terracotta on terracotta-tint", tok::TERRACOTTA, tok::TERRACOTTA_TINT), ("ink on surface", tok::INK, tok::SURFACE), ("ink-soft on card", tok::INK_SOFT, tok::CARD), ("text-on-dark on ink", tok::TEXT_ON_DARK, tok::INK)];
    let weak: Vec<String> = pairs.iter().map(|(n, a, b)| (n, contrast(*a, *b))).filter(|(_, r)| *r < 4.5).map(|(n, r)| format!("{n} {r:.2}")).collect();
    t.check("text pairs pass WCAG AA (4.5:1)", weak.is_empty(), weak.join("; "));
    t.check("amber on light would fail (why it is never text there)", contrast(tok::AMBER, tok::SURFACE) < 3.0, format!("{:.2}", contrast(tok::AMBER, tok::SURFACE)));

    // no colour literals in view files (only theme.rs and widgets.rs may hold them)
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join("ui");
    let mut offenders = vec![];
    if let Ok(rd) = std::fs::read_dir(&dir) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if !name.ends_with(".rs") || name == "theme.rs" || name == "widgets.rs" {
                continue;
            }
            let text = std::fs::read_to_string(e.path()).unwrap_or_default();
            if text.contains("Color32::from_rgb") || text.contains("Color32::from_rgba") || text.contains("Color32::from_gray") {
                offenders.push(name);
            }
        }
    }
    t.check("no colour literal in any view file", offenders.is_empty(), offenders.join(", "));

    // motion: curves, timings, stagger, keyframes, reduced motion
    let ends = CURVES.iter().all(|(_, c)| c.at(0.0).abs() < 1e-3 && (c.at(1.0) - 1.0).abs() < 1e-3);
    t.check("every curve starts at 0 and ends at 1", ends, "");
    let peak = (1..100).map(|i| E_SPRING.at(i as f32 / 100.0)).fold(0.0f32, f32::max);
    t.check("e-spring overshoots (cubic-bezier .34, 1.56, .64, 1)", peak > 1.05, format!("peak {peak:.3}"));
    t.check("e-in starts slowly and e-out fast", E_IN.at(0.25) < 0.1 && E_OUT.at(0.25) > 0.45, format!("{:.2} {:.2}", E_IN.at(0.25), E_OUT.at(0.25)));
    let st = timing(300.0, 0.0, E_OUT).stagger(3, 60.0);
    t.check("stagger = delay + index × stagger", (st.delay - 180.0).abs() < 1e-3, st.delay);
    let f = frame_at(150.0, timing(300.0, 100.0, LINEAR), false);
    t.check("a timed move respects its delay", (f.t - 1.0 / 6.0).abs() < 1e-3 && !f.done, f.t);
    let r = frame_at(60.0, timing(460.0, 0.0, E_SPRING), true);
    let r_end = frame_at(120.0, timing(460.0, 0.0, E_SPRING), true);
    t.check("reduced motion: a move is a fade of 120 ms or less, no movement", r.t == 1.0 && (r.alpha - 0.5).abs() < 1e-3 && r_end.done, format!("{r:?}"));
    let keys = [key(0.0, 0.35, LINEAR), key(2116.0, 0.35, LINEAR), key(2760.0, 0.85, E_STD), key(9200.0, 0.35, E_IN)];
    t.check("keyframe tracks hold, ease between keys, and jump to the end with reduced motion", motion::track(&keys, 1000.0, false) == 0.35 && motion::track(&keys, 2760.0, false) == 0.85 && motion::track(&keys, 0.0, true) == 0.35, "");
    let s = motion::scale_about(eframe::egui::Rect::from_min_size(eframe::egui::pos2(10.0, 10.0), eframe::egui::vec2(100.0, 50.0)), eframe::egui::pos2(10.0, 10.0), 0.9);
    t.check("origin-aware scale keeps the anchor still", s.min == eframe::egui::pos2(10.0, 10.0) && (s.width() - 90.0).abs() < 1e-3, format!("{s:?}"));

    // toasts: one at a time, swap, lifetimes
    let mut q = Toaster::default();
    q.push(0.0, "first".into(), None);
    q.push(0.05, "second".into(), Some("cs".into()));
    t.check("toasts: a new one replaces the one showing (swap 160 ms)", q.current.as_ref().map(|c| c.text.as_str()) == Some("second") && q.leaving.as_ref().map_or(false, |l| l.2) && moves::TOAST_SWAP.ms == 160.0, "");
    t.check("toasts: Undo stays 4 s, plain 2.6 s", q.current.as_ref().map(|c| c.life_ms()) == Some(4000.0) && moves::TOAST_PLAIN_MS == 2600.0, "");
    q.tick(4.2);
    t.check("toasts: an Undo toast leaves after 4 s", q.current.is_none() && q.leaving.is_some(), "");
    q.tick(5.0);
    t.check("toasts: nothing is left showing after it has gone", !q.busy(), "");
}
