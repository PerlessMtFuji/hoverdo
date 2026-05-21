//! System tray integration.
//!
//! Left-click toggles the main window; the menu offers explicit show, show
//! all widgets, hide all widgets, and quit. The main window's X button is
//! rewired to hide-to-tray rather than close, so the tray stays the single
//! exit point.

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, WindowEvent};

use crate::error::{HoverdoError, Result};

const ID_SHOW: &str = "tray:show";
const ID_SHOW_WIDGETS: &str = "tray:show-widgets";
const ID_HIDE_WIDGETS: &str = "tray:hide-widgets";
const ID_QUIT: &str = "tray:quit";

const MAIN_WINDOW: &str = "main";

pub fn install(app: &AppHandle) -> Result<()> {
    let menu = Menu::with_items(
        app,
        &[
            &MenuItem::with_id(app, ID_SHOW, "Show Hoverdo", true, None::<&str>)
                .map_err(boxed)?,
            &MenuItem::with_id(app, ID_SHOW_WIDGETS, "Show all widgets", true, None::<&str>)
                .map_err(boxed)?,
            &MenuItem::with_id(app, ID_HIDE_WIDGETS, "Hide all widgets", true, None::<&str>)
                .map_err(boxed)?,
            &PredefinedMenuItem::separator(app).map_err(boxed)?,
            &MenuItem::with_id(app, ID_QUIT, "Quit", true, None::<&str>).map_err(boxed)?,
        ],
    )
    .map_err(boxed)?;

    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| HoverdoError::internal("no bundled window icon"))?;

    TrayIconBuilder::with_id("hoverdo")
        .icon(icon)
        .tooltip("Hoverdo")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            ID_SHOW => show_main(app),
            ID_SHOW_WIDGETS => set_widgets_visible(app, true),
            ID_HIDE_WIDGETS => set_widgets_visible(app, false),
            ID_QUIT => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                toggle_main(&tray.app_handle().clone());
            }
        })
        .build(app)
        .map_err(boxed)?;

    Ok(())
}

/// Rewire the main window's close button to hide-to-tray. Called once at
/// setup; the closure outlives `app` because it captures only the cheap
/// handle clone.
pub fn install_main_close_to_tray(app: &AppHandle) {
    let Some(main) = app.get_webview_window(MAIN_WINDOW) else {
        return;
    };
    let handle = app.clone();
    main.on_window_event(move |event| {
        if let WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            if let Some(win) = handle.get_webview_window(MAIN_WINDOW) {
                let _ = win.hide();
            }
        }
    });
}

fn show_main(app: &AppHandle) {
    if let Some(win) = app.get_webview_window(MAIN_WINDOW) {
        let _ = win.show();
        let _ = win.unminimize();
        let _ = win.set_focus();
    }
}

fn toggle_main(app: &AppHandle) {
    if let Some(win) = app.get_webview_window(MAIN_WINDOW) {
        match win.is_visible() {
            Ok(true) => {
                let _ = win.hide();
            }
            _ => {
                let _ = win.show();
                let _ = win.unminimize();
                let _ = win.set_focus();
            }
        }
    }
}

fn set_widgets_visible(app: &AppHandle, visible: bool) {
    for (label, win) in app.webview_windows() {
        if label == MAIN_WINDOW {
            continue;
        }
        // Widget labels start with "sticky-" or "todo-" - everything else
        // is some future window we shouldn't toggle blindly.
        if !(label.starts_with("sticky-") || label.starts_with("todo-")) {
            continue;
        }
        if visible {
            let _ = win.show();
        } else {
            let _ = win.hide();
        }
    }
}

fn boxed<E: std::fmt::Display>(e: E) -> HoverdoError {
    HoverdoError::internal(e.to_string())
}
