//! The hook thread's private runtime.
//!
//! One instance lives behind `RUNTIME` and is only ever touched by the hook
//! thread — except `globals` (atomics + callbacks) which the UI thread updates.
//! No lock is taken on the keystroke path, no allocation happens unless we
//! actually inject text.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use vkb_config::AppConfig;
use vkb_core::{
    Engine, EngineAction, EngineOptions, KeyInput, VietnameseEngine,
};
use vkb_platform::hotkeys::Matcher;
use vkb_platform::plan::{decide, Decision, Injection};
use vkb_platform::rules::{self, BaseSettings, EffectiveSettings};
use vkb_platform::{BackendCommand, EngineSink, Hotkey};
use windows::Win32::UI::WindowsAndMessaging::{
    KBDLLHOOKSTRUCT, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

use crate::foreground::{self, AppKey};
use crate::globals;
use crate::hotkeys as hotkey_map;
use crate::ime;
use crate::input::{self, SendMode};
use crate::keys::{self, ModifierState};
use crate::sound;

/// How long a foreground probe is reused even when the HWND does not change
/// (HWNDs are recycled by the OS).
const APP_TTL: Duration = Duration::from_millis(1500);
/// IMM composition checks are cheap but not free; throttle them.
const IME_TTL: Duration = Duration::from_millis(60);

pub struct HookRuntime {
    engine: Box<dyn VietnameseEngine>,
    engine_failed: bool,
    options: EngineOptions,
    hotkeys: Matcher,
    mods: ModifierState,
    /// Bitset of physical keys currently down (autorepeat detection).
    down: [u64; 4],
    cfg: Option<Arc<AppConfig>>,
    base: BaseSettings,
    rule: Option<vkb_config::AppRule>,
    app: AppKey,
    app_name: String,
    app_checked: Instant,
    us_layout: bool,
    ime_active: bool,
    ime_checked: Instant,
    resolved: Option<EffectiveSettings>,
    remembered: HashMap<String, bool>,
    send_mode: SendMode,
    use_clipboard: bool,
    sound_on: bool,
    remember_per_app: bool,
    disable_on_non_us: bool,
    last_global: bool,
    config_enabled: Option<bool>,
    sink: Option<Box<dyn EngineSink>>,
}

impl HookRuntime {
    pub fn new(cfg: Option<Arc<AppConfig>>, sink: Option<Box<dyn EngineSink>>) -> Self {
        let pending = globals::PENDING_HOTKEYS.lock().clone();
        let hotkeys = hotkey_map::compile(&pending.unwrap_or_default());
        let mut runtime = Self {
            engine: Box::new(Engine::new(EngineOptions::default())),
            engine_failed: false,
            options: EngineOptions::default(),
            hotkeys,
            mods: keys::seed_from_os(),
            down: [0; 4],
            cfg: None,
            base: BaseSettings::default(),
            rule: None,
            app: AppKey::default(),
            app_name: String::new(),
            app_checked: Instant::now() - APP_TTL,
            us_layout: true,
            ime_active: false,
            ime_checked: Instant::now() - IME_TTL,
            resolved: None,
            remembered: HashMap::new(),
            send_mode: SendMode::Unicode,
            use_clipboard: false,
            sound_on: true,
            remember_per_app: false,
            disable_on_non_us: false,
            last_global: globals::enabled(),
            config_enabled: None,
            sink,
        };
        match cfg {
            Some(cfg) => runtime.apply_config(cfg),
            None => runtime.apply_engine_options(),
        }
        runtime
    }

    /// Replace the settings the hook consults. Called on the hook thread only.
    pub fn apply_config(&mut self, cfg: Arc<AppConfig>) {
        self.base = BaseSettings {
            enabled: cfg.input.enabled,
            method: cfg.input.method,
            charset: cfg.input.charset,
            macro_enabled: cfg.typing.macro_enabled,
        };
        self.hotkeys = hotkey_map::compile(&cfg.hotkeys);
        self.send_mode = if cfg.typing.send_standard {
            SendMode::Standard
        } else {
            SendMode::Unicode
        };
        self.use_clipboard = cfg.typing.use_clipboard;
        self.sound_on = cfg.system.notification_sound;
        self.remember_per_app = cfg.apps.remember_per_app;
        self.disable_on_non_us = cfg.typing.disable_on_non_us_layout;

        // Adopt `input.enabled` only when it actually changed in the file, so a
        // hotkey toggle is not reset by an unrelated settings save.
        if self.config_enabled != Some(cfg.input.enabled) {
            globals::store_enabled(cfg.input.enabled);
            self.last_global = cfg.input.enabled;
        }
        self.config_enabled = Some(cfg.input.enabled);

        self.cfg = Some(cfg);
        self.app = AppKey::default();
        self.app_name.clear();
        self.resolved = None;
        self.rule = None;
        self.apply_engine_options();
        self.reset_engine();
        self.hotkeys.discard_latches();
    }

    /// Feed one hook event. Returns whether the key must be swallowed.
    pub fn on_event(&mut self, message: u32, info: &KBDLLHOOKSTRUCT) -> Decision {
        let down = message == WM_KEYDOWN || message == WM_SYSKEYDOWN;
        let released = message == WM_KEYUP || message == WM_SYSKEYUP;
        if !down && !released {
            return Decision::PASS;
        }

        let vk = info.vkCode as u16;

        // Our own injections re-enter the hook; never feed them back.
        if info.dwExtraInfo == input::INJECTED_MAGIC {
            return Decision::PASS;
        }
        // Unicode packets come from an IME: leave the composition alone.
        if vk == keys::VK_PACKET {
            return Decision::PASS;
        }

        let is_modifier = keys::is_modifier(vk);
        let was_down = self.is_down(vk);
        self.set_down(vk, down);
        self.mods.apply(vk, down);
        let modifiers = self.mods.modifiers();
        let repeat = down && was_down;
        self.sync_global_toggle();

        // 1. Hotkeys before anything else: the trigger must never reach the app.
        if let Some(hotkey) = self.hotkeys.on_key(vk, down, modifiers, is_modifier) {
            self.on_hotkey(hotkey);
            if down && !is_modifier {
                return Decision::SWALLOW;
            }
            // Modifier-only combos (Ctrl+Shift) are matched but passed through:
            // swallowing bare modifiers would break Ctrl+Shift+X in the target.
        }

        if !down {
            return Decision::PASS;
        }

        // 2. Shortcut chords are never text.
        if self.mods.has_shortcut_modifier() {
            self.reset_engine();
            return Decision::PASS;
        }

        // 3. Per-app enablement, IME composition, non-US layout.
        self.refresh_app();
        if !self.effective_enabled()
            || self.ime_active
            || (self.disable_on_non_us && !self.us_layout)
        {
            self.reset_engine();
            return Decision::PASS;
        }

        // 4. Caret movement and other commands end the current word.
        if keys::is_caret_mover(vk) {
            self.reset_engine();
            return Decision::PASS;
        }

        // 5. Text keys plus the boundary keys the engine cares about.
        let ch = self.char_for(vk, info.scanCode as u16);
        if ch.is_none() && !keys::is_word_boundary(vk) {
            self.reset_engine();
            return Decision::PASS;
        }

        let key = KeyInput {
            vk,
            scan: info.scanCode as u16,
            ch,
            modifiers,
            down: true,
            repeat,
        };
        let action = self.process(key);

        if action == EngineAction::ToggleEnabled {
            let next = !self.effective_enabled();
            self.set_enabled(next);
            return Decision::SWALLOW;
        }

        let decision = decide(&action, ch);
        if decision.swallow {
            let has_sink = self.sink.is_some();
            if has_sink {
                if let Some(sink) = self.sink.as_mut() {
                    if let Err(err) = sink.emit(action) {
                        tracing::warn!(error = %err, "engine sink failed");
                    }
                }
            } else {
                self.execute(&decision);
            }
        }
        decision
    }

    /// Flip Vietnamese on/off. Idempotent and never latches: calling it twice
    /// with opposite values always comes back to the original state.
    pub fn set_enabled(&mut self, enabled: bool) {
        if globals::enabled() != enabled {
            globals::store_enabled(enabled);
            self.last_global = enabled;
            if self.remember_per_app && !self.app_name.is_empty() {
                self.remembered.insert(self.app_name.clone(), enabled);
            }
            if self.sound_on {
                sound::play_notification_sound();
            }
            globals::fire_enabled(enabled);
            globals::fire_command(BackendCommand::Enable(enabled));
        }
        self.reset_engine();
    }

    pub fn effective_enabled(&self) -> bool {
        if let Some(rule) = self.rule.as_ref().and_then(|rule| rule.enabled) {
            return rule;
        }
        if self.remember_per_app {
            if let Some(state) = self.remembered.get(&self.app_name) {
                return *state;
            }
        }
        globals::enabled()
    }

    pub fn active_app_name(&self) -> &str {
        &self.app_name
    }

    // -- internals ---------------------------------------------------------

    fn on_hotkey(&mut self, hotkey: Hotkey) {
        match hotkey {
            Hotkey::Enable => self.set_enabled(true),
            Hotkey::Disable => self.set_enabled(false),
            Hotkey::Toggle => {
                let next = !self.effective_enabled();
                self.set_enabled(next);
            }
            Hotkey::RestoreWord => globals::fire_command(BackendCommand::RestoreWord),
            Hotkey::AppExclude => globals::fire_command(BackendCommand::ToggleAppExclude),
            Hotkey::ConvertTool => globals::fire_command(BackendCommand::OpenConvertTool),
            Hotkey::MacroPanel => globals::fire_command(BackendCommand::OpenMacroPanel),
            Hotkey::MacroToggle => globals::fire_command(BackendCommand::ToggleMacro),
            Hotkey::OpenSettings => globals::fire_command(BackendCommand::OpenSettings),
        }
        self.reset_engine();
        globals::fire_hotkey(hotkey);
    }

    fn process(&mut self, key: KeyInput) -> EngineAction {
        if self.engine_failed {
            return EngineAction::Noop;
        }
        let engine = &mut self.engine;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            engine.process_key(key)
        }));
        match result {
            Ok(action) => action,
            Err(_) => {
                // Sprint 1 ships before the engine lands; a panicking engine must
                // not take the hook thread with it.
                self.engine_failed = true;
                tracing::error!("engine panicked; Vietnamese input suspended");
                EngineAction::Noop
            }
        }
    }

    fn execute(&mut self, decision: &Decision) {
        match &decision.injection {
            Injection::None => {}
            Injection::Beep => {
                if self.sound_on {
                    sound::play_notification_sound();
                }
            }
            Injection::Type(text) => self.deliver(text, 0),
            Injection::Replace { deletes, insert } => self.deliver(insert, *deletes),
        }
    }

    /// Batched delivery: deletes and text go out in one `SendInput` call.
    fn deliver(&mut self, text: &str, deletes: u32) {
        if self.use_clipboard {
            if deletes > 0 {
                let inputs = input::build_segment(deletes, "", self.send_mode);
                input::send(&inputs);
            }
            if !text.is_empty() {
                crate::clipboard::paste_async(text.to_string());
            }
            return;
        }
        let inputs = input::build_segment(deletes, text, self.send_mode);
        input::send(&inputs);
    }

    fn reset_engine(&mut self) {
        if !self.engine_failed {
            self.engine.reset();
        }
    }

    fn refresh_app(&mut self) {
        let now = Instant::now();
        let hwnd = foreground::foreground_hwnd();
        if hwnd != 0 && hwnd == self.app.hwnd && now.saturating_duration_since(self.app_checked) < APP_TTL {
            self.refresh_ime(now, hwnd);
            return;
        }
        self.app_checked = now;
        if hwnd == 0 {
            self.app = AppKey::default();
            self.app_name.clear();
            self.resolved = None;
            self.rule = None;
            return;
        }
        let Some((key, app)) = foreground::probe(hwnd) else {
            self.app = AppKey::default();
            return;
        };
        self.us_layout = foreground::layout_is_us(hwnd);
        self.ime_active = ime::composition_active(hwnd);
        self.ime_checked = now;
        if key != self.app || app.process_name != self.app_name {
            self.app = key;
            self.app_name = app.process_name;
            self.resolve_app();
        }
    }

    fn refresh_ime(&mut self, now: Instant, hwnd: isize) {
        if now.saturating_duration_since(self.ime_checked) >= IME_TTL {
            self.ime_active = ime::composition_active(hwnd);
            self.ime_checked = now;
        }
    }

    fn resolve_app(&mut self) {
        let rule = self
            .cfg
            .as_ref()
            .and_then(|cfg| cfg.apps.find(&self.app_name).cloned());
        let remembered = self.remembered.get(&self.app_name).copied();
        let effective = rules::resolve(self.base, rule.as_ref(), remembered, self.remember_per_app);
        self.rule = rule;
        if self.resolved != Some(effective) {
            self.resolved = Some(effective);
            self.apply_engine_options();
            self.reset_engine();
        }
    }

    fn effective(&self) -> EffectiveSettings {
        self.resolved
            .unwrap_or_else(|| EffectiveSettings::from_base(self.base))
    }

    fn apply_engine_options(&mut self) {
        let effective = self.effective();
        let typing = self.cfg.as_ref().map(|cfg| cfg.typing.clone());
        let options = EngineOptions {
            method: effective.method,
            charset: effective.charset,
            modern_tone: typing.as_ref().map_or(true, |t| t.modern_tone),
            spell_check: typing.as_ref().map_or(false, |t| t.spell_check),
            restore_wrong_word: typing.as_ref().map_or(false, |t| t.restore_wrong_word),
            jfwz_as_consonant: typing.as_ref().map_or(false, |t| t.jfwz_as_consonant),
            macro_enabled: effective.macro_enabled,
            auto_capitalize: typing.as_ref().map_or(false, |t| t.auto_capitalize),
        };
        if options != self.options {
            self.options = options;
            self.engine.set_options(options);
        }
    }

    fn char_for(&mut self, vk: u16, scan: u16) -> Option<char> {
        if let Some(ch) = keys::us_char(vk, self.mods.shift, self.mods.caps_lock) {
            return Some(ch);
        }
        if self.us_layout {
            return None;
        }
        let state = ime::keyboard_state(&self.mods);
        ime::to_unicode(vk, scan, &state)
    }

    fn sync_global_toggle(&mut self) {
        let global = globals::enabled();
        if global != self.last_global {
            self.last_global = global;
            if self.remember_per_app && !self.app_name.is_empty() {
                self.remembered.insert(self.app_name.clone(), global);
            }
        }
    }

    fn is_down(&self, vk: u16) -> bool {
        let index = vk as usize;
        self.down[index / 64] & (1u64 << (index % 64)) != 0
    }

    fn set_down(&mut self, vk: u16, down: bool) {
        let index = vk as usize;
        let bit = 1u64 << (index % 64);
        if down {
            self.down[index / 64] |= bit;
        } else {
            self.down[index / 64] &= !bit;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// The enabled flag is process-wide; keep these tests from racing.
    static LOCK: Mutex<()> = Mutex::new(());

    fn runtime() -> HookRuntime {
        globals::store_enabled(true);
        HookRuntime::new(None, None)
    }

    #[test]
    fn toggle_twice_returns_to_previous_state() {
        let _guard = LOCK.lock().unwrap();
        let mut rt = runtime();
        assert!(rt.effective_enabled());
        rt.set_enabled(false);
        assert!(!rt.effective_enabled());
        assert!(!globals::enabled());
        rt.set_enabled(true);
        assert!(rt.effective_enabled(), "second toggle must restore the state");
        rt.set_enabled(true);
        assert!(rt.effective_enabled(), "repeating the same value is a no-op");
    }

    #[test]
    fn remember_per_app_keeps_state_per_process() {
        let _guard = LOCK.lock().unwrap();
        let mut rt = runtime();
        rt.remember_per_app = true;
        rt.app_name = "notepad.exe".to_string();
        rt.set_enabled(false);
        assert!(!rt.effective_enabled(), "notepad is off");

        rt.app_name = "chrome.exe".to_string();
        rt.rule = None;
        rt.resolved = None;
        // No remembered entry for chrome: the global flag (now false) applies.
        assert!(!rt.effective_enabled());

        rt.set_enabled(true);
        rt.app_name = "notepad.exe".to_string();
        assert!(!rt.effective_enabled(), "notepad remembered as off");
    }

    #[test]
    fn explicit_rule_beats_everything() {
        let _guard = LOCK.lock().unwrap();
        let mut rt = runtime();
        let mut rule = vkb_config::AppRule::new("game.exe");
        rule.enabled = Some(false);
        rt.app_name = "game.exe".to_string();
        rt.rule = Some(rule);
        assert!(!rt.effective_enabled());
    }

    #[test]
    fn key_bitset_tracks_autorepeat() {
        let _guard = LOCK.lock().unwrap();
        let mut rt = runtime();
        assert!(!rt.is_down(0x41));
        rt.set_down(0x41, true);
        assert!(rt.is_down(0x41));
        rt.set_down(0x41, false);
        assert!(!rt.is_down(0x41));
        // High virtual keys must not go out of bounds.
        rt.set_down(0xE7, true);
        assert!(rt.is_down(0xE7));
    }
}
