use egui::{
    epaint::Shadow, Color32, CornerRadius, Margin, Pos2, Stroke, Style, Visuals,
};

/// Material Design 3 (Material You) Design System Tokens & Surfaces
/// https://m3.material.io/styles/color/system/overview
pub struct Material3Theme;

impl Material3Theme {
    // 8dp Grid Spacing Tokens
    #[allow(dead_code)]
    pub const SPACE_XS: f32 = 4.0;
    #[allow(dead_code)]
    pub const SPACE_SM: f32 = 8.0;
    pub const SPACE_MD: f32 = 16.0;
    pub const SPACE_LG: f32 = 24.0;
    #[allow(dead_code)]
    pub const SPACE_XL: f32 = 32.0;

    // Surface Tonal Elevation (Dark Mode)
    #[allow(dead_code)]
    pub const SURFACE_DIM: Color32 = Color32::from_rgb(15, 17, 23); // #0F1117 Canvas Base
    pub const SURFACE: Color32 = Color32::from_rgb(19, 22, 31); // #13161F Window Base
    pub const SURFACE_CONTAINER_LOW: Color32 = Color32::from_rgb(23, 27, 38); // #171B26 App Bar / Panels
    pub const SURFACE_CONTAINER: Color32 = Color32::from_rgb(29, 34, 48); // #1D2230 Card Base
    pub const SURFACE_CONTAINER_HIGH: Color32 = Color32::from_rgb(36, 42, 59); // #242A3B Hovered Cards / Inputs
    pub const SURFACE_CONTAINER_HIGHEST: Color32 = Color32::from_rgb(43, 50, 70); // #2B3246 Active controls

    // Outlines & Dividers
    #[allow(dead_code)]
    pub const OUTLINE: Color32 = Color32::from_rgb(117, 127, 148); // #757F94 Focused Outlines
    pub const OUTLINE_VARIANT: Color32 = Color32::from_rgb(54, 62, 82); // #363E52 Subtle Tonal Outlines

    // Primary (Electric Sky Blue / Cyan)
    pub const PRIMARY: Color32 = Color32::from_rgb(56, 189, 248); // #38BDF8
    pub const ON_PRIMARY: Color32 = Color32::from_rgb(0, 52, 77); // #00344D High Contrast Dark
    pub const PRIMARY_CONTAINER: Color32 = Color32::from_rgb(0, 76, 109); // #004C6D Tonal Button Fill
    pub const ON_PRIMARY_CONTAINER: Color32 = Color32::from_rgb(198, 231, 255); // #C6E7FF

    // Secondary / Tonal Chips
    pub const SECONDARY_CONTAINER: Color32 = Color32::from_rgb(39, 50, 68); // #273244 Active Chip Fill
    pub const ON_SECONDARY_CONTAINER: Color32 = Color32::from_rgb(212, 227, 245); // #D4E3F5

    // Success (Emerald Green)
    pub const SUCCESS: Color32 = Color32::from_rgb(78, 222, 156); // #4EDE9C
    pub const SUCCESS_CONTAINER: Color32 = Color32::from_rgb(18, 56, 38); // #123826
    pub const ON_SUCCESS_CONTAINER: Color32 = Color32::from_rgb(168, 245, 208); // #A8F5D0

    // Error (Coral)
    pub const ERROR: Color32 = Color32::from_rgb(255, 137, 125); // #FF897D
    pub const ERROR_CONTAINER: Color32 = Color32::from_rgb(78, 12, 6); // #4E0C06
    pub const ON_ERROR_CONTAINER: Color32 = Color32::from_rgb(255, 218, 214); // #FFDAD6

    // Material 3 Typography Color Hierarchy
    pub const ON_SURFACE: Color32 = Color32::from_rgb(241, 245, 249); // #F1F5F9 Title / High Emphasis
    pub const ON_SURFACE_VARIANT: Color32 = Color32::from_rgb(148, 163, 184); // #94A3B8 Secondary / Body Medium
    pub const ON_SURFACE_MUTED: Color32 = Color32::from_rgb(100, 116, 139); // #64748B Subtle Hints

    /// Applies Material 3 design tokens to the egui Context
    pub fn apply(ctx: &egui::Context) {
        let mut visuals = Visuals::dark();

        visuals.override_text_color = Some(Self::ON_SURFACE);
        visuals.panel_fill = Self::SURFACE;
        visuals.window_fill = Self::SURFACE;
        visuals.extreme_bg_color = Self::SURFACE_CONTAINER_LOW;

        // Non-interactive widgets
        visuals.widgets.noninteractive.bg_fill = Self::SURFACE_CONTAINER;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Self::OUTLINE_VARIANT);
        visuals.widgets.noninteractive.corner_radius = CornerRadius::same(12);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Self::ON_SURFACE);

        // Inactive widgets (chips, inputs)
        visuals.widgets.inactive.bg_fill = Self::SURFACE_CONTAINER_LOW;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Self::OUTLINE_VARIANT);
        visuals.widgets.inactive.corner_radius = CornerRadius::same(8);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Self::ON_SURFACE_VARIANT);

        // Hovered widgets
        visuals.widgets.hovered.bg_fill = Self::SURFACE_CONTAINER_HIGH;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Self::PRIMARY);
        visuals.widgets.hovered.corner_radius = CornerRadius::same(8);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, Self::ON_SURFACE);

        // Active / Pressed widgets
        visuals.widgets.active.bg_fill = Self::SURFACE_CONTAINER_HIGHEST;
        visuals.widgets.active.bg_stroke = Stroke::new(1.5, Self::PRIMARY);
        visuals.widgets.active.corner_radius = CornerRadius::same(8);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);

        // Selection
        visuals.selection.bg_fill = Color32::from_rgba_premultiplied(10, 50, 75, 120);
        visuals.selection.stroke = Stroke::new(1.0, Self::PRIMARY);

        visuals.window_shadow = Shadow::NONE;
        visuals.popup_shadow = Shadow::NONE;

        let mut style = Style::default();
        style.visuals = visuals;
        style.spacing.item_spacing = egui::vec2(Self::SPACE_MD, Self::SPACE_MD);
        style.spacing.window_margin = Margin::symmetric(Self::SPACE_LG as i8, Self::SPACE_MD as i8);
        style.spacing.button_padding = egui::vec2(16.0, 8.0);

        ctx.set_style(style);
    }

    /// Draws a crisp vector magnifying glass icon with zero font dependency
    pub fn paint_search_icon(painter: &egui::Painter, center: Pos2, color: Color32) {
        let circle_center = Pos2::new(center.x - 2.0, center.y - 2.0);
        painter.circle_stroke(circle_center, 5.0, Stroke::new(1.5, color));
        painter.line_segment(
            [
                Pos2::new(center.x + 1.5, center.y + 1.5),
                Pos2::new(center.x + 6.0, center.y + 6.0),
            ],
            Stroke::new(1.5, color),
        );
    }
}


