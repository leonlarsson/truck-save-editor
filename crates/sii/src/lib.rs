//! Reads and writes SCS Software's SII files (Euro Truck Simulator 2, American Truck Simulator):
//! the `ScsC` encrypted wrapper, binary SII and text SII.
//!
//! Bytes in, bytes out: no disk access and no UI, so this builds for WebAssembly too. The format is
//! described in `docs/sii-format.md`.
//!
//! ```no_run
//! # fn main() -> sii::Result<()> {
//! let bytes = std::fs::read("game.sii").unwrap_or_default();
//! let document = sii::read(&bytes)?;
//! let economy = document.first_of_kind("economy");
//! let text = sii::to_text(&document);
//! # Ok(()) }
//! ```

mod bsii;
mod document;
mod error;
mod scsc;
mod text;
pub mod token;
pub mod value;

pub use document::{Document, Field, Unit, UnitMut, Value};
pub use error::{Error, Result};

/// The three forms a SII file comes in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    /// `ScsC`: encrypted and compressed, wrapping one of the other two.
    Encrypted,
    /// `BSII`: binary.
    Binary,
    /// `SiiNunit`: text.
    Text,
}

/// Which form a file is in, from its first bytes.
pub fn detect(bytes: &[u8]) -> Option<Format> {
    let bytes = bytes.strip_prefix("\u{feff}".as_bytes()).unwrap_or(bytes);
    if bytes.starts_with(b"ScsC") {
        Some(Format::Encrypted)
    } else if bytes.starts_with(b"BSII") {
        Some(Format::Binary)
    } else if bytes.starts_with(b"SiiN") {
        Some(Format::Text)
    } else {
        None
    }
}

/// Reads a SII file in any of its forms.
pub fn read(bytes: &[u8]) -> Result<Document> {
    match detect(bytes) {
        Some(Format::Encrypted) => {
            let inner = scsc::decrypt(bytes)?;
            match detect(&inner) {
                Some(Format::Encrypted) | None => Err(Error::UnknownFormat {
                    start: inner.iter().take(4).copied().collect(),
                }),
                Some(_) => read(&inner),
            }
        }
        Some(Format::Binary) => bsii::decode(bytes),
        Some(Format::Text) => {
            let text = std::str::from_utf8(bytes).map_err(|error| Error::NotUtf8 {
                offset: error.valid_up_to(),
            })?;
            text::parse(text)
        }
        None => Err(Error::UnknownFormat {
            start: bytes.iter().take(4).copied().collect(),
        }),
    }
}

/// Unwraps an `ScsC` file into what it holds. Other forms come back unchanged.
pub fn decrypt(bytes: &[u8]) -> Result<Vec<u8>> {
    match detect(bytes) {
        Some(Format::Encrypted) => scsc::decrypt(bytes),
        _ => Ok(bytes.to_vec()),
    }
}

/// Parses text SII.
pub fn parse_text(text: &str) -> Result<Document> {
    text::parse(text)
}

/// Writes a document as text SII, the way the game writes it.
pub fn to_text(document: &Document) -> String {
    text::write(document)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_every_form() {
        let text =
            "SiiNunit\r\n{\r\neconomy : economy {\r\n experience_points: 5\r\n}\r\n\r\n}\r\n";
        let from_text = read(text.as_bytes()).expect("text");
        let from_encrypted = read(&scsc::tests::encrypt(text.as_bytes())).expect("encrypted");
        assert_eq!(from_text, from_encrypted);
        assert_eq!(to_text(&from_text), text);

        let mut binary = bsii::tests::Builder::new(2);
        binary
            .u32(0)
            .u8(1)
            .u32(7)
            .string("economy")
            .u32(0x27)
            .string("experience_points")
            .u32(0);
        binary.u32(7).token_id("economy").u32(5);
        binary.u32(0).u8(0);
        assert_eq!(read(&binary.0).expect("binary"), from_text);
        assert_eq!(
            read(&scsc::tests::encrypt(&binary.0)).expect("encrypted binary"),
            from_text
        );
    }

    #[test]
    fn rejects_other_files() {
        assert!(matches!(
            read(b"PNG\x89 nope"),
            Err(Error::UnknownFormat { .. })
        ));
        assert!(read(b"").is_err());
    }
}
