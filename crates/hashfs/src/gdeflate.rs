//! GDeflate decompression, for texture payloads in HashFS version 2 archives (compression `0x30`).
//!
//! Ported unchanged in substance from `scs_tools` (https://github.com/Wilps93/scs_tools,
//! `src/gdeflate.rs`), Copyright (c) 2026 scs_tools contributors, MIT license (see
//! `THIRD_PARTY_NOTICES.md`), itself a port of NVIDIA's libdeflate (gdeflate branch). Changes here:
//! `expect`s on fixed-size slices became checked reads, so damaged input is an error, not a crash.
//!
//! `GDeflate` (NVIDIA / Microsoft `DirectStorage`) is DEFLATE whose bitstream is
//! split across 32 parallel "lanes" for GPU-friendly SIMD decoding, plus a
//! deferred-copy scheme: a match's length is decoded on one visit to a lane and
//! its offset on that lane's *next* visit, 32 symbols later.
//!
//! SCS `HashFS` v2 stores texture (tobj) payloads as a `GDeflate` **tile stream**:
//!
//! ```text
//! TileStream header (8 bytes):
//!     u8  id           = 0x04
//!     u8  magic        = 0xFB   (= id ^ 0xFF)
//!     u16 numTiles
//!     u32 bitfield:  tileSizeIdx:2 (=1)  lastTileSize:18  reserved:12
//! u32 tileOffsets[numTiles]   byte offset of each tile; entry [0] is
//!                             repurposed as the *size* of the last tile
//! u8  tileData[...]           numTiles pages, each -> 64 KiB except the last
//! ```
//!
//! Ported from NVIDIA's libdeflate (gdeflate branch),
//! `lib/gdeflate_decompress_template.h` and `lib/deflate_constants.h`.
//!
//! The symbol tables are **DEFLATE64**, not plain DEFLATE. Getting this wrong
//! is the usual reason a `GDeflate` port almost works:
//!
//! * length symbol 285 is base 3 with 16 extra bits (matches up to 65538
//!   bytes, enough to span a whole tile), not the fixed 258 of plain DEFLATE;
//! * offset symbols 30 and 31 exist (base 32769 / 49153, 14 extra bits),
//!   allowing back-references up to 64 KiB.

use std::fmt;

pub const NUM_STREAMS: usize = 32;
pub const BITS_PER_PACKET: u32 = 32;
pub const LOW_WATERMARK_BITS: u32 = 32;
pub const DEFAULT_TILE_SIZE: usize = 64 * 1024;
pub const GDEFLATE_ID: u8 = 0x04;

const LENGTH_BASE: [u32; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
    163, 195, 227, 3,
];
const LENGTH_EXTRA: [u32; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 16,
];
const OFFSET_BASE: [u32; 32] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
    2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577, 32769, 49153,
];
const OFFSET_EXTRA: [u32; 32] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13, 14, 14,
];
const PRECODE_PERM: [usize; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    TooSmall,
    BadHeader {
        id: u8,
        magic: u8,
    },
    UnexpectedTileSizeIndex(u32),
    BadLastTileSize {
        size: usize,
        tiles: usize,
    },
    OutOfInput {
        at: usize,
    },
    OutputOverflow(&'static str),
    MatchBeforeStart(&'static str),
    ReservedBlockType,
    BadCodeLengths,
    TileSize {
        tile: usize,
        got: usize,
        want: usize,
    },
    StreamSize {
        got: usize,
        declared: usize,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooSmall => write!(f, "stream too small"),
            Self::BadHeader { id, magic } => {
                write!(f, "bad tile stream header id={id:#x} magic={magic:#x}")
            }
            Self::UnexpectedTileSizeIndex(i) => write!(f, "unexpected tileSizeIdx {i}"),
            Self::BadLastTileSize { size, tiles } => write!(
                f,
                "lastTileSize {size} is impossible for {tiles} tiles of {DEFAULT_TILE_SIZE} bytes"
            ),
            Self::OutOfInput { at } => write!(f, "ran out of input at offset {at}"),
            Self::OutputOverflow(what) => write!(f, "output overflow ({what})"),
            Self::MatchBeforeStart(what) => write!(f, "match offset before start ({what})"),
            Self::ReservedBlockType => write!(f, "reserved block type 3"),
            Self::BadCodeLengths => write!(f, "code length list overruns its declared size"),
            Self::TileSize { tile, got, want } => {
                write!(f, "tile {tile} produced {got} bytes, expected {want}")
            }
            Self::StreamSize { got, declared } => {
                write!(f, "stream size {got} != declared {declared}")
            }
        }
    }
}

impl std::error::Error for Error {}

/// A decode table entry: the symbol and how many bits it consumed.
#[derive(Clone, Copy, Default)]
struct TableEntry {
    symbol: u16,
    length: u8,
}

struct Table {
    entries: Vec<TableEntry>,
    mask: u64,
}

impl Table {
    const fn new() -> Self {
        Self {
            entries: Vec::new(),
            mask: 0,
        }
    }
}

/// The fixed Huffman codes of RFC 1951 §3.2.6, built once per process.
fn static_tables() -> &'static (Table, Table) {
    static TABLES: std::sync::OnceLock<(Table, Table)> = std::sync::OnceLock::new();
    TABLES.get_or_init(|| {
        let mut litlen = [0u8; 288];
        litlen[..144].fill(8);
        litlen[144..256].fill(9);
        litlen[256..280].fill(7);
        litlen[280..].fill(8);
        (build_table(&litlen), build_table(&[5u8; 32]))
    })
}

fn reverse_bits(mut value: u32, count: u32) -> u32 {
    let mut r = 0;
    for _ in 0..count {
        r = (r << 1) | (value & 1);
        value >>= 1;
    }
    r
}

/// Build a decode table from canonical code lengths. Indexed by the low
/// `max_len` bits of the bit buffer, first-read bit in the LSB.
fn build_table(lengths: &[u8]) -> Table {
    let mut table = Table::new();
    build_table_into(lengths, &mut table);
    table
}

/// The same, reusing `table`'s allocation. A page can hold many dynamic
/// blocks, and rebuilding is common enough that the allocation shows up.
fn build_table_into(lengths: &[u8], table: &mut Table) {
    let max_len = u32::from(lengths.iter().copied().max().unwrap_or(0));
    if max_len == 0 {
        table.entries.clear();
        table.entries.push(TableEntry::default());
        table.mask = 0;
        return;
    }
    // Code lengths are at most 15 bits in DEFLATE, so these are small.
    let mut bl_count = [0u32; 16];
    for &l in lengths {
        if l != 0 {
            bl_count[l as usize] += 1;
        }
    }
    // Canonical first code for each length, MSB-first.
    let mut next_code = [0u32; 17];
    let mut code = 0u32;
    for bits in 1..=max_len as usize {
        code = (code + bl_count[bits - 1]) << 1;
        next_code[bits] = code;
    }

    let size = 1usize << max_len;
    table.entries.clear();
    table.entries.resize(size, TableEntry::default());
    table.mask = (1u64 << max_len) - 1;
    let entries = &mut table.entries;
    for (sym, &l) in lengths.iter().enumerate() {
        if l == 0 {
            continue;
        }
        let c = next_code[l as usize];
        next_code[l as usize] += 1;
        let rev = reverse_bits(c, u32::from(l)) as usize;
        let step = 1usize << l;
        let fill = TableEntry {
            symbol: sym as u16,
            length: l,
        };
        let mut idx = rev;
        while idx < size {
            entries[idx] = fill;
            idx += step;
        }
    }
}

/// The 32 lanes' bit buffers, plus the deferred-copy state per lane.
struct Lanes<'a> {
    bitbuf: [u64; NUM_STREAMS],
    bitsleft: [i32; NUM_STREAMS],
    pending: [bool; NUM_STREAMS],
    cp_len: [u32; NUM_STREAMS],
    cp_dst: [usize; NUM_STREAMS],
    idx: usize,
    data: &'a [u8],
    pos: usize,
}

impl<'a> Lanes<'a> {
    const fn new(data: &'a [u8], pos: usize) -> Self {
        Lanes {
            bitbuf: [0; NUM_STREAMS],
            bitsleft: [0; NUM_STREAMS],
            pending: [false; NUM_STREAMS],
            cp_len: [0; NUM_STREAMS],
            cp_dst: [0; NUM_STREAMS],
            idx: 0,
            data,
            pos,
        }
    }

    #[inline]
    fn packet(&mut self) -> Result<u32, Error> {
        if self.pos + 4 > self.data.len() {
            return Err(Error::OutOfInput { at: self.pos });
        }
        let w = u32::from_le_bytes([
            self.data[self.pos],
            self.data[self.pos + 1],
            self.data[self.pos + 2],
            self.data[self.pos + 3],
        ]);
        self.pos += (BITS_PER_PACKET / 8) as usize;
        Ok(w)
    }

    /// Top the current lane up if it has dropped below the watermark.
    #[inline]
    fn refill(&mut self) -> Result<(), Error> {
        let i = self.idx;
        if self.bitsleft[i] < LOW_WATERMARK_BITS as i32 {
            let w = u64::from(self.packet()?);
            self.bitbuf[i] |= w << self.bitsleft[i];
            self.bitsleft[i] += BITS_PER_PACKET as i32;
        }
        Ok(())
    }

    #[inline]
    const fn peek(&self, n: u32) -> u64 {
        self.bitbuf[self.idx] & ((1u64 << n) - 1)
    }

    #[inline]
    fn drop_bits(&mut self, n: u32) {
        let i = self.idx;
        self.bitbuf[i] >>= n;
        self.bitsleft[i] -= n as i32;
    }

    #[inline]
    fn take(&mut self, n: u32) -> u64 {
        let v = self.peek(n);
        self.drop_bits(n);
        v
    }

    /// Refill this lane, then move to the next one.
    #[inline]
    fn advance(&mut self) -> Result<(), Error> {
        self.refill()?;
        self.idx = (self.idx + 1) % NUM_STREAMS;
        Ok(())
    }

    #[inline]
    fn decode(&mut self, table: &Table) -> u16 {
        let entry = table.entries[(self.bitbuf[self.idx] & table.mask) as usize];
        self.drop_bits(u32::from(entry.length));
        entry.symbol
    }
}

/// Copy a match, propagating byte by byte when the source overlaps the target.
#[inline]
fn copy_match(
    out: &mut [u8],
    dst: usize,
    offset: usize,
    len: usize,
    out_start: usize,
    what: &'static str,
) -> Result<(), Error> {
    if offset > dst || dst - offset < out_start {
        return Err(Error::MatchBeforeStart(what));
    }
    let src = dst - offset;
    if dst + len > out.len() {
        return Err(Error::OutputOverflow(what));
    }
    if offset >= len {
        out.copy_within(src..src + len, dst);
    } else {
        for k in 0..len {
            out[dst + k] = out[src + k];
        }
    }
    Ok(())
}

/// Decode one `GDeflate` page into `out[out_start..out_limit]`.
/// Returns how many bytes it produced.
pub fn decompress_page(
    data: &[u8],
    pos: usize,
    out: &mut [u8],
    out_start: usize,
    out_limit: usize,
) -> Result<usize, Error> {
    let mut l = Lanes::new(data, pos);
    let mut out_pos = out_start;

    // Prime all 32 lanes with one packet each; idx returns to 0.
    for n in 0..NUM_STREAMS {
        l.bitbuf[n] = u64::from(l.packet()?);
        l.bitsleft[n] = BITS_PER_PACKET as i32;
    }

    // Reused across the blocks of this page: a dynamic block rebuilds them,
    // a fixed block borrows the process-wide pair instead.
    let mut dyn_litlen = Table::new();
    let mut dyn_offset = Table::new();
    let mut pre_table = Table::new();
    let mut lens: Vec<u8> = Vec::with_capacity(320);

    loop {
        l.idx = 0;
        // Block header comes from lane 0, with no advance between reads.
        let header = l.take(3);
        let is_final = header & 1 != 0;
        let block_type = (header >> 1) & 3;
        l.refill()?;

        let dynamic = match block_type {
            2 => {
                let counts = l.take(14);
                let num_litlen = (counts & 31) as usize + 257;
                let num_offset = ((counts >> 5) & 31) as usize + 1;
                let num_precode = ((counts >> 10) & 15) as usize + 4;
                l.refill()?;

                let mut precode_lens = [0u8; 19];
                for i in 0..num_precode {
                    precode_lens[PRECODE_PERM[i]] = l.take(3) as u8;
                    l.advance()?;
                }
                build_table_into(&precode_lens, &mut pre_table);

                l.idx = 0;
                let total = num_litlen + num_offset;
                lens.clear();
                while lens.len() < total {
                    let sym = l.decode(&pre_table);
                    match sym {
                        0..=15 => lens.push(sym as u8),
                        16 => {
                            let rep = 3 + l.take(2) as usize;
                            let last = *lens.last().ok_or(Error::BadCodeLengths)?;
                            lens.resize(lens.len() + rep, last);
                        }
                        17 => {
                            let rep = 3 + l.take(3) as usize;
                            lens.resize(lens.len() + rep, 0);
                        }
                        _ => {
                            let rep = 11 + l.take(7) as usize;
                            lens.resize(lens.len() + rep, 0);
                        }
                    }
                    l.advance()?;
                }
                lens.truncate(total);
                build_table_into(&lens[..num_litlen], &mut dyn_litlen);
                build_table_into(&lens[num_litlen..], &mut dyn_offset);
                true
            }
            0 => {
                // Stored block: length, then that many literal bytes, one per
                // lane visit.
                let length = l.take(16) as usize;
                for _ in 0..length {
                    if out_pos >= out_limit {
                        return Err(Error::OutputOverflow("stored block"));
                    }
                    out[out_pos] = l.take(8) as u8;
                    out_pos += 1;
                    l.advance()?;
                }
                if is_final {
                    break;
                }
                continue;
            }
            1 => false, // fixed Huffman codes; see static_tables()
            _ => return Err(Error::ReservedBlockType),
        };

        let (litlen_table, offset_table) = if dynamic {
            (&dyn_litlen, &dyn_offset)
        } else {
            let (a, b) = static_tables();
            (a, b)
        };

        // Main interleaved decode loop.
        l.idx = 0;
        loop {
            if l.pending[l.idx] {
                let osym = l.decode(offset_table) as usize;
                if osym >= OFFSET_BASE.len() {
                    return Err(Error::ReservedBlockType);
                }
                let offset = OFFSET_BASE[osym] as usize + l.take(OFFSET_EXTRA[osym]) as usize;
                let i = l.idx;
                copy_match(
                    out,
                    l.cp_dst[i],
                    offset,
                    l.cp_len[i] as usize,
                    out_start,
                    "match",
                )?;
                l.pending[i] = false;
                l.advance()?;
            } else {
                let sym = l.decode(litlen_table);
                if sym < 256 {
                    if out_pos >= out_limit {
                        return Err(Error::OutputOverflow("literal"));
                    }
                    out[out_pos] = sym as u8;
                    out_pos += 1;
                    l.advance()?;
                    continue;
                }
                if sym == 256 {
                    break;
                }
                // A match: the length is read now, the offset 32 symbols later.
                let li = sym as usize - 257;
                if li >= LENGTH_BASE.len() {
                    return Err(Error::ReservedBlockType);
                }
                let length = LENGTH_BASE[li] + l.take(LENGTH_EXTRA[li]) as u32;
                let i = l.idx;
                l.cp_len[i] = length;
                l.cp_dst[i] = out_pos;
                l.pending[i] = true;
                out_pos += length as usize;
                if out_pos > out_limit {
                    return Err(Error::OutputOverflow("match reserve"));
                }
                l.advance()?;
            }
        }

        // Drain deferred copies from the current lane onward. This must not
        // reset to lane 0: the pending copies belong to the lanes as they are.
        for _ in 0..NUM_STREAMS {
            if l.pending[l.idx] {
                let osym = l.decode(offset_table) as usize;
                if osym >= OFFSET_BASE.len() {
                    return Err(Error::ReservedBlockType);
                }
                let offset = OFFSET_BASE[osym] as usize + l.take(OFFSET_EXTRA[osym]) as usize;
                let i = l.idx;
                copy_match(
                    out,
                    l.cp_dst[i],
                    offset,
                    l.cp_len[i] as usize,
                    out_start,
                    "drain",
                )?;
                l.pending[i] = false;
            }
            l.advance()?;
        }

        if is_final {
            break;
        }
    }

    Ok(out_pos - out_start)
}

/// Header of an SCS `GDeflate` tile stream.
#[derive(Debug, Clone, Copy)]
pub struct TileStream {
    pub num_tiles: usize,
    #[allow(
        dead_code,
        reason = "parsed for completeness; the size is derived into `total`"
    )]
    pub last_tile_size: usize,
    /// Total uncompressed size of the whole stream.
    pub total: usize,
}

/// Read just the tile stream header.
pub fn parse_tile_stream(data: &[u8]) -> Result<TileStream, Error> {
    if data.len() < 8 {
        return Err(Error::TooSmall);
    }
    let id = data[0];
    let magic = data[1];
    if id != GDEFLATE_ID || magic != id ^ 0xFF {
        return Err(Error::BadHeader { id, magic });
    }
    let num_tiles = u16::from_le_bytes([data[2], data[3]]) as usize;
    let bitfield = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    let tile_size_idx = bitfield & 0x3;
    let last_tile_size = ((bitfield >> 2) & 0x3FFFF) as usize;
    if tile_size_idx != 1 {
        return Err(Error::UnexpectedTileSizeIndex(tile_size_idx));
    }
    // `lastTileSize` is an 18-bit field, so it can hold values a 64 KiB tile
    // cannot produce, and it is meaningless without a tile to apply it to.
    // Both cases underflow the subtraction below, which in a release build
    // would wrap to a huge `total` and a doomed allocation.
    if last_tile_size > DEFAULT_TILE_SIZE || (last_tile_size != 0 && num_tiles == 0) {
        return Err(Error::BadLastTileSize {
            size: last_tile_size,
            tiles: num_tiles,
        });
    }
    let mut total = num_tiles * DEFAULT_TILE_SIZE;
    if last_tile_size != 0 {
        total -= DEFAULT_TILE_SIZE - last_tile_size;
    }
    Ok(TileStream {
        num_tiles,
        last_tile_size,
        total,
    })
}

/// Decompress a full SCS `GDeflate` tile stream.
pub fn decompress(data: &[u8], uncompressed_size: Option<usize>) -> Result<Vec<u8>, Error> {
    let stream = parse_tile_stream(data)?;
    let data_start = 8 + stream.num_tiles * 4;
    if data.len() < data_start {
        return Err(Error::TooSmall);
    }
    let tile_offsets: Vec<usize> = data[8..data_start]
        .as_chunks::<4>()
        .0
        .iter()
        .map(|word| u32::from_le_bytes(*word) as usize)
        .collect();

    let mut out = vec![0u8; stream.total];
    for (t, &offset) in tile_offsets.iter().enumerate() {
        // Entry [0] holds the last tile's size, so the first tile starts at 0.
        let page_pos = data_start + if t > 0 { offset } else { 0 };
        let out_start = t * DEFAULT_TILE_SIZE;
        let out_limit = (out_start + DEFAULT_TILE_SIZE).min(stream.total);
        let produced = decompress_page(data, page_pos, &mut out, out_start, out_limit)?;
        let expected = out_limit - out_start;
        if produced != expected {
            return Err(Error::TileSize {
                tile: t,
                got: produced,
                want: expected,
            });
        }
    }

    if let Some(want) = uncompressed_size
        && stream.total != want
    {
        return Err(Error::StreamSize {
            got: stream.total,
            declared: want,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_a_canonical_table() {
        // RFC 1951 §3.2.2 worked example: lengths 2,1,3,3 give canonical codes
        // 10, 0, 110, 111.  The table is indexed LSB-first, so each code fills
        // every index whose low `len` bits equal the reversed code: symbol 1
        // (one bit, code 0) takes the even indices, symbol 0 (code 10 -> 01)
        // takes 1 and 5, and the two three-bit codes take one slot each.
        let t = build_table(&[2, 1, 3, 3]);
        assert_eq!(t.mask, 0b111);
        assert_eq!(t.entries.len(), 8);
        let got: Vec<(u16, u8)> = t.entries.iter().map(|e| (e.symbol, e.length)).collect();
        assert_eq!(
            got,
            vec![
                (1, 1), // 000
                (0, 2), // 001
                (1, 1), // 010
                (2, 3), // 011
                (1, 1), // 100
                (0, 2), // 101
                (1, 1), // 110
                (3, 3), // 111
            ]
        );
    }

    #[test]
    fn empty_lengths_give_an_empty_table() {
        let t = build_table(&[0, 0, 0]);
        assert_eq!(t.mask, 0);
        assert_eq!(t.entries.len(), 1);
    }

    #[test]
    fn reverses_bits() {
        assert_eq!(reverse_bits(0b110, 3), 0b011);
        assert_eq!(reverse_bits(0b1, 1), 0b1);
        assert_eq!(reverse_bits(0b1000_0000, 8), 0b0000_0001);
    }

    #[test]
    fn rejects_a_bad_tile_header() {
        assert_eq!(parse_tile_stream(&[0u8; 4]).unwrap_err(), Error::TooSmall);
        let mut h = [0u8; 8];
        h[0] = 0x04;
        h[1] = 0x00; // magic should be 0xfb
        assert!(matches!(
            parse_tile_stream(&h).unwrap_err(),
            Error::BadHeader { .. }
        ));
    }

    #[test]
    fn reads_a_well_formed_tile_header() {
        let mut h = [0u8; 8];
        h[0] = GDEFLATE_ID;
        h[1] = GDEFLATE_ID ^ 0xFF;
        h[2..4].copy_from_slice(&3u16.to_le_bytes());
        // tileSizeIdx = 1, lastTileSize = 1024
        h[4..8].copy_from_slice(&(1u32 | (1024u32 << 2)).to_le_bytes());
        let s = parse_tile_stream(&h).unwrap();
        assert_eq!(s.num_tiles, 3);
        assert_eq!(s.last_tile_size, 1024);
        assert_eq!(s.total, 2 * DEFAULT_TILE_SIZE + 1024);
    }

    #[test]
    fn copy_match_propagates_when_overlapping() {
        let mut out = vec![0u8; 8];
        out[0] = 0xAB;
        // offset 1, length 4: classic RLE run, must repeat the byte.
        copy_match(&mut out, 1, 1, 4, 0, "test").unwrap();
        assert_eq!(&out[..5], &[0xAB, 0xAB, 0xAB, 0xAB, 0xAB]);
    }

    #[test]
    fn copy_match_rejects_a_reference_before_the_window() {
        let mut out = vec![0u8; 8];
        assert!(copy_match(&mut out, 2, 5, 1, 0, "test").is_err());
    }
}
