//! `INPUT` array construction and delivery.
//!
//! [`build`] is pure: one [`EngineAction`] decision becomes exactly one array,
//! which is handed to a **single** `SendInput` call. For
//! `EngineAction::Replace` that means the target window sees all backspaces and
//! the replacement text in one atomic batch — no intermediate state, no flicker.
//!
//! Encoding inside that batch is not free, though: the Windows 11 Notepad text
//! stack collapses a run of `KEYEVENTF_UNICODE` (`VK_PACKET`) characters that
//! arrives directly after backspaces onto the run's last character (D-E2E-01,
//! reproduced with raw `SendInput` and no hook). So a replacement types its
//! ASCII characters as real VK + scancode keystrokes and reserves the packet
//! path for characters no US key produces. Plain typing keeps the pure packet
//! path: it has no delete block and is not affected.
//!
//! Clipboard is never part of this path (charter C4); the opt-in fallback lives
//! in [`crate::clipboard`].

use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
    KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE, VIRTUAL_KEY,
};

use std::cell::Cell;

use crate::keys;
use vkb_core::Modifiers;
use vkb_platform::plan::Injection;

/// Stamped on every synthetic event so the hook can recognise its own input and
/// never feed it back into the engine.
pub const INJECTED_MAGIC: usize = 0x5649_454B; // "VIEK"

/// The `dwExtraInfo` stamped on the backend's own injections.
///
/// Always [`INJECTED_MAGIC`] in a default build (a constant the optimiser folds
/// away). With the `e2e-harness` feature it is configurable so a test process can
/// tell its backend's injections from those of an installed VietKB.
#[cfg(not(feature = "e2e-harness"))]
#[inline(always)]
pub fn injected_tag() -> usize {
    INJECTED_MAGIC
}

#[cfg(feature = "e2e-harness")]
#[inline]
pub fn injected_tag() -> usize {
    crate::e2e::inject_tag()
}

/// Which of the keys that decide a letter's case the *user* is holding now.
///
/// An injected `Shift` pair cannot assume the key starts up: a user who holds
/// `Shift` through `VIEET` would have it released by the first pair we send, so
/// every later physical letter reaches the app lowercase (D-E2E-06); and a
/// lowercase letter injected while `Shift` is down would arrive as a capital.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Held {
    pub shift: bool,
    pub caps: bool,
}

thread_local! {
    /// Published by the hook for the thread that also builds the injection (the
    /// batch is built inside the hook callback), so tests on other threads never
    /// see each other's keys.
    static HELD: Cell<Held> = const { Cell::new(Held { shift: false, caps: false }) };
}

/// Record the user's current `Shift` / `Caps Lock` state for this thread.
pub fn note_held(held: Held) {
    HELD.with(|cell| cell.set(held));
}

fn held_now() -> Held {
    HELD.with(Cell::get)
}

/// How text is delivered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SendMode {
    /// `KEYEVENTF_UNICODE`: layout-independent, the default.
    Unicode,
    /// `send_standard` (config): virtual key + scancode, for apps that ignore
    /// Unicode events (some games, legacy VB/Delphi controls).
    Standard,
}

/// Build the array for one injection. Empty for `None`/`Beep`.
pub fn build(injection: &Injection, mode: SendMode) -> Vec<INPUT> {
    match injection {
        Injection::None | Injection::Beep => Vec::new(),
        Injection::Type(text) => build_segment(0, text, mode),
        Injection::Replace { deletes, insert } => build_segment(*deletes, insert, mode),
    }
}

/// Backspaces + text in one array, so `Replace` is delivered atomically.
pub fn build_segment(deletes: u32, text: &str, mode: SendMode) -> Vec<INPUT> {
    build_segment_held(deletes, text, mode, held_now())
}

/// [`build_segment`] with the user's `Shift` / `Caps Lock` state given explicitly.
pub fn build_segment_held(deletes: u32, text: &str, mode: SendMode, held: Held) -> Vec<INPUT> {
    let mut out = Vec::with_capacity((deletes as usize) * 2 + text.encode_utf16().count() * 2);
    push_segment(&mut out, deletes, text, mode, held);
    out
}

/// A `restore_word` rewrite: the chord's modifiers are released first, then the
/// deletes and the text, all in one `SendInput` call.
///
/// The restore chord fires *inside* the hook callback of the key-up that
/// releases it (`Combo::fire_on_full_release`), so the OS has not applied that
/// key-up yet and the target still sees `Ctrl`/`Shift` down. Every injected
/// character then arrives shifted — measured CH2: `ba6t5` reached the app as
/// `BA^T%` (Shift still down) or as `Ctrl+letter` shortcuts (Ctrl still down).
/// A key-up for a key that is already up is a no-op for the OS, so releasing the
/// chord's own modifiers here is safe: it only runs when the user has already
/// let go of every key of the chord. Plain typing never uses this path.
pub fn build_restore(deletes: u32, text: &str, mode: SendMode, chord: Modifiers) -> Vec<INPUT> {
    let mut out =
        Vec::with_capacity(8 + (deletes as usize) * 2 + text.encode_utf16().count() * 2);
    push_chord_release(&mut out, chord);
    // The chord's own `Shift` is up after the release above.
    let mut held = held_now();
    if chord.shift {
        held.shift = false;
    }
    push_segment(&mut out, deletes, text, mode, held);
    out
}

fn push_segment(out: &mut Vec<INPUT>, deletes: u32, text: &str, mode: SendMode, held: Held) {
    for _ in 0..deletes {
        push_backspace(out, mode);
    }
    // D-E2E-01: a replacement is a delete block followed by text in the *same*
    // `SendInput` batch. In Notepad's WinUI text stack a run of `VK_PACKET`
    // (`KEYEVENTF_UNICODE`) characters that follows backspaces is not appended
    // character by character — the whole run collapses onto its last character
    // (`Replace{2,"tă"}` over `bật/ta` yields `bật/ăă`, `Replace{3,"tắt"}`
    // yields `ttt`). Reproduced with raw `SendInput` and no hook, so it is the
    // target, not the engine. Keeping ASCII off the packet path in a
    // replacement is what makes the batch survive: measured 8/8 correct for
    // `ba6t5/ta8t1` versus 0/8 on the pure packet path. Plain typing has no
    // delete block and keeps the layout-independent packet path.
    push_text(out, text, mode, deletes > 0, held);
}

/// Key-ups for the modifiers of the chord that just ended.
///
/// Both sides of each modifier: a generic `VK_SHIFT` key-up does not clear
/// `RSHIFT`, and the user may have used either.
fn push_chord_release(out: &mut Vec<INPUT>, chord: Modifiers) {
    if chord.shift {
        push_key(out, keys::VK_LSHIFT, keys::scan_for_vk(keys::VK_LSHIFT), true);
        push_key(out, keys::VK_RSHIFT, keys::scan_for_vk(keys::VK_RSHIFT), true);
    }
    if chord.ctrl {
        push_key(out, keys::VK_LCONTROL, 0x1D, true);
        push_key_ext(out, keys::VK_RCONTROL, 0x1D, true);
    }
    if chord.alt {
        push_key(out, keys::VK_LMENU, 0x38, true);
        push_key_ext(out, keys::VK_RMENU, 0x38, true);
    }
    if chord.win {
        push_key_ext(out, keys::VK_LWIN, 0x5B, true);
        push_key_ext(out, keys::VK_RWIN, 0x5C, true);
    }
}

fn push_backspace(out: &mut Vec<INPUT>, mode: SendMode) {
    let scan = match mode {
        SendMode::Unicode => 0,
        SendMode::Standard => keys::scan_for_vk(keys::VK_BACK),
    };
    push_key(out, keys::VK_BACK, scan, false);
    push_key(out, keys::VK_BACK, scan, true);
}

fn push_text(out: &mut Vec<INPUT>, text: &str, mode: SendMode, scancode_ascii: bool, held: Held) {
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            // A macro expansion may contain line breaks. A raw `\n` code unit is
            // not a newline in an edit control, so type Return instead.
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                push_newline(out);
            }
            '\n' => push_newline(out),
            other => push_char(out, other, mode, scancode_ascii, held),
        }
    }
}

fn push_newline(out: &mut Vec<INPUT>) {
    let scan = keys::scan_for_vk(keys::VK_RETURN);
    push_key(out, keys::VK_RETURN, scan, false);
    push_key(out, keys::VK_RETURN, scan, true);
}

fn push_char(out: &mut Vec<INPUT>, ch: char, mode: SendMode, scancode_ascii: bool, held: Held) {
    // Both modes fall back to a Unicode packet for a character no US key
    // produces (Vietnamese diacritics), so the text is never lost.
    let via_scancode = mode == SendMode::Standard || scancode_ascii;
    match keys::us_vk_for_char(ch).filter(|_| via_scancode) {
        Some((vk, shift)) => push_ascii_key(out, vk, shift, held),
        None => push_unicode_char(out, ch),
    }
}

/// One ASCII key press with the case the text needs, given what the user holds.
///
/// `Shift` is only touched when the key would otherwise come out wrong, and the
/// user's own state is put back afterwards: a held `Shift` is never left
/// released, and an up one is never left pressed.
fn push_ascii_key(out: &mut Vec<INPUT>, vk: u16, want_shift: bool, held: Held) {
    // A letter's case is `Shift` xor `Caps Lock`; every other key follows `Shift`.
    let is_letter = (0x41..=0x5A).contains(&vk);
    let shown_shift = if is_letter { held.shift ^ held.caps } else { held.shift };
    let wrong = want_shift != shown_shift;
    let press_shift = wrong && !held.shift;
    let lift_shift = wrong && held.shift;
    let shift_scan = keys::scan_for_vk(keys::VK_SHIFT);
    if press_shift {
        push_key(out, keys::VK_SHIFT, shift_scan, false);
    }
    if lift_shift {
        push_shift_up(out);
    }
    let scan = keys::scan_for_vk(vk);
    push_key(out, vk, scan, false);
    push_key(out, vk, scan, true);
    if press_shift {
        push_key(out, keys::VK_SHIFT, shift_scan, true);
    }
    if lift_shift {
        push_key(out, keys::VK_LSHIFT, shift_scan, false);
    }
}

/// Key-up for both `Shift` keys: the user may hold either.
fn push_shift_up(out: &mut Vec<INPUT>) {
    push_key(out, keys::VK_LSHIFT, keys::scan_for_vk(keys::VK_LSHIFT), true);
    push_key(out, keys::VK_RSHIFT, keys::scan_for_vk(keys::VK_RSHIFT), true);
}

/// One character as `KEYEVENTF_UNICODE` (layout independent, surrogate-safe).
fn push_unicode_char(out: &mut Vec<INPUT>, ch: char) {
    let mut buf = [0u16; 2];
    for unit in ch.encode_utf16(&mut buf).iter().copied() {
        push_unicode(out, unit, false);
        push_unicode(out, unit, true);
    }
}

fn push_key(out: &mut Vec<INPUT>, vk: u16, scan: u16, up: bool) {
    push_key_tagged(out, vk, scan, up, injected_tag());
}

/// Same, for a key that lives on the extended scan-code set (`RCONTROL`,
/// `RMENU`, `LWIN`/`RWIN`): without the flag a `RCONTROL` key-up with scan
/// `0x1D` reads as the left key.
fn push_key_ext(out: &mut Vec<INPUT>, vk: u16, scan: u16, up: bool) {
    let mut flags = KEYEVENTF_EXTENDEDKEY;
    if up {
        flags |= KEYEVENTF_KEYUP;
    }
    out.push(INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk),
                wScan: scan,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: injected_tag(),
            },
        },
    });
}

fn push_key_tagged(out: &mut Vec<INPUT>, vk: u16, scan: u16, up: bool, extra: usize) {
    let flags = if up {
        KEYEVENTF_KEYUP
    } else {
        KEYBD_EVENT_FLAGS(0)
    };
    out.push(INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk),
                wScan: scan,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: extra,
            },
        },
    });
}

fn push_unicode(out: &mut Vec<INPUT>, unit: u16, up: bool) {
    let mut flags = KEYEVENTF_UNICODE;
    if up {
        flags |= KEYEVENTF_KEYUP;
    }
    out.push(INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(0),
                wScan: unit,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: injected_tag(),
            },
        },
    });
}

/// Outcome of one `SendInput` call, for the `VKB_DEBUG_KEYS` trace.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SendResult {
    /// Events the OS accepted.
    pub sent: u32,
    /// Events in the array.
    pub total: u32,
    /// `GetLastError` right after the call; only meaningful on a short send.
    pub error: u32,
    /// How many `SendInput` calls it took (2 = the retry after a zero send).
    pub attempts: u32,
}

/// Deliver the batch in one call, reporting the error as well as the count.
pub fn send_detailed(inputs: &[INPUT]) -> SendResult {
    if inputs.is_empty() {
        return SendResult::default();
    }
    #[cfg(feature = "e2e-harness")]
    if crate::e2e::capture(inputs) {
        let total = inputs.len() as u32;
        return SendResult {
            sent: total,
            total,
            error: 0,
            attempts: 1,
        };
    }
    let size = std::mem::size_of::<INPUT>() as i32;
    let total = inputs.len() as u32;
    let mut attempts = 0u32;
    let mut sent = unsafe { SendInput(inputs, size) };
    attempts += 1;
    // `0` means the OS accepted nothing at all — the replacement would vanish
    // silently (the key was already swallowed). Retry exactly once, and only
    // when nothing was delivered, so a partial send can never be duplicated.
    if sent == 0 {
        sent = unsafe { SendInput(inputs, size) };
        attempts += 1;
    }
    // Read the error while it still belongs to the last `SendInput` call.
    let error = if sent < total {
        unsafe { windows::Win32::Foundation::GetLastError().0 }
    } else {
        0
    };
    // P2 probe: closes the sample at the injection return. When the probe is off
    // the thread-local mark is `None`, so this is a `Cell::take` and nothing more.
    crate::perf::end();
    SendResult {
        sent,
        total,
        error,
        attempts,
    }
}

/// Deliver the batch in one call. Returns the number of events the OS accepted
/// (fewer than `inputs.len()` means the target blocked input).
pub fn send(inputs: &[INPUT]) -> u32 {
    send_detailed(inputs).sent
}

/// `Ctrl+V` with scancodes. Only the opt-in clipboard fallback uses this.
pub fn ctrl_v_sequence() -> Vec<INPUT> {
    let mut out = Vec::with_capacity(4);
    let ctrl_scan = keys::scan_for_vk(keys::VK_CONTROL);
    let v_scan = keys::scan_for_vk(0x56);
    push_key(&mut out, keys::VK_CONTROL, ctrl_scan, false);
    push_key(&mut out, 0x56, v_scan, false);
    push_key(&mut out, 0x56, v_scan, true);
    push_key(&mut out, keys::VK_CONTROL, ctrl_scan, true);
    out
}

/// Inject one untagged press+release so the hook treats it as real input.
///
/// Only the `#[ignore]`d integration test uses this: every production injection
/// carries [`INJECTED_MAGIC`].
pub fn send_untagged_key(vk: u16) -> u32 {
    let mut out = Vec::with_capacity(2);
    let scan = keys::scan_for_vk(vk);
    push_key_tagged(&mut out, vk, scan, false, 0);
    push_key_tagged(&mut out, vk, scan, true, 0);
    send(&out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bake(injection: &Injection, mode: SendMode) -> Vec<INPUT> {
        build(injection, mode)
    }

    /// (vk, scan, flags, extra_info) of one event.
    fn field(input: &INPUT) -> (u16, u16, u32, usize) {
        let ki = unsafe { input.Anonymous.ki };
        (ki.wVk.0, ki.wScan, ki.dwFlags.0, ki.dwExtraInfo)
    }

    fn is_up(input: &INPUT) -> bool {
        field(input).2 & KEYEVENTF_KEYUP.0 != 0
    }

    fn unicode_seqs(inputs: &[INPUT]) -> Vec<(u16, bool)> {
        inputs
            .iter()
            .map(|input| {
                let ki = unsafe { input.Anonymous.ki };
                (
                    ki.wScan,
                    ki.dwFlags.0 & KEYEVENTF_KEYUP.0 != 0,
                )
            })
            .collect()
    }

    #[test]
    fn nothing_is_built_for_noop_and_beep() {
        assert!(bake(&Injection::None, SendMode::Unicode).is_empty());
        assert!(bake(&Injection::Beep, SendMode::Unicode).is_empty());
    }

    #[test]
    fn replace_is_one_batch_backspaces_then_text() {
        let inputs = bake(
            &Injection::Replace {
                deletes: 2,
                insert: "á".to_string(),
            },
            SendMode::Unicode,
        );
        assert_eq!(inputs.len(), 2 * 2 + 2);
        // 0..4 backspace down/up pairs, then the unicode pair.
        for input in &inputs[..4] {
            assert_eq!(field(input).0, keys::VK_BACK);
        }
        assert!(!is_up(&inputs[0]), "first event is the key-down");
        assert!(is_up(&inputs[1]));
        let seqs = unicode_seqs(&inputs[4..]);
        assert_eq!(seqs, vec![(0x00E1, false), (0x00E1, true)]);
    }

    #[test]
    fn every_event_is_tagged_as_ours() {
        let inputs = bake(&Injection::Type("Việt".to_string()), SendMode::Unicode);
        assert!(!inputs.is_empty());
        for input in &inputs {
            assert_eq!(field(input).3, injected_tag());
        }
    }

    #[test]
    fn unicode_mode_splits_surrogate_pairs() {
        // U+1F600 = D83D DE00 -> four events.
        let inputs = bake(&Injection::Type("😀".to_string()), SendMode::Unicode);
        assert_eq!(inputs.len(), 4);
        assert_eq!(unicode_seqs(&inputs), vec![(0xD83D, false), (0xD83D, true), (0xDE00, false), (0xDE00, true)]);
    }

    #[test]
    fn standard_mode_uses_scancodes_for_ascii() {
        let inputs = bake(&Injection::Type("a".to_string()), SendMode::Standard);
        assert_eq!(inputs.len(), 2, "one key down + one key up");
        let ki = unsafe { inputs[0].Anonymous.ki };
        assert_eq!(ki.wVk.0, 0x41);
        assert_eq!(ki.wScan, keys::scan_for_vk(0x41));
        assert_ne!(ki.wScan, 0, "standard mode must carry a scancode");
        assert_eq!(ki.dwFlags.0 & KEYEVENTF_UNICODE.0, 0);
    }

    #[test]
    fn standard_mode_wraps_uppercase_in_shift() {
        let inputs = bake(&Injection::Type("A".to_string()), SendMode::Standard);
        assert_eq!(inputs.len(), 4);
        assert_eq!(field(&inputs[0]).0, keys::VK_SHIFT);
        assert_eq!(field(&inputs[3]).0, keys::VK_SHIFT);
        assert!(is_up(&inputs[3]), "shift must be released last");
    }

    #[test]
    fn standard_mode_falls_back_to_unicode_for_diacritics() {
        let inputs = bake(&Injection::Type("Đ".to_string()), SendMode::Standard);
        assert_eq!(inputs.len(), 2);
        let ki = unsafe { inputs[0].Anonymous.ki };
        assert_eq!(ki.wVk.0, 0);
        assert_eq!(ki.dwFlags.0 & KEYEVENTF_UNICODE.0, KEYEVENTF_UNICODE.0);
    }

    #[test]
    fn standard_mode_backspace_has_scan_code() {
        let inputs = bake(
            &Injection::Replace {
                deletes: 1,
                insert: String::new(),
            },
            SendMode::Standard,
        );
        assert_eq!(inputs.len(), 2);
        assert_eq!(field(&inputs[0]).1, keys::scan_for_vk(keys::VK_BACK));
    }

    #[test]
    fn macro_newline_is_typed_as_return() {
        let mut out = Vec::new();
        push_text(&mut out, "a\nb", SendMode::Unicode, false, Held::default());
        assert_eq!(out.len(), 6, "a, Return down/up, b");
        assert_eq!(field(&out[2]).0, keys::VK_RETURN);
        assert!(!is_up(&out[2]));
        assert_eq!(field(&out[3]).0, keys::VK_RETURN);
        assert!(is_up(&out[3]));

        let mut crlf = Vec::new();
        push_text(&mut crlf, "a\r\nb", SendMode::Unicode, false, Held::default());
        assert_eq!(crlf.len(), 6, "CRLF must collapse to one newline");
    }

    /// D-E2E-01: inside a replacement the ASCII characters must be real
    /// scancode keystrokes, never `VK_PACKET` — Notepad's WinUI text stack
    /// collapses a packet run that follows backspaces.
    #[test]
    fn replace_types_ascii_with_scancodes_in_unicode_mode() {
        let inputs = bake(
            &Injection::Replace {
                deletes: 2,
                insert: "tă".to_string(),
            },
            SendMode::Unicode,
        );
        // 2 backspace pairs, then T as a real key, then ă as a packet.
        assert_eq!(inputs.len(), 4 + 2 + 2);
        for input in &inputs[..4] {
            assert_eq!(field(input).0, keys::VK_BACK);
        }
        let t_down = field(&inputs[4]);
        assert_eq!(t_down.0, 0x54, "ASCII 't' goes through the VK path");
        assert_eq!(t_down.1, keys::scan_for_vk(0x54));
        assert_eq!(t_down.2 & KEYEVENTF_UNICODE.0, 0, "not a packet");
        assert!(!is_up(&inputs[4]));
        assert!(is_up(&inputs[5]));
        let packet = field(&inputs[6]);
        assert_eq!(packet.0, 0, "non-ASCII stays layout independent");
        assert_eq!(packet.2 & KEYEVENTF_UNICODE.0, KEYEVENTF_UNICODE.0);
        assert_eq!(unicode_seqs(&inputs[6..]), vec![(0x0103, false), (0x0103, true)]);
    }

    /// The packet path is kept for plain typing: no delete block, no collapse.
    #[test]
    fn type_keeps_unicode_packets_in_unicode_mode() {
        let inputs = bake(&Injection::Type("tă".to_string()), SendMode::Unicode);
        assert_eq!(inputs.len(), 4, "two characters, four packet events");
        for input in &inputs {
            let (vk, _, flags, _) = field(input);
            assert_eq!(vk, 0, "plain typing must not press a VK");
            assert_ne!(flags & KEYEVENTF_UNICODE.0, 0);
        }
        assert_eq!(unicode_seqs(&inputs), vec![(0x0074, false), (0x0074, true), (0x0103, false), (0x0103, true)]);
    }

    #[test]
    fn replace_wraps_shifted_ascii_in_shift() {
        let inputs = bake(
            &Injection::Replace {
                deletes: 1,
                insert: "A".to_string(),
            },
            SendMode::Unicode,
        );
        // backspace pair, then shift down, A down, A up, shift up
        assert_eq!(inputs.len(), 2 + 4);
        assert_eq!(field(&inputs[2]).0, keys::VK_SHIFT);
        assert_eq!(field(&inputs[3]).0, 0x41);
        assert!(is_up(&inputs[5]), "shift must be released last");
    }

    /// (vk, up) of every event of a one-letter replacement, without the backspace.
    fn letter_events(text: &str, held: Held) -> Vec<(u16, bool)> {
        build_segment_held(1, text, SendMode::Unicode, held)
            .iter()
            .skip(2)
            .map(|input| (field(input).0, is_up(input)))
            .collect()
    }

    #[test]
    fn a_held_shift_is_never_released_for_a_capital() {
        // D-E2E-06: the pair `Shift` down / `Shift` up cleared the user's own
        // `Shift`, so the next physical letter arrived lowercase.
        let held = Held { shift: true, caps: false };
        assert_eq!(letter_events("T", held), vec![(0x54, false), (0x54, true)]);
    }

    #[test]
    fn a_lowercase_letter_under_a_held_shift_lifts_it_and_puts_it_back() {
        let held = Held { shift: true, caps: false };
        assert_eq!(
            letter_events("t", held),
            vec![
                (keys::VK_LSHIFT, true),
                (keys::VK_RSHIFT, true),
                (0x54, false),
                (0x54, true),
                (keys::VK_LSHIFT, false),
            ]
        );
    }

    #[test]
    fn caps_lock_inverts_the_shift_a_letter_needs() {
        let caps = Held { shift: false, caps: true };
        assert_eq!(
            letter_events("t", caps).first(),
            Some(&(keys::VK_SHIFT, false)),
            "caps on: lowercase needs Shift"
        );
        assert_eq!(
            letter_events("T", caps),
            vec![(0x54, false), (0x54, true)],
            "caps on: a capital needs no Shift"
        );
        // Digits and punctuation ignore Caps Lock.
        assert_eq!(letter_events("5", caps), vec![(0x35, false), (0x35, true)]);
    }

    #[test]
    fn nothing_held_keeps_the_classic_shift_pair() {
        let none = Held::default();
        assert_eq!(
            letter_events("T", none),
            vec![
                (keys::VK_SHIFT, false),
                (0x54, false),
                (0x54, true),
                (keys::VK_SHIFT, true)
            ]
        );
        assert_eq!(letter_events("t", none), vec![(0x54, false), (0x54, true)]);
    }

    #[test]
    fn a_restore_chord_shift_counts_as_released() {
        // The restore chord's own key-up is injected first; the letters after it
        // must not assume Shift is still down.
        note_held(Held { shift: true, caps: false });
        let chord = Modifiers { shift: true, ctrl: true, alt: false, win: false };
        let inputs = build_restore(1, "T", SendMode::Unicode, chord);
        let events: Vec<(u16, bool)> = inputs.iter().map(|i| (field(i).0, is_up(i))).collect();
        let t = events.iter().position(|e| *e == (0x54, false)).expect("T down");
        assert_eq!(
            events[t - 1],
            (keys::VK_SHIFT, false),
            "Shift pressed again for the capital"
        );
        note_held(Held::default());
    }

    #[test]
    fn replace_keeps_surrogate_pairs_whole() {
        let inputs = bake(
            &Injection::Replace {
                deletes: 1,
                insert: "😀".to_string(),
            },
            SendMode::Unicode,
        );
        // backspace pair, then the four packet events of D83D DE00
        assert_eq!(inputs.len(), 2 + 4);
        assert_eq!(
            unicode_seqs(&inputs[2..]),
            vec![(0xD83D, false), (0xD83D, true), (0xDE00, false), (0xDE00, true)]
        );
    }

    /// Every backspace precedes every inserted character, whichever encoding
    /// each character uses.
    #[test]
    fn replace_order_is_all_deletes_then_all_text() {
        let inputs = bake(
            &Injection::Replace {
                deletes: 3,
                insert: "bật".to_string(),
            },
            SendMode::Unicode,
        );
        let backs = inputs.iter().take_while(|i| field(i).0 == keys::VK_BACK).count();
        assert_eq!(backs, 6, "3 backspace pairs first");
        assert!(
            inputs[backs..].iter().all(|i| field(i).0 != keys::VK_BACK),
            "no backspace may follow inserted text"
        );
        // 'b' and 't' are scancodes, 'ậ' is a packet.
        assert_eq!(field(&inputs[6]).0, 0x42);
        assert_eq!(field(&inputs[6]).1, keys::scan_for_vk(0x42));
        let vowel = field(&inputs[8]);
        assert_eq!(vowel.0, 0);
        assert_eq!(vowel.2 & KEYEVENTF_UNICODE.0, KEYEVENTF_UNICODE.0);
        assert_eq!(vowel.1, 0x1EAD);
    }

    #[test]
    fn standard_mode_replace_is_unchanged() {
        let inputs = bake(
            &Injection::Replace {
                deletes: 2,
                insert: "tă".to_string(),
            },
            SendMode::Standard,
        );
        // identical to the Unicode-mode replacement apart from the backspace
        // scan code, which Standard carries.
        assert_eq!(inputs.len(), 8);
        assert_eq!(field(&inputs[0]).1, keys::scan_for_vk(keys::VK_BACK));
        assert_eq!(field(&inputs[4]).0, 0x54);
        assert_eq!(unicode_seqs(&inputs[6..]), vec![(0x0103, false), (0x0103, true)]);
    }

    #[test]
    fn empty_send_is_a_no_op() {
        assert_eq!(send(&[]), 0);
    }

    // -- CH2: a restore batch must type exactly the text it was given ---------

    fn chord_ctrl_shift() -> Modifiers {
        Modifiers {
            shift: true,
            ctrl: true,
            alt: false,
            win: false,
        }
    }

    /// Characters the array produces, each with the Shift state in force when
    /// the key went down. Backspaces and modifier events are not characters.
    fn typed(inputs: &[INPUT], stale: Modifiers) -> Vec<(char, bool)> {
        let mut shift = stale.shift;
        let mut out = Vec::new();
        let mut units: Vec<u16> = Vec::new();
        for input in inputs {
            let (vk, scan, flags, _) = field(input);
            let up = flags & KEYEVENTF_KEYUP.0 != 0;
            match vk {
                keys::VK_LSHIFT | keys::VK_RSHIFT | keys::VK_SHIFT => shift = !up,
                0 if flags & KEYEVENTF_UNICODE.0 != 0 => {
                    if up {
                        if let Some(ch) = char::from_u32(u32::from(scan)) {
                            out.push((ch, shift));
                        } else if units.len() < 2 {
                            units.push(scan);
                            if let Ok(text) = String::from_utf16(&units) {
                                if let Some(ch) = text.chars().next() {
                                    out.push((ch, shift));
                                    units.clear();
                                }
                            }
                        }
                    }
                    continue;
                }
                _ => {
                    if !up {
                        if let Some(ch) = keys::us_char(vk, shift, false) {
                            out.push((ch, shift));
                        }
                    }
                }
            }
        }
        out
    }

    /// Replay the array the way the target sees it, starting from a *stale*
    /// modifier state: the key-up that ended the restore chord is still in
    /// flight while the hook injects. `\u{FFFD}` marks a character the chord ate
    /// as a shortcut.
    fn replay(inputs: &[INPUT], stale: Modifiers) -> String {
        let mut shift = stale.shift;
        let mut ctrl = stale.ctrl;
        let mut alt = stale.alt;
        let mut out = String::new();
        let mut units: Vec<u16> = Vec::new();
        for input in inputs {
            let (vk, scan, flags, _) = field(input);
            let up = flags & KEYEVENTF_KEYUP.0 != 0;
            match vk {
                keys::VK_LSHIFT | keys::VK_RSHIFT | keys::VK_SHIFT => shift = !up,
                keys::VK_LCONTROL | keys::VK_RCONTROL | keys::VK_CONTROL => ctrl = !up,
                keys::VK_LMENU | keys::VK_RMENU | keys::VK_MENU => alt = !up,
                keys::VK_BACK => {
                    if !up {
                        out.pop();
                    }
                }
                keys::VK_RETURN => {
                    if !up {
                        out.push('\n');
                    }
                }
                0 if flags & KEYEVENTF_UNICODE.0 != 0 => {
                    if up {
                        units.push(scan);
                        if let Ok(text) = String::from_utf16(&units) {
                            if let Some(ch) = text.chars().next() {
                                out.push(ch);
                                units.clear();
                            }
                        }
                    }
                }
                _ => {
                    if up {
                        continue;
                    }
                    if ctrl || alt {
                        out.push('\u{FFFD}');
                    } else if let Some(ch) = keys::us_char(vk, shift, false) {
                        out.push(ch);
                    }
                }
            }
        }
        out
    }

    #[test]
    fn restore_of_ba6t5_keeps_every_character_unshifted() {
        let inputs = build_restore(5, "ba6t5", SendMode::Unicode, chord_ctrl_shift());
        assert_eq!(
            typed(&inputs, Modifiers::NONE),
            vec![('b', false), ('a', false), ('6', false), ('t', false), ('5', false)],
            "lowercase letters and digits must not be shifted"
        );
    }

    #[test]
    fn restore_shift_decision_is_per_character_not_per_batch() {
        let cases: [(&str, Vec<(char, bool)>); 5] = [
            ("Ba6t5", vec![('B', true), ('a', false), ('6', false), ('t', false), ('5', false)]),
            ("bAtB", vec![('b', false), ('A', true), ('t', false), ('B', true)]),
            ("%", vec![('%', true)]),
            ("^", vec![('^', true)]),
            ("100%/50%", vec![
                ('1', false), ('0', false), ('0', false), ('%', true), ('/', false),
                ('5', false), ('0', false), ('%', true),
            ]),
        ];
        for (text, want) in cases {
            let inputs = build_restore(1, text, SendMode::Unicode, chord_ctrl_shift());
            assert_eq!(typed(&inputs, Modifiers::NONE), want, "text {text:?}");
        }
    }

    #[test]
    fn restore_keeps_diacritics_and_surrogate_pairs_on_the_packet_path() {
        let inputs = build_restore(3, "ăâđêôơư", SendMode::Unicode, chord_ctrl_shift());
        let want: Vec<(char, bool)> = "ăâđêôơư".chars().map(|ch| (ch, false)).collect();
        assert_eq!(typed(&inputs, Modifiers::NONE), want);

        let emoji = build_restore(1, "😀", SendMode::Unicode, chord_ctrl_shift());
        assert_eq!(typed(&emoji, Modifiers::NONE), vec![('😀', false)]);
    }

    #[test]
    fn restore_of_mixed_text_types_it_verbatim() {
        let text = "Ba6t5 100% ^ăâđêôơư 😀";
        let inputs = build_restore(9, text, SendMode::Unicode, chord_ctrl_shift());
        assert_eq!(
            replay(&inputs, Modifiers::NONE),
            text,
            "no stale modifier: the batch must type exactly the text"
        );
    }

    /// CH2, measured: `ba6t5` reached Chrome/VS Code as `BA^T%` because the
    /// chord's key-up was still in flight, so the whole batch was shifted.
    #[test]
    fn restore_neutralises_a_chord_that_is_still_held() {
        let inputs = build_restore(5, "ba6t5", SendMode::Unicode, chord_ctrl_shift());
        let shift_only = Modifiers {
            shift: true,
            ctrl: false,
            alt: false,
            win: false,
        };
        // The pre-fix array (no release prefix) reproduces the field symptom
        // character for character: every letter and digit arrives shifted.
        let legacy = build_segment(5, "ba6t5", SendMode::Unicode);
        assert_eq!(
            replay(&legacy, shift_only),
            "BA^T%",
            "the measured CH2 symptom must be reproduced by the old batch"
        );
        assert_eq!(
            replay(&inputs, shift_only),
            "ba6t5",
            "the batch must release the chord before typing"
        );
        let stale = Modifiers {
            shift: true,
            ctrl: true,
            alt: false,
            win: false,
        };
        assert_eq!(
            replay(&inputs, stale),
            "ba6t5",
            "Ctrl still down must not turn the batch into shortcuts"
        );
        assert_eq!(
            replay(&legacy, stale),
            "\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}",
            "…the old batch is eaten as `Ctrl+letter` shortcuts"
        );
        // Nothing extra is released for a chord that has no modifiers.
        let plain = build_restore(5, "ba6t5", SendMode::Unicode, Modifiers::NONE);
        assert_eq!(plain.len(), 5 * 2 + 5 * 2, "no prefix events");
    }

    #[test]
    fn restore_prefix_is_key_ups_only() {
        let inputs = build_restore(1, "a", SendMode::Unicode, chord_ctrl_shift());
        // Shift (both sides) and Ctrl (both sides), all key-ups, all ours.
        assert_eq!(inputs.len(), 4 + 2 + 2, "4 release events, backspace, 'a'");
        let released: Vec<(u16, bool, u32)> =
            inputs[..4].iter().map(|i| (field(i).0, is_up(i), field(i).2)).collect();
        assert_eq!(
            released,
            vec![
                (keys::VK_LSHIFT, true, KEYEVENTF_KEYUP.0),
                (keys::VK_RSHIFT, true, KEYEVENTF_KEYUP.0),
                (keys::VK_LCONTROL, true, KEYEVENTF_KEYUP.0),
                (
                    keys::VK_RCONTROL,
                    true,
                    KEYEVENTF_KEYUP.0 | KEYEVENTF_EXTENDEDKEY.0
                ),
            ]
        );
        for input in &inputs {
            assert_eq!(field(input).3, injected_tag(), "every event stays tagged");
        }
    }

    #[test]
    fn ctrl_v_presses_and_releases_in_order() {
        let inputs = ctrl_v_sequence();
        assert_eq!(inputs.len(), 4);
        let keys: Vec<(u16, bool)> = inputs
            .iter()
            .map(|input| {
                let ki = unsafe { input.Anonymous.ki };
                (ki.wVk.0, ki.dwFlags.0 & KEYEVENTF_KEYUP.0 != 0)
            })
            .collect();
        assert_eq!(
            keys,
            vec![
                (keys::VK_CONTROL, false),
                (0x56, false),
                (0x56, true),
                (keys::VK_CONTROL, true)
            ]
        );
    }
}
