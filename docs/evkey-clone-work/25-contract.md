# Contract — frozen Rust + TS surfaces (Sprint 1)

Any change here is a **contract change**: proposed in `log.md`, approved by the Lead,
applied once, then pulled into all three worktrees before continuing.

## 1. Crate dependency rules

| Crate | May depend on | Must NOT depend on |
|---|---|---|
| `vkb-core` | `smallvec`, `unicode-normalization`, `serde`, `thiserror` | anything touching winapi/macos/tauri/std::fs — **checked in CI** |
| `vkb-config` | `vkb-core` (types only), `serde`, `toml` | OS crates |
| `vkb-platform` | `vkb-core` | winapi, tauri |
| `vkb-windows` | `vkb-core`, `vkb-config`, `vkb-platform`, `windows` | tauri |
| `vkb-macos` | same as windows (`core-graphics`, `objc2`) | tauri |
| `vkb-app` | everything + `tauri` | — |
| `vkb-cli` | `vkb-core`, `vkb-config` | tauri, OS crates |

`vkb-app` holds all OS glue that the Tauri shell needs; backends expose plain functions
and callbacks, never `AppHandle` types.

## 2. Key event model (`vkb-platform`)

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub win: bool,
}

/// One physical key transition as delivered by the OS hook.
#[derive(Clone, Copy, Debug)]
pub struct KeyInput {
    /// Windows virtual-key code (macOS backend maps to the same VK space).
    pub vk: u16,
    pub scan: u16,
    pub ch: Option<char>,   // layout-resolved char, lower/upper per shift state
    pub modifiers: Modifiers,
    /// true for key-down, false for key-up.
    pub down: bool,
    pub repeat: bool,
}
```

## 3. Engine action (`vkb-core`)

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EngineAction {
    /// Hook consumes nothing; the key must reach the app untouched.
    Noop,
    /// Hook swallows the key and the backend types `text` once.
    Commit(String),
    /// Hook swallows the key; delete `delete_units` backwards, then type `insert`.
    /// One unit = one Backspace in the target app = one UTF-16 code unit on Windows.
    Replace { delete_units: u32, insert: String },
    /// Same as `Replace`, then the original key is passed through to the app.
    /// Macros committed with Enter/Tab need this: the expansion is typed and the key still acts.
    ReplaceThenPass { delete_units: u32, insert: String },
    /// The key itself must be swallowed and nothing typed (e.g. a modifier trigger).
    Swallow,
    /// Toggle Vietnamese on/off (handled by the app, not the engine).
    ToggleEnabled,
    /// Invalid keystroke: backend beeps and swallows.
    Reject,
}

/// Shared immutable macro table: swapped with one `Arc` store, never copied per keystroke.
pub type MacroTable = std::sync::Arc<std::collections::BTreeMap<String, String>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EngineOptions {
    pub method: InputMethod,
    pub charset: Charset,
    /// true = kiểu mới (`hoà`, `thuý`) · false = kiểu cũ (`hòa`, `thúy`).
    pub modern_tone: bool,
    pub spell_check: bool,
    pub restore_wrong_word: bool,
    pub jfwz_as_consonant: bool,
    pub macro_enabled: bool,
    pub auto_capitalize: bool,
}

pub trait VietnameseEngine: Send {
    fn reset(&mut self);
    fn set_options(&mut self, options: EngineOptions);
    /// Swap the macro table. One `Arc` store; valid from the next keystroke.
    fn set_macros(&mut self, macros: MacroTable);
    /// Pure: no I/O, no allocation on the `Noop` path.
    fn process_key(&mut self, key: KeyInput) -> EngineAction;
    /// Called when the app loses focus or the user moves the caret.
    fn boundary(&mut self);
}
```

`Noop` must not allocate. `process_key` must be O(1) in the current word length up to
`MAX_WORD` (32 chars); beyond that, commit and reset.

## 4. Platform backend (`vkb-platform`)

```rust
pub struct AppInfo {
    pub process_name: String,   // "chrome.exe"
    pub exe_path: Option<PathBuf>,
    pub window_title: Option<String>,
}

pub trait EngineSink: Send {
    fn emit(&mut self, action: EngineAction) -> anyhow::Result<()>;
    fn active_app(&self) -> Option<AppInfo>;
}

pub trait PlatformBackend: Send {
    fn start(&mut self, sink: Box<dyn EngineSink>) -> anyhow::Result<()>;
    fn stop(&mut self) -> anyhow::Result<()>;
    fn set_config(&mut self, config: SharedConfig);
    fn is_running(&self) -> bool;
}
```

Windows specifics that the contract fixes (so the UI can rely on them):

```rust
pub fn install_hotkeys(hotkeys: &HotkeySet, on_hotkey: HotkeyCallback) -> Result<()>;
pub fn set_autostart(enabled: bool) -> Result<()>;
pub fn set_enabled(enabled: bool);
pub fn is_enabled() -> bool;
/// Registers a callback fired when the hook toggles itself (e.g. hotkey pressed).
pub fn set_enabled_callback(cb: impl Fn(bool) + Send + 'static);
```

## 5. Config schema (`vkb-config`)

TOML, `version = 1`. `crates/vkb-config/src/schema.rs` is the authority; this sample
mirrors its `Default` impl. Unknown keys are preserved on write (`AppConfig::extra`), so a
newer build's config is not destroyed by an older one. An empty string removes a hotkey;
a missing key means "use the default".

```toml
version = 1

[input]
enabled = true
method = "telex"          # telex | vni | viqr
charset = "unicode"       # unicode | unicode_nfd | tcvn3 | vni_windows | viqr | viscii | bk_hcm1 | vps | utf8
mechanism = "hook"        # hook | ime

[typing]
modern_tone = true              # true = kiểu mới (hoà, thuý) · false = kiểu cũ (hòa, thúy)
free_tone = true                # "Đặt dấu tự do"
jfwz_as_consonant = true        # "Cho phép 'f j w z' làm phụ âm"
auto_capitalize = false         # "Tự động viết hoa sau Enter . ! ?"
spell_check = true              # "Bật kiểm tra chính tả"
restore_wrong_word = false      # "Tự động khôi phục phím với từ sai"
disable_on_non_us_layout = true # "Tắt TV khi Layout bàn phím khác US"
suggestion_fix_1 = true         # "Sửa lỗi dấu của Gợi ý 1"
suggestion_fix_2 = false        # "Sửa lỗi dấu của Gợi ý 2"
metro_fix = false               # "Hỗ trợ ứng dụng Metro"
send_standard = false           # "Gửi phím kiểu chuẩn"
use_clipboard = false           # "Sử dụng Clipboard để gửi phím"
macro_enabled = true            # "Cho phép gõ tắt"
macro_when_disabled = false     # "Vẫn cho phép gõ tắt khi tắt gõ Tiếng Việt"

[hotkeys]
enable = "F1"
disable = "F2"
toggle = ""                     # optional single-key switch
restore_word = ""
app_exclude = "F11"
convert_tool = "F6"
macro_panel = "F8"
macro_toggle = "F9"
open_settings = ""

[system]
start_with_windows = true
run_as_admin = false
notification_sound = true
start_minimized = true
language = "vi"                 # vi | en
theme = "system"                # system | light | dark
check_updates = true

[ui]
window_width = 1040
window_height = 720
last_tab = "overview"
compact_rows = false

[apps]
remember_per_app = false        # "Loại trừ ứng dụng tự động"

[[apps.rules]]
process = "game.exe"
enabled = false
charset = "unicode"
method = "telex"
macro_enabled = false

[macros]
"btw" = "by the way"
"vn" = "Việt Nam"
```

Rules the code enforces and tests pin:

- `Hotkey::parse` accepts `Ctrl+Shift` (modifier-only combo), `Ctrl+Alt+V`, `F1`, `Alt+Space`,
  case- and order-insensitive; canonical output order is `Ctrl+Alt+Shift+Win+Key`.
- A bare letter with no modifier is rejected; the UI must show the error inline.
- `Charset`/`InputMethod` serialize as the snake_case slugs above; `as_str`/`from_str`
  are the only conversions.


Config path: `%APPDATA%\VietKB\config.toml`. Legacy migration: if absent and
`settings.ini` next to the exe exists, import it once into schema v1.

## 6. Tauri command surface (frozen names + payloads)

```rust
#[tauri::command] fn get_settings() -> SettingsView;
#[tauri::command] fn set_settings(next: SettingsView) -> Result<SettingsView, String>;
#[tauri::command] fn patch_settings(patch: serde_json::Value) -> Result<SettingsView, String>;
#[tauri::command] fn reset_settings(section: String) -> Result<SettingsView, String>;
#[tauri::command] fn get_status() -> StatusView;
#[tauri::command] fn toggle_enabled() -> StatusView;
#[tauri::command] fn set_method(method: String) -> Result<StatusView, String>;
#[tauri::command] fn set_charset(charset: String) -> Result<StatusView, String>;

#[tauri::command] fn list_macros(query: String) -> Vec<MacroRow>;
#[tauri::command] fn upsert_macro(trigger: String, expansion: String) -> Result<(), String>;
#[tauri::command] fn delete_macros(triggers: Vec<String>) -> Result<(), String>;
#[tauri::command] fn import_macros(path: String) -> Result<usize, String>;
#[tauri::command] fn export_macros(path: String) -> Result<usize, String>;

#[tauri::command] fn list_app_rules() -> Vec<AppRuleRow>;
#[tauri::command] fn set_app_rule(process: String, enabled: bool) -> Result<(), String>;
#[tauri::command] fn delete_app_rule(process: String) -> Result<(), String>;
#[tauri::command] fn list_running_apps() -> Vec<RunningApp>;

#[tauri::command] fn parse_hotkey(input: String) -> Result<String, String>; // canonical form or error
#[tauri::command] fn check_hotkey_conflict(input: String, ignore_field: String) -> Option<String>;

#[tauri::command] fn convert_text(input: String, from: String, to: String) -> Result<String, String>;
#[tauri::command] fn open_config_folder() -> Result<(), String>;
#[tauri::command] fn app_info() -> AppInfoView; // version, arch, hook state, config path
```

Views are serialized with `#[serde(rename_all = "camelCase")]`. The TS side gets matching
types from `ui/src/lib/bindings.ts` — hand-written, kept in sync by review; no codegen step
in Sprint 1.

## 7. Frontend contract

- Stack: Vite 5 + TypeScript strict + Svelte 5, hand-written CSS with design tokens in
  `ui/src/styles/tokens.css`. No Tailwind, no UI kit, no icon package beyond inline SVG.
- Every `invoke` goes through `ui/src/lib/api.ts`, one exported function per command.
- Settings changes apply **live** (debounced 150 ms) and never need a Save button; a
  status toast confirms persistence failure.
- Hotkey capture field: focus → listens for keydown → shows canonical string as it is
  typed → Enter commits, Esc cancels, Backspace clears. It must show the exact string the
  backend will store.
