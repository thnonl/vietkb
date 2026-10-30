#!/usr/bin/env python3
"""Generate the macOS menu-bar template icons (crates/vkb-app/icons/tray-mac-*.png).

Template images are black + alpha only; macOS tints them for light/dark menu
bars. They follow the keycap mark: a rounded key with the letter.

  on  (V): filled key, letter knocked out
  off (E): key outline, solid letter

Output: tray-mac-{on,off}-{16,32}.png  (16 = @1x, 32 = @2x). No dependencies.
Usage: python3 scripts/gen-mac-tray-icons.py
"""
import os
import struct
import zlib

OUT = os.path.join(os.path.dirname(__file__), "..", "crates", "vkb-app", "icons")
SS = 8  # supersampling per axis

# Geometry in a 32 x 32 design grid.
KEY = (3.0, 4.0, 29.0, 28.0)  # x0, y0, x1, y1
RADIUS = 6.5
RING = 2.2
V = [(9.5, 9.5), (13.0, 9.5), (16.0, 18.2), (19.0, 9.5), (22.5, 9.5), (17.8, 22.5), (14.2, 22.5)]
E_BARS = [
    (10.5, 9.5, 14.0, 22.5),
    (10.5, 9.5, 21.8, 12.6),
    (10.5, 14.4, 20.0, 17.5),
    (10.5, 19.4, 21.8, 22.5),
]


def in_round_rect(x, y, x0, y0, x1, y1, r):
    if x < x0 or x > x1 or y < y0 or y > y1:
        return False
    cx = min(max(x, x0 + r), x1 - r)
    cy = min(max(y, y0 + r), y1 - r)
    return (x - cx) ** 2 + (y - cy) ** 2 <= r * r


def in_poly(x, y, poly):
    inside = False
    j = len(poly) - 1
    for i in range(len(poly)):
        xi, yi = poly[i]
        xj, yj = poly[j]
        if (yi > y) != (yj > y) and x < (xj - xi) * (y - yi) / (yj - yi) + xi:
            inside = not inside
        j = i
    return inside


def letter(state, x, y):
    if state == "on":
        return in_poly(x, y, V)
    return any(a <= x <= c and b <= y <= d for a, b, c, d in E_BARS)


def hit(state, x, y):
    x0, y0, x1, y1 = KEY
    body = in_round_rect(x, y, x0, y0, x1, y1, RADIUS)
    if state == "on":
        return body and not letter(state, x, y)
    ring = body and not in_round_rect(
        x, y, x0 + RING, y0 + RING, x1 - RING, y1 - RING, max(RADIUS - RING, 1.0)
    )
    return ring or letter(state, x, y)


def render(state, size):
    scale = 32.0 / size
    rows = []
    for py in range(size):
        row = []
        for px in range(size):
            n = 0
            for sy in range(SS):
                for sx in range(SS):
                    x = (px + (sx + 0.5) / SS) * scale
                    y = (py + (sy + 0.5) / SS) * scale
                    n += hit(state, x, y)
            row.append((0, 0, 0, round(255 * n / (SS * SS))))
        rows.append(row)
    return rows


def write_png(path, px):
    h, w = len(px), len(px[0])
    raw = b"".join(b"\0" + bytes(v for p in row for v in p) for row in px)

    def chunk(tag, data):
        return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)

    with open(path, "wb") as f:
        f.write(
            b"\x89PNG\r\n\x1a\n"
            + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(raw, 9))
            + chunk(b"IEND", b"")
        )


if __name__ == "__main__":
    for state in ("on", "off"):
        for size in (16, 32):
            write_png(os.path.join(OUT, f"tray-mac-{state}-{size}.png"), render(state, size))
