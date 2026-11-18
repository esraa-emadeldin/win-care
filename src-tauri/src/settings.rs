//! User settings, stored as JSON in %APPDATA%\win-care\settings.json.
//! A missing or damaged file means defaults, never a crash.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use wincare_core::{CategoryId, Finding};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub start_with_windows: bool,
    /// "system", "light" or "dark".
    pub theme: String,
    pub permanent_delete_caches: bool,
    pub low_disk_alert_pct: u8,
    pub check_updates: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            start_with_windows: false,
            theme: "system".into(),
            permanent_delete_caches: false,
            low_disk_alert_pct: 10,
            check_updates: false,
        }
    }
}

pub fn load(file: &Path) -> Settings {
    fs::read_to_string(file)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

pub fn save(file: &Path, settings: &Settings) -> std::io::Result<()> {
    if let Some(dir) = file.parent() {
        fs::create_dir_all(dir)?;
    }
    let text = serde_json::to_string_pretty(settings).map_err(std::io::Error::other)?;
    fs::write(file, text)
}

/// Permanent delete is allowed only when the user turned it on, and only for
/// caches: never for downloads, leftovers or anything picked in the space map.
pub fn permanent_allowed(settings: &Settings, findings: &[Finding]) -> bool {
    settings.permanent_delete_caches
        && !findings.is_empty()
        && findings.iter().all(|f| {
            matches!(
                f.category,
                CategoryId::UserTemp
                    | CategoryId::BrowserCache
                    | CategoryId::ThumbnailCache
                    | CategoryId::CrashDumps
            )
        })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn finding(category: CategoryId) -> Finding {
        Finding {
            id: "x".into(),
            category,
            path: PathBuf::from(r"C:\x"),
            bytes: 1,
            is_dir: false,
            modified_ms: 0,
            preselected: true,
            selectable: true,
            note: None,
        }
    }

    #[test]
    fn defaults_when_missing() {
        let dir = tempfile::tempdir().unwrap();
        let s = load(&dir.path().join("settings.json"));
        assert_eq!(s, Settings::default());
        assert!(!s.start_with_windows && !s.permanent_delete_caches && !s.check_updates);
        assert_eq!(s.theme, "system");
        assert_eq!(s.low_disk_alert_pct, 10);
    }

    #[test]
    fn defaults_when_corrupt() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("settings.json");
        fs::write(&file, "{ not json").unwrap();
        assert_eq!(load(&file), Settings::default());
    }

    #[test]
    fn roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("sub").join("settings.json");
        let s = Settings {
            theme: "dark".into(),
            low_disk_alert_pct: 15,
            ..Settings::default()
        };
        save(&file, &s).unwrap();
        assert_eq!(load(&file), s);
    }

    #[test]
    fn permanent_blocked_by_default() {
        let s = Settings::default();
        assert!(!permanent_allowed(&s, &[finding(CategoryId::UserTemp)]));
    }

    #[test]
    fn permanent_blocked_for_installers() {
        let s = Settings {
            permanent_delete_caches: true,
            ..Settings::default()
        };
        assert!(permanent_allowed(&s, &[finding(CategoryId::BrowserCache)]));
        assert!(!permanent_allowed(
            &s,
            &[
                finding(CategoryId::BrowserCache),
                finding(CategoryId::OldInstallers)
            ]
        ));
    }
}
