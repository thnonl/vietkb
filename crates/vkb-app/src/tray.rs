//! Tray icon: state and method submenus with checkmarks, and an icon that
//! reflects whether Vietnamese input is on.
//!
//! Mouse: a left click toggles Vietnamese (the same path as the menu's own
//! `Bật/tắt tiếng Việt`), a right click opens the menu. Nothing on the tray
//! opens the settings window any more — that is the menu's `Mở cài đặt…`.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use tauri::menu::{CheckMenuItem, IsMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Wry};

use crate::commands::{self, StatusView};
use crate::update;

pub const TRAY_ID: &str = "vietkb-tray";

const METHODS: [(&str, &str); 2] = [("telex", "Telex"), ("vni", "VNI")];

// Approved keycap mark (design/logo11/keycap): `V` = Vietnamese on, `E` = off.
// Windows takes one bitmap per tray icon, so the size is chosen from the
// monitor scale (16 px @100 %, 20 px @125/150 %, 32 px otherwise) instead of
// shipping a multi-size ICO.
const ON_16: &[u8] = include_bytes!("../icons/tray-on-16.png");
const ON_20: &[u8] = include_bytes!("../icons/tray-on-20.png");
const ON_32: &[u8] = include_bytes!("../icons/tray-on-32.png");
const OFF_16: &[u8] = include_bytes!("../icons/tray-off-16.png");
const OFF_20: &[u8] = include_bytes!("../icons/tray-off-20.png");
const OFF_32: &[u8] = include_bytes!("../icons/tray-off-32.png");

/// Physical tray size for the primary monitor's scale factor.
fn tray_px(app: &AppHandle) -> u32 {
    let scale = app
        .primary_monitor()
        .ok()
        .flatten()
        .map(|monitor| monitor.scale_factor())
        .unwrap_or(1.0);
    if scale < 1.25 {
        16
    } else if scale < 1.75 {
        20
    } else {
        32
    }
}

fn icon_for(app: &AppHandle, status: &StatusView) -> tauri::Result<tauri::image::Image<'static>> {
    let bytes: &'static [u8] = match (status.enabled, tray_px(app)) {
        (true, 16) => ON_16,
        (true, 20) => ON_20,
        (false, 16) => OFF_16,
        (false, 20) => OFF_20,
        (true, _) => ON_32,
        (false, _) => OFF_32,
    };
    if update::available().is_some() {
        // Composed at runtime so the shipped PNGs stay untouched and the badge
        // always matches the size Windows asked for.
        return badged(bytes, tray_px(app));
    }
    tauri::image::Image::from_bytes(bytes)
}

// Update badge: an accent disc with a white download arrow in the bottom-right
// corner, plus a light halo so it reads on a dark taskbar too. Drawn from the
// pixels of the existing icon, not from an extra asset.
const ACCENT: [u8; 3] = [0x58, 0x44, 0xC8];
const HALO: [u8; 3] = [0xFF, 0xFF, 0xFF];

fn coverage(distance: f32, radius: f32) -> f32 {
    (radius - distance + 0.5).clamp(0.0, 1.0)
}

/// Source-over one pixel, straight alpha.
fn blend(pixels: &mut [u8], index: usize, color: [u8; 3], alpha: f32) {
    let alpha = alpha.clamp(0.0, 1.0);
    if alpha <= 0.0 {
        return;
    }
    let dest = pixels[index + 3] as f32 / 255.0;
    let out = alpha + dest * (1.0 - alpha);
    if out <= 0.0 {
        return;
    }
    for channel in 0..3 {
        let dst = pixels[index + channel] as f32;
        let src = color[channel] as f32;
        pixels[index + channel] = ((src * alpha + dst * dest * (1.0 - alpha)) / out).round() as u8;
    }
    pixels[index + 3] = (out * 255.0).round() as u8;
}

fn badged(base: &'static [u8], px: u32) -> tauri::Result<tauri::image::Image<'static>> {
    let image = tauri::image::Image::from_bytes(base)?;
    let (width, height) = (image.width(), image.height());
    let mut pixels = image.rgba().to_vec();

    // 8.6 px across at 16 px: visible in the notification area without hiding
    // the keycap.
    let radius = (px as f32 * 0.27).max(3.5);
    let cx = width as f32 - radius - 0.4;
    let cy = height as f32 - radius - 0.4;

    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 + 0.5 - cx;
            let dy = y as f32 + 0.5 - cy;
            let distance = (dx * dx + dy * dy).sqrt();
            let index = ((y * width + x) * 4) as usize;

            let halo = coverage(distance, radius + 1.0);
            if halo > 0.0 {
                blend(&mut pixels, index, HALO, halo * 0.9);
            }
            let disc = coverage(distance, radius);
            if disc <= 0.0 {
                continue;
            }
            blend(&mut pixels, index, ACCENT, disc);

            let ax = dx.abs();
            let stem = ax <= 0.7 && dy >= -radius * 0.45 && dy <= radius * 0.15;
            let head =
                dy > radius * 0.15 && dy <= radius * 0.52 && ax <= (radius * 0.52 - dy) * 1.1 + 0.5;
            if stem || head {
                blend(&mut pixels, index, HALO, disc);
            }
        }
    }

    Ok(tauri::image::Image::new_owned(pixels, width, height))
}

fn tooltip(status: &StatusView) -> String {
    let base = if status.enabled {
        format!("VietKB · Tiếng Việt · {}", method_label(&status.method))
    } else {
        "VietKB · Đã tắt tiếng Việt".to_string()
    };
    match update::available_version() {
        Some(version) => format!("{base} · Có bản cập nhật v{version}"),
        None => base,
    }
}

fn method_label(slug: &str) -> &str {
    METHODS
        .iter()
        .find(|(value, _)| *value == slug)
        .map(|(_, label)| *label)
        .unwrap_or("Telex")
}

/// Exact label of the tray's update item, so the string is testable.
fn update_menu_label(version: &str) -> String {
    format!("Có bản cập nhật v{version}")
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

    let settings = MenuItem::with_id(app, "settings", "Mở cài đặt…", true, None::<&str>)?;
    let hotkeys = MenuItem::with_id(app, "hotkeys", "Phím tắt…", true, None::<&str>)?;
    let about = MenuItem::with_id(app, "about", "Giới thiệu VietKB…", true, None::<&str>)?;
    // Only while a newer release is known: the item is the tray's third
    // affordance, and it must appear without a restart when the 5 s check finds
    // one (`refresh` rebuilds this menu).
    let update_item = match update::available_version() {
        Some(version) => {
            tracing::debug!(%version, "tray: update item added");
            Some(MenuItem::with_id(
                app,
                "update",
                update_menu_label(&version),
                true,
                None::<&str>,
            )?)
        }
        None => None,
    };
    let quit = MenuItem::with_id(app, "quit", "Thoát VietKB", true, None::<&str>)?;
    let sep_top = PredefinedMenuItem::separator(app)?;
    let sep_open = PredefinedMenuItem::separator(app)?;
    let sep_bottom = PredefinedMenuItem::separator(app)?;

    let mut items: Vec<&dyn IsMenuItem<Wry>> = vec![
        &toggle,
        &sep_top,
        &method_menu,
        &sep_open,
        &settings,
        &hotkeys,
        &about,
    ];
    if let Some(item) = &update_item {
        items.push(item);
    }
    items.push(&sep_bottom);
    items.push(&quit);
    Menu::with_items(app, &items)
}

/// Create the tray icon with the current state. Called once during setup.
pub fn install(app: &AppHandle) -> tauri::Result<()> {
    let status = commands::status_now();
    let menu = build_menu(app, &status)?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon_for(app, &status)?)
        .tooltip(tooltip(&status))
        .menu(&menu)
        // Left click belongs to the toggle below, so the menu is right-click
        // only (Tauri's default on Windows).
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| handle_event(app, event.id.as_ref()))
        .on_tray_icon_event(|_tray, event| {
            // `Up`, not `Down`: one toggle per physical click.
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                toggle_from_left_click();
            }
        })
        .build(app)?;

    Ok(())
}

/// Swallow the second half of a double-click. Windows reports a double-click on
/// the tray as extra click events; without this guard one double-click would
/// flip the state twice and land back where it started.
const CLICK_GUARD: Duration = Duration::from_millis(350);

fn toggle_from_left_click() {
    static LAST_CLICK: Mutex<Option<Instant>> = Mutex::new(None);

    let now = Instant::now();
    let Ok(mut last) = LAST_CLICK.lock() else {
        return;
    };
    if last.is_some_and(|previous| now.duration_since(previous) < CLICK_GUARD) {
        return;
    }
    *last = Some(now);
    drop(last);

    // Same path as the menu item: persist, set `enabled` on the hook, patch the
    // tray icon and tooltip, broadcast to the open windows.
    let _ = commands::toggle_enabled();
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
    tray.set_icon(Some(icon_for(app, status)?))?;
    tray.set_tooltip(Some(tooltip(status)))?;
    Ok(())
}

fn handle_event(app: &AppHandle, id: &str) {
    match id {
        "quit" => crate::quit(app),
        "settings" => crate::show_settings(app),
        "update" => crate::show_about(app),
        "about" => crate::show_about(app),
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
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    /// The tray's third affordance names the exact version, so the string is
    /// asserted here rather than discovered by reading the menu.
    #[test]
    fn update_item_label_names_the_version() {
        assert_eq!(update_menu_label("0.9.9"), "Có bản cập nhật v0.9.9");
    }

    /// Every window the app opens must be in the capability file, or its close
    /// button and IPC are refused at runtime.
    #[test]
    fn every_window_has_a_capability() {
        let capability = include_str!("../capabilities/default.json");
        let line = capability
            .lines()
            .find(|line| line.trim_start().starts_with("\"windows\""))
            .expect("capabilities/default.json has no \"windows\" array");
        let start = line.find('[').expect("no [ in the windows line");
        let end = line.find(']').expect("no ] in the windows line");
        let mut labels: Vec<&str> = line[start + 1..end]
            .split(',')
            .map(|label| label.trim().trim_matches('"'))
            .filter(|label| !label.is_empty())
            .collect();
        labels.sort_unstable();
        assert_eq!(labels, ["about", "advanced", "hotkeys", "macros", "main"]);
    }
}
