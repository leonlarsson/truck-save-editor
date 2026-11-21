//! Reading and writing the typed values inside a field's text form.
//!
//! - integers: `2480350`, `-5`
//! - floats: `0.5`, `16`, or the raw bits as `&3e98ae14` (exact; what the game writes)
//! - booleans: `true`, `false`
//! - strings: `"Before the long haul"`, or bare when they're a plain word (`Rotterdam`)
//! - ids and tokens: `economy`, `_nameless.1ec.ab8c.2808`, `null`

use std::borrow::Cow;

pub fn parse_i64(text: &str) -> Option<i64> {
    text.trim().parse().ok()
}

pub fn parse_u64(text: &str) -> Option<u64> {
    text.trim().parse().ok()
}

/// A float written as a decimal or as `&` and its bits in hex.
pub fn parse_f32(text: &str) -> Option<f32> {
    let text = text.trim();
    match text.strip_prefix('&') {
        Some(bits) => u32::from_str_radix(bits, 16).ok().map(f32::from_bits),
        None => text.parse().ok(),
    }
}

pub fn parse_bool(text: &str) -> Option<bool> {
    match text.trim() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

/// A string's contents: quotes removed and escapes (`\"`, `\\`, `\n`, `\t`, `\xNN`) resolved.
pub fn parse_string(text: &str) -> Cow<'_, str> {
    let text = text.trim();
    let Some(inner) = text
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
    else {
        return Cow::Borrowed(text);
    };
    if !inner.contains('\\') {
        return Cow::Borrowed(inner);
    }

    let mut bytes = Vec::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            let mut buffer = [0u8; 4];
            bytes.extend_from_slice(c.encode_utf8(&mut buffer).as_bytes());
            continue;
        }
        match chars.next() {
            Some('n') => bytes.push(b'\n'),
            Some('t') => bytes.push(b'\t'),
            Some('x') => {
                let hex: String = chars.by_ref().take(2).collect();
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    bytes.push(byte);
                }
            }
            Some(other) => {
                let mut buffer = [0u8; 4];
                bytes.extend_from_slice(other.encode_utf8(&mut buffer).as_bytes());
            }
            None => bytes.push(b'\\'),
        }
    }
    Cow::Owned(String::from_utf8_lossy(&bytes).into_owned())
}

/// A float as the game writes it: whole numbers plainly (`16`), anything else as its exact bits
/// (`&3e98ae14`), so nothing is ever rounded.
pub fn format_f32(value: f32) -> String {
    let mut out = String::new();
    write_f32(&mut out, value);
    out
}

pub(crate) fn write_f32(out: &mut String, value: f32) {
    use std::fmt::Write as _;
    let is_plain_whole_number = value.is_finite()
        && value.fract() == 0.0
        && value.abs() < 1.0e7
        && !(value == 0.0 && value.is_sign_negative());
    if is_plain_whole_number {
        let _ = write!(out, "{}", value as i64);
    } else {
        let _ = write!(out, "&{:08x}", value.to_bits());
    }
}

/// A string as a quoted SII string, with quotes and backslashes escaped.
pub fn format_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    write_quoted(&mut out, value);
    out
}

pub(crate) fn write_quoted(out: &mut String, value: &str) {
    out.push('"');
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out.push('"');
}

/// Whether a string can be written bare, without quotes: a plain word of token characters.
pub fn is_plain_token(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_numbers() {
        assert_eq!(parse_i64("2480350"), Some(2_480_350));
        assert_eq!(parse_i64("-5"), Some(-5));
        assert_eq!(parse_i64("x"), None);
        assert_eq!(parse_f32("0.5"), Some(0.5));
        assert_eq!(parse_f32("&3f800000"), Some(1.0));
        assert_eq!(parse_bool("true"), Some(true));
    }

    #[test]
    fn floats_round_trip_exactly() {
        for value in [0.0, 1.0, -3.0, 0.3, 1.0e-7, 123_456.79, f32::MAX, -0.0] {
            let text = format_f32(value);
            let back = parse_f32(&text).expect("parses");
            assert_eq!(back.to_bits(), value.to_bits(), "{value} as {text}");
        }
        assert_eq!(format_f32(16.0), "16");
        assert_eq!(format_f32(0.3), "&3e99999a");
    }

    #[test]
    fn strings_round_trip() {
        for value in [
            "Rotterdam",
            "Before the long haul",
            "",
            "say \"hi\"",
            "back\\slash",
            "Kraków",
        ] {
            assert_eq!(parse_string(&format_string(value)), value);
        }
        assert_eq!(parse_string("Rotterdam"), "Rotterdam");
        assert_eq!(parse_string("\"a\\x41b\""), "aAb");
    }

    #[test]
    fn knows_plain_tokens() {
        assert!(is_plain_token("economy"));
        assert!(is_plain_token("long_dist_2"));
        assert!(!is_plain_token("Rotterdam"));
        assert!(!is_plain_token("a b"));
        assert!(!is_plain_token(""));
    }
}
