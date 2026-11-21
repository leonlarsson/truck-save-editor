//! Reads SCS Software's HashFS archives (the game's `.scs` files), to look up the game's own data:
//! level steps, names in every language, icons. Read-only, and only what's asked for is read, so
//! even the 10 GB `base.scs` opens instantly.
//!
//! Format: HashFS version 2 (game 1.50 and later) and version 1 (older, and many mods). See
//! `scs_tools`' FORMAT.md (https://github.com/Wilps93/scs_tools) for the full description.
//!
//! ```no_run
//! # fn main() -> hashfs::Result<()> {
//! let mut archive = hashfs::Archive::open("def.scs")?;
//! if let Some(bytes) = archive.read("def/economy_data.sii")? {
//!     // a SII file: hand it to `sii::read`
//! }
//! # Ok(()) }
//! ```

mod cityhash;
mod gdeflate;

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

pub use cityhash::cityhash64;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("couldn't read the archive: {0}")]
    Io(#[from] std::io::Error),

    #[error("this isn't a HashFS archive")]
    NotHashFs,

    #[error("HashFS version {0} isn't supported")]
    UnsupportedVersion(u16),

    #[error("the archive's {0} is damaged")]
    Damaged(&'static str),

    #[error("{path} uses compression {code:#x}, which isn't supported yet")]
    UnsupportedCompression { path: String, code: u8 },

    #[error("{path}: couldn't decompress its texture: {reason}")]
    Texture { path: String, reason: String },
}

/// One file or directory in the archive.
#[derive(Clone, Copy, Debug)]
struct Entry {
    is_directory: bool,
    /// For textures: what the image is.
    image: Option<ImageInfo>,
    offset: u64,
    stored_size: u64,
    size: u64,
    compression: Compression,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Compression {
    None,
    Zlib,
    /// GDeflate (`0x30`), used for textures.
    GDeflate,
    Other(u8),
}

/// What a directory holds.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Listing {
    pub directories: Vec<String>,
    pub files: Vec<String>,
}

/// An open archive.
pub struct Archive<R = BufReader<File>> {
    reader: R,
    version: u16,
    entries: HashMap<u64, Entry>,
}

impl Archive {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Self::from_reader(BufReader::new(File::open(path)?))
    }
}

impl<R: Read + Seek> Archive<R> {
    pub fn from_reader(mut reader: R) -> Result<Self> {
        let mut header = [0u8; 8];
        reader.seek(SeekFrom::Start(0))?;
        reader.read_exact(&mut header)?;
        if &header[0..4] != b"SCS#" {
            return Err(Error::NotHashFs);
        }
        let version = u16::from_le_bytes([header[4], header[5]]);
        let entries = match version {
            1 => read_v1_entries(&mut reader)?,
            2 => read_v2_entries(&mut reader)?,
            other => return Err(Error::UnsupportedVersion(other)),
        };
        Ok(Self {
            reader,
            version,
            entries,
        })
    }

    /// How many files and directories the archive holds.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Whether the archive has a file or directory at this path.
    pub fn contains(&self, path: &str) -> bool {
        self.entries.contains_key(&hash(path))
    }

    /// A file's bytes (`None` if the archive doesn't have it, or it's a directory).
    pub fn read(&mut self, path: &str) -> Result<Option<Vec<u8>>> {
        match self.entries.get(&hash(path)).copied() {
            Some(entry) if !entry.is_directory => self.read_entry(path, entry).map(Some),
            _ => Ok(None),
        }
    }

    /// What a directory holds (`None` if the archive doesn't have it).
    pub fn list(&mut self, path: &str) -> Result<Option<Listing>> {
        let Some(entry) = self.entries.get(&hash(path)).copied() else {
            return Ok(None);
        };
        if !entry.is_directory {
            return Ok(None);
        }
        let bytes = self.read_entry(path, entry)?;
        Ok(Some(match self.version {
            1 => parse_v1_listing(&bytes),
            _ => parse_v2_listing(&bytes)?,
        }))
    }

    /// A texture's first (largest) image, as stored: tightly packed rows of its pixel format.
    /// `None` if the archive has no texture at this path (a `.tobj` file in version 2 archives).
    pub fn read_texture(&mut self, path: &str) -> Result<Option<Texture>> {
        let Some(entry) = self.entries.get(&hash(path)).copied() else {
            return Ok(None);
        };
        let Some(image) = entry.image else {
            return Ok(None);
        };
        let bytes = self.read_entry(path, entry)?;
        Ok(image.first_mip(&bytes))
    }

    fn read_entry(&mut self, path: &str, entry: Entry) -> Result<Vec<u8>> {
        let stored_size =
            usize::try_from(entry.stored_size).map_err(|_| Error::Damaged("entry size"))?;
        let mut stored = vec![0u8; stored_size];
        self.reader.seek(SeekFrom::Start(entry.offset))?;
        self.reader.read_exact(&mut stored)?;
        match entry.compression {
            Compression::None => Ok(stored),
            Compression::Zlib => {
                let mut out = Vec::with_capacity(usize::try_from(entry.size).unwrap_or(0));
                flate2::read::ZlibDecoder::new(stored.as_slice()).read_to_end(&mut out)?;
                Ok(out)
            }
            Compression::GDeflate => {
                gdeflate::decompress(&stored, None).map_err(|error| Error::Texture {
                    path: path.to_string(),
                    reason: error.to_string(),
                })
            }
            Compression::Other(code) => Err(Error::UnsupportedCompression {
                path: path.to_string(),
                code,
            }),
        }
    }
}

/// What a texture entry's image part says (8 bytes): its size, mip levels and pixel format.
#[derive(Clone, Copy, Debug)]
struct ImageInfo {
    width: u32,
    height: u32,
    mip_count: u32,
    is_cubemap: bool,
    /// The format the code suggests; checked against the data's size when reading.
    format: TextureFormat,
}

impl ImageInfo {
    fn read(metadata: &[u8], at: usize) -> Option<Self> {
        let bytes = metadata.get(at..at + 8)?;
        let width = u32::from(u16::from_le_bytes([bytes[0], bytes[1]])) + 1;
        let height = u32::from(u16::from_le_bytes([bytes[2], bytes[3]])) + 1;
        let format = TextureFormat::from_code(bytes[4] >> 4, bytes[5]);
        let mip_count = u32::from(bytes[4] & 0x0F) + 1;
        let is_cubemap = bytes[6] == 0x81;
        Some(Self {
            width,
            height,
            mip_count,
            is_cubemap,
            format,
        })
    }

    /// How many bytes the whole texture takes in the GPU upload layout, in a format: each
    /// subresource (face × mip) has rows padded to 256 bytes, and all but the last start on a
    /// 512-byte boundary.
    fn upload_size(&self, format: TextureFormat) -> usize {
        let (block, bytes_per_block) = format.block();
        let faces = if self.is_cubemap { 6 } else { 1 };
        let mut total = 0;
        for face in 0..faces {
            for mip in 0..self.mip_count {
                let width = (self.width >> mip).max(1);
                let height = (self.height >> mip).max(1);
                let tight_row = width.div_ceil(block) as usize * bytes_per_block;
                let pitch = tight_row.div_ceil(256) * 256;
                let rows = height.div_ceil(block) as usize;
                let is_last = face == faces - 1 && mip == self.mip_count - 1;
                total += pitch * (rows - 1) + tight_row;
                if !is_last {
                    total = total.div_ceil(512) * 512;
                }
            }
        }
        total
    }

    /// The format whose layout fits the data's size: the code's guess when it fits (formats of
    /// one block size can't be told apart by size), else the first that does. Some codes aren't
    /// in the known table; the size settles those.
    fn checked_format(&self, length: usize) -> TextureFormat {
        if self.upload_size(self.format) == length {
            return self.format;
        }
        [
            TextureFormat::Rgba8,
            TextureFormat::Bc7,
            TextureFormat::Bc1,
            TextureFormat::R8,
        ]
        .into_iter()
        .find(|format| self.upload_size(*format) == length)
        .unwrap_or(self.format)
    }

    /// Cuts the first mip level out of the GPU upload layout: rows padded to 256 bytes.
    fn first_mip(&self, bytes: &[u8]) -> Option<Texture> {
        let format = self.checked_format(bytes.len());
        let (block, bytes_per_block) = format.block();
        let columns = self.width.div_ceil(block) as usize;
        let rows = self.height.div_ceil(block) as usize;
        let tight_row = columns * bytes_per_block;
        let pitch = tight_row.div_ceil(256) * 256;
        let mut data = Vec::with_capacity(tight_row * rows);
        for row in 0..rows {
            data.extend_from_slice(bytes.get(row * pitch..row * pitch + tight_row)?);
        }
        Some(Texture {
            width: self.width,
            height: self.height,
            format,
            data,
        })
    }
}

/// A texture's pixel format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextureFormat {
    Bc1,
    Bc4,
    Bc5,
    Bc7,
    Rgba8,
    R8,
}

impl TextureFormat {
    /// From the image part's code (byte 4's high nibble, byte 5), as `scs_tools` verified it on
    /// every texture in the game; unknown codes fall back to BC7.
    fn from_code(high: u8, low: u8) -> Self {
        match (high, low) {
            (0x3, 5) => TextureFormat::Bc5,
            (0x8 | 0x7, 4) => TextureFormat::Bc1,
            (0x0, 5) => TextureFormat::Bc4,
            (0xB | 0xD | 0x8, 5) => TextureFormat::Rgba8,
            (0xD, 3) => TextureFormat::R8,
            _ => TextureFormat::Bc7,
        }
    }

    /// (pixels per block side, bytes per block): 4×4 blocks for BC formats, single pixels else.
    pub fn block(self) -> (u32, usize) {
        match self {
            TextureFormat::Bc1 | TextureFormat::Bc4 => (4, 8),
            TextureFormat::Bc5 | TextureFormat::Bc7 => (4, 16),
            TextureFormat::Rgba8 => (1, 4),
            TextureFormat::R8 => (1, 1),
        }
    }
}

/// A texture's first image, as stored.
#[derive(Clone, Debug, PartialEq)]
pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    /// Tightly packed rows (of blocks, for BC formats).
    pub data: Vec<u8>,
}

/// A path as the archive keys it: no leading slash.
fn hash(path: &str) -> u64 {
    cityhash64(path.trim_start_matches('/').as_bytes())
}

fn u16_at(bytes: &[u8], at: usize) -> u16 {
    bytes
        .get(at..at + 2)
        .map(|b| u16::from_le_bytes([b[0], b[1]]))
        .unwrap_or(0)
}

fn u32_at(bytes: &[u8], at: usize) -> u32 {
    bytes
        .get(at..at + 4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .unwrap_or(0)
}

fn u64_at(bytes: &[u8], at: usize) -> u64 {
    bytes
        .get(at..at + 8)
        .and_then(|b| b.try_into().ok())
        .map(u64::from_le_bytes)
        .unwrap_or(0)
}

fn inflate(stored: &[u8]) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    flate2::read::ZlibDecoder::new(stored).read_to_end(&mut out)?;
    Ok(out)
}

fn read_at<R: Read + Seek>(reader: &mut R, offset: u64, length: usize) -> Result<Vec<u8>> {
    let mut bytes = vec![0u8; length];
    reader.seek(SeekFrom::Start(offset))?;
    reader.read_exact(&mut bytes)?;
    Ok(bytes)
}

/// Version 2: two zlib-compressed tables near the end of the file. The entry table maps each
/// path's hash to "parts" in the metadata table; the data part says where the bytes are.
fn read_v2_entries<R: Read + Seek>(reader: &mut R) -> Result<HashMap<u64, Entry>> {
    let header = read_at(reader, 0, 0x34)?;
    if &header[8..12] != b"CITY" {
        return Err(Error::Damaged("header"));
    }
    let entry_count = u32_at(&header, 0x0C) as usize;
    let entry_table_size = u32_at(&header, 0x10) as usize;
    let metadata_size = u32_at(&header, 0x18) as usize;
    let entry_table_offset = u64_at(&header, 0x1C);
    let metadata_offset = u64_at(&header, 0x24);

    let entry_table = inflate(&read_at(reader, entry_table_offset, entry_table_size)?)?;
    let metadata = inflate(&read_at(reader, metadata_offset, metadata_size)?)?;
    if entry_table.len() < entry_count * 16 {
        return Err(Error::Damaged("entry table"));
    }

    let mut entries = HashMap::with_capacity(entry_count);
    for index in 0..entry_count {
        let at = index * 16;
        let path_hash = u64_at(&entry_table, at);
        let metadata_index = u32_at(&entry_table, at + 8) as usize;
        let part_count = u16_at(&entry_table, at + 12) as usize;
        let flags = u16_at(&entry_table, at + 14);

        // The data part says where the bytes are; an image part (kind 1) describes a texture.
        let mut data_part = None;
        let mut image = None;
        for part in 0..part_count {
            let descriptor = u32_at(&metadata, (metadata_index + part) * 4);
            let kind = (descriptor >> 24) as u8;
            let at = (descriptor & 0x00FF_FFFF) as usize * 4;
            if kind & 0x80 != 0 {
                data_part = Some(at);
            } else if kind == 1 {
                image = ImageInfo::read(&metadata, at);
            }
        }
        let Some(at) = data_part else {
            continue;
        };
        let stored_size = u64::from(u32_at(&metadata, at) & 0x00FF_FFFF);
        let compression_code = metadata.get(at + 3).copied().unwrap_or(0) & 0xF0;
        let size = u64::from(u32_at(&metadata, at + 4) & 0x00FF_FFFF);
        let offset = u64::from(u32_at(&metadata, at + 12)) * 16;
        entries.insert(
            path_hash,
            Entry {
                is_directory: flags & 1 != 0,
                image,
                offset,
                stored_size,
                size,
                compression: match compression_code {
                    0x00 => Compression::None,
                    0x10 => Compression::Zlib,
                    0x30 => Compression::GDeflate,
                    other => Compression::Other(other),
                },
            },
        );
    }
    Ok(entries)
}

/// Version 1: a plain table of 32-byte entries.
fn read_v1_entries<R: Read + Seek>(reader: &mut R) -> Result<HashMap<u64, Entry>> {
    let header = read_at(reader, 0, 20)?;
    let entry_count = u32_at(&header, 12) as usize;
    let table_offset = u64::from(u32_at(&header, 16));
    let table = read_at(reader, table_offset, entry_count * 32)?;
    let mut entries = HashMap::with_capacity(entry_count);
    for index in 0..entry_count {
        let at = index * 32;
        let flags = u32_at(&table, at + 16);
        entries.insert(
            u64_at(&table, at),
            Entry {
                is_directory: flags & 1 != 0,
                image: None,
                offset: u64_at(&table, at + 8),
                size: u64::from(u32_at(&table, at + 24)),
                stored_size: u64::from(u32_at(&table, at + 28)),
                compression: if flags & 2 != 0 {
                    Compression::Zlib
                } else {
                    Compression::None
                },
            },
        );
    }
    Ok(entries)
}

/// Version 2 listing: `u32 count`, a length byte per name, then the names; names starting with
/// `/` are directories.
fn parse_v2_listing(bytes: &[u8]) -> Result<Listing> {
    let count = u32_at(bytes, 0) as usize;
    let lengths = bytes
        .get(4..4 + count)
        .ok_or(Error::Damaged("directory listing"))?;
    let mut at = 4 + count;
    let mut listing = Listing::default();
    for &length in lengths {
        let name = bytes
            .get(at..at + usize::from(length))
            .ok_or(Error::Damaged("directory listing"))?;
        at += usize::from(length);
        let name = String::from_utf8_lossy(name).into_owned();
        match name.strip_prefix('/') {
            Some(directory) => listing.directories.push(directory.to_string()),
            None => listing.files.push(name),
        }
    }
    Ok(listing)
}

/// Version 1 listing: one name per line; `*` marks directories.
fn parse_v1_listing(bytes: &[u8]) -> Listing {
    let mut listing = Listing::default();
    for line in String::from_utf8_lossy(bytes).lines() {
        match line.strip_prefix('*') {
            Some(directory) => listing.directories.push(directory.to_string()),
            None if !line.is_empty() => listing.files.push(line.to_string()),
            None => {}
        }
    }
    listing
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_listings() {
        let mut v2 = 3u32.to_le_bytes().to_vec();
        v2.extend_from_slice(&[4, 8, 9]);
        v2.extend_from_slice(b"/defcity.siisome.file");
        let listing = parse_v2_listing(&v2).expect("parses");
        assert_eq!(listing.directories, ["def"]);
        assert_eq!(listing.files, ["city.sii", "some.file"]);
        assert!(parse_v2_listing(&[5, 0, 0, 0, 1]).is_err());

        let v1 = parse_v1_listing(b"*def\ncity.sii\n");
        assert_eq!(v1.directories, ["def"]);
        assert_eq!(v1.files, ["city.sii"]);
    }

    #[test]
    fn refuses_what_isnt_an_archive() {
        let not_an_archive = std::io::Cursor::new(b"PK\x03\x04 not hashfs".to_vec());
        assert!(matches!(
            Archive::from_reader(not_an_archive),
            Err(Error::NotHashFs)
        ));
    }
}
