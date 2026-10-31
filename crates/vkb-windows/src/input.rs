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
//! path: it has no delete block and is not affected. A replacement character
//! that would need an injected `Shift` pair is a packet as well (D-WIN-09):
//! Notepad mis-delivers `Shift`-wrapped VK events after a delete block.
//!
//! Clipboard is never part of this path (charter C4); the opt-in fallback lives
//! in [`crate::clipboard`].

use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
    KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE, VIRTUAL_KEY,
};

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

/// The Shift keys and Caps Lock the *user* holds, from the hook's own ordered
/// modifier tracking ([`keys::ModifierState`]). The OS key-state reads are not
/// used to build a batch: inside the low-level callback they can lag one event.
///
/// An injected `Shift` pair must not touch a `Shift` the user holds: the key-up
/// clears it in the OS (D-E2E-06: every later physical letter then arrives
/// lowercase). A letter that needs the other case while `Shift` is held is
/// therefore typed as a Unicode packet, which does not depend on `Shift`.
pub use vkb_platform::plan::HeldKeys as Held;

/// Shift key-ups and key-downs a batch needs after it stopped halfway.
///
/// Ownership is explicit, never inferred from the OS: the events of the part of
/// the batch the OS accepted (`sent`) are replayed per Shift key.
///
/// * a key **we pressed** and did not release is released;
/// * a key we **released** that the *user* holds (`user_shift`, from the hook's
///   model) because the restore chord lifted it, and did not press again, is
///   pressed again.
///
/// A Shift the user pressed is never released.
pub fn repair_shift(sent: &[INPUT], user_shift: u8) -> Vec<INPUT> {
    // [left, right, generic]: pressed by us / released by us.
    let mut pressed = [false; 3];
    let mut lifted = [false; 3];
    let slot = |vk: u16| match vk {
        keys::VK_LSHIFT => Some(0),
        keys::VK_RSHIFT => Some(1),
        keys::VK_SHIFT => Some(2),
        _ => None,
    };
    for input in sent {
        let ki = unsafe { input.Anonymous.ki };
        if ki.dwFlags.0 & KEYEVENTF_UNICODE.0 != 0 || ki.dwExtraInfo != injected_tag() {
            continue;
        }
        let Some(i) = slot(ki.wVk.0) else { continue };
        if ki.dwFlags.0 & KEYEVENTF_KEYUP.0 != 0 {
            if pressed[i] {
                pressed[i] = false;
            } else {
                lifted[i] = true;
            }
        } else if lifted[i] {
            lifted[i] = false;
        } else {
            pressed[i] = true;
        }
    }
    let mut out = Vec::new();
    for (i, vk) in [keys::VK_LSHIFT, keys::VK_RSHIFT, keys::VK_SHIFT].into_iter().enumerate() {
        let scan = keys::scan_for_vk(vk);
        if pressed[i] {
            push_key(&mut out, vk, scan, true);
        }
        if lifted[i] && user_shift & (1 << i) != 0 {
            push_key(&mut out, vk, scan, false);
        }
    }
    out
}

/// Send the repair for a batch that stopped after `sent` events.
pub fn resync_shift(batch: &[INPUT], sent: usize, user_shift: u8) {
    let repair = repair_shift(&batch[..sent.min(batch.len())], user_shift);
    send_detailed(&repair);
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

/// Backspaces + text in one array, so `Replace` is delivered atomically, for a
/// user who holds no Shift and has Caps Lock off.
pub fn build_segment(deletes: u32, text: &str, mode: SendMode) -> Vec<INPUT> {
    build_segment_held(deletes, text, mode, Held::default())
}

/// [`build_segment`] against the keys the user holds.
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
///
/// `held` is what the *user* still holds once the chord is released (the hook's
/// model after the key-up that fired it). That Shift was lifted for the batch,
/// so the batch ends by pressing exactly those keys again: the OS must agree
/// with the keyboard afterwards.
pub fn build_restore(
    deletes: u32,
    text: &str,
    mode: SendMode,
    chord: Modifiers,
    held: Held,
) -> Vec<INPUT> {
    let mut out =
        Vec::with_capacity(10 + (deletes as usize) * 2 + text.encode_utf16().count() * 2);
    push_chord_release(&mut out, chord);
    // The chord's own `Shift` is up for the rest of the batch.
    let during = Held {
        shift: if chord.shift { 0 } else { held.shift },
        caps: held.caps,
    };
    push_segment(&mut out, deletes, text, mode, during);
    if chord.shift {
        push_shift_down(&mut out, held.shift);
    }
    out
}

/// Press the Shift keys in `mask` ([`keys::SHIFT_L`] and friends).
fn push_shift_down(out: &mut Vec<INPUT>, mask: u8) {
    if mask & keys::SHIFT_L != 0 {
        push_key(out, keys::VK_LSHIFT, keys::scan_for_vk(keys::VK_LSHIFT), false);
    }
    if mask & keys::SHIFT_R != 0 {
        push_key(out, keys::VK_RSHIFT, keys::scan_for_vk(keys::VK_RSHIFT), false);
    }
    if mask & keys::SHIFT_ANY != 0 {
        push_key(out, keys::VK_SHIFT, keys::scan_for_vk(keys::VK_SHIFT), false);
    }
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
        // D-WIN-09: in the default mode a key that needs a `Shift` pair is a
        // packet too. Notepad mis-delivers `Shift`-wrapped VK events that follow
        // a delete block (macro `qa1` -> `QA ONE` came out right 25-50% of the
        // time); a packet carries the case itself. Only `send_standard` keeps
        // the VK + `Shift` path.
        Some((vk, shift))
            if !shift_is_blocked(vk, shift, held)
                && !(mode == SendMode::Unicode && needs_shift_pair(vk, shift, held)) =>
        {
            push_ascii_key(out, vk, shift, held)
        }
        // No US key produces it, or the key would need the user's held Shift
        // changed: a packet carries the character whatever Shift is doing.
        _ => push_unicode_char(out, ch),
    }
}

/// The case a key shows with the user's keys as they are: a letter follows
/// `Shift` xor `Caps Lock`, every other key follows `Shift`.
fn shows_shift(vk: u16, held: Held) -> bool {
    let is_letter = (0x41..=0x5A).contains(&vk);
    if is_letter {
        held.shift_down() ^ held.caps
    } else {
        held.shift_down()
    }
}

/// `true` when typing `vk` with `want_shift` would need the user's held `Shift`
/// released.
fn shift_is_blocked(vk: u16, want_shift: bool, held: Held) -> bool {
    held.shift_down() && shows_shift(vk, held) != want_shift
}

/// `true` when typing `vk` with `want_shift` needs an injected `Shift` pair.
fn needs_shift_pair(vk: u16, want_shift: bool, held: Held) -> bool {
    !held.shift_down() && shows_shift(vk, held) != want_shift
}

/// One ASCII key press with the case the text needs. A `Shift` pair is only
/// added when the user holds none; a held one is never touched.
fn push_ascii_key(out: &mut Vec<INPUT>, vk: u16, want_shift: bool, held: Held) {
    let press_shift = needs_shift_pair(vk, want_shift, held);
    let shift_scan = keys::scan_for_vk(keys::VK_SHIFT);
    if press_shift {
        push_key(out, keys::VK_SHIFT, shift_scan, false);
    }
    let scan = keys::scan_for_vk(vk);
    push_key(out, vk, scan, false);
    push_key(out, vk, scan, true);
    if press_shift {
        push_key(out, keys::VK_SHIFT, shift_scan, true);
    }
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
    if let Some(accepted) = crate::e2e::capture(inputs) {
        let total = inputs.len() as u32;
        return SendResult {
            sent: accepted as u32,
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
    fn standard_replace_wraps_shifted_ascii_in_shift() {
        let inputs = bake(
            &Injection::Replace {
                deletes: 1,
                insert: "A".to_string(),
            },
            SendMode::Standard,
        );
        // backspace pair, then shift down, A down, A up, shift up
        assert_eq!(inputs.len(), 2 + 4);
        assert_eq!(field(&inputs[2]).0, keys::VK_SHIFT);
        assert_eq!(field(&inputs[3]).0, 0x41);
        assert!(is_up(&inputs[5]), "shift must be released last");
    }

    /// D-WIN-09: a macro expansion (`qa1` -> `QA ONE`) is a delete block plus
    /// text. Notepad mis-delivers `Shift`-wrapped VK events after the deletes,
    /// so the capitals are packets and no Shift VK event is built.
    #[test]
    fn macro_expansion_capitals_are_packets_with_no_shift_events() {
        let inputs = bake(
            &Injection::Replace {
                deletes: 3,
                insert: "QA ONE ".to_string(),
            },
            SendMode::Unicode,
        );
        assert!(
            inputs.iter().all(|i| !matches!(
                field(i).0,
                keys::VK_SHIFT | keys::VK_LSHIFT | keys::VK_RSHIFT
            )),
            "no Shift VK event: {:?}",
            events(&inputs)
        );
        // Three real backspaces, then the text.
        for input in &inputs[..6] {
            assert_eq!(field(input).0, keys::VK_BACK);
        }
        let units: Vec<u16> = unicode_seqs(&inputs[6..])
            .into_iter()
            .filter(|e| !e.1)
            .map(|e| e.0)
            .collect();
        // Capitals are packets; the space is the only real key.
        assert_eq!(units, "QAONE".encode_utf16().collect::<Vec<u16>>());
        let spaces = inputs[6..].iter().filter(|i| field(i).0 == 0x20 && !is_up(i)).count();
        assert_eq!(spaces, 2);
        // A punctuation mark that needs Shift is a packet too.
        let bang = bake(&Injection::Replace { deletes: 1, insert: "!".to_string() }, SendMode::Unicode);
        assert_eq!(unicode_seqs(&bang[2..]), vec![(0x21, false), (0x21, true)]);
    }

    /// (vk, up) of every event of a one-letter replacement, without the backspace.
    fn letter_events(text: &str, held: Held) -> Vec<(u16, bool)> {
        build_segment_held(1, text, SendMode::Unicode, held)
            .iter()
            .skip(2)
            .map(|input| (field(input).0, is_up(input)))
            .collect()
    }

    const LEFT: Held = Held { shift: keys::SHIFT_L, caps: false };
    const RIGHT: Held = Held { shift: keys::SHIFT_R, caps: false };

    #[test]
    fn a_held_shift_is_never_released_for_a_capital() {
        // D-E2E-06: the pair `Shift` down / `Shift` up cleared the user's own
        // `Shift`, so the next physical letter arrived lowercase.
        for held in [LEFT, RIGHT] {
            assert_eq!(letter_events("T", held), vec![(0x54, false), (0x54, true)]);
        }
    }

    #[test]
    fn a_lowercase_letter_under_a_held_shift_is_a_packet_and_touches_no_shift() {
        // Either side held: no key-up, so nothing to put back and nothing left
        // stuck (review items 1 and 2).
        for held in [LEFT, RIGHT, Held { shift: keys::SHIFT_ANY, caps: false }] {
            let events = letter_events("t", held);
            assert_eq!(events, vec![(0, false), (0, true)]);
        }
    }

    #[test]
    fn caps_lock_inverts_the_shift_a_letter_needs() {
        let caps = Held { shift: 0, caps: true };
        assert_eq!(
            letter_events("t", caps),
            vec![(0, false), (0, true)],
            "caps on: lowercase needs the other case, a packet"
        );
        assert_eq!(
            letter_events("T", caps),
            vec![(0x54, false), (0x54, true)],
            "caps on: a capital needs no Shift"
        );
        // Digits and punctuation ignore Caps Lock.
        assert_eq!(letter_events("5", caps), vec![(0x35, false), (0x35, true)]);
        // Caps on and Shift held: a capital would need Shift released.
        let both = Held { shift: keys::SHIFT_L, caps: true };
        assert_eq!(letter_events("T", both), vec![(0, false), (0, true)]);
    }

    #[test]
    fn nothing_held_types_a_capital_as_a_packet_in_unicode_mode() {
        let none = Held::default();
        // D-WIN-09: no injected Shift pair in the default mode.
        assert_eq!(letter_events("T", none), vec![(0, false), (0, true)]);
        // `send_standard` keeps the classic pair.
        let standard: Vec<(u16, bool)> = build_segment_held(1, "T", SendMode::Standard, none)
            .iter()
            .skip(2)
            .map(|i| (field(i).0, is_up(i)))
            .collect();
        assert_eq!(
            standard,
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
    fn restore_chord_shift_is_pressed_again_for_exactly_the_keys_the_user_holds() {
        let chord = Modifiers { shift: true, ctrl: true, alt: false, win: false };
        for (mask, vk) in [
            (keys::SHIFT_L, keys::VK_LSHIFT),
            (keys::SHIFT_R, keys::VK_RSHIFT),
            (keys::SHIFT_ANY, keys::VK_SHIFT),
        ] {
            let inputs = build_restore(1, "T", SendMode::Unicode, chord, Held { shift: mask, caps: false });
            let events: Vec<(u16, bool)> = inputs.iter().map(|i| (field(i).0, is_up(i))).collect();
            assert_eq!(events.last(), Some(&(vk, false)), "the held key is pressed last");
            let presses = events.iter().filter(|e| !e.1 && matches!(e.0, 0xA0 | 0xA1 | 0x10)).count();
            // Only the user's key: the capital is a packet and has no pair.
            assert_eq!(presses, 1, "{events:?}");
            assert!(!events.iter().any(|e| e.0 == 0x54), "T is a packet, not a VK");
        }
        // The user holds nothing once the chord is released: nothing is re-pressed.
        let inputs = build_restore(1, "T", SendMode::Unicode, chord, Held::default());
        let last = inputs.last().map(|i| (field(i).0, is_up(i)));
        assert_eq!(last, Some((0, true)), "the packet is last, no Shift is re-pressed");
    }

    /// (vk, up) of every event.
    fn events(inputs: &[INPUT]) -> Vec<(u16, bool)> {
        inputs.iter().map(|i| (field(i).0, is_up(i))).collect()
    }

    #[test]
    fn repair_releases_only_the_shift_we_pressed() {
        // Capital pair `Shift` down, T down, T up, `Shift` up; the OS took the
        // first two events only.
        let batch = build_segment_held(0, "T", SendMode::Standard, Held::default());
        let repair = repair_shift(&batch[..2], 0);
        assert_eq!(events(&repair), vec![(keys::VK_SHIFT, true)]);
        // The whole pair went through: nothing to repair.
        assert!(repair_shift(&batch, 0).is_empty());
        // The OS took nothing: nothing to repair.
        assert!(repair_shift(&batch[..0], 0).is_empty());
    }

    #[test]
    fn repair_never_releases_a_shift_the_user_pressed() {
        // The user holds right Shift; the batch is a capital, which presses no
        // Shift at all, so there is nothing of ours to release however far it got.
        let batch = build_segment_held(1, "T", SendMode::Unicode, RIGHT);
        for cut in 0..=batch.len() {
            assert!(repair_shift(&batch[..cut], keys::SHIFT_R).is_empty(), "cut at {cut}");
        }
    }

    #[test]
    fn repair_presses_again_a_user_shift_the_restore_chord_lifted() {
        let chord = Modifiers { shift: true, ctrl: true, alt: false, win: false };
        let held = Held { shift: keys::SHIFT_R, caps: false };
        let batch = build_restore(1, "a", SendMode::Unicode, chord, held);
        // Stopped after the chord release, before the closing press.
        let cut = batch.len() - 1;
        let repair = repair_shift(&batch[..cut], keys::SHIFT_R);
        assert_eq!(events(&repair), vec![(keys::VK_RSHIFT, false)]);
        // The user let go meanwhile (the hook's model says so): leave it up.
        assert!(repair_shift(&batch[..cut], 0).is_empty());
        // Whole batch: balanced.
        assert!(repair_shift(&batch, keys::SHIFT_R).is_empty());
    }

    /// An independent statement of the batch `main` built before the Shift
    /// changes: every key as written out by hand, nothing shared with the code
    /// under test except the VK / scancode tables.
    fn main_batch(deletes: u32, text: &str, mode: SendMode) -> Vec<(u16, u16, u32, usize)> {
        let tag = injected_tag();
        let down = 0u32;
        let up = KEYEVENTF_KEYUP.0;
        let mut out = Vec::new();
        let back = if mode == SendMode::Standard { keys::scan_for_vk(keys::VK_BACK) } else { 0 };
        for _ in 0..deletes {
            out.push((keys::VK_BACK, back, down, tag));
            out.push((keys::VK_BACK, back, up, tag));
        }
        let scancode_ascii = mode == SendMode::Standard || deletes > 0;
        let mut chars = text.chars().peekable();
        while let Some(ch) = chars.next() {
            // Main's line-break handling: `\r\n` is one Return, a lone `\r` or
            // `\n` is one Return.
            if ch == '\r' && chars.peek() == Some(&'\n') {
                chars.next();
            }
            if ch == '\r' || ch == '\n' {
                let scan = keys::scan_for_vk(keys::VK_RETURN);
                out.push((keys::VK_RETURN, scan, down, tag));
                out.push((keys::VK_RETURN, scan, up, tag));
                continue;
            }
            // D-WIN-09: in the packet default a key that needs Shift is a packet.
            match keys::us_vk_for_char(ch)
                .filter(|&(_, shift)| scancode_ascii && !(shift && mode == SendMode::Unicode))
            {
                Some((vk, shift)) => {
                    let shift_scan = keys::scan_for_vk(keys::VK_SHIFT);
                    if shift {
                        out.push((keys::VK_SHIFT, shift_scan, down, tag));
                    }
                    out.push((vk, keys::scan_for_vk(vk), down, tag));
                    out.push((vk, keys::scan_for_vk(vk), up, tag));
                    if shift {
                        out.push((keys::VK_SHIFT, shift_scan, up, tag));
                    }
                }
                None => {
                    let mut units = [0u16; 2];
                    for unit in ch.encode_utf16(&mut units).iter().copied() {
                        out.push((0, unit, KEYEVENTF_UNICODE.0, tag));
                        out.push((0, unit, KEYEVENTF_UNICODE.0 | up, tag));
                    }
                }
            }
        }
        out
    }

    #[test]
    fn with_no_shift_held_every_batch_is_byte_identical_to_main() {
        // Ruling C (amended by D-WIN-09: Shift-needing keys are packets in the
        // default mode): without a held Shift (and Caps Lock off) nothing changes,
        // for `send_standard` and for the packet default, with and without a
        // delete block.
        let text = "Việt Nam ăâđêôơư ABC xyz 019 !@#$ .,;:'\"[]{}|\\-=_+<>?/`~\n😀 tAs Đ a\r\nb \r x\ny\r";
        for mode in [SendMode::Standard, SendMode::Unicode] {
            for deletes in [0, 1, 4] {
                let built: Vec<_> = build_segment_held(deletes, text, mode, Held::default())
                    .iter()
                    .map(field)
                    .collect();
                assert_eq!(built, main_batch(deletes, text, mode), "{mode:?} deletes={deletes}");
                // The no-argument entry point is the same batch.
                let plain: Vec<_> = build_segment(deletes, text, mode).iter().map(field).collect();
                assert_eq!(plain, built);
            }
        }
    }

    #[test]
    fn caps_lock_on_with_no_shift_held_gives_a_lowercase_letter_a_shift_pair() {
        // Intended: with Caps Lock on, the bare key shows a capital, so a
        // lowercase letter needs a Shift pair and a capital needs none.
        // This deliberately differs from main, which ignored Caps Lock and so
        // typed an uppercase letter wherever the engine wanted a lowercase one.
        let caps = Held { shift: 0, caps: true };
        let batch = build_segment_held(1, "aB", SendMode::Unicode, caps);
        assert_eq!(
            events(&batch).into_iter().skip(2).collect::<Vec<_>>(),
            vec![(0, false), (0, true), (0x42, false), (0x42, true)]
        );
        // Caps Lock does not affect digits or punctuation.
        let batch = build_segment_held(1, "5.", SendMode::Unicode, caps);
        assert_eq!(batch.len(), 2 + 4, "no Shift events: {:?}", events(&batch));
        // Main's batch has no Shift pair around the `a`: the divergence.
        assert_eq!(main_batch(1, "a", SendMode::Unicode).len(), 2 + 2);
    }

    #[test]
    fn a_packet_for_an_ascii_letter_needs_a_held_shift_and_the_other_case() {
        for mode in [SendMode::Standard, SendMode::Unicode] {
            for held in [Held::default(), Held { shift: 0, caps: true }] {
                // Nothing held: `send_standard` never uses a packet for a letter
                // in a delete block; the default mode does for the other case.
                let inputs = build_segment_held(1, "AbCd", mode, held);
                if mode == SendMode::Standard {
                    assert!(
                        inputs.iter().skip(2).all(|i| field(i).0 != 0),
                        "{mode:?} {held:?}: no packet without a held Shift"
                    );
                } else {
                    assert!(
                        inputs.iter().skip(2).all(|i| {
                            !matches!(field(i).0, keys::VK_SHIFT | keys::VK_LSHIFT | keys::VK_RSHIFT)
                        }),
                        "{mode:?} {held:?}: no Shift pair"
                    );
                }
            }
            // Shift held, the letter shows the case it needs: still a key.
            let inputs = build_segment_held(1, "AB", mode, LEFT);
            assert!(inputs.iter().skip(2).all(|i| field(i).0 != 0));
            // Shift held, other case: the one packet.
            let inputs = build_segment_held(1, "a", mode, LEFT);
            assert!(inputs.iter().skip(2).all(|i| field(i).0 == 0));
        }
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
        let inputs = build_restore(5, "ba6t5", SendMode::Unicode, chord_ctrl_shift(), Held::default());
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
            let std = build_restore(1, text, SendMode::Standard, chord_ctrl_shift(), Held::default());
            assert_eq!(typed(&std, Modifiers::NONE), want, "text {text:?}");
            // The default mode types the same text with packets (D-WIN-09).
            let inputs = build_restore(1, text, SendMode::Unicode, chord_ctrl_shift(), Held::default());
            assert_eq!(replay(&inputs, Modifiers::NONE), text, "text {text:?}");
        }
    }

    #[test]
    fn restore_keeps_diacritics_and_surrogate_pairs_on_the_packet_path() {
        let inputs = build_restore(3, "ăâđêôơư", SendMode::Unicode, chord_ctrl_shift(), Held::default());
        let want: Vec<(char, bool)> = "ăâđêôơư".chars().map(|ch| (ch, false)).collect();
        assert_eq!(typed(&inputs, Modifiers::NONE), want);

        let emoji = build_restore(1, "😀", SendMode::Unicode, chord_ctrl_shift(), Held::default());
        assert_eq!(typed(&emoji, Modifiers::NONE), vec![('😀', false)]);
    }

    #[test]
    fn restore_of_mixed_text_types_it_verbatim() {
        let text = "Ba6t5 100% ^ăâđêôơư 😀";
        let inputs = build_restore(9, text, SendMode::Unicode, chord_ctrl_shift(), Held::default());
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
        let inputs = build_restore(5, "ba6t5", SendMode::Unicode, chord_ctrl_shift(), Held::default());
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
        let plain = build_restore(5, "ba6t5", SendMode::Unicode, Modifiers::NONE, Held::default());
        assert_eq!(plain.len(), 5 * 2 + 5 * 2, "no prefix events");
    }

    #[test]
    fn restore_prefix_is_key_ups_only() {
        let inputs = build_restore(1, "a", SendMode::Unicode, chord_ctrl_shift(), Held::default());
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
