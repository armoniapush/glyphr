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
    previews_dir: PathBuf,
}

impl CatalogClient {
    pub fn new() -> Self {
        let (cache_path, previews_dir) = directories::ProjectDirs::from("com", "antigravity", "antigravity-fonts")
            .map(|dirs| {
                let cache_dir = dirs.cache_dir().to_path_buf();
                (cache_dir.join("catalog_cache.json"), cache_dir.join("previews"))
            })
            .unwrap_or_else(|| (PathBuf::from("catalog_cache.json"), PathBuf::from("previews")));

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(20))
            .user_agent("AntigravityFonts/0.1.0")
            .build()
            .unwrap_or_default();

        Self {
            client,
            cache_path,
            previews_dir,
        }
    }

    /// Checks if raw byte slice starts with valid TrueType or OpenType magic headers.
    pub fn is_valid_truetype(bytes: &[u8]) -> bool {
        if bytes.len() < 4 {
            return false;
        }
        let magic = &bytes[0..4];
        magic == [0x00, 0x01, 0x00, 0x00]
            || magic == b"OTTO"
            || magic == b"true"
            || magic == b"typ1"
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

    /// Fetches and caches a genuine TTF for live preview rendering.
    pub async fn fetch_preview_font(
        &self,
        font_id: &str,
        family_name: &str,
    ) -> Result<Vec<u8>, String> {
        let preview_path = self.previews_dir.join(format!("{font_id}.ttf"));

        // 1. Check local preview cache
        if preview_path.exists() {
            if let Ok(bytes) = fs::read(&preview_path) {
                if Self::is_valid_truetype(&bytes) && ttf_parser::Face::parse(&bytes, 0).is_ok() {
                    return Ok(bytes);
                } else {
                    // Remove stale or corrupted cache file
                    let _ = fs::remove_file(&preview_path);
                }
            }
        }

        // 2. Check if already installed locally in user fonts folder
        if let Some(local_path) = crate::installer::FontInstaller::find_local_font_file(font_id, family_name) {
            if let Ok(bytes) = fs::read(&local_path) {
                if Self::is_valid_truetype(&bytes) && ttf_parser::Face::parse(&bytes, 0).is_ok() {
                    return Ok(bytes);
                }
            }
        }

        // 3. Query GWFH font detail API to get the genuine TTF CDN link
        let detail_url = format!("https://gwfh.mranftl.com/api/fonts/{font_id}");
        if let Ok(resp) = self.client.get(&detail_url).send().await {
            if let Ok(val) = resp.json::<serde_json::Value>().await {
                if let Some(variants) = val.get("variants").and_then(|v| v.as_array()) {
                    // Prefer "regular", "400", or the first available variant
                    let regular_variant = variants
                        .iter()
                        .find(|v| {
                            let id_str = v.get("id").and_then(|i| i.as_str()).unwrap_or("");
                            id_str == "regular" || id_str == "400"
                        })
                        .or_else(|| variants.first());

                    if let Some(v) = regular_variant {
                        if let Some(ttf_url) = v.get("ttf").and_then(|u| u.as_str()) {
                            if let Ok(ttf_resp) = self.client.get(ttf_url).send().await {
                                if let Ok(bytes) = ttf_resp.bytes().await {
                                    if Self::is_valid_truetype(&bytes)
                                        && ttf_parser::Face::parse(&bytes, 0).is_ok()
                                    {
                                        let _ = fs::create_dir_all(&self.previews_dir);
                                        let _ = fs::write(&preview_path, &bytes);
                                        return Ok(bytes.to_vec());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Err(format!("Could not fetch genuine TTF font for {family_name}"))
    }
}
