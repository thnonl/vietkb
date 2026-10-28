#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod charset_convert;
mod commands;
mod running_apps;
mod tray;

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{Manager, WindowEvent};

/// Set only by the tray's quit item. Everything else is "hide to tray".
static QUITTING: AtomicBool = AtomicBool::new(false);

/// Bring the settings window forward from anywhere (tray, hotkey, second launch).
pub fn show_settings(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

/// The real exit. Closing the window never does this.
pub fn quit(app: &tauri::AppHandle) {
    QUITTING.store(true, Ordering::SeqCst);
    app.exit(0);
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "vkb_app=info,vkb_windows=info".into()),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_settings(app);
        }))
        // The `--autostart` argument is what lets the app tell "Windows started
        // me" apart from "the user double-clicked the exe".
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .arg("--autostart")
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        // No updater plugin: update checking is Sprint 4. Registering it without a
        // `plugins.updater` block in tauri.conf.json fails plugin init at startup.
        .setup(|app| {
            let handle = app.handle().clone();
            commands::set_app_handle(handle.clone());

            let config = commands::config_cell().read().clone();

            // Make the Run key match the saved config on every start, so a
            // config copied between machines does not lie about autostart.
            commands::sync_autostart_now(config.system.start_with_windows);

            tray::install(&handle)?;

            tracing::info!(
                enabled = config.input.enabled,
                method = config.input.method.as_str(),
                charset = config.input.charset.as_str(),
                hook_ready = false,
                "VietKB starting (keyboard hook comes from the Windows stream)"
            );

            let launched_by_autostart = std::env::args().any(|arg| arg == "--autostart");
            let should_show = !config.system.start_minimized || !launched_by_autostart;
            if should_show {
                show_settings(&handle);
            }

            Ok(())
        })
        // Closing the window only hides it: the tray keeps the app (and the
        // hook) alive. A real quit is the tray menu's "Thoát VietKB".
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                if crate::QUITTING.load(std::sync::atomic::Ordering::SeqCst) {
                    return;
                }
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::set_settings,
            commands::patch_settings,
            commands::reset_settings,
            commands::get_status,
            commands::toggle_enabled,
            commands::set_method,
            commands::set_charset,
            commands::list_macros,
            commands::upsert_macro,
            commands::delete_macros,
            commands::import_macros,
            commands::export_macros,
            commands::list_app_rules,
            commands::set_app_rule,
            commands::set_app_rule_overrides,
            commands::delete_app_rule,
            commands::list_running_apps,
            commands::parse_hotkey,
            commands::check_hotkey_conflict,
            commands::convert_text,
            commands::open_config_folder,
            commands::restart_app,
            commands::app_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running VietKB");
}
