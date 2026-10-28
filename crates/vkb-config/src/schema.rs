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
    /// `Đặt dấu kiểu mới` — `hòa` (true) vs `hoà` (false).
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
    /// `Tắt TV khi Layout bàn phím khác US`.
    pub disable_on_non_us_layout: bool,
    /// `Sửa lỗi dấu của Gợi ý 1` — browser/Excel autocomplete interference, level 1.
    pub suggestion_fix_1: bool,
    /// `Sửa lỗi dấu của Gợi ý 2` — aggressive variant, off by default.
    pub suggestion_fix_2: bool,
    /// `Hỗ trợ ứng dụng Metro` — UWP/Mail/OneNote workaround.
    pub metro_fix: bool,
    /// `Gửi phím kiểu chuẩn` — standard scancode injection instead of the fast path.
    pub send_standard: bool,
    /// `Sử dụng Clipboard để gửi phím` — last-resort delivery for stubborn apps.
    pub use_clipboard: bool,
    /// `Cho phép gõ tắt`.
    pub macro_enabled: bool,
    /// `Vẫn cho phép gõ tắt khi tắt gõ Tiếng Việt`.
    pub macro_when_disabled: bool,
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
            disable_on_non_us_layout: true,
            suggestion_fix_1: true,
            suggestion_fix_2: false,
            metro_fix: false,
            send_standard: false,
            use_clipboard: false,
            macro_enabled: true,
            macro_when_disabled: false,
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
        default = "hk_unbound",
        serialize_with = "crate::hotkey::opt::serialize",
        deserialize_with = "crate::hotkey::opt::deserialize"
    )]
    pub toggle: Option<Hotkey>,
    /// Phục hồi từ gõ sai.
    #[serde(
        default = "hk_unbound",
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
    /// `F6` — công cụ chuyển mã.
    #[serde(
        default = "hk_convert_tool",
        serialize_with = "crate::hotkey::opt::serialize",
        deserialize_with = "crate::hotkey::opt::deserialize"
    )]
    pub convert_tool: Option<Hotkey>,
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
        default = "hk_unbound",
        serialize_with = "crate::hotkey::opt::serialize",
        deserialize_with = "crate::hotkey::opt::deserialize"
    )]
    pub open_settings: Option<Hotkey>,
}

fn hk_enable() -> Option<Hotkey> {
    Hotkey::parse("F1").ok()
}
fn hk_disable() -> Option<Hotkey> {
    Hotkey::parse("F2").ok()
}
fn hk_app_exclude() -> Option<Hotkey> {
    Hotkey::parse("F11").ok()
}
fn hk_convert_tool() -> Option<Hotkey> {
    Hotkey::parse("F6").ok()
}
fn hk_macro_panel() -> Option<Hotkey> {
    Hotkey::parse("F8").ok()
}
fn hk_macro_toggle() -> Option<Hotkey> {
    Hotkey::parse("F9").ok()
}
fn hk_unbound() -> Option<Hotkey> {
    None
}

impl Default for HotkeySet {
    fn default() -> Self {
        Self {
            enable: Hotkey::parse("F1").ok(),
            disable: Hotkey::parse("F2").ok(),
            toggle: None,
            restore_word: None,
            app_exclude: Hotkey::parse("F11").ok(),
            convert_tool: Hotkey::parse("F6").ok(),
            macro_panel: Hotkey::parse("F8").ok(),
            macro_toggle: Hotkey::parse("F9").ok(),
            open_settings: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    #[default]
    System,
    Light,
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
    pub run_as_admin: bool,
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
            run_as_admin: false,
            notification_sound: true,
            start_minimized: true,
            language: Language::Vi,
            theme: Theme::System,
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
            window_width: 1040,
            window_height: 720,
            last_tab: "overview".to_string(),
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
