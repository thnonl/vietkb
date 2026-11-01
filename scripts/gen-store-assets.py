#!/usr/bin/env python3
"""Generate the MSIX tile assets in packaging/msix/Assets from the keycap "V" icon.

Needs Pillow (`pip install pillow`). Run from anywhere:

    python3 scripts/gen-store-assets.py

Source: design/logo-final/v-<N>.png, the rasters design/logo-final/make.ps1
renders from v.svg with the stroke weight tuned per size (16 px stays readable).
Re-rasterizing v.svg here would throw that tuning away, so the exact sizes the
Windows icon grid asks for (16, 24, 32, 48, 256) are copied as they are, and
only the odd sizes (44, 50, 99) are resampled.

Layout rules
  * Square44x44Logo (+ targetsize-N, + _altform-unplated): the icon fills the
    plate, as in the taskbar and Start list.
  * Square150x150Logo and Wide310x150Logo: the icon is 99 px (66 % of the 150 px
    height) centred on a transparent tile, so Start keeps its own margin.
  * StoreLogo (50x50): the Store and the installer show it small; icon fills it.
"""
import shutil
import sys
from pathlib import Path

try:
    from PIL import Image
except ImportError:
    sys.exit("Pillow is required: pip install pillow")

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / "design" / "logo-final"
OUT = ROOT / "packaging" / "msix" / "Assets"

TARGET_SIZES = (16, 24, 32, 48, 256)
TILE_ICON = 99  # 66 % of 150


def source(size):
    path = SRC / "v-{}.png".format(size)
    if not path.is_file():
        sys.exit("missing {}".format(path))
    return Image.open(path).convert("RGBA")


def resized(image, size):
    return image.resize((size, size), Image.LANCZOS)


def tile(width, height, icon):
    canvas = Image.new("RGBA", (width, height), (0, 0, 0, 0))
    canvas.paste(icon, ((width - icon.width) // 2, (height - icon.height) // 2), icon)
    return canvas


def save(image, name):
    image.save(OUT / name, optimize=True)
    print("{:<52} {}x{}".format(name, image.width, image.height))


def main():
    if OUT.exists():
        shutil.rmtree(OUT)
    OUT.mkdir(parents=True)

    for size in TARGET_SIZES:
        icon = source(size)
        save(icon, "Square44x44Logo.targetsize-{}.png".format(size))
        save(icon, "Square44x44Logo.targetsize-{}_altform-unplated.png".format(size))
    save(resized(source(48), 44), "Square44x44Logo.png")

    icon = resized(source(128), TILE_ICON)
    save(tile(150, 150, icon), "Square150x150Logo.png")
    save(tile(310, 150, icon), "Wide310x150Logo.png")

    save(resized(source(64), 50), "StoreLogo.png")


if __name__ == "__main__":
    main()
