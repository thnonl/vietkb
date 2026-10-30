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
    Engine, EngineAction, EngineOptions, InputMethod, KeyInput, Modifiers, VietnameseEngine,
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

/// Harness negative control: reproduce the pre-D-E2E-03 trigger leak.
#[cfg(feature = "e2e-harness")]
fn legacy_trigger_leak() -> bool {
    crate::e2e::legacy_trigger_leak()
}

#[cfg(not(feature = "e2e-harness"))]
#[inline(always)]
fn legacy_trigger_leak() -> bool {
    false
}

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
    /// Bitset of physical keys whose key-down was swallowed as a hotkey trigger,
    /// so the matching key-up is swallowed too (no orphan key-up for the app).
    trigger_swallowed: [u64; 4],
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
    /// A short send switched this app to Standard scancodes.
    force_standard: bool,
    /// This app refused every send strategy: stop converting for it.
    auto_excluded: bool,
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
            trigger_swallowed: [0; 4],
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
            force_standard: false,
            auto_excluded: false,
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
        self.send_mode = if self.force_standard || cfg.typing.send_standard {
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
        self.trigger_swallowed = [0; 4];
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
        // A mouse button went down since the last key: the caret may have moved,
        // so the word is over (20-spec: "Mouse button down (any) -> boundary()").
        if globals::take_mouse_down() {
            self.boundary_engine();
        }
        let down = message == WM_KEYDOWN || message == WM_SYSKEYDOWN;
        let released = message == WM_KEYUP || message == WM_SYSKEYUP;
        if !down && !released {
            trace.stage("non-key");
            return Decision::PASS;
        }

        let vk = info.vkCode as u16;

        // Our own injections re-enter the hook; never feed them back.
        if info.dwExtraInfo == input::injected_tag() {
            trace.stage("own-injection");
            return Decision::PASS;
        }
        let packet_char = if vk == keys::VK_PACKET {
            let Some(ch) = char::from_u32(u32::from(info.scanCode as u16)) else {
                trace.stage("invalid-packet");
                return Decision::PASS;
            };
            if ch.is_control() {
                trace.stage("control-packet");
                return Decision::PASS;
            }
            Some(ch)
        } else {
            None
        };

        if let Some(ch) = packet_char {
            let modifiers = self.mods.modifiers();
            trace.mods(modifiers, self.mods.caps_lock);
            trace.ch(Some(ch));
            if !down {
                trace.stage("key-up");
                return Decision::PASS;
            }
            if self.mods.has_shortcut_modifier() {
                trace.stage("shortcut");
                self.reset_engine();
                return Decision::PASS;
            }
            self.refresh_app();
            self.sync_word_context();
            if self.ime_active || (!self.effective_enabled() && !self.macro_only_active()) {
                trace.stage("inactive");
                self.reset_engine();
                return Decision::PASS;
            }
            let key = KeyInput {
                vk,
                scan: info.scanCode as u16,
                ch: Some(ch),
                modifiers,
                down: true,
                repeat: false,
            };
            let action = self.process(key);
            trace.action(&action);
            if action == EngineAction::ToggleEnabled {
                trace.stage("engine-toggle");
                self.set_enabled(!self.effective_enabled());
                return Decision::SWALLOW;
            }
            let decision = decide(&action, Some(ch));
            trace.stage(if decision.swallow { "engine-swallow" } else { "engine-pass" });
            if decision.swallow {
                if let Some(sink) = self.sink.as_mut() {
                    if let Err(err) = sink.emit(action) {
                        tracing::warn!(error = %err, "engine sink failed");
                    }
                } else {
                    let sent = self.execute(&decision);
                    trace.send(sent);
                }
            }
            return decision;
        }

        let is_modifier = keys::is_modifier(vk);
        let was_down = self.is_down(vk);
        self.set_down(vk, down);
        self.mods.apply(vk, down);
        let modifiers = self.mods.modifiers();
        trace.mods(modifiers, self.mods.caps_lock);
        let repeat = down && was_down;
        self.sync_global_toggle();

        // A trigger key-down that was swallowed stays swallowed for as long as it
        // is held: an auto-repeat follows the bit, not the matcher latch (the
        // chord's modifier may already be up). A fresh key-down never inherits a
        // stale bit left by a lost key-up.
        if down && !is_modifier && self.trigger_is_swallowed(vk) {
            if repeat {
                trace.stage("hotkey-trigger-repeat");
                return Decision::SWALLOW;
            }
            self.set_trigger_swallowed(vk, false);
        }

        // 1. Hotkeys before anything else: the trigger must never reach the app.
        if let Some(hotkey) = self.hotkeys.on_key(vk, down, modifiers, is_modifier) {
            trace.stage("hotkey");
            let handled = self.on_hotkey(hotkey);
            if !is_modifier {
                if down {
                    if handled {
                        self.set_trigger_swallowed(vk, true);
                        return Decision::SWALLOW;
                    }
                } else if self.take_trigger_swallowed(vk) {
                    return Decision::SWALLOW;
                }
            }
            // Modifier-only combos (Ctrl+Shift) are matched but passed through:
            // swallowing bare modifiers would break Ctrl+Shift+X in the target.
        } else if !is_modifier {
            // Release-mode combos fire on key-up, but the trigger must be kept
            // from the app on the way down as well (D-E2E-03: VS Code opened its
            // command palette on `F1` and ate the keys that followed), and the
            // key-up of a swallowed key-down must not arrive on its own.
            if down {
                if self.hotkeys.swallows_trigger(vk) && !legacy_trigger_leak() {
                    trace.stage("hotkey-trigger");
                    self.set_trigger_swallowed(vk, true);
                    return Decision::SWALLOW;
                }
            } else if self.take_trigger_swallowed(vk) {
                trace.stage("hotkey-trigger-up");
                return Decision::SWALLOW;
            }
        }

        if !down {
            trace.stage("key-up");
            return Decision::PASS;
        }

        // 2. Shortcut chords are never text.
        if self.mods.has_shortcut_modifier() {
            if is_modifier {
                // A modifier on its own types nothing and may still be half of a
                // chord whose key needs the word (`restore_word`), so the word
                // survives until the chord's key arrives.
                trace.stage("modifier");
                return Decision::PASS;
            }
            // A hotkey that fires once the chord is up (`restore_word`) owns this
            // key: swallow it while the word is there, so the app does not run
            // its own `Ctrl+Shift+Z` first and restore on top of a redo.
            if self.hotkeys.awaiting_release() && self.engine_can_restore() {
                trace.stage("hotkey-armed");
                return Decision::SWALLOW;
            }
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
        if self.ime_active || (!enabled && !self.macro_only_active()) {
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
        if ch.is_none() && is_modifier {
            // A bare Shift types nothing and must not end the word: `aS` is `á`
            // and `DD` is `Đ` (20-spec A12). A Ctrl/Alt/Win chord was already
            // handled above, where it does reset.
            trace.stage("modifier");
            return Decision::PASS;
        }
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
            self.engine.set_macro_only(self.macro_only_active());
            globals::fire_enabled(enabled);
            globals::fire_command(BackendCommand::Enable(enabled));
        }
        self.reset_engine();
    }

    /// Vietnamese is off but the owner still wants gõ tắt to work.
    fn macro_only_active(&self) -> bool {
        !self.effective_enabled()
            && self
                .cfg
                .as_ref()
                .is_some_and(|cfg| cfg.typing.macro_when_disabled)
    }

    pub fn effective_enabled(&self) -> bool {
        if self.auto_excluded {
            return false;
        }
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

    /// Apply a configured hotkey. Returns `false` when the action had nothing to
    /// do and the trigger key must keep its normal meaning in the target app.
    fn on_hotkey(&mut self, hotkey: Hotkey) -> bool {
        let handled = match hotkey {
            Hotkey::Enable => {
                self.set_enabled(true);
                true
            }
            Hotkey::Disable => {
                self.set_enabled(false);
                true
            }
            Hotkey::Toggle => {
                let next = !self.effective_enabled();
                self.set_enabled(next);
                true
            }
            Hotkey::RestoreWord => {
                let rewritten = self.restore_word();
                globals::fire_command(BackendCommand::RestoreWord);
                rewritten
            }
            Hotkey::AppExclude => {
                globals::fire_command(BackendCommand::ToggleAppExclude);
                true
            }
            Hotkey::MacroPanel => {
                globals::fire_command(BackendCommand::OpenMacroPanel);
                true
            }
            Hotkey::MacroToggle => {
                globals::fire_command(BackendCommand::ToggleMacro);
                true
            }
            Hotkey::OpenSettings => {
                globals::fire_command(BackendCommand::OpenSettings);
                true
            }
            Hotkey::CycleMethod => {
                let next = next_method(self.base.method);
                self.base.method = next;
                globals::fire_command(BackendCommand::SetMethod(next));
                true
            }
        };
        self.reset_engine();
        globals::fire_hotkey(hotkey);
        handled
    }

    /// `Khôi phục từ gõ sai`: put the raw keystrokes of the current word back.
    ///
    /// The word buffer lives on the hook thread, so the rewrite has to happen
    /// here; `BackendCommand::RestoreWord` stays the app-side notification.
    /// Returns `false` when there is nothing to restore, so the chord keeps its
    /// normal meaning in the app (EVKey lineage: `Ctrl+Shift+Z` is also *redo*).
    fn restore_word(&mut self) -> bool {
        if self.engine_failed {
            return false;
        }
        let engine = &mut self.engine;
        let action = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            engine.restore_word()
        }))
        .unwrap_or(EngineAction::Noop);
        let decision = decide(&action, None);
        if !decision.swallow {
            return false;
        }
        if let Some(sink) = self.sink.as_mut() {
            if let Err(err) = sink.emit(action) {
                tracing::warn!(error = %err, "engine sink failed");
            }
        } else {
            let chord = self.restore_chord();
            let _ = self.execute_with(&decision, chord);
        }
        true
    }

    /// Modifiers of the configured `restore_word` chord.
    ///
    /// The chord fires inside the hook callback of its final key-up, so the
    /// injection must release these itself (`input::build_restore`); otherwise
    /// the target still sees them held and the text arrives shifted (CH2).
    fn restore_chord(&self) -> Modifiers {
        self.cfg
            .as_ref()
            .and_then(|cfg| cfg.hotkeys.restore_word.as_ref())
            .map_or(
                // No config yet: the shipped default chord is `Ctrl+Shift+Z`.
                Modifiers {
                    shift: true,
                    ctrl: true,
                    alt: false,
                    win: false,
                },
                |hotkey| Modifiers {
                    shift: hotkey.modifiers.shift,
                    ctrl: hotkey.modifiers.ctrl,
                    alt: hotkey.modifiers.alt,
                    win: hotkey.modifiers.win,
                },
            )
    }

    /// `true` when the restore hotkey has a word to rewrite.
    fn engine_can_restore(&self) -> bool {
        !self.engine_failed && self.engine.can_restore()
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
        self.execute_with(decision, Modifiers::NONE)
    }

    /// `release_chord`: the injection follows a modifier-chord hotkey
    /// (`restore_word`), so the batch must neutralise the chord's modifiers
    /// first — see [`input::build_restore`].
    fn execute_with(&mut self, decision: &Decision, release_chord: Modifiers) -> Option<SendResult> {
        match &decision.injection {
            Injection::None => None,
            Injection::Beep => {
                if self.sound_on {
                    // Error cue, not a toggle state: `play_error()` is the
                    // `SystemHand` alias (`22-sound-spec.md` §5).
                    sound::play_error();
                }
                None
            }
            Injection::Type(text) => Some(self.deliver_with(text, 0, release_chord)),
            Injection::Replace { deletes, insert } => {
                Some(self.deliver_with(insert, *deletes, release_chord))
            }
        }
    }

    /// Batched delivery: deletes and text go out in one `SendInput` call.
    fn deliver_with(&mut self, text: &str, deletes: u32, chord: Modifiers) -> SendResult {
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
        let build = |deletes: u32, text: &str, mode: SendMode| {
            if chord == Modifiers::NONE {
                input::build_segment(deletes, text, mode)
            } else {
                input::build_restore(deletes, text, mode, chord)
            }
        };
        let inputs = build(deletes, text, self.send_mode);
        let mut result = input::send_detailed(&inputs);
        if result.total > 0 && result.sent < result.total {
            // The app refused (part of) the batch. Step down one strategy for
            // this app: Standard scancodes first, then exclude it, so the user
            // keeps typing raw instead of losing characters.
            self.on_short_send();
            if !self.auto_excluded {
                let retry = build(deletes, text, self.send_mode);
                result = input::send_detailed(&retry);
                if result.total > 0 && result.sent < result.total {
                    self.auto_excluded = true;
                }
            }
        }
        result
    }

    /// After a short `SendInput`, decide what this app gets next.
    fn on_short_send(&mut self) {
        if self.send_mode == SendMode::Unicode && !self.force_standard {
            self.force_standard = true;
            self.send_mode = SendMode::Standard;
        } else {
            self.auto_excluded = true;
        }
    }

    fn reset_engine(&mut self) {
        if !self.engine_failed {
            self.engine.reset();
        }
    }

    /// The zero-focus harness must not depend on (or be disturbed by) which
    /// window the owner happens to have focused: no app rules, no IME state.
    #[cfg(feature = "e2e-harness")]
    fn refresh_app(&mut self) {
        self.ime_active = false;
    }

    #[cfg(not(feature = "e2e-harness"))]
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
            self.force_standard = false;
            self.auto_excluded = false;
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
        // Toggling Vietnamese on/off changes whether macros still run.
        self.engine.set_macro_only(self.macro_only_active());
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

    fn set_trigger_swallowed(&mut self, vk: u16, on: bool) {
        let index = vk as usize;
        let bit = 1u64 << (index % 64);
        if on {
            self.trigger_swallowed[index / 64] |= bit;
        } else {
            self.trigger_swallowed[index / 64] &= !bit;
        }
    }

    fn trigger_is_swallowed(&self, vk: u16) -> bool {
        let index = vk as usize;
        self.trigger_swallowed[index / 64] & (1u64 << (index % 64)) != 0
    }

    /// `true` once, for the key-up that closes a swallowed trigger key-down.
    fn take_trigger_swallowed(&mut self, vk: u16) -> bool {
        let index = vk as usize;
        let bit = 1u64 << (index % 64);
        let was = self.trigger_swallowed[index / 64] & bit != 0;
        self.trigger_swallowed[index / 64] &= !bit;
        was
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
    fn packet_w_transforms_with_empty_word() {
        let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut rt = runtime();
        // Telex `w` becomes `ư` on its own, so a packet carrying `w` must make
        // the engine act instead of being passed through.
        let info = KBDLLHOOKSTRUCT {
            vkCode: u32::from(keys::VK_PACKET),
            scanCode: u32::from('w'),
            flags: Default::default(),
            time: 0,
            dwExtraInfo: 0,
        };
        let decision = rt.on_event(WM_KEYDOWN, &info);
        assert!(decision.swallow, "a packet `w` must be processed by the engine");
    }

    #[test]
    fn plain_packet_with_empty_word_is_noop() {
        let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut rt = runtime();
        let info = KBDLLHOOKSTRUCT {
            vkCode: u32::from(keys::VK_PACKET),
            scanCode: u32::from('a'),
            flags: Default::default(),
            time: 0,
            dwExtraInfo: 0,
        };
        let decision = rt.on_event(WM_KEYDOWN, &info);
        assert!(!decision.swallow);
    }

    #[test]
    fn control_packet_is_not_text() {
        let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut rt = runtime();
        let info = KBDLLHOOKSTRUCT {
            vkCode: u32::from(keys::VK_PACKET),
            scanCode: 0x0008,
            flags: Default::default(),
            time: 0,
            dwExtraInfo: 0,
        };
        assert_eq!(rt.on_event(WM_KEYDOWN, &info), Decision::PASS);
    }

    #[test]
    fn short_send_switches_to_standard_then_excludes() {
        let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut rt = runtime();
        rt.send_mode = SendMode::Unicode;
        assert!(rt.effective_enabled());
        rt.on_short_send();
        assert_eq!(rt.send_mode, SendMode::Standard, "first short send retries with scancodes");
        assert!(!rt.auto_excluded);
        rt.on_short_send();
        assert!(rt.auto_excluded, "second short send excludes the app");
        assert!(!rt.effective_enabled(), "an excluded app types raw");
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

    // -- D-E2E-02: `restore_word` must reach the word buffer ------------------

    struct Recorder(Arc<Mutex<Vec<EngineAction>>>);

    impl vkb_platform::EngineSink for Recorder {
        fn emit(&mut self, action: EngineAction) -> anyhow::Result<()> {
            self.0.lock().unwrap().push(action);
            Ok(())
        }

        fn active_app(&self) -> Option<vkb_platform::AppInfo> {
            None
        }
    }

    fn kb(vk: u32, scan: u32) -> KBDLLHOOKSTRUCT {
        KBDLLHOOKSTRUCT {
            vkCode: vk,
            scanCode: scan,
            flags: Default::default(),
            time: 0,
            dwExtraInfo: 0,
        }
    }

    /// Type through the packet path (no injection without a window) and return
    /// everything the engine decided, so the caller can inspect the last action.
    fn type_chars(rt: &mut HookRuntime, text: &str) {
        for ch in text.chars() {
            rt.on_event(WM_KEYDOWN, &kb(u32::from(keys::VK_PACKET), ch as u32));
            rt.on_event(WM_KEYUP, &kb(u32::from(keys::VK_PACKET), ch as u32));
        }
    }

    #[test]
    fn restore_word_hotkey_rewrites_the_word_on_release() {
        let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        globals::store_enabled(true);
        let toml = r#"
version = 1
[input]
enabled = true
method = "vni"
[typing]
send_standard = true
[hotkeys]
toggle = "Alt+Shift"
restore_word = "Ctrl+Shift+Z"
toggle_mode = "release"
"#;
        let cfg = AppConfig::from_toml(toml).unwrap();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let mut rt = HookRuntime::new(
            Some(Arc::new(cfg)),
            Some(Box::new(Recorder(Arc::clone(&seen)))),
        );

        type_chars(&mut rt, "ba6t5");

        // Ctrl+Shift+Z with left/right modifier VKs, as SendInput delivers them.
        rt.on_event(WM_KEYDOWN, &kb(0xA2, 0x1D));
        rt.on_event(WM_KEYDOWN, &kb(0xA0, 0x2A));
        rt.on_event(WM_KEYDOWN, &kb(0x5A, 0x2C));
        rt.on_event(WM_KEYUP, &kb(0x5A, 0x2C));
        rt.on_event(WM_KEYUP, &kb(0xA0, 0x2A));
        rt.on_event(WM_KEYUP, &kb(0xA2, 0x1D));

        let actions = seen.lock().unwrap();
        let last = actions.last().expect("the hook must emit the restore");
        assert_eq!(
            *last,
            EngineAction::Replace {
                delete_units: 3,
                insert: "ba6t5".to_string()
            },
            "the chord must rewrite `bật` back to the raw keystrokes"
        );
        drop(actions);

        // A settings save republishes the config from the UI thread; the chord
        // must keep working after the matcher is rebuilt.
        let cfg = AppConfig::from_toml(toml).unwrap();
        rt.apply_config(Arc::new(cfg));
        type_chars(&mut rt, "ba6t5");
        rt.on_event(WM_KEYDOWN, &kb(0xA2, 0x1D));
        rt.on_event(WM_KEYDOWN, &kb(0xA0, 0x2A));
        rt.on_event(WM_KEYDOWN, &kb(0x5A, 0x2C));
        rt.on_event(WM_KEYUP, &kb(0x5A, 0x2C));
        rt.on_event(WM_KEYUP, &kb(0xA0, 0x2A));
        rt.on_event(WM_KEYUP, &kb(0xA2, 0x1D));
        let actions = seen.lock().unwrap();
        assert_eq!(
            actions.last(),
            Some(&EngineAction::Replace {
                delete_units: 3,
                insert: "ba6t5".to_string()
            }),
            "the chord must survive a config republish"
        );
        drop(actions);

        // No word to restore: the chord keeps its meaning in the app (redo) and
        // the hook must not swallow the trigger key.
        rt.on_event(WM_KEYDOWN, &kb(0xA2, 0x1D));
        rt.on_event(WM_KEYDOWN, &kb(0xA0, 0x2A));
        let decision = rt.on_event(WM_KEYDOWN, &kb(0x5A, 0x2C));
        assert!(
            !decision.swallow,
            "`Ctrl+Shift+Z` must reach the app when there is nothing to restore"
        );
        rt.on_event(WM_KEYUP, &kb(0x5A, 0x2C));
        rt.on_event(WM_KEYUP, &kb(0xA0, 0x2A));
        rt.on_event(WM_KEYUP, &kb(0xA2, 0x1D));
    }

    /// CH2: the injection runs inside the hook callback of the chord's last
    /// key-up, so the restore batch must release the chord's own modifiers —
    /// taken from the configured `restore_word`, not hardcoded.
    #[test]
    fn restore_releases_the_configured_chord() {
        let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        globals::store_enabled(true);
        let chord = |spec: &str| {
            let toml = format!(
                "version = 1\n[input]\nenabled = true\nmethod = \"vni\"\n\
                 [typing]\nsend_standard = false\n[hotkeys]\nrestore_word = \"{spec}\"\n"
            );
            let cfg = AppConfig::from_toml(&toml).unwrap();
            HookRuntime::new(Some(Arc::new(cfg)), None).restore_chord()
        };
        assert_eq!(
            chord("Ctrl+Shift+Z"),
            vkb_core::Modifiers {
                shift: true,
                ctrl: true,
                alt: false,
                win: false
            }
        );
        assert_eq!(
            chord("Ctrl+Alt+Z"),
            vkb_core::Modifiers {
                shift: false,
                ctrl: true,
                alt: true,
                win: false
            }
        );
        assert_eq!(
            chord("Shift+F24"),
            vkb_core::Modifiers {
                shift: true,
                ctrl: false,
                alt: false,
                win: false
            }
        );
    }

    // -- hotkey trigger swallowing (D-E2E-03) and the mouse boundary ----------

    fn quiet_runtime() -> HookRuntime {
        globals::store_enabled(true);
        let mut cfg = AppConfig::default();
        cfg.system.notification_sound = false;
        HookRuntime::new(Some(Arc::new(cfg)), None)
    }

    fn key(rt: &mut HookRuntime, vk: u16, down: bool) -> Decision {
        let info = KBDLLHOOKSTRUCT {
            vkCode: u32::from(vk),
            scanCode: 0,
            flags: Default::default(),
            time: 0,
            dwExtraInfo: 0,
        };
        rt.on_event(if down { WM_KEYDOWN } else { WM_KEYUP }, &info)
    }

    #[test]
    fn release_mode_trigger_down_and_up_are_swallowed() {
        let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut rt = quiet_runtime();
        assert!(key(&mut rt, keys::VK_F2, true).swallow, "F2 down must not reach the app");
        assert!(key(&mut rt, keys::VK_F2, false).swallow, "and neither does its key-up");
        assert!(!globals::enabled(), "F2 disables on release");
        assert!(key(&mut rt, keys::VK_F1, true).swallow);
        assert!(key(&mut rt, keys::VK_F1, false).swallow);
        assert!(globals::enabled());
        assert!(!key(&mut rt, keys::VK_F1, false).swallow, "a lone key-up is not ours");
    }

    #[test]
    fn auto_repeat_follows_the_bit_not_the_latch() {
        let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut rt = quiet_runtime();
        assert!(key(&mut rt, keys::VK_F1, true).swallow);
        // The chord's latch is gone (a modifier went up mid-hold) ...
        rt.hotkeys.discard_latches();
        // ... yet the repeat of the swallowed key-down is swallowed too.
        assert!(key(&mut rt, keys::VK_F1, true).swallow, "repeat must not leak");
        assert!(key(&mut rt, keys::VK_F1, false).swallow);
        assert!(!rt.trigger_is_swallowed(keys::VK_F1), "key-up clears the bit");
    }

    #[test]
    fn stale_bit_is_cleared_by_a_fresh_key_down() {
        let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut rt = quiet_runtime();
        rt.set_trigger_swallowed(0x41, true); // a lost key-up left this behind
        let decision = key(&mut rt, 0x41, true);
        assert!(!decision.swallow, "a fresh key-down must pass");
        assert!(!rt.trigger_is_swallowed(0x41), "and drop the stale bit");
    }

    #[test]
    fn bit_is_set_only_when_the_key_down_was_swallowed() {
        let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut rt = quiet_runtime();
        // restore_word with nothing to restore passes (redo must reach the app).
        key(&mut rt, keys::VK_LCONTROL, true);
        key(&mut rt, keys::VK_LSHIFT, true);
        let z = key(&mut rt, 0x5A, true);
        assert!(!z.swallow, "nothing to restore: Ctrl+Shift+Z passes");
        assert!(!rt.trigger_is_swallowed(0x5A), "so no bit");
        key(&mut rt, 0x5A, false);
        key(&mut rt, keys::VK_LSHIFT, false);
        key(&mut rt, keys::VK_LCONTROL, false);
    }

    #[test]
    fn apply_config_clears_the_swallowed_bits() {
        let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut rt = quiet_runtime();
        assert!(key(&mut rt, keys::VK_F1, true).swallow);
        assert!(rt.trigger_is_swallowed(keys::VK_F1));
        let mut cfg = AppConfig::default();
        cfg.system.notification_sound = false;
        rt.apply_config(Arc::new(cfg));
        assert!(!rt.trigger_is_swallowed(keys::VK_F1));
    }

    #[test]
    fn mouse_button_down_ends_the_word() {
        let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut rt = quiet_runtime();
        // `aa` composes `â`, so the second key is swallowed ...
        assert!(!key(&mut rt, 0x41, true).swallow);
        assert!(key(&mut rt, 0x41, true).swallow, "aa -> a-circumflex");
        key(&mut rt, 0x41, false);
        key(&mut rt, keys::VK_SPACE, true);
        // ... but a click between the two keys starts a new word.
        assert!(!key(&mut rt, 0x41, true).swallow);
        globals::note_mouse_down();
        assert!(!key(&mut rt, 0x41, true).swallow, "after a click the word is over");
    }
}
