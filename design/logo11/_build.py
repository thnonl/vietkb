# VietKB logo v11 — matte keycap (square face, recessed body, outline and soft desk shadow).
# Size ladder: full (>=192), reduced (96-128), minimal (48-64), flat 16-px drawing (16/20/32).
# Run: python design/logo11/_build.py
import math, pathlib, subprocess
from PIL import Image

HERE = pathlib.Path(__file__).resolve().parent
OUT = HERE / 'keycap'
EDGE = r'C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe'
# Fixed face/rim/letter hues. Body colours are matte derivatives of the face;
# `band` is the bottom surface and is darker than the right wall.
T = {
    'v': dict(face='#FFEBDD', band='#E8BFA6', wall='#F3D4C2', rim='#C94810', a='#E4507C', b='#A84A00'),
    'e': dict(face='#E0F1FD', band='#A8C9E2', wall='#C6DFF1', rim='#1E72D2', a='#1C6CC6', b='#1C6CC6'),
}

def lum(h):
    c = [int(h[i:i + 2], 16) / 255 for i in (1, 3, 5)]
    c = [x / 12.92 if x <= .04045 else ((x + .055) / 1.055) ** 2.4 for x in c]
    return .2126 * c[0] + .7152 * c[1] + .0722 * c[2]
def cr(a, b):
    x, y = sorted((lum(a), lum(b)), reverse=True); return (x + .05) / (y + .05)

def rounded(pts, radii, closed=True):
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

# ---- letters (approved construction from v9/v10)
H_TL, H_TR, H_BR, H_BL = (56, 68), (184, 44), (318.4, 448), (216.8, 448)
T_TL, T_TR = (402, 62), (478, 46)
sh = (H_BR[0] - H_TR[0]) / (H_BR[1] - H_TR[1]); st = (T_TR[0] - H_BR[0]) / (H_BR[1] - T_TR[1])
jy = (T_TL[0] + T_TL[1] * st - H_TR[0] + H_TR[1] * sh) / (sh + st); J = (H_TR[0] + (jy - H_TR[1]) * sh, jy)
V_LET = ('<g transform="translate(-11 10)"><path fill="{b}" d="' + rounded([T_TL, T_TR, H_BR, (H_BR[0] - 14, H_BR[1] - 2), (J[0] - 12, J[1] + 4)], [8, 12, 0, 0, 0]) + '"/>'
         '<path fill="{a}" d="' + rounded([H_TL, H_TR, H_BR, H_BL], [12, 14, 0, 10]) + '"/></g>')
E_LET = '<g transform="translate(-18 0)"><path fill="{a}" d="' + rounded(
    [(96, 48), (440, 48), (428, 120), (200, 120), (200, 220), (400, 220), (388, 292), (200, 292), (200, 392), (452, 392), (440, 464), (96, 464)],
    [14, 14, 14, 8, 8, 14, 14, 8, 8, 14, 14, 14]) + '"/></g>'

def _bb(pts, dx, dy):
    xs, ys = [q[0] + dx for q in pts], [q[1] + dy for q in pts]; return (min(xs), min(ys), max(xs), max(ys))
LET_BB = {'v': _bb([T_TL, T_TR, H_BR, H_TL, H_TR, H_BL], -11, 10),
          'e': _bb([(96, 48), (440, 48), (452, 392), (440, 464), (96, 464)], -18, 0)}

# ---- The FACE is a perfect square (no taper or skew) carrying the letter. The rounded body sits behind it with the
# requested L/R/B/T offsets. Its 2.5-unit left overhang is concealed under the 8-unit rim; only the right wall and
# lower band are visible. The visible body and its 8-unit rim share one rounded-rectangle silhouette.
S, FR = 416, 96                                  # face side and corner radius (23 % of the side)
def key(k, ol, orr, ob, shadow, rim=8, letter_scale=.5, shadow_dx=20, shadow_dy=12, shadow_blur=14):
    c = T[k]
    uw, uh = S + ol + orr, S + ob
    FX = round((512 - uw) / 2 + ol)                  # whole units so the square's edges land on pixels
    FY = round((512 - uh) / 2 - 4)
    BXl, BXr, BYb = FX - ol, FX + S + orr, FY + S + ob
    def face_r(e=0):
        return f'x="{FX - e}" y="{FY - e}" width="{S + 2 * e}" height="{S + 2 * e}" rx="{FR + e}"'
    def rounded_rect(x0, y0, x1, y1, r):
        return (f'M{x0 + r:.1f} {y0:.1f}H{x1 - r:.1f}A{r:.1f} {r:.1f} 0 0 1 {x1:.1f} {y0 + r:.1f}'
                f'V{y1 - r:.1f}A{r:.1f} {r:.1f} 0 0 1 {x1 - r:.1f} {y1:.1f}'
                f'H{x0 + r:.1f}A{r:.1f} {r:.1f} 0 0 1 {x0:.1f} {y1 - r:.1f}'
                f'V{y0 + r:.1f}A{r:.1f} {r:.1f} 0 0 1 {x0 + r:.1f} {y0:.1f}Z')
    # The full body keeps all requested offsets. Its left overhang is concealed under the rim; the visible core starts
    # at the face edge, leaving only the right wall and lower band exposed.
    def bd(e=0):
        return rounded_rect(BXl - e, FY - e, BXr + e, BYb + e, FR + e)
    def visible_body(e=0):
        return rounded_rect(FX - e, FY - e, BXr + e, BYb + e, FR + e)
    q = 1 - math.sqrt(.5)
    fbr = (FX + S - FR * q, FY + S - FR * q)
    bbr = (BXr - FR * q, BYb - FR * q)
    def ext(a, b, t): return (a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t)
    P = lambda pts: ' '.join(f'{x:.1f},{y:.1f}' for x, y in pts)
    p = [f'<defs><clipPath id="b{k}"><path d="{visible_body()}"/></clipPath>'
         f'<clipPath id="s{k}"><path d="{visible_body(rim)}"/></clipPath>'   # one complete outside silhouette
         f'<filter id="sh{k}" x="-20%" y="-80%" width="150%" height="260%"><feGaussianBlur stdDeviation="{shadow_blur}"/></filter></defs>']
    if shadow:
        # Drawn before the opaque cap: it can only be seen on the desk, never through the key itself.
        p.append(f'<ellipse cx="{(BXl + BXr) / 2 + shadow_dx}" cy="{BYb + shadow_dy}" rx="{(BXr - BXl) / 2 - 36}" ry="10" fill="#1C1C1A" opacity="{shadow}" filter="url(#sh{k})"/>')
    k_ = [f'<g clip-path="url(#s{k})">']                 # all key paint is clipped to one silhouette
    k_.append(f'<path d="{bd()}" fill="{c["band"]}" clip-path="url(#b{k})"/>')
    g = []
    if orr:
        g.append(f'<polygon fill="{c["wall"]}" points="{P([(FX + S - FR, FY - 20), (BXr + 20, FY - 20), (BXr + 20, BYb + 20), ext(fbr, bbr, 1), fbr])}"/>')
    if g:
        k_.append(f'<g clip-path="url(#b{k})">{"".join(g)}</g>')
    k_.append(f'<rect {face_r()} fill="{c["face"]}"/>')   # 3) face
    if rim:  # true 8-unit ring at the outside, with no inner stroke or bevel
        k_.append(f'<path d="{visible_body(rim)} {visible_body()}" fill-rule="evenodd" fill="{c["rim"]}"/>')
    # The only face relief is this 3-unit, 8%-opaque body-colour shade.  It is deliberately dark, never a highlight.
    k_.append(f'<line x1="{FX + FR}" y1="{FY + 1.25}" x2="{FX + S - FR}" y2="{FY + 1.25}" '
              f'stroke="{c["band"]}" stroke-opacity=".08" stroke-width="2.5"/>')
    letter = (V_LET if k == 'v' else E_LET).format(a=c['a'], b=c['b'])
    gx0, gy0, gx1, gy1 = LET_BB[k]
    fcx, fcy = FX + S / 2, FY + S / 2
    k_.append(f'<g transform="translate({fcx - letter_scale * (gx0 + gx1) / 2:.1f} {fcy - letter_scale * (gy0 + gy1) / 2:.1f}) scale({letter_scale})">{letter}</g>')
    p.append(''.join(k_) + '</g>')
    return f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" width="512" height="512">{"".join(p)}</svg>'

LADDER = {
    'full':    dict(ol=2.5, orr=30, ob=40, shadow=.18, shadow_dx=20, shadow_dy=12, shadow_blur=14),  # >= 192
    'reduced': dict(ol=1.5, orr=18, ob=24, shadow=.12, shadow_dx=12, shadow_dy=7, shadow_blur=8),     # 96-128 (~60 %)
    'minimal': dict(ol=0, orr=0, ob=0, shadow=0, rim=8),                                              # 48-64: flat, no body
}

# ---- 16 px FLAT tile (redrawn): one rounded square, one face colour, 1 px rim, the letter. Nothing else (no band,
# no shade, no bevel, no shadow). Letter rows 3-12 (10 px), centred on the 16 px square (x 3-13 / 5-11, y 3-13).
# V heavy 2.6->2.3 px, thin 2 px; E 2 px stem and bars with 2 px counters.
def small(k):
    c = T[k]
    letter = ('<polygon fill="{b}" points="10.4,2 13,2 8.8,12 8.2,12 7.8,9.6"/><polygon fill="{a}" points="3,2 5.8,2 8.8,12 6.4,12"/>' if k == 'v'
              else '<polygon fill="{a}" points="5,2 11,2 11,4 7,4 7,6 10,6 10,8 7,8 7,10 11,10 11,12 5,12"/>').format(a=c['a'], b=c['b'])
    return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" width="16" height="16">'
            f'<rect x="0.5" y="0.5" width="15" height="15" rx="3.5" fill="{c["face"]}"/>'
            f'<g transform="translate(0 1)">{letter}</g>'
            f'<rect x="0.5" y="0.5" width="15" height="15" rx="3.5" fill="none" stroke="{c["rim"]}" stroke-width="1"/></svg>')

MEAS = {}
def measure(png, k, tag):
    """Measure the rendered face, letter centering and exposed body overhangs."""
    im = Image.open(png).convert('RGBA'); px = im.load(); W, H = im.size
    rgb = lambda h: tuple(int(h[i:i + 2], 16) for i in (1, 3, 5))
    face = rgb(T[k]['face']); body = [rgb(T[k]['wall']), rgb(T[k]['band'])]
    letters = [rgb(T[k]['a']), rgb(T[k]['b'])]
    near = lambda p, t, tol=8: max(abs(p[i] - t[i]) for i in range(3)) <= tol
    xs, ys = [], []
    for yy in range(H):
        for xx in range(W):
            p = px[xx, yy]
            if p[3] > 250 and near(p, face, 8): xs.append(xx); ys.append(yy)
    x0, x1, y0, y1 = min(xs), max(xs) + 1, min(ys), max(ys) + 1
    w, h = x1 - x0, y1 - y0
    cx, cy = (x0 + x1 - 1) / 2, (y0 + y1 - 1) / 2
    lx, ly = [], []
    for yy in range(H):
        for xx in range(W):
            p = px[xx, yy]
            if p[3] > 200 and any(near(p, t, 4) for t in letters): lx.append(xx); ly.append(yy)
    dcx = ((min(lx) + max(lx)) / 2) - cx
    dcy = ((min(ly) + max(ly)) / 2) - cy
    body_xy = [(xx, yy) for yy in range(H) for xx in range(W)
               if px[xx, yy][3] > 200 and any(near(px[xx, yy], t) for t in body)]
    right = max(xx for xx, yy in body_xy if abs(yy - cy) <= 1) - (x1 - 1)
    bottom = max(yy for xx, yy in body_xy if abs(xx - cx) <= 1) - (y1 - 1)
    f = 420 / W
    ratio = w / h
    cap = (f'face {w} × {h} = {ratio:.3f}; body overhang right {right * f:.1f} px, '
           f'bottom {bottom * f:.1f} px @420; letter centre Δ({dcx:.1f}, {dcy:.1f}) units')
    return {tag: f'{w} × {h} = {ratio:.3f}', tag + '_cap': cap,
            tag + '_ratio': ratio, tag + '_letter_delta': (dcx, dcy),
            tag + '_overhang_420': (right * f, bottom * f)}

def build():
    OUT.mkdir(parents=True, exist_ok=True)
    for k in 've':
        (OUT / f'{k}-master.svg').write_text(key(k, **LADDER['full']), encoding='utf-8')
        (OUT / f'_{k}-reduced.svg').write_text(key(k, **LADDER['reduced']), encoding='utf-8')
        (OUT / f'_{k}-minimal.svg').write_text(key(k, **LADDER['minimal']), encoding='utf-8')
        (OUT / f'{k}-16.svg').write_text(small(k), encoding='utf-8')
    jobs = []
    for k in 've':
        jobs += [(f'{k}-{s}.png', f'{k}-master.svg', s) for s in (512, 256, 192)]
        jobs += [(f'{k}-{s}.png', f'_{k}-reduced.svg', s) for s in (128, 96)]
        jobs += [(f'{k}-{s}.png', f'_{k}-minimal.svg', s) for s in (64, 48)]
        jobs += [(f'{k}-{s}.png', f'{k}-16.svg', s) for s in (32, 20, 16)]
    jobs.append(('_m-taper-v.png', '_cmp-taper-v.svg', 512))
    x, y, rowh, pos = 0, 0, 0, []
    for fn, src, s in jobs:
        if x + s > 1600: x, y, rowh = 0, y + rowh + 8, 0
        pos.append((fn, src, x, y, s)); x += s + 8; rowh = max(rowh, s)
    page, shot = OUT / '_r.html', OUT / '_r.png'
    page.write_text('<!doctype html><html><body style="margin:0;background:transparent">' + ''.join(
        f'<img src="{src}" style="position:absolute;left:{px}px;top:{py}px;width:{s}px;height:{s}px">' for _, src, px, py, s in pos)
        + '</body></html>', encoding='utf-8')
    subprocess.run([EDGE, '--headless=new', '--disable-gpu', '--force-device-scale-factor=1', '--hide-scrollbars',
                    '--default-background-color=00000000', f'--window-size=1610,{y + rowh + 16}', f'--screenshot={shot}', page.as_uri()], capture_output=True)
    im = Image.open(shot).convert('RGBA')
    for fn, _, px, py, s in pos:
        im.crop((px, py, px + s, py + s)).save(OUT / fn)
    page.unlink(); shot.unlink()
    for tag, fn, k in (('v', 'v-512.png', 'v'), ('e', 'e-512.png', 'e')):
        MEAS.update(measure(OUT / fn, k, tag))
    (OUT / '_m-taper-v.png').unlink()
    ratios = []
    for k, c in T.items():
        ratios.append(f'| {k.upper()} | rim `{c["rim"]}` | {cr(c["rim"], "#F3F3F3"):.2f} | {cr(c["rim"], "#202020"):.2f} | {cr(c["rim"], "#808080"):.2f} |')
        ratios.append(f'| {k.upper()} | face `{c["face"]}` | {cr(c["face"], "#F3F3F3"):.2f} | {cr(c["face"], "#202020"):.2f} | {cr(c["face"], "#808080"):.2f} |')
    letters = [f'| V pink `{T["v"]["a"]}` | {cr(T["v"]["a"], T["v"]["face"]):.2f} |', f'| V orange `{T["v"]["b"]}` | {cr(T["v"]["b"], T["v"]["face"]):.2f} |',
               f'| E blue `{T["e"]["a"]}` | {cr(T["e"]["a"], T["e"]["face"]):.2f} |']
    (OUT / '_notes.md').write_text(f'''# Logo v11 — matte keycap

| Tile | Face | Right wall | Bottom band | Rim | Letter |
|---|---|---|---|---|---|
| V | `{T["v"]["face"]}` | `{T["v"]["wall"]}` | `{T["v"]["band"]}` | `{T["v"]["rim"]}` | pink `{T["v"]["a"]}` + orange `{T["v"]["b"]}` |
| E | `{T["e"]["face"]}` | `{T["e"]["wall"]}` | `{T["e"]["band"]}` | `{T["e"]["rim"]}` | `{T["e"]["a"]}` |

## Full-size geometry (512 units)

The face is an exact 416 × 416 rounded square with radius 96. Its letter is centred on the face. The body uses the
same rounded-square construction and offsets left / right / bottom / top by 2.5 / 30 / 40 / 0 units. The left
overhang is hidden beneath the rim, leaving only the right wall and bottom band visible. One 8-unit rim follows the
outside silhouette. No bevel or glossy highlight is used. A 2.5-unit inner top-edge shade uses the body-band colour at
8% opacity. The desk shadow uses 14-unit blur, 20 / 12-unit offset and 18% opacity, below the opaque body.

Rendered 512 px measurements:

| Tile | Face bounds | Ratio | Letter centre offset | Body overhang at 420 px (right / bottom) |
|---|---:|---:|---:|---:|
| V | {MEAS['v']} | {MEAS['v_ratio']:.3f} | {MEAS['v_letter_delta'][0]:.1f}, {MEAS['v_letter_delta'][1]:.1f} units | {MEAS['v_overhang_420'][0]:.1f}, {MEAS['v_overhang_420'][1]:.1f} px |
| E | {MEAS['e']} | {MEAS['e_ratio']:.3f} | {MEAS['e_letter_delta'][0]:.1f}, {MEAS['e_letter_delta'][1]:.1f} units | {MEAS['e_overhang_420'][0]:.1f}, {MEAS['e_overhang_420'][1]:.1f} px |

## Flat 16 / 20 / 32 px drawings

These stay flat: one rounded-square face, 1 px rim and letter only. No body, band, shade, bevel or shadow.

| Tile | Face | Rim | Letter |
|---|---|---|---|
| V | `{T["v"]["face"]}` | `{T["v"]["rim"]}` | pink `{T["v"]["a"]}` + orange `{T["v"]["b"]}` |
| E | `{T["e"]["face"]}` | `{T["e"]["rim"]}` | blue `{T["e"]["a"]}` |

## Contrast
| Tile | Element | vs #F3F3F3 | vs #202020 | vs #808080 |
|---|---|---:|---:|---:|
{chr(10).join(ratios)}

| Letter | vs face |
|---|---:|
{chr(10).join(letters)}
''', encoding='utf-8')

def sheet():
    lad = ''.join(f'<figure><div class="ck pad"><img src="keycap/v-{s}.png" width="{s}"><img src="keycap/e-{s}.png" width="{s}"></div><figcaption>{s} px · {lab}</figcaption></figure>'
                  for s, lab in ((192, 'đầy đủ'), (128, 'giảm'), (96, 'giảm'), (64, 'tối giản'), (48, 'tối giản'), (32, 'phẳng'), (20, 'phẳng'), (16, 'phẳng')))
    tray = lambda bg, lab: f'<figure><div class="tr" style="background:{bg}"><img class="px" src="keycap/v-16.png"><img class="px" src="keycap/e-16.png"></div><figcaption>16 px ×8 · {lab}</figcaption></figure>'
    bar = lambda bg, fg, lab: (f'<figure><div class="bar" style="background:{bg};color:{fg}"><img src="keycap/v-20.png"><b>VietKB</b><i></i><img src="keycap/e-20.png"><span>–</span><span>✕</span></div><figcaption>{lab}</figcaption></figure>')
    html = f'''<!doctype html><html lang="vi"><head><meta charset="utf-8"><title>VietKB — logo v11 keycap phối cảnh</title>
<style>
body{{margin:0;padding:18px 24px;background:#E7E4DE;font:400 13px/18px "Segoe UI Variable Text","Segoe UI",sans-serif;color:#1C1C1A;width:1552px}}
h1{{font:600 22px/30px "Segoe UI Variable Display","Segoe UI",sans-serif;margin:0 0 10px}}
.row{{display:flex;gap:14px;align-items:flex-end;margin-bottom:14px;flex-wrap:wrap}}
.ck{{background:repeating-conic-gradient(#FFFFFF 0 25%,#E9E9E9 0 50%) 0 0/24px 24px;display:flex;width:fit-content;border-radius:8px}}
.dk{{background:#202020;display:flex;width:fit-content;border-radius:8px}} .pad{{padding:6px;gap:8px}}
figure{{margin:0}} figcaption{{font:500 12px/16px "Segoe UI Variable Small","Segoe UI",sans-serif;color:#6B655B;margin-top:4px}}
.tr{{display:flex;gap:6px;padding:4px;border-radius:4px}} .px{{width:128px;height:128px;image-rendering:pixelated}}
.bar{{width:480px;height:36px;display:flex;align-items:center;gap:8px;padding-left:14px;border-radius:8px 8px 0 0;box-shadow:0 0 0 1px rgba(0,0,0,.08)}}
.bar b{{font:600 13px/20px "Segoe UI Variable Text","Segoe UI",sans-serif}} .bar i{{flex:1}} .bar span{{width:38px;text-align:center;opacity:.75;font-size:11px}}
</style></head><body>
<h1>VietKB · logo v11 — keycap phối cảnh</h1>
<div class="row"><figure><div class="ck"><img src="keycap/v-master.svg" width="420"></div><figcaption>V · 420 px<br>{MEAS["v_cap"]}</figcaption></figure>
<figure><div class="ck"><img src="keycap/e-master.svg" width="420"></div><figcaption>E · 420 px<br>{MEAS["e_cap"]}</figcaption></figure></div>
<div class="row"><figure><div class="dk"><img src="keycap/v-master.svg" width="420"><img src="keycap/e-master.svg" width="420"></div><figcaption>420 px · nền #202020</figcaption></figure></div>
<div class="row">{lad}</div>
<div class="row">{tray("#F3F3F3", "taskbar sáng")}{tray("#202020", "taskbar tối")}{tray("#808080", "xám giữa")}</div>
<div class="row">{bar("#F7F7F5", "#1C1C1A", "thanh tiêu đề 480 px, sáng")}{bar("#272727", "#FFFFFF", "thanh tiêu đề 480 px, tối")}</div>
</body></html>'''
    (HERE / 'index.html').write_text(html, encoding='utf-8')
    subprocess.run([EDGE, '--headless=new', '--disable-gpu', '--force-device-scale-factor=1', '--hide-scrollbars',
                    '--window-size=1600,1700', f'--screenshot={HERE / "sheet.png"}', (HERE / 'index.html').as_uri()], capture_output=True)

if __name__ == '__main__':
    build(); sheet(); print('ok')
