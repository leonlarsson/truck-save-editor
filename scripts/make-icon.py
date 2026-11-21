#!/usr/bin/env python3
"""Draws the app icon (crates/app/assets/icon.ico): an amber rounded square with a white truck,
like the logo in the app's sidebar. Our own drawing, made from simple shapes, so it can be
redrawn here without design tools. Python's standard library only.

    python3 scripts/make-icon.py
"""

import os
import struct
import zlib

AMBER = (245, 158, 11)
WHITE = (255, 255, 255)
SIZES = [16, 24, 32, 48, 64, 128, 256]
SUPERSAMPLE = 4  # samples per pixel side, for smooth edges


def rounded_rect(x, y, x0, y0, x1, y1, r):
    """Whether (x, y) is inside the rectangle with corners rounded by r."""
    if not (x0 <= x <= x1 and y0 <= y <= y1):
        return False
    cx = min(max(x, x0 + r), x1 - r)
    cy = min(max(y, y0 + r), y1 - r)
    return (x - cx) ** 2 + (y - cy) ** 2 <= r * r


def circle(x, y, cx, cy, r):
    return (x - cx) ** 2 + (y - cy) ** 2 <= r * r


def colour_at(x, y):
    """The icon on a 0..1 square: None for transparent."""
    if not rounded_rect(x, y, 0.02, 0.02, 0.98, 0.98, 0.22):
        return None
    # The truck: a box body, a cab with a sloped front and a window, two wheels.
    wheel_y = 0.70
    for wheel_x in (0.33, 0.70):
        if circle(x, y, wheel_x, wheel_y, 0.085):
            return AMBER if circle(x, y, wheel_x, wheel_y, 0.035) else WHITE
        # A ring of amber around each wheel, so it stands apart from the body.
        if circle(x, y, wheel_x, wheel_y, 0.12):
            return AMBER
    if rounded_rect(x, y, 0.16, 0.30, 0.58, 0.66, 0.04):
        return WHITE
    in_cab = 0.61 <= x <= 0.84 and 0.40 <= y <= 0.66
    sloped_front = x - 0.74 <= (y - 0.40) * 0.9 or y >= 0.52
    if in_cab and sloped_front:
        is_window = 0.65 <= x <= 0.75 and 0.45 <= y <= 0.53 and x - 0.72 <= (y - 0.45) * 0.9
        return AMBER if is_window else WHITE
    return AMBER


def draw(size):
    """RGBA bytes, row by row."""
    out = bytearray()
    n = SUPERSAMPLE
    for py in range(size):
        for px in range(size):
            r = g = b = a = 0.0
            for sy in range(n):
                for sx in range(n):
                    c = colour_at((px + (sx + 0.5) / n) / size, (py + (sy + 0.5) / n) / size)
                    if c is not None:
                        r += c[0]
                        g += c[1]
                        b += c[2]
                        a += 1
            if a:
                out += bytes((round(r / a), round(g / a), round(b / a), round(255 * a / (n * n))))
            else:
                out += bytes(4)
    return bytes(out)


def png(size, rgba):
    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data) & 0xFFFFFFFF)

    raw = b"".join(b"\0" + rgba[y * size * 4:(y + 1) * size * 4] for y in range(size))
    header = struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0)
    return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", header) + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b"")


def ico(images):
    """An .ico holding PNG images (Windows Vista and later read these at every size)."""
    header = struct.pack("<HHH", 0, 1, len(images))
    offset = 6 + 16 * len(images)
    entries = b""
    data = b""
    for size, image in images:
        dim = 0 if size >= 256 else size
        entries += struct.pack("<BBBBHHII", dim, dim, 0, 0, 1, 32, len(image), offset + len(data))
        data += image
    return header + entries + data


def main():
    root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    images = [(size, png(size, draw(size))) for size in SIZES]
    out = os.path.join(root, "crates", "app", "assets", "icon.ico")
    with open(out, "wb") as f:
        f.write(ico(images))
    # A large PNG too, for the README and for looking at.
    with open(os.path.join(root, "crates", "app", "assets", "icon.png"), "wb") as f:
        f.write(dict(images)[256])
    print(out)


if __name__ == "__main__":
    main()
