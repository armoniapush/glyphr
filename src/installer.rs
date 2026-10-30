use std::fs::{self, File};
use std::io::{self, Cursor, Read, Write};
use std::path::{Path, PathBuf};
use zip::ZipArchive;

pub struct FontInstaller;

impl FontInstaller {
    /// Returns the per-user font installation directory for the current operating system.
    pub fn get_user_fonts_dir() -> io::Result<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
                let path = PathBuf::from(local_app_data)
                    .join("Microsoft")
                    .join("Windows")
                    .join("Fonts");
                fs::create_dir_all(&path)?;
                return Ok(path);
            }
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "LOCALAPPDATA environment variable not found",
            ))
        }

        #[cfg(target_os = "macos")]
        {
            if let Some(home) = directories::BaseDirs::new() {
                let path = home.home_dir().join("Library").join("Fonts");
                fs::create_dir_all(&path)?;
                return Ok(path);
            }
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Home directory not found",
            ))
        }

        #[cfg(all(unix, not(target_os = "macos")))]
        {
            if let Some(dirs) = directories::BaseDirs::new() {
                let path = dirs.data_local_dir().join("fonts");
                fs::create_dir_all(&path)?;
                return Ok(path);
            }
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "User data directory not found",
            ))
        }
    }

    /// Checks if any font file matching the family ID exists in the user fonts directory or registry.
    pub fn is_family_installed(family_id: &str, family_name: &str) -> bool {
        #[cfg(target_os = "windows")]
        {
            // Check Registry first
            if let Ok(hkcu) = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER)
                .open_subkey("Software\\Microsoft\\Windows NT\\CurrentVersion\\Fonts")
            {
                let family_lower = family_name.to_lowercase();
                for (name, _) in hkcu.enum_values().filter_map(|v| v.ok()) {
                    if name.to_lowercase().starts_with(&family_lower) {
                        return true;
                    }
                }
            }

            // Fallback: check files in user fonts folder
            if let Ok(dir) = Self::get_user_fonts_dir() {
                let id_prefix = family_id.to_lowercase().replace(' ', "-");
                if let Ok(entries) = fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let fname = entry.file_name().to_string_lossy().to_lowercase();
                        if (fname.starts_with(&id_prefix)
                            || fname.contains(&family_name.to_lowercase()))
                            && (fname.ends_with(".ttf") || fname.ends_with(".otf"))
                        {
                            return true;
                        }
                    }
                }
            }
            false
        }

        #[cfg(not(target_os = "windows"))]
        {
            if let Ok(dir) = Self::get_user_fonts_dir() {
                let id_prefix = family_id.to_lowercase().replace(' ', "-");
                if let Ok(entries) = fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let fname = entry.file_name().to_string_lossy().to_lowercase();
                        if fname.starts_with(&id_prefix)
                            && (fname.ends_with(".ttf") || fname.ends_with(".otf"))
                        {
                            return true;
                        }
                    }
                }
            }
            false
        }
    }

    /// Finds an existing installed TTF or OTF font file matching the family name or ID.
    pub fn find_local_font_file(family_id: &str, family_name: &str) -> Option<PathBuf> {
        let dir = Self::get_user_fonts_dir().ok()?;
        let id_clean = family_id.to_lowercase().replace(' ', "-");
        let name_clean = family_name.to_lowercase().replace(' ', "");

        if let Ok(entries) = fs::read_dir(&dir) {
            let mut candidates = Vec::new();
            for entry in entries.flatten() {
                let path = entry.path();
                let fname = path.file_name()?.to_string_lossy().to_lowercase();
                if fname.ends_with(".ttf") || fname.ends_with(".otf") {
                    let fname_no_ext = fname.trim_end_matches(".ttf").trim_end_matches(".otf");
                    let clean = fname_no_ext.replace('-', "").replace('_', "");

                    if fname.starts_with(&id_clean) || clean.contains(&name_clean) {
                        candidates.push(path);
                    }
                }
            }

            // Prefer regular or non-bold/italic variants
            if let Some(reg) = candidates.iter().find(|p| {
                let s = p.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
                s.contains("regular") || (!s.contains("bold") && !s.contains("italic") && !s.contains("variable"))
            }) {
                return Some(reg.clone());
            }

            return candidates.into_iter().next();
        }
        None
    }

    /// Extracts zip bytes, saves fonts into user directory, and registers them into Windows.
    pub fn install_from_zip(
        zip_bytes: &[u8],
        family_name: &str,
    ) -> Result<Vec<PathBuf>, String> {
        let fonts_dir = Self::get_user_fonts_dir().map_err(|e| e.to_string())?;
        let cursor = Cursor::new(zip_bytes);
        let mut archive = ZipArchive::new(cursor).map_err(|e| format!("Invalid zip archive: {e}"))?;

        let mut installed_paths = Vec::new();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
            let raw_path = file.mangled_name();
            let file_name = match raw_path.file_name() {
                Some(name) => name.to_string_lossy().to_string(),
                None => continue,
            };

            let ext = Path::new(&file_name)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            if ext != "ttf" && ext != "otf" {
                continue;
            }

            let dest_path = fonts_dir.join(&file_name);
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).map_err(|e| e.to_string())?;

            let mut out = File::create(&dest_path).map_err(|e| e.to_string())?;
            out.write_all(&buf).map_err(|e| e.to_string())?;

            // Determine font display name from TTF tables or construct cleanly
            let font_title = Self::extract_font_title(&buf).unwrap_or_else(|| {
                let stem = Path::new(&file_name)
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy();
                format!("{family_name} ({stem})")
            });

            #[cfg(target_os = "windows")]
            Self::register_windows_font(&dest_path, &font_title, ext == "ttf")?;

            installed_paths.push(dest_path);
        }

        if installed_paths.is_empty() {
            return Err("No TTF or OTF fonts were found in the package".to_string());
        }

        #[cfg(target_os = "windows")]
        Self::broadcast_font_change();

        #[cfg(all(unix, not(target_os = "macos")))]
        {
            let _ = std::process::Command::new("fc-cache")
                .arg("-f")
                .output();
        }

        Ok(installed_paths)
    }

    /// Extracts a normalized family slug (e.g. "roboto", "inter") from the TTF/OTF metadata table.
    #[allow(dead_code)]
    pub fn extract_family_slug(bytes: &[u8]) -> Option<String> {
        let face = ttf_parser::Face::parse(bytes, 0).ok()?;
        for name in face.names() {
            if name.name_id == 1 {
                if let Some(s) = name.to_string() {
                    let clean = s.trim().to_lowercase().replace(' ', "-");
                    if !clean.is_empty() {
                        return Some(clean);
                    }
                }
            }
        }
        None
    }

    /// Extracts the full font name from the TTF/OTF metadata table.
    fn extract_font_title(bytes: &[u8]) -> Option<String> {
        let face = ttf_parser::Face::parse(bytes, 0).ok()?;
        // Try Name ID 4 (Full Name), or 1 (Family) + 2 (Subfamily)
        let mut full_name = None;
        let mut family = None;
        let mut subfamily = None;

        for name in face.names() {
            if let Some(text) = name.to_string() {
                match name.name_id {
                    4 if full_name.is_none() => full_name = Some(text),
                    1 if family.is_none() => family = Some(text),
                    2 if subfamily.is_none() => subfamily = Some(text),
                    _ => {}
                }
            }
        }

        if let Some(name) = full_name {
            if !name.trim().is_empty() {
                return Some(name.trim().to_string());
            }
        }

        match (family, subfamily) {
            (Some(f), Some(s)) => Some(format!("{} {}", f.trim(), s.trim())),
            (Some(f), None) => Some(f.trim().to_string()),
            _ => None,
        }
    }

    #[cfg(target_os = "windows")]
    fn register_windows_font(
        font_path: &Path,
        font_title: &str,
        is_truetype: bool,
    ) -> Result<(), String> {
        use winreg::enums::{HKEY_CURRENT_USER, KEY_SET_VALUE};
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (key, _) = hkcu
            .create_subkey_with_flags(
                "Software\\Microsoft\\Windows NT\\CurrentVersion\\Fonts",
                KEY_SET_VALUE,
            )
            .map_err(|e| format!("Failed to open Windows Fonts registry key: {e}"))?;

        let suffix = if is_truetype { "(TrueType)" } else { "(OpenType)" };
        let value_name = format!("{font_title} {suffix}");
        let full_path_str = font_path.to_string_lossy().to_string();

        key.set_value(&value_name, &full_path_str)
            .map_err(|e| format!("Failed to register font in registry: {e}"))?;

        // Inform GDI via AddFontResourceExW
        unsafe {
            use std::ffi::OsStr;
            use std::os::windows::ffi::OsStrExt;

            let wide_path: Vec<u16> = OsStr::new(&full_path_str)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();

            // FR_PRIVATE = 0x10. Using 0 registers it for the current session across all apps
            windows_sys::Win32::Graphics::Gdi::AddFontResourceExW(
                wide_path.as_ptr(),
                0,
                std::ptr::null_mut(),
            );
        }

        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn broadcast_font_change() {
        unsafe {
            use windows_sys::Win32::UI::WindowsAndMessaging::{
                SendMessageTimeoutW, SMTO_ABORTIFHUNG, WM_FONTCHANGE,
            };

            const HWND_BROADCAST: isize = 0xffff;

            let mut result: usize = 0;
            SendMessageTimeoutW(
                HWND_BROADCAST as _,
                WM_FONTCHANGE,
                0,
                0,
                SMTO_ABORTIFHUNG,
                1000,
                &mut result,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_fonts_dir_exists() {
        let dir = FontInstaller::get_user_fonts_dir();
        assert!(dir.is_ok());
        let path = dir.unwrap();
        assert!(path.exists());
    }
}
