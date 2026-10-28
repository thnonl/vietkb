//! Tauri command surface. Names and payloads are frozen (contract §6).
//! The UI never calls anything else.

use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use vkb_config::{AppConfig, Hotkey};
use vkb_core::{Charset, InputMethod};

static CONFIG: OnceLock<parking_lot::RwLock<AppConfig>> = OnceLock::new();

pub fn config_cell() -> &'static parking_lot::RwLock<AppConfig> {
    CONFIG.get_or_init(|| parking_lot::RwLock::new(vkb_config::load()))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsView {
    pub input: InputView,
    pub typing: TypingView,
    pub hotkeys: HotkeyView,
    pub system: SystemView,
    pub ui: UiView,
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
        hook_ready: cfg.input.enabled,
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

// ---------------------------------------------------------------------------
// Commands (bodies are sprint 1/2 work; signatures frozen)
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn get_settings() -> SettingsView {
    to_view(&config_cell().read())
}

#[tauri::command]
pub fn set_settings(next: SettingsView) -> Result<SettingsView, String> {
    let mut cfg = config_cell().write();
    cfg.input.enabled = next.input.enabled;
    cfg.input.method = InputMethod::from_str(&next.input.method).ok_or("unknown method")?;
    cfg.input.charset = Charset::from_str(&next.input.charset).ok_or("unknown charset")?;
    vkb_config::save(&cfg).map_err(|e| e.to_string())?;
    Ok(to_view(&cfg))
}

#[tauri::command]
pub fn patch_settings(patch: serde_json::Value) -> Result<SettingsView, String> {
    let _ = patch;
    Err("patch_settings lands in sprint 1".into())
}

#[tauri::command]
pub fn reset_settings(section: String) -> Result<SettingsView, String> {
    let mut cfg = config_cell().write();
    let defaults = AppConfig::default();
    match section.as_str() {
        "typing" => cfg.typing = defaults.typing,
        "hotkeys" => cfg.hotkeys = defaults.hotkeys,
        "system" => cfg.system = defaults.system,
        "apps" => cfg.apps = defaults.apps,
        "all" => *cfg = defaults,
        other => return Err(format!("unknown section: {other}")),
    }
    vkb_config::save(&cfg).map_err(|e| e.to_string())?;
    Ok(to_view(&cfg))
}

#[tauri::command]
pub fn get_status() -> StatusView {
    status_view(&config_cell().read())
}

#[tauri::command]
pub fn toggle_enabled() -> StatusView {
    let mut cfg = config_cell().write();
    cfg.input.enabled = !cfg.input.enabled;
    let _ = vkb_config::save(&cfg);
    status_view(&cfg)
}

#[tauri::command]
pub fn set_method(method: String) -> Result<StatusView, String> {
    let mut cfg = config_cell().write();
    cfg.input.method = InputMethod::from_str(&method).ok_or("unknown method")?;
    let _ = vkb_config::save(&cfg);
    Ok(status_view(&cfg))
}

#[tauri::command]
pub fn set_charset(charset: String) -> Result<StatusView, String> {
    let mut cfg = config_cell().write();
    cfg.input.charset = Charset::from_str(&charset).ok_or("unknown charset")?;
    let _ = vkb_config::save(&cfg);
    Ok(status_view(&cfg))
}

#[tauri::command]
pub fn list_macros(query: String) -> Vec<MacroRow> {
    let cfg = config_cell().read();
    let needle = query.to_lowercase();
    cfg.macros
        .iter()
        .filter(|(k, v)| {
            needle.is_empty()
                || k.to_lowercase().contains(&needle)
                || v.to_lowercase().contains(&needle)
        })
        .map(|(k, v)| MacroRow {
            trigger: k.clone(),
            expansion: v.clone(),
        })
        .collect()
}

#[tauri::command]
pub fn upsert_macro(trigger: String, expansion: String) -> Result<(), String> {
    let mut cfg = config_cell().write();
    cfg.macros.insert(trigger, expansion);
    vkb_config::save(&cfg).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_macros(triggers: Vec<String>) -> Result<(), String> {
    let mut cfg = config_cell().write();
    for t in triggers {
        cfg.macros.remove(&t);
    }
    vkb_config::save(&cfg).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn import_macros(path: String) -> Result<usize, String> {
    let _ = path;
    Err("import_macros lands in sprint 2".into())
}

#[tauri::command]
pub fn export_macros(path: String) -> Result<usize, String> {
    let _ = path;
    Err("export_macros lands in sprint 2".into())
}

#[tauri::command]
pub fn list_app_rules() -> Vec<AppRuleRow> {
    let cfg = config_cell().read();
    cfg.apps
        .rules
        .iter()
        .map(|r| AppRuleRow {
            process: r.process.clone(),
            enabled: r.enabled,
            charset: r.charset.map(|c| c.as_str().to_string()),
            method: r.method.map(|m| m.as_str().to_string()),
        })
        .collect()
}

#[tauri::command]
pub fn set_app_rule(process: String, enabled: bool) -> Result<(), String> {
    let mut cfg = config_cell().write();
    if let Some(rule) = cfg.apps.rules.iter_mut().find(|r| r.process.eq_ignore_ascii_case(&process))
    {
        rule.enabled = Some(enabled);
    } else {
        let mut rule = vkb_config::AppRule::new(process);
        rule.enabled = Some(enabled);
        cfg.apps.rules.push(rule);
    }
    vkb_config::save(&cfg).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_app_rule(process: String) -> Result<(), String> {
    let mut cfg = config_cell().write();
    cfg.apps
        .rules
        .retain(|r| !r.process.eq_ignore_ascii_case(&process));
    vkb_config::save(&cfg).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_running_apps() -> Vec<RunningApp> {
    Vec::new()
}

#[tauri::command]
pub fn parse_hotkey(input: String) -> Result<String, String> {
    Hotkey::parse(&input)
        .map(|h| h.as_str())
        .map_err(|e| e.to_string())
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
        ("convert_tool", &cfg.hotkeys.convert_tool),
        ("macro_panel", &cfg.hotkeys.macro_panel),
        ("macro_toggle", &cfg.hotkeys.macro_toggle),
        ("open_settings", &cfg.hotkeys.open_settings),
    ];
    fields
        .iter()
        .find(|(name, hk)| *name != ignore_field && hk.as_ref() == Some(&target))
        .map(|(name, _)| (*name).to_string())
}

#[tauri::command]
pub fn convert_text(input: String, from: String, to: String) -> Result<String, String> {
    let _ = (input, from, to);
    Err("convert_text lands in sprint 2".into())
}

#[tauri::command]
pub fn open_config_folder() -> Result<(), String> {
    Err("open_config_folder lands in sprint 1".into())
}

#[tauri::command]
pub fn app_info() -> AppInfoView {
    AppInfoView {
        version: env!("CARGO_PKG_VERSION").to_string(),
        arch: std::env::consts::ARCH.to_string(),
        config_path: vkb_config::paths::config_file().display().to_string(),
        hook_ready: false,
    }
}
