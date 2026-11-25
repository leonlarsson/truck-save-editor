//! New plates for trucks and trailers the editor adds (bought or copied). The new vehicle's fields
//! come from another (see `purchases.rs`, `copies.rs`), plate included; two of the company's
//! vehicles with one plate would be confusing. So the plate keeps its format (the region after
//! `|`, any formatting tags like `<offset hshift=…>`, where letters and digits go) and gets new
//! letters and digits, from a number that differs for each new vehicle (its id's).

/// Letters for plates: A to Z without I, O and Q, which read as 1 and 0.
const LETTERS: &[u8] = b"ABCDEFGHJKLMNPRSTUVWXYZ";

/// Gives a truck or trailer a fresh plate (see `fresh_plate`), seeded by its id, if it has one.
pub(crate) fn renew_plate(unit: &mut sii::Unit) {
    let Some(seed) = sii::nameless_value(&unit.id) else {
        return;
    };
    for field in &mut unit.fields {
        if field.name == "license_plate"
            && let Some(plate) = field.value.as_single()
        {
            field.value = sii::Value::single(fresh_plate(plate, seed));
        }
    }
}

/// A plate as the save writes it (`"FP02542|california"`, quoted), with its letters and digits
/// swapped for others picked by `seed`. Text inside `<…>` tags and the region are kept.
pub(crate) fn fresh_plate(value: &str, seed: u64) -> String {
    let text = sii::value::parse_string(value);
    let (plate, region) = match text.split_once('|') {
        Some((plate, region)) => (plate, Some(region)),
        None => (text.as_ref(), None),
    };
    // A small, plain pseudo-random sequence (an LCG): the same seed gives the same plate, so the
    // preview and the save agree.
    let mut state = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
    let mut next = |range: u64| {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (state >> 33) % range
    };
    let mut in_tag = false;
    let mut out = String::with_capacity(plate.len());
    for character in plate.chars() {
        match character {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ => {}
        }
        let new = if in_tag {
            character
        } else if character.is_ascii_digit() {
            char::from(b'0' + next(10) as u8)
        } else if character.is_ascii_uppercase() {
            char::from(LETTERS[next(LETTERS.len() as u64) as usize])
        } else if character.is_ascii_lowercase() {
            char::from(LETTERS[next(LETTERS.len() as u64) as usize].to_ascii_lowercase())
        } else {
            character
        };
        out.push(new);
    }
    let text = match region {
        Some(region) => format!("{out}|{region}"),
        None => out,
    };
    sii::value::format_string(&text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_the_format_and_region_with_new_characters() {
        let plate = fresh_plate("\"FP02542|california\"", 0x0246_5b82_0fe8);
        let text = sii::value::parse_string(&plate).to_string();
        let (new, region) = text.split_once('|').expect("a region");
        assert_eq!(region, "california");
        assert_eq!(new.len(), 7);
        assert!(new[..2].chars().all(|c| c.is_ascii_uppercase()));
        assert!(new[2..].chars().all(|c| c.is_ascii_digit()));
        assert_ne!(new, "FP02542");
        // The same seed, the same plate; another, another.
        assert_eq!(
            fresh_plate("\"FP02542|california\"", 0x0246_5b82_0fe8),
            plate
        );
        assert_ne!(
            fresh_plate("\"FP02542|california\"", 0x0246_5b82_0ff0),
            plate
        );
        // Tags are kept as they are.
        let tagged = fresh_plate("\"<offset hshift=-5>AB 123|sweden\"", 1);
        assert!(sii::value::parse_string(&tagged).starts_with("<offset hshift=-5>"));
    }
}
