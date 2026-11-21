//! Text SII: parsing it into a `Document`, and writing a `Document` as the game does.
//!
//! ```text
//! SiiNunit
//! {
//! kind : id {
//!  field: value
//!  array: 2
//!  array[0]: value
//!  array[1]: value
//! }
//!
//! }
//! ```
//!
//! Array items may also be written `array[]: value` (appending). Comments (`#`, `//`, `/* */`)
//! are skipped.

use crate::document::{Document, Field, Unit, Value};
use crate::error::{Error, Result};

/// Writes a document as text SII, with Windows line endings like the game.
pub fn write(document: &Document) -> String {
    // Most fields are short; this guess avoids most reallocations.
    let mut out = String::with_capacity(document.len() * 400);
    out.push_str("SiiNunit\r\n{\r\n");
    for unit in document.units() {
        out.push_str(&unit.kind);
        out.push_str(" : ");
        out.push_str(&unit.id);
        out.push_str(" {\r\n");
        for field in &unit.fields {
            match &field.value {
                Value::Single(text) => push_line(&mut out, &field.name, None, text),
                Value::Array(items) => {
                    push_line(&mut out, &field.name, None, &items.len().to_string());
                    for (index, item) in items.iter().enumerate() {
                        push_line(&mut out, &field.name, Some(index), item);
                    }
                }
            }
        }
        out.push_str("}\r\n\r\n");
    }
    out.push_str("}\r\n");
    out
}

fn push_line(out: &mut String, name: &str, index: Option<usize>, value: &str) {
    use std::fmt::Write as _;
    out.push(' ');
    out.push_str(name);
    if let Some(index) = index {
        let _ = write!(out, "[{index}]");
    }
    out.push_str(": ");
    out.push_str(value);
    out.push_str("\r\n");
}

/// Parses text SII.
pub fn parse(text: &str) -> Result<Document> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut document = Document::new();
    let mut state = State::ExpectMagic;
    let mut current: Option<Unit> = None;
    let mut in_block_comment = false;

    for (index, raw_line) in text.lines().enumerate() {
        let line_number = index + 1;
        let error = |message: &str| Error::Text {
            line: line_number,
            message: message.to_string(),
        };

        let mut line = raw_line.trim();
        if in_block_comment {
            match line.find("*/") {
                Some(end) => {
                    in_block_comment = false;
                    line = line[end + 2..].trim();
                }
                None => continue,
            }
        }
        if line.starts_with("/*") {
            match line.find("*/") {
                Some(end) => line = line[end + 2..].trim(),
                None => {
                    in_block_comment = true;
                    continue;
                }
            }
        }
        if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
            continue;
        }

        match state {
            State::ExpectMagic => {
                if line != "SiiNunit" {
                    return Err(error("expected `SiiNunit` at the start"));
                }
                state = State::ExpectOpen;
            }
            State::ExpectOpen => {
                if line != "{" {
                    return Err(error("expected `{` after `SiiNunit`"));
                }
                state = State::BetweenUnits;
            }
            State::BetweenUnits => {
                if line == "}" {
                    state = State::Done;
                    continue;
                }
                if line == "{" {
                    // The opening brace of a unit whose header was on the line before.
                    if current.is_some() {
                        state = State::InUnit;
                        continue;
                    }
                    return Err(error("unexpected `{`"));
                }
                let header = line.strip_suffix('{').map(str::trim_end);
                let (kind, id) = header
                    .unwrap_or(line)
                    .split_once(':')
                    .ok_or_else(|| error("expected a unit, like `kind : id {`"))?;
                current = Some(Unit::new(kind.trim(), id.trim()));
                if header.is_some() {
                    state = State::InUnit;
                }
            }
            State::InUnit => {
                if line == "}" {
                    if let Some(unit) = current.take() {
                        document.push(unit);
                    }
                    state = State::BetweenUnits;
                    continue;
                }
                let unit = current
                    .as_mut()
                    .ok_or_else(|| error("a field outside a unit"))?;
                let (name, value) = line
                    .split_once(':')
                    .ok_or_else(|| error("expected a field, like `name: value`"))?;
                add_field(unit, name.trim(), value.trim()).map_err(&error)?;
            }
            State::Done => return Err(error("text after the closing `}`")),
        }
    }

    match state {
        State::Done => Ok(document),
        _ => Err(Error::Text {
            line: text.lines().count(),
            message: "the file ends before its closing `}`".to_string(),
        }),
    }
}

enum State {
    ExpectMagic,
    ExpectOpen,
    BetweenUnits,
    InUnit,
    Done,
}

fn add_field(unit: &mut Unit, name: &str, value: &str) -> std::result::Result<(), &'static str> {
    let Some(open) = name.find('[') else {
        unit.fields.push(Field {
            name: name.to_string(),
            value: Value::Single(value.to_string()),
        });
        return Ok(());
    };
    let base = &name[..open];
    let index_text = name[open + 1..]
        .strip_suffix(']')
        .ok_or("an array item needs a closing `]`")?;

    // The array is the latest field with this name: either its count line (`name: 3`), turned
    // into an array now, or an array already started.
    let position = unit.fields.iter().rposition(|field| field.name == base);
    let items = match position.map(|p| &mut unit.fields[p].value) {
        Some(Value::Array(items)) => items,
        Some(single @ Value::Single(_)) => {
            let capacity = single.as_single().and_then(|c| c.parse().ok()).unwrap_or(0);
            *single = Value::Array(Vec::with_capacity(capacity));
            match single {
                Value::Array(items) => items,
                Value::Single(_) => return Err("couldn't start the array"),
            }
        }
        None => {
            unit.fields.push(Field {
                name: base.to_string(),
                value: Value::Array(Vec::new()),
            });
            match unit.fields.last_mut().map(|field| &mut field.value) {
                Some(Value::Array(items)) => items,
                _ => return Err("couldn't start the array"),
            }
        }
    };

    if index_text.is_empty() {
        items.push(value.to_string());
        return Ok(());
    }
    let index: usize = index_text
        .parse()
        .map_err(|_| "an array index must be a number")?;
    if index < items.len() {
        items[index] = value.to_string();
    } else {
        // Indices normally arrive in order; any gap is filled with empty strings.
        items.resize(index, String::new());
        items.push(value.to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "SiiNunit\r\n{\r\nbank : _nameless.1.2 {\r\n money_account: 2480350\r\n name: \"HAULER BACKUP\"\r\n loans: 2\r\n loans[0]: a\r\n loans[1]: b\r\n trucks: 0\r\n}\r\n\r\n}\r\n";

    #[test]
    fn parses_and_writes_back_the_same() {
        let document = parse(SAMPLE).expect("parses");
        let bank = document.unit("_nameless.1.2").expect("bank");
        assert_eq!(bank.single("money_account"), Some("2480350"));
        assert_eq!(bank.items("loans"), ["a", "b"]);
        assert_eq!(bank.single("trucks"), Some("0"));
        assert_eq!(write(&document), SAMPLE);
    }

    #[test]
    fn reads_the_other_ways_of_writing() {
        let text = "\u{feff}SiiNunit\n{\n# a comment\nthing : x.y\n{\n list[]: 1\n list[]: 2\n /* skip\n this */\n v: (1, 2, 3) (1, 0, 0, 0)\n}\n}\n";
        let document = parse(text).expect("parses");
        let thing = document.unit("x.y").expect("thing");
        assert_eq!(thing.items("list"), ["1", "2"]);
        assert_eq!(thing.single("v"), Some("(1, 2, 3) (1, 0, 0, 0)"));
    }

    #[test]
    fn says_which_line_is_wrong() {
        let error = parse("SiiNunit\n{\nunit : a {\n no colon here\n}\n}\n").expect_err("fails");
        assert!(matches!(error, Error::Text { line: 4, .. }), "{error}");
        assert!(parse("SiiNunit\n{\n").is_err());
        assert!(parse("hello").is_err());
    }
}
