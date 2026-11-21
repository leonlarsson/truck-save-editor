//! Encoded tokens: short lowercase names packed into a `u64`, base 38, used by binary SII for
//! ids and token values.

const CHARS: &[u8; 37] = b"0123456789abcdefghijklmnopqrstuvwxyz_";

/// Unpacks a token (`0` is the empty token).
pub fn decode(mut value: u64) -> String {
    let mut out = String::with_capacity(12);
    while value > 0 {
        let index = (value % 38) as usize;
        value /= 38;
        // Digit 0 never occurs in a valid token; it's written as '?' rather than failing.
        out.push(index.checked_sub(1).map_or('?', |i| char::from(CHARS[i])));
    }
    out
}

/// Packs a token, or `None` if it has characters tokens can't hold or is too long (12 at most).
pub fn encode(token: &str) -> Option<u64> {
    if token.len() > 12 {
        return None;
    }
    let mut value: u64 = 0;
    for byte in token.bytes().rev() {
        let index = CHARS.iter().position(|&c| c == byte)? as u64 + 1;
        value = value.checked_mul(38)?.checked_add(index)?;
    }
    Some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips() {
        for token in [
            "",
            "a",
            "economy",
            "player",
            "long_dist",
            "abcdefghijkl",
            "x9_",
        ] {
            let packed = encode(token).expect("encodes");
            assert_eq!(decode(packed), token);
        }
        assert_eq!(encode("a"), Some(11));
    }

    #[test]
    fn rejects_what_isnt_a_token() {
        assert_eq!(encode("Economy"), None);
        assert_eq!(encode("a b"), None);
        assert_eq!(encode("abcdefghijklm"), None);
    }
}
