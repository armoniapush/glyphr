use egui::{
    epaint::Shadow, Color32, CornerRadius, Margin, Pos2, Stroke, Style, Visuals,
};

/// Material Design 3 (Material You) Design System Tokens & Surfaces
/// Enhanced for Deep Cosmic Dark Mode (Antigravity Obsidian Aesthetic)
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

    // Deep Cosmic Obsidian Dark Mode Surfaces (#090A0F Void -> #141722 Cards)
    #[allow(dead_code)]
    pub const SURFACE_DIM: Color32 = Color32::from_rgb(7, 8, 12); // #07080C Pure Deep Void
    pub const SURFACE: Color32 = Color32::from_rgb(10, 11, 16); // #0A0B10 Window Canvas
    pub const SURFACE_CONTAINER_LOW: Color32 = Color32::from_rgb(15, 17, 24); // #0F1118 Title bar & dock panels
    pub const SURFACE_CONTAINER: Color32 = Color32::from_rgb(20, 23, 33); // #141721 Idle Card Base
    pub const SURFACE_CONTAINER_HIGH: Color32 = Color32::from_rgb(27, 32, 45); // #1B202D Hovered Cards / Inputs
    pub const SURFACE_CONTAINER_HIGHEST: Color32 = Color32::from_rgb(36, 43, 60); // #242B3C Active controls / Badges

    // Outlines & Dividers
    #[allow(dead_code)]
    pub const OUTLINE: Color32 = Color32::from_rgb(90, 102, 125); // #5A667D Active outlines
    pub const OUTLINE_VARIANT: Color32 = Color32::from_rgb(33, 38, 52); // #212634 Crisp Hairline Outlines

    // Primary (Electric Sky Cyan / Antigravity Luminous Blue)
    pub const PRIMARY: Color32 = Color32::from_rgb(56, 189, 248); // #38BDF8
    pub const ON_PRIMARY: Color32 = Color32::from_rgb(2, 44, 67); // #022C43 High Contrast Dark
    pub const PRIMARY_CONTAINER: Color32 = Color32::from_rgb(14, 52, 75); // #0E344B Tonal Button Fill
    pub const ON_PRIMARY_CONTAINER: Color32 = Color32::from_rgb(186, 230, 253); // #BAE6FD

    // Accent (Antigravity Iridescent Violet)
    pub const ACCENT_PURPLE: Color32 = Color32::from_rgb(168, 85, 247); // #A855F7
    #[allow(dead_code)]
    pub const ACCENT_PURPLE_CONTAINER: Color32 = Color32::from_rgb(59, 22, 94); // #3B165E
    #[allow(dead_code)]
    pub const ON_ACCENT_PURPLE: Color32 = Color32::from_rgb(243, 232, 255); // #F3E8FF

    // Secondary / Tonal Chips
    pub const SECONDARY_CONTAINER: Color32 = Color32::from_rgb(26, 35, 52); // #1A2334 Active Chip Fill
    pub const ON_SECONDARY_CONTAINER: Color32 = Color32::from_rgb(203, 223, 245); // #CBDEF5

    // Success (Emerald Green)
    pub const SUCCESS: Color32 = Color32::from_rgb(52, 211, 153); // #34D399
    pub const SUCCESS_CONTAINER: Color32 = Color32::from_rgb(12, 44, 30); // #0C2C1E
    pub const ON_SUCCESS_CONTAINER: Color32 = Color32::from_rgb(167, 243, 208); // #A7F3D0

    // Error (Coral)
    pub const ERROR: Color32 = Color32::from_rgb(251, 113, 133); // #FB7185
    pub const ERROR_CONTAINER: Color32 = Color32::from_rgb(76, 14, 25); // #4C0E19
    pub const ON_ERROR_CONTAINER: Color32 = Color32::from_rgb(254, 205, 211); // #FECDD3

    // Material 3 Typography Color Hierarchy
    pub const ON_SURFACE: Color32 = Color32::from_rgb(248, 250, 252); // #F8FAFC High Emphasis Title & Specimen
    pub const ON_SURFACE_VARIANT: Color32 = Color32::from_rgb(148, 163, 184); // #94A3B8 Secondary / Body Medium
    pub const ON_SURFACE_MUTED: Color32 = Color32::from_rgb(100, 116, 139); // #64748B Subtle Hints

    /// Smooth linear color interpolation for 60fps micro-interaction animations
    pub fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
        let factor = t.clamp(0.0, 1.0);
        let r = (a.r() as f32 + (b.r() as f32 - a.r() as f32) * factor).round() as u8;
        let g = (a.g() as f32 + (b.g() as f32 - a.g() as f32) * factor).round() as u8;
        let b_val = (a.b() as f32 + (b.b() as f32 - a.b() as f32) * factor).round() as u8;
        let alpha = (a.a() as f32 + (b.a() as f32 - a.a() as f32) * factor).round() as u8;
        Color32::from_rgba_unmultiplied(r, g, b_val, alpha)
    }

    /// Applies Deep Dark Material 3 design tokens to the egui Context
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
        visuals.selection.bg_fill = Color32::from_rgba_premultiplied(14, 52, 75, 160);
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

    /// Draws an Antigravity iridescent gradient divider line
    pub fn paint_iridescent_line(painter: &egui::Painter, start: Pos2, end: Pos2) {
        let width = end.x - start.x;
        if width <= 0.0 {
            return;
        }

        // Draw segmented line with iridescent gradient from cyan -> purple -> subtle dark
        let segments = 24;
        let step = width / (segments as f32);
        for i in 0..segments {
            let x0 = start.x + (i as f32) * step;
            let x1 = (x0 + step).min(end.x);
            let t = (i as f32) / (segments as f32);

            let color = if t < 0.4 {
                let local_t = t / 0.4;
                Self::lerp_color(Self::PRIMARY, Self::ACCENT_PURPLE, local_t)
            } else if t < 0.8 {
                let local_t = (t - 0.4) / 0.4;
                Self::lerp_color(Self::ACCENT_PURPLE, Self::OUTLINE_VARIANT, local_t)
            } else {
                Self::OUTLINE_VARIANT
            };

            painter.line_segment([Pos2::new(x0, start.y), Pos2::new(x1, start.y)], Stroke::new(1.0, color));
        }
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


