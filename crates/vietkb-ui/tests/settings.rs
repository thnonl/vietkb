use vietkb_config::{AppConfig, CommitMode, InputMethodConfig, LinuxFrontend};
use vietkb_ui::SettingsViewModel;

#[test]
fn settings_view_model_updates_config() {
    let mut config = AppConfig::default();
    let mut view_model = SettingsViewModel::from(&config);
    view_model.method = InputMethodConfig::Vni;
    view_model.enabled = false;
    view_model.linux_frontend = LinuxFrontend::Ibus;
    view_model.commit_mode = CommitMode::Hybrid;

    view_model.apply_to(&mut config);

    assert_eq!(config.input.method, InputMethodConfig::Vni);
    assert!(!config.input.enabled);
    assert_eq!(config.linux.frontend, LinuxFrontend::Ibus);
    assert_eq!(config.linux.commit_mode, CommitMode::Hybrid);
}
