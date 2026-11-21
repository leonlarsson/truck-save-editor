//! Binary SII (`BSII`, versions 1–3), decoded into a `Document` in text form.
//!
//! ```text
//! "BSII" u32 version
//! blocks, each starting with u32 block_type:
//!   0 → structure definition: u8 valid (0 = end of file), u32 id, string name,
//!       then fields (u32 value_type, string name; 0x37 adds a lookup table) until value_type 0
//!   n → a unit of structure n: its id, then one value per field, in definition order
//! string = u32 length + UTF-8; all numbers little-endian
//! ```
//!
//! See `docs/sii-format.md` for the value types.

use std::collections::HashMap;
use std::fmt::Write as _;

use crate::document::{Document, Field, Unit, Value};
use crate::error::{Error, Result};
use crate::token;
use crate::value::{is_plain_token, write_f32, write_quoted};

pub fn decode(bytes: &[u8]) -> Result<Document> {
    let mut reader = Reader { bytes, offset: 0 };
    if reader.take(4)? != b"BSII" {
        return Err(Error::UnknownFormat {
            start: bytes.iter().take(4).copied().collect(),
        });
    }
    let version = reader.u32()?;
    if !(1..=3).contains(&version) {
        return Err(Error::UnsupportedVersion(version));
    }

    let mut structures: HashMap<u32, Structure> = HashMap::new();
    let mut document = Document::new();

    loop {
        let block_offset = reader.offset;
        let block_type = reader.u32()?;
        if block_type == 0 {
            if reader.u8()? == 0 {
                break;
            }
            let id = reader.u32()?;
            let structure = read_structure(&mut reader)?;
            structures.insert(id, structure);
            continue;
        }

        let structure = structures.get(&block_type).ok_or(Error::UnknownStructure {
            id: block_type,
            offset: block_offset,
        })?;
        let mut unit = Unit::new(structure.name.clone(), reader.id()?);
        unit.fields.reserve(structure.fields.len());
        for field in &structure.fields {
            let value = read_value(&mut reader, field, &structure.name, version)?;
            unit.fields.push(Field {
                name: field.name.clone(),
                value,
            });
        }
        document.push(unit);
    }
    Ok(document)
}

struct Structure {
    name: String,
    fields: Vec<FieldDefinition>,
}

struct FieldDefinition {
    value_type: u32,
    name: String,
    /// For lookup values (0x37): index → text.
    lookup: HashMap<u32, String>,
}

fn read_structure(reader: &mut Reader) -> Result<Structure> {
    let name = reader.string()?;
    let mut fields = Vec::new();
    loop {
        let value_type = reader.u32()?;
        if value_type == 0 {
            break;
        }
        let field_name = reader.string()?;
        let mut lookup = HashMap::new();
        if value_type == 0x37 {
            for _ in 0..reader.u32()? {
                let index = reader.u32()?;
                lookup.insert(index, reader.string()?);
            }
        }
        fields.push(FieldDefinition {
            value_type,
            name: field_name,
            lookup,
        });
    }
    Ok(Structure { name, fields })
}

fn read_value(
    reader: &mut Reader,
    field: &FieldDefinition,
    unit: &str,
    version: u32,
) -> Result<Value> {
    let value_type = field.value_type;
    let unknown = |offset| Error::UnknownValueType {
        value_type,
        unit: unit.to_string(),
        field: field.name.clone(),
        offset,
    };

    if value_type == 0x37 {
        let index = reader.u32()?;
        let text = field.lookup.get(&index).map(String::as_str).unwrap_or("");
        return Ok(Value::Single(if is_plain_token(text) {
            text.to_string()
        } else {
            let mut out = String::new();
            write_quoted(&mut out, text);
            out
        }));
    }

    // Each single type is odd and its array type the next even number, except where noted.
    let (single_type, is_array) = match value_type {
        0x2F | 0x35 => (value_type, false),
        0x36 => (0x35, true),
        0x39 | 0x3B | 0x3D => (value_type, false),
        0x3A | 0x3C | 0x3E => (value_type - 1, true),
        t if t % 2 == 0 => (t - 1, true),
        t => (t, false),
    };

    if !is_array {
        let mut out = String::new();
        read_single(reader, single_type, version, &mut out)?
            .ok_or_else(|| unknown(reader.offset))?;
        return Ok(Value::Single(out));
    }

    let count = reader.u32()? as usize;
    // A count can't be larger than the bytes left; checking avoids a huge allocation on a damaged file.
    if count > reader.remaining() {
        return Err(Error::Truncated {
            offset: reader.offset,
        });
    }
    let mut items = Vec::with_capacity(count);
    for _ in 0..count {
        let mut out = String::new();
        read_single(reader, single_type, version, &mut out)?
            .ok_or_else(|| unknown(reader.offset))?;
        items.push(out);
    }
    Ok(Value::Array(items))
}

/// Reads one value of a single type as text into `out`. `Ok(None)` for a type that isn't known.
fn read_single(
    reader: &mut Reader,
    single_type: u32,
    version: u32,
    out: &mut String,
) -> Result<Option<()>> {
    match single_type {
        0x01 => write_quoted(out, &reader.string()?),
        0x03 => {
            let text = token::decode(reader.u64()?);
            if text.is_empty() {
                out.push_str("\"\"");
            } else {
                out.push_str(&text);
            }
        }
        0x05 => write_f32(out, reader.f32()?),
        0x07 => write_floats(out, reader, 2)?,
        0x09 => write_floats(out, reader, 3)?,
        0x11 => {
            let (x, y, z) = (reader.i32()?, reader.i32()?, reader.i32()?);
            let _ = write!(out, "({x}, {y}, {z})");
        }
        0x17 => write_floats(out, reader, 4)?,
        0x19 => write_placement(out, reader, version)?,
        0x25 => {
            let _ = write!(out, "{}", reader.i32()?);
        }
        0x27 | 0x2F => {
            let _ = write!(out, "{}", reader.u32()?);
        }
        0x29 => {
            let _ = write!(out, "{}", i16::from_le_bytes(reader.array()?));
        }
        0x2B => {
            let _ = write!(out, "{}", u16::from_le_bytes(reader.array()?));
        }
        0x31 => {
            let _ = write!(out, "{}", i64::from_le_bytes(reader.array()?));
        }
        0x33 => {
            let _ = write!(out, "{}", reader.u64()?);
        }
        0x35 => out.push_str(if reader.u8()? != 0 { "true" } else { "false" }),
        0x39 | 0x3B | 0x3D => out.push_str(&reader.id()?),
        _ => return Ok(None),
    }
    Ok(Some(()))
}

fn write_floats(out: &mut String, reader: &mut Reader, count: usize) -> Result<()> {
    out.push('(');
    for index in 0..count {
        if index > 0 {
            out.push_str(", ");
        }
        write_f32(out, reader.f32()?);
    }
    out.push(')');
    Ok(())
}

/// A position and rotation: `(x, y, z) (w, x, y, z)`.
///
/// Version 1 stores 7 floats. Version 2 and later store 8: the fourth holds the position's coarse
/// part (the "bias"), which is added to x and z and isn't written in text.
fn write_placement(out: &mut String, reader: &mut Reader, version: u32) -> Result<()> {
    let mut position = [reader.f32()?, reader.f32()?, reader.f32()?];
    if version >= 2 {
        let bias = reader.f32()? as i32;
        position[0] += (((bias & 0xFFF) - 2048) << 9) as f32;
        position[2] += ((((bias >> 12) & 0xFFF) - 2048) << 9) as f32;
    }
    let rotation = [reader.f32()?, reader.f32()?, reader.f32()?, reader.f32()?];
    out.push('(');
    for (index, value) in position.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        write_f32(out, *value);
    }
    out.push_str(") (");
    for (index, value) in rotation.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        write_f32(out, *value);
    }
    out.push(')');
    Ok(())
}

struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl Reader<'_> {
    fn remaining(&self) -> usize {
        self.bytes.len() - self.offset
    }

    fn take(&mut self, count: usize) -> Result<&[u8]> {
        let end = self
            .offset
            .checked_add(count)
            .filter(|&end| end <= self.bytes.len())
            .ok_or(Error::Truncated {
                offset: self.offset,
            })?;
        let slice = &self.bytes[self.offset..end];
        self.offset = end;
        Ok(slice)
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N]> {
        let offset = self.offset;
        self.take(N)?
            .try_into()
            .map_err(|_| Error::Truncated { offset })
    }

    fn u8(&mut self) -> Result<u8> {
        Ok(self.array::<1>()?[0])
    }

    fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.array()?))
    }

    fn i32(&mut self) -> Result<i32> {
        Ok(i32::from_le_bytes(self.array()?))
    }

    fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(self.array()?))
    }

    fn f32(&mut self) -> Result<f32> {
        Ok(f32::from_le_bytes(self.array()?))
    }

    fn string(&mut self) -> Result<String> {
        let length = self.u32()? as usize;
        let offset = self.offset;
        let bytes = self.take(length)?;
        String::from_utf8(bytes.to_vec()).map_err(|error| Error::NotUtf8 {
            offset: offset + error.utf8_error().valid_up_to(),
        })
    }

    /// An id: `null`, `_nameless.1ec.ab8c.2808`, or tokens joined with dots (`company.volatile.x`).
    fn id(&mut self) -> Result<String> {
        let parts = self.u8()?;
        match parts {
            0 => Ok("null".to_string()),
            0xFF => {
                let value = self.u64()?;
                let groups = [48, 32, 16, 0].map(|shift| (value >> shift) & 0xFFFF);
                let first = groups.iter().position(|&g| g != 0).unwrap_or(3);
                let mut out = String::from("_nameless");
                for group in &groups[first..] {
                    let _ = write!(out, ".{group:x}");
                }
                Ok(out)
            }
            _ => {
                let mut out = String::new();
                for index in 0..parts {
                    if index > 0 {
                        out.push('.');
                    }
                    out.push_str(&token::decode(self.u64()?));
                }
                Ok(out)
            }
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// Builds binary SII for tests.
    #[derive(Default)]
    pub(crate) struct Builder(pub(crate) Vec<u8>);

    impl Builder {
        pub(crate) fn new(version: u32) -> Self {
            let mut builder = Self(b"BSII".to_vec());
            builder.u32(version);
            builder
        }
        pub(crate) fn u8(&mut self, v: u8) -> &mut Self {
            self.0.push(v);
            self
        }
        pub(crate) fn u32(&mut self, v: u32) -> &mut Self {
            self.0.extend_from_slice(&v.to_le_bytes());
            self
        }
        pub(crate) fn u64(&mut self, v: u64) -> &mut Self {
            self.0.extend_from_slice(&v.to_le_bytes());
            self
        }
        pub(crate) fn f32(&mut self, v: f32) -> &mut Self {
            self.0.extend_from_slice(&v.to_le_bytes());
            self
        }
        pub(crate) fn string(&mut self, s: &str) -> &mut Self {
            self.u32(s.len() as u32);
            self.0.extend_from_slice(s.as_bytes());
            self
        }
        pub(crate) fn token_id(&mut self, s: &str) -> &mut Self {
            self.u8(1).u64(token::encode(s).expect("token"))
        }
    }

    fn sample() -> Vec<u8> {
        let mut b = Builder::new(3);
        // structure 1: "bank" with several field types
        b.u32(0).u8(1).u32(1).string("bank");
        b.u32(0x25).string("money_account");
        b.u32(0x35).string("overdraft");
        b.u32(0x01).string("name");
        b.u32(0x05).string("rate");
        b.u32(0x3A).string("loans");
        b.u32(0x26).string("history");
        b.u32(0x37)
            .string("status")
            .u32(2)
            .u32(0)
            .string("ok")
            .u32(1)
            .string("In debt");
        b.u32(0x09).string("position");
        b.u32(0);
        // a unit of it
        b.u32(1).u8(0xFF).u64(0x0000_01ec_ab8c_2808);
        b.u32(-5i32 as u32);
        b.u8(1);
        b.string("My \"best\" save");
        b.f32(0.3);
        b.u32(2).token_id("loan_a").u8(0);
        b.u32(0);
        b.u32(1);
        b.f32(1.0).f32(2.5).f32(-0.0);
        // end
        b.u32(0).u8(0);
        b.0
    }

    #[test]
    fn decodes_every_kind_of_value() {
        let document = decode(&sample()).expect("decodes");
        let bank = document.unit("_nameless.1ec.ab8c.2808").expect("bank");
        assert_eq!(bank.kind, "bank");
        assert_eq!(bank.single("money_account"), Some("-5"));
        assert_eq!(bank.single("overdraft"), Some("true"));
        assert_eq!(bank.single("name"), Some("\"My \\\"best\\\" save\""));
        assert_eq!(bank.single("rate"), Some("&3e99999a"));
        assert_eq!(bank.items("loans"), ["loan_a", "null"]);
        assert_eq!(bank.get("history"), Some(&Value::Array(vec![])));
        assert_eq!(bank.single("status"), Some("\"In debt\""));
        assert_eq!(bank.single("position"), Some("(1, &40200000, &80000000)"));
    }

    #[test]
    fn reports_damage_instead_of_crashing() {
        let bytes = sample();
        for length in [0, 3, 8, 20, 60, bytes.len() - 6] {
            assert!(decode(&bytes[..length]).is_err(), "cut at {length}");
        }
        let mut wrong = sample();
        wrong[4] = 9;
        assert!(matches!(decode(&wrong), Err(Error::UnsupportedVersion(9))));
    }

    fn placement(version: u32, values: &[f32]) -> String {
        let mut b = Builder::new(version);
        b.u32(0)
            .u8(1)
            .u32(1)
            .string("thing")
            .u32(0x19)
            .string("placement")
            .u32(0);
        b.u32(1).token_id("thing");
        for value in values {
            b.f32(*value);
        }
        b.u32(0).u8(0);
        let document = decode(&b.0).expect("decodes");
        let thing = document.unit("thing").expect("thing");
        thing.single("placement").expect("placement").to_string()
    }

    #[test]
    fn adds_the_placement_bias_from_version_2() {
        // Bias: x part 2049 (+512), z part 2047 (−512).
        let bias = ((2047 << 12) | 2049) as f32;
        let values = [10.0, 1.0, 20.0, bias, 1.0, 0.0, 0.0, 0.0];
        assert_eq!(placement(3, &values), "(522, 1, -492) (1, 0, 0, 0)");
        assert_eq!(placement(2, &values), "(522, 1, -492) (1, 0, 0, 0)");
        // The neutral bias (2048 in both parts) moves nothing.
        let neutral = ((2048 << 12) | 2048) as f32;
        let values = [10.0, 1.0, 20.0, neutral, 1.0, 0.0, 0.0, 0.0];
        assert_eq!(placement(2, &values), "(10, 1, 20) (1, 0, 0, 0)");
    }

    #[test]
    fn version_1_placements_have_seven_floats() {
        let values = [10.0, 1.0, 20.0, 1.0, 0.0, 0.0, 0.0];
        assert_eq!(placement(1, &values), "(10, 1, 20) (1, 0, 0, 0)");
    }
}
