//! Autostart for the Microsoft Store (MSIX) build: `Windows.ApplicationModel.
//! StartupTask` (manifest task `VietKBStartup`), not the `Run` key.
//!
//! Inside an MSIX package `HKCU\...\Run` is virtualized and Windows never reads
//! it, so none of the registry code is compiled here (feature `store`).
//!
//! `DisabledByUser` / `DisabledByPolicy` cannot be reversed by the app; asking
//! again only returns the same state. Both read as "off" and `set_autostart`
//! still returns `Ok`: the caller reads the real state back, which puts the
//! switch on "off" without an error loop.
//!
//! Every WinRT call runs on a short-lived MTA thread, so it never depends on the
//! apartment of the caller (a Tauri command thread, the backend worker) and the
//! keyboard hook thread is never involved.

use anyhow::{anyhow, bail, Context, Result};
use vkb_config::store::{StartupState, STARTUP_TASK_ID};
use windows::core::HSTRING;
use windows::ApplicationModel::{Package, StartupTask};
use windows::Win32::System::WinRT::{RoInitialize, RO_INIT_MULTITHREADED};

/// Run `work` on its own thread with the WinRT apartment initialised.
fn on_mta<T: Send + 'static>(work: impl FnOnce() -> Result<T> + Send + 'static) -> Result<T> {
    std::thread::Builder::new()
        .name("vkb-winrt".into())
        .spawn(move || {
            // S_FALSE (already initialised) is fine; a thread that is already
            // STA would answer RPC_E_CHANGED_MODE, and the calls still work.
            let _ = unsafe { RoInitialize(RO_INIT_MULTITHREADED) };
            work()
        })
        .context("starting the WinRT worker")?
        .join()
        .map_err(|_| anyhow!("the WinRT worker panicked"))?
}

fn state_of(task: &StartupTask) -> Result<StartupState> {
    Ok(StartupState::from_raw(task.State()?.0))
}

fn task() -> Result<StartupTask> {
    let id = HSTRING::from(STARTUP_TASK_ID);
    Ok(StartupTask::GetAsync(&id)?.get()?)
}

/// Request the startup task on or off. Idempotent.
pub fn set_autostart(enabled: bool) -> Result<()> {
    if vkb_config::paths::running_from_cargo_deps() {
        bail!("refusing to change the startup task from a cargo test binary");
    }
    on_mta(move || {
        let task = task()?;
        let state = state_of(&task)?;
        if enabled {
            if state.is_on() {
                return Ok(());
            }
            if !state.can_request_enable() {
                tracing::warn!(?state, "startup task cannot be enabled by the app");
                return Ok(());
            }
            let after = StartupState::from_raw(task.RequestEnableAsync()?.get()?.0);
            if !after.is_on() {
                tracing::warn!(?after, "startup task was not enabled");
            }
        } else if state == StartupState::Enabled {
            task.Disable()?;
        }
        Ok(())
    })
}

/// The startup task is on (`Enabled` / `EnabledByPolicy`). Any failure reads as off.
pub fn is_autostart_enabled() -> bool {
    let read = on_mta(|| state_of(&task()?));
    match read {
        Ok(state) => state.is_on(),
        Err(error) => {
            tracing::warn!(%error, "could not read the startup task");
            false
        }
    }
}

/// This package's family name as Windows reports it, `None` when the exe runs
/// without package identity (the caller decides whether to fall back to
/// `PACKAGE_FAMILY_NAME`).
pub fn package_family_name() -> Option<String> {
    on_mta(|| Ok(Package::Current()?.Id()?.FamilyName()?.to_string_lossy()))
        .ok()
        .filter(|name| !name.is_empty())
}
