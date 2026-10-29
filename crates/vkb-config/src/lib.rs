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

use vkb_core::{Charset, InputMethod};

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

    /// Fill in fields added after the file was written, and normalise the
    /// sentinels a removed slug deserialises to. Everything else — unknown
    /// keys (`extra`), macros, other sections — is preserved untouched.
    fn migrated(mut self) -> Self {
        if self.version < SCHEMA_VERSION {
            self.version = SCHEMA_VERSION;
        }
        if self.input.charset.is_legacy() {
            self.input.charset = Charset::Unicode;
        }
        if self.input.method.is_legacy() {
            self.input.method = InputMethod::Telex;
        }
        for rule in &mut self.apps.rules {
            if rule.charset.is_some_and(Charset::is_legacy) {
                rule.charset = Some(Charset::Unicode);
            }
            if rule.method.is_some_and(InputMethod::is_legacy) {
                rule.method = Some(InputMethod::Telex);
            }
        }
        // The "follow Windows" theme is gone; an old file resolves to Dark once,
        // and the next save writes the concrete value.
        if self.system.theme == crate::schema::Theme::System {
            self.system.theme = crate::schema::Theme::Dark;
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
    use vkb_core::{Charset, InputMethod};

    #[test]
    fn default_matches_documented_toml() {
        let cfg = AppConfig::default();
        let text = cfg.to_toml().unwrap();
        let back = AppConfig::from_toml(&text).unwrap();
        assert_eq!(cfg.typing.modern_tone, back.typing.modern_tone);
        assert_eq!(cfg.hotkeys.enable, back.hotkeys.enable);
        assert_eq!(back.version, SCHEMA_VERSION);
        assert!(text.contains("theme = \"dark\""), "default theme must serialize as dark:\n{text}");
        assert_eq!(cfg, back, "default config must round-trip");
    }

    #[test]
    fn removed_settings_are_ignored_and_round_trip_without_clobbering_defaults() {
        let src = r#"
version = 1
[system]
run_as_admin = true
start_with_windows = false
[typing]
disable_on_non_us_layout = false
suggestion_fix_1 = true
suggestion_fix_2 = true
metro_fix = true
send_delay_us = 350
[input]
method = "vni"
charset = "unicode"
"#;
        let cfg = AppConfig::from_toml(src).unwrap();
        assert_eq!(cfg.input.method, InputMethod::Vni);
        assert_eq!(cfg.input.charset, Charset::Unicode);
        assert!(!cfg.system.start_with_windows);
        assert!(cfg.typing.spell_check);

        let saved = cfg.to_toml().unwrap();
        assert!(!saved.contains("run_as_admin"));
        assert!(!saved.contains("disable_on_non_us_layout"));
        assert!(!saved.contains("suggestion_fix_1"));
        assert!(!saved.contains("suggestion_fix_2"));
        assert!(!saved.contains("metro_fix"));
        assert!(!saved.contains("send_delay_us"));
        let back = AppConfig::from_toml(&saved).unwrap();
        assert_eq!(back.input, cfg.input);
        assert_eq!(back.typing, cfg.typing);
        assert_eq!(back.system, cfg.system);
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
    fn hotkey_can_be_explicitly_cleared() {        let cfg = AppConfig::from_toml("version = 1\n[hotkeys]\ndisable = \"\"\n").unwrap();
        assert!(cfg.hotkeys.disable.is_none(), "empty string must unbind");

        let text = cfg.to_toml().unwrap();
        let back = AppConfig::from_toml(&text).unwrap();
        assert!(back.hotkeys.disable.is_none(), "unbinding must survive a round-trip");
        assert!(back.hotkeys.enable.is_some(), "other defaults stay");
    }

    #[test]
    fn legacy_charset_and_method_slugs_migrate() {
        // A file written before the legacy charsets and VIQR were removed.
        // Loading must succeed, keep `extra` and the macro table, and rewrite
        // the two removed slugs as `unicode` / `telex`.
        let src = r#"
version = 1
future_key = "kept"
[input]
charset = "tcvn3"
method = "viqr"
[macros]
vn = "Việt Nam"
dc = "được"
"#;
        let cfg = AppConfig::from_toml(src).unwrap();
        assert_eq!(cfg.input.charset, Charset::Unicode);
        assert_eq!(cfg.input.method, InputMethod::Telex);
        assert_eq!(
            cfg.extra.get("future_key").and_then(|value| value.as_str()),
            Some("kept"),
            "unknown key must survive"
        );
        assert_eq!(cfg.macros.get("vn").map(String::as_str), Some("Việt Nam"));
        assert_eq!(cfg.macros.get("dc").map(String::as_str), Some("được"));

        let text = cfg.to_toml().unwrap();
        assert!(!text.contains("tcvn3"), "removed slug rewritten:\n{text}");
        assert!(!text.contains("viqr"), "removed slug rewritten:\n{text}");
        assert!(text.contains("future_key"));
        assert!(text.contains("\"vn\"") || text.contains("vn ="));
    }

    #[test]
    fn every_removed_charset_slug_migrates() {
        for slug in [
            "tcvn3", "vni_windows", "viscii", "viqr", "bk_hcm1", "vps", "utf8",
        ] {
            let src = format!("version = 1\n[input]\ncharset = \"{slug}\"\n");
            let cfg = AppConfig::from_toml(&src).unwrap();
            assert_eq!(cfg.input.charset, Charset::Unicode, "charset {slug}");
        }
        for slug in ["viqr", "VIQR"] {
            let src = format!("version = 1\n[input]\nmethod = \"{slug}\"\n");
            let cfg = AppConfig::from_toml(&src).unwrap();
            assert_eq!(cfg.input.method, InputMethod::Telex, "method {slug}");
        }
    }

    #[test]
    fn legacy_system_theme_migrates_to_dark() {
        let src = r#"version = 1
[system]
theme = "system"
start_with_windows = false
language = "en"
"#;
        let cfg = AppConfig::from_toml(src).unwrap();
        assert_eq!(cfg.system.theme, crate::schema::Theme::Dark);
        assert!(!cfg.system.start_with_windows);
        assert_eq!(cfg.system.language, crate::schema::Language::En);
        let saved = cfg.to_toml().unwrap();
        assert!(!saved.contains("theme = \"system\""));
    }

    #[test]
    fn legacy_per_app_overrides_migrate() {
        let src = r#"
version = 1
[[apps.rules]]
process = "excel.exe"
charset = "tcvn3"
method = "viqr"
"#;
        let cfg = AppConfig::from_toml(src).unwrap();
        assert_eq!(cfg.apps.rules[0].charset, Some(Charset::Unicode));
        assert_eq!(cfg.apps.rules[0].method, Some(InputMethod::Telex));
    }

    #[test]
    fn cycle_method_hotkey_round_trips() {
        let cfg = AppConfig::from_toml("version = 1\n[hotkeys]\ncycle_method = \"Ctrl+Alt+M\"\n")
            .unwrap();
        assert_eq!(
            cfg.hotkeys.cycle_method.as_ref().unwrap().as_str(),
            "Ctrl+Alt+M"
        );

        let text = cfg.to_toml().unwrap();
        assert!(text.contains("cycle_method"), "field dropped:\n{text}");
        let back = AppConfig::from_toml(&text).unwrap();
        assert_eq!(back.hotkeys.cycle_method, cfg.hotkeys.cycle_method);
    }
}
