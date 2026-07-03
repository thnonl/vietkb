use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IniError {
    message: String,
}

impl IniError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for IniError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for IniError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppConfig {
    pub input: InputConfig,
    pub hotkeys: HotkeyConfig,
    #[serde(default)]
    pub typing: TypingConfig,
    #[serde(default)]
    pub system: SystemConfig,
    pub linux: LinuxConfig,
    #[serde(default)]
    pub apps: std::collections::BTreeMap<String, PerAppConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputConfig {
    pub method: InputMethodConfig,
    pub charset: CharsetConfig,
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InputMethodConfig {
    Telex,
    Vni,
    Viqr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CharsetConfig {
    Unicode,
    VniWindows,
    Tcvn3,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyConfig {
    pub toggle: String,
    #[serde(default = "default_restore_hotkey")]
    pub restore_word: String,
    #[serde(default)]
    pub function_keys: FunctionKeyConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionKeyConfig {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub win: bool,
    pub f1_toggle_vietnamese_on: bool,
    pub f2_toggle_vietnamese_off: bool,
    pub f3_unicode: bool,
    pub f4_charset: bool,
    pub f4_charset_value: String,
    pub f5_settings: bool,
    pub f6_convert_tool: bool,
    pub f7_convert_clipboard: bool,
    pub f11_toggle_excluded_apps: bool,
    pub f12_reset: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypingConfig {
    pub old_oa_uy: bool,
    pub smart_mark: bool,
    pub use_j_f_w_as_consonants: bool,
    pub auto_capitalize_after_punctuation: bool,
    pub use_clipboard_to_send_keys: bool,
    pub spell_check: bool,
    pub restore_wrong_word: bool,
    pub us_keyboard_layout_when_off: bool,
    pub suggestion_fix_1: bool,
    pub suggestion_fix_2: bool,
    pub skip_english_words: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemConfig {
    pub run_as_admin: bool,
    pub start_with_windows: bool,
    pub show_dialog_on_startup: bool,
    pub notification_sound: bool,
    pub standard_key_sending: bool,
    pub metro_apps_support: bool,
    pub custom_tray_icon: bool,
    pub excluded_apps: bool,
    pub auto_exclude_apps: bool,
    pub check_updates_on_startup: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinuxConfig {
    pub frontend: LinuxFrontend,
    pub commit_mode: CommitMode,
    pub fallback_mode: FallbackMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LinuxFrontend {
    Ibus,
    Fcitx5,
    Auto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CommitMode {
    Direct,
    Preedit,
    Hybrid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FallbackMode {
    SurroundingTextThenPreedit,
    PreeditOnly,
    CommitOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PerAppConfig {
    pub enabled: Option<bool>,
    pub browser_fix: Option<bool>,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            toggle: "Ctrl+Shift".to_owned(),
            restore_word: default_restore_hotkey(),
            function_keys: FunctionKeyConfig::default(),
        }
    }
}

impl Default for FunctionKeyConfig {
    fn default() -> Self {
        Self {
            ctrl: true,
            shift: true,
            alt: true,
            win: false,
            f1_toggle_vietnamese_on: true,
            f2_toggle_vietnamese_off: true,
            f3_unicode: true,
            f4_charset: true,
            f4_charset_value: "VIQR".to_owned(),
            f5_settings: true,
            f6_convert_tool: true,
            f7_convert_clipboard: true,
            f11_toggle_excluded_apps: true,
            f12_reset: true,
        }
    }
}

impl Default for TypingConfig {
    fn default() -> Self {
        Self {
            old_oa_uy: false,
            smart_mark: true,
            use_j_f_w_as_consonants: true,
            auto_capitalize_after_punctuation: false,
            use_clipboard_to_send_keys: false,
            spell_check: true,
            restore_wrong_word: false,
            us_keyboard_layout_when_off: false,
            suggestion_fix_1: false,
            suggestion_fix_2: false,
            skip_english_words: true,
        }
    }
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            run_as_admin: false,
            start_with_windows: true,
            show_dialog_on_startup: false,
            notification_sound: true,
            standard_key_sending: true,
            metro_apps_support: false,
            custom_tray_icon: false,
            excluded_apps: false,
            auto_exclude_apps: false,
            check_updates_on_startup: true,
        }
    }
}

fn default_restore_hotkey() -> String {
    "Shift+Alt".to_owned()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            input: InputConfig {
                method: InputMethodConfig::Telex,
                charset: CharsetConfig::Unicode,
                enabled: true,
            },
            hotkeys: HotkeyConfig::default(),
            typing: TypingConfig::default(),
            system: SystemConfig::default(),
            linux: LinuxConfig {
                frontend: LinuxFrontend::Auto,
                commit_mode: CommitMode::Direct,
                fallback_mode: FallbackMode::SurroundingTextThenPreedit,
            },
            apps: std::collections::BTreeMap::new(),
        }
    }
}

impl AppConfig {
    pub fn from_toml(input: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(input)
    }

    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    pub fn from_ini(input: &str) -> Result<Self, IniError> {
        let mut config = Self::default();
        let mut section = String::new();

        for (line_index, raw_line) in input.lines().enumerate() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                section = line[1..line.len() - 1].trim().to_owned();
                continue;
            }

            let Some((key, value)) = line.split_once('=') else {
                return Err(IniError::new(format!(
                    "invalid INI line {}",
                    line_index + 1
                )));
            };
            let key = key.trim();
            let value = unquote(value.trim());

            match (section.as_str(), key) {
                ("input", "method") => config.input.method = parse_input_method(value)?,
                ("input", "charset") => config.input.charset = parse_charset(value)?,
                ("input", "enabled") => config.input.enabled = parse_bool(value)?,
                ("hotkeys", "toggle") => config.hotkeys.toggle = value.to_owned(),
                ("hotkeys", "restore_word") => config.hotkeys.restore_word = value.to_owned(),
                ("hotkeys.function_keys", "ctrl") => {
                    config.hotkeys.function_keys.ctrl = parse_bool(value)?
                }
                ("hotkeys.function_keys", "shift") => {
                    config.hotkeys.function_keys.shift = parse_bool(value)?
                }
                ("hotkeys.function_keys", "alt") => {
                    config.hotkeys.function_keys.alt = parse_bool(value)?
                }
                ("hotkeys.function_keys", "win") => {
                    config.hotkeys.function_keys.win = parse_bool(value)?
                }
                ("hotkeys.function_keys", "f1_toggle_vietnamese_on") => {
                    config.hotkeys.function_keys.f1_toggle_vietnamese_on = parse_bool(value)?
                }
                ("hotkeys.function_keys", "f2_toggle_vietnamese_off") => {
                    config.hotkeys.function_keys.f2_toggle_vietnamese_off = parse_bool(value)?
                }
                ("hotkeys.function_keys", "f3_unicode") => {
                    config.hotkeys.function_keys.f3_unicode = parse_bool(value)?
                }
                ("hotkeys.function_keys", "f4_charset") => {
                    config.hotkeys.function_keys.f4_charset = parse_bool(value)?
                }
                ("hotkeys.function_keys", "f4_charset_value") => {
                    config.hotkeys.function_keys.f4_charset_value = value.to_owned()
                }
                ("hotkeys.function_keys", "f5_settings") => {
                    config.hotkeys.function_keys.f5_settings = parse_bool(value)?
                }
                ("hotkeys.function_keys", "f6_convert_tool") => {
                    config.hotkeys.function_keys.f6_convert_tool = parse_bool(value)?
                }
                ("hotkeys.function_keys", "f7_convert_clipboard") => {
                    config.hotkeys.function_keys.f7_convert_clipboard = parse_bool(value)?
                }
                ("hotkeys.function_keys", "f11_toggle_excluded_apps") => {
                    config.hotkeys.function_keys.f11_toggle_excluded_apps = parse_bool(value)?
                }
                ("hotkeys.function_keys", "f12_reset") => {
                    config.hotkeys.function_keys.f12_reset = parse_bool(value)?
                }
                ("typing", "old_oa_uy") => config.typing.old_oa_uy = parse_bool(value)?,
                ("typing", "smart_mark") => config.typing.smart_mark = parse_bool(value)?,
                ("typing", "use_j_f_w_as_consonants") => {
                    config.typing.use_j_f_w_as_consonants = parse_bool(value)?
                }
                ("typing", "auto_capitalize_after_punctuation") => {
                    config.typing.auto_capitalize_after_punctuation = parse_bool(value)?
                }
                ("typing", "use_clipboard_to_send_keys") => {
                    config.typing.use_clipboard_to_send_keys = parse_bool(value)?
                }
                ("typing", "spell_check") => config.typing.spell_check = parse_bool(value)?,
                ("typing", "restore_wrong_word") => {
                    config.typing.restore_wrong_word = parse_bool(value)?
                }
                ("typing", "us_keyboard_layout_when_off") => {
                    config.typing.us_keyboard_layout_when_off = parse_bool(value)?
                }
                ("typing", "suggestion_fix_1") => {
                    config.typing.suggestion_fix_1 = parse_bool(value)?
                }
                ("typing", "suggestion_fix_2") => {
                    config.typing.suggestion_fix_2 = parse_bool(value)?
                }
                ("typing", "skip_english_words") => {
                    config.typing.skip_english_words = parse_bool(value)?
                }
                ("system", "run_as_admin") => config.system.run_as_admin = parse_bool(value)?,
                ("system", "start_with_windows") => {
                    config.system.start_with_windows = parse_bool(value)?
                }
                ("system", "show_dialog_on_startup") => {
                    config.system.show_dialog_on_startup = parse_bool(value)?
                }
                ("system", "notification_sound") => {
                    config.system.notification_sound = parse_bool(value)?
                }
                ("system", "standard_key_sending") => {
                    config.system.standard_key_sending = parse_bool(value)?
                }
                ("system", "metro_apps_support") => {
                    config.system.metro_apps_support = parse_bool(value)?
                }
                ("system", "custom_tray_icon") => {
                    config.system.custom_tray_icon = parse_bool(value)?
                }
                ("system", "excluded_apps") => config.system.excluded_apps = parse_bool(value)?,
                ("system", "auto_exclude_apps") => {
                    config.system.auto_exclude_apps = parse_bool(value)?
                }
                ("system", "check_updates_on_startup") => {
                    config.system.check_updates_on_startup = parse_bool(value)?
                }
                ("linux", "frontend") => config.linux.frontend = parse_linux_frontend(value)?,
                ("linux", "commit_mode") => config.linux.commit_mode = parse_commit_mode(value)?,
                ("linux", "fallback_mode") => {
                    config.linux.fallback_mode = parse_fallback_mode(value)?
                }
                _ => {}
            }
        }

        Ok(config)
    }

    pub fn to_ini(&self) -> String {
        format!(
            concat!(
                "[input]\nmethod={}\ncharset={}\nenabled={}\n\n",
                "[hotkeys]\ntoggle={}\nrestore_word={}\n\n",
                "[hotkeys.function_keys]\nctrl={}\nshift={}\nalt={}\nwin={}\nf1_toggle_vietnamese_on={}\nf2_toggle_vietnamese_off={}\nf3_unicode={}\nf4_charset={}\nf4_charset_value={}\nf5_settings={}\nf6_convert_tool={}\nf7_convert_clipboard={}\nf11_toggle_excluded_apps={}\nf12_reset={}\n\n",
                "[typing]\nold_oa_uy={}\nsmart_mark={}\nuse_j_f_w_as_consonants={}\nauto_capitalize_after_punctuation={}\nuse_clipboard_to_send_keys={}\nspell_check={}\nrestore_wrong_word={}\nus_keyboard_layout_when_off={}\nsuggestion_fix_1={}\nsuggestion_fix_2={}\nskip_english_words={}\n\n",
                "[system]\nrun_as_admin={}\nstart_with_windows={}\nshow_dialog_on_startup={}\nnotification_sound={}\nstandard_key_sending={}\nmetro_apps_support={}\ncustom_tray_icon={}\nexcluded_apps={}\nauto_exclude_apps={}\ncheck_updates_on_startup={}\n\n",
                "[linux]\nfrontend={}\ncommit_mode={}\nfallback_mode={}\n"
            ),
            input_method_to_ini(self.input.method), charset_to_ini(self.input.charset), self.input.enabled,
            self.hotkeys.toggle, self.hotkeys.restore_word,
            self.hotkeys.function_keys.ctrl, self.hotkeys.function_keys.shift, self.hotkeys.function_keys.alt, self.hotkeys.function_keys.win, self.hotkeys.function_keys.f1_toggle_vietnamese_on, self.hotkeys.function_keys.f2_toggle_vietnamese_off, self.hotkeys.function_keys.f3_unicode, self.hotkeys.function_keys.f4_charset, self.hotkeys.function_keys.f4_charset_value, self.hotkeys.function_keys.f5_settings, self.hotkeys.function_keys.f6_convert_tool, self.hotkeys.function_keys.f7_convert_clipboard, self.hotkeys.function_keys.f11_toggle_excluded_apps, self.hotkeys.function_keys.f12_reset,
            self.typing.old_oa_uy, self.typing.smart_mark, self.typing.use_j_f_w_as_consonants, self.typing.auto_capitalize_after_punctuation, self.typing.use_clipboard_to_send_keys, self.typing.spell_check, self.typing.restore_wrong_word, self.typing.us_keyboard_layout_when_off, self.typing.suggestion_fix_1, self.typing.suggestion_fix_2, self.typing.skip_english_words,
            self.system.run_as_admin, self.system.start_with_windows, self.system.show_dialog_on_startup, self.system.notification_sound, self.system.standard_key_sending, self.system.metro_apps_support, self.system.custom_tray_icon, self.system.excluded_apps, self.system.auto_exclude_apps, self.system.check_updates_on_startup,
            linux_frontend_to_ini(self.linux.frontend), commit_mode_to_ini(self.linux.commit_mode), fallback_mode_to_ini(self.linux.fallback_mode),
        )
    }
}

fn unquote(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(value)
}

fn parse_bool(value: &str) -> Result<bool, IniError> {
    match value.to_ascii_lowercase().as_str() {
        "true" | "yes" | "1" | "on" => Ok(true),
        "false" | "no" | "0" | "off" => Ok(false),
        _ => Err(IniError::new(format!("invalid bool: {value}"))),
    }
}

fn parse_input_method(value: &str) -> Result<InputMethodConfig, IniError> {
    match value {
        "telex" => Ok(InputMethodConfig::Telex),
        "vni" => Ok(InputMethodConfig::Vni),
        "viqr" => Ok(InputMethodConfig::Viqr),
        _ => Err(IniError::new(format!("invalid input method: {value}"))),
    }
}

fn input_method_to_ini(value: InputMethodConfig) -> &'static str {
    match value {
        InputMethodConfig::Telex => "telex",
        InputMethodConfig::Vni => "vni",
        InputMethodConfig::Viqr => "viqr",
    }
}

fn parse_charset(value: &str) -> Result<CharsetConfig, IniError> {
    match value {
        "unicode" => Ok(CharsetConfig::Unicode),
        "vni-windows" => Ok(CharsetConfig::VniWindows),
        "tcvn3" => Ok(CharsetConfig::Tcvn3),
        _ => Err(IniError::new(format!("invalid charset: {value}"))),
    }
}

fn charset_to_ini(value: CharsetConfig) -> &'static str {
    match value {
        CharsetConfig::Unicode => "unicode",
        CharsetConfig::VniWindows => "vni-windows",
        CharsetConfig::Tcvn3 => "tcvn3",
    }
}

fn parse_linux_frontend(value: &str) -> Result<LinuxFrontend, IniError> {
    match value {
        "ibus" => Ok(LinuxFrontend::Ibus),
        "fcitx5" => Ok(LinuxFrontend::Fcitx5),
        "auto" => Ok(LinuxFrontend::Auto),
        _ => Err(IniError::new(format!("invalid linux frontend: {value}"))),
    }
}

fn linux_frontend_to_ini(value: LinuxFrontend) -> &'static str {
    match value {
        LinuxFrontend::Ibus => "ibus",
        LinuxFrontend::Fcitx5 => "fcitx5",
        LinuxFrontend::Auto => "auto",
    }
}

fn parse_commit_mode(value: &str) -> Result<CommitMode, IniError> {
    match value {
        "direct" => Ok(CommitMode::Direct),
        "preedit" => Ok(CommitMode::Preedit),
        "hybrid" => Ok(CommitMode::Hybrid),
        _ => Err(IniError::new(format!("invalid commit mode: {value}"))),
    }
}

fn commit_mode_to_ini(value: CommitMode) -> &'static str {
    match value {
        CommitMode::Direct => "direct",
        CommitMode::Preedit => "preedit",
        CommitMode::Hybrid => "hybrid",
    }
}

fn parse_fallback_mode(value: &str) -> Result<FallbackMode, IniError> {
    match value {
        "surrounding-text-then-preedit" => Ok(FallbackMode::SurroundingTextThenPreedit),
        "preedit-only" => Ok(FallbackMode::PreeditOnly),
        "commit-only" => Ok(FallbackMode::CommitOnly),
        _ => Err(IniError::new(format!("invalid fallback mode: {value}"))),
    }
}

fn fallback_mode_to_ini(value: FallbackMode) -> &'static str {
    match value {
        FallbackMode::SurroundingTextThenPreedit => "surrounding-text-then-preedit",
        FallbackMode::PreeditOnly => "preedit-only",
        FallbackMode::CommitOnly => "commit-only",
    }
}
