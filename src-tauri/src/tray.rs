//! The tray icon: left click shows the small popup above it, the menu opens
//! the main window or quits.

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, PhysicalPosition};

/// Gap between the popup and the taskbar, in physical pixels.
const POPUP_GAP: i32 = 12;

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Open win-care", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &quit])?;

    TrayIconBuilder::with_id("main")
        .tooltip("win-care")
        .icon(app.default_window_icon().cloned().expect("app icon"))
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "open" => show_main(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                position,
                ..
            } = event
            {
                toggle_popup(tray.app_handle(), position);
            }
        })
        .build(app)?;
    Ok(())
}

pub fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

/// Shows the popup centred above the click, or hides it if it is open.
fn toggle_popup(app: &AppHandle, click: PhysicalPosition<f64>) {
    let Some(popup) = app.get_webview_window("tray") else {
        return;
    };
    if popup.is_visible().unwrap_or(false) {
        let _ = popup.hide();
        return;
    }
    if let Ok(size) = popup.outer_size() {
        let x = click.x as i32 - size.width as i32 / 2;
        let y = click.y as i32 - size.height as i32 - POPUP_GAP;
        let _ = popup.set_position(PhysicalPosition::new(x.max(0), y.max(0)));
    }
    let _ = popup.show();
    let _ = popup.set_focus();
}
