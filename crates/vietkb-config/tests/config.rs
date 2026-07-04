use vietkb_config::{AppConfig, CommitMode, FallbackMode, LinuxFrontend, PerAppConfig};

#[test]
fn default_config_is_direct_commit() {
    let config = AppConfig::default();
    assert_eq!(config.linux.frontend, LinuxFrontend::Auto);
    assert_eq!(config.linux.commit_mode, CommitMode::Direct);
    assert_eq!(
        config.linux.fallback_mode,
        FallbackMode::SurroundingTextThenPreedit
    );
}

#[test]
fn config_roundtrips_toml() {
    let input = r#"
[input]
method = "telex"
charset = "unicode"
enabled = true

[hotkeys]
toggle = "Ctrl+Shift"

[linux]
frontend = "ibus"
commit_mode = "direct"
fallback_mode = "surrounding-text-then-preedit"

[apps."code.exe"]
enabled = false
browser_fix = true
"#;

    let config = AppConfig::from_toml(input).unwrap_or_else(|err| panic!("parse failed: {err}"));
    assert_eq!(config.linux.frontend, LinuxFrontend::Ibus);
    assert_eq!(
        config.apps.get("code.exe").and_then(|app| app.enabled),
        Some(false)
    );

    let encoded = config
        .to_toml()
        .unwrap_or_else(|err| panic!("serialize failed: {err}"));
    let decoded =
        AppConfig::from_toml(&encoded).unwrap_or_else(|err| panic!("parse failed: {err}"));
    assert_eq!(decoded, config);
}

#[test]
fn config_roundtrips_ini() {
    let mut config = AppConfig::default();
    config.input.enabled = false;
    config.hotkeys.toggle = "Alt+Z".to_owned();
    config.typing.skip_english_words = false;
    config.system.start_with_windows = false;
    config.apps.insert(
        "code.exe".to_owned(),
        PerAppConfig {
            enabled: Some(false),
            browser_fix: Some(true),
        },
    );
    config.apps.insert(
        "notepad.exe".to_owned(),
        PerAppConfig {
            enabled: Some(true),
            browser_fix: None,
        },
    );

    let encoded = config.to_ini();
    let decoded = AppConfig::from_ini(&encoded).unwrap_or_else(|err| panic!("parse failed: {err}"));

    assert_eq!(decoded, config);
}

#[test]
fn parses_ini_app_sections() {
    let input = r#"
[apps.code.exe]
enabled=false
browser_fix=true

[apps.notepad.exe]
enabled=true
"#;

    let config = AppConfig::from_ini(input).unwrap();
    assert_eq!(
        config.apps.get("code.exe").and_then(|app| app.enabled),
        Some(false)
    );
    assert_eq!(
        config.apps.get("code.exe").and_then(|app| app.browser_fix),
        Some(true)
    );
    assert_eq!(
        config.apps.get("notepad.exe").and_then(|app| app.enabled),
        Some(true)
    );
}
