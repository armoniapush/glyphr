#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod catalog;
mod installer;
mod models;
mod theme;

use app::AntigravityFontsApp;
use eframe::egui::ViewportBuilder;

fn main() -> eframe::Result<()> {
    // Global panic hook to capture any unexpected runtime panic cleanly
    std::panic::set_hook(Box::new(|info| {
        if let Some(dirs) = directories::ProjectDirs::from("com", "antigravity", "antigravity-fonts") {
            let log_dir = dirs.cache_dir();
            let _ = std::fs::create_dir_all(log_dir);
            let backtrace = std::backtrace::Backtrace::capture();
            let msg = format!("PANIC: {info}\nBacktrace:\n{backtrace}\n");
            let _ = std::fs::write(log_dir.join("crash.log"), msg);
        }
    }));

    let native_options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([940.0, 740.0])
            .with_min_inner_size([720.0, 520.0])
            .with_title("Antigravity Fonts — Google Fonts Desktop Engine")
            .with_visible(true)
            .with_active(true),
        ..Default::default()
    };

    eframe::run_native(
        "Antigravity Fonts",
        native_options,
        Box::new(|cc| Ok(Box::new(AntigravityFontsApp::new(cc)))),
    )
}
