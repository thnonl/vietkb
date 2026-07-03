# EVKey Rust Rewrite Plan — with Linux/Wayland Support

## 0. Goals

- Portable Vietnamese typing core: pure Rust, testable.
- Cross-platform UI/config.
- OS backends kept separate:
  - Windows: keyboard hook + TSF/IME.
  - macOS: Accessibility/EventTap.
  - Linux X11: IBus/Fcitx5 engine, optional direct X11 mode later.
  - Linux Wayland: IBus/Fcitx5 engine, **direct-commit text immediately after each keystroke**, no global keylogger.
- Linux target behavior: as close to Windows/macOS as possible — processed text is committed into the input right away; user is not forced to press `Space` to commit.

## 1. Rust workspace layout

```text
evkey-rs/
  Cargo.toml
  crates/
    evkey-core/          # Telex/VNI/VIQR engine, charset, macro, state machine
    evkey-config/        # config schema, per-app rules, migration
    evkey-platform/      # shared OS backend trait
    evkey-windows/       # Win hook + TSF impl
    evkey-macos/         # EventTap/Accessibility impl
    evkey-linux-common/  # DBus, desktop/session detect, packaging helpers
    evkey-linux-ibus/    # IBus engine
    evkey-linux-fcitx5/  # Fcitx5 addon/frontend integration
    evkey-linux-x11/     # optional legacy X11 hook mode
    evkey-ui/            # tray/settings UI
    evkey-cli/           # debug/test/config CLI
    evkey-daemon/        # runtime process
  apps/
    evkey-desktop/
  tests/
    fixtures/
    ime-cases/
  packaging/
    windows/
    macos/
    linux/
```

## 2. Core engine

### `evkey-core`

Handles text/state only, no OS calls.

Proposed API:

```rust
pub trait VietnameseEngine {
    fn reset(&mut self);
    fn process_key(&mut self, input: KeyInput) -> EngineAction;
    fn set_input_method(&mut self, method: InputMethod);
    fn set_charset(&mut self, charset: Charset);
}
```

`EngineAction`:

- `Noop`
- `Commit(String)`
- `Replace { delete_graphemes: usize, insert: String }`
- `Preedit(String)`
- `ToggleVietnamese`
- `BeepOrReject`

Modules:

- `telex`
- `vni`
- `viqr`
- `tone`
- `vowel`
- `normalization`
- `charset`
- `macro_expand`
- `spelling_policy`
- `clipboard_safe_replace`
- `per_app_policy`

Test matrix:

- Telex/VNI/VIQR.
- Unicode NFC/NFD.
- Correct tone placement on vowels.
- `ươ`, `ưu`, `uy`, `oa`, `oe`, `qu`, `gi`.
- Backspace behavior.
- Macro.
- Per-app disable.
- Browser/address-bar cases.

## 3. Platform abstraction

### `evkey-platform`

```rust
pub trait PlatformBackend {
    fn start(&mut self, sink: EngineSink) -> anyhow::Result<()>;
    fn stop(&mut self) -> anyhow::Result<()>;
    fn active_app(&self) -> Option<AppInfo>;
    fn emit(&mut self, action: EngineAction) -> anyhow::Result<()>;
}
```

Backends contain no Vietnamese logic. They only:

- Receive key/text events.
- Forward events to core.
- Apply output to the target app.
- Get active app/window.
- Handle tray/autostart/permissions.

## 4. Linux strategy

### Standards-first approach

Wayland does not allow safe global keyboard hooks. Correct plan:

1. **IBus engine, direct-commit**: supports GNOME, KDE, many GTK/Qt/Electron apps.
2. **Fcitx5 addon, direct-commit**: supports KDE/Arch/NixOS/power users.
3. X11 direct hook is optional legacy only, not the main path.

### Direct-commit design

The Linux engine **does not use long preedit like traditional IMEs by default**. Each key is processed as follows:

1. Receive key event from IBus/Fcitx5.
2. Forward key to `evkey-core`.
3. If core returns `Commit(text)` → commit text immediately.
4. If core returns `Replace { delete_graphemes, insert }`:
   - prefer surrounding text API to delete the just-typed characters;
   - commit `insert` immediately;
   - do not wait for `Space`/`Enter`.
5. If the app does not support surrounding text well → fall back per policy: short preedit, backspace simulation if the framework allows, or best-effort commit.

UX goals:

- Typing `tieengs` → the input receives and displays `tiếng` incrementally.
- User does not need to press `Space` to "push" text into the input field.
- `Space` is just a normal space character, or a macro trigger if configured.

### Wayland support

Wayland backend goes through the input method framework but configured for direct-commit:

- GNOME Wayland → IBus direct-commit engine.
- KDE Wayland → Fcitx5 direct-commit addon, IBus fallback.
- wlroots/Sway → Fcitx5 + `text-input-v3`/`input-method-v2` ecosystem.
- No global key capture as default mode.

Linux deliverables:

```text
crates/evkey-linux-ibus/
  org.evkey.EVKey.xml
  engine.rs
  dbus.rs
  direct_commit.rs
  surrounding_text.rs
  fallback.rs

crates/evkey-linux-fcitx5/
  addon/
  bridge.rs
  direct_commit.rs
  surrounding_text.rs
  fallback.rs

packaging/linux/
  deb/
  rpm/
  arch/
  flatpak/ optional
```

### Linux runtime detection

- `$XDG_SESSION_TYPE`
- `$WAYLAND_DISPLAY`
- `$DISPLAY`
- `$GTK_IM_MODULE`
- `$QT_IM_MODULE`
- `$XMODIFIERS`
- DBus session availability.
- Installed IM frameworks.

CLI:

```text
evkey doctor
evkey ibus install
evkey fcitx5 install
evkey status
```

## 5. Windows rewrite

First phase — maintain parity:

- Low-level keyboard hook mode.
- Push by message / clipboard-safe replacement.
- Per-app config.
- Tray + autostart.
- Updater separated from core into `evkey-updater`.

Then TSF/IME:

- `evkey-windows-tsf`.
- Better composition/preedit.
- Fewer game/browser issues.

## 6. macOS rewrite

- EventTap.
- Accessibility permission flow.
- Menu bar app.
- Per-app rules via bundle id.
- Secure input detection.
- No hooking in password/secure fields.

## 7. UI/config

UI options:

### Option A: Tauri

Pros:

- Rust-native bridge.
- Fast cross-platform.
- Nice settings UI.

Cons:

- Larger app size.

### Option B: egui

Pros:

- Pure Rust.
- Lighter than Electron/Tauri webview issues.

Recommendation: **egui for settings + native tray**, since EVKey must stay lightweight.

Config:

```toml
[input]
method = "telex"
charset = "unicode"
enabled = true

[hotkeys]
toggle = "Ctrl+Shift"

[apps]
"com.apple.Terminal".enabled = false
"code.exe".browser_fix = true

[linux]
frontend = "ibus"
commit_mode = "direct"
fallback_mode = "surrounding-text-then-preedit"
```

Storage locations:

- Windows: `%APPDATA%\EVKey\config.toml`
- macOS: `~/Library/Application Support/EVKey/config.toml`
- Linux: `~/.config/evkey/config.toml`

Linux `commit_mode`:

- `direct`: default — commit immediately after each keystroke, closest to Windows/macOS.
- `preedit`: compatibility mode for apps with broken surrounding text support.
- `hybrid`: direct when the app supports it well, short preedit otherwise.

## 8. Build/packaging

CI matrix:

- Windows x64/arm64.
- macOS universal.
- Linux x86_64/aarch64.

Linux packages:

- `.deb`
- `.rpm`
- Arch `PKGBUILD`
- AppImage only for the settings app — not sufficient for IME installation.
- Flatpak is difficult due to IME integration sandboxing → not prioritized.

## 9. Migration from legacy EVKey

1. Audit C++ features:
   - input methods
   - charsets
   - macro
   - per-app
   - browser fix
   - game compatibility
   - updater
2. Write a compatibility spec.
3. Convert old config if the format is readable.
4. Build a test corpus from real bug reports.

## 10. Roadmap

### Phase 1 — Core Rust MVP, 3–4 weeks

- `evkey-core`
- Telex/VNI/VIQR
- Unicode charset
- basic macro
- 500+ test cases
- CLI simulator:

```text
evkey-cli simulate --method telex "tieengs Vieetj"
```

Exit criteria:

- Core passes the test corpus.
- No OS dependencies.

### Phase 2 — Linux IBus direct-commit MVP, 4–6 weeks

- IBus engine.
- Direct-commit mode as default.
- Surrounding text delete + commit replacement.
- Wayland GNOME working.
- X11 via IBus working.
- `evkey doctor`.
- Basic config.

Exit criteria:

- Typing works in Firefox/Chrome/VS Code/LibreOffice/GTK Terminal.
- Wayland GNOME passes smoke test.
- Vietnamese text is committed into the input immediately after each keystroke — no `Space` required.
- Apps without surrounding text support have a clear fallback.

### Phase 3 — Fcitx5 direct-commit, 4–6 weeks

- Fcitx5 addon/bridge.
- Direct-commit mode as default.
- Surrounding text delete + commit replacement.
- KDE Wayland.
- wlroots best-effort docs.

Exit criteria:

- KDE Wayland + Fcitx5 stable.
- Arch/NixOS path documented.
- Vietnamese text is committed into the input immediately after each keystroke — no `Space` required.

### Phase 4 — Windows backend, 4–8 weeks

- Hook backend parity.
- Tray/settings.
- Per-app rules.
- Browser fix.

Exit criteria:

- Equivalent to current EVKey in common flows.

### Phase 5 — macOS backend, 3–6 weeks

- EventTap.
- Permission UI.
- Menu bar.
- Bundle packaging.

### Phase 6 — polish/release, 3–4 weeks

- Local-only crash logs.
- Optional auto-update.
- Code signing.
- Docs.
- Website download links.
- Reproducible release checklist.

## 11. Major risks

- Wayland global hook is impossible by design → must use IBus/Fcitx5.
- App compatibility varies across GTK/Qt/Electron.
- Direct-commit depends on surrounding text support; apps with poor support may need fallback.
- IME preedit behavior differs from Windows hook; long preedit avoided by default, only used as fallback.
- Fcitx5 native addon may require a C++ ABI bridge; consider a Rust DBus/process bridge to reduce risk.
- Clipboard-based replacement causes privacy/UX issues → last-resort fallback only, disabled by default.

## 12. Recommended priority order

1. Solid Rust core + extensive tests.
2. Linux IBus first — the standards-compliant Wayland path.
3. Fcitx5 for KDE/wlroots.
4. Windows/macOS after to reach parity.
5. UI after core/backend — avoid locking architecture too early.

## 13. Definition of Done

- Wayland GNOME: Vietnamese typing via IBus, no global hook.
- KDE Wayland: typing via Fcitx5 or IBus.
- Linux default commit mode is `direct`.
- Vietnamese text is committed into the input immediately after each keystroke — no need to press `Space` to push text.
- X11: works via IBus/Fcitx5, optional legacy hook.
- Windows/macOS: no core behavior regression.
- Core engine has golden tests.
- No keystroke content collected; logs do not record keystrokes by default.
