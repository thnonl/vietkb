from pathlib import Path
from PIL import Image

sizes = [(16, 16), (20, 20), (24, 24), (32, 32), (48, 48)]
icons_dir = Path('crates/vietkb-gui/icons')

for name in ('on.ico', 'off.ico'):
    src = icons_dir / name
    backup = icons_dir / f'{name}.bak'
    if not backup.exists():
        backup.write_bytes(src.read_bytes())

    img = Image.open(src).convert('RGBA')
    img.save(src, format='ICO', sizes=sizes)

    check = Image.open(src)
    print(name, src.stat().st_size, getattr(check, 'n_frames', '?'), check.size)
