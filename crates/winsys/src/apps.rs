//! Installed apps, read from the same places Windows Settings uses:
//! the Uninstall registry keys and the Store (MSIX) package list.

use std::collections::HashSet;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

use windows::Management::Deployment::PackageManager;
use windows::core::HSTRING;
use winreg::RegKey;
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ};

use wincare_core::{App, AppKind};

const UNINSTALL: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall";
const UNINSTALL_32: &str = r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall";

/// Every installed app a user would recognise, without system components,
/// updates and frameworks. Reads only.
pub fn list_apps() -> Vec<App> {
    let mut apps = Vec::new();
    for (hive, hive_name, path) in [
        (HKEY_LOCAL_MACHINE, "hklm", UNINSTALL),
        (HKEY_LOCAL_MACHINE, "hklm32", UNINSTALL_32),
        (HKEY_CURRENT_USER, "hkcu", UNINSTALL),
    ] {
        apps.extend(registry_apps(&RegKey::predef(hive), hive_name, path));
    }
    apps.extend(msix_apps());

    // The same app is often registered twice (e.g. per-user and per-machine).
    let mut seen = HashSet::new();
    apps.retain(|a| seen.insert((a.name.to_lowercase(), a.version.clone())));
    apps.sort_by_key(|a| a.name.to_lowercase());
    apps
}

fn registry_apps(hive: &RegKey, hive_name: &str, path: &str) -> Vec<App> {
    let Ok(root) = hive.open_subkey_with_flags(path, KEY_READ) else {
        return Vec::new();
    };
    root.enum_keys()
        .flatten()
        .filter_map(|sub| {
            let key = root.open_subkey_with_flags(&sub, KEY_READ).ok()?;
            let get = |name: &str| -> Option<String> {
                key.get_value::<String, _>(name)
                    .ok()
                    .map(|v| v.trim().to_string())
                    .filter(|v| !v.is_empty())
            };
            let system: u32 = key.get_value("SystemComponent").unwrap_or(0);
            if system == 1 || get("ParentKeyName").is_some() {
                return None;
            }
            let name = get("DisplayName")?;
            let uninstall_cmd = get("UninstallString")?;
            let size_kb: Option<u32> = key.get_value("EstimatedSize").ok();
            Some(App {
                id: format!("{hive_name}:{sub}").to_lowercase(),
                name,
                publisher: get("Publisher"),
                version: get("DisplayVersion"),
                bytes: size_kb.filter(|kb| *kb > 0).map(|kb| u64::from(kb) * 1024),
                last_used_ms: None,
                kind: AppKind::Win32 {
                    uninstall_cmd,
                    quiet_cmd: get("QuietUninstallString"),
                    install_location: get("InstallLocation")
                        .map(|p| PathBuf::from(p.trim_matches('"'))),
                },
            })
        })
        .collect()
}

/// Store and MSIX apps for the current user, without frameworks, resource
/// packages and apps that ship with Windows.
fn msix_apps() -> Vec<App> {
    let Ok(manager) = PackageManager::new() else {
        return Vec::new();
    };
    let Ok(packages) = manager.FindPackagesByUserSecurityId(&HSTRING::new()) else {
        return Vec::new();
    };
    packages
        .into_iter()
        .filter_map(|p| {
            if p.IsFramework().unwrap_or(true) || p.IsResourcePackage().unwrap_or(true) {
                return None;
            }
            // SignatureKind 4 = System: part of Windows itself.
            if p.SignatureKind().map(|k| k.0).unwrap_or(4) == 4 {
                return None;
            }
            let name = p.DisplayName().ok()?.to_string();
            if name.trim().is_empty() || name.starts_with("ms-resource:") {
                return None;
            }
            let id = p.Id().ok()?;
            let full_name = id.FullName().ok()?.to_string();
            let v = id.Version().ok()?;
            Some(App {
                id: format!("msix:{full_name}").to_lowercase(),
                name,
                publisher: p.PublisherDisplayName().ok().map(|s| s.to_string()),
                version: Some(format!(
                    "{}.{}.{}.{}",
                    v.Major, v.Minor, v.Build, v.Revision
                )),
                bytes: None,
                last_used_ms: None,
                kind: AppKind::Msix { full_name },
            })
        })
        .collect()
}

/// Runs the app's own uninstaller and waits for it. Its window and any UAC
/// prompt belong to the vendor's uninstaller, not to win-care.
pub fn uninstall(app: &App) -> Result<i32, String> {
    match &app.kind {
        AppKind::Win32 { uninstall_cmd, .. } => {
            // The registry stores a full command line, quotes included, so
            // hand it to cmd unchanged instead of re-splitting it.
            let status = Command::new("cmd")
                .raw_arg(format!("/C \"{uninstall_cmd}\""))
                .status()
                .map_err(|e| format!("could not start the uninstaller: {e}"))?;
            Ok(status.code().unwrap_or(-1))
        }
        AppKind::Msix { full_name } => {
            let manager = PackageManager::new().map_err(|e| e.message())?;
            manager
                .RemovePackageAsync(&HSTRING::from(full_name.as_str()))
                .and_then(|op| op.join())
                .map_err(|e| e.message())?;
            Ok(0)
        }
    }
}
