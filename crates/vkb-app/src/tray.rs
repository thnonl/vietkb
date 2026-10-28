//! Tray icon: state, method and charset submenus with checkmarks, and an icon
//! that reflects whether Vietnamese input is on.

use tauri::menu::{CheckMenuItem, IsMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Wry};

use crate::commands::{self, StatusView};

pub const TRAY_ID: &str = "vietkb-tray";

const METHODS: [(&str, &str); 3] = [("telex", "Telex"), ("vni", "VNI"), ("viqr", "VIQR")];

const CHARSETS: [(&str, &str); 6] = [
    ("unicode", "Unicode dựng sẵn"),
    ("unicode_nfd", "Unicode tổ hợp"),
    ("tcvn3", "TCVN3 (ABC)"),
    ("vni_windows", "VNI Windows"),
    ("viqr", "VIQR"),
    ("viscii", "VISCII"),
];

fn charset_short(slug: &str) -> &str {
    match slug {
        "unicode" => "Unicode",
        "unicode_nfd" => "Unicode tổ hợp",
        "tcvn3" => "TCVN3",
        "vni_windows" => "VNI Windows",
        "viqr" => "VIQR",
        "viscii" => "VISCII",
        _ => "Unicode",
    }
}

fn icon_for(status: &StatusView) -> tauri::Result<tauri::image::Image<'static>> {
    let bytes: &'static [u8] = if status.enabled {
        include_bytes!("../icons/tray-on.png")
    } else {
        include_bytes!("../icons/tray-off.png")
    };
    tauri::image::Image::from_bytes(bytes)
}

fn tooltip(status: &StatusView) -> String {
    if status.enabled {
        format!(
            "VietKB · Tiếng Việt · {} · {}",
            method_label(&status.method),
            charset_short(&status.charset)
        )
    } else {
        "VietKB · Đã tắt tiếng Việt".to_string()
    }
}

fn method_label(slug: &str) -> &str {
    METHODS
        .iter()
        .find(|(value, _)| *value == slug)
        .map(|(_, label)| *label)
        .unwrap_or("Telex")
}

fn build_menu(app: &AppHandle, status: &StatusView) -> tauri::Result<Menu<Wry>> {
    let toggle = CheckMenuItem::with_id(
        app,
        "toggle",
        "Bật/tắt tiếng Việt",
        true,
        status.enabled,
        None::<&str>,
    )?;

    let mut method_items = Vec::with_capacity(METHODS.len());
    for (slug, label) in METHODS {
        method_items.push(CheckMenuItem::with_id(
            app,
            format!("method:{slug}"),
            label,
            true,
            status.method == slug,
            None::<&str>,
        )?);
    }
    let method_refs: Vec<&dyn IsMenuItem<Wry>> = method_items
        .iter()
        .map(|item| item as &dyn IsMenuItem<Wry>)
        .collect();
    let method_menu = Submenu::with_items(app, "Kiểu gõ", true, &method_refs)?;

    let mut charset_items = Vec::with_capacity(CHARSETS.len());
    for (slug, label) in CHARSETS {
        charset_items.push(CheckMenuItem::with_id(
            app,
            format!("charset:{slug}"),
            label,
            true,
            status.charset == slug,
            None::<&str>,
        )?);
    }
    let charset_refs: Vec<&dyn IsMenuItem<Wry>> = charset_items
        .iter()
        .map(|item| item as &dyn IsMenuItem<Wry>)
        .collect();
    let charset_menu = Submenu::with_items(app, "Bảng mã", true, &charset_refs)?;

    let settings = MenuItem::with_id(app, "settings", "Mở cài đặt…", true, None::<&str>)?;
    let hotkeys = MenuItem::with_id(app, "hotkeys", "Phím tắt…", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Thoát VietKB", true, None::<&str>)?;
    let sep_top = PredefinedMenuItem::separator(app)?;
    let sep_open = PredefinedMenuItem::separator(app)?;
    let sep_bottom = PredefinedMenuItem::separator(app)?;

    let items: Vec<&dyn IsMenuItem<Wry>> = vec![
        &toggle,
        &sep_top,
        &method_menu,
        &charset_menu,
        &sep_open,
        &settings,
        &hotkeys,
        &sep_bottom,
        &quit,
    ];
    Menu::with_items(app, &items)
}

/// Create the tray icon with the current state. Called once during setup.
pub fn install(app: &AppHandle) -> tauri::Result<()> {
    let status = commands::status_now();
    let menu = build_menu(app, &status)?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon_for(&status)?)
        .tooltip(tooltip(&status))
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| handle_event(app, event.id.as_ref()))
        .build(app)?;

    Ok(())
}

/// Rebuild the menu and icon after any state change. Never panics: a tray that
/// fails to refresh must not take the app down.
pub fn refresh(app: &AppHandle, status: &StatusView) {
    if let Err(error) = try_refresh(app, status) {
        tracing::warn!(%error, "could not refresh tray menu");
    }
}

fn try_refresh(app: &AppHandle, status: &StatusView) -> tauri::Result<()> {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return Ok(());
    };
    let menu = build_menu(app, status)?;
    tray.set_menu(Some(menu))?;
    tray.set_icon(Some(icon_for(status)?))?;
    tray.set_tooltip(Some(tooltip(status)))?;
    Ok(())
}

fn handle_event(app: &AppHandle, id: &str) {
    match id {
        "quit" => crate::quit(app),
        "settings" => crate::show_settings(app),
        "hotkeys" => {
            if let Err(error) = crate::show_aux(app, "hotkeys") {
                tracing::warn!(%error, "tray: could not open the hotkeys window");
            }
        }
        "toggle" => {
            let _ = commands::toggle_enabled();
        }
        _ => {
            if let Some(slug) = id.strip_prefix("method:") {
                if let Err(error) = commands::set_method(slug.to_string()) {
                    tracing::warn!(%error, "tray: set method failed");
                }
            } else if let Some(slug) = id.strip_prefix("charset:") {
                if let Err(error) = commands::set_charset(slug.to_string()) {
                    tracing::warn!(%error, "tray: set charset failed");
                }
            }
        }
    }
}
