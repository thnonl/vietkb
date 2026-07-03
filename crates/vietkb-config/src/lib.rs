use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppConfig {
    pub input: InputConfig,
    pub hotkeys: HotkeyConfig,
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub toggle: String,
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

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            input: InputConfig {
                method: InputMethodConfig::Telex,
                charset: CharsetConfig::Unicode,
                enabled: true,
            },
            hotkeys: HotkeyConfig {
                toggle: "Ctrl+Shift".to_owned(),
            },
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
}
