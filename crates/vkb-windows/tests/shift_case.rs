//! D-E2E-06: capitals lost on the real desktop, replayed in-process.
//!
//! The zero-focus harness (`examples/zero_focus_e2e.rs`) keeps a separate
//! `Shift` per event source, so it cannot see two things a real desktop does:
//! the user's key state and the injected key state are *one* state, and an
//! injected `Shift` release clears a `Shift` the user is still holding. This
//! test models that one shared state, feeds the hook the exact event shapes the
//! QA script (`qa/host-e2e/QN.cs`: `SendInput`, virtual keys, `Shift` down /
//! letter / `Shift` up) and a physical keyboard produce, and applies whatever the
//! hook injects in between, in order.
//!
//! Needs `--features e2e-harness` (the capture seam); nothing reaches the desktop.
#![cfg(all(windows, feature = "e2e-harness"))]

use vkb_windows::e2e;
use vkb_windows::keys::{self, VK_CAPITAL, VK_LSHIFT, VK_PACKET, VK_SHIFT};
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
            VK_SHIFT => self.generic_shift = down,
            VK_LSHIFT => self.left_shift = down,
            keys::VK_RSHIFT => self.right_shift = down,
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

    fn shown(&self) -> String {
        self.text.iter().collect()
    }
}

/// The capture buffer and the enabled flag are process-wide.
static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());

struct Rig {
    rt: HookRuntime,
    os: Desktop,
    _serial: std::sync::MutexGuard<'static, ()>,
}

impl Rig {
    fn new() -> Self {
        let serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        e2e::configure(TAG, INJECT_TAG);
        e2e::set_capture(true);
        Self {
            rt: HookRuntime::new(None, None),
            os: Desktop::default(),
            _serial: serial,
        }
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
        for event in e2e::take_captured() {
            self.os.apply(event.vk, event.scan, !event.up);
            self.os.injected.push((event.vk, !event.up));
        }
        if !swallowed {
            self.os.apply(vk, 0, down);
        }
    }

    fn tap(&mut self, vk: u16) {
        self.key(vk, true);
        self.key(vk, false);
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
    // The three rows of the host run that failed, Shift as the QA sent it
    // (generic `VK_SHIFT`) and as a keyboard sends it (`VK_LSHIFT`).
    for shift_vk in [VK_SHIFT, VK_LSHIFT] {
        for (keys_typed, expected) in [
            ("VIEETJ", "VIỆT"),
            ("Vieejt", "Việt"),
            ("Tieesng Vieetj raast ddepj.", "Tiếng Việt rất đẹp."),
            ("AAS DD", "Ấ Đ"),
        ] {
            let mut rig = Rig::new();
            rig.type_per_letter_shift(shift_vk, keys_typed);
            assert_eq!(rig.os.shown(), expected, "{keys_typed} with shift vk {shift_vk:#x}");
        }
    }
}

#[test]
fn a_capital_the_user_typed_is_never_retyped() {
    // `Vieejt`: V is on screen before `ee` arrives. The rewrite must start after
    // it, so no injected Shift pair is involved at all.
    let mut rig = Rig::new();
    rig.type_per_letter_shift(VK_SHIFT, "Vieejt");
    assert_eq!(rig.os.shown(), "Việt");
    assert!(
        !rig.os.injected.iter().any(|(vk, _)| matches!(*vk, VK_SHIFT | VK_LSHIFT)),
        "injected: {:?}",
        rig.os.injected
    );
}

#[test]
fn shift_held_through_the_word_stays_held() {
    // A user who holds Shift for a short all-caps word. The first rewrite used to
    // release Shift with its own key-up, so `T` then reached the app as `t`.
    for shift_vk in [VK_SHIFT, VK_LSHIFT] {
        let mut rig = Rig::new();
        rig.key(shift_vk, true);
        for ch in "VIEETJ".chars() {
            rig.tap(u16::from(ch as u8));
        }
        assert!(rig.os.shift(), "the user's Shift must still be down");
        rig.key(shift_vk, false);
        assert_eq!(rig.os.shown(), "VIỆT", "shift vk {shift_vk:#x}");
    }
}

#[test]
fn caps_lock_on_types_lowercase_words_lowercase() {
    // Caps Lock on, user types `vieejt` (shown as capitals) and then Shift+letter
    // for a lowercase one: the injected letters must follow the real case.
    let mut rig = Rig::new();
    rig.tap(VK_CAPITAL);
    rig.type_per_letter_shift(VK_SHIFT, "vieejt");
    assert_eq!(rig.os.shown(), "VIỆT");
}
