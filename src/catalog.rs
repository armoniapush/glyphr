use crate::models::FontItem;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

const API_CATALOG_URL: &str = "https://gwfh.mranftl.com/api/fonts";
const CACHE_MAX_AGE: Duration = Duration::from_secs(7 * 24 * 3600); // 7 days

pub struct CatalogClient {
    client: reqwest::Client,
    cache_path: PathBuf,
}

impl CatalogClient {
    pub fn new() -> Self {
        let cache_path = directories::ProjectDirs::from("com", "antigravity", "antigravity-fonts")
            .map(|dirs| dirs.cache_dir().join("catalog_cache.json"))
            .unwrap_or_else(|| PathBuf::from("catalog_cache.json"));

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(20))
            .user_agent("AntigravityFonts/0.1.0")
            .build()
            .unwrap_or_default();

        Self { client, cache_path }
    }

    /// Loads fonts from disk cache if present and valid.
    pub fn load_from_cache(&self) -> Option<Vec<FontItem>> {
        if !self.cache_path.exists() {
            return None;
        }

        let file = File::open(&self.cache_path).ok()?;
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).ok()
    }

    /// Checks if cache needs refreshing (older than 7 days or missing).
    pub fn is_cache_stale(&self) -> bool {
        if let Ok(metadata) = fs::metadata(&self.cache_path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = SystemTime::now().duration_since(modified) {
                    return elapsed > CACHE_MAX_AGE;
                }
            }
        }
        true
    }

    /// Fetches the latest catalog from the API and caches it.
    pub async fn fetch_and_cache(&self) -> Result<Vec<FontItem>, String> {
        let resp = self
            .client
            .get(API_CATALOG_URL)
            .send()
            .await
            .map_err(|e| format!("Failed to connect to Google Fonts catalog API: {e}"))?;

        if !resp.status().is_success() {
            return Err(format!("Catalog API returned HTTP {}", resp.status()));
        }

        let fonts: Vec<FontItem> = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse catalog JSON: {e}"))?;

        // Save to cache
        if let Some(parent) = self.cache_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        if let Ok(file) = File::create(&self.cache_path) {
            let writer = BufWriter::new(file);
            let _ = serde_json::to_writer(writer, &fonts);
        }

        Ok(fonts)
    }

    /// Downloads the TTF zip archive for a specific font family.
    pub async fn download_font_zip(&self, font_id: &str) -> Result<Vec<u8>, String> {
        let url = format!("https://gwfh.mranftl.com/api/fonts/{font_id}?download=zip&formats=ttf");

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to download font {font_id}: {e}"))?;

        if !resp.status().is_success() {
            return Err(format!("Download failed with HTTP {}", resp.status()));
        }

        let bytes = resp
            .bytes()
            .await
            .map_err(|e| format!("Failed reading font bytes: {e}"))?;

        Ok(bytes.to_vec())
    }
}
