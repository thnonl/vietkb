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
            // Our own state (temporary text / restored data): never the user's
            // data to snapshot.
            FormatClass::Marker => return Verdict::Lossy,
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
pub const SETTLE_MS: u64 = 1200;

// Slow WinForms UI threads read the clipboard late (D-WIN-17): keep a floor.
const _: () = assert!(SETTLE_MS >= 1200);

/// Milliseconds still to wait before restoring, `elapsed_ms` after the last
/// write; `0` = due.
pub fn settle_remaining_ms(elapsed_ms: u64) -> u64 {
    SETTLE_MS.saturating_sub(elapsed_ms)
}

/// Pause before the worker opens the clipboard to re-snapshot after a foreign
/// change, so it never collides with the target's own `OpenClipboard` for a
/// paste in flight (D-WIN-17) and bursts of changes collapse into one capture.
pub const REFRESH_DELAY_MS: u32 = 100;

/// Restore rounds (each = several busy-clipboard attempts) before a session is
/// parked for the mode-off / shutdown flush instead of being retried.
pub const MAX_RESTORE_ROUNDS: u32 = 6;

/// Delay before the next restore round, `None` = out of rounds: the user's data
/// is parked (never dropped) and the flush tries once more.
pub fn restore_retry(rounds_done: u32) -> Option<u32> {
    const DELAYS_MS: [u32; MAX_RESTORE_ROUNDS as usize] = [500, 500, 1000, 1000, 2000, 2000];
    DELAYS_MS.get(rounds_done as usize).copied()
}

/// Open attempts per restore round (`attempt` 0..MAX_OPEN_ATTEMPTS).
pub const MAX_OPEN_ATTEMPTS: u32 = 5;

/// Pause before the next `OpenClipboard` attempt after attempt `attempt`
/// (0-based) failed, `elapsed_ms` into this restore; `None` = stop trying.
/// Normal restores back off 20 << attempt (about 300 ms in total); a flush job
/// (`max_wait_ms` from the book: deadline - now) never sleeps past its budget,
/// and with a tiny budget it makes exactly one try.
pub fn open_retry_delay(attempt: u32, elapsed_ms: u64, max_wait_ms: Option<u64>) -> Option<u64> {
    if attempt + 1 >= MAX_OPEN_ATTEMPTS {
        return None;
    }
    let delay = 20u64 << attempt;
    match max_wait_ms {
        Some(max) if elapsed_ms.saturating_add(delay) > max => None,
        _ => Some(delay),
    }
}

/// FNV-1a over the UTF-16 units of the text we put on the clipboard. Only used
/// to recognise OUR OWN leftover text on a clipboard that OUR window owns
/// ([`should_clear`]); there is no text-based rescue of a restore any more (a
/// manager that re-sets our text without our marker makes the restore abandon
/// it as foreign, a documented limit).
pub fn text_fingerprint(text: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for unit in text.encode_utf16() {
        for byte in unit.to_le_bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    hash
}

/// Restore attempts 0..3 open the clipboard with the hook's owner window; the
/// later ones fall back to the NULL owner (the handle may have gone stale).
/// Proof checks always use the hook owner, whichever handle opened.
pub fn uses_hook_owner(attempt: u32) -> bool {
    attempt < 3
}

/// Index of the newest session among their generations (ties: the last).
/// Only the newest session is ever restored; older ones are retired, or an
/// older copy of the user's data could overwrite a newer one.
pub fn pick_newest(generations: &[u64]) -> Option<usize> {
    generations.iter().enumerate().max_by_key(|(i, g)| (**g, *i)).map(|(i, _)| i)
}

/// Proof for the urgent restore (a failed write left the clipboard half
/// written): our owner window is the owner, OR the sequence number is one of the
/// two recorded at the failure. With the NULL owner only the number counts.
pub fn urgent_proof(owner_is_ours: bool, seq_matches: bool) -> bool {
    owner_is_ours || seq_matches
}

/// Total user-data bytes the hook may leave queued for the worker to zero and
/// free. Above this the caller frees on its own thread (rare) and logs.
pub const MAX_GARBAGE_BYTES: usize = 128 << 20;

/// `true` = queue `add` bytes for the worker; `false` = over the cap, free now.
pub fn garbage_admit(retained: usize, add: usize) -> bool {
    retained.saturating_add(add) <= MAX_GARBAGE_BYTES
}

/// NULL-owner fallback (no window to compare): the marker alone proves nothing,
/// so the number must also be the one read while open (`in_open`).
pub fn null_owner_ok(in_open: u32, s1: u32) -> bool {
    in_open != 0 && s1 == in_open
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

/// How the clipboard was opened for one of our writes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OwnerCheck {
    /// Our owner window: `matches` = `GetClipboardOwner()` is that window.
    Window { matches: bool },
    /// The NULL-owner fallback: no window to compare.
    Null,
}

/// After one of OUR writes, with the clipboard closed: the verdict from owner,
/// marker and a stable bracket. `post_close` is the number read right after the
/// close (`CloseClipboard` advances it, D-WIN-16). The NULL-owner path compares
/// against it; the in-open number can never match after the bump and the
/// adoption of the restored data would silently never happen. (The caller
/// records the in-open number in `OwnSeqs`; it is not an input here.)
pub fn write_verdict(
    owner: OwnerCheck,
    marker_present: bool,
    post_close: u32,
    s1: u32,
    s2: u32,
) -> WriteVerdict {
    let owner_ok = match owner {
        OwnerCheck::Window { matches } => matches,
        OwnerCheck::Null => null_owner_ok(post_close, s1),
    };
    classify_write(owner_ok, marker_present, s1, s2)
}

/// Restore or abandon, from what was observed while the clipboard was open.
///
/// `owner_marker_ours`: our owner window owns it AND the marker is present
/// (never true for the NULL-owner fallback). `owner_is_ours`: the owner window
/// alone (urgent data of a failed write has no marker). A clipboard that
/// matches none of this (a copy, a manager that re-set our text without the
/// marker) is somebody else's: abandon, never overwrite it.
pub fn restore_decision(
    urgent: bool,
    own: OwnSeqs,
    owner_marker_ours: bool,
    owner_is_ours: bool,
    seq: u32,
) -> RestoreAction {
    if urgent {
        return if urgent_proof(owner_is_ours, own.matches(seq)) {
            RestoreAction::Restore
        } else {
            RestoreAction::Abandon
        };
    }
    restore_action(own, owner_marker_ours, seq)
}

/// After a give-up: empty the clipboard when OUR window still owns it and either
/// the marker is present or its text equals the text we recorded before the
/// write (`text_equal`; only ever read from our own window, so it cannot hang).
/// A foreign owner is never cleared: it could be the user's own copy.
pub fn should_clear(owner_is_ours: bool, marker_present: bool, text_equal: bool) -> bool {
    owner_is_ours && (marker_present || text_equal)
}

/// May `GetClipboardData` be called on the OPEN clipboard? Only when our own
/// window (or nobody) owns it, or the current owner answered a `WM_NULL` probe
/// sent right before the read sequence, with the clipboard open (so the owner
/// cannot change in between). A non-answering foreign owner is never read: its
/// delayed render could block the worker with the clipboard locked system-wide.
pub fn may_read(owner_is_ours_or_none: bool, probe_answered: bool) -> bool {
    owner_is_ours_or_none || probe_answered
}

/// What to do with the user's data when writing our temporary text failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FailedWrite {
    /// The clipboard was already emptied: the user's data exists only in
    /// memory, queue its restore (urgent).
    Urgent,
    /// `EmptyClipboard` failed, nothing changed and the user's data is still on
    /// the clipboard: drop our copy of the snapshot (a fresh one follows).
    RetireSnapshot,
    /// Nothing changed and there is no new data: the running session stays as is.
    KeepSession,
}

/// `starting`: this paste would have started a session from a snapshot.
/// `emptied`: `EmptyClipboard` succeeded before the write failed.
pub fn failed_write_plan(starting: bool, emptied: bool) -> FailedWrite {
    match (emptied, starting) {
        (true, _) => FailedWrite::Urgent,
        (false, true) => FailedWrite::RetireSnapshot,
        (false, false) => FailedWrite::KeepSession,
    }
}

/// Length of the text in a `CF_UNICODETEXT` buffer that may hold more than our
/// text: the index of the first NUL within the first `our_units + 1` units.
/// `None` = no terminator that early, so the text is longer than ours (cannot be
/// ours); never reads or decodes beyond our own length.
pub fn bounded_text_len(units: &[u16], our_units: usize) -> Option<usize> {
    let bound = units.len().min(our_units + 1);
    units[..bound].iter().position(|u| *u == 0)
}

/// The size our text occupies as `CF_UNICODETEXT` (units + the terminator).
/// A clipboard text of any other size cannot be ours: compare before decoding.
pub fn our_text_bytes(units: usize) -> usize {
    2 * (units + 1)
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

/// Who wrote the current clipboard, for a state we last wrote as `own`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ownership {
    /// The sequence number is one of ours.
    Ours,
    /// The number moved (a late bump) but owner window + marker still prove the
    /// content is ours: treat as ours and refresh the recorded number.
    OursAtNewSeq(u32),
    /// Somebody else wrote.
    Foreign,
}

/// `owner_marker_ours`: `GetClipboardOwner()` is our window (or the NULL-owner
/// fallback) AND the marker format is present, probed without a sequence
/// comparison. A mismatch alone never drops a session that is provably ours.
pub fn ownership(own: Option<OwnSeqs>, owner_marker_ours: bool, current: u32) -> Ownership {
    match own {
        Some(o) if o.matches(current) => Ownership::Ours,
        Some(_) if owner_marker_ours && current != 0 => Ownership::OursAtNewSeq(current),
        _ => Ownership::Foreign,
    }
}

/// Restore only while the clipboard is still provably one of our writes.
pub fn restore_action(expected: OwnSeqs, owner_marker_ours: bool, current: u32) -> RestoreAction {
    match ownership(Some(expected), owner_marker_ours, current) {
        Ownership::Foreign => RestoreAction::Abandon,
        _ => RestoreAction::Restore,
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
        // A clipboard carrying our marker is our own state: never snapshotted.
        let f = [(CF_UNICODETEXT, None), (0xC0F0, Some(OWNED_MARKER))];
        assert_eq!(snapshot_verdict(f), Verdict::Lossy);
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
        assert_eq!(restore_action(own(7, 9), false, 9), RestoreAction::Restore);
        assert_eq!(restore_action(own(7, 9), false, 7), RestoreAction::Restore);
        assert_eq!(restore_action(own(7, 9), false, 10), RestoreAction::Abandon);
        assert_eq!(restore_action(own(0, 0), false, 0), RestoreAction::Abandon);
        assert_eq!(restore_action(own(u32::MAX, u32::MAX), true, 0), RestoreAction::Abandon);
    }

    #[test]
    fn seq_mismatch_with_owner_and_marker_is_still_ours() {
        // Late bump: the number moved but owner + marker are ours.
        assert_eq!(ownership(Some(own(7, 9)), true, 12), Ownership::OursAtNewSeq(12));
        assert_eq!(restore_action(own(7, 9), true, 12), RestoreAction::Restore);
        // Same mismatch, owner or marker not ours: foreign.
        assert_eq!(ownership(Some(own(7, 9)), false, 12), Ownership::Foreign);
        assert_eq!(restore_action(own(7, 9), false, 12), RestoreAction::Abandon);
        // A matching number is ours regardless of the probe.
        assert_eq!(ownership(Some(own(7, 9)), false, 9), Ownership::Ours);
        // No recorded write, or an unavailable number: never ours.
        assert_eq!(ownership(None, true, 12), Ownership::Foreign);
        assert_eq!(ownership(Some(own(7, 9)), true, 0), Ownership::Foreign);
    }

    #[test]
    fn open_retries_back_off_and_respect_a_flush_budget() {
        // Normal: 20, 40, 80, 160, then stop after the fifth try.
        let normal: Vec<Option<u64>> = (0..5).map(|a| open_retry_delay(a, 0, None)).collect();
        assert_eq!(normal, vec![Some(20), Some(40), Some(80), Some(160), None]);
        // A flush budget caps the total: 100 ms allows 20 + 40 but not + 80.
        assert_eq!(open_retry_delay(0, 0, Some(100)), Some(20));
        assert_eq!(open_retry_delay(1, 20, Some(100)), Some(40));
        assert_eq!(open_retry_delay(2, 60, Some(100)), None);
        // A tiny (or exhausted) budget: one try, no sleeping.
        assert_eq!(open_retry_delay(0, 0, Some(0)), None);
        assert_eq!(open_retry_delay(0, 0, Some(19)), None);
        assert_eq!(open_retry_delay(0, 500, Some(100)), None);
    }

    #[test]
    fn null_owner_adoption_compares_the_post_close_number() {
        // The close advanced the number: 40 while open, 43 after, stable probes.
        assert_eq!(write_verdict(OwnerCheck::Null, true, 43, 43, 43), WriteVerdict::Ours(43));
        // A probe that does not match the post-close number is not proven ours
        // (in particular the in-open number 40 never matches after the bump).
        assert_eq!(write_verdict(OwnerCheck::Null, true, 43, 44, 44), WriteVerdict::Foreign);
        assert_eq!(write_verdict(OwnerCheck::Null, true, 43, 40, 40), WriteVerdict::Foreign);
        // Window owners: the owner flag decides.
        assert_eq!(
            write_verdict(OwnerCheck::Window { matches: true }, true, 43, 43, 43),
            WriteVerdict::Ours(43)
        );
        assert_eq!(
            write_verdict(OwnerCheck::Window { matches: false }, true, 43, 43, 43),
            WriteVerdict::Foreign
        );
        // No marker: foreign, owner or not.
        assert_eq!(write_verdict(OwnerCheck::Null, false, 43, 43, 43), WriteVerdict::Foreign);
    }

    #[test]
    fn restore_decision_covers_every_proof() {
        let own = OwnSeqs { in_open: 10, after_close: 13 };
        // Numbers match.
        assert_eq!(restore_decision(false, own, false, false, 13), RestoreAction::Restore);
        // Late bump but owner + marker ours.
        assert_eq!(restore_decision(false, own, true, true, 14), RestoreAction::Restore);
        // A foreign copy, or our text re-set by a manager without the marker:
        // abandoned, never overwritten.
        assert_eq!(restore_decision(false, own, false, false, 20), RestoreAction::Abandon);
        // Our window owns it but the marker is gone and the number moved: no.
        assert_eq!(restore_decision(false, own, false, true, 20), RestoreAction::Abandon);
        // Urgent data (failed write, no marker): owner window or number, never
        // the NULL owner alone.
        assert_eq!(restore_decision(true, own, false, true, 99), RestoreAction::Restore);
        assert_eq!(restore_decision(true, own, false, false, 10), RestoreAction::Restore);
        assert_eq!(restore_decision(true, own, false, false, 99), RestoreAction::Abandon);
    }

    #[test]
    fn clear_decision_only_touches_our_window() {
        // Owner ours + marker, or owner ours + the text we recorded.
        assert!(should_clear(true, true, false));
        assert!(should_clear(true, false, true));
        // Owner ours but nothing of ours left (e.g. emptied): nothing to clear.
        assert!(!should_clear(true, false, false));
        // A foreign owner is never cleared, whatever it holds.
        assert!(!should_clear(false, true, true));
        assert!(!should_clear(false, false, true));
        assert!(!should_clear(false, true, false));
    }

    #[test]
    fn a_foreign_owner_is_read_only_after_it_answered() {
        // Our window / no owner: nothing can hang.
        assert!(may_read(true, false));
        // A foreign owner: only with an answered probe taken with the clipboard open.
        assert!(may_read(false, true));
        assert!(!may_read(false, false));
    }

    #[test]
    fn failed_writes_are_handled_by_what_they_changed() {
        // Emptied: the data lives only in memory, restore it.
        assert_eq!(failed_write_plan(true, true), FailedWrite::Urgent);
        assert_eq!(failed_write_plan(false, true), FailedWrite::Urgent);
        // EmptyClipboard failed: the user's data is untouched on the clipboard.
        assert_eq!(failed_write_plan(true, false), FailedWrite::RetireSnapshot);
        assert_eq!(failed_write_plan(false, false), FailedWrite::KeepSession);
    }

    #[test]
    fn bounded_text_len_never_looks_past_our_length() {
        let ours = [b'a' as u16, b'b' as u16, 0];
        assert_eq!(bounded_text_len(&ours, 2), Some(2));
        // A bigger buffer with our text and garbage after the NUL.
        let big = [b'a' as u16, b'b' as u16, 0, 7, 7, 7];
        assert_eq!(bounded_text_len(&big, 2), Some(2));
        // A longer foreign text: no terminator within our length + 1.
        let longer = [b'a' as u16, b'b' as u16, b'c' as u16, 0];
        assert_eq!(bounded_text_len(&longer, 2), None);
        // A buffer shorter than the bound without a NUL.
        assert_eq!(bounded_text_len(&[1, 2], 5), None);
        assert_eq!(bounded_text_len(&[], 0), None);
        // An empty text of ours.
        assert_eq!(bounded_text_len(&[0, 9], 0), Some(0));
    }

    #[test]
    fn text_size_check_precedes_decoding() {
        assert_eq!(our_text_bytes(0), 2);
        assert_eq!(our_text_bytes(5), 12);
    }

    #[test]
    fn fingerprint_tells_our_text_apart() {
        assert_eq!(text_fingerprint("tiếng"), text_fingerprint("tiếng"));
        assert_ne!(text_fingerprint("tiếng"), text_fingerprint("tieng"));
        assert_ne!(text_fingerprint(""), text_fingerprint("a"));
    }

    #[test]
    fn owner_fallback_only_after_three_attempts() {
        assert!(uses_hook_owner(0) && uses_hook_owner(2));
        assert!(!uses_hook_owner(3) && !uses_hook_owner(4));
    }

    #[test]
    fn only_the_newest_session_is_picked() {
        assert_eq!(pick_newest(&[]), None);
        assert_eq!(pick_newest(&[4]), Some(0));
        assert_eq!(pick_newest(&[3, 9, 5]), Some(1));
        // Equal generations: the later entry wins.
        assert_eq!(pick_newest(&[7, 7]), Some(1));
    }

    #[test]
    fn urgent_restore_needs_owner_or_sequence() {
        assert!(urgent_proof(true, false));
        assert!(urgent_proof(false, true));
        // NULL owner and no sequence match: refuse.
        assert!(!urgent_proof(false, false));
    }

    #[test]
    fn restore_rounds_back_off_then_park() {
        assert_eq!(restore_retry(0), Some(500));
        assert_eq!(restore_retry(MAX_RESTORE_ROUNDS - 1), Some(2000));
        assert_eq!(restore_retry(MAX_RESTORE_ROUNDS), None);
        assert_eq!(restore_retry(u32::MAX), None);
    }

    #[test]
    fn null_owner_needs_a_sequence_match() {
        assert!(null_owner_ok(10, 10));
        assert!(!null_owner_ok(10, 13));
        assert!(!null_owner_ok(0, 0));
    }

    #[test]
    fn garbage_queue_is_capped_by_bytes() {
        assert!(garbage_admit(0, MAX_GARBAGE_BYTES));
        assert!(!garbage_admit(0, MAX_GARBAGE_BYTES + 1));
        assert!(!garbage_admit(MAX_GARBAGE_BYTES - 1, 2));
        assert!(!garbage_admit(usize::MAX, 1));
    }
}

