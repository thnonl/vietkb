# Verification helper: renders the 512 masters at 4x on #202020 and #808080, crops the top-left corner and the top
# edge (200 x 120 at 512-scale -> 800 x 480), stacks them into _zoom.png; then samples the rendered 512 PNG 2 px
# outside the silhouette. Run: python design/logo11/_zoom.py [tag]
import pathlib, subprocess, sys
from PIL import Image

HERE = pathlib.Path(__file__).resolve().parent
EDGE = r'C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe'
tag = sys.argv[1] if len(sys.argv) > 1 else 'now'
cells = [(k, bg) for k in 've' for bg in ('#202020', '#808080')]
page = HERE / '_z.html'
page.write_text('<!doctype html><html><body style="margin:0">' + ''.join(
    f'<div style="position:absolute;left:0;top:{i * 2048}px;width:2048px;height:2048px;background:{bg}">'
    f'<img src="keycap/{k}-master.svg" width="2048" height="2048"></div>' for i, (k, bg) in enumerate(cells)) + '</body></html>', encoding='utf-8')
shot = HERE / '_z.png'
subprocess.run([EDGE, '--headless=new', '--disable-gpu', '--force-device-scale-factor=1', '--hide-scrollbars',
                f'--window-size=2048,{2048 * len(cells)}', f'--screenshot={shot}', page.as_uri()], capture_output=True)
im = Image.open(shot).convert('RGB')
out = Image.new('RGB', (1600 + 8, (480 + 8) * len(cells)), '#E7E4DE')
for i, (k, bg) in enumerate(cells):
    oy = i * 2048
    tl = im.crop((0, oy, 800, oy + 480))                  # 512 units (0..200, 0..120) x4 : top-left corner
    te = im.crop((4 * 160, oy, 4 * 160 + 800, oy + 480))  # top edge (x 160..360)
    out.paste(tl, (0, i * 488)); out.paste(te, (808, i * 488))
out.save(HERE / f'_zoom-{tag}.png')
page.unlink(); shot.unlink()

# outside sample on the transparent 512 PNG: for each row/column, find the silhouette (alpha > 128) and test the pixel
# 2 px outside it; anything with alpha > 8 and light (every channel > 200) counts as bleed.
for k in 've':
    p = Image.open(HERE / 'keycap' / f'{k}-512.png').convert('RGBA'); px = p.load(); W, H = p.size
    bad, n = [], 0
    for y in range(H):
        xs = [x for x in range(W) if px[x, y][3] > 128]
        if not xs: continue
        for x in (xs[0] - 2, xs[-1] + 2):
            if 0 <= x < W:
                n += 1; c = px[x, y]
                if c[3] > 8 and min(c[:3]) > 200: bad.append((x, y, c))
    for x in range(W):
        ys = [y for y in range(H) if px[x, y][3] > 128]
        if not ys: continue
        y = ys[0] - 2
        if y >= 0:
            n += 1; c = px[x, y]
            if c[3] > 8 and min(c[:3]) > 200: bad.append((x, y, c))
    print(k, 'samples', n, 'light pixels outside', len(bad), bad[:4])
