//! Microsoft Store (MSIX) build: there is no self-updater.
//!
//! The Store delivers updates, so this build has none of `update.rs`: no
//! GitHub request, no download, no installer launch, no `vkb:update` event, no
//! `check_update` / `update_status` / `download_update` / `install_update`
//! commands. What stays is the tray's "is there an update" view (always
//! "no", which keeps the tray badge and menu item out) and `open_url`, which
//! only hands one of the About window's fixed links to the user's browser.

/// The tray asks whether to draw the update badge: never, in this build.
pub fn available() -> Option<()> {
    None
}

/// The tray asks for the version to name in its menu item and tooltip: none.
pub fn available_version() -> Option<String> {
    None
}

/// Open one of the links the About window shows. Anything else is refused: the
/// page must not be able to hand arbitrary URLs to the shell.
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
    app.opener().open_url(trimmed, None::<&str>).map_err(|error| {
        crate::i18n::tf("err.update.openLink", &[("error", &error.to_string())])
    })
}
