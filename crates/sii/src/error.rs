//! What can go wrong reading a SII file. Every error says where, so a damaged or unexpected save
//! gives a message someone can act on (or report), never a crash.

/// Shorthand for results in this crate.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("this isn't a SII file (it starts with {start:?})")]
    UnknownFormat { start: Vec<u8> },

    #[error("the file ends early, at byte {offset}")]
    Truncated { offset: usize },

    #[error("couldn't decrypt the file: {0}")]
    Decrypt(&'static str),

    #[error("couldn't decompress the file: {0}")]
    Decompress(String),

    #[error("the file decompressed to {actual} bytes, but says it should be {expected}")]
    SizeMismatch { expected: usize, actual: usize },

    #[error("binary SII version {0} isn't supported")]
    UnsupportedVersion(u32),

    #[error("data at byte {offset} uses structure {id}, which wasn't defined")]
    UnknownStructure { id: u32, offset: usize },

    #[error("unknown value type 0x{value_type:02x} in `{unit}.{field}` at byte {offset}")]
    UnknownValueType {
        value_type: u32,
        unit: String,
        field: String,
        offset: usize,
    },

    #[error("text isn't valid UTF-8 (at byte {offset})")]
    NotUtf8 { offset: usize },

    #[error("line {line}: {message}")]
    Text { line: usize, message: String },
}
