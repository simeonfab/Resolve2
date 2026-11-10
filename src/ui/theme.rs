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
    /// From the Pack B canvas (not in the brief's 24): body text on the ink hero, the amber-light
    /// card's edge, and the dashed edge of an empty card.
    pub const HERO_BODY: Color32 = hex(0xD8D3C7);
    pub const AMBER_EDGE: Color32 = hex(0xE9D9A8);
    pub const DASHED_EDGE: Color32 = hex(0xCFC8BA);
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
    /// The intake question: 29 as the Pack A canvas draws it (the brief said 30; Simeon chose
    /// the canvas on 8 Nov), line height 1.22.
    pub const QUESTION: f32 = 29.0;
    /// egui sets DM Serif about 1.7% wider than the browser does (measured on the canvas's first
    /// line: 675 against 664), so the question loses 0.2 px per letter to wrap as drawn.
    pub const QUESTION_TRACKING: f32 = -0.2;
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
    /// Sidebar group labels (Inter 600 11, letter spacing .04em).
    pub const GROUP: f32 = 11.0;
    pub const GROUP_SPACING: f32 = GROUP * 0.04;
    /// Sidebar items (Inter 500 13.5).
    pub const NAV: f32 = 13.5;
    /// Home (Pack B canvas): the greeting (DM Serif 32), the hero title (28; 34 above 1600
    /// wide), the hero's body (14.5), list rows (13.5 to 14), card heads (12, 700).
    pub const GREETING: f32 = 32.0;
    pub const HERO_TITLE: f32 = 28.0;
    pub const HERO_TITLE_WIDE: f32 = 34.0;
    pub const HERO_BODY: f32 = 14.5;
    pub const ROW: f32 = 14.0;
    pub const ROW_SMALL: f32 = 13.5;
    pub const CARD_HEAD: f32 = 12.0;
    /// The Add task field's text.
    pub const QUICK: f32 = 17.0;
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
    /// Calendar (Pack B): side margin, the left column (calendars) and the gap to the grid.
    pub const CAL_SIDE: f32 = 32.0;
    pub const CAL_LEFT: f32 = 188.0;
    pub const CAL_GAP: f32 = 18.0;
    /// The event editor (Pack B "Calendar: event"): width and the label column.
    pub const EVENT_W: f32 = 520.0;
    pub const EVENT_LABEL_W: f32 = 104.0;
    pub const CAP_SETTINGS: f32 = 1100.0;
    pub const CAP_TEAM: f32 = 1320.0;
    /// Roadmap (Pack C): lanes 12 apart, the right rail 270 (it stays 270 at 1920 while the lanes
    /// widen), 16 between them; the whole picture is 944 wide, 40 from the top, its hypothesis
    /// column 190.
    pub const CAP_ROADMAP: f32 = 1600.0;
    pub const RM_LANE_GAP: f32 = 12.0;
    pub const RM_RAIL: f32 = 270.0;
    pub const RM_RAIL_GAP: f32 = 16.0;
    pub const RM_TOGGLE: f32 = 24.0;
    pub const RM_CARD_PAD: f32 = 12.0;
    pub const WHOLE_W: f32 = 944.0;
    pub const WHOLE_TOP: f32 = 40.0;
    pub const WHOLE_LABEL_W: f32 = 190.0;
    pub const WHOLE_CELL_H: f32 = 54.0;
    /// Frameworks (Pack C): registry cards 330 and 270, the node editor 436 beside the tree,
    /// tree levels 32 apart, the node label column 40.
    pub const FW_REGISTRY_W: f32 = 330.0;
    pub const FW_REGISTRY_MORE_W: f32 = 270.0;
    pub const FW_EDITOR: f32 = 436.0;
    pub const FW_INDENT: f32 = 32.0;
    pub const FW_LABEL_W: f32 = 40.0;
    /// The proposal editor (Pack C): a 60 top bar, the section list 210, 24 to the cards.
    pub const PE_BAR: f32 = 60.0;
    pub const PE_LIST: f32 = 210.0;
    pub const PE_GAP: f32 = 24.0;
    /// Knowledge base (Pack C): library cards 120 high and 12 apart, 3 across (4 from a 1200
    /// wide library), the find field 240, the Add sheet 620, the preview panel 520, the New page
    /// dialog 684.
    pub const LIB_CARD_H: f32 = 120.0;
    pub const LIB_GAP: f32 = 12.0;
    pub const LIB_FOUR_FROM: f32 = 1200.0;
    pub const FIND_W: f32 = 240.0;
    pub const ADD_SHEET_W: f32 = 620.0;
    pub const PREVIEW_W: f32 = 520.0;
    pub const NEW_PAGE_W: f32 = 684.0;
    /// The page editor (Pack C): a 52 top bar, text 680 to 720 wide for reading, the rail 250
    /// with 30 before it.
    pub const PAGE_BAR: f32 = 52.0;
    pub const PAGE_COL: f32 = 720.0;
    pub const PAGE_RAIL: f32 = 250.0;
    pub const PAGE_GAP: f32 = 30.0;
    /// The blocks' left gutter (their + and drag handle, 52, and the gap after it).
    pub const PAGE_GUTTER: f32 = 60.0;
    /// Databases (Pack C): the list of databases 200 wide beside the table.
    pub const DB_LIST: f32 = 200.0;
    /// AI Team (Pack C): the roles column 280 beside the chat.
    pub const TEAM_LIST: f32 = 280.0;
    /// Handoffs (Pack C): the new-handoff and list column 360 beside the brief.
    pub const HO_LEFT: f32 = 360.0;
    /// Settings (Pack C): two columns 16 apart, the left a little wider; field heights 40.
    pub const SET_GAP: f32 = 16.0;
    pub const SET_LEFT_SHARE: f32 = 0.52;
    /// Help (Pack C): the section list 200, the reading column 680; the Report dialog 560.
    pub const HELP_LIST: f32 = 200.0;
    pub const HELP_COL: f32 = 680.0;
    pub const REPORT_W: f32 = 560.0;
    /// Sidebar.
    pub const SIDEBAR: f32 = 248.0;
    pub const SIDEBAR_COLLAPSED: f32 = 64.0;
    pub const NAV_ITEM_H: f32 = 32.0;
    /// Sidebar inner padding; icons sit 20 from an item's left edge (centred when collapsed,
    /// so they don't move), labels at 40.
    pub const NAV_PAD: f32 = 12.0;
    pub const NAV_ICON: f32 = 16.0;
    /// Gap between sidebar items.
    pub const NAV_GAP: f32 = 2.0;
    pub const NAV_ICON_X: f32 = 20.0;
    pub const NAV_LABEL_X: f32 = 40.0;
    /// Workspace tile and the sidebar's two buttons.
    pub const NAV_TILE: f32 = 26.0;
    pub const NAV_BUTTON_H: f32 = 34.0;
    /// Tooltip beside a collapsed item.
    pub const NAV_TIP_GAP: f32 = 10.0;
    /// Ask sheet: 720 wide, 82% of the window high, flush with the bottom.
    pub const ASK_SHEET_W: f32 = 720.0;
    pub const ASK_SHEET_HEIGHT: f32 = 0.82;
    /// The strip the Ask bar floats in (kept clear of screen content).
    pub const ASK_STRIP: f32 = ASK_BOTTOM + ASK_H + S12;
    /// Dialogs sit 70 to 90 px from the top; the search palette is 680 wide.
    pub const DIALOG_TOP: f32 = 80.0;
    pub const SEARCH_W: f32 = 680.0;
    pub const SEARCH_LIST_H: f32 = 440.0;
    /// Home (Pack B canvas): columns 1.25 : 1 with a 20 gap, the right column at most 440, the
    /// hero padded 26 / 30, list times 52 wide, 16 px marks, 1600 is where wide sizes start.
    pub const HOME_GAP: f32 = 20.0;
    /// Plan my day: a sheet 760 wide, 88% high; times 92 wide.
    pub const PLAN_SHEET_W: f32 = 760.0;
    pub const PLAN_SHEET_HEIGHT: f32 = 0.88;
    pub const PLAN_TIME_W: f32 = 92.0;
    /// Add task: a 640 dialog, 110 from the top.
    pub const QUICK_W: f32 = 640.0;
    pub const QUICK_TOP: f32 = 110.0;
    /// The task editor: a 440 push panel from the right (B-E9).
    pub const TASK_PANEL: f32 = 440.0;
    /// The Board's columns end this far above the window's bottom (header and Ask bar).
    pub const BOARD_TOP_OFFSET: f32 = 290.0;
    pub const HOME_ASIDE_MAX: f32 = 440.0;
    pub const HOME_LEFT_SHARE: f32 = 1.25 / 2.25;
    pub const HERO_PAD_Y: f32 = 26.0;
    pub const HERO_PAD_X: f32 = 30.0;
    pub const AGENDA_TIME_W: f32 = 52.0;
    pub const MARK: f32 = 16.0;
    pub const WIDE_FROM: f32 = 1600.0;
    pub const BAR_H: f32 = 5.0;
    pub const HOME_SKELETON_HERO: f32 = 300.0;
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
    /// The intake frame (Pack A): top bar 64, rail 308 (72 on wide steps), pane padding 40 top
    /// and 56 sides, column 760 (800 on Confirm, 960 at 1600 and wider), heard panel 380 (440).
    pub const INTAKE_BAR: f32 = 64.0;
    pub const INTAKE_PROGRESS_W: f32 = 360.0;
    pub const INTAKE_PROGRESS_H: f32 = 3.0;
    pub const RAIL: f32 = 308.0;
    pub const RAIL_NARROW: f32 = 72.0;
    pub const RAIL_PAD: f32 = 24.0;
    pub const RAIL_DOT: f32 = 26.0;
    pub const PANE_TOP: f32 = 40.0;
    pub const PANE_SIDE: f32 = 56.0;
    pub const INTAKE_COL: f32 = 760.0;
    pub const INTAKE_COL_CONFIRM: f32 = 800.0;
    pub const INTAKE_COL_WIDE: f32 = 960.0;
    pub const HEARD_W: f32 = 380.0;
    pub const HEARD_W_WIDE: f32 = 440.0;
    pub const INTAKE_GAP: f32 = 40.0;
    /// Questions: the pane's side margin there (the canvas's column is 691 at 1280).
    pub const PANE_SIDE_QUESTIONS: f32 = 48.0;
    /// Front door and bridge: the centred column.
    pub const FRONT_COL: f32 = 460.0;
    pub const BRIDGE_CARD_W: f32 = 640.0;
    pub const REVEAL_COL: f32 = 620.0;
    /// Path cards on Welcome; the icon tile inside them.
    pub const PATH_TILE: f32 = 40.0;
    /// The answer box and the Bring it in box (minimum heights).
    pub const ANSWER_MIN_H: f32 = 108.0;
    pub const BRING_MIN_H: f32 = 290.0;
    /// The waiting card's ring and the 15 s bar.
    pub const WAIT_RING: f32 = 40.0;
    pub const WAIT_CUTOFF_S: f32 = 15.0;
    /// The resume card.
    pub const RESUME_W: f32 = 580.0;
    /// The intake's large buttons (Continue →, Next →): 44 high.
    pub const BUTTON_H_LARGE: f32 = 44.0;
    pub const BUTTON_PAD_LARGE: f32 = 22.0;
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
    /// The workspace tile in the sidebar.
    pub const TILE: f32 = 7.0;
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
    /// The halo round the reveal's Open my project (Pack A 11): a 6 px ring at .14.
    pub const HALO_ALPHA: f32 = 0.14;
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

/// `over` laid over `base` at `a` (a CSS rgba shadow over an opaque page), as a solid colour.
pub fn mix(base: Color32, over: Color32, a: f32) -> Color32 {
    let m = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * a.clamp(0.0, 1.0)).round() as u8;
    Color32::from_rgb(m(base.r(), over.r()), m(base.g(), over.g()), m(base.b(), over.b()))
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

/// Inter Medium (500): the sidebar items, and wherever the brief or canvas says 500.
pub fn medium() -> FontFamily {
    FontFamily::Name("medium".into())
}

pub fn fm(size: f32) -> FontId {
    FontId::new(size, medium())
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
    fonts.font_data.insert("inter-medium".into(), FontData::from_static(include_bytes!("../../assets/fonts/Inter-Medium.ttf")));
    fonts.font_data.insert("inter-semi".into(), FontData::from_static(include_bytes!("../../assets/fonts/Inter-SemiBold.ttf")));
    fonts.font_data.insert("inter-bold".into(), FontData::from_static(include_bytes!("../../assets/fonts/Inter-Bold.ttf")));
    fonts.font_data.insert("dm-serif".into(), FontData::from_static(include_bytes!("../../assets/fonts/DMSerifDisplay-Regular.ttf")));
    prop.insert(0, "inter".into());
    semi.insert(0, "inter-semi".into());
    boldf.insert(0, "inter-bold".into());
    // medium: Inter Medium first, then the regular chain for anything outside its subset
    let mut med = prop.clone();
    med.insert(0, "inter-medium".into());
    fonts.families.insert(medium(), med);
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
