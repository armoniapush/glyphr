use egui::{
    epaint::Shadow, Color32, CornerRadius, Margin, Stroke, Style, Visuals,
};

/// Material 3 (Material You) Design System Tokens for Google Antigravity
/// Based on https://m3.material.io/styles/color/system/overview
pub struct Material3Theme;

impl Material3Theme {
    // Surface Tonal Elevation (Dark Mode)
    pub const SURFACE_DIM: Color32 = Color32::from_rgb(15, 17, 23); // #0F1117 Canvas Base
    pub const SURFACE: Color32 = Color32::from_rgb(20, 23, 31); // #14171F Window Base
    pub const SURFACE_CONTAINER_LOW: Color32 = Color32::from_rgb(24, 28, 38); // #181C26 Subtle Panels
    pub const SURFACE_CONTAINER: Color32 = Color32::from_rgb(30, 36, 49); // #1E2431 Standard Cards
    pub const SURFACE_CONTAINER_HIGH: Color32 = Color32::from_rgb(38, 46, 62); // #262E3E Hovered Cards
    pub const SURFACE_CONTAINER_HIGHEST: Color32 = Color32::from_rgb(46, 56, 76); // #2E384C Search / Active Elements

    // Outlines & Dividers
    #[allow(dead_code)]
    pub const OUTLINE: Color32 = Color32::from_rgb(71, 85, 112); // #475570 Focused Outline
    pub const OUTLINE_VARIANT: Color32 = Color32::from_rgb(42, 50, 68); // #2A3244 Subtle Card Outline

    // Primary Accents (Electric Antigravity Cyan / M3 Blue)
    pub const PRIMARY: Color32 = Color32::from_rgb(56, 189, 248); // #38BDF8
    pub const ON_PRIMARY: Color32 = Color32::from_rgb(11, 20, 36); // #0B1424 High Contrast Dark
    pub const PRIMARY_CONTAINER: Color32 = Color32::from_rgb(14, 58, 92); // #0E3A5C Tonal Container
    pub const ON_PRIMARY_CONTAINER: Color32 = Color32::from_rgb(224, 242, 254); // #E0F2FE

    // Secondary Accents (Indigo / Neon Purple)
    #[allow(dead_code)]
    pub const SECONDARY: Color32 = Color32::from_rgb(165, 180, 252); // #A5B4FC
    pub const SECONDARY_CONTAINER: Color32 = Color32::from_rgb(49, 46, 129); // #312E81
    pub const ON_SECONDARY_CONTAINER: Color32 = Color32::from_rgb(224, 231, 255); // #E0E7FF

    // Success (Emerald Green)
    pub const SUCCESS: Color32 = Color32::from_rgb(52, 211, 153); // #34D399
    pub const SUCCESS_CONTAINER: Color32 = Color32::from_rgb(6, 78, 59); // #064E3B
    pub const ON_SUCCESS_CONTAINER: Color32 = Color32::from_rgb(209, 250, 229); // #D1FAE5

    // Error (Crimson)
    pub const ERROR: Color32 = Color32::from_rgb(248, 113, 113); // #F87171
    pub const ERROR_CONTAINER: Color32 = Color32::from_rgb(127, 29, 29); // #7F1D1D
    pub const ON_ERROR_CONTAINER: Color32 = Color32::from_rgb(254, 226, 226); // #FEE2E2

    // Neutral Text Hierarchy
    pub const ON_SURFACE: Color32 = Color32::from_rgb(248, 250, 252); // #F8FAFC High Emphasis Title
    pub const ON_SURFACE_VARIANT: Color32 = Color32::from_rgb(160, 174, 192); // #A0AEC0 Medium Emphasis Body
    pub const ON_SURFACE_MUTED: Color32 = Color32::from_rgb(100, 116, 139); // #64748B Subtle Hints

    /// Applies Material 3 styling tokens to the egui Context
    pub fn apply(ctx: &egui::Context) {
        let mut visuals = Visuals::dark();

        visuals.override_text_color = Some(Self::ON_SURFACE);
        visuals.panel_fill = Self::SURFACE_DIM;
        visuals.window_fill = Self::SURFACE;
        visuals.extreme_bg_color = Self::SURFACE_CONTAINER_LOW;

        // Non-interactive widgets
        visuals.widgets.noninteractive.bg_fill = Self::SURFACE_CONTAINER;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Self::OUTLINE_VARIANT);
        visuals.widgets.noninteractive.corner_radius = CornerRadius::same(12);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Self::ON_SURFACE);

        // Inactive widgets (buttons, chips)
        visuals.widgets.inactive.bg_fill = Self::SURFACE_CONTAINER;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Self::OUTLINE_VARIANT);
        visuals.widgets.inactive.corner_radius = CornerRadius::same(12);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Self::ON_SURFACE);

        // Hovered widgets
        visuals.widgets.hovered.bg_fill = Self::SURFACE_CONTAINER_HIGH;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Self::PRIMARY);
        visuals.widgets.hovered.corner_radius = CornerRadius::same(12);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::WHITE);

        // Active / Pressed widgets
        visuals.widgets.active.bg_fill = Self::SURFACE_CONTAINER_HIGHEST;
        visuals.widgets.active.bg_stroke = Stroke::new(1.5, Self::PRIMARY);
        visuals.widgets.active.corner_radius = CornerRadius::same(12);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);

        // Open menu widgets
        visuals.widgets.open.bg_fill = Self::SURFACE_CONTAINER;
        visuals.widgets.open.bg_stroke = Stroke::new(1.0, Self::PRIMARY);
        visuals.widgets.open.corner_radius = CornerRadius::same(12);

        // Selection
        visuals.selection.bg_fill = Color32::from_rgba_premultiplied(56, 189, 248, 50);
        visuals.selection.stroke = Stroke::new(1.0, Self::PRIMARY);

        visuals.window_shadow = Shadow::NONE;
        visuals.popup_shadow = Shadow::NONE;

        let mut style = Style::default();
        style.visuals = visuals;
        style.spacing.item_spacing = egui::vec2(12.0, 12.0);
        style.spacing.window_margin = Margin::same(18);
        style.spacing.button_padding = egui::vec2(16.0, 8.0);

        ctx.set_style(style);
    }
}
