//! How numbers are shown and read back. One place, so every screen writes them the same way, in
//! the way the app's language writes them (`2,480,350` in English, `2 480 350` in Swedish).

use std::sync::atomic::{AtomicU8, Ordering};

/// How a language writes numbers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumberStyle {
    /// `2,480,350` and `2.5`.
    Comma,
    /// `2 480 350` (a no-break space, so a number never wraps) and `2,5`.
    Space,
}

impl NumberStyle {
    fn grouping(self) -> char {
        match self {
            NumberStyle::Comma => ',',
            NumberStyle::Space => '\u{a0}',
        }
    }
}

/// The style in use, for the whole app (set with the language; see `i18n::set_language`). An
/// atomic rather than passing it everywhere: numbers are formatted in many places that have no
/// app context at hand, and it changes only when the language does.
static STYLE: AtomicU8 = AtomicU8::new(0);

pub fn set_style(style: NumberStyle) {
    STYLE.store(style as u8, Ordering::Relaxed);
}

fn style() -> NumberStyle {
    match STYLE.load(Ordering::Relaxed) {
        1 => NumberStyle::Space,
        _ => NumberStyle::Comma,
    }
}

/// `2480350` → `"2,480,350"` (or `"2 480 350"`, in the app's language).
pub fn thousands(value: i64) -> String {
    thousands_with(value, style())
}

fn thousands_with(value: i64, style: NumberStyle) -> String {
    let digits = value.unsigned_abs().to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3 + 1);
    if value < 0 {
        out.push('-');
    }
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            out.push(style.grouping());
        }
        out.push(digit);
    }
    out
}

/// `20.45` with 1 place → `"20.5"` (or `"20,5"`), whole part grouped like `thousands`; a whole
/// result drops its decimals (`20.0` → `"20"`).
pub fn decimal(value: f64, places: usize) -> String {
    decimal_with(value, places, style())
}

fn decimal_with(value: f64, places: usize, style: NumberStyle) -> String {
    let scale = 10f64.powi(places as i32);
    let rounded = (value * scale).round() / scale;
    let whole = rounded.trunc();
    let mut out = thousands_with(whole as i64, style);
    if rounded < 0.0 && whole == 0.0 {
        out.insert(0, '-');
    }
    let fraction = ((rounded - whole).abs() * scale).round() as i64;
    if fraction > 0 {
        out.push(match style {
            NumberStyle::Comma => '.',
            NumberStyle::Space => ',',
        });
        out.push_str(&format!("{fraction:0places$}"));
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
    parse_amount_with(text, current, style())
}

/// `parse_amount` in a given style: in `Space` style a comma is the decimal mark (`2,5m`), and
/// spaces group thousands; in `Comma` style commas group thousands. A point is a decimal mark in
/// both, since that's what people type for `2.5m` whatever their language.
fn parse_amount_with(text: &str, current: i64, style: NumberStyle) -> Option<i64> {
    let cleaned: String = text
        .chars()
        .filter(|c| {
            let grouping = match style {
                NumberStyle::Comma => matches!(c, ','),
                NumberStyle::Space => false,
            };
            !grouping && !c.is_whitespace() && !matches!(c, '_' | '€' | '$')
        })
        .map(|c| if c == ',' { '.' } else { c })
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
    fn writes_and_reads_the_swedish_way() {
        assert_eq!(
            thousands_with(2_480_350, NumberStyle::Space),
            "2\u{a0}480\u{a0}350"
        );
        assert_eq!(
            parse_amount_with("2 480 350", 0, NumberStyle::Space),
            Some(2_480_350)
        );
        assert_eq!(
            parse_amount_with("2\u{a0}480", 0, NumberStyle::Space),
            Some(2_480)
        );
        assert_eq!(
            parse_amount_with("2,5m", 0, NumberStyle::Space),
            Some(2_500_000)
        );
        assert_eq!(
            parse_amount_with("2.5m", 0, NumberStyle::Space),
            Some(2_500_000)
        );
    }

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

    #[test]
    fn writes_decimals_in_each_style() {
        assert_eq!(decimal_with(20.45, 1, NumberStyle::Comma), "20.5");
        assert_eq!(decimal_with(20.0, 1, NumberStyle::Comma), "20");
        assert_eq!(decimal_with(1234.5, 1, NumberStyle::Space), "1\u{a0}234,5");
        assert_eq!(decimal_with(0.05, 2, NumberStyle::Comma), "0.05");
    }
}
