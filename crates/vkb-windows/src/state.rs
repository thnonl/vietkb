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
    Engine, EngineAction, EngineOptions, InputMethod, KeyInput, VietnameseEngine,
};
use vkb_platform::hotkeys::Matcher;
use vkb_platform::plan::{decide, Decision, Injection};
use vkb_platform::rules::{self, BaseSettings, EffectiveSettings};
use vkb_platform::{BackendCommand, EngineSink, Hotkey};
use windows::Win32::UI::WindowsAndMessaging::{
    KBDLLHOOKSTRUCT, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

use crate::debuglog;
use crate::foreground::{self, AppKey};
use crate::globals;
use crate::hotkeys as hotkey_map;
use crate::ime;
use crate::input::{self, SendMode, SendResult};
use crate::keys::{self, ModifierState};
use crate::sound;

/// How long a foreground probe is reused even when the HWND does not change
/// (HWNDs are recycled by the OS).
const APP_TTL: Duration = Duration::from_millis(1500);
/// IMM composition checks are cheap but not free; throttle them.
const IME_TTL: Duration = Duration::from_millis(60);

/// The `Đổi kiểu gõ` cycle: Telex ↔ VNI.
pub fn next_method(current: InputMethod) -> InputMethod {
    match current {
        InputMethod::Telex | InputMethod::Legacy => InputMethod::Vni,
        InputMethod::Vni => InputMethod::Telex,
    }
}

/// The context a half-typed word belongs to.
///
/// Both fields are read from the already-refreshed foreground cache, so
/// building one costs nothing beyond two register moves.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct WordContext {
    /// Foreground window the word was started in.
    pub hwnd: isize,
    /// Whether Vietnamese input was active there (exclusion/override state).
    pub enabled: bool,
}

/// `true` when the pending word must be dropped before this key is processed.
///
/// `None` means no word has been started yet, so there is nothing to flush.
/// Staying in the same window with the same enablement never resets, which is
/// what keeps a normal sentence in one word buffer.
pub(crate) fn context_changed(prev: Option<WordContext>, next: WordContext) -> bool {
    match prev {
        Some(prev) => prev.hwnd != next.hwnd || prev.enabled != next.enabled,
        None => false,
    }
}

pub struct HookRuntime {
    engine: Box<dyn VietnameseEngine>,
    engine_failed: bool,
    macro_fingerprint: u64,
    options: EngineOptions,
    hotkeys: Matcher,
    mods: ModifierState,
    /// Bitset of physical keys currently down (autorepeat detection).
    down: [u64; 4],
    cfg: Option<Arc<AppConfig>>,
    base: BaseSettings,
    rule: Option<vkb_config::AppRule>,
    app: AppKey,
    /// Context of the word currently in the engine, if any. Compared against
    /// the refreshed foreground cache on every keystroke.
    word_context: Option<WordContext>,
    app_name: String,
    app_checked: Instant,
    ime_active: bool,
    ime_checked: Instant,
    resolved: Option<EffectiveSettings>,
    remembered: HashMap<String, bool>,
    send_mode: SendMode,
    use_clipboard: bool,
    sound_on: bool,
    remember_per_app: bool,
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
            macro_fingerprint: 0,
            options: EngineOptions::default(),
            hotkeys,
            mods: keys::seed_from_os(),
            down: [0; 4],
            cfg: None,
            base: BaseSettings::default(),
            rule: None,
            app: AppKey::default(),
            word_context: None,
            app_name: String::new(),
            app_checked: Instant::now() - APP_TTL,
            ime_active: false,
            ime_checked: Instant::now() - IME_TTL,
            resolved: None,
            remembered: HashMap::new(),
            send_mode: SendMode::Unicode,
            use_clipboard: false,
            sound_on: true,
            remember_per_app: false,
            last_global: globals::enabled(),
            config_enabled: None,
            sink,
        };
        match cfg {
            Some(cfg) => runtime.apply_config(cfg),
            None => runtime.apply_engine_options(),
        }
        if debuglog::enabled() {
            debuglog::note(&format!(
                "runtime-start enabled={} method={:?} charset={:?} spell_check={} jfwz_as_consonant={} send_mode={:?} clipboard={} hotkeys={}",
                runtime.effective_enabled(),
                runtime.base.method,
                runtime.base.charset,
                runtime.options.spell_check,
                runtime.options.jfwz_as_consonant,
                runtime.send_mode,
                runtime.use_clipboard,
                runtime.hotkeys.len(),
            ));
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

        // Adopt `input.enabled` only when it actually changed in the file, so a
        // hotkey toggle is not reset by an unrelated settings save.
        if self.config_enabled != Some(cfg.input.enabled) {
            globals::store_enabled(cfg.input.enabled);
            self.last_global = cfg.input.enabled;
        }
        self.config_enabled = Some(cfg.input.enabled);

        self.cfg = Some(cfg);
        self.app = AppKey::default();
        self.word_context = None;
        self.app_name.clear();
        self.resolved = None;
        self.rule = None;
        self.apply_engine_options();
        self.reset_engine();
        self.hotkeys.discard_latches();
        if debuglog::enabled() {
            debuglog::note(&format!(
                "config-applied global_enabled={} base_enabled={} app_rules={} hotkeys={} ime={}",
                globals::enabled(),
                self.base.enabled,
                self.cfg.as_ref().map_or(0, |cfg| cfg.apps.rules.len()),
                self.hotkeys.len(),
                self.ime_active,
            ));
        }
    }

    /// Feed one hook event. Returns whether the key must be swallowed.
    pub fn on_event(&mut self, message: u32, info: &KBDLLHOOKSTRUCT) -> Decision {
        let mut trace = debuglog::Trace::new(message, info);
        let decision = self.route(message, info, &mut trace);
        trace.finish(&decision);
        decision
    }

    fn route(
        &mut self,
        message: u32,
        info: &KBDLLHOOKSTRUCT,
        trace: &mut debuglog::Trace,
    ) -> Decision {
        let down = message == WM_KEYDOWN || message == WM_SYSKEYDOWN;
        let released = message == WM_KEYUP || message == WM_SYSKEYUP;
        if !down && !released {
            trace.stage("non-key");
            return Decision::PASS;
        }

        let vk = info.vkCode as u16;

        // Our own injections re-enter the hook; never feed them back.
        if info.dwExtraInfo == input::INJECTED_MAGIC {
            trace.stage("own-injection");
            return Decision::PASS;
        }
        // Unicode packets come from an IME: leave the composition alone.
        if vk == keys::VK_PACKET {
            trace.stage("ime-packet");
            return Decision::PASS;
        }

        let is_modifier = keys::is_modifier(vk);
        let was_down = self.is_down(vk);
        self.set_down(vk, down);
        self.mods.apply(vk, down);
        let modifiers = self.mods.modifiers();
        trace.mods(modifiers, self.mods.caps_lock);
        let repeat = down && was_down;
        self.sync_global_toggle();

        // 1. Hotkeys before anything else: the trigger must never reach the app.
        if let Some(hotkey) = self.hotkeys.on_key(vk, down, modifiers, is_modifier) {
            trace.stage("hotkey");
            self.on_hotkey(hotkey);
            if down && !is_modifier {
                return Decision::SWALLOW;
            }
            // Modifier-only combos (Ctrl+Shift) are matched but passed through:
            // swallowing bare modifiers would break Ctrl+Shift+X in the target.
        }

        if !down {
            trace.stage("key-up");
            return Decision::PASS;
        }

        // 2. Shortcut chords are never text.
        if self.mods.has_shortcut_modifier() {
            trace.stage("shortcut");
            self.reset_engine();
            return Decision::PASS;
        }

        // 3. Per-app enablement and IME composition.
        self.refresh_app();
        // A different foreground window (or a changed exclusion/override) must
        // not inherit the half-typed word. `refresh_app` just refreshed the
        // cache, so this is only an integer + flag comparison on the common path.
        self.sync_word_context();
        let enabled = self.effective_enabled();
        if !enabled || self.ime_active {
            trace.inactive(if !enabled {
                "disabled"
            } else {
                "ime-composition"
            });
            trace.stage("inactive");
            self.reset_engine();
            return Decision::PASS;
        }

        // 4. Caret movement and other commands end the current word.
        if keys::is_caret_mover(vk) {
            trace.stage("caret-mover");
            self.reset_engine();
            return Decision::PASS;
        }

        // 5. Text keys plus the boundary keys the engine cares about.
        let ch = self.char_for(vk, info.scanCode as u16);
        trace.ch(ch);
        if ch.is_none() && !keys::is_word_boundary(vk) {
            trace.stage("non-text");
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
        trace.action(&action);

        if action == EngineAction::ToggleEnabled {
            trace.stage("engine-toggle");
            let next = !self.effective_enabled();
            self.set_enabled(next);
            return Decision::SWALLOW;
        }

        let decision = decide(&action, ch);
        trace.stage(if decision.swallow {
            "engine-swallow"
        } else {
            "engine-pass"
        });
        if decision.swallow {
            let has_sink = self.sink.is_some();
            if has_sink {
                if let Some(sink) = self.sink.as_mut() {
                    if let Err(err) = sink.emit(action) {
                        tracing::warn!(error = %err, "engine sink failed");
                    }
                }
            } else {
                let sent = self.execute(&decision);
                trace.send(sent);
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
                globals::fire_command(BackendCommand::RememberApp {
                    process: self.app_name.clone(),
                    enabled,
                });
            }
            if self.sound_on {
                sound::play_toggle(enabled);
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
            Hotkey::MacroPanel => globals::fire_command(BackendCommand::OpenMacroPanel),
            Hotkey::MacroToggle => globals::fire_command(BackendCommand::ToggleMacro),
            Hotkey::OpenSettings => globals::fire_command(BackendCommand::OpenSettings),
            Hotkey::CycleMethod => {
                let next = next_method(self.base.method);
                self.base.method = next;
                globals::fire_command(BackendCommand::SetMethod(next));
            }
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

    fn execute(&mut self, decision: &Decision) -> Option<SendResult> {
        match &decision.injection {
            Injection::None => None,
            Injection::Beep => {
                if self.sound_on {
                    sound::play_toggle(globals::enabled());
                }
                None
            }
            Injection::Type(text) => Some(self.deliver(text, 0)),
            Injection::Replace { deletes, insert } => Some(self.deliver(insert, *deletes)),
        }
    }

    /// Batched delivery: deletes and text go out in one `SendInput` call.
    fn deliver(&mut self, text: &str, deletes: u32) -> SendResult {
        if self.use_clipboard {
            if deletes > 0 {
                let inputs = input::build_segment(deletes, "", self.send_mode);
                input::send(&inputs);
            }
            if !text.is_empty() {
                crate::clipboard::paste_async(text.to_string());
            }
            return SendResult::default();
        }
        let inputs = input::build_segment(deletes, text, self.send_mode);
        input::send_detailed(&inputs)
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

    /// Flush a word that belongs to a previous context.
    ///
    /// Reads what [`Self::refresh_app`] just cached, so it adds no Win32 call
    /// and no allocation to the keystroke path. Entering an excluded app or
    /// leaving one also lands here through `effective_enabled`.
    fn sync_word_context(&mut self) {
        let next = WordContext {
            hwnd: self.app.hwnd,
            enabled: self.effective_enabled(),
        };
        if context_changed(self.word_context, next) {
            self.boundary_engine();
        }
        self.word_context = Some(next);
    }

    fn boundary_engine(&mut self) {
        if !self.engine_failed {
            self.engine.boundary();
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

    // MSRV is 1.80; `is_none_or` (Rust 1.82) is what this lint asks for, so the
    // `map_or(true, ..)` calls stay until the MSRV moves.
    #[allow(clippy::unnecessary_map_or)]
    fn apply_engine_options(&mut self) {
        let effective = self.effective();
        let typing = self.cfg.as_ref().map(|cfg| cfg.typing.clone());
        let options = EngineOptions {
            method: effective.method,
            charset: effective.charset,
            modern_tone: typing.as_ref().map_or(true, |t| t.modern_tone),
            spell_check: typing.as_ref().is_some_and(|t| t.spell_check),
            restore_wrong_word: typing.as_ref().is_some_and(|t| t.restore_wrong_word),
            jfwz_as_consonant: typing.as_ref().is_some_and(|t| t.jfwz_as_consonant),
            macro_enabled: effective.macro_enabled,
            macro_case_follow: typing.as_ref().map_or(true, |t| t.macro_case_follow),
            auto_capitalize: typing.as_ref().is_some_and(|t| t.auto_capitalize),
            free_tone: typing.as_ref().map_or(true, |t| t.free_tone),
            auto_horn_uo: typing.as_ref().map_or(true, |t| t.auto_horn_uo),
            raw_zone: typing.as_ref().map_or(true, |t| t.raw_zone),
        };
        if options != self.options {
            self.options = options;
            self.engine.set_options(options);
        }
        // Macros live outside `EngineOptions`; push them when the table
        // actually changes, never on the keystroke path.
        let fingerprint = self.cfg.as_ref().map_or(0, |cfg| {
            let mut hash = 0xcbf29ce484222325u64;
            for (trigger, expansion) in cfg.macros.iter() {
                for byte in trigger.bytes().chain(expansion.bytes()) {
                    hash = (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3);
                }
                hash = (hash ^ 0x1f).wrapping_mul(0x100000001b3);
            }
            hash
        });
        if fingerprint != self.macro_fingerprint {
            self.macro_fingerprint = fingerprint;
            let macros = self
                .cfg
                .as_ref()
                .map(|cfg| {
                    vkb_core::MacroTable::from_pairs(
                        cfg.macros.iter().map(|(k, v)| (k.as_str(), v.as_str())),
                    )
                })
                .unwrap_or_default();
            self.engine.set_macros(macros);
        }
    }

    fn char_for(&mut self, vk: u16, scan: u16) -> Option<char> {
        if let Some(ch) = keys::us_char(vk, self.mods.shift, self.mods.caps_lock) {
            return Some(ch);
        }
        // Numpad and non-character system keys must never become text via the
        // thread layout translator; only ordinary layout-dependent keys fall back.
        if (0x60..=0x69).contains(&vk) || matches!(vk, 0x5D | 0xAD..=0xB7 | 0xE2) {
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
    fn numpad_stays_non_text() {
        let mut rt = runtime();
        assert_eq!(rt.char_for(0x60, 0x52), None);
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
    fn remembered_app_rule_round_trips_through_config() {
        let _guard = LOCK.lock().unwrap();
        let mut cfg = AppConfig::default();
        cfg.apps.remember_per_app = true;
        remember_rule(&mut cfg, "notepad.exe", false);
        let encoded = toml::to_string(&cfg).unwrap();
        let mut decoded: AppConfig = toml::from_str(&encoded).unwrap();
        assert_eq!(decoded.apps.rules.len(), 1);
        assert_eq!(decoded.apps.rules[0].enabled, Some(false));
        remember_rule(&mut decoded, "Notepad.EXE", true);
        let decoded: AppConfig = toml::from_str(&toml::to_string(&decoded).unwrap()).unwrap();
        assert_eq!(decoded.apps.rules.len(), 1);
        assert_eq!(decoded.apps.rules[0].enabled, Some(true));
    }

    fn remember_rule(cfg: &mut AppConfig, process: &str, enabled: bool) {
        if let Some(rule) = cfg.apps.rules.iter_mut().find(|rule| rule.process.eq_ignore_ascii_case(process)) {
            rule.enabled = Some(enabled);
        } else {
            let mut rule = vkb_config::AppRule::new(process);
            rule.enabled = Some(enabled);
            cfg.apps.rules.push(rule);
        }
    }

    #[test]
    fn remember_per_app_off_writes_nothing() {
        let mut cfg = AppConfig::default();
        cfg.apps.remember_per_app = false;
        if cfg.apps.remember_per_app {
            remember_rule(&mut cfg, "notepad.exe", false);
        }
        assert!(cfg.apps.rules.is_empty());
    }

    #[test]
    fn startup_honours_a_persisted_disabled_rule() {
        let _guard = LOCK.lock().unwrap();
        let mut cfg = AppConfig::default();
        cfg.apps.remember_per_app = true;
        let mut rule = vkb_config::AppRule::new("notepad.exe");
        rule.enabled = Some(false);
        cfg.apps.rules.push(rule);
        let mut rt = HookRuntime::new(Some(Arc::new(cfg)), None);
        rt.app_name = "notepad.exe".to_string();
        rt.rule = rt.cfg.as_ref().and_then(|cfg| cfg.apps.find(&rt.app_name).cloned());
        assert!(!rt.effective_enabled());
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
    fn method_cycle_wraps_between_two_methods() {
        assert_eq!(next_method(InputMethod::Telex), InputMethod::Vni);
        assert_eq!(next_method(InputMethod::Vni), InputMethod::Telex);
        // The sentinel from an old `viqr` config restarts at VNI.
        assert_eq!(next_method(InputMethod::Legacy), InputMethod::Vni);
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

    /// Regression: a half-typed word must not survive a foreground change.
    #[test]
    fn context_change_flushes_the_pending_word() {
        let in_a = WordContext {
            hwnd: 0x00A1,
            enabled: true,
        };
        let in_b = WordContext {
            hwnd: 0x00B2,
            enabled: true,
        };
        // First key after start: nothing to flush yet.
        assert!(!context_changed(None, in_a));
        // Same window, same enablement: the word keeps growing.
        assert!(!context_changed(Some(in_a), in_a));
        // A different foreground window ends the word.
        assert!(context_changed(Some(in_a), in_b));
        // Return trip flushes again rather than reviving the old buffer.
        assert!(context_changed(Some(in_b), in_a));
        // Entering (or leaving) an excluded/overridden app ends it too.
        assert!(context_changed(Some(in_a), WordContext { enabled: false, ..in_a }));
        assert!(context_changed(
            Some(WordContext { enabled: false, ..in_a }),
            in_a
        ));
    }
}
