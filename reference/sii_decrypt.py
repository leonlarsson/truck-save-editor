"""Decrypt SCS 'ScsC' save files (AES-256-CBC + zlib). Usage: sii_decrypt.py IN OUT"""
import struct, sys, zlib
from cryptography.hazmat.primitives.ciphers import Cipher, algorithms, modes

KEY = bytes.fromhex("2a5fcb1791d22fb60245b3d8369ed0b2c27371563fbf1f3c9edf6b11825a5d0a")

def decrypt(data: bytes) -> bytes:
    if data[:4] != b"ScsC":
        return data  # already plain text or binary
    iv = data[36:52]
    size = struct.unpack("<I", data[52:56])[0]
    dec = Cipher(algorithms.AES(KEY), modes.CBC(iv)).decryptor()
    plain = dec.update(data[56:]) + dec.finalize()
    out = zlib.decompress(plain)
    assert len(out) == size, (len(out), size)
    return out

if __name__ == "__main__":
    src, dst = sys.argv[1], sys.argv[2]
    out = decrypt(open(src, "rb").read())
    open(dst, "wb").write(out)
    print(dst, len(out), out[:8])
