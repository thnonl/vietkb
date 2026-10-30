//! Pure planning of an injection: how many Backspaces, which UTF-16 chunks.
//!
//! The FFI layer turns each [`Step`] into one key-down/key-up pair and tags it
//! with [`crate::state::MARKER`]. Kept free of Apple types so it is unit-tested.

/// `CGEventKeyboardSetUnicodeString` documents a 20-unit limit per event; stay
/// well under it so a pair is never truncated.
pub const MAX_UNITS_PER_EVENT: usize = 16;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Step {
    /// A real Backspace key press (keycode 51).
    Backspace,
    /// Typed text carried by the event's Unicode string.
    Text(Vec<u16>),
}

/// Split `text` into chunks of at most [`MAX_UNITS_PER_EVENT`] UTF-16 units that
/// never cut a surrogate pair, and never separate a base letter from the
/// combining marks that follow it (the NFD charset emits `e` + U+0302 + U+0323).
pub fn chunk_utf16(text: &str) -> Vec<Vec<u16>> {
    let mut chunks: Vec<Vec<u16>> = Vec::new();
    let mut current: Vec<u16> = Vec::new();
    let mut cluster: Vec<u16> = Vec::new();
    let mut buf = [0u16; 2];
    for ch in text.chars() {
        let units = ch.encode_utf16(&mut buf);
        if is_combining(ch) && !cluster.is_empty() {
            cluster.extend_from_slice(units);
            continue;
        }
        flush(&mut chunks, &mut current, &mut cluster);
        cluster.extend_from_slice(units);
    }
    flush(&mut chunks, &mut current, &mut cluster);
    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
}

fn flush(chunks: &mut Vec<Vec<u16>>, current: &mut Vec<u16>, cluster: &mut Vec<u16>) {
    if cluster.is_empty() {
        return;
    }
    if !current.is_empty() && current.len() + cluster.len() > MAX_UNITS_PER_EVENT {
        chunks.push(std::mem::take(current));
    }
    current.append(cluster);
}

fn is_combining(ch: char) -> bool {
    matches!(ch, '\u{0300}'..='\u{036F}' | '\u{1AB0}'..='\u{1AFF}' | '\u{1DC0}'..='\u{1DFF}' | '\u{20D0}'..='\u{20FF}' | '\u{FE20}'..='\u{FE2F}')
}

/// The ordered key presses for "delete `deletes` characters, then type `text`".
pub fn plan(deletes: u32, text: &str) -> Vec<Step> {
    let mut steps = Vec::with_capacity(deletes as usize + 1);
    steps.extend((0..deletes).map(|_| Step::Backspace));
    steps.extend(chunk_utf16(text).into_iter().map(Step::Text));
    steps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deletes_come_first_then_text() {
        let steps = plan(2, "ệ");
        assert_eq!(
            steps,
            vec![Step::Backspace, Step::Backspace, Step::Text(vec![0x1EC7])]
        );
    }

    #[test]
    fn nothing_to_do_is_empty() {
        assert!(plan(0, "").is_empty());
    }

    #[test]
    fn long_text_is_chunked_under_the_limit() {
        let text = "aáảãạăắằẳẵặâấầẩẫậeéèẻẽẹ";
        let chunks = chunk_utf16(text);
        assert!(chunks.len() > 1);
        assert!(chunks.iter().all(|c| c.len() <= MAX_UNITS_PER_EVENT));
        let joined: Vec<u16> = chunks.concat();
        assert_eq!(String::from_utf16(&joined).unwrap(), text);
    }

    #[test]
    fn surrogate_pairs_are_never_split() {
        let text = "😀".repeat(9); // 18 units
        let chunks = chunk_utf16(&text);
        for chunk in &chunks {
            assert!(String::from_utf16(chunk).is_ok(), "chunk split a pair: {chunk:?}");
        }
        assert_eq!(String::from_utf16(&chunks.concat()).unwrap(), text);
    }

    #[test]
    fn combining_marks_stay_with_their_base() {
        // 15 plain units, then `e` + U+0302 + U+0323: the cluster must move as one.
        let text = format!("{}e\u{0302}\u{0323}", "x".repeat(15));
        let chunks = chunk_utf16(&text);
        let last = chunks.last().unwrap();
        assert_eq!(String::from_utf16(last).unwrap(), "e\u{0302}\u{0323}");
        assert_eq!(String::from_utf16(&chunks.concat()).unwrap(), text);
    }
}
