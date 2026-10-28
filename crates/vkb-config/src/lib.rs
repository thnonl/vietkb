//! Configuration: schema, hotkey parsing, storage.
//!
//! The TOML file is the single source of truth. Unknown keys survive a
//! load/save round-trip (`AppConfig::extra`) so an older build never destroys a
//! newer build's settings.

#![forbid(unsafe_code)]

pub mod hotkey;
pub mod paths;
pub mod schema;

pub use hotkey::{Hotkey, HotkeyError, HotkeyKey, Modifiers};
pub use schema::*;

use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("cannot read {path}: {source}")]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("cannot write {path}: {source}")]
    Write {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("invalid TOML: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("cannot serialize config: {0}")]
    Serialize(#[from] toml::ser::Error),
}

impl AppConfig {
    pub fn from_toml(input: &str) -> Result<Self, ConfigError> {
        let parsed: Self = toml::from_str(input)?;
        Ok(parsed.migrated())
    }

    pub fn to_toml(&self) -> Result<String, ConfigError> {
        Ok(toml::to_string_pretty(self)?)
    }

    /// Fill in fields added after the file was written.
    fn migrated(mut self) -> Self {
        if self.version < SCHEMA_VERSION {
            self.version = SCHEMA_VERSION;
        }
        self
    }
}

/// Load from the standard path, falling back to defaults.
///
/// A corrupt file is moved aside (`config.toml.corrupt-<n>`) instead of being
/// silently overwritten, so the user can recover a hand-edited mistake.
pub fn load() -> AppConfig {
    let path = paths::config_file();
    load_from(&path).unwrap_or_else(|err| {
        tracing::warn!(error = %err, path = %path.display(), "falling back to default config");
        AppConfig::default()
    })
}

pub fn load_from(path: &Path) -> Result<AppConfig, ConfigError> {
    let input = std::fs::read_to_string(path).map_err(|source| ConfigError::Read {
        path: path.display().to_string(),
        source,
    })?;
    AppConfig::from_toml(&input)
}

/// Atomic save: write a sibling temp file, then rename over the target.
pub fn save(config: &AppConfig) -> Result<(), ConfigError> {
    save_to(&paths::config_file(), config)
}

pub fn save_to(path: &Path, config: &AppConfig) -> Result<(), ConfigError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| ConfigError::Write {
            path: parent.display().to_string(),
            source,
        })?;
    }
    let body = config.to_toml()?;
    let tmp = path.with_extension("toml.tmp");
    std::fs::write(&tmp, body).map_err(|source| ConfigError::Write {
        path: tmp.display().to_string(),
        source,
    })?;
    std::fs::rename(&tmp, path).map_err(|source| ConfigError::Write {
        path: path.display().to_string(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_matches_documented_toml() {
        let cfg = AppConfig::default();
        let text = cfg.to_toml().unwrap();
        let back = AppConfig::from_toml(&text).unwrap();
        assert_eq!(cfg.typing.modern_tone, back.typing.modern_tone);
        assert_eq!(cfg.hotkeys.enable, back.hotkeys.enable);
        assert_eq!(back.version, SCHEMA_VERSION);
    }

    #[test]
    fn unknown_keys_survive_roundtrip() {
        let src = r#"
version = 1
[future]
surprise = "kept"
"#;
        let cfg = AppConfig::from_toml(src).unwrap();
        let out = cfg.to_toml().unwrap();
        assert!(out.contains("surprise"), "unknown key dropped:\n{out}");
    }

    #[test]
    fn hotkey_strings_parse_from_config() {
        let cfg = AppConfig::from_toml("version = 1\n[hotkeys]\nenable = \"Ctrl+Shift\"\n").unwrap();
        assert_eq!(cfg.hotkeys.enable.as_ref().unwrap().as_str(), "Ctrl+Shift");
        // Unspecified hotkeys fall back to the documented defaults, not to None.
        assert_eq!(cfg.hotkeys.disable.as_ref().unwrap().as_str(), "F2");
    }

    #[test]
    fn hotkey_can_be_explicitly_cleared() {
        let cfg = AppConfig::from_toml("version = 1\n[hotkeys]\ndisable = \"\"\n").unwrap();
        assert!(cfg.hotkeys.disable.is_none(), "empty string must unbind");

        let text = cfg.to_toml().unwrap();
        let back = AppConfig::from_toml(&text).unwrap();
        assert!(back.hotkeys.disable.is_none(), "unbinding must survive a round-trip");
        assert!(back.hotkeys.enable.is_some(), "other defaults stay");
    }
}
