# VietKB logo v9 — owner palette (V = vàng + cam, E = xanh trời nhạt), 512 master + separately drawn 16 px.
# Run: python design/logo9/_build.py
import math, pathlib, subprocess
from PIL import Image

HERE = pathlib.Path(__file__).resolve().parent
OUT = HERE / 'yellow-orange'
EDGE = r'C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe'
BGS = {'#F3F3F3': 'taskbar sáng', '#F7F7F5': 'cửa sổ sáng', '#202020': 'taskbar tối', '#272727': 'cửa sổ tối', '#808080': 'xám giữa (tham khảo)'}
# heavy = vàng, thin = cam, E = xanh trời. Dark-background set = the owner's bright hues; light-background set =
# the same hues darkened only as far as 3:1 on #F3F3F3 requires (relative luminance ~0.255 / 0.20).
COL = {'light': dict(heavy='#AD8400', thin='#CF5605', e='#1192D3'),
       'dark': dict(heavy='#FACC15', thin='#FB923C', e='#7DD3FC')}

def lum(h):
    c = [int(h[i:i + 2], 16) / 255 for i in (1, 3, 5)]
    c = [x / 12.92 if x <= .04045 else ((x + .055) / 1.055) ** 2.4 for x in c]
    return .2126 * c[0] + .7152 * c[1] + .0722 * c[2]
def cr(a, b):
    x, y = sorted((lum(a), lum(b)), reverse=True); return (x + .05) / (y + .05)

def rounded(pts, radii):
    n, out = len(pts), []
    def toward(a, b, d):
        dx, dy = b[0] - a[0], b[1] - a[1]; L = math.hypot(dx, dy); d = min(d, L / 2)
        return (a[0] + dx / L * d, a[1] + dy / L * d)
    for i in range(n):
        p0, p1, p2 = pts[i - 1], pts[i], pts[(i + 1) % n]
        if radii[i] <= 0: out.append(('L', p1)); continue
        out.append(('L', toward(p1, p0, radii[i]))); out.append(('Q', p1, toward(p1, p2, radii[i])))
    d = f'M{out[0][1][0]:.1f} {out[0][1][1]:.1f}'
    for s in out[1:] + out[:1]:
        d += f'L{s[1][0]:.1f} {s[1][1]:.1f}' if s[0] == 'L' else f'Q{s[1][0]:.1f} {s[1][1]:.1f} {s[2][0]:.1f} {s[2][1]:.1f}'
    return d + 'Z'

# ---------------------------------------------------------------- 512 master (geometry from logo8, refined)
H_TL, H_TR, H_BR, H_BL = (56, 68), (184, 44), (318.4, 448), (216.8, 448)
T_TL, T_TR = (402, 62), (478, 46)
sh = (H_BR[0] - H_TR[0]) / (H_BR[1] - H_TR[1]); st = (T_TR[0] - H_BR[0]) / (H_BR[1] - T_TR[1])
jy = (T_TL[0] + T_TL[1] * st - H_TR[0] + H_TR[1] * sh) / (sh + st); J = (H_TR[0] + (jy - H_TR[1]) * sh, jy)
# thin drawn first (under), ending exactly at the heavy's foot corner; hidden 12-unit overlap inside the heavy.
V_MASTER = ('<g transform="translate(-11 10)">'
            '<path fill="{b}" d="' + rounded([T_TL, T_TR, H_BR, (H_BR[0] - 14, H_BR[1] - 2), (J[0] - 12, J[1] + 4)], [8, 12, 0, 0, 0]) + '"/>'
            '<path fill="{a}" d="' + rounded([H_TL, H_TR, H_BR, H_BL], [12, 14, 0, 10]) + '"/></g>')
E_PTS = [(96, 48), (440, 48), (428, 120), (200, 120), (200, 220), (400, 220), (388, 292), (200, 292),
         (200, 392), (452, 392), (440, 464), (96, 464)]
E_MASTER = '<g transform="translate(-18 0)"><path fill="{e}" d="' + rounded(E_PTS, [14, 14, 14, 8, 8, 14, 14, 8, 8, 14, 14, 14]) + '"/></g>'

# ---------------------------------------------------------------- 16 px optical size (redrawn)
# integer pixel grid: heavy 3.55 -> 3.2 px perpendicular, thin 2.4 px, flat tops, square corners, interior 2.0 px at y = 8;
# E: 3 px stem and bars, two 2 px counters, every edge on a whole pixel.
V16 = ('<polygon fill="{b}" points="12.2,1 14.8,1 9.5,14 8.7,14 7.9,11"/>'
       '<polygon fill="{a}" points="1,1 4.8,1 9.5,14 6.1,14"/>')
E16 = '<polygon fill="{e}" points="3,1 13,1 13,4 6,4 6,6 12,6 12,9 6,9 6,11 13,11 13,14 3,14"/>'

def svg(vb, body): return f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {vb} {vb}" width="{vb}" height="{vb}">{body}</svg>'

def build():
    OUT.mkdir(parents=True, exist_ok=True)
    for var, c in (('', COL['light']), ('-dark', COL['dark'])):
        (OUT / f'v-master{var}.svg').write_text(svg(512, V_MASTER.format(a=c['heavy'], b=c['thin'])), encoding='utf-8')
        (OUT / f'e-master{var}.svg').write_text(svg(512, E_MASTER.format(e=c['e'])), encoding='utf-8')
        (OUT / f'v-16{var}.svg').write_text(svg(16, V16.format(a=c['heavy'], b=c['thin'])), encoding='utf-8')
        (OUT / f'e-16{var}.svg').write_text(svg(16, E16.format(e=c['e'])), encoding='utf-8')
    jobs = []
    for l in 've':
        for var in ('', '-dark'):
            jobs += [(f'{l}-{sz}{var}.png', f'{l}-master{var}.svg', sz) for sz in (512, 256, 128, 96, 64)]
            jobs += [(f'{l}-{sz}{var}.png', f'{l}-16{var}.svg', sz) for sz in (32, 20, 16)]
    x, y, rowh, pos = 0, 0, 0, []
    for fn, src, sz in jobs:
        if x + sz > 1600: x, y, rowh = 0, y + rowh + 8, 0
        pos.append((fn, src, x, y, sz)); x += sz + 8; rowh = max(rowh, sz)
    page, shot = OUT / '_r.html', OUT / '_r.png'
    page.write_text('<!doctype html><html><body style="margin:0;background:transparent">' + ''.join(
        f'<img src="{s}" style="position:absolute;left:{px}px;top:{py}px;width:{z}px;height:{z}px">' for _, s, px, py, z in pos)
        + '</body></html>', encoding='utf-8')
    subprocess.run([EDGE, '--headless=new', '--disable-gpu', '--force-device-scale-factor=1', '--hide-scrollbars',
                    '--default-background-color=00000000', f'--window-size=1610,{y + rowh + 16}', f'--screenshot={shot}', page.as_uri()], capture_output=True)
    im = Image.open(shot).convert('RGBA')
    for fn, _, px, py, z in pos:
        im.crop((px, py, px + z, py + z)).save(OUT / fn)
    page.unlink(); shot.unlink()
    rows = []
    for var in ('light', 'dark'):
        for role, h in COL[var].items():
            rows.append(f'| {var} | {role} | `{h}` | ' + ' | '.join(f'{cr(h, b):.2f}' for b in BGS) + ' |')
    (OUT / '_notes.md').write_text(f'''# Logo v9 — vàng + cam / xanh trời (owner palette)

V = bật: heavy stroke **vàng**, thin stroke **cam**. E = tắt: solid **xanh trời nhạt**. Transparent background.

## Two lightness variants of the same hues

| Variant | Role | Hex | {' | '.join(f'{b} ({n})' for b, n in BGS.items())} |
|---|---|---|{'---|' * len(BGS)}
{chr(10).join(rows)}

- **Dark-taskbar / dark-window variant**: the owner's own bright colours (`#FACC15` vàng, `#FB923C` cam, `#7DD3FC` xanh trời); all ≥ 6.6 : 1 on `#202020` and ≥ 6.4 : 1 on `#272727`.
- **Light-taskbar / light-window variant — a compromise.** None of the three bright colours can be used on a light background: `#FACC15` is 1.38 : 1 on `#F3F3F3`, `#7DD3FC` 1.50 : 1, `#FB923C` 2.04 : 1. To reach 3 : 1 they were darkened along the same hue to `#AD8400` (vàng → **dark gold / mustard**, no longer reads as "yellow"), `#CF5605` (cam → burnt orange, still orange) and `#1192D3` (xanh trời nhạt → **medium sky blue**, no longer "light"). The light variant keeps the hue relationships (yellow-orange vs blue) but loses the brightness the owner chose; it is the cost of being visible on `#F3F3F3`.
- On mid-grey `#808080` no colour of either set reaches 3 : 1 (1.1–2.6) — a transparent mark cannot be legible on every mid-tone; the two-variant rule assumes a light or a dark taskbar.

## Two-size rule numbers
- **Master (512 units, ≥ 64 px)**: heavy 113 → 95 units perpendicular, thin ~71 units, all terminals cut at the same ~11° rise with 8–14-unit radii; join = thin right edge ends exactly at the heavy foot corner, thin left edge ends on the heavy right edge, thin drawn underneath (no cap, no gap). E: 104-unit stem, 72-unit bars, equal 100-unit counters, one path. Both letters optically centred in the 512 box.
- **16 px optical size (redrawn)**: glyph 13 px tall on whole pixels (y 1–14). Heavy 3.55 → 3.2 px, thin 2.4 px, flat tops and square corners (terminal cut and radii removed), interior 2.0 px open at y = 8. E: 3 px stem and bars (thicker than the master ratio) with two 2 px counters, every edge on a whole pixel. 20 / 32 px are rendered from the 16 drawing.
''', encoding='utf-8')

def sheet():
    def ladder(var, ck):
        suf = '' if var == 'light' else '-dark'
        cells = ''.join(f'<figure><div class="{ck} pad"><img src="yellow-orange/v-{s}{suf}.png" width="{s}" height="{s}"><img src="yellow-orange/e-{s}{suf}.png" width="{s}" height="{s}"></div><figcaption>{s} px</figcaption></figure>'
                        for s in (96, 32, 20, 16))
        return (f'<figure><div class="{ck} pad"><img src="yellow-orange/v-master{suf}.svg" width="192"><img src="yellow-orange/e-master{suf}.svg" width="192"></div><figcaption>192 px</figcaption></figure>{cells}')
    html = f'''<!doctype html><html lang="vi"><head><meta charset="utf-8"><title>VietKB — logo v9 vàng + cam / xanh trời</title>
<style>
body{{margin:0;padding:18px 24px;background:#E7E4DE;font:400 13px/18px "Segoe UI Variable Text","Segoe UI",sans-serif;color:#1C1C1A;width:1552px}}
h1{{font:600 22px/30px "Segoe UI Variable Display","Segoe UI",sans-serif;margin:0 0 4px}} .lead{{margin:0 0 10px;color:#57524A}}
section{{background:#FAF9F6;border-radius:12px;box-shadow:0 0 0 1px #E0DBD2;padding:12px;margin-bottom:12px;display:flex;gap:16px}}
h2{{margin:0 0 6px;font:600 15px/22px "Segoe UI Variable Text","Segoe UI",sans-serif}} h2 code{{font:500 12px/16px "Cascadia Mono",Consolas,monospace;color:#57524A;margin-left:6px}}
.ck{{background:repeating-conic-gradient(#FFFFFF 0 25%,#E9E9E9 0 50%) 0 0/24px 24px;display:flex;width:fit-content}}
.ckd{{background:repeating-conic-gradient(#202020 0 25%,#2B2B2B 0 50%) 0 0/24px 24px;display:flex;width:fit-content}}
.big{{border-radius:8px;overflow:hidden}} .pad{{padding:6px;gap:8px;border-radius:6px}}
.side{{display:flex;flex-direction:column;gap:10px}} .lad{{display:flex;gap:12px;align-items:flex-end;flex-wrap:wrap}}
figure{{margin:0}} figcaption{{font:500 11.5px/16px "Segoe UI Variable Small","Segoe UI",sans-serif;color:#6B655B;margin-top:3px}}
.tray{{display:flex;gap:10px}} .tl,.td{{display:flex;gap:8px;padding:8px;border-radius:8px}} .tl{{background:#F3F3F3}} .td{{background:#202020}}
.px{{width:128px;height:128px;image-rendering:pixelated}}
.grey{{background:#808080;border-radius:10px;padding:10px 14px;display:flex;gap:14px;align-items:center;color:#fff}}
</style></head><body>
<h1>VietKB · logo v9 — V vàng + cam, E xanh trời nhạt</h1>
<p class="lead">Bản gốc 512 (≥ 64 px) và bản 16 px vẽ riêng. Hai biến thể độ sáng cùng sắc: bản nền tối = màu chủ sản phẩm chọn; bản nền sáng = cùng sắc làm tối tới 3:1 (thoả hiệp, xem _notes.md).</p>
<section><div><h2>Bản nền sáng <code>{COL["light"]["heavy"]} · {COL["light"]["thin"]} · {COL["light"]["e"]}</code></h2>
<div class="big ck"><img src="yellow-orange/v-master.svg" width="420" height="420"><img src="yellow-orange/e-master.svg" width="420" height="420"></div></div>
<div class="side"><h2>Các cỡ</h2><div class="lad">{ladder("light", "ck")}</div>
<div class="tray"><div class="tl"><img class="px" src="yellow-orange/v-16.png"><img class="px" src="yellow-orange/e-16.png"></div></div><figcaption>16 px ×8 trên taskbar sáng #F3F3F3</figcaption></div></section>
<section><div><h2>Bản nền tối <code>{COL["dark"]["heavy"]} · {COL["dark"]["thin"]} · {COL["dark"]["e"]}</code></h2>
<div class="big ckd"><img src="yellow-orange/v-master-dark.svg" width="420" height="420"><img src="yellow-orange/e-master-dark.svg" width="420" height="420"></div></div>
<div class="side"><h2>Các cỡ</h2><div class="lad">{ladder("dark", "ckd")}</div>
<div class="tray"><div class="td"><img class="px" src="yellow-orange/v-16-dark.png"><img class="px" src="yellow-orange/e-16-dark.png"></div></div><figcaption>16 px ×8 trên taskbar tối #202020</figcaption></div></section>
<div class="grey"><b>Trên xám giữa #808080</b><img src="yellow-orange/v-master.svg" width="96"><img src="yellow-orange/e-master.svg" width="96"><img src="yellow-orange/v-master-dark.svg" width="96"><img src="yellow-orange/e-master-dark.svg" width="96">
<img src="yellow-orange/v-32.png"><img src="yellow-orange/e-32.png"><img src="yellow-orange/v-32-dark.png"><img src="yellow-orange/e-32-dark.png"><img src="yellow-orange/v-16.png"><img src="yellow-orange/v-16-dark.png"><span>bản sáng · bản tối</span></div>
</body></html>'''
    (HERE / 'index.html').write_text(html, encoding='utf-8')
    subprocess.run([EDGE, '--headless=new', '--disable-gpu', '--force-device-scale-factor=1', '--hide-scrollbars',
                    '--window-size=1600,1360', f'--screenshot={HERE / "sheet.png"}', (HERE / 'index.html').as_uri()], capture_output=True)

if __name__ == '__main__':
    build(); sheet(); print('ok', J)
