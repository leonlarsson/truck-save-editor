//! How numbers are shown and read back. One place, so every screen writes them the same way.

/// `2480350` → `"2,480,350"`.
pub fn thousands(value: i64) -> String {
    let digits = value.unsigned_abs().to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3 + 1);
    if value < 0 {
        out.push('-');
    }
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            out.push(',');
        }
        out.push(digit);
    }
    out
}

/// `"2,480,350"` → `Some(2480350)`. Ignores separators and spaces; `None` for anything else.
pub fn parse_whole_number(text: &str) -> Option<i64> {
    let cleaned: String = text
        .chars()
        .filter(|c| !matches!(c, ',' | ' ' | '_'))
        .collect();
    cleaned.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_thousands() {
        assert_eq!(thousands(0), "0");
        assert_eq!(thousands(999), "999");
        assert_eq!(thousands(1_000), "1,000");
        assert_eq!(thousands(2_480_350), "2,480,350");
        assert_eq!(thousands(-1_234_567), "-1,234,567");
    }

    #[test]
    fn reads_numbers_back() {
        assert_eq!(parse_whole_number("2,480,350"), Some(2_480_350));
        assert_eq!(parse_whole_number(" 12 000 "), Some(12_000));
        assert_eq!(parse_whole_number("12a"), None);
        assert_eq!(parse_whole_number(""), None);
    }
}
