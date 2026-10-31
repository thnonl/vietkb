//! Pure decisions for the Windows clipboard send mode (D-WIN-12..15).
//!
//! No OS calls: the Windows backend feeds these functions the numbers it read
//! from the clipboard and acts on the answers. Keeping them here lets the host
//! test suite cover them.

/// One key event of the paste chord.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChordKey {
    pub vk: u16,
    pub scan: u16,
    pub up: bool,
}

pub const VK_V: u16 = 0x56;
pub const VK_LCONTROL: u16 = 0xA2;
/// Set-1 scan codes: left Ctrl, `V`.
pub const SCAN_LCONTROL: u16 = 0x1D;
pub const SCAN_V: u16 = 0x2F;

/// `Ctrl+V` as a real chord: left Ctrl down, V down, V up, left Ctrl up.
///
/// Uses `VK_LCONTROL` (not the generic `VK_CONTROL`) with its own scan code so
/// the key-state table and the message stream agree on which Ctrl is down. The
/// old chord used the generic code and was sent from a different thread than the
/// Backspaces; a target that missed the Ctrl state received a plain `v`
/// (D-WIN-14).
pub fn paste_chord() -> [ChordKey; 4] {
    [
        ChordKey { vk: VK_LCONTROL, scan: SCAN_LCONTROL, up: false },
        ChordKey { vk: VK_V, scan: SCAN_V, up: false },
        ChordKey { vk: VK_V, scan: SCAN_V, up: true },
        ChordKey { vk: VK_LCONTROL, scan: SCAN_LCONTROL, up: true },
    ]
}

// Standard clipboard formats.
pub const CF_TEXT: u32 = 1;
pub const CF_BITMAP: u32 = 2;
pub const CF_METAFILEPICT: u32 = 3;
pub const CF_DIB: u32 = 8;
pub const CF_PALETTE: u32 = 9;
pub const CF_UNICODETEXT: u32 = 13;
pub const CF_ENHMETAFILE: u32 = 14;
pub const CF_OWNERDISPLAY: u32 = 0x80;
pub const CF_DSPTEXT: u32 = 0x81;
pub const CF_DSPBITMAP: u32 = 0x82;
pub const CF_DSPMETAFILEPICT: u32 = 0x83;
pub const CF_DSPENHMETAFILE: u32 = 0x8E;
pub const CF_DIBV5: u32 = 17;

/// First registered (`RegisterClipboardFormat`) format id.
pub const REGISTERED_FIRST: u32 = 0xC000;

/// Registered names that keep our temporary text out of Win+V history, cloud
/// sync and clipboard monitors. Each is written as a `DWORD 0`.
pub const PRIVACY_FORMATS: [&str; 3] = [
    "CanIncludeInClipboardHistory",
    "CanUploadToCloudClipboard",
    "ExcludeClipboardContentFromMonitorProcessing",
];

/// How the snapshot treats one format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormatClass {
    /// `HGLOBAL` memory: copy byte for byte and put back.
    Copy,
    /// A GDI handle Windows re-synthesizes from a copied format (CF_BITMAP and
    /// CF_PALETTE from the DIB, display variants): skip, nothing is lost.
    Derived,
    /// A privacy flag format: dropped, replaced by ours.
    Flag,
    /// Cannot be copied and is not derivable (metafiles, private handle ranges,
    /// OLE `DataObject`, owner-display): restoring would lose it.
    Lossy,
}

/// Classify a format id (`name` is its registered name, when `id >= 0xC000`).
pub fn classify_format(id: u32, name: Option<&str>) -> FormatClass {
    match id {
        CF_BITMAP | CF_PALETTE | CF_DSPBITMAP | CF_DSPTEXT => FormatClass::Derived,
        CF_METAFILEPICT | CF_ENHMETAFILE | CF_DSPMETAFILEPICT | CF_DSPENHMETAFILE
        | CF_OWNERDISPLAY => FormatClass::Lossy,
        // CF_PRIVATEFIRST..CF_PRIVATELAST and CF_GDIOBJFIRST..CF_GDIOBJLAST hold
        // application-defined handles.
        0x200..=0x3FF => FormatClass::Lossy,
        id if id >= REGISTERED_FIRST => match name {
            Some(n) if PRIVACY_FORMATS.iter().any(|f| f.eq_ignore_ascii_case(n)) => {
                FormatClass::Flag
            }
            // The OLE data object only lives in the owner process.
            Some(n) if n.eq_ignore_ascii_case("DataObject") => FormatClass::Lossy,
            Some(_) => FormatClass::Copy,
            // A registered id we cannot name is not worth the risk.
            None => FormatClass::Lossy,
        },
        // CF_TEXT, CF_DIB, CF_DIBV5, CF_UNICODETEXT, CF_HDROP, CF_LOCALE, ...: all HGLOBAL.
        1 | 4..=8 | 10..=13 | 15..=17 => FormatClass::Copy,
        _ => FormatClass::Lossy,
    }
}

/// What the clipboard holds, as far as a safe round trip is concerned.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// Everything can be put back (an empty clipboard is faithful too).
    Faithful,
    /// Something would be lost: do not use the clipboard, type the text instead.
    Lossy,
}

/// Verdict for a whole clipboard. `formats` is `(id, registered name)` in
/// enumeration order. A GDI-only bitmap (CF_BITMAP without any DIB format) is
/// lossy: there is nothing to rebuild it from.
pub fn snapshot_verdict<'a>(formats: impl IntoIterator<Item = (u32, Option<&'a str>)>) -> Verdict {
    let mut has_dib = false;
    let mut needs_dib = false;
    for (id, name) in formats {
        match classify_format(id, name) {
            FormatClass::Lossy => return Verdict::Lossy,
            FormatClass::Derived if matches!(id, CF_BITMAP | CF_PALETTE) => needs_dib = true,
            _ => {}
        }
        if matches!(id, CF_DIB | CF_DIBV5) {
            has_dib = true;
        }
    }
    if needs_dib && !has_dib {
        Verdict::Lossy
    } else {
        Verdict::Faithful
    }
}

/// Per-format and total byte budgets for a snapshot.
pub const MAX_FORMAT_BYTES: usize = 32 << 20;
pub const MAX_TOTAL_BYTES: usize = 64 << 20;

/// `true` while one more format of `size` bytes still fits the budget.
pub fn within_budget(total_so_far: usize, size: usize) -> bool {
    size <= MAX_FORMAT_BYTES && total_so_far.saturating_add(size) <= MAX_TOTAL_BYTES
}

/// How long the clipboard keeps our text after the last paste chord before it is
/// put back. There is no signal for "the target consumed the paste", so this is
/// a conservative wait (slow Electron/VM targets read the clipboard well after
/// the key event); a newer job inside the window extends it.
pub const SETTLE_MS: u64 = 400;

/// Milliseconds still to wait before restoring, `elapsed_ms` after the last
/// write; `0` = due.
pub fn settle_remaining_ms(elapsed_ms: u64) -> u64 {
    SETTLE_MS.saturating_sub(elapsed_ms)
}

/// `true` only when the clipboard is exactly as we left it: no one else wrote
/// to it since we did. `0` means the sequence number is unavailable, which is
/// never treated as "ours".
pub fn still_ours(expected_seq: u32, current_seq: u32) -> bool {
    expected_seq != 0 && expected_seq == current_seq
}

/// What to do with the saved user clipboard.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RestoreAction {
    /// Put the snapshot back.
    Restore,
    /// Someone else changed the clipboard since our write: drop the snapshot.
    Abandon,
}

pub fn restore_action(expected_seq: u32, current_seq: u32) -> RestoreAction {
    if still_ours(expected_seq, current_seq) {
        RestoreAction::Restore
    } else {
        RestoreAction::Abandon
    }
}

/// More formats than this: treat the clipboard as not safely enumerable.
pub const MAX_FORMATS: usize = 512;

/// `true` while a clipboard with `count` formats is still safely enumerable.
pub fn format_count_ok(count: usize) -> bool {
    count <= MAX_FORMATS
}

/// What the hook does at expansion time. It never blocks on the clipboard: it
/// compares sequence numbers only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PasteDecision {
    /// Our temporary text is still on the clipboard from the previous paste of
    /// this session: write the next text, keep the session's snapshot.
    ContinueSession,
    /// The ready snapshot matches the clipboard exactly: take it, paste.
    StartSession,
    /// No fresh snapshot (missing, lossy, clipboard changed since, sequence
    /// unavailable) or mode off: type the text with Unicode `SendInput`.
    TypeDirectly,
}

/// `session_expected`: sequence right after our last write, while a session is
/// active. `snapshot_seq`: sequence the ready (faithful) snapshot was taken at.
/// `current`: `GetClipboardSequenceNumber()` now. `0` is "unavailable" and never
/// matches.
pub fn decide_paste(
    enabled: bool,
    session_expected: Option<u32>,
    snapshot_seq: Option<u32>,
    current: u32,
) -> PasteDecision {
    if !enabled || current == 0 {
        return PasteDecision::TypeDirectly;
    }
    if session_expected == Some(current) {
        return PasteDecision::ContinueSession;
    }
    if snapshot_seq == Some(current) {
        return PasteDecision::StartSession;
    }
    PasteDecision::TypeDirectly
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flag_formats_do_not_make_a_snapshot_lossy() {
        let f = [
            (CF_UNICODETEXT, None),
            (0xC001, Some("CanUploadToCloudClipboard")),
            (0xC002, Some("ExcludeClipboardContentFromMonitorProcessing")),
        ];
        assert_eq!(snapshot_verdict(f), Verdict::Faithful);
    }

    #[test]
    fn data_object_makes_a_snapshot_lossy() {
        let f = [(CF_UNICODETEXT, None), (0xC010, Some("DataObject"))];
        assert_eq!(snapshot_verdict(f), Verdict::Lossy);
    }

    #[test]
    fn display_variants_are_derived_or_lossy() {
        assert_eq!(classify_format(CF_DSPTEXT, None), FormatClass::Derived);
        assert_eq!(classify_format(CF_DSPBITMAP, None), FormatClass::Derived);
        assert_eq!(classify_format(CF_DSPMETAFILEPICT, None), FormatClass::Lossy);
        assert_eq!(classify_format(CF_DSPENHMETAFILE, None), FormatClass::Lossy);
    }

    #[test]
    fn format_count_limit() {
        assert!(format_count_ok(0));
        assert!(format_count_ok(MAX_FORMATS));
        assert!(!format_count_ok(MAX_FORMATS + 1));
    }

    #[test]
    fn paste_needs_a_fresh_snapshot() {
        use PasteDecision::*;
        // No snapshot, no session: type.
        assert_eq!(decide_paste(true, None, None, 10), TypeDirectly);
        // Snapshot taken at the current sequence: paste.
        assert_eq!(decide_paste(true, None, Some(10), 10), StartSession);
        // Clipboard changed since the snapshot: type.
        assert_eq!(decide_paste(true, None, Some(10), 11), TypeDirectly);
        // Mode off: type, whatever is ready.
        assert_eq!(decide_paste(false, Some(10), Some(10), 10), TypeDirectly);
    }

    #[test]
    fn session_continues_only_while_the_clipboard_is_ours() {
        use PasteDecision::*;
        assert_eq!(decide_paste(true, Some(20), None, 20), ContinueSession);
        // Someone copied since our write: the session is dead.
        assert_eq!(decide_paste(true, Some(20), None, 21), TypeDirectly);
        // ... unless a fresh snapshot of the new content is ready.
        assert_eq!(decide_paste(true, Some(20), Some(21), 21), StartSession);
        // The session wins when both match (our own temp text).
        assert_eq!(decide_paste(true, Some(20), Some(20), 20), ContinueSession);
    }

    #[test]
    fn unavailable_sequence_never_pastes() {
        use PasteDecision::*;
        assert_eq!(decide_paste(true, Some(0), Some(0), 0), TypeDirectly);
        assert_eq!(decide_paste(true, None, Some(0), 0), TypeDirectly);
        assert_eq!(decide_paste(true, None, Some(u32::MAX), 0), TypeDirectly);
        assert_eq!(decide_paste(true, None, Some(u32::MAX), u32::MAX), StartSession);
    }

    #[test]
    fn settle_counts_from_the_last_write() {
        assert_eq!(settle_remaining_ms(0), SETTLE_MS);
        assert_eq!(settle_remaining_ms(SETTLE_MS - 1), 1);
        assert_eq!(settle_remaining_ms(SETTLE_MS), 0);
        assert_eq!(settle_remaining_ms(u64::MAX), 0);
    }

    #[test]
    fn sequence_zero_and_wrap() {
        assert!(!still_ours(u32::MAX, 0));
        assert!(still_ours(u32::MAX, u32::MAX));
        assert_eq!(restore_action(u32::MAX, 0), RestoreAction::Abandon);
        assert_eq!(restore_action(5, 0), RestoreAction::Abandon);
    }


    #[test]
    fn chord_is_a_balanced_left_ctrl_v() {
        let c = paste_chord();
        assert_eq!(
            c.map(|k| (k.vk, k.scan, k.up)),
            [
                (VK_LCONTROL, 0x1D, false),
                (VK_V, 0x2F, false),
                (VK_V, 0x2F, true),
                (VK_LCONTROL, 0x1D, true),
            ]
        );
    }

    #[test]
    fn gdi_handles_are_never_copied() {
        for id in [CF_BITMAP, CF_PALETTE, CF_DSPBITMAP] {
            assert_eq!(classify_format(id, None), FormatClass::Derived);
        }
        for id in [CF_METAFILEPICT, CF_ENHMETAFILE, 0x205, 0x305, CF_OWNERDISPLAY] {
            assert_eq!(classify_format(id, None), FormatClass::Lossy);
        }
    }

    #[test]
    fn global_memory_formats_are_copied() {
        for id in [CF_TEXT, CF_UNICODETEXT, CF_DIB, CF_DIBV5, 15, 16, 7] {
            assert_eq!(classify_format(id, None), FormatClass::Copy, "{id}");
        }
        assert_eq!(classify_format(0xC0A1, Some("PNG")), FormatClass::Copy);
        assert_eq!(classify_format(0xC0A2, Some("HTML Format")), FormatClass::Copy);
    }

    #[test]
    fn registered_edge_cases() {
        assert_eq!(
            classify_format(0xC001, Some("canincludeinclipboardhistory")),
            FormatClass::Flag
        );
        assert_eq!(classify_format(0xC002, Some("DataObject")), FormatClass::Lossy);
        assert_eq!(classify_format(0xC003, None), FormatClass::Lossy);
        assert_eq!(classify_format(0x1234, None), FormatClass::Lossy);
    }

    #[test]
    fn verdicts() {
        assert_eq!(snapshot_verdict([]), Verdict::Faithful);
        let text = [(CF_UNICODETEXT, None), (CF_TEXT, None)];
        assert_eq!(snapshot_verdict(text), Verdict::Faithful);
        // Screenshot: CF_BITMAP is synthesized next to the DIBs.
        let shot = [(CF_DIB, None), (CF_DIBV5, None), (CF_BITMAP, None), (0xC0A1, Some("PNG"))];
        assert_eq!(snapshot_verdict(shot), Verdict::Faithful);
        // GDI-only bitmap: nothing to rebuild it from.
        assert_eq!(snapshot_verdict([(CF_BITMAP, None)]), Verdict::Lossy);
        assert_eq!(
            snapshot_verdict([(CF_UNICODETEXT, None), (CF_ENHMETAFILE, None)]),
            Verdict::Lossy
        );
    }

    #[test]
    fn budget() {
        assert!(within_budget(0, MAX_FORMAT_BYTES));
        assert!(!within_budget(0, MAX_FORMAT_BYTES + 1));
        assert!(!within_budget(MAX_TOTAL_BYTES - 10, 11));
        assert!(!within_budget(usize::MAX, 1));
    }

    #[test]
    fn restore_only_when_nobody_else_wrote() {
        assert_eq!(restore_action(7, 7), RestoreAction::Restore);
        assert_eq!(restore_action(7, 8), RestoreAction::Abandon);
        assert_eq!(restore_action(0, 0), RestoreAction::Abandon);
        assert!(!still_ours(0, 5));
    }
}
