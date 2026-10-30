use egui::{
    epaint::{Shadow, Vertex},
    Color32, CornerRadius, FontId, Margin, Mesh, Pos2, Rect, Sense, Shape, Stroke, StrokeKind,
    Style, Ui, Visuals, vec2,
};

/// Antigravity Cosmic & Iridescent Design System (Google Antigravity & Stitch Dark Mode)
pub struct AntigravityTheme;

impl AntigravityTheme {
    // Cosmic Void & Surface Palette
    pub const VOID_BG: Color32 = Color32::from_rgb(8, 10, 15); // #080A0F Deep Space Void
    pub const SURFACE_GLASS: Color32 = Color32::from_rgb(13, 16, 24); // #0D1018 Frosted Glass
    pub const CARD_BG: Color32 = Color32::from_rgb(16, 20, 30); // #10141E Obsidian Specimen Card
    pub const CARD_HOVER_BG: Color32 = Color32::from_rgb(22, 28, 42); // #161C2A Luminous Card
    pub const CONTROL_BG: Color32 = Color32::from_rgb(18, 23, 34); // #121722 Inputs & Dock

    // Iridescent Accents (Prismatic Google Antigravity)
    pub const IRIS_CYAN: Color32 = Color32::from_rgb(34, 211, 238); // #22D3EE Electric Cyan
    #[allow(dead_code)]
    pub const IRIS_BLUE: Color32 = Color32::from_rgb(56, 189, 248); // #38BDF8 Sky Glow
    pub const IRIS_INDIGO: Color32 = Color32::from_rgb(99, 102, 241); // #6366F1 Cosmic Indigo
    pub const IRIS_VIOLET: Color32 = Color32::from_rgb(168, 85, 247); // #A855F7 Neon Violet
    pub const IRIS_PINK: Color32 = Color32::from_rgb(244, 63, 94); // #F43F5E Iridescent Rose

    // Status Accents
    pub const SUCCESS_MINT: Color32 = Color32::from_rgb(52, 211, 153); // #34D399 Mint Green
    pub const ERROR_ROSE: Color32 = Color32::from_rgb(251, 113, 133); // #FB7185 Coral Rose

    // Translucent Tint Overlays (Mathematically Valid Premultiplied Alpha)
    pub const TINT_CYAN: Color32 = Color32::from_rgba_premultiplied(4, 25, 28, 30);
    pub const TINT_VIOLET: Color32 = Color32::from_rgba_premultiplied(20, 10, 29, 30);
    pub const TINT_PINK: Color32 = Color32::from_rgba_premultiplied(29, 7, 11, 30);
    pub const TINT_SUCCESS: Color32 = Color32::from_rgba_premultiplied(6, 25, 18, 30);
    #[allow(dead_code)]
    pub const TINT_SURFACE: Color32 = Color32::from_rgba_premultiplied(12, 12, 12, 12);

    // Subtle Borders & Outlines
    pub const BORDER_IDLE: Color32 = Color32::from_rgba_premultiplied(18, 18, 18, 18);
    pub const BORDER_SUBTLE: Color32 = Color32::from_rgba_premultiplied(23, 25, 29, 40);
    #[allow(dead_code)]
    pub const BORDER_FOCUS: Color32 = Color32::from_rgb(56, 189, 248);

    // High Legibility Typography
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(248, 250, 252); // #F8FAFC Crisp White
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(148, 163, 184); // #94A3B8 Slate Gray
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(100, 116, 139); // #64748B Subtle Muted

    /// Applies the Cosmic Iridescent theme to the egui Context
    pub fn apply(ctx: &egui::Context) {
        let mut visuals = Visuals::dark();

        visuals.override_text_color = Some(Self::TEXT_PRIMARY);
        visuals.panel_fill = Self::VOID_BG;
        visuals.window_fill = Self::VOID_BG;
        visuals.extreme_bg_color = Self::CONTROL_BG;

        // Non-interactive widgets
        visuals.widgets.noninteractive.bg_fill = Self::CARD_BG;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Self::BORDER_IDLE);
        visuals.widgets.noninteractive.corner_radius = CornerRadius::same(12);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Self::TEXT_PRIMARY);

        // Inactive widgets (chips, inputs)
        visuals.widgets.inactive.bg_fill = Self::CONTROL_BG;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Self::BORDER_SUBTLE);
        visuals.widgets.inactive.corner_radius = CornerRadius::same(12);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Self::TEXT_SECONDARY);

        // Hovered widgets
        visuals.widgets.hovered.bg_fill = Self::CARD_HOVER_BG;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Self::IRIS_CYAN);
        visuals.widgets.hovered.corner_radius = CornerRadius::same(12);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, Self::TEXT_PRIMARY);

        // Active / Pressed widgets
        visuals.widgets.active.bg_fill = Self::CARD_HOVER_BG;
        visuals.widgets.active.bg_stroke = Stroke::new(1.5, Self::IRIS_VIOLET);
        visuals.widgets.active.corner_radius = CornerRadius::same(12);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);

        // Selection
        visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(56, 189, 248, 50);
        visuals.selection.stroke = Stroke::new(1.0, Self::IRIS_CYAN);

        visuals.window_shadow = Shadow::NONE;
        visuals.popup_shadow = Shadow::NONE;

        let mut style = Style::default();
        style.visuals = visuals;
        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        style.spacing.window_margin = Margin::symmetric(24, 16);
        style.spacing.button_padding = egui::vec2(12.0, 6.0);

        ctx.set_style(style);
    }

    /// Renders a Google-style rounded pill badge with crisp typography and subtle alpha blending
    pub fn render_badge(ui: &mut Ui, text: &str, fg: Color32, bg: Color32, border: Color32) {
        let padding = vec2(7.0, 2.5);
        let font_id = FontId::proportional(10.5);
        let text_galley = ui.painter().layout_no_wrap(text.to_string(), font_id, fg);
        let size = text_galley.size() + padding * 2.0;
        let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
        let painter = ui.painter();
        let radius = CornerRadius::same((size.y * 0.5) as u8);
        painter.rect_filled(rect, radius, bg);
        painter.rect_stroke(rect, radius, Stroke::new(1.0, border), StrokeKind::Inside);
        painter.galley(rect.min + padding, text_galley, fg);
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

    /// Paints a glowing horizontal iridescent gradient line (Cyan -> Violet -> Pink)
    pub fn paint_iridescent_line(painter: &egui::Painter, rect: Rect) {
        let width = rect.width();
        let mid_x = rect.min.x + width * 0.5;

        // Left half: Cyan to Violet
        let mut mesh1 = Mesh::default();
        let idx1 = mesh1.vertices.len() as u32;
        mesh1.vertices.push(Vertex { pos: rect.left_top(), uv: Pos2::ZERO, color: Self::IRIS_CYAN });
        mesh1.vertices.push(Vertex { pos: Pos2::new(mid_x, rect.min.y), uv: Pos2::ZERO, color: Self::IRIS_VIOLET });
        mesh1.vertices.push(Vertex { pos: Pos2::new(mid_x, rect.max.y), uv: Pos2::ZERO, color: Self::IRIS_VIOLET });
        mesh1.vertices.push(Vertex { pos: rect.left_bottom(), uv: Pos2::ZERO, color: Self::IRIS_CYAN });
        mesh1.add_triangle(idx1, idx1 + 1, idx1 + 2);
        mesh1.add_triangle(idx1, idx1 + 2, idx1 + 3);
        painter.add(Shape::mesh(mesh1));

        // Right half: Violet to Pink
        let mut mesh2 = Mesh::default();
        let idx2 = mesh2.vertices.len() as u32;
        mesh2.vertices.push(Vertex { pos: Pos2::new(mid_x, rect.min.y), uv: Pos2::ZERO, color: Self::IRIS_VIOLET });
        mesh2.vertices.push(Vertex { pos: rect.right_top(), uv: Pos2::ZERO, color: Self::IRIS_PINK });
        mesh2.vertices.push(Vertex { pos: rect.right_bottom(), uv: Pos2::ZERO, color: Self::IRIS_PINK });
        mesh2.vertices.push(Vertex { pos: Pos2::new(mid_x, rect.max.y), uv: Pos2::ZERO, color: Self::IRIS_VIOLET });
        mesh2.add_triangle(idx2, idx2 + 1, idx2 + 2);
        mesh2.add_triangle(idx2, idx2 + 2, idx2 + 3);
        painter.add(Shape::mesh(mesh2));
    }
}

