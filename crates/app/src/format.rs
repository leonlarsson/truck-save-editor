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

/// `1_000_000` → `"1M"`: the short form people can also type into an amount field.
pub fn compact(value: i64) -> String {
    let units: [(i64, &str); 3] = [(1_000_000_000, "B"), (1_000_000, "M"), (1_000, "k")];
    for (size, unit) in units {
        if value != 0 && value % size == 0 {
            return format!("{}{unit}", value / size);
        }
    }
    thousands(value)
}

/// Reads what someone typed into an amount field, relative to the `current` value.
///
/// - `2480350`, `2,480,350` → that amount
/// - `2.5m`, `10k`, `1b` → with a multiplier (k = thousand, m = million, b = billion; any case)
/// - `+1m`, `-500k` → added to or taken from `current`
///
/// `None` when it isn't an amount, or doesn't fit.
pub fn parse_amount(text: &str, current: i64) -> Option<i64> {
    let cleaned: String = text
        .chars()
        .filter(|c| !matches!(c, ',' | ' ' | '_' | '€' | '$'))
        .collect::<String>()
        .to_lowercase()
        .replace('−', "-");

    let (sign, rest) = match cleaned.chars().next()? {
        '+' => (Some(1), &cleaned[1..]),
        '-' => (Some(-1), &cleaned[1..]),
        _ => (None, cleaned.as_str()),
    };

    let (number, multiplier) = match rest.chars().last()? {
        'k' => (&rest[..rest.len() - 1], 1_000.0),
        'm' => (&rest[..rest.len() - 1], 1_000_000.0),
        'b' => (&rest[..rest.len() - 1], 1_000_000_000.0),
        _ => (rest, 1.0),
    };
    if number.is_empty() || !number.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return None;
    }
    let amount = (number.parse::<f64>().ok()? * multiplier).round();
    // Beyond about 9e15, f64 can no longer hold every whole number exactly.
    if amount > 9.0e15 {
        return None;
    }
    let amount = amount as i64;

    match sign {
        Some(1) => current.checked_add(amount),
        Some(_) => current.checked_sub(amount),
        None => Some(amount),
    }
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
    fn shortens_round_amounts() {
        assert_eq!(compact(100_000), "100k");
        assert_eq!(compact(1_000_000), "1M");
        assert_eq!(compact(10_000_000), "10M");
        assert_eq!(compact(2_000_000_000), "2B");
        assert_eq!(compact(1_234), "1,234");
    }

    #[test]
    fn reads_plain_amounts() {
        assert_eq!(parse_amount("2,480,350", 0), Some(2_480_350));
        assert_eq!(parse_amount(" 12 000 ", 0), Some(12_000));
        assert_eq!(parse_amount("€ 5,000", 0), Some(5_000));
        assert_eq!(parse_amount("0", 99), Some(0));
    }

    #[test]
    fn reads_multipliers() {
        assert_eq!(parse_amount("2.5m", 0), Some(2_500_000));
        assert_eq!(parse_amount("10K", 0), Some(10_000));
        assert_eq!(parse_amount("1b", 0), Some(1_000_000_000));
        assert_eq!(parse_amount("0.5k", 0), Some(500));
    }

    #[test]
    fn adds_and_takes_away() {
        assert_eq!(parse_amount("+1m", 500), Some(1_000_500));
        assert_eq!(parse_amount("-500k", 2_000_000), Some(1_500_000));
        assert_eq!(parse_amount("−200", 1_000), Some(800));
        assert_eq!(parse_amount("-5", 0), Some(-5));
    }

    #[test]
    fn rejects_what_isnt_an_amount() {
        assert_eq!(parse_amount("", 0), None);
        assert_eq!(parse_amount("+", 0), None);
        assert_eq!(parse_amount("m", 0), None);
        assert_eq!(parse_amount("12a", 0), None);
        assert_eq!(parse_amount("1.2.3", 0), None);
        assert_eq!(parse_amount("99999999999b", 0), None);
    }
}
