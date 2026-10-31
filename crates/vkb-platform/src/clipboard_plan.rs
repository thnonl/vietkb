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
    /// Our own ownership marker ([`OWNED_MARKER`]): not user data, never copied.
    Marker,
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
            Some(n) if n.eq_ignore_ascii_case(OWNED_MARKER) => FormatClass::Marker,
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

/// Pause before the worker opens the clipboard to re-snapshot after a foreign
/// change, so it never collides with the target's own `OpenClipboard` for a
/// paste in flight (D-WIN-17) and bursts of changes collapse into one capture.
pub const REFRESH_DELAY_MS: u32 = 100;

/// Most retired buffers the hook may leave queued for the worker to free.
pub const GARBAGE_CAP: usize = 4;

/// Append `item`, keeping at most `cap` entries: the oldest is evicted and
/// returned so the caller can drop (zero) it right away.
pub fn push_bounded<T>(queue: &mut Vec<T>, item: T, cap: usize) -> Option<T> {
    queue.push(item);
    if queue.len() > cap.max(1) {
        Some(queue.remove(0))
    } else {
        None
    }
}

/// `true` once the settle wait is over (`elapsed_ms` after the last write).
pub fn restore_due(elapsed_ms: u64) -> bool {
    settle_remaining_ms(elapsed_ms) == 0
}

/// The clipboard sequence numbers that identify one of OUR writes. A write
/// moves the number while the clipboard is open and `CloseClipboard` moves it
/// again (D-WIN-16), so both values are ours; which one a listener sees depends
/// on timing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OwnSeqs {
    pub in_open: u32,
    pub after_close: u32,
}

impl OwnSeqs {
    /// `0` means "unavailable" and never matches.
    pub fn matches(&self, current: u32) -> bool {
        current != 0 && (current == self.in_open || current == self.after_close)
    }
}

/// Is a `WM_CLIPBOARDUPDATE` our own write? `session` / `restored` are the
/// numbers recorded under the shared lock for our last paste and for the user's
/// data we put back (adopted as the next snapshot, so not re-captured);
/// `current` is read AFTER taking the same lock, so a write in flight cannot
/// look foreign.
pub fn notification_is_ours(
    session: Option<OwnSeqs>,
    restored: Option<OwnSeqs>,
    current: u32,
) -> bool {
    session.is_some_and(|s| s.matches(current)) || restored.is_some_and(|s| s.matches(current))
}

/// Name of the private marker format set together with every clipboard content
/// WE write (temporary text and restored data). Its presence, together with
/// the owner window, proves a clipboard state is ours without comparing
/// sequence-number distances. It is not user data: never snapshotted.
pub const OWNED_MARKER: &str = "VietKB.Owned";

/// What the bracket after one of our writes found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WriteVerdict {
    /// Owner and marker are ours and the sequence number was stable: `seq`.
    Ours(u32),
    /// Owner/marker prove somebody else wrote: the clipboard is theirs.
    Foreign,
    /// Owner and marker are ours but the number kept moving (our own close
    /// bump landing late, or a write racing it): keep the session, remember the
    /// latest number. User data is never discarded on uncertainty.
    Unstable(u32),
}

/// `owner_is_ours`: `GetClipboardOwner()` is our window (or we wrote with the
/// NULL owner, in which case only the marker counts). `s1` / `s2`: sequence
/// number read before and after the owner/marker probes, with no clipboard open.
pub fn classify_write(owner_is_ours: bool, marker_present: bool, s1: u32, s2: u32) -> WriteVerdict {
    if !owner_is_ours || !marker_present {
        WriteVerdict::Foreign
    } else if s1 == s2 && s2 != 0 {
        WriteVerdict::Ours(s2)
    } else {
        WriteVerdict::Unstable(s2)
    }
}

/// Snapshot of a foreign (read-only) clipboard: accepted only at a number that
/// did not move across the bracket.
pub fn snapshot_stable(s1: u32, s2: u32) -> Option<u32> {
    (s1 == s2 && s2 != 0).then_some(s2)
}

/// Privacy: does a clipboard flag format forbid keeping a copy of the data?
/// `ExcludeClipboardContentFromMonitorProcessing` always does; the
/// `CanIncludeInClipboardHistory` / `CanUploadToCloudClipboard` DWORDs do when 0
/// (or unreadable: be conservative).
pub fn flag_blocks_snapshot(name: &str, value: Option<u32>) -> bool {
    if name.eq_ignore_ascii_case("ExcludeClipboardContentFromMonitorProcessing") {
        return true;
    }
    !matches!(value, Some(v) if v != 0)
}

/// What to do with the saved user clipboard.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RestoreAction {
    /// Put the snapshot back.
    Restore,
    /// Someone else changed the clipboard since our write: drop the snapshot.
    Abandon,
}

/// Restore only while the clipboard is still exactly one of our writes.
pub fn restore_action(expected: OwnSeqs, current: u32) -> RestoreAction {
    if expected.matches(current) {
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

/// `session_expected`: the numbers of our last write, while a session is
/// active. `snapshot_seq`: sequence the ready (faithful) snapshot was taken at.
/// `current`: `GetClipboardSequenceNumber()` now. `0` is "unavailable" and never
/// matches.
pub fn decide_paste(
    enabled: bool,
    session_expected: Option<OwnSeqs>,
    snapshot_seq: Option<u32>,
    current: u32,
) -> PasteDecision {
    if !enabled || current == 0 {
        return PasteDecision::TypeDirectly;
    }
    if session_expected.is_some_and(|s| s.matches(current)) {
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
    fn settle_counts_from_the_last_write() {
        assert_eq!(settle_remaining_ms(0), SETTLE_MS);
        assert_eq!(settle_remaining_ms(SETTLE_MS - 1), 1);
        assert_eq!(settle_remaining_ms(SETTLE_MS), 0);
        assert_eq!(settle_remaining_ms(u64::MAX), 0);
    }

    #[test]
    fn bounded_queue_evicts_the_oldest() {
        let mut q = Vec::new();
        for i in 0..GARBAGE_CAP {
            assert_eq!(push_bounded(&mut q, i, GARBAGE_CAP), None);
        }
        assert_eq!(push_bounded(&mut q, 99, GARBAGE_CAP), Some(0));
        assert_eq!(q, vec![1, 2, 3, 99]);
        // A zero cap still keeps one entry.
        let mut one = Vec::new();
        assert_eq!(push_bounded(&mut one, 1, 0), None);
        assert_eq!(push_bounded(&mut one, 2, 0), Some(1));
        assert_eq!(one, vec![2]);
    }

    #[test]
    fn restore_is_due_only_after_the_settle_wait() {
        assert!(!restore_due(0));
        assert!(!restore_due(SETTLE_MS - 1));
        assert!(restore_due(SETTLE_MS));
    }

    #[test]
    fn privacy_flags_block_the_snapshot() {
        assert!(flag_blocks_snapshot("ExcludeClipboardContentFromMonitorProcessing", Some(1)));
        assert!(flag_blocks_snapshot("ExcludeClipboardContentFromMonitorProcessing", None));
        assert!(flag_blocks_snapshot("CanIncludeInClipboardHistory", Some(0)));
        assert!(flag_blocks_snapshot("CanUploadToCloudClipboard", Some(0)));
        assert!(flag_blocks_snapshot("CanUploadToCloudClipboard", None));
        assert!(!flag_blocks_snapshot("CanIncludeInClipboardHistory", Some(1)));
        assert!(!flag_blocks_snapshot("canuploadtocloudclipboard", Some(1)));
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

    fn own(in_open: u32, after_close: u32) -> OwnSeqs {
        OwnSeqs { in_open, after_close }
    }

    #[test]
    fn own_seqs_match_both_values_and_never_zero() {
        let o = own(100, 103);
        assert!(o.matches(100));
        assert!(o.matches(103));
        assert!(!o.matches(101));
        assert!(!o.matches(104));
        assert!(!own(0, 0).matches(0));
    }

    #[test]
    fn write_verdicts() {
        // Stable, owner and marker ours.
        assert_eq!(classify_write(true, true, 103, 103), WriteVerdict::Ours(103));
        // Number still moving but owner+marker ours: keep the session.
        assert_eq!(classify_write(true, true, 103, 104), WriteVerdict::Unstable(104));
        assert_eq!(classify_write(true, true, 0, 0), WriteVerdict::Unstable(0));
        // Owner or marker prove somebody else wrote.
        assert_eq!(classify_write(false, true, 103, 103), WriteVerdict::Foreign);
        assert_eq!(classify_write(true, false, 103, 103), WriteVerdict::Foreign);
        assert_eq!(classify_write(false, false, 1, 2), WriteVerdict::Foreign);
    }

    #[test]
    fn snapshot_needs_a_stable_bracket() {
        assert_eq!(snapshot_stable(7, 7), Some(7));
        assert_eq!(snapshot_stable(7, 8), None);
        assert_eq!(snapshot_stable(0, 0), None);
    }

    #[test]
    fn marker_is_never_user_data() {
        assert_eq!(classify_format(0xC0F0, Some("vietkb.owned")), FormatClass::Marker);
        // It does not make a snapshot lossy.
        let f = [(CF_UNICODETEXT, None), (0xC0F0, Some(OWNED_MARKER))];
        assert_eq!(snapshot_verdict(f), Verdict::Faithful);
    }

    #[test]
    fn our_own_notification_is_recognised() {
        // Either the in-open or the after-close number is ours.
        assert!(notification_is_ours(Some(own(100, 103)), None, 100));
        assert!(notification_is_ours(Some(own(100, 103)), None, 103));
        assert!(!notification_is_ours(Some(own(100, 103)), None, 104));
        assert!(!notification_is_ours(None, None, 30));
        assert!(!notification_is_ours(Some(own(0, 0)), Some(own(0, 0)), 0));
        // Our restore (adopted as the snapshot) is not re-captured, a later
        // foreign write is.
        assert!(notification_is_ours(None, Some(own(40, 43)), 43));
        assert!(!notification_is_ours(None, Some(own(40, 43)), 44));
    }

    #[test]
    fn paste_needs_a_fresh_snapshot() {
        use PasteDecision::*;
        assert_eq!(decide_paste(true, None, None, 10), TypeDirectly);
        assert_eq!(decide_paste(true, None, Some(10), 10), StartSession);
        assert_eq!(decide_paste(true, None, Some(10), 11), TypeDirectly);
        assert_eq!(decide_paste(false, Some(own(10, 10)), Some(10), 10), TypeDirectly);
    }

    #[test]
    fn session_continues_only_while_the_clipboard_is_ours() {
        use PasteDecision::*;
        assert_eq!(decide_paste(true, Some(own(18, 20)), None, 20), ContinueSession);
        assert_eq!(decide_paste(true, Some(own(18, 20)), None, 18), ContinueSession);
        assert_eq!(decide_paste(true, Some(own(18, 20)), None, 21), TypeDirectly);
        assert_eq!(decide_paste(true, Some(own(18, 20)), Some(21), 21), StartSession);
        assert_eq!(decide_paste(true, Some(own(20, 20)), Some(20), 20), ContinueSession);
    }

    #[test]
    fn adopted_restore_is_a_matching_snapshot() {
        use PasteDecision::*;
        assert_eq!(decide_paste(true, None, Some(40), 40), StartSession);
        assert_eq!(decide_paste(true, None, Some(40), 41), TypeDirectly);
    }

    #[test]
    fn unavailable_sequence_never_pastes() {
        use PasteDecision::*;
        assert_eq!(decide_paste(true, Some(own(0, 0)), Some(0), 0), TypeDirectly);
        assert_eq!(decide_paste(true, None, Some(u32::MAX), 0), TypeDirectly);
        assert_eq!(decide_paste(true, None, Some(u32::MAX), u32::MAX), StartSession);
    }

    #[test]
    fn restore_only_while_the_clipboard_is_one_of_our_writes() {
        assert_eq!(restore_action(own(7, 9), 9), RestoreAction::Restore);
        assert_eq!(restore_action(own(7, 9), 7), RestoreAction::Restore);
        assert_eq!(restore_action(own(7, 9), 10), RestoreAction::Abandon);
        assert_eq!(restore_action(own(0, 0), 0), RestoreAction::Abandon);
        assert_eq!(restore_action(own(u32::MAX, u32::MAX), 0), RestoreAction::Abandon);
    }
}
