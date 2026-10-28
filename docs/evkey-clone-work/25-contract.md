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
    /// Hook swallows the key; backend deletes `delete_graphemes` chars backwards, then types `insert`.
    Replace { delete_graphemes: u32, insert: String },
    /// The key itself must be swallowed and nothing typed (e.g. a modifier trigger).
    Swallow,
    /// Toggle Vietnamese on/off (handled by the app, not the engine).
    ToggleEnabled,
    /// Invalid keystroke: backend beeps and swallows.
    Reject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EngineOptions {
    pub method: InputMethod,
    pub charset: Charset,
    pub old_tone_style: bool,
    pub spell_check: bool,
    pub restore_wrong_word: bool,
    pub jfwz_as_consonant: bool,
    pub macro_enabled: bool,
    pub auto_capitalize: bool,
}

pub trait VietnameseEngine: Send {
    fn reset(&mut self);
    fn set_options(&mut self, options: EngineOptions);
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

TOML, `version = 1`. Unknown keys are preserved on write (round-trip via
`toml::Value` merge), so a newer build's config is not destroyed by an older one.

```toml
version = 1

[input]
enabled = true
method = "telex"          # telex | vni | viqr
charset = "unicode"       # unicode | unicode_nfd | tcvn3 | vni_windows | viqr | viscii | utf8

[typing]
old_tone_style = false     # gõ dấu kiểu cũ (oà / uý)
spell_check = false        # kiểm tra chính tả
restore_wrong_word = true  # tự động khôi phục từ gõ sai
jfwz_as_consonant = false  # dùng j, f, w, z làm phụ âm
auto_capitalize = false    # tự động viết hoa sau dấu câu
macro_enabled = true       # gõ tắt

[hotkeys]
toggle = "Ctrl+Shift"
restore_word = "Ctrl+Shift+Z"
charset_next = ""
method_next = ""
open_settings = ""

[system]
start_with_windows = true
start_minimized = true
notification_sound = false
language = "vi"            # vi | en
theme = "system"           # system | light | dark

[ui]
window_width = 980
window_height = 680
last_tab = "typing"

[[apps.rules]]
process = "game.exe"
enabled = false

[macros]
"btw" = "by the way"
"vn" = "Việt Nam"
```

Rust shape (field names and TOML keys differ only via `#[serde(rename_all = "snake_case")]`):

```rust
pub const SCHEMA_VERSION: u32 = 1;

pub struct AppConfig {
    pub version: u32,
    pub input: InputConfig,
    pub typing: TypingConfig,
    pub hotkeys: HotkeySet,
    pub system: SystemConfig,
    pub ui: UiConfig,
    pub apps: AppRules,
    pub macros: BTreeMap<String, String>, // trigger -> expansion
    #[serde(flatten)]
    pub extra: toml::Table,               // unknown keys, preserved verbatim
}

pub struct HotkeySet {
    pub toggle: Hotkey,
    pub restore_word: Option<Hotkey>,
    pub charset_next: Option<Hotkey>,
    pub method_next: Option<Hotkey>,
    pub open_settings: Option<Hotkey>,
}

/// `Hotkey` parses "Ctrl+Shift+Space" into modifiers + a key, and re-serializes
/// canonically. Order-insensitive parse; `Ctrl+Shift` alone means "left Ctrl + left Shift"
/// and is treated as a keyboard-combo trigger, not a char.
pub struct Hotkey { /* modifiers: Modifiers, key: HotkeyKey */ }
impl Hotkey {
    pub fn parse(s: &str) -> Result<Self, HotkeyError>;
    pub fn as_str(&self) -> String;
}
```

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
