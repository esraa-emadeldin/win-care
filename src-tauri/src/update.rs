//! The opt-in update check: at most once a day, only when the user turned it
//! on, and nothing is installed until they say yes.

use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use tauri::{AppHandle, Manager};
use tauri_plugin_updater::UpdaterExt;

use crate::state::AppState;

const DAY_MS: u64 = 24 * 60 * 60 * 1000;

/// True when the last check was a day or more ago (or never happened).
pub fn is_due(last_ms: u64, now_ms: u64) -> bool {
    last_ms == 0 || (now_ms >= last_ms && now_ms - last_ms >= DAY_MS)
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as u64)
}

/// The newer version, if one exists and a check was due. Never downloads.
#[tauri::command]
pub async fn check_update(app: AppHandle) -> Result<Option<String>, String> {
    let state = app.state::<AppState>();
    if !state.settings.lock().unwrap().check_updates {
        return Ok(None);
    }
    let stamp = state.data_dir.join("last-update-check");
    let last = fs::read_to_string(&stamp)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0);
    let now = now_ms();
    if !is_due(last, now) {
        return Ok(None);
    }
    let _ = fs::create_dir_all(&state.data_dir);
    let _ = fs::write(&stamp, now.to_string());
    let update = app
        .updater()
        .map_err(|e| e.to_string())?
        .check()
        .await
        .map_err(|e| e.to_string())?;
    Ok(update.map(|u| u.version))
}

/// Downloads, verifies the signature and installs; the installer restarts the app.
#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), String> {
    let update = app
        .updater()
        .map_err(|e| e.to_string())?
        .check()
        .await
        .map_err(|e| e.to_string())?
        .ok_or("No update available")?;
    update
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOUR: u64 = 60 * 60 * 1000;

    #[test]
    fn due_only_once_a_day() {
        let now = 1_000 * 24 * HOUR;
        assert!(is_due(0, now), "never checked");
        assert!(!is_due(now - 23 * HOUR, now));
        assert!(is_due(now - 25 * HOUR, now));
        assert!(!is_due(now + HOUR, now), "clock moved back: wait");
    }
}
