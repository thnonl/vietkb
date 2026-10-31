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

#[cfg(test)]
mod tests {
    use super::*;

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
