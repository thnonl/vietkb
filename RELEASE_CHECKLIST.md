# Release checklist

## Version/update metadata

- Release repo: `https://github.com/thnonl/vietkb`
- Update endpoint: `https://github.com/thnonl/vietkb/releases/latest/download/latest.json`
- Update signing private key: `%USERPROFILE%\.tauri\vietkb.key` (do not commit/share)
- Update public key is embedded in `crates/vietkb-gui/tauri.conf.json`.
- Update versions before build:
  - `crates/vietkb-gui/Cargo.toml` → `[package].version`
  - `crates/vietkb-gui/tauri.conf.json` → `version`
- Build updater artifacts with `scripts/build-windows-release.ps1`.
- GitHub Releases assets must include the Windows installer, its `.sig`, and `latest.json`.
- `latest.json` must reference the uploaded installer URL and inline `.sig` content.

## Core

- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- golden typing corpus passes
- no logs include typed content by default

## Linux

- GNOME Wayland IBus smoke test
- KDE Wayland Fcitx5 smoke test
- X11 IBus/Fcitx5 smoke test
- direct commit verified: no Space/Enter needed to commit text
- fallback behavior documented for apps without surrounding text

## Windows

- keyboard hook smoke test
- per-app disable smoke test
- browser field smoke test
- autostart smoke test

## macOS

- Accessibility permission flow smoke test
- secure input detection smoke test
- menu bar lifecycle smoke test

## Packaging

- `.deb` install/uninstall
- `.rpm` install/uninstall
- Arch package install/uninstall
- Windows installer smoke test
- macOS app bundle notarization check

## Privacy

- no key content telemetry
- crash logs local-only unless user opts in
- clipboard fallback disabled by default
