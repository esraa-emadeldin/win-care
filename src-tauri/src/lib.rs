//! win-care: the Tauri app. Wires commands, tray, windows and the background
//! low-disk check together. All real work lives in wincare-core/-winsys.

mod commands;
mod settings;
mod state;
mod tray;
mod update;

use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

use tauri::{Manager, WindowEvent};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_notification::NotificationExt;

use state::AppState;

/// How often the background check looks at free disk space.
const DISK_CHECK_EVERY: Duration = Duration::from_secs(60);
/// At most one low-disk notification per day.
const NOTIFY_AT_MOST_EVERY: Duration = Duration::from_secs(24 * 60 * 60);

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            tray::show_main(app);
        }))
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let env = wincare_winsys::system_env()?;
            let data_dir = std::env::var_os("APPDATA")
                .map(PathBuf::from)
                .ok_or("APPDATA is not set")?
                .join("win-care");
            app.manage(AppState::new(env, data_dir));
            tray::create(app.handle())?;
            // Started at login: stay in the tray.
            if std::env::args().any(|a| a == "--minimized")
                && let Some(main) = app.get_webview_window("main")
            {
                let _ = main.hide();
            }
            start_disk_watch(app.handle().clone());
            Ok(())
        })
        .on_window_event(|window, event| match (window.label(), event) {
            // Closing the main window keeps win-care in the tray.
            ("main", WindowEvent::CloseRequested { api, .. }) => {
                api.prevent_close();
                let _ = window.hide();
            }
            ("tray", WindowEvent::Focused(false)) => {
                let _ = window.hide();
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            commands::smart_scan,
            commands::make_plan,
            commands::apply_plan,
            commands::list_apps,
            commands::uninstall_app,
            commands::list_startup,
            commands::set_startup,
            commands::health,
            commands::run_maintenance,
            commands::recycle_bin_size,
            commands::drives,
            commands::space_scan,
            commands::cancel_space_scan,
            commands::vitals,
            commands::get_settings,
            commands::set_settings,
            commands::activity,
            commands::open_target,
            commands::reveal_path,
            update::check_update,
            update::install_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running win-care");
}

/// Every minute, checks free space on the system drive; warns once a day at
/// most when it drops below the user's threshold.
fn start_disk_watch(app: tauri::AppHandle) {
    thread::spawn(move || {
        let mut last_notified: Option<Instant> = None;
        loop {
            thread::sleep(DISK_CHECK_EVERY);
            let pct = app
                .state::<AppState>()
                .settings
                .lock()
                .unwrap()
                .low_disk_alert_pct;
            let system = std::env::var("SystemDrive").unwrap_or_else(|_| "C:".into());
            let letter = system.chars().next().unwrap_or('C');
            let Some(drive) = wincare_winsys::drives()
                .into_iter()
                .find(|d| d.letter == letter)
            else {
                continue;
            };
            let low = drive.total > 0 && drive.free * 100 / drive.total < u64::from(pct);
            let due = last_notified.is_none_or(|t| t.elapsed() >= NOTIFY_AT_MOST_EVERY);
            if low && due {
                let _ = app
                    .notification()
                    .builder()
                    .title("Your PC is low on space")
                    .body(format!(
                        "Less than {pct} % free on {letter}:. Open win-care to free up space."
                    ))
                    .show();
                last_notified = Some(Instant::now());
            }
        }
    });
}
