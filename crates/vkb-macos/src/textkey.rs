//! Which virtual keycode carries injected text.
//!
//! Text is posted as a key event with a Unicode string. If that keycode is a
//! physical key the user is still holding (typically 'a' = 0), macOS drops the
//! event. So pick a keycode from a small pool that is not currently held.

/// Letter keys on QWERTY/AZERTY/QWERTZ/Dvorak, never dead keys without Option:
/// A, B, V, C. No punctuation (0x2A is dead on French, 0x2C reaches web pages as "/").
pub const POOL: [u16; 4] = [0x00, 0x0B, 0x09, 0x08];

/// First pool keycode that is not held; the first one when all are held.
pub fn choose(mut held: impl FnMut(u16) -> bool) -> u16 {
    POOL.iter().copied().find(|&k| !held(k)).unwrap_or(POOL[0])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pick(down: &[u16]) -> u16 {
        choose(|k| down.contains(&k))
    }

    #[test]
    fn nothing_held_uses_a() {
        assert_eq!(pick(&[]), 0x00);
    }

    #[test]
    fn a_held_uses_b() {
        assert_eq!(pick(&[0x00]), 0x0B);
    }

    #[test]
    fn a_and_b_held_uses_v() {
        assert_eq!(pick(&[0x00, 0x0B]), 0x09);
    }

    #[test]
    fn all_held_is_deterministic() {
        assert_eq!(pick(&[0x00, 0x0B, 0x09, 0x08]), 0x00);
    }
}
