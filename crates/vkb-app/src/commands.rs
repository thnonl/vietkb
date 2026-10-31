//! Tauri command surface. Names and payloads are frozen (contract §6).
//! The UI never calls anything else.
//!
//! Everything here persists through `vkb_config`, which owns the file format.
//! Commands are synchronous on purpose: the config is a small TOML file and a
//! 150 ms debounce on the UI side already coalesces typing bursts.

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, OnceLock};

use serde::{Deserialize, Serialize};
use tauri::Emitter;
use vkb_config::{AppConfig, Hotkey};
use vkb_core::{Charset, InputMethod};

/// The one config cell. It is an `Arc` so the very same cell can be handed to
/// the Windows backend (`vkb_platform::SharedConfig`), which is what makes a
/// config change reach the hook without a restart.
static CONFIG: OnceLock<vkb_platform::SharedConfig> = OnceLock::new();
static APP_HANDLE: OnceLock<tauri::AppHandle> = OnceLock::new();
/// Serializes every read-modify-write of the config (UI patches and backend
/// hotkey writes), so neither can overwrite the other's field (S-17).
static CONFIG_WRITE: parking_lot::Mutex<()> = parking_lot::Mutex::new(());

/// The last attempt to write config.toml failed: the next `update_config` must
/// write even if it changes nothing, so the file catches up with the cell.
static CONFIG_UNSAVED: AtomicBool = AtomicBool::new(false);

static AUTOSTART_SYNCED: AtomicBool = AtomicBool::new(false);
static AUTOSTART_STATE: AtomicBool = AtomicBool::new(false);
/// Language / theme (as enum discriminants) last applied to the windows; `u8::MAX`
/// until the first change is processed.
static APPLIED_LANGUAGE: AtomicU8 = AtomicU8::new(u8::MAX);
#[cfg(target_os = "macos")]
static APPLIED_THEME: AtomicU8 = AtomicU8::new(u8::MAX);

fn shared() -> &'static vkb_platform::SharedConfig {
    CONFIG.get_or_init(|| Arc::new(parking_lot::RwLock::new(vkb_config::load())))
}

/// The config cell as the backend expects it; cloning the `Arc` is the whole
/// hand-off.
#[cfg(any(windows, target_os = "macos"))]
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
    #[cfg(target_os = "macos")]
    {
        vkb_macos::is_hook_ready()
    }
    #[cfg(not(any(windows, target_os = "macos")))]
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
    pub send_standard: bool,
    pub use_clipboard: bool,
    pub macro_enabled: bool,
    pub macro_when_disabled: bool,
    pub auto_horn_uo: bool,
    pub raw_zone: bool,
    pub macro_case_follow: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyView {
    pub enable: Option<String>,
    pub disable: Option<String>,
    pub toggle: Option<String>,
    pub restore_word: Option<String>,
    pub app_exclude: Option<String>,
    pub macro_panel: Option<String>,
    pub macro_toggle: Option<String>,
    pub open_settings: Option<String>,
    pub cycle_method: Option<String>,
    pub toggle_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemView {
    pub start_with_windows: bool,
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
    /// macOS: secure event input is making the tap pass typed keys through
    /// unconverted right now (S-18). Always `false` elsewhere.
    #[serde(default)]
    pub secure_input_blocking: bool,
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
            send_standard: cfg.typing.send_standard,
            use_clipboard: cfg.typing.use_clipboard,
            macro_enabled: cfg.typing.macro_enabled,
            macro_when_disabled: cfg.typing.macro_when_disabled,
            auto_horn_uo: cfg.typing.auto_horn_uo,
            raw_zone: cfg.typing.raw_zone,
            macro_case_follow: cfg.typing.macro_case_follow,
            },
        hotkeys: HotkeyView {
            enable: cfg.hotkeys.enable.as_ref().map(Hotkey::as_str),
            disable: cfg.hotkeys.disable.as_ref().map(Hotkey::as_str),
            toggle: cfg.hotkeys.toggle.as_ref().map(Hotkey::as_str),
            restore_word: cfg.hotkeys.restore_word.as_ref().map(Hotkey::as_str),
            app_exclude: cfg.hotkeys.app_exclude.as_ref().map(Hotkey::as_str),
            macro_panel: cfg.hotkeys.macro_panel.as_ref().map(Hotkey::as_str),
            macro_toggle: cfg.hotkeys.macro_toggle.as_ref().map(Hotkey::as_str),
            open_settings: cfg.hotkeys.open_settings.as_ref().map(Hotkey::as_str),
            cycle_method: cfg.hotkeys.cycle_method.as_ref().map(Hotkey::as_str),
            toggle_mode: match cfg.hotkeys.toggle_mode {
                vkb_config::ToggleMode::Release => "release".into(),
                vkb_config::ToggleMode::Press => "press".into(),
            },
        },
        system: SystemView {
            start_with_windows: cfg.system.start_with_windows,
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
        secure_input_blocking: secure_input_blocking_now(),
    }
}

fn secure_input_blocking_now() -> bool {
    #[cfg(target_os = "macos")]
    {
        vkb_macos::secure_input_blocking()
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

pub fn status_now() -> StatusView {
    status_view(&config_cell().read())
}

/// Persist, refresh the tray, broadcast to open windows and keep autostart in
/// sync with the config.
fn after_change() {
    let (status, autostart, language, theme) = {
        let cfg = config_cell().read();
        (
            status_view(&cfg),
            cfg.system.start_with_windows,
            cfg.system.language as u8,
            cfg.system.theme as u8,
        )
    };
    sync_autostart(autostart);
    // Window titles and the native appearance only matter when they changed.
    let language_changed = APPLIED_LANGUAGE.swap(language, Ordering::Relaxed) != language;
    #[cfg(target_os = "macos")]
    let theme_changed = APPLIED_THEME.swap(theme, Ordering::Relaxed) != theme;
    #[cfg(not(target_os = "macos"))]
    let _ = theme;

    // Hand the new config to the hook: one `Arc` swap plus a wake-up, never a
    // reinstall, and the enabled flag rides along inside the config.
    #[cfg(windows)]
    crate::windows_backend::push_config();
    #[cfg(target_os = "macos")]
    crate::macos_backend::push_config();

    if let Some(app) = app_handle() {
        // The tray menu and native window titles follow `system.language`.
        if language_changed {
            crate::i18n::retitle_windows(app);
        }
        // macOS: the native title bar and menus follow the chosen appearance.
        #[cfg(target_os = "macos")]
        if theme_changed {
            crate::macos_shell::apply_theme(app, &config_cell().read().system.theme);
        }
        crate::tray::refresh(app, &status);
        if let Err(error) = app.emit("vkb:status", status) {
            tracing::warn!(%error, "could not broadcast status");
        }
    }
}

fn sync_autostart(enabled: bool) {
    if AUTOSTART_SYNCED.load(Ordering::Relaxed) && AUTOSTART_STATE.load(Ordering::Relaxed) == enabled
    {
        return;
    }

    // Dev channel: write the `VietKB Dev` value ourselves, HKCU only, quoted
    // exe plus the flag. The plugin would use an unquoted command line and may
    // pick HKLM, so the two channels must not share that writer.
    #[cfg(any(target_os = "macos", all(windows, feature = "dev-channel")))]
    {
        match crate::autostart::set(enabled) {
            Ok(()) => {
                AUTOSTART_SYNCED.store(true, Ordering::Relaxed);
                AUTOSTART_STATE.store(enabled, Ordering::Relaxed);
            }
            Err(error) => tracing::warn!(%error, enabled, "autostart update failed"),
        }
    }

    #[cfg(not(any(target_os = "macos", all(windows, feature = "dev-channel"))))]
    {
        use tauri_plugin_autostart::ManagerExt;

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
}

/// What the `Run` key says, independent of `config.toml`. `None` when the
/// platform cannot answer, so callers fall back to the saved value.
fn autostart_registry_state() -> Option<bool> {
    #[cfg(any(windows, target_os = "macos"))]
    {
        // macOS: the LaunchAgent plist is the truth, like the `Run` key.
        Some(crate::autostart::is_enabled())
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        use tauri_plugin_autostart::ManagerExt;
        app_handle()?.autolaunch().is_enabled().ok()
    }
}

/// Make the saved config agree with the `Run` key before the first window is
/// built. The registry wins: an entry removed in Windows' own startup settings
/// must not be re-created by a stale config on the next start.
pub fn adopt_autostart_registry(fallback: bool) {
    let Some(actual) = autostart_registry_state() else {
        sync_autostart(fallback);
        return;
    };

    // Same lock as every other config write, so this read-modify-write cannot
    // interleave with a UI patch or a backend hotkey write. No `after_change`:
    // this runs before the tray and the first window exist.
    let _guard = CONFIG_WRITE.lock();
    let mut next = config_cell().read().clone();
    if next.system.start_with_windows == actual {
        return;
    }
    next.system.start_with_windows = actual;
    if let Err(error) = commit_locked(next) {
        tracing::warn!(%error, "could not persist the adopted autostart value");
        return;
    }
    tracing::info!(enabled = actual, "autostart adopted from the registry");
}

/// Registry truth for the main window's switch. Reading it on load is what
/// makes the toggle show Windows' state instead of echoing the config file.
#[tauri::command]
pub fn autostart_state() -> bool {
    autostart_registry_state().unwrap_or_else(|| config_cell().read().system.start_with_windows)
}

/// Write the `Run` entry for this channel (production `VietKB`, dev channel
/// `VietKB Dev`), read it back and persist what the registry actually holds.
/// Returns that value so the switch lands on the truth even when the write
/// fails.
#[tauri::command]
pub fn set_autostart(enabled: bool) -> Result<bool, String> {
    #[cfg(any(windows, target_os = "macos"))]
    crate::autostart::set(enabled)?;

    #[cfg(not(any(windows, target_os = "macos")))]
    {
        use tauri_plugin_autostart::ManagerExt;
        let app = app_handle().ok_or_else(|| crate::i18n::t("err.notReady"))?;
        let manager = app.autolaunch();
        let result = if enabled {
            manager.enable()
        } else {
            manager.disable()
        };
        result.map_err(|error| error.to_string())?;
    }

    let actual = autostart_registry_state().unwrap_or(enabled);
    // The key is already what we meant; keep `after_change` from rewriting it.
    AUTOSTART_SYNCED.store(true, Ordering::Relaxed);
    AUTOSTART_STATE.store(actual, Ordering::Relaxed);

    update_config(|cfg| {
        cfg.system.start_with_windows = actual;
        Ok(())
    })?;

    Ok(actual)
}

/// Persist `next` and store it in the live cell. The caller holds
/// `CONFIG_WRITE`. Nothing here may wait on the main thread: the follow-up
/// ([`after_change`], tray and window work) runs after the lock is released.
fn commit_locked(next: AppConfig) -> Result<SettingsView, String> {
    if let Err(error) = vkb_config::save(&next) {
        CONFIG_UNSAVED.store(true, Ordering::Relaxed);
        return Err(error.to_string());
    }
    CONFIG_UNSAVED.store(false, Ordering::Relaxed);
    let view = {
        let mut cfg = config_cell().write();
        // macOS: the tap's enabled flag is its own atomic, not part of the
        // config it reads. Forward it only when this save actually changes it
        // (the settings switch goes through here); a save that leaves it alone
        // must not overwrite a toggle the tap just made itself.
        #[cfg(target_os = "macos")]
        if cfg.input.enabled != next.input.enabled {
            vkb_macos::set_enabled(next.input.enabled);
        }
        *cfg = next;
        to_view(&cfg)
    };
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
        .ok_or_else(|| crate::i18n::tf("err.patch.notObject", &[("section", section)]))
}

fn as_bool(value: &serde_json::Value, field: &str) -> Result<bool, String> {
    value
        .as_bool()
        .ok_or_else(|| crate::i18n::tf("err.patch.notBool", &[("field", field)]))
}

fn as_str<'a>(value: &'a serde_json::Value, field: &str) -> Result<&'a str, String> {
    value
        .as_str()
        .ok_or_else(|| crate::i18n::tf("err.patch.notString", &[("field", field)]))
}

fn patch_input(cfg: &mut AppConfig, value: &serde_json::Value) -> Result<(), String> {
    for (key, item) in object(value, "input")? {
        match key.as_str() {
            "enabled" => cfg.input.enabled = as_bool(item, "input.enabled")?,
            "method" => {
                cfg.input.method = InputMethod::from_str(as_str(item, "input.method")?)
                    .ok_or_else(|| crate::i18n::t("err.method.invalid"))?
            }
            "charset" => {
                cfg.input.charset = Charset::from_str(as_str(item, "input.charset")?)
                    .ok_or_else(|| crate::i18n::t("err.patch.charset"))?
            }
            "mechanism" => {
                cfg.input.mechanism = match as_str(item, "input.mechanism")? {
                    "hook" => vkb_config::Mechanism::Hook,
                    "ime" => vkb_config::Mechanism::Ime,
                    other => return Err(crate::i18n::tf("err.patch.mechanism", &[("value", other)])),
                }
            }
            other => return Err(crate::i18n::tf("err.patch.unknownField", &[("field", &format!("input.{other}"))])),
        }
    }
    Ok(())
}

fn patch_typing(cfg: &mut AppConfig, value: &serde_json::Value) -> Result<(), String> {
    for (key, item) in object(value, "typing")? {
        let flag = as_bool(item, &format!("typing.{key}"))?;
        match key.as_str() {
            "modernTone" => cfg.typing.modern_tone = flag,
            "freeTone" => cfg.typing.free_tone = flag,
            "jfwzAsConsonant" => cfg.typing.jfwz_as_consonant = flag,
            "autoCapitalize" => cfg.typing.auto_capitalize = flag,
            "spellCheck" => cfg.typing.spell_check = flag,
            "restoreWrongWord" => cfg.typing.restore_wrong_word = flag,
            "sendStandard" => cfg.typing.send_standard = flag,
            "useClipboard" => cfg.typing.use_clipboard = flag,
            "macroEnabled" => cfg.typing.macro_enabled = flag,
            "macroWhenDisabled" => cfg.typing.macro_when_disabled = flag,
            "autoHornUo" => cfg.typing.auto_horn_uo = flag,
            "rawZone" => cfg.typing.raw_zone = flag,
            "macroCaseFollow" => cfg.typing.macro_case_follow = flag,
            other => return Err(crate::i18n::tf("err.patch.unknownField", &[("field", &format!("typing.{other}"))])),
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
        .map_err(|error| format!("{field}: {}", hotkey_error_text(&error)))
}

/// Localized text of a hotkey parse failure (the error type's own message is
/// Vietnamese only). macOS names the modifiers as the keyboard prints them.
fn hotkey_error_text(error: &vkb_config::HotkeyError) -> String {
    use vkb_config::HotkeyError;
    match error {
        HotkeyError::Empty => crate::i18n::t("err.hotkey.empty"),
        HotkeyError::ModifierOnly(key) => {
            crate::i18n::tf("err.hotkey.modifierOnly", &[("key", key)])
        }
        HotkeyError::UnknownKey(key) => crate::i18n::tf("err.hotkey.unknownKey", &[("key", key)]),
        HotkeyError::NoModifier(key) => crate::i18n::tf(
            if cfg!(target_os = "macos") {
                "err.hotkey.noModifier.mac"
            } else {
                "err.hotkey.noModifier"
            },
            &[("key", key)],
        ),
    }
}

fn patch_hotkeys(cfg: &mut AppConfig, value: &serde_json::Value) -> Result<(), String> {
    for (key, item) in object(value, "hotkeys")? {
        match key.as_str() {
            "enable" => cfg.hotkeys.enable = patch_hotkey(item, "hotkeys.enable")?,
            "disable" => cfg.hotkeys.disable = patch_hotkey(item, "hotkeys.disable")?,
            "toggle" => cfg.hotkeys.toggle = patch_hotkey(item, "hotkeys.toggle")?,
            "restoreWord" => cfg.hotkeys.restore_word = patch_hotkey(item, "hotkeys.restoreWord")?,
            "appExclude" => cfg.hotkeys.app_exclude = patch_hotkey(item, "hotkeys.appExclude")?,
            "macroPanel" => cfg.hotkeys.macro_panel = patch_hotkey(item, "hotkeys.macroPanel")?,
            "macroToggle" => cfg.hotkeys.macro_toggle = patch_hotkey(item, "hotkeys.macroToggle")?,
            "openSettings" => {
                cfg.hotkeys.open_settings = patch_hotkey(item, "hotkeys.openSettings")?
            }
            "cycleMethod" => {
                cfg.hotkeys.cycle_method = patch_hotkey(item, "hotkeys.cycleMethod")?
            }
            "toggleMode" => {
                cfg.hotkeys.toggle_mode = match as_str(item, "hotkeys.toggleMode")? {
                    "release" => vkb_config::ToggleMode::Release,
                    "press" => vkb_config::ToggleMode::Press,
                    other => return Err(crate::i18n::tf("err.patch.toggleMode", &[("value", other)])),
                }
            }
            other => return Err(crate::i18n::tf("err.patch.unknownField", &[("field", &format!("hotkeys.{other}"))])),
        }
    }
    Ok(())
}

fn patch_system(cfg: &mut AppConfig, value: &serde_json::Value) -> Result<(), String> {
    for (key, item) in object(value, "system")? {
        match key.as_str() {
            "startWithWindows" => cfg.system.start_with_windows = as_bool(item, "system.startWithWindows")?,
            "notificationSound" => {
                cfg.system.notification_sound = as_bool(item, "system.notificationSound")?
            }
            "startMinimized" => cfg.system.start_minimized = as_bool(item, "system.startMinimized")?,
            "language" => {
                cfg.system.language = match as_str(item, "system.language")? {
                    "vi" => vkb_config::Language::Vi,
                    "en" => vkb_config::Language::En,
                    other => return Err(crate::i18n::tf("err.patch.language", &[("value", other)])),
                }
            }
            "theme" => {
                cfg.system.theme = match as_str(item, "system.theme")? {
                    "system" => vkb_config::Theme::System,
                    "light" => vkb_config::Theme::Light,
                    "dark" => vkb_config::Theme::Dark,
                    other => return Err(crate::i18n::tf("err.patch.theme", &[("value", other)])),
                }
            }
            "checkUpdates" => cfg.system.check_updates = as_bool(item, "system.checkUpdates")?,
            other => return Err(crate::i18n::tf("err.patch.unknownField", &[("field", &format!("system.{other}"))])),
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
                    return Err(crate::i18n::tf("err.patch.screen", &[("value", tab)]));
                }
            }
            "windowWidth" => {
                let width = item.as_u64().ok_or_else(|| crate::i18n::tf("err.patch.notNumber", &[("field", "ui.windowWidth")]))?;
                cfg.ui.window_width = width.clamp(720, 4096) as u32;
            }
            "windowHeight" => {
                let height = item.as_u64().ok_or_else(|| crate::i18n::tf("err.patch.notNumber", &[("field", "ui.windowHeight")]))?;
                cfg.ui.window_height = height.clamp(480, 4096) as u32;
            }
            other => return Err(crate::i18n::tf("err.patch.unknownField", &[("field", &format!("ui.{other}"))])),
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
            other => return Err(crate::i18n::tf("err.patch.unknownField", &[("field", &format!("apps.{other}"))])),
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
            other => return Err(crate::i18n::tf("err.patch.unknownSection", &[("value", other)])),
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
    update_config(|cfg| apply_patch(cfg, &patch))
}

/// Apply `mutate` to the live config as one atomic read-modify-write: nothing
/// else can change the config between the read and the save, so a UI patch and
/// a backend hotkey write each land as a field patch on the latest state.
///
/// A mutation that leaves the config unchanged is not written to disk (so the
/// "is it already that value?" test belongs inside `mutate`, not before it).
///
/// `CONFIG_WRITE` is **not re-entrant**: never call `update_config` (or anything
/// that does) from inside `mutate`. Debug builds assert on it.
pub(crate) fn update_config(
    mutate: impl FnOnce(&mut AppConfig) -> Result<(), String>,
) -> Result<SettingsView, String> {
    thread_local! {
        static IN_UPDATE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    }
    debug_assert!(
        !IN_UPDATE.with(|flag| flag.get()),
        "update_config called from inside an update_config closure (CONFIG_WRITE is not re-entrant)"
    );
    let view = {
        let _guard = CONFIG_WRITE.lock();
        let mut draft = config_cell().read().clone();
        IN_UPDATE.with(|flag| flag.set(true));
        let mutated = mutate(&mut draft);
        IN_UPDATE.with(|flag| flag.set(false));
        mutated?;
        if !CONFIG_UNSAVED.load(Ordering::Relaxed) && draft == *config_cell().read() {
            to_view(&draft)
        } else {
            commit_locked(draft)?
        }
    };
    // Outside the lock: the tray refresh and window retitle wait on the main
    // thread, which may itself be blocked on `CONFIG_WRITE` in a sync command.
    after_change();
    Ok(view)
}

#[tauri::command]
pub fn reset_settings(section: String) -> Result<SettingsView, String> {
    update_config(|draft| reset_section(draft, &section))
}

/// Put one section (or `all`) of `draft` back to the defaults; the theme goes
/// back to `system` with the rest of `system`.
fn reset_section(draft: &mut AppConfig, section: &str) -> Result<(), String> {
    let defaults = AppConfig::default();
    {
        match section {
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
            other => return Err(crate::i18n::tf("err.reset.invalid", &[("value", other)])),
        }
        Ok(())
    }
}

#[tauri::command]
pub fn get_status() -> StatusView {
    status_now()
}

#[tauri::command]
pub fn toggle_enabled() -> StatusView {
    let mut play_sound = false;
    let mut enabled = false;
    let saved = update_config(|draft| {
        draft.input.enabled = !draft.input.enabled;
        play_sound = draft.system.notification_sound;
        enabled = draft.input.enabled;
        Ok(())
    });
    match saved {
        Ok(view) => {
            // `after_change` already pushed the config; the flag itself is set
            // here too so the very next keystroke sees it, not the next tick.
            #[cfg(windows)]
            vkb_windows::set_enabled(enabled);
            #[cfg(target_os = "macos")]
            vkb_macos::set_enabled(enabled);
            if play_sound {
                play_notification(enabled);
            }
            status_view_from_view(&view)
        }
        Err(error) => {
            tracing::warn!(%error, "could not toggle");
            status_now()
        }
    }
}

/// Play the cue for the state that was just set, not a hard-coded "on"
/// (`D-SND-03`): the UI switch and the tray item call `toggle_enabled`, so OFF
/// must play the off sound.
fn play_notification(enabled: bool) {
    #[cfg(windows)]
    {
        vkb_windows::sound::play_toggle(enabled);
    }
    #[cfg(target_os = "macos")]
    {
        vkb_macos::sound::play_toggle(enabled);
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        let _ = enabled;
    }
}

fn status_view_from_view(view: &SettingsView) -> StatusView {
    StatusView {
        enabled: view.input.enabled,
        method: view.input.method.clone(),
        charset: view.input.charset.clone(),
        mechanism: view.input.mechanism.clone(),
        hook_ready: hook_ready_now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        secure_input_blocking: secure_input_blocking_now(),
    }
}

#[tauri::command]
pub fn set_method(method: String) -> Result<StatusView, String> {
    let parsed =
        InputMethod::from_str(&method).ok_or_else(|| crate::i18n::t("err.method.invalid"))?;
    let view = update_config(|draft| {
        draft.input.method = parsed;
        Ok(())
    })?;
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
        return Err(crate::i18n::t("err.macro.noTrigger"));
    }
    if trigger.chars().any(char::is_whitespace) {
        return Err(crate::i18n::t("err.macro.spaces"));
    }
    if trigger_len > 32 {
        return Err(crate::i18n::tf(
            "err.macro.triggerLong",
            &[("n", &trigger_len.to_string())],
        ));
    }
    if expansion.is_empty() {
        return Err(crate::i18n::t("err.macro.noText"));
    }
    if expansion_len > 500 {
        return Err(crate::i18n::tf(
            "err.macro.textLong",
            &[("n", &expansion_len.to_string())],
        ));
    }
    Ok((trigger.to_string(), expansion.to_string()))
}

#[tauri::command]
pub fn upsert_macro(trigger: String, expansion: String) -> Result<(), String> {
    let (trigger, expansion) = validate_macro(&trigger, &expansion)?;
    update_config(|draft| {
        draft.macros.insert(trigger, expansion);
        Ok(())
    })
    .map(|_| ())
}

#[tauri::command]
pub fn delete_macros(triggers: Vec<String>) -> Result<(), String> {
    update_config(|draft| {
        for trigger in triggers {
            draft.macros.remove(&trigger);
        }
        Ok(())
    })
    .map(|_| ())
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

fn bad_line(display: &str, line_no: usize) -> String {
    crate::i18n::tf(
        "err.import.badLine",
        &[("file", display), ("line", &line_no.to_string())],
    )
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
        return Err(crate::i18n::tf("err.import.tooBig", &[("file", &display)]));
    }
    let body = String::from_utf8(bytes)
        .map_err(|_| crate::i18n::tf("err.import.notUtf8", &[("file", &display)]))?;
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
            return Err(bad_line(&display, line_no));
        };
        match validate_macro(&unescape_tsv(trigger), &unescape_tsv(expansion)) {
            Ok(row) => parsed.push(row),
            Err(_) => return Err(bad_line(&display, line_no)),
        }
        if parsed.len() > MACRO_MAX_ROWS {
            return Err(crate::i18n::tf("err.import.tooMany", &[("file", &display)]));
        }
    }

    if parsed.is_empty() {
        return Err(crate::i18n::tf("err.import.noLines", &[("file", &display)]));
    }

    update_config(|draft| {
        for (trigger, expansion) in &parsed {
            draft.macros.insert(trigger.clone(), expansion.clone());
        }
        Ok(())
    })?;
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
        return Err(crate::i18n::t("err.rule.noProcess"));
    }
    update_config(|draft| {
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
        Ok(())
    })
    .map(|_| ())
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
        Some(slug) => Some(Charset::from_str(slug).ok_or_else(|| crate::i18n::t("err.patch.charset"))?),
        None => None,
    };
    let method = match method.as_deref().map(str::trim).filter(|value| !value.is_empty()) {
        Some(slug) => Some(InputMethod::from_str(slug).ok_or_else(|| crate::i18n::t("err.method.invalid"))?),
        None => None,
    };

    update_config(|draft| {
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
        Ok(())
    })
    .map(|_| ())
}

#[tauri::command]
pub fn delete_app_rule(process: String) -> Result<(), String> {
    update_config(|draft| {
        draft
            .apps
            .rules
            .retain(|rule| !rule.process.eq_ignore_ascii_case(&process));
        Ok(())
    })
    .map(|_| ())
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
    let fields: [(&str, &Option<Hotkey>); 9] = [
        ("enable", &cfg.hotkeys.enable),
        ("disable", &cfg.hotkeys.disable),
        ("toggle", &cfg.hotkeys.toggle),
        ("restore_word", &cfg.hotkeys.restore_word),
        ("app_exclude", &cfg.hotkeys.app_exclude),
        ("macro_panel", &cfg.hotkeys.macro_panel),
        ("macro_toggle", &cfg.hotkeys.macro_toggle),
        ("open_settings", &cfg.hotkeys.open_settings),
        ("cycle_method", &cfg.hotkeys.cycle_method),
    ];
    fields
        .iter()
        .find(|(name, hotkey)| *name != ignore_field && hotkey.as_ref() == Some(&target))
        .map(|(name, _)| (*name).to_string())
}

/// The System Settings panes the permission banner may open. Opening a pane
/// for the user is allowed (27-mac-decisions M3); scripting it is not.
const PRIVACY_PANES: [(&str, &str); 2] = [
    (
        "accessibility",
        "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility",
    ),
    (
        "input_monitoring",
        "x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent",
    ),
];

fn privacy_pane_url(pane: &str) -> Option<&'static str> {
    PRIVACY_PANES
        .iter()
        .find(|(name, _)| *name == pane)
        .map(|(_, url)| *url)
}

/// Open the Accessibility or Input Monitoring pane of System Settings. macOS
/// shows the permission prompt once; after a denial the request API cannot ask
/// again, so the banner sends the user to the pane instead.
#[tauri::command]
pub fn open_privacy_pane(app: tauri::AppHandle, pane: String) -> Result<(), String> {
    let url = privacy_pane_url(&pane)
        .ok_or_else(|| crate::i18n::tf("err.patch.unknownField", &[("field", &pane)]))?;
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|error| crate::i18n::tf("err.update.openLink", &[("error", &error.to_string())]))
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
            .map_err(|error| crate::i18n::tf("err.openFolder", &[("error", &error.to_string())]))
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&dir)
            .spawn()
            .map(|_| ())
            .map_err(|error| crate::i18n::tf("err.openFolder", &[("error", &error.to_string())]))
    }
    #[cfg(all(not(windows), not(target_os = "macos")))]
    {
        std::process::Command::new("xdg-open")
            .arg(&dir)
            .spawn()
            .map(|_| ())
            .map_err(|error| crate::i18n::tf("err.openFolder", &[("error", &error.to_string())]))
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

/// `"windows"`, `"macos"` or `"linux"`: lets the page pick platform wording,
/// symbols and the window chrome without sniffing the user agent.
#[tauri::command]
pub fn platform() -> &'static str {
    std::env::consts::OS
}

/// Whether the app may observe and post keyboard events. Always true off macOS
/// (Windows needs no grant). The page shows the Accessibility banner on `false`.
#[tauri::command]
pub fn accessibility_status() -> bool {
    #[cfg(target_os = "macos")]
    {
        let granted = vkb_macos::has_accessibility_permission();
        // The banner polls this. The moment the grant appears, install the tap
        // (no-op when it already runs) so no restart is needed.
        if granted {
            crate::macos_backend::ensure_installed();
        }
        granted
    }
    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}

/// Whether macOS lets the app listen to the keyboard (Input Monitoring, 10.15+).
/// Never prompts. Always true off macOS. The page shows its own hint on `false`.
#[tauri::command]
pub fn input_monitoring_status() -> bool {
    #[cfg(target_os = "macos")]
    {
        vkb_macos::has_input_monitoring_permission()
    }
    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}

/// Ask macOS for Input Monitoring (system prompt, once). Returns the status
/// after asking. No-op `true` off macOS.
#[tauri::command]
pub fn request_input_monitoring() -> bool {
    #[cfg(target_os = "macos")]
    {
        vkb_macos::request_input_monitoring_permission()
    }
    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}

/// Label of the window whose capture field set the flag, so that window's
/// focus loss or destruction (and only that) clears it.
static CAPTURE_OWNER: parking_lot::Mutex<Option<String>> = parking_lot::Mutex::new(None);

/// The hotkey capture field is focused (`true`, repeated every few seconds while
/// it stays so) or has stopped (`false`): while it is, the keyboard hook does
/// not match or swallow hotkeys, so the chord being recorded reaches the page
/// instead of firing its action (S-07, spec C.6 step 2). The backends also let
/// the flag lapse after ~30 s without a refresh.
#[tauri::command]
pub fn set_hotkey_capture(window: tauri::Window, active: bool) {
    let mut owner = CAPTURE_OWNER.lock();
    if active {
        // A window that is not focused cannot be recording (its blur was lost
        // or the webview is hidden): never re-arm what `Focused(false)` cleared.
        if !window.is_focused().unwrap_or(false) {
            return;
        }
        *owner = Some(window.label().to_string());
    } else if owner.as_deref().is_some_and(|label| label != window.label()) {
        // Another window took over the capture; this one's "stopped" is stale.
        return;
    } else {
        *owner = None;
    }
    apply_hotkey_capture(active);
}

/// A window lost focus or was destroyed: if it was the one capturing, it can
/// no longer say it stopped (a crashed or hidden webview sends no blur).
pub fn clear_hotkey_capture_for(label: &str) {
    let mut owner = CAPTURE_OWNER.lock();
    if owner.as_deref() == Some(label) {
        *owner = None;
        apply_hotkey_capture(false);
    }
}

fn apply_hotkey_capture(active: bool) {
    #[cfg(windows)]
    vkb_windows::set_hotkey_capture(active);
    #[cfg(target_os = "macos")]
    vkb_macos::set_hotkey_capture(active);
    #[cfg(not(any(windows, target_os = "macos")))]
    let _ = active;
}

/// Ask macOS for the Accessibility grant (system prompt, which links to System
/// Settings). Returns the status after asking. No-op `true` off macOS.
#[tauri::command]
pub fn request_accessibility() -> bool {
    #[cfg(target_os = "macos")]
    {
        let granted = vkb_macos::request_accessibility_permission();
        if granted {
            crate::macos_backend::ensure_installed();
        }
        granted
    }
    #[cfg(not(target_os = "macos"))]
    {
        true
    }
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
    fn only_the_two_privacy_panes_can_be_opened() {
        assert!(privacy_pane_url("accessibility").unwrap().ends_with("Privacy_Accessibility"));
        assert!(privacy_pane_url("input_monitoring").unwrap().ends_with("Privacy_ListenEvent"));
        assert_eq!(privacy_pane_url("https://example.com"), None);
    }

    #[test]
    fn reset_all_returns_the_theme_to_system() {
        let mut cfg = AppConfig::default();
        apply_patch(&mut cfg, &serde_json::json!({ "system": { "theme": "dark" } })).unwrap();
        assert_eq!(cfg.system.theme, vkb_config::Theme::Dark);
        reset_section(&mut cfg, "all").unwrap();
        assert_eq!(cfg.system.theme, vkb_config::Theme::System);
        apply_patch(&mut cfg, &serde_json::json!({ "system": { "theme": "light" } })).unwrap();
        reset_section(&mut cfg, "system").unwrap();
        assert_eq!(cfg.system.theme, vkb_config::Theme::System);
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
        assert!(error.starts_with("hotkeys.toggle: "), "{error}");
        assert!(error.contains("Ctrl") || error.contains("Control"), "{error}");
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
