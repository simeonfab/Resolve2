use eframe::egui::{self, Color32, FontData, FontDefinitions, FontFamily, FontId, Rounding, Stroke, TextStyle};

/// Resolve 2 design tokens. Derived from the original Resolve brand reference
/// (resolve-docs strategy/resolve-brand-reference.md): ink #1A1A18, cream surface #F2EDE4,
/// border #DDD8CE, amber #C8A040, terracotta #8B4A2A (#C97A4A on dark), dark page #141311,
/// dark pane #201F1C. Amber is the single accent; ink text sits on amber for contrast.
#[derive(Clone, Copy)]
pub struct Pal {
    pub dark: bool,
    pub bg: Color32,
    pub side: Color32,
    pub card: Color32,
    pub hover: Color32,
    pub line: Color32,
    pub text: Color32,
    pub dim: Color32,
    pub faint: Color32,
    pub accent: Color32,
    pub accent_soft: Color32,
    /// Text/icons drawn on top of `accent`.
    pub on_accent: Color32,
    pub blue: Color32,
    pub green: Color32,
    pub orange: Color32,
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
    /// Small labels above headlines.
    pub eyebrow: Color32,
    /// Test-mode banner.
    pub banner_bg: Color32,
    pub banner_text: Color32,
}

/// Type sizes (one place to change them).
pub mod size {
    pub const HERO: f32 = 30.0;
    pub const QUESTION: f32 = 28.0;
    pub const TITLE: f32 = 22.0;
    pub const SECTION: f32 = 15.0;
    pub const BODY: f32 = 14.0;
    pub const ANSWER: f32 = 17.0;
    pub const SMALL: f32 = 12.5;
    pub const TINY: f32 = 11.5;
}

/// Spacing and radii.
pub mod space {
    pub const RADIUS: f32 = 10.0;
    pub const RADIUS_SMALL: f32 = 6.0;
    pub const CARD_PAD: f32 = 16.0;
    pub const COLUMN: f32 = 680.0;
    pub const SIDE_PANEL: f32 = 340.0;
}

pub fn pal(dark: bool) -> Pal {
    if dark {
        Pal {
            dark,
            bg: Color32::from_rgb(32, 31, 28),
            side: Color32::from_rgb(24, 23, 20),
            card: Color32::from_rgb(40, 38, 34),
            hover: Color32::from_rgb(50, 48, 43),
            line: Color32::from_rgb(58, 56, 50),
            text: Color32::from_rgb(245, 241, 232),
            dim: Color32::from_rgb(180, 175, 163),
            faint: Color32::from_rgb(138, 133, 120),
            accent: Color32::from_rgb(200, 160, 64),
            accent_soft: Color32::from_rgb(64, 54, 32),
            on_accent: Color32::from_rgb(26, 26, 24),
            blue: Color32::from_rgb(122, 156, 196),
            green: Color32::from_rgb(132, 168, 112),
            orange: Color32::from_rgb(201, 122, 74),
            purple: Color32::from_rgb(170, 140, 200),
            callout: Color32::from_rgb(46, 42, 33),
            code: Color32::from_rgb(28, 27, 24),
            field: Color32::from_rgb(26, 26, 24),
            toast_bg: Color32::from_rgb(58, 56, 50),
            toast_text: Color32::from_rgb(251, 248, 242),
            ink: Color32::from_rgb(20, 19, 17),
            on_ink: Color32::from_rgb(251, 248, 242),
            on_ink_soft: Color32::from_rgb(201, 196, 183),
            eyebrow: Color32::from_rgb(201, 122, 74),
            banner_bg: Color32::from_rgb(200, 160, 64),
            banner_text: Color32::from_rgb(26, 26, 24),
        }
    } else {
        Pal {
            dark,
            bg: Color32::from_rgb(250, 248, 243),
            side: Color32::from_rgb(242, 237, 228),
            card: Color32::from_rgb(255, 255, 255),
            hover: Color32::from_rgb(234, 228, 216),
            line: Color32::from_rgb(221, 216, 206),
            text: Color32::from_rgb(26, 26, 24),
            dim: Color32::from_rgb(90, 90, 84),
            faint: Color32::from_rgb(154, 154, 146),
            accent: Color32::from_rgb(176, 136, 40),
            accent_soft: Color32::from_rgb(251, 245, 230),
            on_accent: Color32::from_rgb(26, 26, 24),
            blue: Color32::from_rgb(58, 96, 140),
            green: Color32::from_rgb(94, 125, 79),
            orange: Color32::from_rgb(139, 74, 42),
            purple: Color32::from_rgb(110, 80, 150),
            callout: Color32::from_rgb(251, 245, 230),
            code: Color32::from_rgb(244, 240, 232),
            field: Color32::from_rgb(255, 255, 255),
            toast_bg: Color32::from_rgb(26, 26, 24),
            toast_text: Color32::from_rgb(251, 248, 242),
            ink: Color32::from_rgb(26, 26, 24),
            on_ink: Color32::from_rgb(251, 248, 242),
            on_ink_soft: Color32::from_rgb(201, 196, 183),
            eyebrow: Color32::from_rgb(139, 74, 42),
            banner_bg: Color32::from_rgb(200, 160, 64),
            banner_text: Color32::from_rgb(26, 26, 24),
        }
    }
}
pub fn rgb(c: [u8; 3]) -> Color32 {
    Color32::from_rgb(c[0], c[1], c[2])
}

pub fn prio_color(p: u8, pal: &Pal) -> Color32 {
    match p {
        1 => Color32::from_rgb(201, 122, 74),
        2 => Color32::from_rgb(200, 160, 64),
        3 => Color32::from_rgb(122, 156, 196),
        _ => pal.faint,
    }
}

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
    fonts.families.insert(FontFamily::Proportional, prop);
    fonts.families.insert(bold(), boldf.clone());
    let mut serif_f = boldf.clone();
    if let Some(b) = load("georgia.ttf") {
        fonts.font_data.insert("georgia".into(), FontData::from_owned(b));
        serif_f.insert(0, "georgia".into());
    }
    fonts.families.insert(serif(), serif_f);
    fonts.families.insert(semibold(), semi);
    ctx.set_fonts(fonts);
}

pub fn apply(ctx: &egui::Context, p: &Pal) {
    let mut style = (*ctx.style()).clone();
    let mut v = if p.dark { egui::Visuals::dark() } else { egui::Visuals::light() };
    v.panel_fill = p.bg;
    v.window_fill = p.card;
    v.window_stroke = Stroke::new(1.0, p.line);
    v.window_rounding = Rounding::same(10.0);
    v.menu_rounding = Rounding::same(8.0);
    v.extreme_bg_color = p.field;
    v.faint_bg_color = p.hover;
    v.code_bg_color = p.code;
    v.hyperlink_color = p.blue;
    v.override_text_color = Some(p.text);
    v.selection.bg_fill = p.accent.linear_multiply(0.45);
    v.selection.stroke = Stroke::new(1.0, p.accent);
    v.window_shadow = egui::epaint::Shadow { offset: egui::vec2(0.0, 8.0), blur: 24.0, spread: 0.0, color: Color32::from_black_alpha(if p.dark { 120 } else { 40 }) };
    v.popup_shadow = egui::epaint::Shadow { offset: egui::vec2(0.0, 4.0), blur: 14.0, spread: 0.0, color: Color32::from_black_alpha(if p.dark { 110 } else { 35 }) };
    let r = Rounding::same(6.0);
    v.widgets.noninteractive.bg_fill = p.card;
    v.widgets.noninteractive.weak_bg_fill = p.card;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, p.line);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, p.text);
    v.widgets.noninteractive.rounding = r;
    v.widgets.inactive.bg_fill = p.hover;
    v.widgets.inactive.weak_bg_fill = if p.dark { Color32::from_rgb(46, 46, 46) } else { Color32::from_rgb(244, 242, 239) };
    v.widgets.inactive.bg_stroke = Stroke::NONE;
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, p.text);
    v.widgets.inactive.rounding = r;
    v.widgets.hovered.bg_fill = p.hover;
    v.widgets.hovered.weak_bg_fill = if p.dark { Color32::from_rgb(58, 58, 58) } else { Color32::from_rgb(234, 232, 228) };
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, p.line);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, p.text);
    v.widgets.hovered.rounding = r;
    v.widgets.active.bg_fill = p.accent_soft;
    v.widgets.active.weak_bg_fill = p.accent_soft;
    v.widgets.active.bg_stroke = Stroke::new(1.0, p.accent);
    v.widgets.active.fg_stroke = Stroke::new(1.0, p.text);
    v.widgets.active.rounding = r;
    v.widgets.open = v.widgets.hovered;
    style.visuals = v;
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(10.0, 5.0);
    style.spacing.interact_size.y = 26.0;
    style.spacing.menu_margin = egui::Margin::same(6.0);
    style.spacing.window_margin = egui::Margin::same(16.0);
    style.spacing.scroll = egui::style::ScrollStyle::thin();
    style.text_styles = [
        (TextStyle::Heading, fb(22.0)),
        (TextStyle::Body, fp(14.0)),
        (TextStyle::Button, fp(14.0)),
        (TextStyle::Small, fp(12.0)),
        (TextStyle::Monospace, FontId::monospace(13.0)),
    ]
    .into();
    ctx.set_style(style);
}
