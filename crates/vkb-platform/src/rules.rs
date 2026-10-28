//! Per-app rule resolution. Pure: the caller supplies the process name.
//!
//! Precedence for the enable flag (highest first):
//! 1. `AppRule.enabled` — an explicit rule always wins.
//! 2. remembered state for that process (`apps.remember_per_app`).
//! 3. global `input.enabled`.
//!
//! Charset / method / macro only ever come from a matching rule; otherwise the
//! global value is inherited.

use vkb_config::AppRule;
use vkb_core::{Charset, InputMethod};

/// Global settings for the current session.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BaseSettings {
    pub enabled: bool,
    pub method: InputMethod,
    pub charset: Charset,
    pub macro_enabled: bool,
}

impl Default for BaseSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            method: InputMethod::Telex,
            charset: Charset::Unicode,
            macro_enabled: true,
        }
    }
}

/// Settings actually used for the next keystroke.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EffectiveSettings {
    pub enabled: bool,
    pub method: InputMethod,
    pub charset: Charset,
    pub macro_enabled: bool,
}

impl EffectiveSettings {
    pub fn from_base(base: BaseSettings) -> Self {
        Self {
            enabled: base.enabled,
            method: base.method,
            charset: base.charset,
            macro_enabled: base.macro_enabled,
        }
    }
}

/// Resolve the effective settings for one process.
pub fn resolve(
    base: BaseSettings,
    rule: Option<&AppRule>,
    remembered: Option<bool>,
    remember_per_app: bool,
) -> EffectiveSettings {
    let mut out = EffectiveSettings::from_base(base);

    if remember_per_app {
        if let Some(state) = remembered {
            out.enabled = state;
        }
    }

    if let Some(rule) = rule {
        if let Some(enabled) = rule.enabled {
            out.enabled = enabled;
        }
        if let Some(charset) = rule.charset {
            out.charset = charset;
        }
        if let Some(method) = rule.method {
            out.method = method;
        }
        if let Some(macro_on) = rule.macro_enabled {
            out.macro_enabled = macro_on;
        }
    }

    out
}

/// Canonical process key: basename only, lowercased, trimmed.
/// `C:\Program Files\Google\Chrome\chrome.exe` -> `chrome.exe`.
pub fn normalize_process_name(value: &str) -> String {
    let trimmed = value.trim();
    let base = trimmed
        .rsplit(['\\', '/'])
        .next()
        .unwrap_or(trimmed)
        .trim();
    base.to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use vkb_core::{Charset, InputMethod};

    fn base() -> BaseSettings {
        BaseSettings {
            enabled: true,
            method: InputMethod::Telex,
            charset: Charset::Unicode,
            macro_enabled: true,
        }
    }

    fn rule_disabled() -> AppRule {
        let mut rule = AppRule::new("GAME.EXE");
        rule.enabled = Some(false);
        rule
    }

    #[test]
    fn no_rule_inherits_global() {
        let eff = resolve(base(), None, None, false);
        assert_eq!(eff, EffectiveSettings::from_base(base()));
    }

    #[test]
    fn version_rule_disables_app() {
        let rule = rule_disabled();
        assert_eq!(rule.process, "game.exe", "AppRule::new lowercases");
        let eff = resolve(base(), Some(&rule), None, false);
        assert!(!eff.enabled, "game.exe must be excluded");
        assert_eq!(eff.charset, Charset::Unicode, "unset fields inherit");
    }

    #[test]
    fn explicit_rule_beats_remembered_state() {
        let rule = rule_disabled();
        // Remembered says "Vietnamese ON" but the explicit rule says off.
        let eff = resolve(base(), Some(&rule), Some(true), true);
        assert!(!eff.enabled);
    }

    #[test]
    fn remembered_state_applies_only_when_enabled_in_config() {
        let on = resolve(base(), None, Some(false), true);
        assert!(!on.enabled);
        let ignored = resolve(base(), None, Some(false), false);
        assert!(ignored.enabled, "remember_per_app off => remembered ignored");
    }

    #[test]
    fn rule_overrides_charset_method_and_macro() {
        let mut rule = AppRule::new("excel.exe");
        rule.charset = Some(Charset::Tcvn3);
        rule.method = Some(InputMethod::Vni);
        rule.macro_enabled = Some(false);
        let eff = resolve(base(), Some(&rule), None, false);
        assert_eq!(eff.charset, Charset::Tcvn3);
        assert_eq!(eff.method, InputMethod::Vni);
        assert!(!eff.macro_enabled);
        assert!(eff.enabled, "unspecified enabled inherits");
    }

    #[test]
    fn rule_can_force_enable_globally_disabled_app() {
        let mut off = base();
        off.enabled = false;
        let mut rule = AppRule::new("code.exe");
        rule.enabled = Some(true);
        let eff = resolve(off, Some(&rule), None, false);
        assert!(eff.enabled);
    }

    #[test]
    fn process_name_normalisation() {
        assert_eq!(
            normalize_process_name(r"C:\Program Files\Google\Chrome\chrome.exe"),
            "chrome.exe"
        );
        assert_eq!(normalize_process_name("  NOTEPAD.EXE "), "notepad.exe");
        assert_eq!(normalize_process_name("zalo"), "zalo");
    }

    #[test]
    fn rule_matching_is_case_insensitive() {
        let rule = AppRule::new("Chrome.exe");
        assert!(vkb_config::AppRuleSet {
            remember_per_app: false,
            rules: vec![rule],
        }
        .find(&normalize_process_name("CHROME.EXE"))
        .is_some());
    }
}
