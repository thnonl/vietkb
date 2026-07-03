use vietkb_config::{AppConfig, CommitMode, InputMethodConfig, LinuxFrontend};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsViewModel {
    pub method: InputMethodConfig,
    pub enabled: bool,
    pub linux_frontend: LinuxFrontend,
    pub commit_mode: CommitMode,
}

impl From<&AppConfig> for SettingsViewModel {
    fn from(config: &AppConfig) -> Self {
        Self {
            method: config.input.method,
            enabled: config.input.enabled,
            linux_frontend: config.linux.frontend,
            commit_mode: config.linux.commit_mode,
        }
    }
}

impl SettingsViewModel {
    pub fn apply_to(self, config: &mut AppConfig) {
        config.input.method = self.method;
        config.input.enabled = self.enabled;
        config.linux.frontend = self.linux_frontend;
        config.linux.commit_mode = self.commit_mode;
    }
}
