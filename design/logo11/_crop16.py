# 8x nearest-neighbour crops of the flat 16 px V tile, on a light and a dark tray.
# Run: python design/logo11/_crop16.py  ->  crop-v16-light.png / crop-v16-dark.png
import pathlib
from PIL import Image

HERE = pathlib.Path(__file__).resolve().parent
Z, PAD = 8, 12
src = Image.open(HERE / 'keycap' / 'v-16.png').convert('RGBA')
big = src.resize((src.width * Z, src.height * Z), Image.NEAREST)
for bg, name in (('#F3F3F3', 'light'), ('#202020', 'dark')):
    out = Image.new('RGB', (big.width + 2 * PAD, big.height + 2 * PAD), bg)
    out.paste(big, (PAD, PAD), big)
    out.save(HERE / f'crop-v16-{name}.png')
    print(f'crop-v16-{name}.png', out.size)
