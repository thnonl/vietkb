#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::Manager;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "vkb_app=info,vkb_windows=info".into()),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let config = commands::config_cell().read().clone();
            tracing::info!(
                enabled = config.input.enabled,
                method = config.input.method.as_str(),
                "VietKB starting (sprint 0 scaffold: no hook installed yet)"
            );

            let toggle = MenuItem::with_id(app, "toggle", "Bật/Tắt tiếng Việt", true, None::<&str>)?;
            let settings = MenuItem::with_id(app, "settings", "Cài đặt…", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Thoát", true, None::<&str>)?;
            let sep = PredefinedMenuItem::separator(app)?;
            let menu = Menu::with_items(app, &[&toggle, &settings, &sep, &quit])?;

            TrayIconBuilder::with_id("main")
                .tooltip("VietKB")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "toggle" => {
                        let status = commands::toggle_enabled();
                        tracing::info!(enabled = status.enabled, "toggled from tray");
                    }
                    "settings" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;

            // Start hidden in the tray unless the user asked otherwise.
            if !config.system.start_minimized {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                }
            }
            Ok(())
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
            commands::delete_app_rule,
            commands::list_running_apps,
            commands::parse_hotkey,
            commands::check_hotkey_conflict,
            commands::convert_text,
            commands::open_config_folder,
            commands::app_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running VietKB");
}
