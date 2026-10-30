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
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .build()
            .unwrap_or_default();

        Self {
            client,
            cache_path,
            previews_dir,
        }
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

    /// Fetches and caches a single regular TTF for live preview rendering.
    pub async fn fetch_preview_font(
        &self,
        font_id: &str,
        family_name: &str,
    ) -> Result<Vec<u8>, String> {
        let preview_path = self.previews_dir.join(format!("{font_id}.ttf"));

        // 1. Check local preview cache
        if preview_path.exists() {
            if let Ok(bytes) = fs::read(&preview_path) {
                if !bytes.is_empty() {
                    return Ok(bytes);
                }
            }
        }

        // 2. Check if already installed locally in user fonts folder
        if let Some(local_path) = crate::installer::FontInstaller::find_local_font_file(font_id, family_name) {
            if let Ok(bytes) = fs::read(&local_path) {
                if !bytes.is_empty() {
                    return Ok(bytes);
                }
            }
        }

        // 3. Fetch from Google Fonts CSS API
        let encoded_family = family_name.replace(' ', "+");
        let css_url = format!("https://fonts.googleapis.com/css2?family={encoded_family}");

        if let Ok(resp) = self.client.get(&css_url).send().await {
            if let Ok(css) = resp.text().await {
                if let Some(start_idx) = css.find("url(") {
                    let rest = &css[start_idx + 4..];
                    if let Some(end_idx) = rest.find(')') {
                        let ttf_url = rest[..end_idx].trim().trim_matches('"').trim_matches('\'');
                        if ttf_url.starts_with("https://") {
                            if let Ok(ttf_resp) = self.client.get(ttf_url).send().await {
                                if let Ok(bytes) = ttf_resp.bytes().await {
                                    if !bytes.is_empty() {
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

        // 3. Fallback: GWFH font detail API
        let detail_url = format!("https://gwfh.mranftl.com/api/fonts/{font_id}");
        if let Ok(resp) = self.client.get(&detail_url).send().await {
            if let Ok(val) = resp.json::<serde_json::Value>().await {
                if let Some(variants) = val.get("variants").and_then(|v| v.as_array()) {
                    let regular_variant = variants
                        .iter()
                        .find(|v| v.get("id").and_then(|i| i.as_str()) == Some("regular"))
                        .or_else(|| variants.first());

                    if let Some(v) = regular_variant {
                        if let Some(ttf_url) = v.get("ttf").and_then(|u| u.as_str()) {
                            if let Ok(ttf_resp) = self.client.get(ttf_url).send().await {
                                if let Ok(bytes) = ttf_resp.bytes().await {
                                    if !bytes.is_empty() {
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

        Err(format!("Could not fetch preview font for {family_name}"))
    }
}
