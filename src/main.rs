#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod catalog;
mod installer;
mod models;
mod theme;

use app::AntigravityFontsApp;
use eframe::egui::ViewportBuilder;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([940.0, 740.0])
            .with_min_inner_size([720.0, 520.0])
            .with_title("Antigravity Fonts — Google Fonts Desktop Engine"),
        ..Default::default()
    };

    eframe::run_native(
        "Antigravity Fonts",
        native_options,
        Box::new(|cc| Ok(Box::new(AntigravityFontsApp::new(cc)))),
    )
}
