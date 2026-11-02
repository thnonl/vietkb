//! Update check, download and install.
//!
//! Source of truth is the GitHub Releases API for the public `thnonl/vietkb`
//! repository. Nothing here may block the UI thread or take the app down: every
//! network call runs on a blocking worker, every failure is turned into a
//! readable Vietnamese sentence, and "no releases yet" is a normal answer, not
//! an error.
//!
//! HTTP is `ureq` (rustls, no OpenSSL, no shelling out to curl). It is the only
//! dependency this feature adds.

use std::io::Write;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::i18n::tf;

/// The repository the app ships from.
pub const REPO_URL: &str = "https://github.com/thnonl/vietkb";
const LATEST_API: &str = "https://api.github.com/repos/thnonl/vietkb/releases/latest";
/// GitHub rejects a request without one; it also asks for a versioned Accept.
const USER_AGENT: &str = concat!(
    "VietKB/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/thnonl/vietkb)"
);
const HTTP_TIMEOUT: Duration = Duration::from_secs(20);
const STARTUP_DELAY: Duration = Duration::from_secs(5);
/// How often the running app asks GitHub again after the startup check.
const CHECK_INTERVAL: Duration = Duration::from_secs(12 * 60 * 60);
/// i18n key shown when the release has no installer asset for this platform
/// (no Windows x64 installer, or no `.dmg` on macOS).
#[cfg(not(target_os = "macos"))]
pub const NO_INSTALLER: &str = "err.update.noInstaller";
#[cfg(target_os = "macos")]
pub const NO_INSTALLER: &str = "err.update.noDmg";

// ---------------------------------------------------------------------------
// Wire types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
struct ReleaseJson {
    #[serde(default)]
    tag_name: Option<String>,
    #[serde(default)]
    body: Option<String>,
    #[serde(default)]
    published_at: Option<String>,
    #[serde(default)]
    html_url: Option<String>,
    #[serde(default)]
    assets: Vec<AssetJson>,
}

#[derive(Debug, Clone, Deserialize)]
struct AssetJson {
    #[serde(default)]
    name: String,
    #[serde(default)]
    browser_download_url: String,
    #[serde(default)]
    size: u64,
    /// GitHub's own digest of the asset, `sha256:<hex>`.
    #[serde(default)]
    digest: Option<String>,
}

// ---------------------------------------------------------------------------
// Public types (serialised straight to the page, camelCase)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAsset {
    pub name: String,
    pub url: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    /// Normalised, no `v` prefix (`0.2.0`).
    pub version: String,
    /// The tag exactly as published (`v0.2.0`).
    pub tag: String,
    pub changelog: String,
    pub published_at: Option<String>,
    pub html_url: String,
    pub assets: Vec<UpdateAsset>,
    /// The `*x64-setup.exe` asset, when the release has one.
    pub asset_name: Option<String>,
    pub asset_url: Option<String>,
    pub asset_size: Option<u64>,
    /// The installer's SHA-256 as GitHub reports it (`digest`); never sent to
    /// the page.
    #[serde(skip)]
    pub asset_sha256: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum UpdateState {
    Idle,
    Checking,
    None,
    Available,
    Error,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatus {
    pub state: UpdateState,
    pub current_version: String,
    pub update: Option<UpdateInfo>,
    /// Readable text for the status line: the rate-limit note, the network
    /// error, …
    pub message: Option<String>,
    /// Unix seconds of the last completed check.
    pub checked_at: Option<u64>,
}

impl UpdateStatus {
    fn idle() -> Self {
        Self {
            state: UpdateState::Idle,
            current_version: env!("CARGO_PKG_VERSION").to_string(),
            update: None,
            message: None,
            checked_at: None,
        }
    }

    fn checking() -> Self {
        Self {
            state: UpdateState::Checking,
            current_version: env!("CARGO_PKG_VERSION").to_string(),
            update: None,
            message: None,
            checked_at: None,
        }
    }

    fn none() -> Self {
        Self {
            state: UpdateState::None,
            checked_at: Some(now_secs()),
            ..Self::idle()
        }
    }

    fn available(info: UpdateInfo) -> Self {
        Self {
            state: UpdateState::Available,
            update: Some(info),
            checked_at: Some(now_secs()),
            ..Self::idle()
        }
    }

    fn error(message: impl Into<String>) -> Self {
        Self {
            state: UpdateState::Error,
            message: Some(message.into()),
            checked_at: Some(now_secs()),
            ..Self::idle()
        }
    }
}

/// What a finished download produced.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadOutcome {
    pub path: String,
    pub version: String,
    /// `false` when the release notes carried no `SHA256:` line — the file was
    /// written but its contents were not proven.
    pub verified: bool,
}

#[derive(Debug, Clone)]
struct Downloaded {
    path: std::path::PathBuf,
    version: String,
    verified: bool,
}

// ---------------------------------------------------------------------------
// Shared state
// ---------------------------------------------------------------------------

static STATUS: OnceLock<parking_lot::RwLock<UpdateStatus>> = OnceLock::new();
static DOWNLOADED: OnceLock<parking_lot::Mutex<Option<Downloaded>>> = OnceLock::new();

/// Counts the update operations running (manual check, download, timer tick)
/// and how many have ever begun. The timer takes the gate only when it is idle,
/// and drops its result if anything else began while its request was out.
struct Gate {
    busy: AtomicUsize,
    epoch: AtomicU64,
}

/// Held while an operation runs; released on drop (also on panic/cancel).
struct Guard<'a> {
    gate: &'a Gate,
    started_at: u64,
}

impl Gate {
    const fn new() -> Self {
        Self { busy: AtomicUsize::new(0), epoch: AtomicU64::new(0) }
    }

    /// A manual operation: always allowed, and it supersedes a running tick.
    fn begin(&self) -> Guard<'_> {
        // busy first: a tick's compare_exchange then fails, or it sees the epoch
        // change and `superseded()` drops its result.
        self.busy.fetch_add(1, Ordering::SeqCst);
        self.epoch.fetch_add(1, Ordering::SeqCst);
        Guard { gate: self, started_at: self.epoch.load(Ordering::SeqCst) }
    }

    /// The timer: only when nothing else runs, decided atomically.
    fn try_begin(&self) -> Option<Guard<'_>> {
        self.busy.compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst).ok()?;
        Some(Guard { gate: self, started_at: self.epoch.load(Ordering::SeqCst) })
    }

    fn is_busy(&self) -> bool {
        self.busy.load(Ordering::SeqCst) > 0
    }
}

impl Guard<'_> {
    /// Another operation began after this one did.
    fn superseded(&self) -> bool {
        self.gate.epoch.load(Ordering::SeqCst) != self.started_at
            || self.gate.busy.load(Ordering::SeqCst) > 1
    }
}

impl Drop for Guard<'_> {
    fn drop(&mut self) {
        self.gate.busy.fetch_sub(1, Ordering::SeqCst);
    }
}

static GATE: Gate = Gate::new();

fn state() -> &'static parking_lot::RwLock<UpdateStatus> {
    STATUS.get_or_init(|| parking_lot::RwLock::new(UpdateStatus::idle()))
}

pub fn current() -> UpdateStatus {
    state().read().clone()
}

/// The update, when one is waiting. The tray and the title bar read this.
pub fn available() -> Option<UpdateInfo> {
    state().read().update.clone()
}

pub fn available_version() -> Option<String> {
    state().read().update.as_ref().map(|info| info.version.clone())
}

// ---------------------------------------------------------------------------
// Version comparison
// ---------------------------------------------------------------------------

/// `v0.2.0`, `0.2.0`, `0.2`, `v0.2.0-beta.1` → `(0, 2, 0)`.
fn parse_version(raw: &str) -> Option<(u64, u64, u64)> {
    let trimmed = raw.trim();
    let trimmed = trimmed
        .strip_prefix('v')
        .or_else(|| trimmed.strip_prefix('V'))
        .unwrap_or(trimmed);
    let core = trimmed.split(['-', '+']).next()?.trim();
    if core.is_empty() {
        return None;
    }
    let mut parts = core.split('.');
    let major = parts.next()?.parse::<u64>().ok()?;
    let minor = parts.next().unwrap_or("0").parse::<u64>().ok()?;
    let patch = parts.next().unwrap_or("0").parse::<u64>().ok()?;
    Some((major, minor, patch))
}

fn is_newer(remote: &str, local: &str) -> bool {
    match (parse_version(remote), parse_version(local)) {
        (Some(remote), Some(local)) => remote > local,
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// HTTP
// ---------------------------------------------------------------------------

fn agent() -> ureq::Agent {
    let config = ureq::Agent::config_builder()
        // 404 and 403 are answers we want to read, not transport failures.
        .http_status_as_error(false)
        .timeout_global(Some(HTTP_TIMEOUT))
        .user_agent(USER_AGENT)
        .max_redirects(5)
        .build();
    ureq::Agent::new_with_config(config)
}

fn http_get(agent: &ureq::Agent, url: &str) -> Result<(u16, String), String> {
    let mut response = agent
        .get(url)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .call()
        .map_err(|error| tf("err.update.connect", &[("error", &error.to_string())]))?;
    let status = response.status().as_u16();
    let body = response.body_mut().read_to_string().unwrap_or_default();
    Ok((status, body))
}

/// `VKB_UPDATE_FAKE=1` answers with the built-in stub; a path answers with that
/// file's JSON. Used to prove the badge/menu/changelog without a real release.
fn fake_source() -> Option<String> {
    let value = std::env::var("VKB_UPDATE_FAKE").ok()?;
    let value = value.trim();
    if value.is_empty() || value == "0" {
        return None;
    }
    if value == "1" || value.eq_ignore_ascii_case("true") {
        return Some(FAKE_RELEASE.to_string());
    }
    match std::fs::read_to_string(value) {
        Ok(text) => Some(text),
        Err(error) => {
            tracing::warn!(%error, path = value, "VKB_UPDATE_FAKE is not readable");
            Some(FAKE_RELEASE.to_string())
        }
    }
}

const FAKE_RELEASE: &str = r#"{
  "tag_name": "v0.9.9",
  "name": "VietKB 0.9.9",
  "published_at": "2026-10-20T09:00:00Z",
  "html_url": "https://github.com/thnonl/vietkb/releases/tag/v0.9.9",
  "body": "VietKB 0.9.9 (bản thử cập nhật)\n\n- Kiểm tra cập nhật từ GitHub Releases\n- Huy hiệu cập nhật trên thanh tiêu đề và khay hệ thống\n- Cửa sổ Giới thiệu có nhật ký thay đổi\n- Tải và chạy trình cài đặt ngay trong app\n",
  "assets": [
    { "name": "latest.json", "browser_download_url": "https://github.com/thnonl/vietkb/releases/download/v0.9.9/latest.json", "size": 879 },
    { "name": "VietKB_0.9.9_x64-setup.exe", "browser_download_url": "https://github.com/thnonl/vietkb/releases/download/v0.9.9/VietKB_0.9.9_x64-setup.exe", "size": 3145728 }
  ]
}"#;

fn fetch_latest() -> Result<Option<UpdateInfo>, String> {
    let body = match fake_source() {
        Some(body) => body,
        None => {
            let url = std::env::var("VKB_UPDATE_URL").unwrap_or_else(|_| LATEST_API.to_string());
            let agent = agent();
            let (status, body) = http_get(&agent, &url)?;
            match status {
                200 => body,
                // No release has ever been published: a valid, quiet state.
                404 => return Ok(None),
                403 | 429 => {
                    return Err(tf("err.update.rateLimited", &[("status", &status.to_string())]))
                }
                other => return Err(tf("err.update.http", &[("status", &other.to_string())])),
            }
        }
    };
    parse_release(&body)
}

fn parse_release(body: &str) -> Result<Option<UpdateInfo>, String> {
    parse_release_for(body, cfg!(target_os = "macos"))
}

/// Whether `name` is the installer asset this platform would download: the NSIS
/// setup on Windows, the universal `.dmg` on macOS. `latest.json` and `.sig`
/// files never match either.
fn is_installer_asset(name: &str, macos: bool) -> bool {
    let lower = name.to_ascii_lowercase();
    if macos {
        lower.ends_with(".dmg")
    } else {
        lower.ends_with("x64-setup.exe")
    }
}

fn parse_release_for(body: &str, macos: bool) -> Result<Option<UpdateInfo>, String> {
    let release: ReleaseJson = serde_json::from_str(body)
        .map_err(|error| tf("err.update.parse", &[("error", &error.to_string())]))?;

    let Some(tag) = release.tag_name.as_deref().map(str::trim).filter(|t| !t.is_empty()) else {
        // `{}` or an empty body: nothing published.
        return Ok(None);
    };
    let Some(version) = normalize_version(tag) else {
        return Ok(None);
    };
    if !is_newer(&version, env!("CARGO_PKG_VERSION")) {
        return Ok(None);
    }

    let assets: Vec<UpdateAsset> = release
        .assets
        .iter()
        .filter(|asset| !asset.name.is_empty())
        .map(|asset| UpdateAsset {
            name: asset.name.clone(),
            url: asset.browser_download_url.clone(),
            size: asset.size,
        })
        .collect();

    let installer = assets
        .iter()
        .find(|asset| is_installer_asset(&asset.name, macos));

    Ok(Some(UpdateInfo {
        version: version.clone(),
        tag: tag.to_string(),
        changelog: release.body.unwrap_or_default(),
        published_at: release.published_at,
        html_url: release
            .html_url
            .unwrap_or_else(|| format!("{REPO_URL}/releases/tag/{tag}")),
        asset_name: installer.map(|asset| asset.name.clone()),
        asset_url: installer.map(|asset| asset.url.clone()),
        asset_size: installer.map(|asset| asset.size),
        asset_sha256: release
            .assets
            .iter()
            .find(|raw| Some(raw.name.as_str()) == installer.map(|a| a.name.as_str()))
            .and_then(|raw| raw.digest.as_deref())
            .and_then(digest_sha256),
        assets,
    }))
}

fn normalize_version(tag: &str) -> Option<String> {
    let (major, minor, patch) = parse_version(tag)?;
    Some(format!("{major}.{minor}.{patch}"))
}

// ---------------------------------------------------------------------------
// Check
// ---------------------------------------------------------------------------

/// Blocking check. Always returns a state; never panics, never waits forever.
pub fn run_check() -> UpdateStatus {
    match fetch_latest() {
        Ok(Some(info)) => UpdateStatus::available(info),
        Ok(None) => UpdateStatus::none(),
        Err(message) => {
            tracing::warn!(%message, "update check failed");
            UpdateStatus::error(message)
        }
    }
}

/// Store the result, refresh the tray and tell every open window.
pub fn publish(app: &tauri::AppHandle, status: UpdateStatus) {
    // One line per outcome: "none" (up to date or no release), "available" with
    // the version, or "error" with the sentence the page shows.
    tracing::debug!(
        state = ?status.state,
        version = status.update.as_ref().map(|info| info.version.as_str()).unwrap_or("-"),
        message = status.message.as_deref().unwrap_or("-"),
        "update state"
    );
    // A downloaded installer is only good for the version it was fetched for.
    if matches!(status.state, UpdateState::Available | UpdateState::None) {
        let wanted = status.update.as_ref().map(|info| info.version.as_str());
        if let Some(cell) = DOWNLOADED.get() {
            let mut held = cell.lock();
            if held.as_ref().is_some_and(|d| Some(d.version.as_str()) != wanted) {
                *held = None;
            }
        }
    }
    *state().write() = status.clone();
    crate::tray::refresh(app, &crate::commands::status_now());
    use tauri::Emitter;
    if let Err(error) = app.emit("vkb:update", status) {
        tracing::warn!(%error, "could not broadcast the update state");
    }
}

/// What one timer tick should do. Pure, so it is testable without a network.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tick {
    /// `system.check_updates` is off: do nothing, ask again next tick.
    SkipDisabled,
    /// A manual check or a download is running: leave it alone.
    SkipBusy,
    Check,
}

fn decide_tick(check_updates: bool, in_flight: bool) -> Tick {
    if !check_updates {
        Tick::SkipDisabled
    } else if in_flight {
        Tick::SkipBusy
    } else {
        Tick::Check
    }
}

/// Whether a periodic result replaces the stored status. A failed tick (offline,
/// 403/429) must not erase a known update, so an error is kept out of the store
/// while one is held; it is published only when there is nothing to lose.
fn should_publish(prev: &UpdateStatus, new: &UpdateStatus) -> bool {
    match new.state {
        UpdateState::Available | UpdateState::None => true,
        UpdateState::Error => prev.update.is_none(),
        UpdateState::Idle | UpdateState::Checking => false,
    }
}

/// One tick: decide, then (maybe) run the check through the same
/// `run_check` + `publish` path as the manual button. The next tick simply
/// tries again after an error (403/429 included): no retry in between.
fn run_tick(app: &tauri::AppHandle) {
    let enabled = crate::commands::config_cell().read().system.check_updates;
    let busy = GATE.is_busy();
    let decision = decide_tick(enabled, busy);
    if decision != Tick::Check {
        tracing::debug!(?decision, "update tick skipped");
        return;
    }
    let Some(guard) = GATE.try_begin() else {
        tracing::debug!("update tick skipped: an operation began first");
        return;
    };
    let status = run_check();
    if guard.superseded() {
        tracing::debug!("update tick dropped: a manual operation began meanwhile");
        return;
    }
    if status.state == UpdateState::Error {
        tracing::warn!(
            message = status.message.as_deref().unwrap_or("-"),
            "periodic update check failed"
        );
    }
    if !should_publish(&current(), &status) {
        return;
    }
    if status.state == UpdateState::Available {
        tracing::info!(
            version = status.update.as_ref().map(|i| i.version.as_str()).unwrap_or(""),
            "update available"
        );
    }
    publish(app, status);
}

/// Longest single sleep. The deadline is wall-clock, so a laptop that slept
/// through it checks within one slice of waking.
const SLEEP_SLICE: Duration = Duration::from_secs(15 * 60);

/// How long to sleep before the next look at the clock, or `None` when the
/// deadline has passed. A clock moved far backwards (remaining beyond one whole
/// interval) counts as due rather than waiting out the skew.
fn sleep_slice(now: SystemTime, deadline: SystemTime) -> Option<Duration> {
    match deadline.duration_since(now) {
        Ok(left) if left <= CHECK_INTERVAL && !left.is_zero() => Some(left.min(SLEEP_SLICE)),
        _ => None,
    }
}

fn sleep_until(deadline: SystemTime) {
    while let Some(slice) = sleep_slice(SystemTime::now(), deadline) {
        std::thread::sleep(slice);
    }
}

/// One named thread for the life of the app: a check five seconds after
/// launch, then one every `CHECK_INTERVAL` of real time. The setting is read at
/// each tick, so toggling it needs no restart. A panicking tick is logged and
/// the timer carries on in debug and test builds only: release builds use
/// `panic = "abort"`, where `catch_unwind` cannot catch. Never touches the hook or UI threads.
pub fn schedule_startup_check(app: tauri::AppHandle) {
    let spawned = std::thread::Builder::new().name("vkb-update-timer".into()).spawn(move || {
        sleep_until(SystemTime::now() + STARTUP_DELAY);
        loop {
            let tick = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run_tick(&app)));
            if tick.is_err() {
                tracing::error!("update timer tick panicked; it will run again next interval");
            }
            sleep_until(SystemTime::now() + CHECK_INTERVAL);
        }
    });
    if let Err(error) = spawned {
        tracing::warn!(%error, "could not start the update timer");
    }
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn check_update(app: tauri::AppHandle) -> UpdateStatus {
    let _busy = GATE.begin();
    // Paint `Đang kiểm tra…` before the request goes out.
    publish(&app, UpdateStatus::checking());
    let status = tauri::async_runtime::spawn_blocking(run_check)
        .await
        .unwrap_or_else(|error| UpdateStatus::error(tf("err.update.checkFailed", &[("error", &error.to_string())])));
    publish(&app, status.clone());
    status
}

#[tauri::command]
pub fn update_status() -> UpdateStatus {
    current()
}

#[tauri::command]
pub async fn download_update() -> Result<DownloadOutcome, String> {
    let _busy = GATE.begin();
    let Some(info) = available() else {
        return Err(crate::i18n::t("err.update.nothingToDownload"));
    };
    let Some(url) = info.asset_url.clone() else {
        return Err(crate::i18n::t(NO_INSTALLER));
    };
    let name = info
        .asset_name
        .clone()
        .unwrap_or_else(|| format!("VietKB_{}_x64-setup.exe", info.version));
    let changelog = info.changelog.clone();
    let digest = info.asset_sha256.clone();

    let outcome = tauri::async_runtime::spawn_blocking(move || {
        download_to_temp(&url, &name, &changelog, digest.as_deref(), &info.version)
    })
    .await
    .unwrap_or_else(|error| Err(tf("err.update.downloadFailed", &[("error", &error.to_string())])))?;

    // A check may have published a newer version while this one downloaded.
    if available_version().as_deref() != Some(outcome.version.as_str()) {
        return Err(crate::i18n::t("err.update.notDownloaded"));
    }
    *DOWNLOADED
        .get_or_init(|| parking_lot::Mutex::new(None))
        .lock() = Some(outcome.clone());

    Ok(DownloadOutcome {
        path: outcome.path.display().to_string(),
        version: outcome.version,
        verified: outcome.verified,
    })
}

/// Flags handed to the NSIS installer.
///
/// `/S` runs it without any page, so `Cập nhật ngay` really does install by
/// itself. `/UPDATE` is Tauri's in-place mode: the reinstall page skips the
/// "uninstall before installing" branch, the old uninstaller never runs, and
/// WebView2/shortcut work is skipped — files are simply overwritten.
#[cfg(any(windows, test))]
const INSTALLER_ARGS: [&str; 2] = ["/S", "/UPDATE"];

/// The batch the app hands itself to before quitting: run the setup, then bring
/// VietKB back. The installer kills this process (silent mode, Restart Manager),
/// so nothing inside the app can do the relaunch itself.
#[cfg(any(windows, test))]
fn update_script(installer: &std::path::Path, exe: &std::path::Path) -> String {
    let setup = installer.display();
    let app = exe.display();
    format!(
        "@echo off\r\n\
         \"{setup}\" {args}\r\n\
         if exist \"{app}\" start \"\" \"{app}\"\r\n",
        args = INSTALLER_ARGS.join(" ")
    )
}

/// macOS: replace the running bundle with the one inside the downloaded `.dmg`,
/// then relaunch. Returns `"installing"` once the helper script is running (the
/// app quits right after), or `"page"` when this install cannot be replaced in
/// place and the release page was opened instead. Everything the script needs is
/// passed as arguments: no path is ever spliced into the script text.
#[cfg(target_os = "macos")]
#[tauri::command]
pub fn install_update(app: tauri::AppHandle) -> Result<&'static str, String> {
    let downloaded = DOWNLOADED
        .get_or_init(|| parking_lot::Mutex::new(None))
        .lock()
        .clone()
        .ok_or_else(|| crate::i18n::t("err.update.notDownloaded"))?;
    if available_version().as_deref() != Some(downloaded.version.as_str()) {
        return Err(crate::i18n::t("err.update.notDownloaded"));
    }
    if !downloaded.path.exists() {
        return Err(crate::i18n::t("err.update.installerGone"));
    }

    let exe = std::env::current_exe()
        .map_err(|error| tf("err.update.exePath", &[("error", &error.to_string())]))?;
    let Some(bundle) = bundle_from_exe(&exe) else {
        return Err(crate::i18n::t("err.update.macNotBundle"));
    };
    let writable = bundle.parent().is_some_and(dir_is_writable);
    if let Some(reason) = fallback_reason(&bundle, writable) {
        tracing::info!(?reason, bundle = %bundle.display(), "in-place update not possible; opening the release page");
        open_release_page(&app)?;
        return Ok("page");
    }

    let dir = download_dir();
    let script = dir.join("apply-update.sh");
    std::fs::write(&script, MAC_UPDATE_SCRIPT)
        .map_err(|error| tf("err.update.writeScript", &[("error", &error.to_string())]))?;
    let log = dir.join("apply-update.log");
    let args = mac_script_args(std::process::id(), &downloaded.path, &bundle, &downloaded.version, &log);

    // Its own process group and no stdio: it outlives this process, which
    // exits below, and reopens VietKB once the bundle is replaced.
    use std::os::unix::process::CommandExt;
    std::process::Command::new("/bin/sh")
        .arg(&script)
        .args(&args)
        .current_dir(std::env::temp_dir())
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .process_group(0)
        .spawn()
        .map_err(|error| tf("err.update.runInstaller", &[("error", &error.to_string())]))?;

    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(300));
        crate::quit(&app);
    });
    Ok("installing")
}

#[cfg(target_os = "macos")]
fn open_release_page(app: &tauri::AppHandle) -> Result<(), String> {
    let url = available()
        .map(|info| info.html_url)
        .filter(|url| url.starts_with(REPO_URL))
        .unwrap_or_else(|| format!("{REPO_URL}/releases/latest"));
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|error| tf("err.update.openPage", &[("error", &error.to_string())]))
}

/// `.../VietKB.app/Contents/MacOS/vietkb` -> `.../VietKB.app`; `None` for
/// anything that is not inside a `.app` bundle (a `cargo run` binary, say).
#[cfg(any(target_os = "macos", test))]
fn bundle_from_exe(exe: &std::path::Path) -> Option<std::path::PathBuf> {
    let macos_dir = exe.parent()?;
    let contents = macos_dir.parent()?;
    let bundle = contents.parent()?;
    let is_bundle = macos_dir.file_name()? == "MacOS"
        && contents.file_name()? == "Contents"
        && bundle.extension().is_some_and(|ext| ext == "app");
    is_bundle.then(|| bundle.to_path_buf())
}

/// Why the bundle cannot be swapped under the running app.
#[cfg(any(target_os = "macos", test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Fallback {
    /// Gatekeeper's App Translocation copy: a read-only, randomised path, not
    /// the bundle the user installed.
    Translocated,
    /// The folder holding the bundle cannot be written (the mounted `.dmg`, a
    /// read-only volume, an admin-owned `/Applications`).
    NotWritable,
}

#[cfg(any(target_os = "macos", test))]
fn is_translocated(bundle: &std::path::Path) -> bool {
    bundle.components().any(|part| part.as_os_str() == "AppTranslocation")
}

#[cfg(any(target_os = "macos", test))]
fn fallback_reason(bundle: &std::path::Path, parent_writable: bool) -> Option<Fallback> {
    if is_translocated(bundle) {
        Some(Fallback::Translocated)
    } else if !parent_writable {
        Some(Fallback::NotWritable)
    } else {
        None
    }
}

/// Whether a file can be created in `dir` (a read-only volume answers no even
/// when the mode bits say yes).
#[cfg(target_os = "macos")]
fn dir_is_writable(dir: &std::path::Path) -> bool {
    let probe = dir.join(format!(".vietkb-write-probe-{}", std::process::id()));
    match std::fs::File::create(&probe) {
        Ok(_) => {
            let _ = std::fs::remove_file(&probe);
            true
        }
        Err(_) => false,
    }
}

/// The helper's arguments. Paths travel as separate `argv` entries, so spaces
/// and quotes in them cannot change what the shell runs.
#[cfg(any(target_os = "macos", test))]
fn mac_script_args(
    pid: u32,
    dmg: &std::path::Path,
    bundle: &std::path::Path,
    version: &str,
    log: &std::path::Path,
) -> Vec<std::ffi::OsString> {
    vec![
        pid.to_string().into(),
        dmg.as_os_str().to_owned(),
        bundle.as_os_str().to_owned(),
        version.into(),
        MAC_BUNDLE_ID.into(),
        log.as_os_str().to_owned(),
    ]
}

#[cfg(any(target_os = "macos", test))]
const MAC_BUNDLE_ID: &str = "vn.vietkb.app";

/// Static text, nothing interpolated. Usage:
/// `apply-update.sh PID DMG BUNDLE VERSION BUNDLE_ID LOG`.
#[cfg(any(target_os = "macos", test))]
const MAC_UPDATE_SCRIPT: &str = include_str!("apply-update.sh");

/// Launch the installer that `download_update` wrote, then leave cleanly.
#[cfg(not(target_os = "macos"))]
#[tauri::command]
pub fn install_update(app: tauri::AppHandle) -> Result<(), String> {
    let downloaded = DOWNLOADED
        .get_or_init(|| parking_lot::Mutex::new(None))
        .lock()
        .clone()
        .ok_or_else(|| crate::i18n::t("err.update.notDownloaded"))?;

    // Never run an installer that is not for the update currently on offer.
    if available_version().as_deref() != Some(downloaded.version.as_str()) {
        return Err(crate::i18n::t("err.update.notDownloaded"));
    }

    if !downloaded.path.exists() {
        return Err(crate::i18n::t("err.update.installerGone"));
    }

    #[cfg(windows)]
    {
        let exe = std::env::current_exe()
            .map_err(|error| tf("err.update.exePath", &[("error", &error.to_string())]))?;
        let script_path = download_dir().join("apply-update.cmd");
        std::fs::write(&script_path, update_script(&downloaded.path, &exe))
            .map_err(|error| tf("err.update.writeScript", &[("error", &error.to_string())]))?;

        // Detached from this process, so it survives the quit below and can
        // start the new build once the installer is done.
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        use std::os::windows::process::CommandExt;
        std::process::Command::new("cmd.exe")
            .arg("/C")
            .arg(&script_path)
            .current_dir(std::env::temp_dir())
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|error| tf("err.update.runInstaller", &[("error", &error.to_string())]))?;

        // Let `cmd` reach the setup before the hook and the single-instance
        // lock are released; the installer then replaces the files.
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(300));
            crate::quit(&app);
        });
        Ok(())
    }
    #[cfg(not(windows))]
    {
        let _ = app;
        Err(crate::i18n::t("err.update.winOnly"))
    }
}

/// Open one of the two links the About window shows. Anything else is refused:
/// the page must not be able to hand arbitrary URLs to the shell.
#[tauri::command]
pub fn open_url(app: tauri::AppHandle, url: String) -> Result<(), String> {
    const ALLOWED: [&str; 3] = [
        "https://github.com/thnonl/vietkb",
        "https://ko-fi.com/N4X728S6TT",
        "https://github.com/thnonl/vietkb/issues/new",
    ];
    let trimmed = url.trim();
    if !ALLOWED.iter().any(|allowed| trimmed.starts_with(allowed)) {
        return Err(crate::i18n::t("err.update.linkBlocked"));
    }
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_url(trimmed, None::<&str>)
        .map_err(|error| tf("err.update.openLink", &[("error", &error.to_string())]))
}

// ---------------------------------------------------------------------------
// Download
// ---------------------------------------------------------------------------

fn download_dir() -> std::path::PathBuf {
    std::env::temp_dir().join("VietKB-update")
}

/// The installer name comes from a remote JSON: keep it a plain file name.
fn sanitize_file_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_') { ch } else { '_' })
        .collect();
    let cleaned = cleaned.trim_matches('.');
    if cleaned.is_empty() {
        "VietKB-update_x64-setup.exe".to_string()
    } else {
        cleaned.to_string()
    }
}

fn download_to_temp(
    url: &str,
    name: &str,
    changelog: &str,
    digest: Option<&str>,
    version: &str,
) -> Result<Downloaded, String> {
    let dir = download_dir();
    std::fs::create_dir_all(&dir)
        .map_err(|error| tf("err.update.mkdir", &[("error", &error.to_string())]))?;

    let file_name = sanitize_file_name(name);
    let part = dir.join(format!("{file_name}.part"));
    let target = dir.join(&file_name);

    let agent = agent();
    let mut response = agent
        .get(url)
        .header("Accept", "application/octet-stream")
        .call()
        .map_err(|error| tf("err.update.downloadFailed", &[("error", &error.to_string())]))?;
    let status = response.status().as_u16();
    if status != 200 {
        return Err(tf("err.update.serverDownload", &[("status", &status.to_string())]));
    }

    let mut file =
        std::fs::File::create(&part)
        .map_err(|error| tf("err.update.writeFile", &[("error", &error.to_string())]))?;
    let copied = {
        let mut reader = response.body_mut().as_reader();
        std::io::copy(&mut reader, &mut file)
    };
    file.flush().ok();
    drop(file);
    if let Err(error) = copied {
        let _ = std::fs::remove_file(&part);
        return Err(tf("err.update.interrupted", &[("error", &error.to_string())]));
    }
    if let Err(error) = std::fs::rename(&part, &target) {
        let _ = std::fs::remove_file(&part);
        return Err(tf("err.update.finalize", &[("error", &error.to_string())]));
    }

    let expected = if cfg!(target_os = "macos") {
        expected_sha256_mac(digest, changelog, name)
    } else {
        expected_sha256(changelog)
    };
    let verified = match expected {
        Some(expected) => {
            let bytes = std::fs::read(&target)
                .map_err(|error| tf("err.update.reread", &[("error", &error.to_string())]))?;
            let actual = sha256_hex(&bytes);
            if actual != expected {
                let _ = std::fs::remove_file(&target);
                return Err(crate::i18n::t("err.update.shaMismatch"));
            }
            true
        }
        None => false,
    };

    Ok(Downloaded {
        path: target,
        version: version.to_string(),
        verified,
    })
}

/// A line `SHA256: <hex>` in the release notes, if the maintainer wrote one.
fn expected_sha256(changelog: &str) -> Option<String> {
    for line in changelog.lines() {
        let trimmed = line.trim().trim_start_matches(['*', '-', '#', ' ']).trim();
        let without_backticks = trimmed.replace(['`', '*'], "");
        let Some((label, value)) = without_backticks.split_once(':') else {
            continue;
        };
        if !label.trim().eq_ignore_ascii_case("sha256") {
            continue;
        }
        let hex = value.trim().trim_matches(['"', '\'', ' ']).to_ascii_lowercase();
        if hex.len() == 64 && hex.chars().all(|ch| ch.is_ascii_hexdigit()) {
            return Some(hex);
        }
    }
    None
}

/// `sha256:<hex>` (GitHub's asset `digest`) -> the lowercase hex.
fn digest_sha256(digest: &str) -> Option<String> {
    let hex = digest.trim().strip_prefix("sha256:")?.to_ascii_lowercase();
    (hex.len() == 64 && hex.chars().all(|ch| ch.is_ascii_hexdigit())).then_some(hex)
}

/// What the `.dmg` must hash to. GitHub's own digest of that very asset wins.
/// Failing that, only a note line that names the file counts: the notes may
/// carry the Windows installer's bare `SHA256:` line, which says nothing about
/// the dmg and must never be compared against it.
fn expected_sha256_mac(digest: Option<&str>, changelog: &str, asset: &str) -> Option<String> {
    if let Some(hex) = digest.and_then(digest_sha256) {
        return Some(hex);
    }
    let asset = asset.to_ascii_lowercase();
    changelog
        .lines()
        .filter(|line| line.to_ascii_lowercase().contains(&asset))
        .find_map(|line| {
            line.split(|ch: char| !ch.is_ascii_hexdigit())
                .find(|token| token.len() == 64)
                .map(str::to_ascii_lowercase)
        })
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// SHA-256 (FIPS 180-4). Hand-rolled: the crate budget for this feature is the
// HTTP client only.
// ---------------------------------------------------------------------------

const SHA256_K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

fn sha256_hex(data: &[u8]) -> String {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];

    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut message = data.to_vec();
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in message.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (index, word) in chunk.chunks_exact(4).enumerate() {
            w[index] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for index in 16..64 {
            let s0 = w[index - 15].rotate_right(7)
                ^ w[index - 15].rotate_right(18)
                ^ (w[index - 15] >> 3);
            let s1 = w[index - 2].rotate_right(17)
                ^ w[index - 2].rotate_right(19)
                ^ (w[index - 2] >> 10);
            w[index] = w[index - 16]
                .wrapping_add(s0)
                .wrapping_add(w[index - 7])
                .wrapping_add(s1);
        }

        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
            (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);

        for index in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(SHA256_K[index])
                .wrapping_add(w[index]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }

    h.iter().map(|word| format!("{word:08x}")).collect()
}

// ---------------------------------------------------------------------------
// Tests (no network: the parser and the digest only)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_is_skipped_when_the_setting_is_off() {
        assert_eq!(decide_tick(false, false), Tick::SkipDisabled);
        assert_eq!(decide_tick(false, true), Tick::SkipDisabled);
    }

    #[test]
    fn tick_is_skipped_while_a_check_or_download_runs() {
        assert_eq!(decide_tick(true, true), Tick::SkipBusy);
        assert_eq!(decide_tick(true, false), Tick::Check);
    }

    #[test]
    fn gate_try_begin_is_exclusive_and_releases() {
        let gate = Gate::new();
        let first = gate.try_begin().expect("idle gate");
        assert!(gate.try_begin().is_none());
        assert!(gate.is_busy());
        drop(first);
        assert!(!gate.is_busy());
        assert!(gate.try_begin().is_some());
    }

    #[test]
    fn a_manual_operation_supersedes_a_running_tick() {
        let gate = Gate::new();
        let tick = gate.try_begin().unwrap();
        assert!(!tick.superseded());
        let manual = gate.begin();
        assert!(tick.superseded());
        drop(manual);
        // Even after the manual one finished, the tick's result stays dropped.
        assert!(tick.superseded());
    }

    fn known_update() -> UpdateStatus {
        UpdateStatus::available(parse_release(FAKE_RELEASE).unwrap().expect("fake release"))
    }

    #[test]
    fn a_failed_tick_never_erases_a_known_update() {
        let known = known_update();
        let error = UpdateStatus::error("offline");
        assert!(!should_publish(&known, &error));
        // Nothing to lose: the reason may be shown.
        assert!(should_publish(&UpdateStatus::none(), &error));
        // Real answers always replace.
        assert!(should_publish(&known, &UpdateStatus::none()));
        assert!(should_publish(&UpdateStatus::none(), &known));
    }

    #[test]
    fn sleep_slices_follow_the_wall_clock() {
        let now = SystemTime::now();
        let hour = Duration::from_secs(3600);
        assert_eq!(sleep_slice(now, now + hour), Some(SLEEP_SLICE));
        assert_eq!(sleep_slice(now, now + Duration::from_secs(60)), Some(Duration::from_secs(60)));
        // Woke after the deadline: due now.
        assert_eq!(sleep_slice(now + hour, now), None);
        assert_eq!(sleep_slice(now, now), None);
        // Clock jumped back beyond a whole interval: treat as due.
        assert_eq!(sleep_slice(now, now + CHECK_INTERVAL + hour), None);
    }

    #[test]
    fn versions_parse_and_order_like_semver() {
        assert_eq!(parse_version("0.1.0"), Some((0, 1, 0)));
        assert_eq!(parse_version("v0.2.0"), Some((0, 2, 0)));
        assert_eq!(parse_version("V1.2.3"), Some((1, 2, 3)));
        assert_eq!(parse_version("0.2"), Some((0, 2, 0)));
        assert_eq!(parse_version("0.2.0-beta.1"), Some((0, 2, 0)));
        assert_eq!(parse_version("nope"), None);

        assert!(is_newer("0.1.1", "0.1.0"));
        assert!(is_newer("0.2.0", "0.1.9"));
        assert!(is_newer("v0.2.0", "0.1.0"));
        assert!(!is_newer("0.1.0", "0.1.0"));
        // The only published release (v0.1.1) is older than this 0.2.0 build.
        assert!(!is_newer("0.1.1", "0.2.0"));
        assert!(!is_newer("0.0.9", "0.1.0"));
    }

    #[test]
    fn normalises_the_tag_prefix() {
        assert_eq!(normalize_version("v0.2.0").as_deref(), Some("0.2.0"));
        assert_eq!(normalize_version("0.2.0").as_deref(), Some("0.2.0"));
    }

    #[test]
    fn picks_the_x64_setup_asset() {
        let body = r#"{
            "tag_name": "v99.0.0",
            "body": "- nhanh hơn",
            "published_at": "2026-01-02T03:04:05Z",
            "html_url": "https://github.com/thnonl/vietkb/releases/tag/v99.0.0",
            "assets": [
                {"name": "latest.json", "browser_download_url": "https://x/latest.json", "size": 12},
                {"name": "VietKB_99.0.0_X64-SETUP.EXE", "browser_download_url": "https://x/setup.exe", "size": 42}
            ]
        }"#;
        let info = parse_release_for(body, false).expect("parses").expect("is newer");
        assert_eq!(info.version, "99.0.0");
        assert_eq!(info.tag, "v99.0.0");
        assert_eq!(info.changelog, "- nhanh hơn");
        assert_eq!(info.asset_name.as_deref(), Some("VietKB_99.0.0_X64-SETUP.EXE"));
        assert_eq!(info.asset_size, Some(42));
        assert_eq!(info.assets.len(), 2);
    }

    #[test]
    fn an_older_or_equal_release_is_not_an_update() {
        let body = r#"{"tag_name":"v0.0.1","assets":[]}"#;
        assert!(parse_release(body).expect("parses").is_none());
    }

    #[test]
    fn an_empty_response_is_not_an_error() {
        assert!(parse_release("{}").expect("parses").is_none());
    }

    #[test]
    fn reads_the_sha256_line_from_the_notes() {
        let notes = "VietKB 0.2.0\n\n- fix\nSHA256: 20EA466949DF56C5912940BC8DF021FF6C3A545CCCEC5E43DE4EC95BA492AC71\n";
        assert_eq!(
            expected_sha256(notes).as_deref(),
            Some("20ea466949df56c5912940bc8df021ff6c3a545cccec5e43de4ec95ba492ac71")
        );
        assert_eq!(expected_sha256("no digest here"), None);
        assert_eq!(expected_sha256("SHA256: nothex"), None);
    }

    #[test]
    fn sha256_matches_fips_vectors() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            sha256_hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
    }

    #[test]
    fn installer_names_stay_plain_file_names() {
        assert_eq!(
            sanitize_file_name("VietKB_0.2.0_x64-setup.exe"),
            "VietKB_0.2.0_x64-setup.exe"
        );
        assert_eq!(sanitize_file_name("../../evil.exe"), "_.._evil.exe");
        assert_eq!(sanitize_file_name(""), "VietKB-update_x64-setup.exe");
    }

    #[test]
    fn the_missing_installer_message_is_the_promised_one() {
        #[cfg(not(target_os = "macos"))]
        assert_eq!(
            crate::i18n::tr(crate::i18n::Lang::Vi, NO_INSTALLER),
            "Bản phát hành không có bộ cài cho Windows x64"
        );
        #[cfg(target_os = "macos")]
        assert_eq!(
            crate::i18n::tr(crate::i18n::Lang::Vi, NO_INSTALLER),
            "Bản phát hành không có tệp .dmg cho macOS"
        );
    }

    #[test]
    fn macos_picks_the_dmg_and_ignores_the_windows_setup() {
        let body = r#"{
            "tag_name": "v99.0.0",
            "html_url": "https://github.com/thnonl/vietkb/releases/tag/v99.0.0",
            "assets": [
                {"name": "latest.json", "browser_download_url": "https://x/latest.json", "size": 12},
                {"name": "VietKB_99.0.0_x64-setup.exe", "browser_download_url": "https://x/setup.exe", "size": 42},
                {"name": "VietKB_99.0.0_universal.dmg", "browser_download_url": "https://x/a.dmg", "size": 77}
            ]
        }"#;
        let mac = parse_release_for(body, true).expect("parses").expect("newer");
        assert_eq!(mac.asset_name.as_deref(), Some("VietKB_99.0.0_universal.dmg"));
        let win = parse_release_for(body, false).expect("parses").expect("newer");
        assert_eq!(win.asset_name.as_deref(), Some("VietKB_99.0.0_x64-setup.exe"));
        // The version comparison itself is platform-independent.
        assert_eq!(mac.version, win.version);
    }

    #[test]
    fn the_update_installs_silently_and_in_place() {
        // `/S` = no pages at all; `/UPDATE` = Tauri's in-place branch, which
        // never runs the old uninstaller.
        assert_eq!(INSTALLER_ARGS, ["/S", "/UPDATE"]);
    }

    #[test]
    fn the_update_script_runs_the_setup_then_reopens_the_app() {
        let script = update_script(
            std::path::Path::new(r"C:\Users\me\AppData\Local\Temp\VietKB-update\VietKB_0.3.0_x64-setup.exe"),
            std::path::Path::new(r"C:\Users\me\AppData\Local\VietKB\vietkb.exe"),
        );
        let setup = script.lines().nth(1).expect("setup line");
        assert_eq!(
            setup,
            r#""C:\Users\me\AppData\Local\Temp\VietKB-update\VietKB_0.3.0_x64-setup.exe" /S /UPDATE"#
        );
        let relaunch = script.lines().nth(2).expect("relaunch line");
        assert_eq!(
            relaunch,
            r#"if exist "C:\Users\me\AppData\Local\VietKB\vietkb.exe" start "" "C:\Users\me\AppData\Local\VietKB\vietkb.exe""#
        );
        assert!(script.starts_with("@echo off"));
    }

    #[test]
    fn a_release_without_an_installer_has_no_asset() {
        // `latest.json` and the `.sig` must never be picked as the download.
        let body = r#"{"tag_name":"v9.9.9","assets":[
            {"name":"latest.json","browser_download_url":"https://x/latest.json","size":879},
            {"name":"VietKB_9.9.9_x64-setup.exe.sig","browser_download_url":"https://x/a.sig","size":755}]}"#;
        let info = parse_release_for(body, false).expect("parses").expect("newer");
        assert!(info.asset_url.is_none());
        assert!(info.asset_name.is_none());
    }

    #[test]
    fn bundle_is_derived_from_the_executable_path() {
        use std::path::Path;
        assert_eq!(
            bundle_from_exe(Path::new("/Applications/VietKB.app/Contents/MacOS/vietkb")),
            Some(Path::new("/Applications/VietKB.app").to_path_buf())
        );
        assert_eq!(
            bundle_from_exe(Path::new("/Users/me/My Apps/Viet KB.app/Contents/MacOS/vietkb")),
            Some(Path::new("/Users/me/My Apps/Viet KB.app").to_path_buf())
        );
        // A dev run, and look-alikes, are not bundles.
        assert_eq!(bundle_from_exe(Path::new("/repo/target/debug/vietkb")), None);
        assert_eq!(bundle_from_exe(Path::new("/Applications/VietKB.app/vietkb")), None);
        assert_eq!(bundle_from_exe(Path::new("/x/VietKB/Contents/MacOS/vietkb")), None);
        assert_eq!(bundle_from_exe(Path::new("/x/VietKB.app/Contents/Other/vietkb")), None);
        assert_eq!(bundle_from_exe(Path::new("vietkb")), None);
    }

    #[test]
    fn translocated_or_unwritable_bundles_fall_back_to_the_page() {
        use std::path::Path;
        let app = Path::new("/Applications/VietKB.app");
        assert_eq!(fallback_reason(app, true), None);
        assert_eq!(fallback_reason(app, false), Some(Fallback::NotWritable));
        // Running from the mounted dmg: the volume is read-only.
        assert_eq!(
            fallback_reason(Path::new("/Volumes/VietKB/VietKB.app"), false),
            Some(Fallback::NotWritable)
        );
        let moved = Path::new("/private/var/folders/ab/cd_ef/T/AppTranslocation/9A1B-22/d/VietKB.app");
        assert!(is_translocated(moved));
        // Translocation wins even if the probe says writable.
        assert_eq!(fallback_reason(moved, true), Some(Fallback::Translocated));
        assert!(!is_translocated(Path::new("/Users/me/AppTranslocationNotes/VietKB.app")));
    }

    #[test]
    fn script_paths_travel_as_single_arguments() {
        use std::path::Path;
        let args = mac_script_args(
            4242,
            Path::new("/tmp/Viet KB update/VietKB_0.3.0_universal.dmg"),
            Path::new("/Users/me/My Apps/Viet'KB \"x\" $(id).app"),
            "0.3.0",
            Path::new("/tmp/Viet KB update/apply-update.log"),
        );
        let args: Vec<String> = args.iter().map(|a| a.to_string_lossy().into_owned()).collect();
        assert_eq!(
            args,
            [
                "4242",
                "/tmp/Viet KB update/VietKB_0.3.0_universal.dmg",
                "/Users/me/My Apps/Viet'KB \"x\" $(id).app",
                "0.3.0",
                "vn.vietkb.app",
                "/tmp/Viet KB update/apply-update.log",
            ]
        );
    }

    #[test]
    fn the_mac_script_is_static_and_reads_its_arguments_once() {
        assert!(MAC_UPDATE_SCRIPT.starts_with("#!/bin/sh"));
        for needle in [
            "kill -0",
            "hdiutil attach -nobrowse -readonly",
            "CFBundleIdentifier",
            "CFBundleShortVersionString",
            "ditto",
            "xattr -dr com.apple.quarantine",
            "hdiutil detach",
            "open \"$BUNDLE\"",
        ] {
            assert!(MAC_UPDATE_SCRIPT.contains(needle), "script lacks {needle}");
        }
        // Positional arguments are only ever copied into variables.
        for line in MAC_UPDATE_SCRIPT
            .lines()
            .filter(|l| !l.trim_start().starts_with('#'))
            .filter(|l| l.contains("$1") || l.contains("$2") || l.contains("$3"))
        {
            assert!(line.contains('='), "positional argument used directly: {line}");
        }
    }

    #[test]
    fn mac_digest_comes_from_github_or_a_line_naming_the_dmg() {
        let hex = "ab".repeat(32);
        let other = "cd".repeat(32);
        // GitHub's digest of the asset wins.
        assert_eq!(
            expected_sha256_mac(Some(&format!("sha256:{}", hex.to_uppercase())), "SHA256: nothing", "a.dmg"),
            Some(hex.clone())
        );
        // The Windows installer's bare line is never taken for the dmg.
        assert_eq!(expected_sha256_mac(None, &format!("SHA256: {other}"), "a.dmg"), None);
        let notes = format!("- SHA256: {other}  VietKB_1_x64-setup.exe\n- SHA256: {hex}  VietKB_1_universal.dmg\n");
        assert_eq!(expected_sha256_mac(None, &notes, "VietKB_1_universal.dmg"), Some(hex));
        assert_eq!(digest_sha256("md5:abc"), None);
        assert_eq!(digest_sha256("sha256:short"), None);
    }

    #[test]
    fn the_dmg_asset_carries_githubs_digest() {
        let hex = "0f".repeat(32);
        let other = "11".repeat(32);
        let body = format!(
            r#"{{"tag_name":"v99.0.0","assets":[
            {{"name":"VietKB_99.0.0_x64-setup.exe","browser_download_url":"https://x/s.exe","size":1,"digest":"sha256:{other}"}},
            {{"name":"VietKB_99.0.0_universal.dmg","browser_download_url":"https://x/a.dmg","size":2,"digest":"sha256:{hex}"}}]}}"#
        );
        let mac = parse_release_for(&body, true).unwrap().unwrap();
        assert_eq!(mac.asset_url.as_deref(), Some("https://x/a.dmg"));
        assert_eq!(mac.asset_sha256, Some(hex));
    }

    #[test]
    #[ignore = "network: hits the live GitHub API for thnonl/vietkb"]
    fn live_release_is_not_newer_than_this_build() {
        let status = run_check();
        println!(
            "LIVE state={:?} version={:?} message={:?}",
            status.state,
            status.update.as_ref().map(|info| info.version.clone()),
            status.message
        );
        assert_eq!(status.state, UpdateState::None);
    }

    #[test]
    #[ignore = "sets VKB_UPDATE_FAKE=1 for this process"]
    fn fake_release_reports_0_9_9() {
        std::env::set_var("VKB_UPDATE_FAKE", "1");
        let status = run_check();
        std::env::remove_var("VKB_UPDATE_FAKE");
        let info = status.update.clone().expect("fake update");
        println!(
            "FAKE state={:?} version={} tag={} asset={:?} changelog={:?}",
            status.state, info.version, info.tag, info.asset_name, info.changelog
        );
        assert_eq!(status.state, UpdateState::Available);
        assert_eq!(info.version, "0.9.9");
        assert_eq!(info.asset_name.as_deref(), Some("VietKB_0.9.9_x64-setup.exe"));
        assert!(info.changelog.contains("Huy hiệu cập nhật"));
    }
}
