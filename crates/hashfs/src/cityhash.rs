//! `CityHash64`, exactly the variant SCS Software's HashFS archives use to key their files.
//!
//! This is Google's CityHash v1.0.x (city.cc, Copyright 2011 Google, MIT license). Later CityHash
//! releases changed the algorithm, so a current CityHash library does **not** give the same hashes.
//! Ported from `scs_tools` by Wilps93 (MIT, https://github.com/Wilps93/scs_tools), with its test
//! vectors.
//!
//! Paths are hashed as raw bytes with no leading slash and forward slashes (`def/city.sii`); the
//! archive's root directory is the empty string.

const K0: u64 = 0xc3a5c85c97cb3127;
const K1: u64 = 0xb492b66fbe98f273;
const K2: u64 = 0x9ae16a3b2f90404f;
const K3: u64 = 0xc949d7c7509e6557;
const K_MUL: u64 = 0x9ddfea08eb382d69;

// Callers only fetch within the input (every length bucket checks its own bounds), so the
// zero fallback is never taken; it keeps the hash total instead of panicking.
fn fetch64(s: &[u8], i: usize) -> u64 {
    s.get(i..i + 8)
        .and_then(|b| b.try_into().ok())
        .map(u64::from_le_bytes)
        .unwrap_or(0)
}

fn fetch32(s: &[u8], i: usize) -> u32 {
    s.get(i..i + 4)
        .and_then(|b| b.try_into().ok())
        .map(u32::from_le_bytes)
        .unwrap_or(0)
}

/// Rotate right, passing 0 through unchanged the way the reference does.
const fn rotate(val: u64, shift: u32) -> u64 {
    if shift == 0 {
        val
    } else {
        val.rotate_right(shift)
    }
}

const fn shift_mix(val: u64) -> u64 {
    val ^ (val >> 47)
}

const fn hash128to64(low: u64, high: u64) -> u64 {
    let a = (low ^ high).wrapping_mul(K_MUL);
    let a = a ^ (a >> 47);
    let b = (high ^ a).wrapping_mul(K_MUL);
    let b = b ^ (b >> 47);
    b.wrapping_mul(K_MUL)
}

const fn hash_len16(u: u64, v: u64) -> u64 {
    hash128to64(u, v)
}

fn hash_len0to16(s: &[u8]) -> u64 {
    let len = s.len();
    if len > 8 {
        let a = fetch64(s, 0);
        let b = fetch64(s, len - 8);
        return hash_len16(a, (b.wrapping_add(len as u64)).rotate_right(len as u32)) ^ b;
    }
    if len >= 4 {
        let a = u64::from(fetch32(s, 0));
        return hash_len16(len as u64 + (a << 3), u64::from(fetch32(s, len - 4)));
    }
    if len > 0 {
        let a = u32::from(s[0]);
        let b = u32::from(s[len >> 1]);
        let c = u32::from(s[len - 1]);
        let y = u64::from(a.wrapping_add(b << 8));
        let z = u64::from((len as u32).wrapping_add(c << 2));
        return shift_mix(y.wrapping_mul(K2) ^ z.wrapping_mul(K3)).wrapping_mul(K2);
    }
    K2
}

fn hash_len17to32(s: &[u8]) -> u64 {
    let len = s.len();
    let a = fetch64(s, 0).wrapping_mul(K1);
    let b = fetch64(s, 8);
    let c = fetch64(s, len - 8).wrapping_mul(K2);
    let d = fetch64(s, len - 16).wrapping_mul(K0);
    hash_len16(
        rotate(a.wrapping_sub(b), 43)
            .wrapping_add(rotate(c, 30))
            .wrapping_add(d),
        a.wrapping_add(rotate(b ^ K3, 20))
            .wrapping_sub(c)
            .wrapping_add(len as u64),
    )
}

const fn weak_hash_raw(w: u64, x: u64, y: u64, z: u64, a: u64, b: u64) -> (u64, u64) {
    let mut a = a.wrapping_add(w);
    let mut b = rotate(b.wrapping_add(a).wrapping_add(z), 21);
    let c = a;
    a = a.wrapping_add(x);
    a = a.wrapping_add(y);
    b = b.wrapping_add(rotate(a, 44));
    (a.wrapping_add(z), b.wrapping_add(c))
}

fn weak_hash(s: &[u8], off: usize, a: u64, b: u64) -> (u64, u64) {
    weak_hash_raw(
        fetch64(s, off),
        fetch64(s, off + 8),
        fetch64(s, off + 16),
        fetch64(s, off + 24),
        a,
        b,
    )
}

fn hash_len33to64(s: &[u8]) -> u64 {
    let len = s.len();
    let mut z = fetch64(s, 24);
    let mut a = fetch64(s, 0).wrapping_add(
        (len as u64)
            .wrapping_add(fetch64(s, len - 16))
            .wrapping_mul(K0),
    );
    let mut b = rotate(a.wrapping_add(z), 52);
    let mut c = rotate(a, 37);
    a = a.wrapping_add(fetch64(s, 8));
    c = c.wrapping_add(rotate(a, 7));
    a = a.wrapping_add(fetch64(s, 16));
    let vf = a.wrapping_add(z);
    let vs = b.wrapping_add(rotate(a, 31)).wrapping_add(c);

    a = fetch64(s, 16).wrapping_add(fetch64(s, len - 32));
    z = fetch64(s, len - 8);
    b = rotate(a.wrapping_add(z), 52);
    c = rotate(a, 37);
    a = a.wrapping_add(fetch64(s, len - 24));
    c = c.wrapping_add(rotate(a, 7));
    a = a.wrapping_add(fetch64(s, len - 16));
    let wf = a.wrapping_add(z);
    let ws = b.wrapping_add(rotate(a, 31)).wrapping_add(c);

    let r = shift_mix(
        vf.wrapping_add(ws)
            .wrapping_mul(K2)
            .wrapping_add(wf.wrapping_add(vs).wrapping_mul(K0)),
    );
    shift_mix(r.wrapping_mul(K0).wrapping_add(vs)).wrapping_mul(K2)
}

/// `CityHash64` of `data`, matching the SCS `HashFS` variant.
pub fn cityhash64(data: &[u8]) -> u64 {
    let len = data.len();
    if len <= 32 {
        return if len <= 16 {
            hash_len0to16(data)
        } else {
            hash_len17to32(data)
        };
    }
    if len <= 64 {
        return hash_len33to64(data);
    }

    let s = data;
    let mut off = 0usize;
    let mut x = fetch64(s, len - 40);
    let mut y = fetch64(s, len - 16).wrapping_add(fetch64(s, len - 56));
    let mut z = hash_len16(
        fetch64(s, len - 48).wrapping_add(len as u64),
        fetch64(s, len - 24),
    );
    let mut v = weak_hash(s, len - 64, len as u64, z);
    let mut w = weak_hash(s, len - 32, y.wrapping_add(K1), x);
    x = x.wrapping_mul(K1).wrapping_add(fetch64(s, 0));

    let mut remaining = (len - 1) & !63;
    loop {
        x = rotate(
            x.wrapping_add(y)
                .wrapping_add(v.0)
                .wrapping_add(fetch64(s, off + 8)),
            37,
        )
        .wrapping_mul(K1);
        y = rotate(y.wrapping_add(v.1).wrapping_add(fetch64(s, off + 48)), 42).wrapping_mul(K1);
        x ^= w.1;
        y = y.wrapping_add(v.0).wrapping_add(fetch64(s, off + 40));
        z = rotate(z.wrapping_add(w.0), 33).wrapping_mul(K1);
        v = weak_hash(s, off, v.1.wrapping_mul(K1), x.wrapping_add(w.0));
        w = weak_hash(
            s,
            off + 32,
            z.wrapping_add(w.1),
            y.wrapping_add(fetch64(s, off + 16)),
        );
        std::mem::swap(&mut z, &mut x);
        off += 64;
        remaining -= 64;
        if remaining == 0 {
            break;
        }
    }

    hash_len16(
        hash_len16(v.0, w.0)
            .wrapping_add(shift_mix(y).wrapping_mul(K1))
            .wrapping_add(z),
        hash_len16(v.1, w.1).wrapping_add(x),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Vectors taken from the Python implementation this was ported from,
    /// which resolves 627,716 of 627,727 entries across a full retail install.
    /// The inputs cover every length bucket: 0-16, 17-32, 33-64 and the
    /// 64-byte main loop.
    #[test]
    fn matches_the_python_implementation() {
        let cases: &[(&[u8], u64)] = &[
            (b"", 0x9ae16a3b2f90404f),
            (b"a", 0x2420662cd003acfa),
            (b"abc", 0x3a912f483a4ece31),
            (b"def/city.sii", 0x9a5cb1ebaac98229),
            (b"version.txt", 0x9df28354d1b80f29),
            (b"xxxxxxxxxxxxxxxxx", 0xccde5b4c17d97655),
            (b"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx", 0x83d2f4e954e9c837),
            (
                b"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
                0x3613c0a0c0ee3c93,
            ),
            (
                b"yyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy\
                  yyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy",
                0x07a9c72386a250d6,
            ),
            (b"automat/01/0123456789abcdef.dds", 0xf567a715b38ab762),
        ];
        for (input, expected) in cases {
            assert_eq!(
                cityhash64(input),
                *expected,
                "len {} input {:?}",
                input.len(),
                String::from_utf8_lossy(&input[..input.len().min(20)])
            );
        }
        assert_eq!(cityhash64(b""), K2, "empty string hashes to k2");
    }
}
