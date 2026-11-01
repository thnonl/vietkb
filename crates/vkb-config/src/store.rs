//! Microsoft Store (MSIX) build: package identity and the pure logic around it.
//!
//! Compiled on every platform so the rules can be unit-tested anywhere; only the
//! `store` feature of `vkb-app` / `vkb-windows` ever calls into the WinRT side.

use std::path::{Path, PathBuf};

/// Package Family Name from Partner Center (`Identity Name` + `_` + publisher id).
/// Fallback only: the app asks Windows for its own family name at runtime.
pub const PACKAGE_FAMILY_NAME: &str = "40278Noamo.VietKB_8eydtcdf0ng4p";

/// `Application Id` in `packaging/msix/AppxManifest.xml`.
pub const APP_ID: &str = "App";

/// `uap5:StartupTask TaskId` in the manifest.
pub const STARTUP_TASK_ID: &str = "VietKBStartup";

/// `Windows.ApplicationModel.StartupTaskState`, by its raw value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupState {
    Disabled,
    DisabledByUser,
    Enabled,
    DisabledByPolicy,
    EnabledByPolicy,
    /// A value this build does not know: treated as off.
    Unknown(i32),
}

impl StartupState {
    pub fn from_raw(raw: i32) -> Self {
        match raw {
            0 => Self::Disabled,
            1 => Self::DisabledByUser,
            2 => Self::Enabled,
            3 => Self::DisabledByPolicy,
            4 => Self::EnabledByPolicy,
            other => Self::Unknown(other),
        }
    }

    /// Windows will start the app at sign-in.
    pub fn is_on(self) -> bool {
        matches!(self, Self::Enabled | Self::EnabledByPolicy)
    }

    /// `RequestEnableAsync` can change this state. A user or policy decision
    /// cannot be undone by the app (the call just returns the same state), so
    /// the app must not keep asking.
    pub fn can_request_enable(self) -> bool {
        matches!(self, Self::Disabled)
    }
}

/// Explorer runs outside the package container, so the real on-disk home of the
/// virtualized `%APPDATA%\<app_dir>` is
/// `%LOCALAPPDATA%\Packages\<PFN>\LocalCache\Roaming\<app_dir>`.
pub fn virtualized_config_dir(local_app_data: &Path, family_name: &str, app_dir: &str) -> PathBuf {
    local_app_data
        .join("Packages")
        .join(family_name)
        .join("LocalCache")
        .join("Roaming")
        .join(app_dir)
}

/// `shell:AppsFolder\<PFN>!<AppId>`: launches the packaged app from Explorer.
pub fn apps_folder_target(family_name: &str) -> String {
    format!(r"shell:AppsFolder\{family_name}!{APP_ID}")
}

/// PowerShell command line (the `-Command` argument) that waits for `pid` to
/// exit (at most 10 s, silent if it is already gone) and then activates the
/// package through the shell.
pub fn relaunch_script(pid: u32, family_name: &str) -> String {
    format!(
        "Wait-Process -Id {pid} -Timeout 10 -ErrorAction SilentlyContinue; Start-Process '{}'",
        apps_folder_target(family_name)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_states_map_to_on_and_off() {
        assert!(StartupState::from_raw(2).is_on(), "Enabled");
        assert!(StartupState::from_raw(4).is_on(), "EnabledByPolicy");
        assert!(!StartupState::from_raw(0).is_on(), "Disabled");
        assert!(!StartupState::from_raw(1).is_on(), "DisabledByUser");
        assert!(!StartupState::from_raw(3).is_on(), "DisabledByPolicy");
        assert!(!StartupState::from_raw(99).is_on(), "unknown is off");
    }

    #[test]
    fn only_a_plain_disabled_task_can_be_enabled_by_the_app() {
        assert!(StartupState::Disabled.can_request_enable());
        assert!(!StartupState::DisabledByUser.can_request_enable());
        assert!(!StartupState::DisabledByPolicy.can_request_enable());
        assert!(!StartupState::Enabled.can_request_enable());
        assert!(!StartupState::Unknown(7).can_request_enable());
    }

    #[test]
    fn virtualized_dir_sits_under_the_package_local_cache() {
        let dir = virtualized_config_dir(
            Path::new("LocalAppData"),
            PACKAGE_FAMILY_NAME,
            "VietKB",
        );
        let expected: PathBuf = [
            "LocalAppData",
            "Packages",
            "40278Noamo.VietKB_8eydtcdf0ng4p",
            "LocalCache",
            "Roaming",
            "VietKB",
        ]
        .iter()
        .collect();
        assert_eq!(dir, expected);
    }

    #[test]
    fn apps_folder_target_uses_the_app_id() {
        assert_eq!(
            apps_folder_target(PACKAGE_FAMILY_NAME),
            r"shell:AppsFolder\40278Noamo.VietKB_8eydtcdf0ng4p!App"
        );
    }

    #[test]
    fn relaunch_script_waits_for_the_pid_then_activates_the_package() {
        assert_eq!(
            relaunch_script(4242, PACKAGE_FAMILY_NAME),
            "Wait-Process -Id 4242 -Timeout 10 -ErrorAction SilentlyContinue; \
             Start-Process 'shell:AppsFolder\\40278Noamo.VietKB_8eydtcdf0ng4p!App'"
        );
    }

    #[test]
    fn identity_constants_match_the_manifest() {
        let manifest = include_str!("../../../packaging/msix/AppxManifest.xml");
        assert!(manifest.contains(&format!("Id=\"{APP_ID}\"")));
        assert!(manifest.contains(&format!("TaskId=\"{STARTUP_TASK_ID}\"")));
        assert!(PACKAGE_FAMILY_NAME.starts_with("40278Noamo.VietKB_"));
        assert!(manifest.contains("Name=\"40278Noamo.VietKB\""));
    }
}
