//! The tap thread's private runtime: raw macOS events in, swallow/pass decisions
//! and injections out. Mirrors `vkb-windows/src/state.rs`; the OS is reached
//! only through [`Host`], so the whole state machine is unit-tested without an
//! event tap, Accessibility or any synthetic input.
//!
//! One instance lives on the tap thread. No lock is taken on the keystroke path
//! and nothing allocates unless text is actually injected.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use vkb_config::AppConfig;
use vkb_core::{
    Engine, EngineAction, EngineOptions, InputMethod, KeyInput, Modifiers, VietnameseEngine,
};
use vkb_platform::hotkeys::Matcher;
use vkb_platform::plan::{decide, keep_unchanged_head, Decision, HeldKeys, Injection};
use vkb_platform::rules::{self, BaseSettings, EffectiveSettings};
use vkb_platform::{BackendCommand, EngineSink, Hotkey};

use crate::globals;
use crate::hotkeys as hotkey_map;
use crate::keys::{self, ModifierState};

/// Value stored in `kCGEventSourceUserData` of every event we inject ("VKMAC1").
/// The tap passes tagged events through untouched and never feeds them back.
pub const MARKER: i64 = 0x0000_564B_4D41_4331;

/// `true` for an event this backend injected.
#[inline]
pub fn is_injected(user_data: i64) -> bool {
    user_data == MARKER
}

/// How long a bundle-id lookup is reused while the frontmost pid is unchanged.
const APP_TTL: Duration = Duration::from_millis(1500);
/// A swallowed trigger counts as still held (a repeat is swallowed) only while
/// its last swallowed down is this recent.
/// Minimum gap between two "secure input blocks the tap" warnings.
const SECURE_LOG_INTERVAL: Duration = Duration::from_secs(60);
const TRIGGER_REPEAT_WINDOW: Duration = Duration::from_millis(1200);

/// What the tap callback received.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventKind {
    KeyDown,
    KeyUp,
    FlagsChanged,
    /// Any mouse button down: the caret may have moved, the word is over.
    MouseDown,
}

/// One event, already read out of the `CGEvent` by the FFI layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RawEvent {
    pub kind: EventKind,
    /// `kCGKeyboardEventKeycode` (layout independent).
    pub keycode: u16,
    /// `CGEventFlags`.
    pub flags: u64,
    /// `kCGKeyboardEventAutorepeat`.
    pub autorepeat: bool,
    /// The single character the event types under the active layout and
    /// modifiers (`CGEventKeyboardGetUnicodeString`), if exactly one.
    pub typed: Option<char>,
    /// `kCGEventSourceUserData`.
    pub user_data: i64,
}

impl RawEvent {
    pub fn key(kind: EventKind, keycode: u16, flags: u64, typed: Option<char>) -> Self {
        Self {
            kind,
            keycode,
            flags,
            autorepeat: false,
            typed,
            user_data: 0,
        }
    }
}

/// The operating-system services the runtime needs.
pub trait Host: Send {
    /// Frontmost application pid (0 = unknown). Cheap: called on every key-down.
    fn frontmost_pid(&mut self) -> i32;
    /// Lowercased bundle identifier of the frontmost application. Allocates;
    /// called only when the pid changes or the TTL lapses.
    fn frontmost_bundle(&mut self) -> Option<String>;
    /// `IsSecureEventInputEnabled()`.
    fn secure_input(&mut self) -> bool;
    /// Delete `deletes` characters backwards, then type `text`, in one batch.
    fn replace(&mut self, deletes: u32, text: &str);
    /// Re-post a physical key as a tagged press (down + up) with empty flags.
    /// Used after a macro expansion so the original Enter/Tab lands after the text.
    fn repost_key(&mut self, keycode: u16);
    fn toggle_sound(&mut self, enabled: bool);
    fn error_sound(&mut self);
}

/// The `Đổi kiểu gõ` cycle: Telex <-> VNI.
pub fn next_method(current: InputMethod) -> InputMethod {
    match current {
        InputMethod::Telex | InputMethod::Legacy => InputMethod::Vni,
        InputMethod::Vni => InputMethod::Telex,
    }
}

/// The context a half-typed word belongs to.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct WordContext {
    pub pid: i32,
    pub enabled: bool,
}

/// `true` when the pending word must be dropped before this key is processed.
pub(crate) fn context_changed(prev: Option<WordContext>, next: WordContext) -> bool {
    match prev {
        Some(prev) => prev.pid != next.pid || prev.enabled != next.enabled,
        None => false,
    }
}

pub struct Runtime {
    host: Box<dyn Host>,
    engine: Box<dyn VietnameseEngine>,
    engine_failed: bool,
    macro_fingerprint: u64,
    options: EngineOptions,
    hotkeys: Matcher,
    mods: ModifierState,
    /// Bitset of physical keys (by Windows VK) currently down.
    down: [u64; 4],
    /// Keys whose key-down was swallowed as a hotkey trigger (the key-up follows).
    trigger_swallowed: [u64; 4],
    trigger_last: Option<(u16, Instant)>,
    cfg: Option<Arc<AppConfig>>,
    config_version: u64,
    base: BaseSettings,
    rule: Option<vkb_config::AppRule>,
    app_pid: i32,
    app_name: String,
    app_checked: Instant,
    word_context: Option<WordContext>,
    resolved: Option<EffectiveSettings>,
    remembered: HashMap<String, bool>,
    sound_on: bool,
    remember_per_app: bool,
    last_global: bool,
    config_enabled: Option<bool>,
    sink: Option<Box<dyn EngineSink>>,
    secure_logged: Option<Instant>,
}

impl Runtime {
    pub fn new(
        host: Box<dyn Host>,
        cfg: Option<Arc<AppConfig>>,
        sink: Option<Box<dyn EngineSink>>,
    ) -> Self {
        let mut runtime = Self {
            host,
            engine: Box::new(Engine::new(EngineOptions::default())),
            engine_failed: false,
            macro_fingerprint: 0,
            options: EngineOptions::default(),
            hotkeys: hotkey_map::compile(&vkb_config::HotkeySet::default()),
            mods: ModifierState::default(),
            down: [0; 4],
            trigger_swallowed: [0; 4],
            trigger_last: None,
            cfg: None,
            config_version: globals::config_version(),
            base: BaseSettings::default(),
            rule: None,
            app_pid: 0,
            app_name: String::new(),
            app_checked: Instant::now() - APP_TTL,
            word_context: None,
            resolved: None,
            remembered: HashMap::new(),
            sound_on: true,
            remember_per_app: false,
            last_global: globals::enabled(),
            config_enabled: None,
            sink,
            secure_logged: None,
        };
        match cfg {
            Some(cfg) => runtime.apply_config(cfg),
            None => runtime.apply_engine_options(),
        }
        runtime
    }

    /// Re-read the shared config when the UI published a new one.
    pub fn poll_config(&mut self) {
        let version = globals::config_version();
        if version != self.config_version {
            self.config_version = version;
            if let Some(cfg) = globals::snapshot() {
                self.apply_config(cfg);
            }
        }
    }

    /// Replace the settings the tap consults. Called on the tap thread only.
    pub fn apply_config(&mut self, cfg: Arc<AppConfig>) {
        self.base = BaseSettings {
            enabled: cfg.input.enabled,
            method: cfg.input.method,
            charset: cfg.input.charset,
            macro_enabled: cfg.typing.macro_enabled,
        };
        self.hotkeys = hotkey_map::compile(&cfg.hotkeys);
        self.sound_on = cfg.system.notification_sound;
        self.remember_per_app = cfg.apps.remember_per_app;

        // Adopt `input.enabled` only when it changed in the file, so a hotkey
        // toggle is not reset by an unrelated settings save.
        if self.config_enabled != Some(cfg.input.enabled) {
            globals::store_enabled(cfg.input.enabled);
            self.last_global = cfg.input.enabled;
        }
        self.config_enabled = Some(cfg.input.enabled);

        self.cfg = Some(cfg);
        self.app_pid = 0;
        self.word_context = None;
        self.app_name.clear();
        self.resolved = None;
        self.rule = None;
        self.apply_engine_options();
        self.reset_engine();
        self.hotkeys.discard_latches();
        self.trigger_swallowed = [0; 4];
    }

    /// Forget the word and every latched hotkey (tap re-enabled after a timeout).
    /// `trigger_swallowed` is kept: the key-up of a swallowed trigger must still
    /// be swallowed, or the app would see an orphan key-up.
    pub fn reset(&mut self) {
        self.reset_engine();
        self.hotkeys.discard_latches();
        self.down = [0; 4];
    }

    /// Hand the injection sink back (tap creation failed before it was used).
    pub fn take_sink(&mut self) -> Option<Box<dyn EngineSink>> {
        self.sink.take()
    }

    /// Feed one tap event. The returned decision says whether to swallow it.
    pub fn on_event(&mut self, ev: &RawEvent) -> Decision {
        // Our own injections re-enter the tap; never feed them back.
        if is_injected(ev.user_data) {
            return Decision::PASS;
        }
        if ev.kind == EventKind::MouseDown {
            // Any mouse button: the caret may have moved ("Mouse button down
            // -> boundary()" in 20-spec).
            self.boundary_engine();
            return Decision::PASS;
        }
        if ev.kind == EventKind::KeyDown && self.host.secure_input() {
            // Password fields: never convert, never swallow new keys. A trigger
            // already swallowed keeps its key-up swallowed (bits are kept).
            if self
                .secure_logged
                .map_or(true, |at| at.elapsed() >= SECURE_LOG_INTERVAL)
            {
                self.secure_logged = Some(Instant::now());
                tracing::warn!("secure event input is on; VietKB is passing keys through");
            }
            self.reset_engine();
            self.hotkeys.discard_latches();
            return Decision::PASS;
        }
        self.route(ev)
    }

    fn route(&mut self, ev: &RawEvent) -> Decision {
        let vk = keys::vk_for_keycode(ev.keycode);
        self.mods.sync(ev.flags);

        let down = match ev.kind {
            EventKind::KeyDown => true,
            EventKind::KeyUp => false,
            EventKind::FlagsChanged => {
                if ev.keycode == keys::MAC_CAPS_LOCK {
                    // Caps Lock state is in the flags; the key ends the word
                    // exactly like the Windows hook's non-text key path.
                    self.reset_engine();
                    return Decision::PASS;
                }
                if !keys::is_modifier(vk) {
                    return Decision::PASS; // fn and unknown modifiers
                }
                keys::modifier_key_down(ev.keycode, ev.flags)
            }
            EventKind::MouseDown => return Decision::PASS,
        };

        let is_modifier = keys::is_modifier(vk);
        let modifiers = self.mods.modifiers();
        let repeat = down && ev.autorepeat;
        self.set_down(vk, down);
        self.sync_global_toggle();

        // A swallowed trigger stays swallowed while held (auto-repeat).
        if down && !is_modifier && self.trigger_is_swallowed(vk) {
            let held = self
                .trigger_last
                .is_some_and(|(key, at)| key == vk && at.elapsed() < TRIGGER_REPEAT_WINDOW);
            if repeat && held {
                self.trigger_last = Some((vk, Instant::now()));
                return Decision::SWALLOW;
            }
            self.set_trigger_swallowed(vk, false);
        }

        // 1. Hotkeys first: the trigger must never reach the app.
        if let Some(hotkey) = self.hotkeys.on_key(vk, down, modifiers, is_modifier) {
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
            // Modifier-only combos (Ctrl+Shift) are matched but passed through.
        } else if !is_modifier {
            if down {
                if self.hotkeys.swallows_trigger(vk) {
                    self.set_trigger_swallowed(vk, true);
                    return Decision::SWALLOW;
                }
            } else if self.take_trigger_swallowed(vk) {
                return Decision::SWALLOW;
            }
        }

        if !down {
            return Decision::PASS;
        }

        // 2. Shortcut chords are never text.
        if self.mods.has_shortcut_modifier() {
            if is_modifier {
                // A modifier on its own types nothing and may be half of a
                // chord whose key needs the word (`restore_word`).
                return Decision::PASS;
            }
            // A release-armed hotkey owns this key while the word is there.
            if self.hotkeys.awaiting_release() && self.engine_can_restore() {
                return Decision::SWALLOW;
            }
            self.reset_engine();
            return Decision::PASS;
        }

        // 3. Per-app enablement.
        self.refresh_app();
        self.sync_word_context();
        let enabled = self.effective_enabled();
        if !enabled && !self.macro_only_active() {
            self.reset_engine();
            return Decision::PASS;
        }

        // 4. Caret movement and other commands end the current word.
        if keys::is_caret_mover(vk) {
            self.reset_engine();
            return Decision::PASS;
        }

        // 5. Text keys plus the boundary keys the engine cares about.
        let ch = self.char_for(vk, ev.typed);
        if ch.is_none() && is_modifier {
            // A bare Shift types nothing and must not end the word (`aS`).
            return Decision::PASS;
        }
        if ch.is_none() && !keys::is_word_boundary(vk) {
            self.reset_engine();
            return Decision::PASS;
        }

        let key = KeyInput {
            vk,
            scan: ev.keycode,
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
        if matches!(action, EngineAction::ReplaceThenPass { .. }) {
            // Macro committed by Enter/Tab: swallow the physical key, type the
            // expansion, then re-inject the key as a tagged event so the order
            // never depends on how the OS interleaves our posts with the key.
            self.deliver(action, &decision);
            self.host.repost_key(ev.keycode);
            return Decision::SWALLOW;
        }
        if decision.swallow {
            self.deliver(action, &decision);
        }
        decision
    }

    /// Route an engine decision to the injection sink or the host.
    fn deliver(&mut self, action: EngineAction, decision: &Decision) {
        if self.sink.is_some() {
            let held = self.held();
            if let Some(sink) = self.sink.as_mut() {
                if let Err(err) = sink.emit_with(action, held) {
                    tracing::warn!(error = %err, "engine sink failed");
                }
            }
            return;
        }
        self.execute(decision);
    }

    fn execute(&mut self, decision: &Decision) {
        match &decision.injection {
            Injection::None => {}
            Injection::Beep => {
                if self.sound_on {
                    self.host.error_sound();
                }
            }
            Injection::Type(text) => self.host.replace(0, text),
            Injection::Replace { deletes, insert } => self.host.replace(*deletes, insert),
        }
    }

    /// Flip Vietnamese on/off. Idempotent and never latches.
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
                self.host.toggle_sound(enabled);
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
        handled
    }

    /// `Khôi phục từ gõ sai`: put the raw keystrokes of the current word back.
    /// Returns `false` when there is nothing to restore, so the chord keeps its
    /// normal meaning in the app (`Ctrl+Shift+Z` is also redo elsewhere).
    fn restore_word(&mut self) -> bool {
        if self.engine_failed {
            return false;
        }
        let engine = &mut self.engine;
        let action = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let shown: vkb_core::charset::TextBuf = engine.on_screen().iter().copied().collect();
            keep_unchanged_head(&shown, engine.restore_word())
        }))
        .unwrap_or(EngineAction::Noop);
        let decision = decide(&action, None);
        if !decision.swallow {
            return false;
        }
        self.deliver(action, &decision);
        true
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
            // The word the app shows before this key, so the rewrite can leave
            // its unchanged head alone (D-E2E-06).
            let shown: vkb_core::charset::TextBuf = engine.on_screen().iter().copied().collect();
            keep_unchanged_head(&shown, engine.process_key(key))
        }));
        match result {
            Ok(action) => action,
            Err(_) => {
                self.engine_failed = true;
                tracing::error!("engine panicked; Vietnamese input suspended");
                EngineAction::Noop
            }
        }
    }

    /// The Shift keys and Caps Lock held at the event being handled. Injected
    /// events carry empty flags on a private source, so macOS needs no Shift
    /// juggling; the value is still handed to an `EngineSink`.
    fn held(&self) -> HeldKeys {
        HeldKeys {
            shift: self.mods.shift_keys,
            caps: self.mods.caps_lock,
        }
    }

    fn reset_engine(&mut self) {
        if !self.engine_failed {
            self.engine.reset();
        }
    }

    fn boundary_engine(&mut self) {
        if !self.engine_failed {
            self.engine.boundary();
        }
    }

    /// The character the engine sees for this key.
    ///
    /// Keypad and function keys never type. Otherwise the layout-resolved
    /// character comes from the event itself, so non-US layouts work. Control
    /// characters only pass for the boundary keys the engine knows by VK.
    fn char_for(&self, vk: u16, typed: Option<char>) -> Option<char> {
        if keys::never_text(vk) {
            return None;
        }
        if let Some(ch) = typed {
            if ch.is_ascii_alphabetic() {
                // Synthetic events (UI automation, remote-desktop and remapping
                // tools) take their string from the keycode alone and ignore
                // the Shift / Caps Lock flags they carry, so `V` arrives as
                // `v`. The flags are what the app applies, so they decide the
                // case (D-MAC-01).
                return Some(if self.mods.shift ^ self.mods.caps_lock {
                    ch.to_ascii_uppercase()
                } else {
                    ch.to_ascii_lowercase()
                });
            }
            if !ch.is_control() || keys::is_word_boundary(vk) {
                return Some(ch);
            }
            return None;
        }
        match vk {
            keys::VK_SPACE => Some(' '),
            keys::VK_TAB => Some('\t'),
            keys::VK_RETURN => Some('\r'),
            // The event carried no string for a letter: take the US character.
            0x41..=0x5A => keys::us_char(vk, self.mods.shift, self.mods.caps_lock),
            _ => None,
        }
    }

    fn refresh_app(&mut self) {
        let now = Instant::now();
        let pid = self.host.frontmost_pid();
        if pid != 0
            && pid == self.app_pid
            && now.saturating_duration_since(self.app_checked) < APP_TTL
        {
            return;
        }
        self.app_checked = now;
        if pid == 0 {
            self.app_pid = 0;
            self.app_name.clear();
            self.resolved = None;
            self.rule = None;
            return;
        }
        let name = self.host.frontmost_bundle().unwrap_or_default();
        if pid != self.app_pid || name != self.app_name {
            self.app_pid = pid;
            self.app_name = name;
            self.resolve_app();
        }
    }

    /// Flush a word that belongs to a previous context (another app, or the
    /// same app after an exclusion/override flipped).
    fn sync_word_context(&mut self) {
        let next = WordContext {
            pid: self.app_pid,
            enabled: self.effective_enabled(),
        };
        if context_changed(self.word_context, next) {
            self.boundary_engine();
        }
        self.word_context = Some(next);
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

    // MSRV is 1.80; `is_none_or` needs 1.82, so `map_or(true, ..)` stays.
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
        // Macros live outside `EngineOptions`; push them when the table changes.
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
        self.engine.set_macro_only(self.macro_only_active());
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

    fn set_trigger_swallowed(&mut self, vk: u16, on: bool) {
        let index = vk as usize;
        let bit = 1u64 << (index % 64);
        if on {
            self.trigger_last = Some((vk, Instant::now()));
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

/// Modifiers of the default `restore_word` chord, for hosts that must release it.
pub const DEFAULT_RESTORE_CHORD: Modifiers = Modifiers {
    shift: true,
    ctrl: true,
    alt: false,
    win: false,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::flags;
    use parking_lot::Mutex;
    use vkb_config::Hotkey as CfgHotkey;

    #[derive(Default)]
    struct Log {
        replaced: Vec<(u32, String)>,
        toggles: Vec<bool>,
        reposted: Vec<u16>,
        errors: u32,
    }

    struct MockHost {
        log: Arc<Mutex<Log>>,
        pid: i32,
        bundle: String,
        secure: Arc<Mutex<bool>>,
    }

    impl Host for MockHost {
        fn frontmost_pid(&mut self) -> i32 {
            self.pid
        }
        fn frontmost_bundle(&mut self) -> Option<String> {
            Some(self.bundle.clone())
        }
        fn secure_input(&mut self) -> bool {
            *self.secure.lock()
        }
        fn replace(&mut self, deletes: u32, text: &str) {
            self.log.lock().replaced.push((deletes, text.to_string()));
        }
        fn repost_key(&mut self, keycode: u16) {
            self.log.lock().reposted.push(keycode);
        }
        fn toggle_sound(&mut self, enabled: bool) {
            self.log.lock().toggles.push(enabled);
        }
        fn error_sound(&mut self) {
            self.log.lock().errors += 1;
        }
    }

    struct Rig {
        rt: Runtime,
        log: Arc<Mutex<Log>>,
        secure: Arc<Mutex<bool>>,
    }

    fn rig(cfg: Option<AppConfig>) -> Rig {
        let log = Arc::new(Mutex::new(Log::default()));
        let secure = Arc::new(Mutex::new(false));
        globals::store_enabled(true);
        let host = MockHost {
            log: log.clone(),
            pid: 4242,
            bundle: "com.apple.textedit".to_string(),
            secure: secure.clone(),
        };
        Rig {
            rt: Runtime::new(Box::new(host), cfg.map(Arc::new), None),
            log,
            secure,
        }
    }

    fn quiet_config() -> AppConfig {
        let mut cfg = AppConfig::default();
        cfg.system.notification_sound = true;
        cfg
    }

    fn mac_code(ch: char) -> u16 {
        let (vk, _) = keys::us_vk_for_char(ch).unwrap();
        keys::keycode_for_vk(vk).unwrap()
    }

    fn press(rt: &mut Runtime, ch: char) -> Decision {
        let code = mac_code(ch);
        let down = RawEvent::key(EventKind::KeyDown, code, 0, Some(ch));
        let decision = rt.on_event(&down);
        rt.on_event(&RawEvent::key(EventKind::KeyUp, code, 0, Some(ch)));
        decision
    }

    #[test]
    fn marker_detection() {
        assert!(is_injected(MARKER));
        assert!(!is_injected(0));
        assert!(!is_injected(MARKER + 1));
    }

    #[test]
    fn injected_events_pass_straight_through() {
        let _guard = globals::TEST_LOCK.lock();
        let mut rig = rig(None);
        let mut ev = RawEvent::key(EventKind::KeyDown, mac_code('s'), 0, Some('s'));
        ev.user_data = MARKER;
        assert_eq!(rig.rt.on_event(&ev), Decision::PASS);
        assert!(rig.log.lock().replaced.is_empty());
    }

    #[test]
    fn telex_tone_is_injected_and_the_key_swallowed() {
        let _guard = globals::TEST_LOCK.lock();
        let mut rig = rig(None);
        // `a` types itself; `s` turns it into `á`: one delete, then the new text.
        assert_eq!(press(&mut rig.rt, 'a'), Decision::PASS);
        let decision = press(&mut rig.rt, 's');
        assert!(decision.swallow, "the tone key is consumed by the engine");
        assert_eq!(rig.log.lock().replaced, vec![(1, "á".to_string())]);
    }

    #[test]
    fn full_word_telex_viet() {
        let _guard = globals::TEST_LOCK.lock();
        let mut rig = rig(None);
        for ch in "vieetj".chars() {
            press(&mut rig.rt, ch);
        }
        let log = rig.log.lock();
        // Whatever the exact batching, the last injection must end the word as `ệt`.
        let (_, last) = log.replaced.last().expect("the engine injected");
        assert!(last.contains('ệ'), "got {last:?}");
    }

    #[test]
    fn plain_text_is_not_touched() {
        let _guard = globals::TEST_LOCK.lock();
        let mut rig = rig(None);
        for ch in "xyz".chars() {
            assert_eq!(press(&mut rig.rt, ch), Decision::PASS);
        }
        assert!(rig.log.lock().replaced.is_empty());
    }

    #[test]
    fn disabled_state_passes_everything() {
        let _guard = globals::TEST_LOCK.lock();
        let mut rig = rig(None);
        globals::store_enabled(false);
        press(&mut rig.rt, 'a');
        assert_eq!(press(&mut rig.rt, 's'), Decision::PASS);
        assert!(rig.log.lock().replaced.is_empty());
        globals::store_enabled(true);
    }

    #[test]
    fn secure_input_passes_everything_untouched() {
        let _guard = globals::TEST_LOCK.lock();
        let mut rig = rig(None);
        press(&mut rig.rt, 'a');
        *rig.secure.lock() = true;
        assert_eq!(press(&mut rig.rt, 's'), Decision::PASS);
        assert!(rig.log.lock().replaced.is_empty());
        // The word was dropped: after secure input ends `s` starts fresh.
        *rig.secure.lock() = false;
        assert_eq!(press(&mut rig.rt, 's'), Decision::PASS);
        assert!(rig.log.lock().replaced.is_empty());
    }

    #[test]
    fn mouse_click_ends_the_word() {
        let _guard = globals::TEST_LOCK.lock();
        let mut rig = rig(None);
        press(&mut rig.rt, 'a');
        let click = RawEvent::key(EventKind::MouseDown, 0, 0, None);
        assert_eq!(rig.rt.on_event(&click), Decision::PASS);
        assert_eq!(press(&mut rig.rt, 's'), Decision::PASS);
        assert!(rig.log.lock().replaced.is_empty());
    }

    #[test]
    fn caret_keys_end_the_word() {
        let _guard = globals::TEST_LOCK.lock();
        let mut rig = rig(None);
        press(&mut rig.rt, 'a');
        let left = RawEvent::key(EventKind::KeyDown, 123, 0, None);
        assert_eq!(rig.rt.on_event(&left), Decision::PASS);
        assert_eq!(press(&mut rig.rt, 's'), Decision::PASS);
        assert!(rig.log.lock().replaced.is_empty());
    }

    #[test]
    fn command_chords_are_shortcuts_not_text() {
        let _guard = globals::TEST_LOCK.lock();
        let mut rig = rig(None);
        press(&mut rig.rt, 'a');
        let cmd_c = RawEvent::key(
            EventKind::KeyDown,
            mac_code('c'),
            flags::COMMAND | flags::DEV_LCMD,
            Some('c'),
        );
        assert_eq!(rig.rt.on_event(&cmd_c), Decision::PASS);
        assert_eq!(press(&mut rig.rt, 's'), Decision::PASS, "word was reset by the chord");
    }

    #[test]
    fn keypad_digits_never_become_text() {
        let _guard = globals::TEST_LOCK.lock();
        let mut rig = rig(None);
        let kp1 = RawEvent::key(EventKind::KeyDown, 83, 0, Some('1'));
        assert_eq!(rig.rt.on_event(&kp1), Decision::PASS);
        assert!(rig.log.lock().replaced.is_empty());
        assert_eq!(rig.rt.char_for(keys::VK_NUMPAD0 + 1, Some('1')), None);
    }

    #[test]
    fn char_for_uses_the_layout_character_not_the_keycode() {
        let _guard = globals::TEST_LOCK.lock();
        let rig = rig(None);
        // The `Q` key on an AZERTY layout types `a`: the event's character wins.
        assert_eq!(rig.rt.char_for(0x51, Some('a')), Some('a'));
        // Dead key / no string: a letter falls back to US, punctuation to nothing.
        assert_eq!(rig.rt.char_for(0x41, None), Some('a'));
        assert_eq!(rig.rt.char_for(keys::VK_OEM_7, None), None);
        // Control characters are only legal on the boundary keys.
        assert_eq!(rig.rt.char_for(keys::VK_RETURN, Some('\r')), Some('\r'));
        assert_eq!(rig.rt.char_for(0x41, Some('\u{1}')), None);
        assert_eq!(rig.rt.char_for(keys::VK_SPACE, None), Some(' '));
    }

    #[test]
    fn default_toggle_hotkey_flips_state_on_release_and_plays_the_cue() {
        let _guard = globals::TEST_LOCK.lock();
        let mut rig = rig(Some(quiet_config()));
        assert!(globals::enabled());
        let ctrl = flags::CONTROL | flags::DEV_LCTRL;
        let both = ctrl | flags::SHIFT | flags::DEV_LSHIFT;
        let ev = |code, fl| RawEvent::key(EventKind::FlagsChanged, code, fl, None);
        // Control down, Shift down: nothing yet (release mode).
        assert_eq!(rig.rt.on_event(&ev(keys::MAC_CONTROL, ctrl)), Decision::PASS);
        assert_eq!(rig.rt.on_event(&ev(keys::MAC_SHIFT, both)), Decision::PASS);
        assert!(globals::enabled());
        // Shift up: the toggle fires; the modifier events themselves pass through.
        assert_eq!(rig.rt.on_event(&ev(keys::MAC_SHIFT, ctrl)), Decision::PASS);
        assert!(!globals::enabled());
        assert_eq!(rig.log.lock().toggles, vec![false]);
        rig.rt.on_event(&ev(keys::MAC_CONTROL, 0));
        // And again: back on.
        rig.rt.on_event(&ev(keys::MAC_CONTROL, ctrl));
        rig.rt.on_event(&ev(keys::MAC_SHIFT, both));
        rig.rt.on_event(&ev(keys::MAC_SHIFT, ctrl));
        assert!(globals::enabled());
        assert_eq!(rig.log.lock().toggles, vec![false, true]);
    }

    #[test]
    fn typing_while_holding_the_toggle_modifiers_does_not_toggle() {
        let _guard = globals::TEST_LOCK.lock();
        let mut rig = rig(Some(quiet_config()));
        let ctrl = flags::CONTROL | flags::DEV_LCTRL;
        let both = ctrl | flags::SHIFT | flags::DEV_LSHIFT;
        let ev = |kind, code, fl| RawEvent::key(kind, code, fl, None);
        rig.rt.on_event(&ev(EventKind::FlagsChanged, keys::MAC_CONTROL, ctrl));
        rig.rt.on_event(&ev(EventKind::FlagsChanged, keys::MAC_SHIFT, both));
        // Ctrl+Shift+Q is not a hotkey: it passes and cancels the toggle.
        assert_eq!(
            rig.rt.on_event(&ev(EventKind::KeyDown, mac_code('q'), both)),
            Decision::PASS
        );
        rig.rt.on_event(&ev(EventKind::KeyUp, mac_code('q'), both));
        rig.rt.on_event(&ev(EventKind::FlagsChanged, keys::MAC_SHIFT, ctrl));
        assert!(globals::enabled());
        assert!(rig.log.lock().toggles.is_empty());
    }

    #[test]
    fn open_settings_chord_is_swallowed_and_reports_the_command() {
        let _guard = globals::TEST_LOCK.lock();
        let commands = Arc::new(Mutex::new(Vec::new()));
        let sink = commands.clone();
        globals::set_command_callback(move |cmd| sink.lock().push(cmd));
        let mut rig = rig(Some(quiet_config()));
        let fl = flags::CONTROL | flags::ALTERNATE | flags::DEV_LCTRL | flags::DEV_LALT;
        let s = mac_code('s');
        // Release mode: the key-down is swallowed already, the action runs on key-up.
        let down = RawEvent::key(EventKind::KeyDown, s, fl, Some('s'));
        rig.rt.on_event(&RawEvent::key(EventKind::FlagsChanged, keys::MAC_CONTROL, flags::CONTROL | flags::DEV_LCTRL, None));
        rig.rt.on_event(&RawEvent::key(EventKind::FlagsChanged, keys::MAC_OPTION, fl, None));
        assert_eq!(rig.rt.on_event(&down), Decision::SWALLOW);
        let up = RawEvent::key(EventKind::KeyUp, s, fl, Some('s'));
        assert_eq!(rig.rt.on_event(&up), Decision::SWALLOW);
        assert!(commands.lock().contains(&BackendCommand::OpenSettings));
        // Auto-repeat of the held trigger stays swallowed too.
        let mut repeat = down;
        repeat.autorepeat = true;
        rig.rt.on_event(&down);
        assert_eq!(rig.rt.on_event(&repeat), Decision::SWALLOW);
    }

    #[test]
    fn restore_word_rewrites_raw_keys_and_keeps_chord_free_when_nothing_to_restore() {
        let _guard = globals::TEST_LOCK.lock();
        let mut cfg = quiet_config();
        cfg.hotkeys.toggle = None;
        cfg.hotkeys.toggle_mode = vkb_config::ToggleMode::Press;
        cfg.hotkeys.restore_word = CfgHotkey::parse("Ctrl+Shift+Z").ok();
        let mut rig = rig(Some(cfg));
        let chord = flags::CONTROL | flags::SHIFT | flags::DEV_LCTRL | flags::DEV_LSHIFT;
        let z = mac_code('z');
        let ev = |kind, code, fl, typed| RawEvent::key(kind, code, fl, typed);

        // Nothing typed: the chord is passed through so the app can redo.
        rig.rt.on_event(&ev(EventKind::FlagsChanged, keys::MAC_CONTROL, flags::CONTROL | flags::DEV_LCTRL, None));
        rig.rt.on_event(&ev(EventKind::FlagsChanged, keys::MAC_SHIFT, chord, None));
        rig.rt.on_event(&ev(EventKind::KeyDown, z, chord, Some('\u{1a}')));
        rig.rt.on_event(&ev(EventKind::KeyUp, z, chord, None));
        rig.rt.on_event(&ev(EventKind::FlagsChanged, keys::MAC_SHIFT, flags::CONTROL | flags::DEV_LCTRL, None));
        rig.rt.on_event(&ev(EventKind::FlagsChanged, keys::MAC_CONTROL, 0, None));
        assert!(rig.log.lock().replaced.is_empty());

        // A converted word: the chord restores it.
        press(&mut rig.rt, 'a');
        press(&mut rig.rt, 's');
        rig.log.lock().replaced.clear();
        rig.rt.on_event(&ev(EventKind::FlagsChanged, keys::MAC_CONTROL, flags::CONTROL | flags::DEV_LCTRL, None));
        rig.rt.on_event(&ev(EventKind::FlagsChanged, keys::MAC_SHIFT, chord, None));
        assert_eq!(rig.rt.on_event(&ev(EventKind::KeyDown, z, chord, Some('\u{1a}'))), Decision::SWALLOW);
        rig.rt.on_event(&ev(EventKind::KeyUp, z, chord, None));
        rig.rt.on_event(&ev(EventKind::FlagsChanged, keys::MAC_SHIFT, flags::CONTROL | flags::DEV_LCTRL, None));
        rig.rt.on_event(&ev(EventKind::FlagsChanged, keys::MAC_CONTROL, 0, None));
        assert_eq!(rig.log.lock().replaced, vec![(1, "as".to_string())]);
    }

    #[test]
    fn per_app_rule_disables_vietnamese() {
        let _guard = globals::TEST_LOCK.lock();
        let mut cfg = quiet_config();
        let mut rule = vkb_config::AppRule::new("com.apple.textedit");
        rule.enabled = Some(false);
        cfg.apps.rules.push(rule);
        let mut rig = rig(Some(cfg));
        press(&mut rig.rt, 'a');
        assert_eq!(press(&mut rig.rt, 's'), Decision::PASS);
        assert_eq!(rig.rt.active_app_name(), "com.apple.textedit");
        assert!(!rig.rt.effective_enabled());
        assert!(rig.log.lock().replaced.is_empty());
    }

    #[test]
    fn word_is_dropped_when_the_frontmost_app_changes() {
        let _guard = globals::TEST_LOCK.lock();
        let log = Arc::new(Mutex::new(Log::default()));
        struct Switch(Arc<Mutex<i32>>, Arc<Mutex<Log>>);
        impl Host for Switch {
            fn frontmost_pid(&mut self) -> i32 {
                *self.0.lock()
            }
            fn frontmost_bundle(&mut self) -> Option<String> {
                Some(format!("app.{}", *self.0.lock()))
            }
            fn secure_input(&mut self) -> bool {
                false
            }
            fn replace(&mut self, d: u32, t: &str) {
                self.1.lock().replaced.push((d, t.to_string()));
            }
            fn repost_key(&mut self, _: u16) {}
            fn toggle_sound(&mut self, _: bool) {}
            fn error_sound(&mut self) {}
        }
        globals::store_enabled(true);
        let pid = Arc::new(Mutex::new(10));
        let mut rt = Runtime::new(Box::new(Switch(pid.clone(), log.clone())), None, None);
        press(&mut rt, 'a');
        *pid.lock() = 11;
        // `s` in the other app must not retroactively tone the old `a`.
        assert_eq!(press(&mut rt, 's'), Decision::PASS);
        assert!(log.lock().replaced.is_empty());
    }

    #[test]
    fn published_config_is_picked_up_by_poll_config() {
        let _guard = globals::TEST_LOCK.lock();
        let mut rig = rig(None);
        let mut cfg = AppConfig::default();
        cfg.input.method = InputMethod::Vni;
        globals::publish_config(Arc::new(parking_lot::RwLock::new(cfg)));
        rig.rt.poll_config();
        assert_eq!(rig.rt.base.method, InputMethod::Vni);
        // VNI: `a1` -> `á`.
        press(&mut rig.rt, 'a');
        assert!(press(&mut rig.rt, '1').swallow);
        assert_eq!(rig.log.lock().replaced, vec![(1, "á".to_string())]);
    }

    #[test]
    fn macro_commit_on_enter_swallows_the_physical_key() {
        let _guard = globals::TEST_LOCK.lock();
        let mut cfg = quiet_config();
        cfg.typing.macro_enabled = true;
        cfg.macros.insert("vn".to_string(), "Việt Nam".to_string());
        let mut rig = rig(Some(cfg));
        press(&mut rig.rt, 'v');
        press(&mut rig.rt, 'n');
        let enter = RawEvent::key(EventKind::KeyDown, keys::MAC_RETURN, 0, Some('\r'));
        // The engine currently commits a macro as `Replace` with the Enter in the
        // text (`ReplaceThenPass` is never produced); either way the physical key
        // is swallowed and exactly one copy of Enter reaches the app.
        assert!(rig.rt.on_event(&enter).swallow);
        let log = rig.log.lock();
        let (_, text) = log.replaced.last().expect("macro injected");
        assert!(text.starts_with("Việt Nam"), "got {text:?}");
        assert!(log.reposted.len() <= 1);
    }

    #[test]
    fn swallowed_trigger_keyup_is_swallowed_even_under_secure_input() {
        let _guard = globals::TEST_LOCK.lock();
        let mut cfg = quiet_config();
        cfg.hotkeys.macro_panel = CfgHotkey::parse("F5").ok();
        let mut rig = rig(Some(cfg));
        let f5 = 96;
        assert_eq!(
            rig.rt.on_event(&RawEvent::key(EventKind::KeyDown, f5, 0, None)),
            Decision::SWALLOW
        );
        *rig.secure.lock() = true;
        // Key-up is not gated by secure input and must not leak.
        assert_eq!(
            rig.rt.on_event(&RawEvent::key(EventKind::KeyUp, f5, 0, None)),
            Decision::SWALLOW
        );
        // After a tap reset the bit still holds.
        *rig.secure.lock() = false;
        rig.rt.on_event(&RawEvent::key(EventKind::KeyDown, f5, 0, None));
        rig.rt.reset();
        assert_eq!(
            rig.rt.on_event(&RawEvent::key(EventKind::KeyUp, f5, 0, None)),
            Decision::SWALLOW
        );
    }

    #[test]
    fn cycle_method_alternates() {
        assert_eq!(next_method(InputMethod::Telex), InputMethod::Vni);
        assert_eq!(next_method(InputMethod::Vni), InputMethod::Telex);
    }

    #[test]
    fn word_context_change_detection() {
        let a = WordContext { pid: 1, enabled: true };
        assert!(!context_changed(None, a));
        assert!(!context_changed(Some(a), a));
        assert!(context_changed(Some(a), WordContext { pid: 2, enabled: true }));
        assert!(context_changed(Some(a), WordContext { pid: 1, enabled: false }));
    }

    #[test]
    fn caps_lock_flag_change_resets_without_swallowing() {
        let _guard = globals::TEST_LOCK.lock();
        let mut rig = rig(None);
        press(&mut rig.rt, 'a');
        let caps = RawEvent::key(EventKind::FlagsChanged, keys::MAC_CAPS_LOCK, flags::ALPHA_SHIFT, None);
        assert_eq!(rig.rt.on_event(&caps), Decision::PASS);
        assert_eq!(press(&mut rig.rt, 's'), Decision::PASS);
    }

    /// Types `text` like the E2E harness: Shift down/up around every capital
    /// letter, digits and lowercase letters unshifted. `string_follows_flags`
    /// false mimics synthetic events whose Unicode string ignores Shift.
    fn type_shifted(rt: &mut Runtime, text: &str, string_follows_flags: bool) {
        for ch in text.chars() {
            let shifted = ch.is_ascii_uppercase();
            let code = mac_code(ch.to_ascii_lowercase());
            let fl = if shifted { flags::SHIFT | flags::DEV_LSHIFT } else { 0 };
            let typed = if string_follows_flags { ch } else { ch.to_ascii_lowercase() };
            if shifted {
                rt.on_event(&RawEvent::key(EventKind::FlagsChanged, keys::MAC_SHIFT, fl, None));
            }
            rt.on_event(&RawEvent::key(EventKind::KeyDown, code, fl, Some(typed)));
            rt.on_event(&RawEvent::key(EventKind::KeyUp, code, fl, Some(typed)));
            if shifted {
                rt.on_event(&RawEvent::key(EventKind::FlagsChanged, keys::MAC_SHIFT, 0, None));
            }
        }
    }

    fn injections(word: &str, string_follows_flags: bool) -> Vec<(u32, String)> {
        let mut cfg = quiet_config();
        cfg.input.method = InputMethod::Vni;
        let mut rig = rig(Some(cfg));
        type_shifted(&mut rig.rt, word, string_follows_flags);
        let replaced = rig.log.lock().replaced.clone();
        replaced
    }

    /// D-MAC-01: a synthetic `V` carries the string `v` plus the Shift flag; the
    /// engine must still see a capital, or the rewrite comes out lowercase.
    #[test]
    fn vni_all_caps_word_takes_mark_and_tone_digits() {
        let _guard = globals::TEST_LOCK.lock();
        for follows in [true, false] {
            assert_eq!(
                injections("VIE6T5", follows),
                vec![(1, "Ê".to_string()), (2, "ỆT".to_string())],
                "string_follows_flags={follows}"
            );
            assert_eq!(injections("VIE6", follows), vec![(1, "Ê".to_string())]);
        }
        // A tone digit after a capitals-only word gives the same rewrite either way.
        for word in ["VIE5", "VIE6T"] {
            assert_eq!(injections(word, false), injections(word, true), "{word}");
            assert!(!injections(word, false).is_empty(), "{word} must be converted");
        }
    }

    #[test]
    fn caps_lock_flag_decides_letter_case_when_the_string_disagrees() {
        let _guard = globals::TEST_LOCK.lock();
        let mut rig = rig(None);
        let rt = &mut rig.rt;
        // Caps Lock on, string says `v`: the engine sees `V`; Shift+Caps is lowercase again.
        rt.mods.sync(flags::ALPHA_SHIFT);
        assert_eq!(rt.char_for(0x56, Some('v')), Some('V'));
        rt.mods.sync(flags::ALPHA_SHIFT | flags::SHIFT);
        assert_eq!(rt.char_for(0x56, Some('V')), Some('v'));
        // Non-letters and non-ASCII letters keep the layout's own character.
        assert_eq!(rt.char_for(0x36, Some('6')), Some('6'));
        assert_eq!(rt.char_for(0x41, Some('ă')), Some('ă'));
    }
}
