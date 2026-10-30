use crate::catalog::CatalogClient;
use crate::installer::FontInstaller;
use crate::models::{CategoryFilter, FontItem, InstallState};
use crate::theme::Material3Theme;
use egui::{
    vec2, Align, Align2, Color32, CornerRadius, FontId, Layout, Margin, Pos2, Rect, RichText,
    ScrollArea, Sense, Stroke, StrokeKind, Ui,
};
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;

enum WorkerCmd {
    FetchCatalog,
    InstallFont { id: String, family: String },
    FetchPreview { id: String, family: String },
}

enum WorkerEvent {
    CatalogLoaded(Vec<FontItem>),
    CatalogFailed(String),
    InstallProgress { id: String, state: InstallState },
    InstallDone { id: String, result: Result<(), String> },
    PreviewLoaded { id: String, bytes: Vec<u8> },
}

pub struct AntigravityFontsApp {
    // Fonts data
    all_fonts: Vec<FontItem>,
    filtered_indices: Vec<usize>,
    installed_families: HashSet<String>,
    font_states: HashMap<String, InstallState>,

    // Dynamic Font Definitions for real typographic rendering
    font_definitions: egui::FontDefinitions,
    active_fonts: HashSet<String>,
    pending_fonts: HashSet<String>,
    requested_previews: HashSet<String>,

    // Search and filters
    search_query: String,
    selected_category: CategoryFilter,
    preview_text: String,
    preview_size: f32,

    // Status & Micro-interactions Toast
    status_message: String,
    is_loading_catalog: bool,
    toast_notification: Option<(String, std::time::Instant)>,

    // Worker channels
    tx_worker: Sender<WorkerCmd>,
    rx_ui: Receiver<WorkerEvent>,
}

impl AntigravityFontsApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Material3Theme::apply(&cc.egui_ctx);

        let (tx_worker, rx_worker) = channel::<WorkerCmd>();
        let (tx_ui, rx_ui) = channel::<WorkerEvent>();

        let catalog_client = Arc::new(CatalogClient::new());

        // Spawn Tokio worker thread for all asynchronous network and filesystem tasks
        let client_clone = catalog_client.clone();
        let tx_ui_clone = tx_ui.clone();
        std::thread::spawn(move || {
            let rt = match tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(e) => {
                    let _ = tx_ui_clone.send(WorkerEvent::CatalogFailed(format!(
                        "Failed initializing async runtime: {e}"
                    )));
                    return;
                }
            };

            rt.block_on(async move {
                // First check local cache for 0ms startup
                if let Some(cached) = client_clone.load_from_cache() {
                    let _ = tx_ui_clone.send(WorkerEvent::CatalogLoaded(cached));
                }

                // If cache is stale or missing, fetch in background
                if client_clone.is_cache_stale() {
                    match client_clone.fetch_and_cache().await {
                        Ok(fonts) => {
                            let _ = tx_ui_clone.send(WorkerEvent::CatalogLoaded(fonts));
                        }
                        Err(e) => {
                            let _ = tx_ui_clone.send(WorkerEvent::CatalogFailed(e));
                        }
                    }
                }

                // Handle incoming worker commands
                while let Ok(cmd) = rx_worker.recv() {
                    match cmd {
                        WorkerCmd::FetchCatalog => {
                            match client_clone.fetch_and_cache().await {
                                Ok(fonts) => {
                                    let _ = tx_ui_clone.send(WorkerEvent::CatalogLoaded(fonts));
                                }
                                Err(e) => {
                                    let _ = tx_ui_clone.send(WorkerEvent::CatalogFailed(e));
                                }
                            }
                        }
                        WorkerCmd::InstallFont { id, family } => {
                            let _ = tx_ui_clone.send(WorkerEvent::InstallProgress {
                                id: id.clone(),
                                state: InstallState::Downloading,
                            });

                            match client_clone.download_font_zip(&id).await {
                                Ok(zip_bytes) => {
                                    let _ = tx_ui_clone.send(WorkerEvent::InstallProgress {
                                        id: id.clone(),
                                        state: InstallState::Installing,
                                    });

                                    let install_result =
                                        FontInstaller::install_from_zip(&zip_bytes, &family)
                                            .map(|_| ());

                                    let _ = tx_ui_clone.send(WorkerEvent::InstallDone {
                                        id,
                                        result: install_result,
                                    });
                                }
                                Err(err) => {
                                    let _ = tx_ui_clone.send(WorkerEvent::InstallDone {
                                        id,
                                        result: Err(err),
                                    });
                                }
                            }
                        }
                        WorkerCmd::FetchPreview { id, family } => {
                            if let Ok(bytes) = client_clone.fetch_preview_font(&id, &family).await {
                                let _ = tx_ui_clone.send(WorkerEvent::PreviewLoaded { id, bytes });
                            }
                        }
                    }
                }
            });
        });

        let font_definitions = egui::FontDefinitions::default();

        let mut app = Self {
            all_fonts: Vec::new(),
            filtered_indices: Vec::new(),
            installed_families: HashSet::new(),
            font_states: HashMap::new(),
            font_definitions,
            active_fonts: HashSet::new(),
            pending_fonts: HashSet::new(),
            requested_previews: HashSet::new(),
            search_query: String::new(),
            selected_category: CategoryFilter::All,
            preview_text: "DANIELDOBLES".to_string(),
            preview_size: 24.0,
            status_message: "Initializing Google Fonts catalog...".to_string(),
            is_loading_catalog: true,
            toast_notification: None,
            tx_worker,
            rx_ui,
        };

        app.refresh_installed_cache();
        app
    }

    /// Scans installed fonts on the operating system
    fn refresh_installed_cache(&mut self) {
        let mut installed = HashSet::new();
        for font in &self.all_fonts {
            if FontInstaller::is_family_installed(&font.id, &font.family) {
                installed.insert(font.id.clone());
            }
        }
        self.installed_families = installed;
    }

    /// Applies active search query and category filter
    fn apply_filters(&mut self) {
        let query = self.search_query.trim().to_lowercase();
        let cat = self.selected_category;

        self.filtered_indices = self
            .all_fonts
            .iter()
            .enumerate()
            .filter_map(|(idx, font)| {
                let is_installed = self.installed_families.contains(&font.id);
                if !cat.matches(&font.category, is_installed) {
                    return None;
                }

                if query.is_empty() {
                    return Some(idx);
                }

                let matches_name = font.family.to_lowercase().contains(&query);
                let matches_id = font.id.to_lowercase().contains(&query);
                let matches_cat = font.category.to_lowercase().contains(&query);

                if matches_name || matches_id || matches_cat {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect();
    }

    /// Handles events coming back from background worker
    fn process_worker_events(&mut self, ctx: &egui::Context) {
        let mut needs_filter = false;
        let mut fonts_updated = false;

        while let Ok(event) = self.rx_ui.try_recv() {
            match event {
                WorkerEvent::CatalogLoaded(fonts) => {
                    self.all_fonts = fonts;
                    self.is_loading_catalog = false;
                    self.status_message = format!("{} fonts loaded", self.all_fonts.len());
                    self.refresh_installed_cache();
                    needs_filter = true;
                }
                WorkerEvent::CatalogFailed(err) => {
                    self.is_loading_catalog = false;
                    self.status_message = format!("Catalog sync warning: {err}");
                }
                WorkerEvent::InstallProgress { id, state } => {
                    self.font_states.insert(id, state);
                }
                WorkerEvent::InstallDone { id, result } => {
                    match result {
                        Ok(()) => {
                            self.installed_families.insert(id.clone());
                            self.font_states.insert(id.clone(), InstallState::Installed);
                            self.status_message = format!("Installed {id} successfully!");
                            self.toast_notification = Some((
                                format!("✓ Installed {id} in Windows!"),
                                std::time::Instant::now(),
                            ));

                            // Immediately queue preview fetch for newly installed font
                            if let Some(font_item) = self.all_fonts.iter().find(|f| f.id == id) {
                                let _ = self.tx_worker.send(WorkerCmd::FetchPreview {
                                    id: id.clone(),
                                    family: font_item.family.clone(),
                                });
                            }

                            needs_filter = true;
                        }
                        Err(e) => {
                            self.font_states
                                .insert(id.clone(), InstallState::Error(e.clone()));
                            self.status_message = format!("Failed installing {id}: {e}");
                        }
                    }
                }
                WorkerEvent::PreviewLoaded { id, bytes } => {
                    if ttf_parser::Face::parse(&bytes, 0).is_ok() {
                        self.font_definitions.font_data.insert(
                            id.clone(),
                            Arc::new(egui::FontData::from_owned(bytes)),
                        );
                        self.font_definitions.families.insert(
                            egui::FontFamily::Name(Arc::from(id.as_str())),
                            vec![id.clone(), "Ubuntu-Light".to_owned()],
                        );
                        self.pending_fonts.insert(id);
                        fonts_updated = true;
                    }
                }
            }
            ctx.request_repaint();
        }

        if fonts_updated {
            ctx.set_fonts(self.font_definitions.clone());
            ctx.request_repaint();
        }

        if needs_filter {
            self.apply_filters();
        }
    }

    /// Renders custom professional frameless Title Bar
    fn render_custom_title_bar(&mut self, ui: &mut Ui) {
        let height = 36.0;
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
        let painter = ui.painter();

        // Dark acrylic title bar background & bottom iridescent accent divider
        painter.rect_filled(rect, CornerRadius::ZERO, Material3Theme::SURFACE_CONTAINER_LOW);
        Material3Theme::paint_iridescent_line(
            painter,
            Pos2::new(rect.min.x, rect.max.y),
            Pos2::new(rect.max.x, rect.max.y),
        );

        // Window drag interaction across entire title bar
        let drag_resp = ui.interact(rect, ui.id().with("titlebar_drag"), Sense::click_and_drag());
        if drag_resp.dragged_by(egui::PointerButton::Primary) {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }
        if drag_resp.double_clicked() {
            let is_max = ui.input(|i| i.viewport().maximized.unwrap_or(false));
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Maximized(!is_max));
        }

        // Inner title bar items
        let inner_rect = rect.shrink2(vec2(12.0, 4.0));
        let mut bar_ui = ui.new_child(egui::UiBuilder::new().max_rect(inner_rect));

        bar_ui.horizontal(|ui| {
            // Logo emblem
            let logo_rect = Rect::from_center_size(
                Pos2::new(rect.min.x + 22.0, rect.center().y),
                vec2(22.0, 22.0),
            );
            ui.painter().rect_filled(
                logo_rect,
                CornerRadius::same(6),
                Material3Theme::PRIMARY_CONTAINER,
            );
            ui.painter().text(
                logo_rect.center(),
                Align2::CENTER_CENTER,
                "AG",
                FontId::proportional(11.0),
                Material3Theme::ON_PRIMARY_CONTAINER,
            );

            ui.add_space(24.0);

            ui.label(
                RichText::new("Antigravity Fonts")
                    .font(FontId::proportional(13.5))
                    .strong()
                    .color(Material3Theme::ON_SURFACE),
            );

            ui.add_space(4.0);

            let (status_color, status_text) = if self.is_loading_catalog {
                (Material3Theme::PRIMARY, "Syncing...")
            } else {
                (Material3Theme::SUCCESS, "Ready")
            };
            ui.label(
                RichText::new(format!("•  {status_text} ({} fonts)", self.all_fonts.len()))
                    .font(FontId::proportional(11.0))
                    .color(status_color),
            );

            // Right side: Sync button and Window controls
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                // Close button (Windows 11 style red hover)
                let close_resp = ui.add(
                    egui::Button::new(RichText::new("✕").font(FontId::proportional(12.0)))
                        .min_size(vec2(36.0, 26.0))
                        .frame(false),
                );
                if close_resp.hovered() {
                    ui.painter().rect_filled(
                        close_resp.rect,
                        CornerRadius::same(4),
                        Color32::from_rgb(196, 43, 28),
                    );
                    ui.painter().text(
                        close_resp.rect.center(),
                        Align2::CENTER_CENTER,
                        "✕",
                        FontId::proportional(12.0),
                        Color32::WHITE,
                    );
                }
                if close_resp.clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }

                // Maximize / Restore button
                let is_max = ui.input(|i| i.viewport().maximized.unwrap_or(false));
                let max_symbol = if is_max { "🗗" } else { "□" };
                if ui
                    .add(
                        egui::Button::new(
                            RichText::new(max_symbol).font(FontId::proportional(11.5)),
                        )
                        .min_size(vec2(36.0, 26.0))
                        .frame(false),
                    )
                    .clicked()
                {
                    ui.ctx()
                        .send_viewport_cmd(egui::ViewportCommand::Maximized(!is_max));
                }

                // Minimize button
                if ui
                    .add(
                        egui::Button::new(RichText::new("—").font(FontId::proportional(12.0)))
                            .min_size(vec2(36.0, 26.0))
                            .frame(false),
                    )
                    .clicked()
                {
                    ui.ctx()
                        .send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                }

                ui.add_space(8.0);

                // Sync button
                let sync_btn = egui::Button::new(
                    RichText::new("⟳ Sync")
                        .font(FontId::proportional(11.5))
                        .color(Material3Theme::ON_SURFACE_VARIANT),
                )
                .min_size(vec2(60.0, 24.0))
                .fill(Material3Theme::SURFACE_CONTAINER)
                .stroke(Stroke::new(1.0, Material3Theme::OUTLINE_VARIANT))
                .corner_radius(CornerRadius::same(12));

                if ui.add(sync_btn).clicked() {
                    self.is_loading_catalog = true;
                    self.status_message = "Refreshing catalog...".to_string();
                    let _ = self.tx_worker.send(WorkerCmd::FetchCatalog);
                }
            });
        });
    }

    /// Renders Integrated Search & Filter Ribbon
    fn render_integrated_toolbar(&mut self, ui: &mut Ui) {
        ui.add_space(8.0);

        // Row 1: Search Bar & Categories on a single neat line
        ui.horizontal(|ui| {
            // Compact M3 Search Bar
            let height = 34.0;
            let (rect, _) = ui.allocate_exact_size(vec2(190.0, height), Sense::hover());

            let painter = ui.painter();
            painter.rect_filled(
                rect,
                CornerRadius::same(17),
                Material3Theme::SURFACE_CONTAINER_HIGH,
            );
            painter.rect_stroke(
                rect,
                CornerRadius::same(17),
                Stroke::new(1.0, Material3Theme::OUTLINE_VARIANT),
                StrokeKind::Inside,
            );

            Material3Theme::paint_search_icon(
                &painter,
                Pos2::new(rect.min.x + 16.0, rect.center().y),
                Material3Theme::ON_SURFACE_VARIANT,
            );

            let inner_rect = Rect::from_min_max(
                Pos2::new(rect.min.x + 32.0, rect.min.y + 2.0),
                Pos2::new(rect.max.x - 26.0, rect.max.y - 2.0),
            );
            let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(inner_rect));
            let response = child_ui.add(
                egui::TextEdit::singleline(&mut self.search_query)
                    .hint_text("Search...")
                    .frame(false)
                    .font(FontId::proportional(13.0))
                    .text_color(Material3Theme::ON_SURFACE),
            );
            if response.changed() {
                self.apply_filters();
            }

            if !self.search_query.is_empty() {
                let clear_rect = Rect::from_center_size(
                    Pos2::new(rect.max.x - 14.0, rect.center().y),
                    vec2(16.0, 16.0),
                );
                if ui.allocate_rect(clear_rect, Sense::click()).clicked() {
                    self.search_query.clear();
                    self.apply_filters();
                }
                ui.painter().text(
                    clear_rect.center(),
                    Align2::CENTER_CENTER,
                    "✕",
                    FontId::proportional(10.0),
                    Material3Theme::ON_SURFACE_VARIANT,
                );
            }

            ui.add_space(6.0);

            // Filter chips on the same row
            let categories = [
                CategoryFilter::All,
                CategoryFilter::SansSerif,
                CategoryFilter::Serif,
                CategoryFilter::Display,
                CategoryFilter::Handwriting,
                CategoryFilter::Monospace,
                CategoryFilter::InstalledOnly,
            ];

            for cat in categories {
                let is_selected = self.selected_category == cat;
                let label = if cat == CategoryFilter::InstalledOnly {
                    format!("Installed ({})", self.installed_families.len())
                } else {
                    cat.as_str().to_string()
                };

                let chip_text = if is_selected {
                    format!("✓ {label}")
                } else {
                    label
                };

                let (bg, stroke, fg) = if is_selected {
                    (
                        Material3Theme::SECONDARY_CONTAINER,
                        Material3Theme::PRIMARY,
                        Material3Theme::ON_SECONDARY_CONTAINER,
                    )
                } else {
                    (
                        Material3Theme::SURFACE_CONTAINER_LOW,
                        Material3Theme::OUTLINE_VARIANT,
                        Material3Theme::ON_SURFACE_VARIANT,
                    )
                };

                let btn = egui::Button::new(
                    RichText::new(chip_text)
                        .font(FontId::proportional(11.5))
                        .color(fg),
                )
                .fill(bg)
                .stroke(Stroke::new(1.0, stroke))
                .corner_radius(CornerRadius::same(8));

                if ui.add(btn).clicked() {
                    self.selected_category = cat;
                    self.apply_filters();
                }
            }
        });

        ui.add_space(6.0);

        // Row 2: Specimen Controls Ribbon
        egui::Frame::NONE
            .fill(Material3Theme::SURFACE_CONTAINER_LOW)
            .stroke(Stroke::new(1.0, Material3Theme::OUTLINE_VARIANT))
            .corner_radius(CornerRadius::same(10))
            .inner_margin(Margin::symmetric(14, 6))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Specimen:")
                            .font(FontId::proportional(12.0))
                            .color(Material3Theme::ON_SURFACE_VARIANT),
                    );

                    ui.add(
                        egui::TextEdit::singleline(&mut self.preview_text)
                            .desired_width(200.0)
                            .font(FontId::proportional(13.0))
                            .text_color(Material3Theme::ON_SURFACE),
                    );

                    let presets = [
                        ("Aa", "The quick brown fox jumps over the lazy dog"),
                        ("123", "0123456789 !@#$%^&*()"),
                        ("Caps", "DANIEL DOBLES"),
                        ("Pangram", "Sphinx of black quartz, judge my vow"),
                    ];

                    for (label, text) in presets {
                        let btn = egui::Button::new(
                            RichText::new(label)
                                .font(FontId::proportional(11.0))
                                .color(Material3Theme::ON_SURFACE_VARIANT),
                        )
                        .fill(Material3Theme::SURFACE_CONTAINER)
                        .stroke(Stroke::new(1.0, Material3Theme::OUTLINE_VARIANT))
                        .corner_radius(CornerRadius::same(6));

                        if ui.add(btn).clicked() {
                            self.preview_text = text.to_string();
                        }
                    }

                    ui.add_space(10.0);

                    ui.label(
                        RichText::new("Size:")
                            .font(FontId::proportional(12.0))
                            .color(Material3Theme::ON_SURFACE_VARIANT),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.preview_size, 16.0..=56.0).show_value(false),
                    );

                    let size_text = RichText::new(format!("{:.0}px", self.preview_size))
                        .font(FontId::proportional(11.0))
                        .color(Material3Theme::ON_PRIMARY_CONTAINER);
                    ui.label(
                        egui::WidgetText::RichText(size_text)
                            .background_color(Material3Theme::PRIMARY_CONTAINER),
                    );
                });
            });

        ui.add_space(6.0);
    }

    /// Renders an individual Material 3 Card (Direct Frame show, guaranteed specimen visibility)
    fn render_font_card(&mut self, ui: &mut Ui, font: &FontItem) {
        let is_installed = self.installed_families.contains(&font.id);
        let current_state = self.font_states.get(&font.id).cloned();
        let card_id = ui.id().with("card").with(&font.id);

        let was_hovered = ui.data(|d| d.get_temp::<bool>(card_id).unwrap_or(false));
        let hover_t = ui.ctx().animate_bool(card_id, was_hovered);

        // Animated surface fill
        let card_fill = Material3Theme::lerp_color(
            Material3Theme::SURFACE_CONTAINER,
            Material3Theme::SURFACE_CONTAINER_HIGH,
            hover_t,
        );

        // Dynamic animated border stroke
        let is_busy = matches!(
            &current_state,
            Some(InstallState::Downloading) | Some(InstallState::Installing)
        );

        let border_stroke = if is_busy {
            let time = ui.input(|i| i.time);
            let pulse = ((time * 6.0).sin() * 0.5 + 0.5) as f32;
            ui.ctx().request_repaint();
            let stroke_col = Material3Theme::lerp_color(
                Material3Theme::PRIMARY,
                Material3Theme::ACCENT_PURPLE,
                pulse,
            );
            Stroke::new(1.5, stroke_col)
        } else if matches!(&current_state, Some(InstallState::Error(_))) {
            Stroke::new(1.5, Material3Theme::ERROR)
        } else if is_installed {
            let idle_green = Color32::from_rgb(18, 52, 36);
            let active_green = Material3Theme::SUCCESS;
            let stroke_col = Material3Theme::lerp_color(idle_green, active_green, hover_t);
            Stroke::new(1.0 + hover_t * 0.3, stroke_col)
        } else {
            let stroke_col = Material3Theme::lerp_color(
                Material3Theme::OUTLINE_VARIANT,
                Material3Theme::PRIMARY,
                hover_t * 0.85,
            );
            Stroke::new(1.0 + hover_t * 0.3, stroke_col)
        };

        let card_resp = egui::Frame::NONE
            .fill(card_fill)
            .stroke(border_stroke)
            .corner_radius(CornerRadius::same(12))
            .inner_margin(Margin::symmetric(16, 12))
            .show(ui, |ui| {
                // Row 1: Header (Font Name, Secondary Metadata, Quick Micro-Actions, Action Button)
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(&font.family)
                            .font(FontId::proportional(16.5))
                            .strong()
                            .color(Material3Theme::ON_SURFACE),
                    );

                    ui.add_space(8.0);

                    let variants_count = font.variants.len();
                    let styles_text = if variants_count == 1 {
                        "1 style".to_string()
                    } else {
                        format!("{variants_count} styles")
                    };

                    let meta_str = if font.popularity > 0 {
                        format!(
                            "{}  •  {}  •  #{}",
                            font.category, styles_text, font.popularity
                        )
                    } else {
                        format!("{}  •  {}", font.category, styles_text)
                    };

                    ui.label(
                        RichText::new(meta_str)
                            .font(FontId::proportional(11.5))
                            .color(Material3Theme::ON_SURFACE_VARIANT),
                    );

                    // Right side: Micro-actions + Install Button
                    let total_actions_w = 216.0;
                    let avail = ui.available_width();
                    if avail > total_actions_w {
                        ui.add_space(avail - total_actions_w);
                    }

                    // Micro-action 1: Copy CSS declaration
                    let css_fg = Material3Theme::lerp_color(
                        Material3Theme::ON_SURFACE_MUTED,
                        Material3Theme::ON_SURFACE,
                        hover_t,
                    );
                    let css_btn = egui::Button::new(
                        RichText::new("📋 CSS")
                            .font(FontId::proportional(10.5))
                            .color(css_fg),
                    )
                    .min_size(vec2(52.0, 24.0))
                    .fill(Material3Theme::SURFACE_CONTAINER_LOW)
                    .stroke(Stroke::new(1.0, Material3Theme::OUTLINE_VARIANT))
                    .corner_radius(CornerRadius::same(8));

                    if ui
                        .add(css_btn)
                        .on_hover_text("Copy font-family CSS declaration")
                        .clicked()
                    {
                        let fallback = match font.category.as_str() {
                            "serif" => "serif",
                            "monospace" => "monospace",
                            "handwriting" => "cursive",
                            "display" => "sans-serif",
                            _ => "sans-serif",
                        };
                        let css_rule = format!("font-family: '{}', {};", font.family, fallback);
                        ui.ctx().copy_text(css_rule.clone());
                        self.toast_notification = Some((
                            format!("Copied: {css_rule}"),
                            std::time::Instant::now(),
                        ));
                    }

                    // Micro-action 2: Google Fonts specimen link
                    let web_fg = Material3Theme::lerp_color(
                        Material3Theme::ON_SURFACE_MUTED,
                        Material3Theme::PRIMARY,
                        hover_t,
                    );
                    let web_btn = egui::Button::new(
                        RichText::new("↗ Web")
                            .font(FontId::proportional(10.5))
                            .color(web_fg),
                    )
                    .min_size(vec2(52.0, 24.0))
                    .fill(Material3Theme::SURFACE_CONTAINER_LOW)
                    .stroke(Stroke::new(1.0, Material3Theme::OUTLINE_VARIANT))
                    .corner_radius(CornerRadius::same(8));

                    let web_url = format!(
                        "https://fonts.google.com/specimen/{}",
                        font.family.replace(' ', "+")
                    );
                    if ui
                        .add(web_btn)
                        .on_hover_text("Open full specimen & license on Google Fonts")
                        .clicked()
                    {
                        ui.ctx().open_url(egui::OpenUrl::new_tab(web_url));
                    }

                    // Main Install / Status Button
                    let btn_w = 92.0;
                    match &current_state {
                        Some(InstallState::Downloading) => {
                            let time = ui.input(|i| i.time);
                            let pulse = ((time * 5.0).sin() * 0.5 + 0.5) as f32;
                            let pulse_fill = Material3Theme::lerp_color(
                                Material3Theme::PRIMARY_CONTAINER,
                                Material3Theme::SURFACE_CONTAINER_HIGHEST,
                                pulse,
                            );
                            let btn = egui::Button::new(
                                RichText::new("⏳ Downloading")
                                    .font(FontId::proportional(10.5))
                                    .color(Material3Theme::ON_PRIMARY_CONTAINER),
                            )
                            .min_size(vec2(btn_w, 24.0))
                            .fill(pulse_fill)
                            .corner_radius(CornerRadius::same(12));
                            ui.add_enabled(false, btn);
                        }
                        Some(InstallState::Installing) => {
                            let time = ui.input(|i| i.time);
                            let pulse = ((time * 5.0).sin() * 0.5 + 0.5) as f32;
                            let pulse_fill = Material3Theme::lerp_color(
                                Material3Theme::PRIMARY_CONTAINER,
                                Material3Theme::SURFACE_CONTAINER_HIGHEST,
                                pulse,
                            );
                            let btn = egui::Button::new(
                                RichText::new("⚡ Installing")
                                    .font(FontId::proportional(10.5))
                                    .color(Material3Theme::ON_PRIMARY_CONTAINER),
                            )
                            .min_size(vec2(btn_w, 24.0))
                            .fill(pulse_fill)
                            .corner_radius(CornerRadius::same(12));
                            ui.add_enabled(false, btn);
                        }
                        Some(InstallState::Error(err)) => {
                            let retry_btn = egui::Button::new(
                                RichText::new("↻ Retry")
                                    .font(FontId::proportional(10.5))
                                    .color(Material3Theme::ON_ERROR_CONTAINER),
                            )
                            .min_size(vec2(btn_w, 24.0))
                            .fill(Material3Theme::ERROR_CONTAINER)
                            .stroke(Stroke::new(1.0, Material3Theme::ERROR))
                            .corner_radius(CornerRadius::same(12));

                            if ui.add(retry_btn).on_hover_text(err).clicked() {
                                let _ = self.tx_worker.send(WorkerCmd::InstallFont {
                                    id: font.id.clone(),
                                    family: font.family.clone(),
                                });
                            }
                        }
                        _ if is_installed => {
                            let btn = egui::Button::new(
                                RichText::new("✓ Installed")
                                    .font(FontId::proportional(10.5))
                                    .strong()
                                    .color(Material3Theme::ON_SUCCESS_CONTAINER),
                            )
                            .min_size(vec2(btn_w, 24.0))
                            .fill(Material3Theme::SUCCESS_CONTAINER)
                            .stroke(Stroke::new(1.0, Material3Theme::SUCCESS))
                            .corner_radius(CornerRadius::same(12));

                            if ui
                                .add(btn)
                                .on_hover_text("Click to re-install or repair font in Windows")
                                .clicked()
                            {
                                let _ = self.tx_worker.send(WorkerCmd::InstallFont {
                                    id: font.id.clone(),
                                    family: font.family.clone(),
                                });
                            }
                        }
                        _ => {
                            let btn = egui::Button::new(
                                RichText::new("↓ Install")
                                    .font(FontId::proportional(11.0))
                                    .strong()
                                    .color(Material3Theme::ON_PRIMARY),
                            )
                            .min_size(vec2(btn_w, 24.0))
                            .fill(Material3Theme::PRIMARY)
                            .corner_radius(CornerRadius::same(12));

                            if ui.add(btn).clicked() {
                                let _ = self.tx_worker.send(WorkerCmd::InstallFont {
                                    id: font.id.clone(),
                                    family: font.family.clone(),
                                });
                            }
                        }
                    }
                });

                ui.add_space(8.0);

                // Row 2: Specimen Typography Preview Line
                let is_active = self.active_fonts.contains(&font.id);
                let preview_family = if is_active {
                    egui::FontFamily::Name(Arc::from(font.id.as_str()))
                } else {
                    if !self.requested_previews.contains(&font.id) {
                        self.requested_previews.insert(font.id.clone());
                        let _ = self.tx_worker.send(WorkerCmd::FetchPreview {
                            id: font.id.clone(),
                            family: font.family.clone(),
                        });
                    }
                    egui::FontFamily::Proportional
                };

                let preview_text = if self.preview_text.trim().is_empty() {
                    "The quick brown fox jumps over the lazy dog"
                } else {
                    &self.preview_text
                };

                ui.label(
                    RichText::new(preview_text)
                        .font(FontId::new(self.preview_size, preview_family))
                        .color(Material3Theme::ON_SURFACE),
                );
            });

        // Store hover state for next frame's silky smooth interpolation
        let card_hovered = ui.rect_contains_pointer(card_resp.response.rect);
        ui.data_mut(|d| d.insert_temp(card_id, card_hovered));
    }

    /// Renders scrollable font catalog in a clean, responsive single-column stream
    fn render_font_list(&mut self, ui: &mut Ui) {
        let total_count = self.filtered_indices.len();
        if total_count == 0 {
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);
                if self.is_loading_catalog {
                    ui.label(
                        RichText::new("Connecting to Google Fonts catalog...")
                            .font(FontId::proportional(16.0))
                            .color(Material3Theme::PRIMARY),
                    );
                } else {
                    ui.label(
                        RichText::new("No fonts matched your search.")
                            .font(FontId::proportional(16.0))
                            .color(Material3Theme::ON_SURFACE_VARIANT),
                    );
                    ui.label(
                        RichText::new("Try adjusting your query or selecting 'All' categories.")
                            .font(FontId::proportional(12.0))
                            .color(Material3Theme::ON_SURFACE_MUTED),
                    );
                }
            });
            return;
        }

        let card_spacing = 8.0;
        let row_height = self.preview_size * 1.25 + 56.0 + card_spacing;
        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show_rows(ui, row_height, total_count, |ui, row_range| {
                for i in row_range {
                    if let Some(&font_idx) = self.filtered_indices.get(i) {
                        if let Some(font) = self.all_fonts.get(font_idx).cloned() {
                            self.render_font_card(ui, &font);
                            ui.add_space(card_spacing);
                        }
                    }
                }
            });
    }

    /// Renders floating micro-interaction toast notification (e.g. for Copy CSS, Installed, etc.)
    fn render_toast_notification(&mut self, ctx: &egui::Context) {
        if let Some((msg, created_at)) = &self.toast_notification {
            let elapsed = created_at.elapsed().as_secs_f32();
            if elapsed > 2.5 {
                self.toast_notification = None;
                return;
            }

            ctx.request_repaint();

            let alpha = if elapsed < 1.8 {
                1.0
            } else {
                ((2.5 - elapsed) / 0.7).clamp(0.0, 1.0)
            };

            let screen_rect = ctx.screen_rect();
            let toast_pos = Pos2::new(screen_rect.center().x, screen_rect.max.y - 36.0);

            let painter = ctx.layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::new("toast_notification_layer"),
            ));

            let font_id = FontId::proportional(12.0);
            let text_color = Color32::from_rgba_unmultiplied(248, 250, 252, (255.0 * alpha) as u8);
            let bg_color = Color32::from_rgba_unmultiplied(20, 23, 33, (240.0 * alpha) as u8);
            let border_color = Color32::from_rgba_unmultiplied(56, 189, 248, (220.0 * alpha) as u8);

            let galley = painter.layout_no_wrap(msg.clone(), font_id, text_color);
            let padding = vec2(14.0, 7.0);
            let toast_rect = Rect::from_center_size(toast_pos, galley.size() + padding * 2.0);

            painter.rect_filled(toast_rect, CornerRadius::same(12), bg_color);
            painter.rect_stroke(
                toast_rect,
                CornerRadius::same(12),
                Stroke::new(1.0, border_color),
                StrokeKind::Inside,
            );
            painter.galley(toast_rect.min + padding, galley, text_color);
        }
    }
}

impl eframe::App for AntigravityFontsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Promote pending fonts that were registered in previous frame to active
        if !self.pending_fonts.is_empty() {
            let drained: Vec<String> = self.pending_fonts.drain().collect();
            for id in drained {
                self.active_fonts.insert(id);
            }
        }

        self.process_worker_events(ctx);

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(Material3Theme::SURFACE))
            .show(ctx, |ui| {
                self.render_custom_title_bar(ui);
                egui::Frame::NONE
                    .inner_margin(Margin::symmetric(16, 0))
                    .show(ui, |ui| {
                        self.render_integrated_toolbar(ui);
                        self.render_font_list(ui);
                    });
            });

        self.render_toast_notification(ctx);
    }
}
