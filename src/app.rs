use crate::catalog::CatalogClient;
use crate::installer::FontInstaller;
use crate::models::{CategoryFilter, FontItem, InstallState};
use crate::theme::AntigravityTheme;
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
    registered_fonts: HashSet<String>,
    requested_previews: HashSet<String>,

    // Search and filters
    search_query: String,
    selected_category: CategoryFilter,
    preview_text: String,
    preview_size: f32,

    // Status
    status_message: String,
    is_loading_catalog: bool,

    // Worker channels
    tx_worker: Sender<WorkerCmd>,
    rx_ui: Receiver<WorkerEvent>,
}

impl AntigravityFontsApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        AntigravityTheme::apply(&cc.egui_ctx);

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
        let registered_fonts = HashSet::new();

        let mut app = Self {
            all_fonts: Vec::new(),
            filtered_indices: Vec::new(),
            installed_families: HashSet::new(),
            font_states: HashMap::new(),
            font_definitions,
            registered_fonts,
            requested_previews: HashSet::new(),
            search_query: String::new(),
            selected_category: CategoryFilter::All,
            preview_text: "DANIELDOBLES".to_string(),
            preview_size: 24.0,
            status_message: "Initializing Google Fonts catalog...".to_string(),
            is_loading_catalog: true,
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
                        self.registered_fonts.insert(id);
                        fonts_updated = true;
                    }
                }
            }
            ctx.request_repaint();
        }

        if fonts_updated {
            ctx.set_fonts(self.font_definitions.clone());
        }

        if needs_filter {
            self.apply_filters();
        }
    }

    /// Renders the top Antigravity header bar
    fn render_header(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // Antigravity Logo Badge
            let (rect, _) = ui.allocate_exact_size(vec2(38.0, 38.0), Sense::hover());
            let painter = ui.painter();
            painter.rect_filled(
                rect,
                CornerRadius::same(10),
                Color32::from_rgb(22, 28, 42),
            );
            painter.rect_stroke(
                rect,
                CornerRadius::same(10),
                Stroke::new(1.0, AntigravityTheme::BORDER_FOCUS),
                StrokeKind::Inside,
            );
            painter.text(
                rect.center(),
                Align2::CENTER_CENTER,
                "AG",
                FontId::proportional(17.0),
                AntigravityTheme::ACCENT_CYAN,
            );

            ui.add_space(8.0);

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Antigravity Fonts")
                            .font(FontId::proportional(19.0))
                            .strong()
                            .color(AntigravityTheme::TEXT_PRIMARY),
                    );

                    let status_color = if self.is_loading_catalog {
                        AntigravityTheme::ACCENT_CYAN
                    } else {
                        AntigravityTheme::SUCCESS_GREEN
                    };

                    ui.label(
                        RichText::new(format!("● {}", self.status_message))
                            .font(FontId::proportional(12.0))
                            .color(status_color),
                    );
                });

                ui.label(
                    RichText::new("Ultra-lightweight Google Fonts discovery & 1-click silent installer")
                        .font(FontId::proportional(11.0))
                        .color(AntigravityTheme::TEXT_MUTED),
                );
            });

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let refresh_btn = egui::Button::new(
                    RichText::new("⟳ Sync Catalog")
                        .font(FontId::proportional(12.0))
                        .color(AntigravityTheme::TEXT_PRIMARY),
                )
                .fill(AntigravityTheme::BG_CARD)
                .stroke(Stroke::new(1.0, AntigravityTheme::BORDER_SUBTLE))
                .corner_radius(CornerRadius::same(8));

                if ui.add(refresh_btn).clicked() {
                    self.is_loading_catalog = true;
                    self.status_message = "Refreshing catalog...".to_string();
                    let _ = self.tx_worker.send(WorkerCmd::FetchCatalog);
                }
            });
        });
    }

    /// Renders search bar and category chips
    fn render_search_and_filters(&mut self, ui: &mut Ui) {
        ui.add_space(8.0);

        // Search bar
        ui.horizontal(|ui| {
            let search_rect = ui.available_rect_before_wrap();
            let height = 38.0;
            let (rect, _) = ui.allocate_exact_size(vec2(search_rect.width(), height), Sense::hover());

            // Paint background container and search icon
            ui.painter().rect_filled(
                rect,
                CornerRadius::same(10),
                AntigravityTheme::BG_INPUT,
            );
            ui.painter().rect_stroke(
                rect,
                CornerRadius::same(10),
                Stroke::new(1.0, AntigravityTheme::BORDER_SUBTLE),
                StrokeKind::Inside,
            );
            ui.painter().text(
                Pos2::new(rect.min.x + 16.0, rect.center().y),
                Align2::LEFT_CENTER,
                "🔍",
                FontId::proportional(14.0),
                AntigravityTheme::TEXT_MUTED,
            );

            // Inner TextEdit
            let inner_rect = Rect::from_min_max(
                Pos2::new(rect.min.x + 40.0, rect.min.y + 2.0),
                Pos2::new(rect.max.x - 40.0, rect.max.y - 2.0),
            );

            let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(inner_rect));
            let response = child_ui.add(
                egui::TextEdit::singleline(&mut self.search_query)
                    .hint_text("Search by family name, category, or style (e.g. Inter, Roboto, Space, Serif)...")
                    .frame(false)
                    .font(FontId::proportional(14.0))
                    .text_color(AntigravityTheme::TEXT_PRIMARY),
            );

            if response.changed() {
                self.apply_filters();
            }

            // Clear button if search has text
            if !self.search_query.is_empty() {
                let clear_rect = Rect::from_center_size(
                    Pos2::new(rect.max.x - 20.0, rect.center().y),
                    vec2(24.0, 24.0),
                );
                let clear_resp = ui.allocate_rect(clear_rect, Sense::click());
                if clear_resp.clicked() {
                    self.search_query.clear();
                    self.apply_filters();
                }
                ui.painter().text(
                    clear_rect.center(),
                    Align2::CENTER_CENTER,
                    "✕",
                    FontId::proportional(12.0),
                    AntigravityTheme::TEXT_MUTED,
                );
            }
        });

        ui.add_space(8.0);

        // Category pills
        ui.horizontal_wrapped(|ui| {
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

                let (bg_color, stroke_color, text_color) = if is_selected {
                    (
                        Color32::from_rgb(28, 42, 64),
                        AntigravityTheme::BORDER_FOCUS,
                        AntigravityTheme::ACCENT_CYAN,
                    )
                } else {
                    (
                        AntigravityTheme::BG_BADGE,
                        AntigravityTheme::BORDER_SUBTLE,
                        AntigravityTheme::TEXT_MUTED,
                    )
                };

                let btn = egui::Button::new(
                    RichText::new(label)
                        .font(FontId::proportional(12.0))
                        .color(text_color),
                )
                .fill(bg_color)
                .stroke(Stroke::new(1.0, stroke_color))
                .corner_radius(CornerRadius::same(16));

                if ui.add(btn).clicked() {
                    self.selected_category = cat;
                    self.apply_filters();
                }
            }
        });
    }

    /// Renders customizable live preview text bar
    fn render_preview_controls(&mut self, ui: &mut Ui) {
        ui.add_space(4.0);
        egui::Frame::NONE
            .fill(AntigravityTheme::BG_SURFACE)
            .stroke(Stroke::new(1.0, AntigravityTheme::BORDER_SUBTLE))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(Margin::symmetric(12, 8))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Preview Text:")
                            .font(FontId::proportional(12.0))
                            .color(AntigravityTheme::TEXT_MUTED),
                    );

                    ui.add(
                        egui::TextEdit::singleline(&mut self.preview_text)
                            .desired_width(ui.available_width() - 200.0)
                            .font(FontId::proportional(13.0)),
                    );

                    ui.label(
                        RichText::new("Size:")
                            .font(FontId::proportional(12.0))
                            .color(AntigravityTheme::TEXT_MUTED),
                    );

                    ui.add(
                        egui::Slider::new(&mut self.preview_size, 14.0..=48.0)
                            .suffix("px")
                            .show_value(true),
                    );
                });
            });
    }

    /// Renders an individual font card
    fn render_font_card(&mut self, ui: &mut Ui, font: &FontItem) {
        let is_installed = self.installed_families.contains(&font.id);
        let current_state = self.font_states.get(&font.id).cloned();

        let border_color = match &current_state {
            Some(InstallState::Downloading) | Some(InstallState::Installing) => {
                AntigravityTheme::BORDER_FOCUS
            }
            Some(InstallState::Installed) => AntigravityTheme::SUCCESS_GREEN,
            Some(InstallState::Error(_)) => AntigravityTheme::ERROR_RED,
            _ if is_installed => Color32::from_rgb(20, 83, 45), // Forest green border
            _ => AntigravityTheme::BORDER_SUBTLE,
        };

        egui::Frame::NONE
            .fill(AntigravityTheme::BG_CARD)
            .stroke(Stroke::new(1.0, border_color))
            .corner_radius(CornerRadius::same(10))
            .inner_margin(Margin::same(14))
            .show(ui, |ui| {
                // Header row: Family name, metadata badges, install button
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(&font.family)
                                    .font(FontId::proportional(17.0))
                                    .strong()
                                    .color(AntigravityTheme::TEXT_PRIMARY),
                            );

                            // Category pill badge
                            let cat_badge = RichText::new(&font.category)
                                .font(FontId::proportional(10.0))
                                .color(AntigravityTheme::ACCENT_CYAN);
                            ui.label(
                                egui::WidgetText::RichText(cat_badge).background_color(
                                    Color32::from_rgba_premultiplied(56, 189, 248, 25),
                                ),
                            );

                            // Variants pill badge
                            let variants_count = font.variants.len();
                            let style_str = if variants_count == 1 {
                                "1 style".to_string()
                            } else {
                                format!("{variants_count} styles")
                            };
                            let styles_badge = RichText::new(style_str)
                                .font(FontId::proportional(10.0))
                                .color(AntigravityTheme::TEXT_MUTED);
                            ui.label(
                                egui::WidgetText::RichText(styles_badge)
                                    .background_color(AntigravityTheme::BG_BADGE),
                            );

                            // Popularity rank badge
                            if font.popularity > 0 {
                                let rank_text = format!("#{}", font.popularity);
                                let rank_badge = RichText::new(rank_text)
                                    .font(FontId::proportional(10.0))
                                    .color(AntigravityTheme::ACCENT_INDIGO);
                                ui.label(
                                    egui::WidgetText::RichText(rank_badge).background_color(
                                        Color32::from_rgba_premultiplied(99, 102, 241, 25),
                                    ),
                                );
                            }
                        });
                    });

                    // Action Button on the right
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        match &current_state {
                            Some(InstallState::Downloading) => {
                                let btn = egui::Button::new(
                                    RichText::new("⏳ Downloading...")
                                        .font(FontId::proportional(12.0))
                                        .color(AntigravityTheme::ACCENT_CYAN),
                                )
                                .fill(Color32::from_rgb(22, 33, 49))
                                .stroke(Stroke::new(1.0, AntigravityTheme::BORDER_FOCUS))
                                .corner_radius(CornerRadius::same(8));
                                ui.add_enabled(false, btn);
                            }
                            Some(InstallState::Installing) => {
                                let btn = egui::Button::new(
                                    RichText::new("⚡ Registering...")
                                        .font(FontId::proportional(12.0))
                                        .color(AntigravityTheme::ACCENT_INDIGO),
                                )
                                .fill(Color32::from_rgb(28, 26, 50))
                                .stroke(Stroke::new(1.0, AntigravityTheme::BORDER_ACTIVE))
                                .corner_radius(CornerRadius::same(8));
                                ui.add_enabled(false, btn);
                            }
                            Some(InstallState::Error(err)) => {
                                ui.horizontal(|ui| {
                                    let retry_btn = egui::Button::new(
                                        RichText::new("↻ Retry")
                                            .font(FontId::proportional(11.0))
                                            .color(AntigravityTheme::ERROR_RED),
                                    )
                                    .fill(Color32::from_rgb(45, 18, 18))
                                    .stroke(Stroke::new(1.0, AntigravityTheme::ERROR_RED))
                                    .corner_radius(CornerRadius::same(6));

                                    if ui.add(retry_btn).clicked() {
                                        let _ = self.tx_worker.send(WorkerCmd::InstallFont {
                                            id: font.id.clone(),
                                            family: font.family.clone(),
                                        });
                                    }

                                    ui.label(
                                        RichText::new(format!("Error: {err}"))
                                            .font(FontId::proportional(10.0))
                                            .color(AntigravityTheme::ERROR_RED),
                                    );
                                });
                            }
                            _ if is_installed => {
                                let btn = egui::Button::new(
                                    RichText::new("✓ Installed")
                                        .font(FontId::proportional(12.0))
                                        .strong()
                                        .color(AntigravityTheme::SUCCESS_GREEN),
                                )
                                .fill(AntigravityTheme::SUCCESS_BG)
                                .stroke(Stroke::new(1.0, AntigravityTheme::SUCCESS_GREEN))
                                .corner_radius(CornerRadius::same(8));

                                if ui.add(btn).on_hover_text("Re-install or update this font").clicked() {
                                    let _ = self.tx_worker.send(WorkerCmd::InstallFont {
                                        id: font.id.clone(),
                                        family: font.family.clone(),
                                    });
                                }
                            }
                            _ => {
                                let btn = egui::Button::new(
                                    RichText::new("↓ Install Font")
                                        .font(FontId::proportional(12.0))
                                        .strong()
                                        .color(Color32::WHITE),
                                )
                                .fill(Color32::from_rgb(14, 116, 144))
                                .stroke(Stroke::new(1.0, AntigravityTheme::BORDER_FOCUS))
                                .corner_radius(CornerRadius::same(8));

                                if ui.add(btn).clicked() {
                                    let _ = self.tx_worker.send(WorkerCmd::InstallFont {
                                        id: font.id.clone(),
                                        family: font.family.clone(),
                                    });
                                }
                            }
                        }
                    });
                });

                ui.add_space(8.0);

                // Determine Font Family for preview rendering
                let has_real_font = self.registered_fonts.contains(&font.id);
                let preview_family = if has_real_font {
                    egui::FontFamily::Name(Arc::from(font.id.as_str()))
                } else {
                    // Queue background download for preview font if not yet requested
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
                        .color(AntigravityTheme::TEXT_PRIMARY),
                );
            });
    }

    /// Renders scrollable font list with virtual scrolling
    fn render_font_list(&mut self, ui: &mut Ui) {
        let total_count = self.filtered_indices.len();
        if total_count == 0 {
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);
                if self.is_loading_catalog {
                    ui.label(
                        RichText::new("Connecting to Google Fonts catalog...")
                            .font(FontId::proportional(16.0))
                            .color(AntigravityTheme::ACCENT_CYAN),
                    );
                } else {
                    ui.label(
                        RichText::new("No fonts matched your search.")
                            .font(FontId::proportional(16.0))
                            .color(AntigravityTheme::TEXT_MUTED),
                    );
                    ui.label(
                        RichText::new("Try adjusting your query or selecting 'All' categories.")
                            .font(FontId::proportional(12.0))
                            .color(AntigravityTheme::TEXT_FAINT),
                    );
                }
            });
            return;
        }

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("Showing {total_count} font families"))
                    .font(FontId::proportional(12.0))
                    .color(AntigravityTheme::TEXT_MUTED),
            );
        });
        ui.add_space(4.0);

        // Approximate row height for virtual scrolling
        let row_height = 110.0;
        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show_rows(ui, row_height, total_count, |ui, row_range| {
                for i in row_range {
                    if let Some(&font_idx) = self.filtered_indices.get(i) {
                        if let Some(font) = self.all_fonts.get(font_idx).cloned() {
                            self.render_font_card(ui, &font);
                            ui.add_space(8.0);
                        }
                    }
                }
            });
    }
}

impl eframe::App for AntigravityFontsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_worker_events(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_header(ui);
            ui.add_space(6.0);
            self.render_search_and_filters(ui);
            ui.add_space(6.0);
            self.render_preview_controls(ui);
            ui.add_space(6.0);
            self.render_font_list(ui);
        });
    }
}
