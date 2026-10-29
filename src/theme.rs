use egui::{
    epaint::Shadow, Color32, CornerRadius, Margin, Stroke, Style, Visuals,
};

pub struct AntigravityTheme;

impl AntigravityTheme {
    // Backgrounds
    pub const BG_CANVAS: Color32 = Color32::from_rgb(10, 12, 16); // #0A0C10
    pub const BG_SURFACE: Color32 = Color32::from_rgb(17, 21, 30); // #11151E
    pub const BG_CARD: Color32 = Color32::from_rgb(20, 25, 37); // #141925
    pub const BG_CARD_HOVER: Color32 = Color32::from_rgb(27, 34, 50); // #1B2232
    pub const BG_INPUT: Color32 = Color32::from_rgb(15, 18, 26); // #0F121A
    pub const BG_BADGE: Color32 = Color32::from_rgb(26, 33, 49); // #1A2131

    // Borders
    pub const BORDER_SUBTLE: Color32 = Color32::from_rgb(34, 42, 60); // #222A3C
    pub const BORDER_FOCUS: Color32 = Color32::from_rgb(56, 189, 248); // #38BDF8
    pub const BORDER_ACTIVE: Color32 = Color32::from_rgb(99, 102, 241); // #6366F1

    // Text
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(248, 250, 252); // #F8FAFC
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(148, 163, 184); // #94A3B8
    pub const TEXT_FAINT: Color32 = Color32::from_rgb(100, 116, 139); // #64748B

    // Accents
    pub const ACCENT_CYAN: Color32 = Color32::from_rgb(56, 189, 248); // #38BDF8
    pub const ACCENT_INDIGO: Color32 = Color32::from_rgb(99, 102, 241); // #6366F1
    #[allow(dead_code)]
    pub const ACCENT_PURPLE: Color32 = Color32::from_rgb(168, 85, 247); // #A855F7

    // Statuses
    pub const SUCCESS_GREEN: Color32 = Color32::from_rgb(52, 211, 153); // #34D399
    pub const SUCCESS_BG: Color32 = Color32::from_rgb(6, 78, 59); // #064E3B
    pub const ERROR_RED: Color32 = Color32::from_rgb(248, 113, 113); // #F87171

    pub fn apply(ctx: &egui::Context) {
        let mut visuals = Visuals::dark();

        visuals.override_text_color = Some(Self::TEXT_PRIMARY);
        visuals.panel_fill = Self::BG_CANVAS;
        visuals.window_fill = Self::BG_SURFACE;
        visuals.extreme_bg_color = Self::BG_INPUT;

        // Non-interactive widgets
        visuals.widgets.noninteractive.bg_fill = Self::BG_SURFACE;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Self::BORDER_SUBTLE);
        visuals.widgets.noninteractive.corner_radius = CornerRadius::same(8);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Self::TEXT_PRIMARY);

        // Inactive widgets (buttons, chips)
        visuals.widgets.inactive.bg_fill = Self::BG_CARD;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Self::BORDER_SUBTLE);
        visuals.widgets.inactive.corner_radius = CornerRadius::same(8);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Self::TEXT_PRIMARY);

        // Hovered widgets
        visuals.widgets.hovered.bg_fill = Self::BG_CARD_HOVER;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Self::ACCENT_CYAN);
        visuals.widgets.hovered.corner_radius = CornerRadius::same(8);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::WHITE);

        // Active / Pressed widgets
        visuals.widgets.active.bg_fill = Self::BG_CARD_HOVER;
        visuals.widgets.active.bg_stroke = Stroke::new(1.5, Self::ACCENT_INDIGO);
        visuals.widgets.active.corner_radius = CornerRadius::same(8);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);

        // Open menu widgets
        visuals.widgets.open.bg_fill = Self::BG_CARD;
        visuals.widgets.open.bg_stroke = Stroke::new(1.0, Self::ACCENT_CYAN);
        visuals.widgets.open.corner_radius = CornerRadius::same(8);

        // Selection
        visuals.selection.bg_fill = Color32::from_rgba_premultiplied(56, 189, 248, 60);
        visuals.selection.stroke = Stroke::new(1.0, Self::ACCENT_CYAN);

        // Shadows & windows
        visuals.window_shadow = Shadow::NONE;
        visuals.popup_shadow = Shadow::NONE;

        let mut style = Style::default();
        style.visuals = visuals;
        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        style.spacing.window_margin = Margin::same(16);
        style.spacing.button_padding = egui::vec2(12.0, 6.0);

        ctx.set_style(style);
    }
}
