//! D-E2E-06: capitals lost on the real desktop, replayed in-process.
//!
//! The zero-focus harness used to keep a separate `Shift` per event source, so it
//! could not see what a real desktop does: the user's keys and the injected keys
//! are *one* state, and an injected `Shift` release clears a `Shift` the user is
//! still holding. This test models that one shared state, answers the injector's
//! `GetAsyncKeyState` / `GetKeyState` reads from it (`input::set_key_probe`), feeds
//! the hook the event shapes the QA script (`qa/host-e2e/QN.cs`: `SendInput`,
//! virtual keys, `Shift` down / letter / `Shift` up) and a physical keyboard
//! produce, and applies whatever the hook injects, in order.
//!
//! Needs `--features e2e-harness` (the capture seam); nothing reaches the desktop.
#![cfg(all(windows, feature = "e2e-harness"))]

use std::cell::RefCell;
use std::rc::Rc;

use vkb_windows::e2e;
use vkb_windows::input::{self, KeyProbe};
use vkb_windows::keys::{self, VK_CAPITAL, VK_LSHIFT, VK_PACKET, VK_RSHIFT, VK_SHIFT};
use vkb_windows::state::HookRuntime;
use windows::Win32::UI::WindowsAndMessaging::{KBDLLHOOKSTRUCT, WM_KEYDOWN, WM_KEYUP};

const TAG: usize = 0x7001;
const INJECT_TAG: usize = 0x7002;

/// What the target application sees: one key state, one text buffer.
#[derive(Default)]
struct Desktop {
    generic_shift: bool,
    left_shift: bool,
    right_shift: bool,
    caps: bool,
    text: Vec<char>,
    /// Every injected key event, for assertions about what the hook sent.
    injected: Vec<(u16, bool)>,
}

impl Desktop {
    fn shift(&self) -> bool {
        self.generic_shift || self.left_shift || self.right_shift
    }

    fn apply(&mut self, vk: u16, scan: u16, down: bool) {
        match vk {
            // A generic key-up releases whichever Shift is down.
            VK_SHIFT if down => self.generic_shift = true,
            VK_SHIFT => {
                self.generic_shift = false;
                self.left_shift = false;
                self.right_shift = false;
            }
            VK_LSHIFT => self.left_shift = down,
            VK_RSHIFT => self.right_shift = down,
            VK_CAPITAL => {
                if down {
                    self.caps = !self.caps;
                }
            }
            VK_PACKET if down => self.text.extend(char::from_u32(u32::from(scan))),
            keys::VK_BACK if down => {
                self.text.pop();
            }
            _ if down => {
                if let Some(ch) = keys::us_char(vk, self.shift(), self.caps) {
                    self.text.push(ch);
                }
            }
            _ => {}
        }
    }

    /// Take in what the injector has sent so far, as `SendInput` would have
    /// applied it by the time the injector reads the key state again.
    fn absorb(&mut self) {
        for event in e2e::take_captured() {
            self.apply(event.vk, event.scan, !event.up);
            self.injected.push((event.vk, !event.up));
        }
    }
}

/// The injector's view of the desktop.
struct Probe(Rc<RefCell<Desktop>>);

impl KeyProbe for Probe {
    fn down(&self, vk: u16) -> bool {
        let mut os = self.0.borrow_mut();
        os.absorb();
        match vk {
            VK_LSHIFT => os.left_shift,
            VK_RSHIFT => os.right_shift,
            VK_SHIFT => os.shift(),
            _ => false,
        }
    }

    fn toggled(&self, vk: u16) -> bool {
        let mut os = self.0.borrow_mut();
        os.absorb();
        vk == VK_CAPITAL && os.caps
    }
}

/// The capture buffer and the enabled flag are process-wide.
static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());

struct Rig {
    rt: HookRuntime,
    os: Rc<RefCell<Desktop>>,
    _serial: std::sync::MutexGuard<'static, ()>,
}

impl Drop for Rig {
    fn drop(&mut self) {
        input::set_key_probe(None);
        e2e::set_capture(false);
    }
}

impl Rig {
    fn new() -> Self {
        let serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        e2e::configure(TAG, INJECT_TAG);
        e2e::set_capture(true);
        let os = Rc::new(RefCell::new(Desktop::default()));
        input::set_key_probe(Some(Box::new(Probe(os.clone()))));
        Self {
            rt: HookRuntime::new(None, None),
            os,
            _serial: serial,
        }
    }

    fn shown(&self) -> String {
        self.os.borrow().text.iter().collect()
    }

    fn shift_down(&self) -> bool {
        self.os.borrow().shift()
    }

    fn injected_shift(&self) -> bool {
        self.os
            .borrow()
            .injected
            .iter()
            .any(|(vk, _)| matches!(*vk, VK_SHIFT | VK_LSHIFT | VK_RSHIFT))
    }

    /// One physical event: the hook decides, its injections land first, then the
    /// key itself unless it was swallowed.
    fn key(&mut self, vk: u16, down: bool) {
        let info = KBDLLHOOKSTRUCT {
            vkCode: u32::from(vk),
            scanCode: 0,
            flags: Default::default(),
            time: 0,
            dwExtraInfo: TAG,
        };
        let message = if down { WM_KEYDOWN } else { WM_KEYUP };
        let swallowed = e2e::feed(&mut self.rt, message, &info);
        let mut os = self.os.borrow_mut();
        os.absorb();
        if !swallowed {
            os.apply(vk, 0, down);
        }
    }

    fn tap(&mut self, vk: u16) {
        self.key(vk, true);
        self.key(vk, false);
    }

    fn tap_text(&mut self, text: &str) {
        for ch in text.chars() {
            self.tap(u16::from(ch.to_ascii_uppercase() as u8));
        }
    }

    /// The QA script: a capital is `Shift` down, letter, `Shift` up, per letter.
    fn type_per_letter_shift(&mut self, shift_vk: u16, text: &str) {
        for ch in text.chars() {
            let vk = match ch {
                ' ' => keys::VK_SPACE,
                '.' => keys::VK_OEM_PERIOD,
                letter => u16::from(letter.to_ascii_uppercase() as u8),
            };
            if ch.is_ascii_uppercase() {
                self.key(shift_vk, true);
                self.tap(vk);
                self.key(shift_vk, false);
            } else {
                self.tap(vk);
            }
        }
    }
}

#[test]
fn qa_per_letter_shift_keeps_every_capital() {
    // The rows of the host run that failed, Shift as the QA sent it (generic
    // `VK_SHIFT`) and as a keyboard sends it (`VK_LSHIFT`, `VK_RSHIFT`).
    for shift_vk in [VK_SHIFT, VK_LSHIFT, VK_RSHIFT] {
        for (keys_typed, expected) in [
            ("VIEETJ", "VIỆT"),
            ("Vieejt", "Việt"),
            ("Tieesng Vieetj raast ddepj.", "Tiếng Việt rất đẹp."),
            ("AAS DD", "Ấ Đ"),
        ] {
            let mut rig = Rig::new();
            rig.type_per_letter_shift(shift_vk, keys_typed);
            assert_eq!(rig.shown(), expected, "{keys_typed} with shift vk {shift_vk:#x}");
            assert!(!rig.shift_down(), "no Shift left down after {keys_typed}");
        }
    }
}

#[test]
fn a_capital_the_user_typed_is_never_retyped() {
    // `Vieejt`: V is on screen before `ee` arrives. The rewrite must start after
    // it, so no injected Shift pair is involved at all.
    let mut rig = Rig::new();
    rig.type_per_letter_shift(VK_SHIFT, "Vieejt");
    assert_eq!(rig.shown(), "Việt");
    assert!(!rig.injected_shift(), "injected: {:?}", rig.os.borrow().injected);
}

#[test]
fn shift_held_through_the_word_stays_held() {
    // A user who holds Shift for a short all-caps word, on either side. The
    // first rewrite used to release Shift with its own key-up, so `T` then
    // reached the app as `t`; a lowercase retype must not release it either.
    for shift_vk in [VK_SHIFT, VK_LSHIFT, VK_RSHIFT] {
        let mut rig = Rig::new();
        rig.key(shift_vk, true);
        rig.tap_text("VIEETJ");
        assert!(rig.shift_down(), "the user's Shift must still be down ({shift_vk:#x})");
        rig.key(shift_vk, false);
        assert_eq!(rig.shown(), "VIỆT", "shift vk {shift_vk:#x}");
        assert!(!rig.shift_down());
    }
}

#[test]
fn a_held_shift_is_not_pressed_or_released_by_the_rewrite() {
    // Right Shift held through the word: the hook may inject packets, never a
    // Shift event, so nothing can be left stuck whatever the user does next.
    let mut rig = Rig::new();
    rig.key(VK_RSHIFT, true);
    rig.tap_text("VIEETJ");
    assert!(!rig.injected_shift(), "injected: {:?}", rig.os.borrow().injected);
    rig.key(VK_RSHIFT, false);
    assert!(!rig.shift_down());
}

#[test]
fn caps_lock_on_types_lowercase_words_lowercase() {
    // Caps Lock on, user types `vieejt` (shown as capitals) and then Shift+letter
    // for a lowercase one: the injected letters must follow the real case.
    let mut rig = Rig::new();
    rig.tap(VK_CAPITAL);
    rig.type_per_letter_shift(VK_SHIFT, "vieejt");
    assert_eq!(rig.shown(), "VIỆT");
}

#[test]
fn caps_lock_with_shift_held_for_every_letter_types_lowercase() {
    // Caps on: Shift+letter is lowercase. The user holds Shift for each letter,
    // so the retyped `t` (wanted lowercase, shown as a capital) would need that
    // Shift released: it goes out as a packet instead.
    let mut rig = Rig::new();
    rig.tap(VK_CAPITAL);
    rig.type_per_letter_shift(VK_SHIFT, "VIEETJ");
    assert_eq!(rig.shown(), "việt");
    assert!(!rig.shift_down());
}

#[test]
fn the_restore_chord_leaves_the_keyboard_and_the_os_in_agreement() {
    // Ctrl+Shift+Z on a word: the hook lifts the chord's modifiers for its batch
    // and must end with the OS matching the keys the user holds (none here).
    let mut rig = Rig::new();
    rig.type_per_letter_shift(VK_LSHIFT, "Vieejt");
    assert_eq!(rig.shown(), "Việt");
    rig.key(keys::VK_LCONTROL, true);
    rig.key(VK_LSHIFT, true);
    rig.tap(0x5A);
    rig.key(VK_LSHIFT, false);
    rig.key(keys::VK_LCONTROL, false);
    assert_eq!(rig.shown(), "Vieejt", "the word is back to its keystrokes");
    assert!(!rig.shift_down(), "no Shift left down");
}

#[test]
fn a_short_send_finishes_the_same_batch_instead_of_repeating_it() {
    // `Vie` + `e`: the batch is backspace, backspace-up, `ê` down, `ê` up. The OS
    // takes two events and stops. Rebuilding the whole batch would delete `i` too.
    let mut rig = Rig::new();
    rig.type_per_letter_shift(VK_SHIFT, "Vie");
    e2e::short_next_batches(&[2]);
    rig.tap(0x45);
    assert_eq!(rig.shown(), "Viê");
}

#[test]
fn a_batch_that_stops_halfway_leaves_no_shift_of_ours_down() {
    // `VIEeTj`: the tone rewrite retypes `T` inside an injected Shift pair (the
    // user holds no Shift for `j`). The OS takes the Shift press and the key,
    // then refuses the rest, twice. Our half-pressed Shift must be released.
    let mut rig = Rig::new();
    rig.type_per_letter_shift(VK_SHIFT, "VIEeT");
    e2e::short_next_batches(&[7, 0]);
    rig.tap(0x4A);
    assert!(!rig.shift_down(), "injected: {:?}", rig.os.borrow().injected);
}

#[test]
fn a_mouse_click_or_arrow_ends_the_word_so_nothing_stale_is_trimmed() {
    // Review question 10: whatever moves the caret reaches the engine, so the
    // text it believes is on screen never outlives the word.
    {
        let mut rig = Rig::new();
        rig.tap_text("vie");
        e2e::mouse_button_down();
        rig.tap_text("e");
        assert_eq!(rig.shown(), "viee", "the click ended the word");
    }

    let mut rig = Rig::new();
    rig.tap_text("vie");
    rig.tap(keys::VK_LEFT);
    rig.tap_text("e");
    assert_eq!(rig.shown(), "viee", "an arrow key ended the word");
}

#[test]
fn nfd_output_never_splits_a_base_letter_from_its_marks() {
    use std::sync::Arc;
    let mut config = vkb_config::AppConfig::default();
    config.input.enabled = true;
    config.input.charset = vkb_core::Charset::UnicodeNfd;
    config.system.notification_sound = false;
    let mut rig = Rig::new();
    rig.rt.apply_config(Arc::new(config));
    rig.type_per_letter_shift(VK_SHIFT, "Vieejt");
    assert_eq!(rig.shown(), "Vie\u{302}\u{323}t");
}
