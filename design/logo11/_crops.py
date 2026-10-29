# 4x crops of the keycap top for eyeballing: top-left corner (x 0-200, y 0-120) and top edge (x 160-360, y 0-120),
# rendered at 2048 px and composited on #202020 (top row) and #808080 (bottom row).
# Run: python design/logo11/_crops.py  ->  crop-v-top.png / crop-e-top.png
import pathlib, subprocess
from PIL import Image

HERE = pathlib.Path(__file__).resolve().parent
EDGE = r'C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe'
CELL_W, CELL_H, SCALE = 800, 480, 4

for k in 've':
    cells = [('#202020', 0), ('#808080', 2048)]
    page = HERE / '_c.html'
    page.write_text('<!doctype html><html><body style="margin:0">' + ''.join(
        f'<div style="position:absolute;left:0;top:{oy}px;width:2048px;height:2048px;background:{bg}">'
        f'<img src="keycap/{k}-master.svg" width="2048" height="2048"></div>' for bg, oy in cells)
        + '</body></html>', encoding='utf-8')
    shot = HERE / '_c.png'
    subprocess.run([EDGE, '--headless=new', '--disable-gpu', '--force-device-scale-factor=1', '--hide-scrollbars',
                    f'--window-size=2048,4096', f'--screenshot={shot}', page.as_uri()], capture_output=True)
    im = Image.open(shot).convert('RGB')
    out = Image.new('RGB', (CELL_W * 2 + 8, CELL_H * 2 + 8), '#E7E4DE')
    for i, (_, oy) in enumerate(cells):
        out.paste(im.crop((0, oy, CELL_W, oy + CELL_H)), (0, i * (CELL_H + 8)))                    # top-left corner
        out.paste(im.crop((160 * SCALE, oy, 160 * SCALE + CELL_W, oy + CELL_H)),
                  (CELL_W + 8, i * (CELL_H + 8)))                                                  # top edge
    out.save(HERE / f'crop-{k}-top.png')
    page.unlink(); shot.unlink()
    print('crop-' + k + '-top.png', out.size)
