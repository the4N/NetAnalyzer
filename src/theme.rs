// ─────────────────────────────────────────────────────────────
// NetAnalyzer - Custom Dark Theme
// ─────────────────────────────────────────────────────────────

use eframe::egui::{self, Color32, CornerRadius, Stroke, Visuals, style::Widgets, style::WidgetVisuals};

// ── Color Palette ──────────────────────────────────────────

pub const BG_DARKEST: Color32 = Color32::from_rgb(15, 17, 23);       // #0F1117
pub const BG_DARK: Color32 = Color32::from_rgb(22, 24, 35);          // #161823
pub const BG_SURFACE: Color32 = Color32::from_rgb(26, 29, 46);       // #1A1D2E
pub const BG_ELEVATED: Color32 = Color32::from_rgb(32, 36, 56);      // #202438
pub const BG_HOVER: Color32 = Color32::from_rgb(40, 44, 68);         // #282C44

pub const PRIMARY: Color32 = Color32::from_rgb(59, 130, 246);        // #3B82F6 Electric Blue
pub const PRIMARY_HOVER: Color32 = Color32::from_rgb(96, 165, 250);  // #60A5FA
pub const SECONDARY: Color32 = Color32::from_rgb(6, 182, 212);       // #06B6D4 Cyan
pub const ACCENT: Color32 = Color32::from_rgb(139, 92, 246);         // #8B5CF6 Purple

pub const SUCCESS: Color32 = Color32::from_rgb(16, 185, 129);        // #10B981 Emerald
pub const WARNING: Color32 = Color32::from_rgb(245, 158, 11);        // #F59E0B Amber
pub const ERROR: Color32 = Color32::from_rgb(239, 68, 68);           // #EF4444 Rose
pub const INFO: Color32 = Color32::from_rgb(56, 189, 248);           // #38BDF8 Sky

pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(226, 232, 240);   // #E2E8F0
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(148, 163, 184); // #94A3B8
pub const TEXT_MUTED: Color32 = Color32::from_rgb(100, 116, 139);     // #64748B
pub const TEXT_ON_PRIMARY: Color32 = Color32::from_rgb(255, 255, 255);

pub const BORDER: Color32 = Color32::from_rgb(45, 50, 75);           // #2D324B
pub const BORDER_HOVER: Color32 = Color32::from_rgb(60, 66, 100);    // #3C4264

// ── Signal Strength Colors ─────────────────────────────────

pub fn signal_color(dbm: i32) -> Color32 {
    if dbm >= -50 { SUCCESS }
    else if dbm >= -60 { Color32::from_rgb(52, 211, 153) }
    else if dbm >= -70 { WARNING }
    else if dbm >= -80 { Color32::from_rgb(251, 146, 60) }
    else { ERROR }
}

pub fn channel_congestion_color(count: usize) -> Color32 {
    if count <= 2 { SUCCESS }
    else if count <= 5 { WARNING }
    else { ERROR }
}

// ── Apply Theme ────────────────────────────────────────────

pub fn apply_theme(ctx: &egui::Context) {
    let mut visuals = Visuals::dark();

    // Window & Panel backgrounds
    visuals.panel_fill = BG_DARK;
    visuals.window_fill = BG_SURFACE;
    visuals.extreme_bg_color = BG_DARKEST;
    visuals.faint_bg_color = BG_ELEVATED;

    // Selection
    visuals.selection.bg_fill = PRIMARY.linear_multiply(0.3);
    visuals.selection.stroke = Stroke::new(1.0, PRIMARY);

    // Hyperlinks
    visuals.hyperlink_color = SECONDARY;

    // Window
    visuals.window_corner_radius = CornerRadius::same(8);
    visuals.window_stroke = Stroke::new(1.0, BORDER);
    visuals.window_shadow = egui::epaint::Shadow {
        offset: [0, 4],
        blur: 16,
        spread: 0,
        color: Color32::from_black_alpha(80),
    };

    // Widgets
    visuals.widgets = Widgets {
        noninteractive: WidgetVisuals {
            bg_fill: BG_SURFACE,
            weak_bg_fill: BG_SURFACE,
            bg_stroke: Stroke::new(1.0, BORDER),
            corner_radius: CornerRadius::same(6),
            fg_stroke: Stroke::new(1.0, TEXT_SECONDARY),
            expansion: 0.0,
        },
        inactive: WidgetVisuals {
            bg_fill: BG_ELEVATED,
            weak_bg_fill: BG_ELEVATED,
            bg_stroke: Stroke::new(1.0, BORDER),
            corner_radius: CornerRadius::same(6),
            fg_stroke: Stroke::new(1.0, TEXT_PRIMARY),
            expansion: 0.0,
        },
        hovered: WidgetVisuals {
            bg_fill: BG_HOVER,
            weak_bg_fill: BG_HOVER,
            bg_stroke: Stroke::new(1.0, BORDER_HOVER),
            corner_radius: CornerRadius::same(6),
            fg_stroke: Stroke::new(1.5, TEXT_PRIMARY),
            expansion: 1.0,
        },
        active: WidgetVisuals {
            bg_fill: PRIMARY.linear_multiply(0.3),
            weak_bg_fill: PRIMARY.linear_multiply(0.3),
            bg_stroke: Stroke::new(1.0, PRIMARY),
            corner_radius: CornerRadius::same(6),
            fg_stroke: Stroke::new(2.0, TEXT_PRIMARY),
            expansion: 1.0,
        },
        open: WidgetVisuals {
            bg_fill: BG_ELEVATED,
            weak_bg_fill: BG_ELEVATED,
            bg_stroke: Stroke::new(1.0, BORDER),
            corner_radius: CornerRadius::same(6),
            fg_stroke: Stroke::new(1.0, TEXT_PRIMARY),
            expansion: 0.0,
        },
    };

    ctx.set_visuals(visuals);

    // Custom style tweaks
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(12.0, 6.0);
    style.spacing.window_margin = egui::Margin::same(12);
    ctx.set_style(style);
}

// ── Helper: styled frame for cards/panels ──────────────────

pub fn card_frame() -> egui::Frame {
    egui::Frame::default()
        .fill(BG_SURFACE)
        .corner_radius(CornerRadius::same(8))
        .stroke(Stroke::new(1.0, BORDER))
        .inner_margin(egui::Margin::same(16))
        .outer_margin(egui::Margin::same(4))
}

pub fn sidebar_frame() -> egui::Frame {
    egui::Frame::default()
        .fill(BG_DARKEST)
        .inner_margin(egui::Margin::same(8))
}

pub fn content_frame() -> egui::Frame {
    egui::Frame::default()
        .fill(BG_DARK)
        .inner_margin(egui::Margin::same(20))
}

pub fn status_frame() -> egui::Frame {
    egui::Frame::default()
        .fill(BG_DARKEST)
        .inner_margin(egui::Margin::symmetric(12, 4))
}
