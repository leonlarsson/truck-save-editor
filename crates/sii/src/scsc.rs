//! The `ScsC` wrapper: AES-256-CBC encryption around a zlib stream.
//!
//! ```text
//! 0   4   "ScsC"
//! 4   32  HMAC-SHA256 (not checked)
//! 36  16  AES IV
//! 52  4   u32 LE: size after decompressing
//! 56  …   AES-256-CBC ciphertext (PKCS7) → zlib stream → the payload
//! ```

use std::io::Read as _;

use aes::cipher::{BlockModeDecrypt as _, KeyIvInit as _, block_padding::Pkcs7};

use crate::error::{Error, Result};

/// The key SCS uses for every save. Public and widely documented (for example by SII_Decrypt).
const KEY: [u8; 32] = [
    0x2a, 0x5f, 0xcb, 0x17, 0x91, 0xd2, 0x2f, 0xb6, 0x02, 0x45, 0xb3, 0xd8, 0x36, 0x9e, 0xd0, 0xb2,
    0xc2, 0x73, 0x71, 0x56, 0x3f, 0xbf, 0x1f, 0x3c, 0x9e, 0xdf, 0x6b, 0x11, 0x82, 0x5a, 0x5d, 0x0a,
];

const HEADER_LEN: usize = 56;

type Decryptor = cbc::Decryptor<aes::Aes256>;

/// Unwraps an `ScsC` file into what it holds (binary or text SII).
pub fn decrypt(bytes: &[u8]) -> Result<Vec<u8>> {
    if bytes.len() < HEADER_LEN {
        return Err(Error::Truncated {
            offset: bytes.len(),
        });
    }
    let iv: [u8; 16] = bytes[36..52]
        .try_into()
        .map_err(|_| Error::Decrypt("the IV is missing"))?;
    let expected = u32::from_le_bytes(
        bytes[52..56]
            .try_into()
            .map_err(|_| Error::Decrypt("the size is missing"))?,
    ) as usize;

    let mut buffer = bytes[HEADER_LEN..].to_vec();
    let compressed = Decryptor::new(&KEY.into(), &iv.into())
        .decrypt_padded::<Pkcs7>(&mut buffer)
        .map_err(|_| Error::Decrypt("the data or its padding is damaged"))?;

    let mut out = Vec::with_capacity(expected);
    flate2::read::ZlibDecoder::new(compressed)
        .read_to_end(&mut out)
        .map_err(|error| Error::Decompress(error.to_string()))?;
    if out.len() != expected {
        return Err(Error::SizeMismatch {
            expected,
            actual: out.len(),
        });
    }
    Ok(out)
}

#[cfg(test)]
pub(crate) mod tests {
    use std::io::Write as _;

    use aes::cipher::BlockModeEncrypt as _;

    use super::*;

    /// Wraps a payload the way the game does, for tests.
    pub(crate) fn encrypt(payload: &[u8]) -> Vec<u8> {
        let mut encoder =
            flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(payload).expect("compresses");
        let compressed = encoder.finish().expect("compresses");

        let iv = [7u8; 16];
        let mut buffer = compressed.clone();
        buffer.resize(compressed.len() + 16, 0);
        let ciphertext = cbc::Encryptor::<aes::Aes256>::new(&KEY.into(), &iv.into())
            .encrypt_padded::<Pkcs7>(&mut buffer, compressed.len())
            .expect("encrypts")
            .to_vec();

        let mut out = b"ScsC".to_vec();
        out.extend_from_slice(&[0u8; 32]);
        out.extend_from_slice(&iv);
        out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        out.extend_from_slice(&ciphertext);
        out
    }

    #[test]
    fn decrypts_what_was_encrypted() {
        let payload = b"SiiNunit\r\n{\r\n}\r\n".repeat(50);
        assert_eq!(decrypt(&encrypt(&payload)).expect("decrypts"), payload);
    }

    #[test]
    fn reports_damage_instead_of_crashing() {
        let mut damaged = encrypt(b"SiiNunit\r\n{\r\n}\r\n");
        damaged.truncate(60);
        assert!(decrypt(&damaged).is_err());
        assert!(matches!(decrypt(b"ScsC"), Err(Error::Truncated { .. })));
    }
}
