use eframe::egui::{self, Color32, FontData, FontDefinitions, FontFamily, FontId, Rounding, Stroke, TextStyle};

// Resolve 2 design tokens: the only place colours, type sizes, spacing, radii and shadows are
// defined (build brief 7 Nov 2026, "Tokens"). Views read them through `Pal` (light or dark) or
// directly from `tok`, `size`, `space`, `radius` and `shadow`. No other hues exist in the app.

/// The brief's named colour tokens, exactly as written there.
pub mod tok {
    use eframe::egui::Color32;
    const fn hex(v: u32) -> Color32 {
        Color32::from_rgb((v >> 16) as u8, (v >> 8) as u8, v as u8)
    }
    /// Sidebar, intake top bar and rail, front door, phone "not connected".
    pub const NAV_BG: Color32 = hex(0x1E1E1A);
    /// Text on light, dark hero cards, primary dark buttons, Ask bar.
    pub const INK: Color32 = hex(0x1A1A18);
    /// Cards and fields on dark, phone intake.
    pub const PANE: Color32 = hex(0x201F1C);
    /// Work area.
    pub const SURFACE: Color32 = hex(0xF2EDE4);
    /// Cards, fields, light phone bar.
    pub const CARD: Color32 = hex(0xFFFFFF);
    /// Card and field edges.
    pub const BORDER: Color32 = hex(0xDDD8CE);
    /// Accent, CTA fill, focus ring, active nav icon. Never text on light.
    pub const AMBER: Color32 = hex(0xC8A040);
    /// Assumption tag, highlights.
    pub const AMBER_LIGHT: Color32 = hex(0xFBF5E6);
    /// Amber meaning as text, on amber-light.
    pub const AMBER_TEXT: Color32 = hex(0x7A5B12);
    /// Secondary text.
    pub const INK_SOFT: Color32 = hex(0x4A4A44);
    /// Disabled and decoration only.
    pub const INK_GHOST: Color32 = hex(0x9A9A92);
    /// Eyebrows, links, errors on light, Work calendar.
    pub const TERRACOTTA: Color32 = hex(0x8B4A2A);
    /// Waiting, priority, error panels on light.
    pub const TERRACOTTA_TINT: Color32 = hex(0xF5E8E1);
    /// Errors on dark.
    pub const TERRACOTTA_ON_DARK: Color32 = hex(0xC97A4A);
    /// Evidence marks, Personal calendar, saved dot.
    pub const GREEN: Color32 = hex(0x5E7D4F);
    /// Evidence tag text, Supported status (on green-tint).
    pub const GREEN_TEXT: Color32 = hex(0x3F5A33);
    pub const GREEN_TINT: Color32 = hex(0xE8EFE3);
    /// Text on dark.
    pub const TEXT_ON_DARK: Color32 = hex(0xF5F1E8);
    /// Secondary text on dark.
    pub const TEXT_ON_DARK_SOFT: Color32 = hex(0xB4AFA3);
    /// Smallest text on dark.
    pub const TEXT_ON_DARK_MUTED: Color32 = hex(0x8A8578);
    /// Sheet backgrounds, dialog footers, hovered rows.
    pub const SURFACE_RAISED: Color32 = hex(0xF7F3EC);
    /// Board and roadmap columns.
    pub const SURFACE_SUNK: Color32 = hex(0xEAE4D9);
    /// Segmented control track, progress track, disabled button.
    pub const TRACK: Color32 = hex(0xE4DED2);
    /// Rows inside a card.
    pub const DIVIDER: Color32 = hex(0xEEE9DF);

    /// Derived dark tints of pane (the brief draws dark surfaces as pane and ink only; these
    /// lift a card, a hovered row and a track a little above pane, in the same hue).
    pub const PANE_RAISED: Color32 = hex(0x292723);
    pub const PANE_HOVER: Color32 = hex(0x2F2D28);
    pub const PANE_TRACK: Color32 = hex(0x37352F);
    /// The brief's translucent fills, worked out as the opaque colour a browser shows (egui
    /// blends translucent fills brighter than CSS does, so flat fills use these):
    /// rgba(200,160,64,.16) on nav-bg (the sidebar's active item).
    pub const AMBER_16_ON_NAV: Color32 = hex(0x393320);
    /// rgba(200,160,64,.10) on pane (callouts on dark).
    pub const AMBER_10_ON_PANE: Color32 = hex(0x312C20);
    /// rgba(139,74,42,.22) on pane (error panels on dark).
    pub const TERRACOTTA_22_ON_PANE: Color32 = hex(0x38291F);

    /// Every token with its name and value, for the component gallery.
    pub const ALL: [(&str, Color32, &str); 24] = [
        ("nav-bg", NAV_BG, "#1E1E1A"),
        ("ink", INK, "#1A1A18"),
        ("pane", PANE, "#201F1C"),
        ("surface", SURFACE, "#F2EDE4"),
        ("card", CARD, "#FFFFFF"),
        ("border", BORDER, "#DDD8CE"),
        ("amber", AMBER, "#C8A040"),
        ("amber-light", AMBER_LIGHT, "#FBF5E6"),
        ("amber-text", AMBER_TEXT, "#7A5B12"),
        ("ink-soft", INK_SOFT, "#4A4A44"),
        ("ink-ghost", INK_GHOST, "#9A9A92"),
        ("terracotta", TERRACOTTA, "#8B4A2A"),
        ("terracotta-tint", TERRACOTTA_TINT, "#F5E8E1"),
        ("terracotta-on-dark", TERRACOTTA_ON_DARK, "#C97A4A"),
        ("green", GREEN, "#5E7D4F"),
        ("green-text", GREEN_TEXT, "#3F5A33"),
        ("green-tint", GREEN_TINT, "#E8EFE3"),
        ("text-on-dark", TEXT_ON_DARK, "#F5F1E8"),
        ("text-on-dark-soft", TEXT_ON_DARK_SOFT, "#B4AFA3"),
        ("text-on-dark-muted", TEXT_ON_DARK_MUTED, "#8A8578"),
        ("surface-raised", SURFACE_RAISED, "#F7F3EC"),
        ("surface-sunk", SURFACE_SUNK, "#EAE4D9"),
        ("track", TRACK, "#E4DED2"),
        ("divider", DIVIDER, "#EEE9DF"),
    ];

    /// Translucent amber and ink, written as the brief writes them (rgba with alpha 0..1).
    pub fn amber_a(a: f32) -> Color32 {
        Color32::from_rgba_unmultiplied(200, 160, 64, (a * 255.0).round() as u8)
    }
    pub fn ink_a(a: f32) -> Color32 {
        Color32::from_rgba_unmultiplied(26, 26, 24, (a * 255.0).round() as u8)
    }
    pub fn white_a(a: f32) -> Color32 {
        Color32::from_rgba_unmultiplied(255, 255, 255, (a * 255.0).round() as u8)
    }
    pub fn terracotta_a(a: f32) -> Color32 {
        Color32::from_rgba_unmultiplied(139, 74, 42, (a * 255.0).round() as u8)
    }
}

/// Colours for the current appearance (light or dark), built only from `tok`.
#[derive(Clone, Copy)]
pub struct Pal {
    pub dark: bool,
    /// Work area.
    pub bg: Color32,
    /// Sidebar.
    pub side: Color32,
    pub card: Color32,
    pub hover: Color32,
    pub line: Color32,
    pub text: Color32,
    pub dim: Color32,
    pub faint: Color32,
    /// Accent fill (CTA, focus ring, active icon). Never text on light: use `accent_text`.
    pub accent: Color32,
    pub accent_soft: Color32,
    /// Amber meaning as text (amber-text on light, amber on dark).
    pub accent_text: Color32,
    /// Text/icons drawn on top of `accent`.
    pub on_accent: Color32,
    /// Neutral secondary accent (was blue; the brief has no blue).
    pub blue: Color32,
    /// Green mark (dots, bars).
    pub green: Color32,
    /// Green as text (green-text on light).
    pub green_text: Color32,
    pub green_tint: Color32,
    /// Terracotta: eyebrows, links, errors (terracotta-on-dark on dark).
    pub orange: Color32,
    pub orange_tint: Color32,
    /// Was purple; mapped to amber (the brief has no purple).
    pub purple: Color32,
    pub callout: Color32,
    pub code: Color32,
    pub field: Color32,
    pub toast_bg: Color32,
    pub toast_text: Color32,
    /// Dark ink surfaces (intake top bar, the one dominant card per screen) and text on them.
    pub ink: Color32,
    pub on_ink: Color32,
    pub on_ink_soft: Color32,
    pub on_ink_muted: Color32,
    /// Small labels above headlines.
    pub eyebrow: Color32,
    /// Test-mode banner.
    pub banner_bg: Color32,
    pub banner_text: Color32,
    /// Derived tints (Pack B decision B9).
    pub surface_raised: Color32,
    pub surface_sunk: Color32,
    pub track: Color32,
    pub divider: Color32,
    /// The sidebar's colour and its text.
    pub nav_bg: Color32,
}

/// Type sizes. HERO, QUESTION and ANSWER follow the brief; STEP_TITLE is the screen and step
/// headline size (brief "title" 30 to 36).
pub mod size {
    /// "Hi Simeon." only.
    pub const HERO: f32 = 52.0;
    /// Front door, reveal, intake 05.
    pub const DISPLAY: f32 = 44.0;
    /// Screen and step headlines.
    pub const STEP_TITLE: f32 = 30.0;
    /// One intake question at a time.
    pub const QUESTION: f32 = 30.0;
    /// Hero cards, theme cards, sheet titles (card-serif 20 to 28).
    pub const CARD_SERIF: f32 = 24.0;
    pub const TITLE: f32 = 22.0;
    pub const SECTION: f32 = 15.0;
    pub const BODY: f32 = 14.0;
    /// Text boxes.
    pub const ANSWER: f32 = 15.0;
    pub const BUTTON: f32 = 14.0;
    /// Eyebrows, card titles, group labels (label 12.5 to 14).
    pub const LABEL: f32 = 13.0;
    pub const SMALL: f32 = 12.5;
    pub const TINY: f32 = 11.5;
    /// Tags, pills, counts.
    pub const TAG: f32 = 11.0;
    /// Sidebar group labels.
    pub const GROUP: f32 = 11.0;
    /// Line heights (multipliers), as the brief writes them.
    pub const LH_HERO: f32 = 1.05;
    pub const LH_DISPLAY: f32 = 1.1;
    pub const LH_TITLE: f32 = 1.15;
    pub const LH_QUESTION: f32 = 1.22;
    pub const LH_CARD_SERIF: f32 = 1.25;
    pub const LH_BODY: f32 = 1.5;
    pub const LH_ANSWER: f32 = 1.55;
}

/// Spacing (4 px base) and fixed widths.
pub mod space {
    pub const S4: f32 = 4.0;
    pub const S6: f32 = 6.0;
    pub const S8: f32 = 8.0;
    pub const S10: f32 = 10.0;
    pub const S12: f32 = 12.0;
    pub const S14: f32 = 14.0;
    pub const S16: f32 = 16.0;
    pub const S18: f32 = 18.0;
    pub const S22: f32 = 22.0;
    pub const S24: f32 = 24.0;
    pub const S26: f32 = 26.0;
    pub const S30: f32 = 30.0;
    pub const S32: f32 = 32.0;
    pub const S40: f32 = 40.0;
    /// Work area padding: 30 top, 40 sides (32 on Calendar and Roadmap).
    pub const WORK_TOP: f32 = 30.0;
    pub const WORK_SIDE: f32 = 40.0;
    pub const WORK_SIDE_WIDE: f32 = 32.0;
    // kept for existing views
    pub const RADIUS: f32 = 10.0;
    pub const RADIUS_SMALL: f32 = 6.0;
    pub const CARD_PAD: f32 = 16.0;
    pub const COLUMN: f32 = 680.0;
    pub const SIDE_PANEL: f32 = 340.0;
    /// Reading text width (680 to 720).
    pub const READING: f32 = 700.0;
    /// 1920 content caps (Rules, item 6).
    pub const CAP_DEFAULT: f32 = 1240.0;
    pub const CAP_TASKS: f32 = 1100.0;
    pub const CAP_SETTINGS: f32 = 1100.0;
    pub const CAP_TEAM: f32 = 1320.0;
    /// Sidebar.
    pub const SIDEBAR: f32 = 248.0;
    pub const SIDEBAR_COLLAPSED: f32 = 64.0;
    pub const NAV_ITEM_H: f32 = 32.0;
    /// Ask Resolve bar.
    pub const ASK_W: f32 = 560.0;
    pub const ASK_H: f32 = 48.0;
    pub const ASK_BOTTOM: f32 = 20.0;
    pub const MIC: f32 = 34.0;
    /// Toasts: 460 wide, 10 above the Ask bar, 3 px timer bar.
    pub const TOAST_W: f32 = 460.0;
    pub const TOAST_GAP: f32 = 10.0;
    pub const TOAST_TIMER: f32 = 3.0;
    /// Field heights.
    pub const FIELD_H: f32 = 38.0;
    pub const BUTTON_H: f32 = 36.0;
    pub const BUTTON_H_SMALL: f32 = 28.0;
    /// Focus ring width.
    pub const FOCUS_RING: f32 = 3.0;
}

/// Stroke widths.
pub mod stroke {
    /// Card and field edges, dividers.
    pub const HAIR: f32 = 1.0;
    /// Icons (the brief's stroke 1.4).
    pub const ICON: f32 = 1.4;
    /// Drawn lines and curves.
    pub const LINE: f32 = 1.6;
    /// Marks, outlines that must read at a glance.
    pub const MARK: f32 = 2.0;
    /// Small dots on a track.
    pub const DOT: f32 = 4.0;
}

/// Radii.
pub mod radius {
    pub const PILL: f32 = 999.0;
    pub const CHIP: f32 = 7.0;
    pub const SMALL_BUTTON: f32 = 8.0;
    pub const BUTTON: f32 = 10.0;
    pub const FIELD: f32 = 10.0;
    pub const CARD: f32 = 13.0;
    pub const TOAST: f32 = 14.0;
    pub const DIALOG: f32 = 16.0;
    pub const HERO: f32 = 16.0;
    pub const SHEET: f32 = 20.0;
    pub const NAV_ITEM: f32 = 8.0;
    pub const GATE: f32 = 6.0;
}

/// Shadows and rings, as the brief writes them (CSS box-shadow: x, y, blur, spread, colour).
pub mod shadow {
    use eframe::egui::{epaint::Shadow, vec2, Color32};
    const fn ink(alpha: u8) -> Color32 {
        // rgba(26,26,24,a), premultiplied
        Color32::from_rgba_premultiplied(((26u32 * alpha as u32) / 255) as u8, ((26u32 * alpha as u32) / 255) as u8, ((24u32 * alpha as u32) / 255) as u8, alpha)
    }
    const fn sh(x: f32, y: f32, blur: f32, spread: f32, c: Color32) -> Shadow {
        Shadow { offset: vec2(x, y), blur, spread, color: c }
    }
    /// card-lift: 0 1px 2px rgba(26,26,24,.04), 0 10px 30px rgba(26,26,24,.07)
    pub const CARD_LIFT: [Shadow; 2] = [sh(0.0, 1.0, 2.0, 0.0, ink(10)), sh(0.0, 10.0, 30.0, 0.0, ink(18))];
    /// dialog: 0 30px 80px rgba(26,26,24,.35)
    pub const DIALOG: Shadow = sh(0.0, 30.0, 80.0, 0.0, ink(89));
    /// sheet: 0 -18px 60px rgba(26,26,24,.25)
    pub const SHEET: Shadow = sh(0.0, -18.0, 60.0, 0.0, ink(64));
    /// side panel: -18px 0 50px rgba(26,26,24,.14)
    pub const SIDE_PANEL: Shadow = sh(-18.0, 0.0, 50.0, 0.0, ink(36));
    /// Ask bar: 0 10px 30px rgba(26,26,24,.22)
    pub const ASK_BAR: Shadow = sh(0.0, 10.0, 30.0, 0.0, ink(56));
    /// focus: 0 0 0 3px rgba(200,160,64,.22) (drawn as a ring, plus an amber border)
    pub const FOCUS_ALPHA: f32 = 0.22;
    /// backdrop alphas: panel .18, sheet .32, dialog .38
    pub const BACKDROP_PANEL: f32 = 0.18;
    pub const BACKDROP_SHEET: f32 = 0.32;
    pub const BACKDROP_DIALOG: f32 = 0.38;
}

/// The status pills (Frameworks, records, roadmap gates): (text, fill) for a status.
pub fn status_pair(status: &str) -> (Color32, Color32) {
    match status {
        "Supported" | "Gate open" | "Done" | "Reached" | "Decided" => (tok::GREEN_TEXT, tok::GREEN_TINT),
        "Weakened" | "Planned" | "Running" => (tok::AMBER_TEXT, tok::AMBER_LIGHT),
        "Refuted" | "Blocked" | "Missed" => (tok::TERRACOTTA, tok::TERRACOTTA_TINT),
        _ => (tok::INK_SOFT, tok::SURFACE),
    }
}

/// Knowledge base card kinds: (text, fill).
pub fn kind_pair(kind: &str) -> (Color32, Color32) {
    match kind {
        "Research" | "Evidence" => (tok::GREEN_TEXT, tok::GREEN_TINT),
        "Hypothesis" | "Assumption" => (tok::AMBER_TEXT, tok::AMBER_LIGHT),
        "Open question" | "Cut" => (tok::TERRACOTTA, tok::TERRACOTTA_TINT),
        _ => (tok::INK, tok::SURFACE),
    }
}

pub fn pal(dark: bool) -> Pal {
    use tok::*;
    if dark {
        Pal {
            dark,
            bg: PANE,
            side: NAV_BG,
            card: PANE_RAISED,
            hover: PANE_HOVER,
            line: white_a(0.08),
            text: TEXT_ON_DARK,
            dim: TEXT_ON_DARK_SOFT,
            faint: TEXT_ON_DARK_MUTED,
            accent: AMBER,
            accent_soft: AMBER_16_ON_NAV,
            accent_text: AMBER,
            on_accent: INK,
            blue: TEXT_ON_DARK_SOFT,
            green: GREEN,
            green_text: GREEN_TINT,
            green_tint: PANE_TRACK,
            orange: TERRACOTTA_ON_DARK,
            orange_tint: TERRACOTTA_22_ON_PANE,
            purple: AMBER,
            callout: AMBER_10_ON_PANE,
            code: INK,
            field: INK,
            toast_bg: INK,
            toast_text: TEXT_ON_DARK,
            ink: INK,
            on_ink: TEXT_ON_DARK,
            on_ink_soft: TEXT_ON_DARK_SOFT,
            on_ink_muted: TEXT_ON_DARK_MUTED,
            eyebrow: TERRACOTTA_ON_DARK,
            banner_bg: AMBER,
            banner_text: INK,
            surface_raised: PANE_HOVER,
            surface_sunk: NAV_BG,
            track: PANE_TRACK,
            divider: PANE_HOVER,
            nav_bg: NAV_BG,
        }
    } else {
        Pal {
            dark,
            bg: SURFACE,
            side: NAV_BG,
            card: CARD,
            hover: SURFACE_RAISED,
            line: BORDER,
            text: INK,
            dim: INK_SOFT,
            faint: INK_GHOST,
            accent: AMBER,
            accent_soft: AMBER_LIGHT,
            accent_text: AMBER_TEXT,
            on_accent: INK,
            blue: INK_SOFT,
            green: GREEN,
            green_text: GREEN_TEXT,
            green_tint: GREEN_TINT,
            orange: TERRACOTTA,
            orange_tint: TERRACOTTA_TINT,
            purple: AMBER_TEXT,
            callout: AMBER_LIGHT,
            code: SURFACE_RAISED,
            field: CARD,
            toast_bg: INK,
            toast_text: TEXT_ON_DARK,
            ink: INK,
            on_ink: TEXT_ON_DARK,
            on_ink_soft: TEXT_ON_DARK_SOFT,
            on_ink_muted: TEXT_ON_DARK_MUTED,
            eyebrow: TERRACOTTA,
            banner_bg: AMBER,
            banner_text: INK,
            surface_raised: SURFACE_RAISED,
            surface_sunk: SURFACE_SUNK,
            track: TRACK,
            divider: DIVIDER,
            nav_bg: NAV_BG,
        }
    }
}

pub fn rgb(c: [u8; 3]) -> Color32 {
    Color32::from_rgb(c[0], c[1], c[2])
}

/// Task priority colours: p1 terracotta, p2 amber, p3 ink-soft, p4 ghost.
pub fn prio_color(p: u8, pal: &Pal) -> Color32 {
    match p {
        1 => pal.orange,
        2 => pal.accent,
        3 => pal.dim,
        _ => pal.faint,
    }
}

/// Calendar and database option colours, from the tokens only (the brief has no other hues).
pub const SERIES: [Color32; 6] = [tok::TERRACOTTA, tok::GREEN, tok::AMBER, tok::INK_SOFT, tok::AMBER_TEXT, tok::GREEN_TEXT];

/// The colours a person can pick for a workspace or a project (stored as RGB in their data):
/// the same six, so nothing outside the tokens can be chosen.
pub const PICK_COLOURS: [[u8; 3]; 6] = [[0x8B, 0x4A, 0x2A], [0x5E, 0x7D, 0x4F], [0xC8, 0xA0, 0x40], [0x4A, 0x4A, 0x44], [0x7A, 0x5B, 0x12], [0x3F, 0x5A, 0x33]];

pub fn bold() -> FontFamily {
    FontFamily::Name("bold".into())
}

pub fn serif() -> FontFamily {
    FontFamily::Name("serif".into())
}

/// The one dominant headline per screen (brand rule). Uses the system's Georgia at runtime;
/// nothing is bundled. Falls back to bold sans.
pub fn fserif(size: f32) -> FontId {
    FontId::new(size, serif())
}

pub fn semibold() -> FontFamily {
    FontFamily::Name("semibold".into())
}

pub fn fb(size: f32) -> FontId {
    FontId::new(size, bold())
}

pub fn fs(size: f32) -> FontId {
    FontId::new(size, semibold())
}

pub fn fp(size: f32) -> FontId {
    FontId::proportional(size)
}

/// Use Windows' Segoe UI when it's there (crisper, has a real bold); fall back to egui's fonts.
pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    let dir = std::env::var("WINDIR").map(|w| format!("{w}\\Fonts")).unwrap_or_else(|_| "C:\\Windows\\Fonts".into());
    let load = |name: &str| std::fs::read(format!("{dir}\\{name}")).ok();
    let base: Vec<String> = fonts.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    let mut prop = base.clone();
    let mut boldf = base.clone();
    let mut semi = base.clone();
    if let Some(b) = load("segoeui.ttf") {
        fonts.font_data.insert("segoe".into(), FontData::from_owned(b));
        prop.insert(0, "segoe".into());
    }
    if let Some(b) = load("segoeuib.ttf") {
        fonts.font_data.insert("segoe-bold".into(), FontData::from_owned(b));
        boldf.insert(0, "segoe-bold".into());
    } else if prop.first().map(|s| s == "segoe").unwrap_or(false) {
        boldf.insert(0, "segoe".into());
    }
    if let Some(b) = load("seguisb.ttf") {
        fonts.font_data.insert("segoe-semi".into(), FontData::from_owned(b));
        semi.insert(0, "segoe-semi".into());
    } else {
        semi = boldf.clone();
    }
    // Segoe UI Emoji's outlines give monochrome emoji for page icons that egui's small emoji font lacks
    if let Some(b) = load("seguiemj.ttf") {
        fonts.font_data.insert("segoe-emoji".into(), FontData::from_owned(b));
        for f in [&mut prop, &mut boldf, &mut semi] {
            let at = f.iter().position(|n| n == "segoe").map(|i| i + 1).unwrap_or(0).min(f.len());
            f.insert(at.max(1).min(f.len()), "segoe-emoji".into());
        }
    }
    if let Some(b) = load("seguisym.ttf") {
        fonts.font_data.insert("segoe-sym".into(), FontData::from_owned(b));
        for f in [&mut prop, &mut boldf, &mut semi] {
            f.push("segoe-sym".into());
        }
    }
    if let Some(b) = load("consola.ttf") {
        fonts.font_data.insert("consolas".into(), FontData::from_owned(b));
        if let Some(m) = fonts.families.get_mut(&FontFamily::Monospace) {
            m.insert(0, "consolas".into());
        }
    }
    // Resolve brand fonts, bundled (SIL OFL 1.1 — licences in assets/fonts): Inter for text,
    // DM Serif Display for the one dominant headline per screen. System fonts stay as fallbacks
    // for characters outside their Latin subset.
    fonts.font_data.insert("inter".into(), FontData::from_static(include_bytes!("../../assets/fonts/Inter-Regular.ttf")));
    fonts.font_data.insert("inter-semi".into(), FontData::from_static(include_bytes!("../../assets/fonts/Inter-SemiBold.ttf")));
    fonts.font_data.insert("inter-bold".into(), FontData::from_static(include_bytes!("../../assets/fonts/Inter-Bold.ttf")));
    fonts.font_data.insert("dm-serif".into(), FontData::from_static(include_bytes!("../../assets/fonts/DMSerifDisplay-Regular.ttf")));
    prop.insert(0, "inter".into());
    semi.insert(0, "inter-semi".into());
    boldf.insert(0, "inter-bold".into());
    fonts.families.insert(FontFamily::Proportional, prop);
    fonts.families.insert(bold(), boldf.clone());
    let mut serif_f = boldf.clone();
    if let Some(b) = load("georgia.ttf") {
        fonts.font_data.insert("georgia".into(), FontData::from_owned(b));
        serif_f.insert(0, "georgia".into());
    }
    serif_f.insert(0, "dm-serif".into());
    fonts.families.insert(serif(), serif_f);
    fonts.families.insert(semibold(), semi);
    ctx.set_fonts(fonts);
}

/// The sidebar is nav-bg in both appearances (brief, The shell), so it always draws with the
/// dark palette.
pub fn sidebar_pal() -> Pal {
    pal(true)
}

/// egui's visuals for a palette (also used for the always-dark sidebar).
pub fn visuals(p: &Pal) -> egui::Visuals {
    let mut v = if p.dark { egui::Visuals::dark() } else { egui::Visuals::light() };
    v.panel_fill = p.bg;
    v.window_fill = p.card;
    v.window_stroke = Stroke::new(1.0, p.line);
    v.window_rounding = Rounding::same(radius::DIALOG);
    v.menu_rounding = Rounding::same(radius::SMALL_BUTTON);
    v.extreme_bg_color = p.field;
    v.faint_bg_color = p.hover;
    v.code_bg_color = p.code;
    v.hyperlink_color = p.orange;
    v.override_text_color = Some(p.text);
    v.selection.bg_fill = tok::amber_a(0.35);
    v.selection.stroke = Stroke::new(1.0, p.accent);
    v.window_shadow = shadow::DIALOG;
    v.popup_shadow = shadow::CARD_LIFT[1];
    let r = Rounding::same(radius::SMALL_BUTTON);
    v.widgets.noninteractive.bg_fill = p.card;
    v.widgets.noninteractive.weak_bg_fill = p.card;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, p.line);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, p.text);
    v.widgets.noninteractive.rounding = r;
    v.widgets.inactive.bg_fill = p.hover;
    v.widgets.inactive.weak_bg_fill = p.surface_raised;
    v.widgets.inactive.bg_stroke = Stroke::NONE;
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, p.text);
    v.widgets.inactive.rounding = r;
    v.widgets.hovered.bg_fill = p.hover;
    v.widgets.hovered.weak_bg_fill = p.track;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, p.line);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, p.text);
    v.widgets.hovered.rounding = r;
    v.widgets.active.bg_fill = p.accent_soft;
    v.widgets.active.weak_bg_fill = p.accent_soft;
    v.widgets.active.bg_stroke = Stroke::new(1.0, p.accent);
    v.widgets.active.fg_stroke = Stroke::new(1.0, p.text);
    v.widgets.active.rounding = r;
    v.widgets.open = v.widgets.hovered;
    v
}

pub fn apply(ctx: &egui::Context, p: &Pal) {
    let mut style = (*ctx.style()).clone();
    style.visuals = visuals(p);
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(10.0, 5.0);
    style.spacing.interact_size.y = 26.0;
    style.spacing.menu_margin = egui::Margin::same(6.0);
    style.spacing.window_margin = egui::Margin::same(16.0);
    style.spacing.scroll = egui::style::ScrollStyle::thin();
    style.text_styles = [
        (TextStyle::Heading, fserif(size::TITLE)),
        (TextStyle::Body, fp(size::BODY)),
        (TextStyle::Button, fs(size::BUTTON)),
        (TextStyle::Small, fp(size::SMALL)),
        (TextStyle::Monospace, FontId::monospace(13.0)),
    ]
    .into();
    ctx.set_style(style);
}
