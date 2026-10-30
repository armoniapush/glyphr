use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontItem {
    pub id: String,
    pub family: String,
    pub category: String,
    #[serde(default)]
    pub variants: Vec<String>,
    #[serde(default)]
    pub subsets: Vec<String>,
    #[serde(default)]
    pub popularity: u32,
    #[serde(rename = "lastModified", default)]
    pub last_modified: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CategoryFilter {
    All,
    SansSerif,
    Serif,
    Display,
    Handwriting,
    Monospace,
    InstalledOnly,
}

impl CategoryFilter {
    pub fn as_str(&self) -> &'static str {
        match self {
            CategoryFilter::All => "All",
            CategoryFilter::SansSerif => "Sans-serif",
            CategoryFilter::Serif => "Serif",
            CategoryFilter::Display => "Display",
            CategoryFilter::Handwriting => "Handwriting",
            CategoryFilter::Monospace => "Monospace",
            CategoryFilter::InstalledOnly => "Installed",
        }
    }

    pub fn matches(&self, category: &str, is_installed: bool) -> bool {
        match self {
            CategoryFilter::All => true,
            CategoryFilter::SansSerif => category.eq_ignore_ascii_case("sans-serif"),
            CategoryFilter::Serif => category.eq_ignore_ascii_case("serif"),
            CategoryFilter::Display => category.eq_ignore_ascii_case("display"),
            CategoryFilter::Handwriting => category.eq_ignore_ascii_case("handwriting"),
            CategoryFilter::Monospace => category.eq_ignore_ascii_case("monospace"),
            CategoryFilter::InstalledOnly => is_installed,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallState {
    Downloading,
    Installing,
    Installed,
    Error(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rasterize_inter() {
        let path = std::path::PathBuf::from(r"C:\Users\armon\AppData\Local\antigravity\antigravity-fonts\cache\previews\inter.ttf");
        if path.exists() {
            let bytes = std::fs::read(&path).unwrap();
            let mut font_defs = egui::FontDefinitions::default();
            font_defs.font_data.insert("inter".to_string(), std::sync::Arc::new(egui::FontData::from_owned(bytes)));
            font_defs.families.insert(egui::FontFamily::Name(std::sync::Arc::from("inter")), vec!["inter".to_string()]);
            
            let ctx = egui::Context::default();
            ctx.set_fonts(font_defs);
            let _ = ctx.run(Default::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.label(egui::RichText::new("DANIELDOBLES").font(egui::FontId::new(24.0, egui::FontFamily::Name(std::sync::Arc::from("inter")))));
                });
            });
            println!("Rasterized inter successfully!");
        }
    }
}
