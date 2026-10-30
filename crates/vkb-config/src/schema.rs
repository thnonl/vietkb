//! Config schema. Field-by-field parity notes live in
//! `docs/evkey-clone-work/20-spec.md` §B; the TOML in the contract is generated
//! from this file, so this is the authority.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use vkb_core::{Charset, InputMethod};

use crate::hotkey::Hotkey;

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub version: u32,
    pub input: InputConfig,
    pub typing: TypingConfig,
    pub hotkeys: HotkeySet,
    pub system: SystemConfig,
    pub ui: UiConfig,
    pub apps: AppRuleSet,
    /// trigger -> expansion. Ordered so the file diffs cleanly.
    pub macros: BTreeMap<String, String>,
    /// Unknown keys from a newer schema, preserved verbatim on save.
    #[serde(flatten)]
    pub extra: toml::Table,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: SCHEMA_VERSION,
            input: InputConfig::default(),
            typing: TypingConfig::default(),
            hotkeys: HotkeySet::default(),
            system: SystemConfig::default(),
            ui: UiConfig::default(),
            apps: AppRuleSet::default(),
            macros: BTreeMap::new(),
            extra: toml::Table::new(),
        }
    }
}

/// How the app delivers text to the target program.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Mechanism {
    /// Low-level keyboard hook + synthetic input. Default, works everywhere.
    #[default]
    Hook,
    /// IME/TSF path, for apps where a hook misbehaves (some UWP/Metro, games).
    Ime,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct InputConfig {
    pub enabled: bool,
    pub method: InputMethod,
    pub charset: Charset,
    pub mechanism: Mechanism,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            method: InputMethod::Telex,
            charset: Charset::Unicode,
            mechanism: Mechanism::Hook,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct TypingConfig {
    /// `Đặt dấu kiểu mới` — **true** = kiểu mới: `hòa`, `thúy`, `khỏe` (tone on the **first**
    /// vowel of an open `oa`/`oe`/`uy` syllable). **false** = kiểu cũ: `hoà`, `thuý`, `khoẻ`.
    /// Mirrors `vkb_core::EngineOptions::modern_tone`.
    pub modern_tone: bool,
    /// `Đặt dấu tự do` — allow the mark to be typed after the vowel run or at word end.
    pub free_tone: bool,
    /// `Cho phép 'f j w z' làm phụ âm`.
    pub jfwz_as_consonant: bool,
    /// `Tự động viết hoa sau "Enter . ! ?"`.
    pub auto_capitalize: bool,
    /// `Bật kiểm tra chính tả`.
    pub spell_check: bool,
    /// `Tự động khôi phục phím với từ sai` (only meaningful with `spell_check`).
    pub restore_wrong_word: bool,
    /// `Gửi phím kiểu chuẩn` — standard scancode injection instead of the fast path.
    pub send_standard: bool,
    /// `Sử dụng Clipboard để gửi phím` — last-resort delivery for stubborn apps.
    pub use_clipboard: bool,
    /// `Cho phép gõ tắt`.
    pub macro_enabled: bool,
    /// `Vẫn cho phép gõ tắt khi tắt gõ Tiếng Việt`.
    pub macro_when_disabled: bool,
    /// `Tự sửa ưo thành ươ` — auto-horn when `w` follows `uo`/`ưo`.
    pub auto_horn_uo: bool,
    /// `Giữ nguyên email, địa chỉ web` — never add marks inside URL/email/code.
    pub raw_zone: bool,
    /// `Viết hoa theo từ tắt` — macro output follows the trigger's capitalisation.
    pub macro_case_follow: bool,
}

impl Default for TypingConfig {
    fn default() -> Self {
        Self {
            modern_tone: true,
            free_tone: true,
            jfwz_as_consonant: true,
            auto_capitalize: false,
            spell_check: true,
            restore_wrong_word: false,
            send_standard: false,
            use_clipboard: false,
            macro_enabled: true,
            macro_when_disabled: false,
            auto_horn_uo: true,
            raw_zone: true,
            macro_case_follow: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HotkeySet {
    /// `F1` — bật tiếng Việt. `None`/`""` = không dùng phím tắt.
    #[serde(
        default = "hk_enable",
        serialize_with = "crate::hotkey::opt::serialize",
        deserialize_with = "crate::hotkey::opt::deserialize"
    )]
    pub enable: Option<Hotkey>,
    /// `F2` — tắt tiếng Việt.
    #[serde(
        default = "hk_disable",
        serialize_with = "crate::hotkey::opt::serialize",
        deserialize_with = "crate::hotkey::opt::deserialize"
    )]
    pub disable: Option<Hotkey>,
    /// Optional single-key toggle (`Ctrl+Shift` in EVKey lineage).
    #[serde(
        default = "hk_toggle",
        serialize_with = "crate::hotkey::opt::serialize",
        deserialize_with = "crate::hotkey::opt::deserialize"
    )]
    pub toggle: Option<Hotkey>,
    /// Phục hồi từ gõ sai.
    #[serde(
        default = "hk_restore_word",
        serialize_with = "crate::hotkey::opt::serialize",
        deserialize_with = "crate::hotkey::opt::deserialize"
    )]
    pub restore_word: Option<Hotkey>,
    /// `F11` — bật/tắt loại trừ ứng dụng.
    #[serde(
        default = "hk_app_exclude",
        serialize_with = "crate::hotkey::opt::serialize",
        deserialize_with = "crate::hotkey::opt::deserialize"
    )]
    pub app_exclude: Option<Hotkey>,
    /// `F8` — bảng gõ tắt.
    #[serde(
        default = "hk_macro_panel",
        serialize_with = "crate::hotkey::opt::serialize",
        deserialize_with = "crate::hotkey::opt::deserialize"
    )]
    pub macro_panel: Option<Hotkey>,
    /// `F9` — bật/tắt gõ tắt.
    #[serde(
        default = "hk_macro_toggle",
        serialize_with = "crate::hotkey::opt::serialize",
        deserialize_with = "crate::hotkey::opt::deserialize"
    )]
    pub macro_toggle: Option<Hotkey>,
    /// Mở cửa sổ cài đặt.
    #[serde(
        default = "hk_open_settings",
        serialize_with = "crate::hotkey::opt::serialize",
        deserialize_with = "crate::hotkey::opt::deserialize"
    )]
    pub open_settings: Option<Hotkey>,
    /// `Đổi kiểu gõ` — cycles Telex ↔ VNI.
    #[serde(
        default = "hk_unbound",
        serialize_with = "crate::hotkey::opt::serialize",
        deserialize_with = "crate::hotkey::opt::deserialize"
    )]
    pub cycle_method: Option<Hotkey>,
    /// `Kiểu chuyển bật/tắt`: switch on release or on press.
    #[serde(default)]
    pub toggle_mode: ToggleMode,
}

/// How the toggle hotkey fires: on key release (EVKey's default) or on press.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ToggleMode {
    #[default]
    Release,
    Press,
}

/// Platform default for a hotkey: Windows/Linux string, macOS string (`None` = unbound).
/// Decision M2 (`27-mac-decisions.md`): function keys are brightness/media keys on
/// Mac laptops (and `F11` is Show Desktop), so they are not bound by default there.
fn platform_hotkey(other: &str, mac: Option<&str>) -> Option<Hotkey> {
    if cfg!(target_os = "macos") {
        mac.and_then(|text| Hotkey::parse(text).ok())
    } else {
        Hotkey::parse(other).ok()
    }
}

fn hk_enable() -> Option<Hotkey> {
    platform_hotkey("F1", None)
}
fn hk_disable() -> Option<Hotkey> {
    platform_hotkey("F2", None)
}
fn hk_app_exclude() -> Option<Hotkey> {
    platform_hotkey("F11", None)
}
fn hk_macro_panel() -> Option<Hotkey> {
    platform_hotkey("F8", None)
}
fn hk_macro_toggle() -> Option<Hotkey> {
    platform_hotkey("F9", None)
}
fn hk_toggle() -> Option<Hotkey> {
    Hotkey::parse("Ctrl+Shift").ok()
}
fn hk_restore_word() -> Option<Hotkey> {
    Hotkey::parse("Ctrl+Shift+Z").ok()
}
fn hk_open_settings() -> Option<Hotkey> {
    Hotkey::parse("Ctrl+Alt+S").ok()
}
fn hk_unbound() -> Option<Hotkey> {
    None
}

impl Default for HotkeySet {
    fn default() -> Self {
        Self {
            enable: hk_enable(),
            disable: hk_disable(),
            toggle: Hotkey::parse("Ctrl+Shift").ok(),
            restore_word: Hotkey::parse("Ctrl+Shift+Z").ok(),
            app_exclude: hk_app_exclude(),
            macro_panel: hk_macro_panel(),
            macro_toggle: hk_macro_toggle(),
            open_settings: Hotkey::parse("Ctrl+Alt+S").ok(),
            cycle_method: None,
            toggle_mode: ToggleMode::Release,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    /// Legacy value from older files; `AppConfig::migrated` resolves it to
    /// `Dark` on load and the UI no longer offers it.
    System,
    Light,
    #[default]
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    #[default]
    Vi,
    En,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SystemConfig {
    pub start_with_windows: bool,
    pub notification_sound: bool,
    /// Start hidden in the tray without showing the window.
    pub start_minimized: bool,
    pub language: Language,
    pub theme: Theme,
    pub check_updates: bool,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            start_with_windows: true,
            notification_sound: true,
            start_minimized: true,
            language: Language::Vi,
            theme: Theme::Dark,
            check_updates: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiConfig {
    pub window_width: u32,
    pub window_height: u32,
    pub last_tab: String,
    /// Remember the toggles the user collapsed on the dashboard.
    pub compact_rows: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            window_width: 720,
            window_height: 560,
            last_tab: "go".to_string(),
            compact_rows: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppRule {
    /// Matched case-insensitively against the executable name, e.g. `game.exe`.
    pub process: String,
    /// `None` = leave global state alone for this app.
    pub enabled: Option<bool>,
    pub charset: Option<Charset>,
    pub method: Option<InputMethod>,
    /// `None` = inherit the global macro switch.
    pub macro_enabled: Option<bool>,
}

impl AppRule {
    pub fn new(process: impl Into<String>) -> Self {
        Self {
            process: process.into().to_ascii_lowercase(),
            enabled: None,
            charset: None,
            method: None,
            macro_enabled: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AppRuleSet {
    /// `Loại trừ ứng dụng tự động` — remember Vietnamese on/off per app.
    pub remember_per_app: bool,
    /// `Loại trừ ứng dụng` — explicit rules.
    pub rules: Vec<AppRule>,
}

impl AppRuleSet {
    pub fn find(&self, process: &str) -> Option<&AppRule> {
        let needle = process.to_ascii_lowercase();
        self.rules
            .iter()
            .find(|rule| rule.process.eq_ignore_ascii_case(&needle))
    }
}

#[cfg(test)]
mod tests {
    /// Forward compatibility for the removed `[system]` sound key (see
    /// `22-sound-spec.md` §6/§9). The name is assembled from parts so the
    /// zero-leftover grep stays meaningful: this test is the only place that
    /// writes it, and it asserts the key is ignored, not supported.
    #[test]
    fn removed_sound_key_still_loads() {
        const REMOVED: &str = concat!("sound", "_pack");
        for value in ["\"go_moc\"", "\"system\"", "\"khong_ton_tai\"", "7"] {
            let src = format!(
                "version = 1\n[system]\n{REMOVED} = {value}\nnotification_sound = false\n[typing]\nspell_check = false\n[macros]\nvn = \"Việt Nam\"\n"
            );
            let cfg = crate::AppConfig::from_toml(&src)
                .unwrap_or_else(|err| panic!("{value} must not fail the load: {err}"));
            assert!(!cfg.system.notification_sound);
            assert!(!cfg.typing.spell_check, "other keys survive");
            assert_eq!(cfg.macros.get("vn").map(String::as_str), Some("Việt Nam"));
            let saved = cfg.to_toml().unwrap();
            assert!(
                !saved.contains(REMOVED),
                "the removed key must not be written back:\n{saved}"
            );
        }
    }
}