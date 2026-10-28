//! Tauri command surface. Names and payloads are frozen (contract §6).
//! The UI never calls anything else.
//!
//! Everything here persists through `vkb_config`, which owns the file format.
//! Commands are synchronous on purpose: the config is a small TOML file and a
//! 150 ms debounce on the UI side already coalesces typing bursts.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};

use serde::{Deserialize, Serialize};
use tauri::Emitter;
use vkb_config::{AppConfig, Hotkey};
use vkb_core::{Charset, InputMethod};

use crate::charset_convert;

/// The one config cell. It is an `Arc` so the very same cell can be handed to
/// the Windows backend (`vkb_platform::SharedConfig`), which is what makes a
/// config change reach the hook without a restart.
static CONFIG: OnceLock<vkb_platform::SharedConfig> = OnceLock::new();
static APP_HANDLE: OnceLock<tauri::AppHandle> = OnceLock::new();

static AUTOSTART_SYNCED: AtomicBool = AtomicBool::new(false);
static AUTOSTART_STATE: AtomicBool = AtomicBool::new(false);

fn shared() -> &'static vkb_platform::SharedConfig {
    CONFIG.get_or_init(|| Arc::new(parking_lot::RwLock::new(vkb_config::load())))
}

/// The config cell as the backend expects it; cloning the `Arc` is the whole
/// hand-off.
pub fn shared_config() -> vkb_platform::SharedConfig {
    shared().clone()
}

pub fn config_cell() -> &'static parking_lot::RwLock<AppConfig> {
    shared().as_ref()
}

pub fn set_app_handle(handle: tauri::AppHandle) {
    let _ = APP_HANDLE.set(handle);
}

pub(crate) fn app_handle() -> Option<&'static tauri::AppHandle> {
    APP_HANDLE.get()
}

/// Whether the keyboard hook is installed and pumping. Never a constant: it is
/// the backend's own liveness flag, so a hook that died reads `false`.
pub fn hook_ready_now() -> bool {
    #[cfg(windows)]
    {
        vkb_windows::is_hook_ready()
    }
    #[cfg(not(windows))]
    {
        false
    }
}

// ---------------------------------------------------------------------------
// Views
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsView {
    pub input: InputView,
    pub typing: TypingView,
    pub hotkeys: HotkeyView,
    pub system: SystemView,
    pub ui: UiView,
    pub apps: AppsView,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputView {
    pub enabled: bool,
    pub method: String,
    pub charset: String,
    pub mechanism: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypingView {
    pub modern_tone: bool,
    pub free_tone: bool,
    pub jfwz_as_consonant: bool,
    pub auto_capitalize: bool,
    pub spell_check: bool,
    pub restore_wrong_word: bool,
    pub disable_on_non_us_layout: bool,
    pub suggestion_fix_1: bool,
    pub suggestion_fix_2: bool,
    pub metro_fix: bool,
    pub send_standard: bool,
    pub use_clipboard: bool,
    pub macro_enabled: bool,
    pub macro_when_disabled: bool,
    pub auto_horn_uo: bool,
    pub raw_zone: bool,
    pub macro_case_follow: bool,
    pub send_delay_us: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyView {
    pub enable: Option<String>,
    pub disable: Option<String>,
    pub toggle: Option<String>,
    pub restore_word: Option<String>,
    pub app_exclude: Option<String>,
    pub convert_tool: Option<String>,
    pub macro_panel: Option<String>,
    pub macro_toggle: Option<String>,
    pub open_settings: Option<String>,
    pub cycle_charset: Option<String>,
    pub cycle_method: Option<String>,
    pub toggle_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemView {
    pub start_with_windows: bool,
    pub run_as_admin: bool,
    pub notification_sound: bool,
    pub start_minimized: bool,
    pub language: String,
    pub theme: String,
    pub check_updates: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiView {
    pub window_width: u32,
    pub window_height: u32,
    pub last_tab: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppsView {
    /// "Loại trừ ứng dụng tự động"
    pub remember_per_app: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusView {
    pub enabled: bool,
    pub method: String,
    pub charset: String,
    pub mechanism: String,
    pub hook_ready: bool,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MacroRow {
    pub trigger: String,
    pub expansion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppRuleRow {
    pub process: String,
    pub enabled: Option<bool>,
    pub charset: Option<String>,
    pub method: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunningApp {
    pub process: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfoView {
    pub version: String,
    pub arch: String,
    pub config_path: String,
    pub hook_ready: bool,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

pub fn to_view(cfg: &AppConfig) -> SettingsView {
    SettingsView {
        input: InputView {
            enabled: cfg.input.enabled,
            method: cfg.input.method.as_str().to_string(),
            charset: cfg.input.charset.as_str().to_string(),
            mechanism: match cfg.input.mechanism {
                vkb_config::Mechanism::Hook => "hook".into(),
                vkb_config::Mechanism::Ime => "ime".into(),
            },
        },
        typing: TypingView {
            modern_tone: cfg.typing.modern_tone,
            free_tone: cfg.typing.free_tone,
            jfwz_as_consonant: cfg.typing.jfwz_as_consonant,
            auto_capitalize: cfg.typing.auto_capitalize,
            spell_check: cfg.typing.spell_check,
            restore_wrong_word: cfg.typing.restore_wrong_word,
            disable_on_non_us_layout: cfg.typing.disable_on_non_us_layout,
            suggestion_fix_1: cfg.typing.suggestion_fix_1,
            suggestion_fix_2: cfg.typing.suggestion_fix_2,
            metro_fix: cfg.typing.metro_fix,
            send_standard: cfg.typing.send_standard,
            use_clipboard: cfg.typing.use_clipboard,
            macro_enabled: cfg.typing.macro_enabled,
            macro_when_disabled: cfg.typing.macro_when_disabled,
            auto_horn_uo: cfg.typing.auto_horn_uo,
            raw_zone: cfg.typing.raw_zone,
            macro_case_follow: cfg.typing.macro_case_follow,
            send_delay_us: cfg.typing.send_delay_us,
        },
        hotkeys: HotkeyView {
            enable: cfg.hotkeys.enable.as_ref().map(Hotkey::as_str),
            disable: cfg.hotkeys.disable.as_ref().map(Hotkey::as_str),
            toggle: cfg.hotkeys.toggle.as_ref().map(Hotkey::as_str),
            restore_word: cfg.hotkeys.restore_word.as_ref().map(Hotkey::as_str),
            app_exclude: cfg.hotkeys.app_exclude.as_ref().map(Hotkey::as_str),
            convert_tool: cfg.hotkeys.convert_tool.as_ref().map(Hotkey::as_str),
            macro_panel: cfg.hotkeys.macro_panel.as_ref().map(Hotkey::as_str),
            macro_toggle: cfg.hotkeys.macro_toggle.as_ref().map(Hotkey::as_str),
            open_settings: cfg.hotkeys.open_settings.as_ref().map(Hotkey::as_str),
            cycle_charset: cfg.hotkeys.cycle_charset.as_ref().map(Hotkey::as_str),
            cycle_method: cfg.hotkeys.cycle_method.as_ref().map(Hotkey::as_str),
            toggle_mode: match cfg.hotkeys.toggle_mode {
                vkb_config::ToggleMode::Release => "release".into(),
                vkb_config::ToggleMode::Press => "press".into(),
            },
        },
        system: SystemView {
            start_with_windows: cfg.system.start_with_windows,
            run_as_admin: cfg.system.run_as_admin,
            notification_sound: cfg.system.notification_sound,
            start_minimized: cfg.system.start_minimized,
            language: match cfg.system.language {
                vkb_config::Language::Vi => "vi".into(),
                vkb_config::Language::En => "en".into(),
            },
            theme: match cfg.system.theme {
                vkb_config::Theme::System => "system".into(),
                vkb_config::Theme::Light => "light".into(),
                vkb_config::Theme::Dark => "dark".into(),
            },
            check_updates: cfg.system.check_updates,
        },
        ui: UiView {
            window_width: cfg.ui.window_width,
            window_height: cfg.ui.window_height,
            last_tab: cfg.ui.last_tab.clone(),
        },
        apps: AppsView {
            remember_per_app: cfg.apps.remember_per_app,
        },
    }
}

pub fn status_view(cfg: &AppConfig) -> StatusView {
    StatusView {
        enabled: cfg.input.enabled,
        method: cfg.input.method.as_str().into(),
        charset: cfg.input.charset.as_str().into(),
        mechanism: match cfg.input.mechanism {
            vkb_config::Mechanism::Hook => "hook".into(),
            vkb_config::Mechanism::Ime => "ime".into(),
        },
        hook_ready: hook_ready_now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

pub fn status_now() -> StatusView {
    status_view(&config_cell().read())
}

/// Persist, refresh the tray, broadcast to open windows and keep autostart in
/// sync with the config.
fn after_change() {
    let (status, autostart) = {
        let cfg = config_cell().read();
        (status_view(&cfg), cfg.system.start_with_windows)
    };
    sync_autostart(autostart);

    // Hand the new config to the hook: one `Arc` swap plus a wake-up, never a
    // reinstall, and the enabled flag rides along inside the config.
    #[cfg(windows)]
    crate::windows_backend::push_config();

    if let Some(app) = app_handle() {
        crate::tray::refresh(app, &status);
        if let Err(error) = app.emit("vkb:status", status) {
            tracing::warn!(%error, "could not broadcast status");
        }
    }
}

fn sync_autostart(enabled: bool) {    use tauri_plugin_autostart::ManagerExt;

    if AUTOSTART_SYNCED.load(Ordering::Relaxed) && AUTOSTART_STATE.load(Ordering::Relaxed) == enabled
    {
        return;
    }
    let Some(app) = app_handle() else {
        return;
    };
    let manager = app.autolaunch();
    let result = if enabled { manager.enable() } else { manager.disable() };
    match result {
        Ok(()) => {
            AUTOSTART_SYNCED.store(true, Ordering::Relaxed);
            AUTOSTART_STATE.store(enabled, Ordering::Relaxed);
        }
        Err(error) => tracing::warn!(%error, enabled, "autostart update failed"),
    }
}

/// Public entry point for `main`'s setup: applies the saved autostart value.
pub fn sync_autostart_now(enabled: bool) {
    sync_autostart(enabled);
}

pub(crate) fn save_and_publish(next: AppConfig) -> Result<SettingsView, String> {
    vkb_config::save(&next).map_err(|error| error.to_string())?;
    let view = {
        let mut cfg = config_cell().write();
        *cfg = next;
        to_view(&cfg)
    };
    after_change();
    Ok(view)
}

// ---------------------------------------------------------------------------
// Patch parsing (camelCase, one section at a time)
// ---------------------------------------------------------------------------

fn object<'a>(
    value: &'a serde_json::Value,
    section: &str,
) -> Result<&'a serde_json::Map<String, serde_json::Value>, String> {
    value
        .as_object()
        .ok_or_else(|| format!("{section} phải là một object JSON"))
}

fn as_bool(value: &serde_json::Value, field: &str) -> Result<bool, String> {
    value
        .as_bool()
        .ok_or_else(|| format!("{field} phải là true hoặc false"))
}

fn as_str<'a>(value: &'a serde_json::Value, field: &str) -> Result<&'a str, String> {
    value
        .as_str()
        .ok_or_else(|| format!("{field} phải là chuỗi"))
}

fn patch_input(cfg: &mut AppConfig, value: &serde_json::Value) -> Result<(), String> {
    for (key, item) in object(value, "input")? {
        match key.as_str() {
            "enabled" => cfg.input.enabled = as_bool(item, "input.enabled")?,
            "method" => {
                cfg.input.method = InputMethod::from_str(as_str(item, "input.method")?)
                    .ok_or("kiểu gõ không hợp lệ")?
            }
            "charset" => {
                cfg.input.charset = Charset::from_str(as_str(item, "input.charset")?)
                    .ok_or("bảng mã không hợp lệ")?
            }
            "mechanism" => {
                cfg.input.mechanism = match as_str(item, "input.mechanism")? {
                    "hook" => vkb_config::Mechanism::Hook,
                    "ime" => vkb_config::Mechanism::Ime,
                    other => return Err(format!("cơ chế không hợp lệ: {other}")),
                }
            }
            other => return Err(format!("input.{other} không hợp lệ")),
        }
    }
    Ok(())
}

fn patch_typing(cfg: &mut AppConfig, value: &serde_json::Value) -> Result<(), String> {
    for (key, item) in object(value, "typing")? {
        if key == "sendDelayUs" {
            let delay = item
                .as_u64()
                .ok_or("typing.sendDelayUs phải là số")?
                .min(5000);
            cfg.typing.send_delay_us = delay as u32;
            continue;
        }
        let flag = as_bool(item, &format!("typing.{key}"))?;
        match key.as_str() {
            "modernTone" => cfg.typing.modern_tone = flag,
            "freeTone" => cfg.typing.free_tone = flag,
            "jfwzAsConsonant" => cfg.typing.jfwz_as_consonant = flag,
            "autoCapitalize" => cfg.typing.auto_capitalize = flag,
            "spellCheck" => cfg.typing.spell_check = flag,
            "restoreWrongWord" => cfg.typing.restore_wrong_word = flag,
            "disableOnNonUsLayout" => cfg.typing.disable_on_non_us_layout = flag,
            "suggestionFix1" => cfg.typing.suggestion_fix_1 = flag,
            "suggestionFix2" => cfg.typing.suggestion_fix_2 = flag,
            "metroFix" => cfg.typing.metro_fix = flag,
            "sendStandard" => cfg.typing.send_standard = flag,
            "useClipboard" => cfg.typing.use_clipboard = flag,
            "macroEnabled" => cfg.typing.macro_enabled = flag,
            "macroWhenDisabled" => cfg.typing.macro_when_disabled = flag,
            "autoHornUo" => cfg.typing.auto_horn_uo = flag,
            "rawZone" => cfg.typing.raw_zone = flag,
            "macroCaseFollow" => cfg.typing.macro_case_follow = flag,
            other => return Err(format!("typing.{other} không hợp lệ")),
        }
    }
    Ok(())
}

fn patch_hotkey(value: &serde_json::Value, field: &str) -> Result<Option<Hotkey>, String> {
    if value.is_null() {
        return Ok(None);
    }
    let raw = as_str(value, field)?.trim();
    if raw.is_empty() {
        return Ok(None);
    }
    Hotkey::parse(raw)
        .map(Some)
        .map_err(|error| format!("{field}: {error}"))
}

fn patch_hotkeys(cfg: &mut AppConfig, value: &serde_json::Value) -> Result<(), String> {
    for (key, item) in object(value, "hotkeys")? {
        match key.as_str() {
            "enable" => cfg.hotkeys.enable = patch_hotkey(item, "hotkeys.enable")?,
            "disable" => cfg.hotkeys.disable = patch_hotkey(item, "hotkeys.disable")?,
            "toggle" => cfg.hotkeys.toggle = patch_hotkey(item, "hotkeys.toggle")?,
            "restoreWord" => cfg.hotkeys.restore_word = patch_hotkey(item, "hotkeys.restoreWord")?,
            "appExclude" => cfg.hotkeys.app_exclude = patch_hotkey(item, "hotkeys.appExclude")?,
            "convertTool" => cfg.hotkeys.convert_tool = patch_hotkey(item, "hotkeys.convertTool")?,
            "macroPanel" => cfg.hotkeys.macro_panel = patch_hotkey(item, "hotkeys.macroPanel")?,
            "macroToggle" => cfg.hotkeys.macro_toggle = patch_hotkey(item, "hotkeys.macroToggle")?,
            "openSettings" => {
                cfg.hotkeys.open_settings = patch_hotkey(item, "hotkeys.openSettings")?
            }
            "cycleCharset" => {
                cfg.hotkeys.cycle_charset = patch_hotkey(item, "hotkeys.cycleCharset")?
            }
            "cycleMethod" => {
                cfg.hotkeys.cycle_method = patch_hotkey(item, "hotkeys.cycleMethod")?
            }
            "toggleMode" => {
                cfg.hotkeys.toggle_mode = match as_str(item, "hotkeys.toggleMode")? {
                    "release" => vkb_config::ToggleMode::Release,
                    "press" => vkb_config::ToggleMode::Press,
                    other => return Err(format!("kiểu chuyển không hợp lệ: {other}")),
                }
            }
            other => return Err(format!("hotkeys.{other} không hợp lệ")),
        }
    }
    Ok(())
}

fn patch_system(cfg: &mut AppConfig, value: &serde_json::Value) -> Result<(), String> {
    for (key, item) in object(value, "system")? {
        match key.as_str() {
            "startWithWindows" => cfg.system.start_with_windows = as_bool(item, "system.startWithWindows")?,
            "runAsAdmin" => cfg.system.run_as_admin = as_bool(item, "system.runAsAdmin")?,
            "notificationSound" => {
                cfg.system.notification_sound = as_bool(item, "system.notificationSound")?
            }
            "startMinimized" => cfg.system.start_minimized = as_bool(item, "system.startMinimized")?,
            "language" => {
                cfg.system.language = match as_str(item, "system.language")? {
                    "vi" => vkb_config::Language::Vi,
                    "en" => vkb_config::Language::En,
                    other => return Err(format!("ngôn ngữ không hợp lệ: {other}")),
                }
            }
            "theme" => {
                cfg.system.theme = match as_str(item, "system.theme")? {
                    "system" => vkb_config::Theme::System,
                    "light" => vkb_config::Theme::Light,
                    "dark" => vkb_config::Theme::Dark,
                    other => return Err(format!("giao diện không hợp lệ: {other}")),
                }
            }
            "checkUpdates" => cfg.system.check_updates = as_bool(item, "system.checkUpdates")?,
            other => return Err(format!("system.{other} không hợp lệ")),
        }
    }
    Ok(())
}

/// Charter C9: four tabs, no dashboard.
const TABS: [&str; 4] = ["go", "macros", "hotkeys", "system"];

fn legacy_tab(tab: &str) -> Option<&'static str> {
    match tab {
        "overview" | "typing" | "charset" => Some("go"),
        "apps" | "about" => Some("system"),
        _ => None,
    }
}

fn patch_ui(cfg: &mut AppConfig, value: &serde_json::Value) -> Result<(), String> {
    for (key, item) in object(value, "ui")? {
        match key.as_str() {
            "lastTab" => {
                let tab = as_str(item, "ui.lastTab")?;
                if let Some(mapped) = legacy_tab(tab) {
                    cfg.ui.last_tab = mapped.to_string();
                } else if TABS.contains(&tab) {
                    cfg.ui.last_tab = tab.to_string();
                } else {
                    return Err(format!("màn hình không hợp lệ: {tab}"));
                }
            }
            "windowWidth" => {
                let width = item.as_u64().ok_or("ui.windowWidth phải là số")?;
                cfg.ui.window_width = width.clamp(720, 4096) as u32;
            }
            "windowHeight" => {
                let height = item.as_u64().ok_or("ui.windowHeight phải là số")?;
                cfg.ui.window_height = height.clamp(480, 4096) as u32;
            }
            other => return Err(format!("ui.{other} không hợp lệ")),
        }
    }
    Ok(())
}

fn patch_apps(cfg: &mut AppConfig, value: &serde_json::Value) -> Result<(), String> {
    for (key, item) in object(value, "apps")? {
        match key.as_str() {
            "rememberPerApp" => {
                cfg.apps.remember_per_app = as_bool(item, "apps.rememberPerApp")?
            }
            other => return Err(format!("apps.{other} không hợp lệ")),
        }
    }
    Ok(())
}

fn apply_patch(cfg: &mut AppConfig, patch: &serde_json::Value) -> Result<(), String> {
    for (key, value) in object(patch, "patch")? {
        match key.as_str() {
            "input" => patch_input(cfg, value)?,
            "typing" => patch_typing(cfg, value)?,
            "hotkeys" => patch_hotkeys(cfg, value)?,
            "system" => patch_system(cfg, value)?,
            "ui" => patch_ui(cfg, value)?,
            "apps" => patch_apps(cfg, value)?,
            other => return Err(format!("mục cấu hình không hợp lệ: {other}")),
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn get_settings() -> SettingsView {
    to_view(&config_cell().read())
}

#[tauri::command]
pub fn set_settings(next: SettingsView) -> Result<SettingsView, String> {
    let patch = serde_json::to_value(&next).map_err(|error| error.to_string())?;
    patch_settings(patch)
}

#[tauri::command]
pub fn patch_settings(patch: serde_json::Value) -> Result<SettingsView, String> {
    let mut draft = config_cell().read().clone();
    apply_patch(&mut draft, &patch)?;
    save_and_publish(draft)
}

#[tauri::command]
pub fn reset_settings(section: String) -> Result<SettingsView, String> {
    let defaults = AppConfig::default();
    let mut draft = config_cell().read().clone();
    match section.as_str() {
        "input" => draft.input = defaults.input,
        "typing" => draft.typing = defaults.typing,
        "hotkeys" => draft.hotkeys = defaults.hotkeys,
        "system" => draft.system = defaults.system,
        "ui" => draft.ui = defaults.ui,
        "apps" => draft.apps = defaults.apps,
        // "all" resets settings, never the user's macros: those are data, not
        // preferences, and losing 5000 of them to one click would be a defect.
        "all" => {
            draft.input = defaults.input;
            draft.typing = defaults.typing;
            draft.hotkeys = defaults.hotkeys;
            draft.system = defaults.system;
            draft.ui = defaults.ui;
            draft.apps = defaults.apps;
        }
        other => return Err(format!("mục không hợp lệ: {other}")),
    }
    save_and_publish(draft)
}

#[tauri::command]
pub fn get_status() -> StatusView {
    status_now()
}

#[tauri::command]
pub fn toggle_enabled() -> StatusView {
    let mut draft = config_cell().read().clone();
    draft.input.enabled = !draft.input.enabled;
    let play_sound = draft.system.notification_sound;
    let enabled = draft.input.enabled;
    match save_and_publish(draft) {
        Ok(view) => {
            // `after_change` already pushed the config; the flag itself is set
            // here too so the very next keystroke sees it, not the next tick.
            #[cfg(windows)]
            vkb_windows::set_enabled(enabled);
            if play_sound {
                play_notification();
            }
            status_view_from_view(&view)
        }
        Err(error) => {
            tracing::warn!(%error, "could not toggle");
            status_now()
        }
    }
}

fn play_notification() {
    #[cfg(windows)]
    vkb_windows::play_notification_sound();
}

fn status_view_from_view(view: &SettingsView) -> StatusView {
    StatusView {
        enabled: view.input.enabled,
        method: view.input.method.clone(),
        charset: view.input.charset.clone(),
        mechanism: view.input.mechanism.clone(),
        hook_ready: hook_ready_now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

#[tauri::command]
pub fn set_method(method: String) -> Result<StatusView, String> {
    let parsed = InputMethod::from_str(&method).ok_or("kiểu gõ không hợp lệ")?;
    let mut draft = config_cell().read().clone();
    draft.input.method = parsed;
    let view = save_and_publish(draft)?;
    Ok(status_view_from_view(&view))
}

#[tauri::command]
pub fn set_charset(charset: String) -> Result<StatusView, String> {
    let parsed = Charset::from_str(&charset).ok_or("bảng mã không hợp lệ")?;
    let mut draft = config_cell().read().clone();
    draft.input.charset = parsed;
    let view = save_and_publish(draft)?;
    Ok(status_view_from_view(&view))
}

#[tauri::command]
pub fn list_macros(query: String) -> Vec<MacroRow> {
    let cfg = config_cell().read();
    let needle = query.trim().to_lowercase();
    cfg.macros
        .iter()
        .filter(|(trigger, expansion)| {
            needle.is_empty()
                || trigger.to_lowercase().contains(&needle)
                || expansion.to_lowercase().contains(&needle)
        })
        .map(|(trigger, expansion)| MacroRow {
            trigger: trigger.clone(),
            expansion: expansion.clone(),
        })
        .collect()
}

fn validate_macro(trigger: &str, expansion: &str) -> Result<(String, String), String> {
    let trigger = trigger.trim();
    let expansion = expansion.trim();
    let trigger_len = trigger.chars().count();
    let expansion_len = expansion.chars().count();
    if trigger.is_empty() {
        return Err("Nhập từ tắt, ví dụ dc".into());
    }
    if trigger.chars().any(char::is_whitespace) {
        return Err("Từ tắt viết liền, không có dấu cách".into());
    }
    if trigger_len > 32 {
        return Err(format!(
            "Từ tắt dài tối đa 32 ký tự, đang có {trigger_len}"
        ));
    }
    if expansion.is_empty() {
        return Err("Phần thay thế không được để trống — muốn bỏ thì xoá cả mục.".into());
    }
    if expansion_len > 500 {
        return Err(format!(
            "Nội dung dài tối đa 500 ký tự, đang có {expansion_len}"
        ));
    }
    Ok((trigger.to_string(), expansion.to_string()))
}

#[tauri::command]
pub fn upsert_macro(trigger: String, expansion: String) -> Result<(), String> {
    let (trigger, expansion) = validate_macro(&trigger, &expansion)?;
    let mut draft = config_cell().read().clone();
    draft.macros.insert(trigger, expansion);
    save_and_publish(draft).map(|_| ())
}

#[tauri::command]
pub fn delete_macros(triggers: Vec<String>) -> Result<(), String> {
    let mut draft = config_cell().read().clone();
    for trigger in triggers {
        draft.macros.remove(&trigger);
    }
    save_and_publish(draft).map(|_| ())
}

const MACRO_MAX_ROWS: usize = 5000;
const MACRO_MAX_BYTES: usize = 2_000_000;

fn escape_tsv(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\t', "\\t")
        .replace('\n', "\\n")
        .replace('\r', "")
}

fn unescape_tsv(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('\\') => out.push('\\'),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

/// Import a TSV macro list (27 C#8): UTF-8, no BOM, `trigger<TAB>expansion`,
/// optional `#vkb1` header, `\n`/`\t` escapes. All-or-nothing: the first bad
/// line rejects the whole file and nothing is written.
#[tauri::command]
pub fn import_macros(path: String) -> Result<usize, String> {
    let display = std::path::Path::new(&path)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| path.clone());
    let bytes = std::fs::read(&path).map_err(|error| error.to_string())?;
    if bytes.len() > MACRO_MAX_BYTES {
        return Err(format!("Không đọc được tệp {display}: tệp lớn hơn 2 MB."));
    }
    let body = String::from_utf8(bytes).map_err(|_| {
        format!("Không đọc được tệp {display}: tệp không phải UTF-8.")
    })?;
    let body = body.trim_start_matches('\u{feff}');

    let mut parsed: Vec<(String, String)> = Vec::new();
    for (index, raw) in body.lines().enumerate() {
        let line_no = index + 1;
        let line = raw.trim_end_matches('\r');
        if line.trim().is_empty() || (index == 0 && line.trim() == "#vkb1") {
            continue;
        }
        if line.starts_with('#') {
            continue;
        }
        let Some((trigger, expansion)) = line.split_once('\t') else {
            return Err(format!(
                "Không đọc được tệp {display}: dòng {line_no} không đúng dạng. Chưa nhập mục nào."
            ));
        };
        match validate_macro(&unescape_tsv(trigger), &unescape_tsv(expansion)) {
            Ok(row) => parsed.push(row),
            Err(_) => {
                return Err(format!(
                    "Không đọc được tệp {display}: dòng {line_no} không đúng dạng. Chưa nhập mục nào."
                ))
            }
        }
        if parsed.len() > MACRO_MAX_ROWS {
            return Err(format!(
                "Không đọc được tệp {display}: quá 5.000 mục. Chưa nhập mục nào."
            ));
        }
    }

    if parsed.is_empty() {
        return Err(format!(
            "Không đọc được tệp {display}: không có dòng nào. Chưa nhập mục nào."
        ));
    }

    let mut draft = config_cell().read().clone();
    for (trigger, expansion) in &parsed {
        draft.macros.insert(trigger.clone(), expansion.clone());
    }
    save_and_publish(draft)?;
    Ok(parsed.len())
}

#[tauri::command]
pub fn export_macros(path: String) -> Result<usize, String> {
    let cfg = config_cell().read();
    let mut body = String::with_capacity(cfg.macros.len() * 40 + 8);
    body.push_str("#vkb1\n");
    for (trigger, expansion) in cfg.macros.iter() {
        body.push_str(&escape_tsv(trigger));
        body.push('\t');
        body.push_str(&escape_tsv(expansion));
        body.push('\n');
    }
    let target = std::path::Path::new(&path);
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    std::fs::write(target, body).map_err(|error| error.to_string())?;
    Ok(cfg.macros.len())
}

#[tauri::command]
pub fn list_app_rules() -> Vec<AppRuleRow> {
    let cfg = config_cell().read();
    cfg.apps
        .rules
        .iter()
        .map(|rule| AppRuleRow {
            process: rule.process.clone(),
            enabled: rule.enabled,
            charset: rule.charset.map(|charset| charset.as_str().to_string()),
            method: rule.method.map(|method| method.as_str().to_string()),
        })
        .collect()
}

#[tauri::command]
pub fn set_app_rule(process: String, enabled: bool) -> Result<(), String> {
    let process = process.trim().to_ascii_lowercase();
    if process.is_empty() {
        return Err("tên tiến trình không được để trống".into());
    }
    let mut draft = config_cell().read().clone();
    let exists = draft
        .apps
        .rules
        .iter()
        .any(|rule| rule.process.eq_ignore_ascii_case(&process));
    if exists {
        if let Some(rule) = draft
            .apps
            .rules
            .iter_mut()
            .find(|rule| rule.process.eq_ignore_ascii_case(&process))
        {
            rule.enabled = Some(enabled);
        }
    } else {
        let mut rule = vkb_config::AppRule::new(process);
        rule.enabled = Some(enabled);
        draft.apps.rules.push(rule);
    }
    save_and_publish(draft).map(|_| ())
}

/// Additive command (not in contract §6): the frozen `set_app_rule` can only
/// toggle `enabled`, but the screen also needs per-app charset/method overrides.
#[tauri::command]
pub fn set_app_rule_overrides(
    process: String,
    charset: Option<String>,
    method: Option<String>,
) -> Result<(), String> {
    let process = process.trim().to_string();
    let charset = match charset.as_deref().map(str::trim).filter(|value| !value.is_empty()) {
        Some(slug) => Some(Charset::from_str(slug).ok_or("bảng mã không hợp lệ")?),
        None => None,
    };
    let method = match method.as_deref().map(str::trim).filter(|value| !value.is_empty()) {
        Some(slug) => Some(InputMethod::from_str(slug).ok_or("kiểu gõ không hợp lệ")?),
        None => None,
    };

    let mut draft = config_cell().read().clone();
    if let Some(rule) = draft
        .apps
        .rules
        .iter_mut()
        .find(|rule| rule.process.eq_ignore_ascii_case(&process))
    {
        rule.charset = charset;
        rule.method = method;
    } else {
        let mut rule = vkb_config::AppRule::new(process);
        rule.enabled = Some(true);
        rule.charset = charset;
        rule.method = method;
        draft.apps.rules.push(rule);
    }
    save_and_publish(draft).map(|_| ())
}

#[tauri::command]
pub fn delete_app_rule(process: String) -> Result<(), String> {
    let mut draft = config_cell().read().clone();
    draft
        .apps
        .rules
        .retain(|rule| !rule.process.eq_ignore_ascii_case(&process));
    save_and_publish(draft).map(|_| ())
}

#[tauri::command]
pub fn list_running_apps() -> Vec<RunningApp> {
    crate::running_apps::list()
        .into_iter()
        .map(|(process, title)| RunningApp { process, title })
        .collect()
}

#[tauri::command]
pub fn parse_hotkey(input: String) -> Result<String, String> {
    Hotkey::parse(&input)
        .map(|hotkey| hotkey.as_str())
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn check_hotkey_conflict(input: String, ignore_field: String) -> Option<String> {
    let target = Hotkey::parse(&input).ok()?;
    let cfg = config_cell().read();
    let fields: [(&str, &Option<Hotkey>); 11] = [
        ("enable", &cfg.hotkeys.enable),
        ("disable", &cfg.hotkeys.disable),
        ("toggle", &cfg.hotkeys.toggle),
        ("restore_word", &cfg.hotkeys.restore_word),
        ("app_exclude", &cfg.hotkeys.app_exclude),
        ("convert_tool", &cfg.hotkeys.convert_tool),
        ("macro_panel", &cfg.hotkeys.macro_panel),
        ("macro_toggle", &cfg.hotkeys.macro_toggle),
        ("open_settings", &cfg.hotkeys.open_settings),
        ("cycle_charset", &cfg.hotkeys.cycle_charset),
        ("cycle_method", &cfg.hotkeys.cycle_method),
    ];
    fields
        .iter()
        .find(|(name, hotkey)| *name != ignore_field && hotkey.as_ref() == Some(&target))
        .map(|(name, _)| (*name).to_string())
}

#[tauri::command]
pub fn convert_text(input: String, from: String, to: String) -> Result<String, String> {
    charset_convert::convert(&input, &from, &to)
}

#[tauri::command]
pub fn open_config_folder() -> Result<(), String> {
    let dir = vkb_config::paths::config_dir();
    std::fs::create_dir_all(&dir).map_err(|error| error.to_string())?;

    #[cfg(windows)]
    {
        std::process::Command::new("explorer.exe")
            .arg(&dir)
            .spawn()
            .map(|_| ())
            .map_err(|error| format!("không mở được thư mục: {error}"))
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&dir)
            .spawn()
            .map(|_| ())
            .map_err(|error| format!("không mở được thư mục: {error}"))
    }
    #[cfg(all(not(windows), not(target_os = "macos")))]
    {
        std::process::Command::new("xdg-open")
            .arg(&dir)
            .spawn()
            .map(|_| ())
            .map_err(|error| format!("không mở được thư mục: {error}"))
    }
}

#[tauri::command]
pub fn restart_app(app: tauri::AppHandle) {
    app.restart();
}

/// Presentation-only window management: the frozen §6 commands are untouched.
/// Async so WebView2 creation is posted to the UI thread instead of blocking
/// the IPC handler that called us.
#[tauri::command]
pub async fn open_window(app: tauri::AppHandle, kind: String) -> Result<(), String> {
    crate::show_aux(&app, &kind)
}

#[tauri::command]
pub fn quit_app(app: tauri::AppHandle) {
    crate::quit(&app);
}

#[tauri::command]
pub fn app_info() -> AppInfoView {    AppInfoView {
        version: env!("CARGO_PKG_VERSION").to_string(),
        arch: std::env::consts::ARCH.to_string(),
        config_path: vkb_config::paths::config_file().display().to_string(),
        hook_ready: hook_ready_now(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn patched(patch: &str) -> AppConfig {
        let mut cfg = AppConfig::default();
        let value: serde_json::Value = serde_json::from_str(patch).unwrap();
        apply_patch(&mut cfg, &value).unwrap();
        cfg
    }

    #[test]
    fn patch_only_touches_named_fields() {
        let cfg = patched(r#"{"typing":{"modernTone":false},"input":{"method":"vni"}}"#);
        assert!(!cfg.typing.modern_tone);
        assert_eq!(cfg.input.method, InputMethod::Vni);
        assert!(cfg.typing.spell_check, "untouched fields keep their value");
    }

    #[test]
    fn patch_sets_and_clears_hotkeys() {
        let cfg = patched(r#"{"hotkeys":{"enable":"Ctrl+Shift","disable":""}}"#);
        assert_eq!(cfg.hotkeys.enable.as_ref().unwrap().as_str(), "Ctrl+Shift");
        assert!(cfg.hotkeys.disable.is_none());
    }

    #[test]
    fn patch_rejects_unknown_keys() {
        let mut cfg = AppConfig::default();
        let value = serde_json::json!({ "typing": { "nope": true } });
        assert!(apply_patch(&mut cfg, &value).is_err());
        let value = serde_json::json!({ "mystery": {} });
        assert!(apply_patch(&mut cfg, &value).is_err());
    }

    #[test]
    fn patch_rejects_a_bare_letter_hotkey() {
        let mut cfg = AppConfig::default();
        let value = serde_json::json!({ "hotkeys": { "toggle": "A" } });
        let error = apply_patch(&mut cfg, &value).unwrap_err();
        assert!(error.contains("Ctrl"), "{error}");
    }

    #[test]
    fn patch_accepts_the_settings_view_shape() {
        let view = to_view(&AppConfig::default());
        let value = serde_json::to_value(&view).unwrap();
        let mut cfg = AppConfig::default();
        apply_patch(&mut cfg, &value).unwrap();
        assert_eq!(cfg.ui.last_tab, "go");
    }

    #[test]
    fn legacy_tab_names_map_to_the_four_tab_panel() {
        let mut cfg = AppConfig::default();
        let value = serde_json::json!({ "ui": { "lastTab": "overview" } });
        apply_patch(&mut cfg, &value).unwrap();
        assert_eq!(cfg.ui.last_tab, "go");

        let value = serde_json::json!({ "ui": { "lastTab": "about" } });
        apply_patch(&mut cfg, &value).unwrap();
        assert_eq!(cfg.ui.last_tab, "system");
    }
}
