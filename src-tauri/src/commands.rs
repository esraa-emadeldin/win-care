//! The commands the UI calls. Long work runs on a blocking thread so the
//! window never freezes. Every change goes scan -> plan -> apply.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_opener::OpenerExt;

use wincare_core::{
    Action, ActivityEntry, App, ApplyReport, CancelToken, CategoryId, Check, CleanupResult,
    Finding, Guard, Plan, SpaceNode, StartupItem, append_activity, apply, find_leftovers,
    finding_for_path, make_plan as core_make_plan, read_activity, scan_cleanup, scan_space,
    still_installed,
};
use wincare_winsys::{
    Drive, Vitals, VitalsSampler, WinFileOps, drives as win_drives, empty_recycle_bin,
    health_checks, list_apps as win_list_apps, list_startup as win_list_startup,
    recycle_bin_size as win_recycle_bin_size, restart_explorer, running_process_names,
    set_startup_enabled, uninstall,
};

use crate::settings::{self, Settings, permanent_allowed};
use crate::state::AppState;

/// How long to wait for an uninstaller that keeps running in the background.
const UNINSTALL_WAIT: std::time::Duration = std::time::Duration::from_secs(120);
const UNINSTALL_POLL: std::time::Duration = std::time::Duration::from_secs(2);

/// How deep the space map tree goes; deeper sizes are still counted.
const SPACE_DEPTH: u8 = 4;

type CmdResult<T> = Result<T, String>;

/// Runs `f` on a blocking thread with access to the app state.
async fn blocking<T: Send + 'static>(
    app: &AppHandle,
    f: impl FnOnce(&AppHandle, &AppState) -> CmdResult<T> + Send + 'static,
) -> CmdResult<T> {
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        f(&app, &state)
    })
    .await
    .map_err(|e| e.to_string())?
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as u64)
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SmartScan {
    pub cleanup: CleanupResult,
    pub cleanup_bytes: u64,
    pub apps_count: u32,
    pub apps_bytes: u64,
    pub startup_count: u32,
    pub health: Vec<Check>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ScanProgress {
    area: &'static str,
    label: String,
}

#[tauri::command]
pub async fn smart_scan(app: AppHandle) -> CmdResult<SmartScan> {
    blocking(&app, |app, st| {
        let progress = |area, label: &str| {
            let _ = app.emit(
                "scan-progress",
                ScanProgress {
                    area,
                    label: label.into(),
                },
            );
        };
        progress("cleanup", "Looking for junk files");
        let running = running_process_names();
        let cleanup = scan_cleanup(
            &st.env,
            now_ms(),
            &|name| running.iter().any(|r| r == name),
            &CancelToken::new(),
            &mut |_, _| {},
        );
        let cleanup_bytes = cleanup
            .findings
            .iter()
            .filter(|f| f.preselected)
            .map(|f| f.bytes)
            .sum();
        *st.last_cleanup.lock().unwrap() = Some(cleanup.clone());

        progress("apps", "Counting installed apps");
        let apps = win_list_apps();
        let apps_bytes = apps.iter().filter_map(|a| a.bytes).sum();
        let apps_count = apps.len() as u32;
        *st.apps.lock().unwrap() = apps;

        progress("startup", "Checking what starts with Windows");
        let startup = win_list_startup();
        let startup_count = startup.iter().filter(|s| s.enabled).count() as u32;
        *st.startup.lock().unwrap() = startup;

        progress("health", "Checking your PC's health");
        let alert = st.settings.lock().unwrap().low_disk_alert_pct;
        let health = health_checks(alert);

        Ok(SmartScan {
            cleanup,
            cleanup_bytes,
            apps_count,
            apps_bytes,
            startup_count,
            health,
        })
    })
    .await
}

/// Builds a plan from paths the UI selected. Only paths a scan found (or the
/// space map showed) are accepted; anything else is ignored.
#[tauri::command]
pub async fn make_plan(app: AppHandle, paths: Vec<String>, action: Action) -> CmdResult<Plan> {
    blocking(&app, move |_, st| {
        let wanted: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();
        let findings = known_findings(st, &wanted);
        if action == Action::DeletePermanently
            && !permanent_allowed(&st.settings.lock().unwrap(), &findings)
        {
            return Err(
                "Permanent delete is only allowed for caches, when turned on in Settings".into(),
            );
        }
        let plan = core_make_plan(findings, action, &WinFileOps, &Guard::new(&st.env));
        st.plans
            .lock()
            .unwrap()
            .insert(plan.id.clone(), plan.clone());
        Ok(plan)
    })
    .await
}

fn known_findings(st: &AppState, wanted: &[PathBuf]) -> Vec<Finding> {
    let same = |a: &Path, b: &Path| {
        a.to_string_lossy()
            .eq_ignore_ascii_case(&b.to_string_lossy())
    };
    let mut pool: Vec<Finding> = st
        .last_cleanup
        .lock()
        .unwrap()
        .as_ref()
        .map(|c| c.findings.clone())
        .unwrap_or_default();
    pool.extend(st.last_leftovers.lock().unwrap().iter().cloned());
    let space = st.space_paths.lock().unwrap().clone();

    wanted
        .iter()
        .filter_map(|p| {
            if let Some(f) = pool.iter().find(|f| same(&f.path, p)) {
                return Some(f.clone());
            }
            if space.iter().any(|s| same(s, p)) {
                return finding_for_path(p, CategoryId::SpaceMap);
            }
            None
        })
        .collect()
}

#[tauri::command]
pub async fn apply_plan(app: AppHandle, plan_id: String) -> CmdResult<ApplyReport> {
    blocking(&app, move |_, st| {
        let plan = st
            .plans
            .lock()
            .unwrap()
            .remove(&plan_id)
            .ok_or("This plan is no longer valid; scan again")?;
        let report = apply(&plan, &WinFileOps, &Guard::new(&st.env));
        let entry = ActivityEntry {
            at_ms: now_ms(),
            action: plan.action,
            done: report.done.len() as u64,
            freed_bytes: report.freed_bytes,
            skipped: report.skipped.len() as u64,
            paths: report.done.clone(),
        };
        let _ = append_activity(&st.activity_file(), &entry);
        // What was moved is no longer junk to offer.
        if let Some(c) = st.last_cleanup.lock().unwrap().as_mut() {
            c.findings.retain(|f| !report.done.contains(&f.path));
        }
        st.last_leftovers
            .lock()
            .unwrap()
            .retain(|f| !report.done.contains(&f.path));
        Ok(report)
    })
    .await
}

#[tauri::command]
pub async fn list_apps(app: AppHandle) -> CmdResult<Vec<App>> {
    blocking(&app, |_, st| {
        let apps = win_list_apps();
        *st.apps.lock().unwrap() = apps.clone();
        Ok(apps)
    })
    .await
}

/// Runs the app's own uninstaller, then looks for what it left behind. If the
/// app is still installed afterwards (cancelled or failed), nothing is offered.
#[tauri::command]
pub async fn uninstall_app(app: AppHandle, id: String) -> CmdResult<Vec<Finding>> {
    blocking(&app, move |_, st| {
        let target = st
            .apps
            .lock()
            .unwrap()
            .iter()
            .find(|a| a.id == id)
            .cloned()
            .ok_or("App not found; refresh the list")?;
        uninstall(&target)?;
        // Many uninstallers (NSIS, Inno) hand over to a copy of themselves and
        // exit at once, so wait until the app is really gone, or give up.
        let deadline = std::time::Instant::now() + UNINSTALL_WAIT;
        let mut after = win_list_apps();
        while still_installed(&target, &after) && std::time::Instant::now() < deadline {
            std::thread::sleep(UNINSTALL_POLL);
            after = win_list_apps();
        }
        *st.apps.lock().unwrap() = after.clone();
        if still_installed(&target, &after) {
            st.last_leftovers.lock().unwrap().clear();
            return Ok(Vec::new());
        }
        let leftovers = find_leftovers(&target, &after, &st.env);
        *st.last_leftovers.lock().unwrap() = leftovers.clone();
        Ok(leftovers)
    })
    .await
}

#[tauri::command]
pub async fn list_startup(app: AppHandle) -> CmdResult<Vec<StartupItem>> {
    blocking(&app, |_, st| {
        let items = win_list_startup();
        *st.startup.lock().unwrap() = items.clone();
        Ok(items)
    })
    .await
}

#[tauri::command]
pub async fn set_startup(app: AppHandle, id: String, enabled: bool) -> CmdResult<()> {
    blocking(&app, move |_, st| {
        let mut items = st.startup.lock().unwrap();
        let item = items
            .iter_mut()
            .find(|i| i.id == id)
            .ok_or("Startup item not found; refresh the list")?;
        set_startup_enabled(item, enabled)?;
        item.enabled = enabled;
        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn health(app: AppHandle) -> CmdResult<Vec<Check>> {
    blocking(&app, |_, st| {
        let alert = st.settings.lock().unwrap().low_disk_alert_pct;
        Ok(health_checks(alert))
    })
    .await
}

#[tauri::command]
pub async fn run_maintenance(app: AppHandle, task: String) -> CmdResult<String> {
    blocking(&app, move |_, _| match task.as_str() {
        "restartExplorer" => restart_explorer(),
        "emptyRecycleBin" => empty_recycle_bin().map(|()| "Recycle Bin emptied".into()),
        other => Err(format!("unknown task {other}")),
    })
    .await
}

#[tauri::command]
pub async fn recycle_bin_size() -> CmdResult<u64> {
    tauri::async_runtime::spawn_blocking(win_recycle_bin_size)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn drives() -> CmdResult<Vec<Drive>> {
    tauri::async_runtime::spawn_blocking(win_drives)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn space_scan(app: AppHandle, root: String) -> CmdResult<SpaceNode> {
    blocking(&app, move |app, st| {
        let cancel = CancelToken::new();
        *st.space_cancel.lock().unwrap() = cancel.clone();
        let tree = scan_space(Path::new(&root), SPACE_DEPTH, &cancel, &mut |node| {
            let _ = app.emit("space-partial", node);
        });
        let mut paths = Vec::new();
        collect_paths(&tree, &mut paths);
        *st.space_paths.lock().unwrap() = paths;
        Ok(tree)
    })
    .await
}

fn collect_paths(node: &SpaceNode, out: &mut Vec<PathBuf>) {
    if !node.path.as_os_str().is_empty() {
        out.push(node.path.clone());
    }
    for child in &node.children {
        collect_paths(child, out);
    }
}

#[tauri::command]
pub fn cancel_space_scan(state: tauri::State<'_, AppState>) {
    state.space_cancel.lock().unwrap().cancel();
}

#[tauri::command]
pub async fn vitals(app: AppHandle) -> CmdResult<Vitals> {
    blocking(&app, |_, st| {
        let mut sampler = st.sampler.lock().unwrap();
        if sampler.is_none() {
            *sampler = Some(VitalsSampler::new()?);
        }
        Ok(sampler.as_mut().unwrap().sample())
    })
    .await
}

#[tauri::command]
pub fn get_settings(state: tauri::State<'_, AppState>) -> Settings {
    state.settings.lock().unwrap().clone()
}

#[tauri::command]
pub fn set_settings(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    settings: Settings,
) -> CmdResult<()> {
    let autostart = app.autolaunch();
    let result = if settings.start_with_windows {
        autostart.enable()
    } else {
        autostart.disable()
    };
    result.map_err(|e| e.to_string())?;
    settings::save(&state.settings_file(), &settings).map_err(|e| e.to_string())?;
    *state.settings.lock().unwrap() = settings;
    Ok(())
}

#[tauri::command]
pub fn activity(state: tauri::State<'_, AppState>, limit: u32) -> Vec<ActivityEntry> {
    read_activity(&state.activity_file(), limit as usize)
}

/// Opens a Settings page, Task Scheduler or the Recycle Bin. Only these
/// targets are allowed, so the UI cannot be used to launch arbitrary things.
#[tauri::command]
pub fn open_target(app: AppHandle, target: String) -> CmdResult<()> {
    let allowed = target.starts_with("ms-settings:")
        || target.starts_with("windowsdefender:")
        || target == "shell:RecycleBinFolder"
        || target == "taskschd.msc";
    if !allowed {
        return Err(format!("not allowed: {target}"));
    }
    app.opener()
        .open_path(target, None::<&str>)
        .map_err(|e| e.to_string())
}

/// Shows a file or folder in Explorer.
#[tauri::command]
pub fn reveal_path(app: AppHandle, path: String) -> CmdResult<()> {
    app.opener()
        .reveal_item_in_dir(&path)
        .map_err(|e| e.to_string())
}
