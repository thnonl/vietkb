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
use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

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
/// Shown when the release has no Windows x64 installer asset.
pub const NO_INSTALLER: &str = "Bản phát hành không có bộ cài cho Windows x64";

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
        .map_err(|error| format!("Không kết nối được máy chủ cập nhật: {error}"))?;
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
                    return Err(format!(
                        "GitHub đang giới hạn lượt kiểm tra (HTTP {status}). \
                         Thử lại sau vài phút."
                    ))
                }
                other => return Err(format!("Máy chủ cập nhật trả lời HTTP {other}.")),
            }
        }
    };
    parse_release(&body)
}

fn parse_release(body: &str) -> Result<Option<UpdateInfo>, String> {
    let release: ReleaseJson = serde_json::from_str(body)
        .map_err(|error| format!("Phản hồi cập nhật không đọc được: {error}"))?;

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
        .find(|asset| asset.name.to_ascii_lowercase().ends_with("x64-setup.exe"));

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
    *state().write() = status.clone();
    crate::tray::refresh(app, &crate::commands::status_now());
    use tauri::Emitter;
    if let Err(error) = app.emit("vkb:update", status) {
        tracing::warn!(%error, "could not broadcast the update state");
    }
}

/// Five seconds after launch, on a worker thread. Silent on failure — the
/// About window will show the reason the next time it is opened.
pub fn schedule_startup_check(app: tauri::AppHandle) {
    tauri::async_runtime::spawn_blocking(move || {
        std::thread::sleep(STARTUP_DELAY);
        if !crate::commands::config_cell().read().system.check_updates {
            return;
        }
        let status = run_check();
        if status.state == UpdateState::Available {
            tracing::info!(
                version = status.update.as_ref().map(|i| i.version.as_str()).unwrap_or(""),
                "update available"
            );
        }
        publish(&app, status);
    });
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn check_update(app: tauri::AppHandle) -> UpdateStatus {
    // Paint `Đang kiểm tra…` before the request goes out.
    publish(&app, UpdateStatus::checking());
    let status = tauri::async_runtime::spawn_blocking(run_check)
        .await
        .unwrap_or_else(|error| UpdateStatus::error(format!("Không kiểm tra được: {error}")));
    publish(&app, status.clone());
    status
}

#[tauri::command]
pub fn update_status() -> UpdateStatus {
    current()
}

#[tauri::command]
pub async fn download_update() -> Result<DownloadOutcome, String> {
    let Some(info) = available() else {
        return Err("Chưa có bản cập nhật để tải.".to_string());
    };
    let Some(url) = info.asset_url.clone() else {
        return Err(NO_INSTALLER.to_string());
    };
    let name = info
        .asset_name
        .clone()
        .unwrap_or_else(|| format!("VietKB_{}_x64-setup.exe", info.version));
    let changelog = info.changelog.clone();

    let outcome = tauri::async_runtime::spawn_blocking(move || {
        download_to_temp(&url, &name, &changelog, &info.version)
    })
    .await
    .unwrap_or_else(|error| Err(format!("Không tải được bản cập nhật: {error}")))?;

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
const INSTALLER_ARGS: [&str; 2] = ["/S", "/UPDATE"];

/// The batch the app hands itself to before quitting: run the setup, then bring
/// VietKB back. The installer kills this process (silent mode, Restart Manager),
/// so nothing inside the app can do the relaunch itself.
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

/// Launch the installer that `download_update` wrote, then leave cleanly.
#[tauri::command]
pub fn install_update(app: tauri::AppHandle) -> Result<(), String> {
    let downloaded = DOWNLOADED
        .get_or_init(|| parking_lot::Mutex::new(None))
        .lock()
        .clone()
        .ok_or_else(|| "Chưa tải bản cập nhật.".to_string())?;

    if !downloaded.path.exists() {
        return Err("Tệp cài đặt không còn trong thư mục tạm.".to_string());
    }

    #[cfg(windows)]
    {
        let exe = std::env::current_exe()
            .map_err(|error| format!("Không xác định được tệp chạy: {error}"))?;
        let script_path = download_dir().join("apply-update.cmd");
        std::fs::write(&script_path, update_script(&downloaded.path, &exe))
            .map_err(|error| format!("Không ghi được kịch bản cập nhật: {error}"))?;

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
            .map_err(|error| format!("Không chạy được trình cài đặt: {error}"))?;

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
        Err("Cài đặt tại chỗ chỉ hỗ trợ Windows.".to_string())
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
        return Err("Liên kết không được phép mở.".to_string());
    }
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_url(trimmed, None::<&str>)
        .map_err(|error| format!("Không mở được liên kết: {error}"))
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
    version: &str,
) -> Result<Downloaded, String> {
    let dir = download_dir();
    std::fs::create_dir_all(&dir)
        .map_err(|error| format!("Không tạo được thư mục tải: {error}"))?;

    let file_name = sanitize_file_name(name);
    let part = dir.join(format!("{file_name}.part"));
    let target = dir.join(&file_name);

    let agent = agent();
    let mut response = agent
        .get(url)
        .header("Accept", "application/octet-stream")
        .call()
        .map_err(|error| format!("Không tải được bản cập nhật: {error}"))?;
    let status = response.status().as_u16();
    if status != 200 {
        return Err(format!("Máy chủ tải bản cập nhật trả lời HTTP {status}."));
    }

    let mut file =
        std::fs::File::create(&part).map_err(|error| format!("Không ghi được tệp tải: {error}"))?;
    let copied = {
        let mut reader = response.body_mut().as_reader();
        std::io::copy(&mut reader, &mut file)
    };
    file.flush().ok();
    drop(file);
    if let Err(error) = copied {
        let _ = std::fs::remove_file(&part);
        return Err(format!("Tải bản cập nhật bị gián đoạn: {error}"));
    }
    if let Err(error) = std::fs::rename(&part, &target) {
        let _ = std::fs::remove_file(&part);
        return Err(format!("Không hoàn tất được tệp tải: {error}"));
    }

    let verified = match expected_sha256(changelog) {
        Some(expected) => {
            let bytes = std::fs::read(&target)
                .map_err(|error| format!("Không đọc lại được tệp tải: {error}"))?;
            let actual = sha256_hex(&bytes);
            if actual != expected {
                let _ = std::fs::remove_file(&target);
                return Err(
                    "SHA256 của tệp tải về không khớp với bản phát hành — đã huỷ tệp."
                        .to_string(),
                );
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
        let info = parse_release(body).expect("parses").expect("is newer");
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
        assert_eq!(NO_INSTALLER, "Bản phát hành không có bộ cài cho Windows x64");
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
        let info = parse_release(body).expect("parses").expect("newer");
        assert!(info.asset_url.is_none());
        assert!(info.asset_name.is_none());
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
