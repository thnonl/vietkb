# VietKB logo v10 — retro keycap tiles. V (bật): warm face, deep-orange rim, pink + orange V.
# E (tắt): light-blue face, deep-blue rim, blue E. 512 master (face + rim + lip + highlight + shadow),
# flat keycap for 48–128 px, separately drawn 16 px tile for 16/20/32. Run: python design/logo10/_build.py
import math, pathlib, subprocess
from PIL import Image

HERE = pathlib.Path(__file__).resolve().parent
OUT = HERE / 'keycap'
EDGE = r'C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe'
T = {
    'v': dict(face='#FFEBDD', lip='#F5C5A3', lipt='#F8D2B6', hi='#FFF8F3', rim='#C94810', a='#CF1A65', b='#CF5605'),   # a = heavy pink, b = thin orange
    'e': dict(face='#E0F1FD', lip='#A7D0F2', lipt='#B8D9F4', hi='#F4FAFE', rim='#1E72D2', a='#1C6CC6', b='#1C6CC6'),
}
BGS = ['#F3F3F3', '#202020', '#808080']

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

# ---- letter masters (unchanged construction from logo9: designed terminals, thin stroke under heavy, single-path E)
H_TL, H_TR, H_BR, H_BL = (56, 68), (184, 44), (318.4, 448), (216.8, 448)
T_TL, T_TR = (402, 62), (478, 46)
sh = (H_BR[0] - H_TR[0]) / (H_BR[1] - H_TR[1]); st = (T_TR[0] - H_BR[0]) / (H_BR[1] - T_TR[1])
jy = (T_TL[0] + T_TL[1] * st - H_TR[0] + H_TR[1] * sh) / (sh + st); J = (H_TR[0] + (jy - H_TR[1]) * sh, jy)
V_LET = ('<g transform="translate(-11 10)"><path fill="{b}" d="' + rounded([T_TL, T_TR, H_BR, (H_BR[0] - 14, H_BR[1] - 2), (J[0] - 12, J[1] + 4)], [8, 12, 0, 0, 0]) + '"/>'
         '<path fill="{a}" d="' + rounded([H_TL, H_TR, H_BR, H_BL], [12, 14, 0, 10]) + '"/></g>')
E_LET = '<g transform="translate(-18 0)"><path fill="{a}" d="' + rounded(
    [(96, 48), (440, 48), (428, 120), (200, 120), (200, 220), (400, 220), (388, 292), (200, 292), (200, 392), (452, 392), (440, 464), (96, 464)],
    [14, 14, 14, 8, 8, 14, 14, 8, 8, 14, 14, 14]) + '"/></g>'

# ---- tile geometry (512): flat face 456 x 420 (used 48-128 px), radius 88; rim 8 units; highlight 4 units.
FX, FY, FW, FH, R, LIP, RIM = 28, 18, 456, 420, 88, 26, 8
CX, CY = FX + FW / 2, FY + FH / 2
# full keycap: base 456 x 462 at y 14; face shifted up by K = 51 (11.0 % of the tile height) -> face 456 x 411.
BY, BH, K = 14, 462, 51
# v10c: tile 456 x 462 at y 14; interior divided at DIV = 83 % of the height (y 397.5 -> 398); lip 78 units,
# with one lighter tonal step in its top 26 units.
DIV, STEP = 398, 26
HLB = f'M{FX + 9} {BY + R}A{R - 9} {R - 9} 0 0 1 {FX + R} {BY + 9}H{FX + FW - R}A{R - 9} {R - 9} 0 0 1 {FX + FW - 9} {BY + R}'
HL = f'M{FX + 9} {FY + R}A{R - 9} {R - 9} 0 0 1 {FX + R} {FY + 9}H{FX + FW - R}A{R - 9} {R - 9} 0 0 1 {FX + FW - 9} {FY + R}'

def master(k, full):
    c = T[k]
    letter = (V_LET if k == 'v' else E_LET).format(a=c['a'], b=c['b'])
    parts = []
    if full:
        # ONE silhouette (a single rounded rectangle) divided inside by a straight horizontal edge at DIV:
        # face above, front lip below (filled to the outer contour via a clip, square inner corners, one tonal
        # step), the rim strokes only the outer silhouette, the highlight runs along the top edge only.
        sil = f'x="{FX}" y="{BY}" width="{FW}" height="{BH}" rx="{R}"'
        parts.append('<defs><filter id="s" x="-20%" y="-60%" width="140%" height="220%"><feGaussianBlur stdDeviation="8"/></filter>'
                     f'<clipPath id="t"><rect {sil}/></clipPath></defs>'
                     f'<ellipse cx="{CX}" cy="{BY + BH + 4}" rx="{FW / 2 - 40}" ry="9" fill="#1C1C1A" opacity=".26" filter="url(#s)"/>')
        parts.append(f'<rect {sil} fill="{c["face"]}"/>')
        parts.append(f'<g clip-path="url(#t)"><rect x="{FX - 10}" y="{DIV}" width="{FW + 20}" height="{BY + BH - DIV + 10}" fill="{c["lip"]}"/>'
                     f'<rect x="{FX - 10}" y="{DIV}" width="{FW + 20}" height="{STEP}" fill="{c["lipt"]}"/></g>')
        parts.append(f'<path d="{HLB}" fill="none" stroke="{c["hi"]}" stroke-width="4" stroke-linecap="round"/>')
        parts.append(f'<rect {sil} fill="none" stroke="{c["rim"]}" stroke-width="{RIM}"/>')
        cy = BY + (DIV - BY) / 2
    else:  # flat keycap for 48-128 px: face + rim only
        parts.append(f'<rect x="{FX}" y="{FY}" width="{FW}" height="{FH}" rx="{R}" fill="{c["face"]}" stroke="{c["rim"]}" stroke-width="{RIM + 4}"/>')
        cy = CY
    parts.append(f'<g transform="translate({CX} {cy}) scale(.56) translate(-256 -256)">{letter}</g>')
    return f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" width="512" height="512">{"".join(parts)}</svg>'

# ---- 16 px optical size: whole-pixel tile (rim 1 px = thickened from 0.3 px scaled), no lip, no highlight, no shadow;
# letter redrawn in a 10 px field: V heavy 2.7 -> 2.3 px, thin 2.2 px; E 2 px stem and bars, 2 px counters.
TILE16 = '<rect x="0.5" y="0.5" width="15" height="15" rx="3.5" fill="{face}" stroke="{rim}" stroke-width="1"/>'
V16 = '<polygon fill="{b}" points="10.6,3 13,3 9,13 8.4,13 8,10.5"/><polygon fill="{a}" points="3,3 5.8,3 9,13 6.6,13"/>'
E16 = '<polygon fill="{a}" points="5,3 11,3 11,5 7,5 7,7 10.5,7 10.5,9 7,9 7,11 11,11 11,13 5,13"/>'
def small(k):
    c = T[k]
    return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" width="16" height="16">{TILE16.format(**c)}'
            f'{(V16 if k == "v" else E16).format(a=c["a"], b=c["b"])}</svg>')

def build():
    OUT.mkdir(parents=True, exist_ok=True)
    for k in 've':
        (OUT / f'{k}-master.svg').write_text(master(k, True), encoding='utf-8')
        (OUT / f'_{k}-flat.svg').write_text(master(k, False), encoding='utf-8')
        (OUT / f'{k}-16.svg').write_text(small(k), encoding='utf-8')
    jobs = []
    for k in 've':
        jobs += [(f'{k}-{s}.png', f'{k}-master.svg', s) for s in (512, 256)]
        jobs += [(f'{k}-{s}.png', f'_{k}-flat.svg', s) for s in (128, 96, 64, 48)]
        jobs += [(f'{k}-{s}.png', f'{k}-16.svg', s) for s in (32, 20, 16)]
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
    lines = []
    for k, c in T.items():
        lines.append(f'| {k.upper()} | rim `{c["rim"]}` | ' + ' | '.join(f'{cr(c["rim"], b):.2f}' for b in BGS) + f' | {cr(c["rim"], c["face"]):.2f} |')
        lines.append(f'| {k.upper()} | face `{c["face"]}` | ' + ' | '.join(f'{cr(c["face"], b):.2f}' for b in BGS) + ' | — |')
    letters = [f'| V heavy (pink) `{T["v"]["a"]}` | {cr(T["v"]["a"], T["v"]["face"]):.2f} |', f'| V thin (orange) `{T["v"]["b"]}` | {cr(T["v"]["b"], T["v"]["face"]):.2f} |',
               f'| E (blue) `{T["e"]["a"]}` | {cr(T["e"]["a"], T["e"]["face"]):.2f} |']
    (OUT / '_notes.md').write_text(f'''# Logo v10 — retro keycap

| Tile | Face | Lip (2 steps darker, same hue) | Top highlight | Rim | Letter |
|---|---|---|---|---|---|
| V (bật) | `{T["v"]["face"]}` | `{T["v"]["lip"]}` | `{T["v"]["hi"]}` | `{T["v"]["rim"]}` | pink `{T["v"]["a"]}` (heavy) + orange `{T["v"]["b"]}` (thin) |
| E (tắt) | `{T["e"]["face"]}` | `{T["e"]["lip"]}` | `{T["e"]["hi"]}` | `{T["e"]["rim"]}` | blue `{T["e"]["a"]}` (rim hue, one step darker) |

No gradient anywhere (face, lip, rim, highlight are flat colours); the only soft element is the contact shadow below the tile.

## Contrast
| Tile | Element | vs #F3F3F3 | vs #202020 | vs #808080 | vs own face |
|---|---|---|---|---|---|
{chr(10).join(lines)}

| Letter stroke | vs tile face |
|---|---|
{chr(10).join(letters)}

- Rims clear 3:1 on both taskbars (≈ 4.3 / 3.4). Deep orange had to stay at relative luminance ≈ 0.17 for that: lighter fails on `#202020`, darker fails nothing but turns brown.
- V thin orange is 3.65:1 on the face (passes 3:1, not 4.5): reaching 4.5 would push it to the rim's rust and the two oranges would merge — kept as orange, noted as the weakest pair. Pink and blue clear 4.5.
- Mid-grey `#808080`: the rim fails (1.2) but the light face is 3.42:1 against it, so the tile still separates from mid-grey by its face — the opaque tile solves the case the transparent mark could not.
- **No dark variant**: the tile is opaque and its rim clears 3:1 on both taskbars, so one version serves light and dark.

## Sizes
- **512 / 256 (full keycap)**: ONE rounded rectangle 456 × 462 units (radius 88) is the whole key; its interior is divided by a straight horizontal edge at y 398 (83 % of the height): face above, front lip below (78 units: 64 px at 420, 29 px at 192) filled to the outer contour with square inner corners and one lighter tonal step (`lipt`) in its top 26 units; the 8-unit rim strokes the outer silhouette only; 4-unit highlight along the top inner edge; soft contact shadow (blur 9, 28 % ink) below the tile only. Letter at 56 % of the box.
- **48–128 (flat keycap)**: lip, highlight and shadow removed; rim thickened to 12 units so it stays ≥ 1.3 px at 48.
- **16 / 20 / 32 (redrawn 16 px tile)**: whole-pixel tile, radius 3.5, **rim 1 px** (a scaled rim would be 0.3 px), no lip, no highlight, no shadow; letter redrawn in a 10 px field — V heavy 2.7 → 2.3 px, thin 2.2 px, interior ~1.9 px at mid-height; E 2 px stem and bars with 2 px counters.
''', encoding='utf-8')

def sheet():
    sizes = ''.join(f'<figure><div class="ck pad"><img src="keycap/v-{s}.png" width="{s}"><img src="keycap/e-{s}.png" width="{s}"></div><figcaption>{s} px</figcaption></figure>'
                    for s in (64, 32, 20, 16))
    tray = lambda bg, lab: f'<figure><div class="tr" style="background:{bg}"><img class="px" src="keycap/v-16.png"><img class="px" src="keycap/e-16.png"></div><figcaption>16 px ×8 · {lab}</figcaption></figure>'
    bar = lambda bg, fg, lab: (f'<figure><div class="bar" style="background:{bg};color:{fg}"><img src="keycap/v-20.png"><b>VietKB</b><i></i><img src="keycap/e-20.png"><span>tắt</span><span>–</span><span>✕</span></div>'
                               f'<figcaption>{lab}</figcaption></figure>')
    html = f'''<!doctype html><html lang="vi"><head><meta charset="utf-8"><title>VietKB — logo v10 keycap</title>
<style>
body{{margin:0;padding:18px 24px;background:#E7E4DE;font:400 13px/18px "Segoe UI Variable Text","Segoe UI",sans-serif;color:#1C1C1A;width:1552px}}
h1{{font:600 22px/30px "Segoe UI Variable Display","Segoe UI",sans-serif;margin:0 0 10px}}
.row{{display:flex;gap:14px;align-items:flex-end;margin-bottom:12px}}
.ck{{background:repeating-conic-gradient(#FFFFFF 0 25%,#E9E9E9 0 50%) 0 0/24px 24px;display:flex;width:fit-content;border-radius:8px}}
.dk{{background:#202020;display:flex;width:fit-content;border-radius:8px}} .pad{{padding:6px;gap:8px}}
figure{{margin:0}} figcaption{{font:500 12px/16px "Segoe UI Variable Small","Segoe UI",sans-serif;color:#6B655B;margin-top:4px}}
.tr{{display:flex;gap:10px;padding:10px;border-radius:8px}} .px{{width:128px;height:128px;image-rendering:pixelated}}
.bar{{width:480px;height:36px;display:flex;align-items:center;gap:8px;padding-left:14px;border-radius:8px 8px 0 0;box-shadow:0 0 0 1px rgba(0,0,0,.08)}}
.bar b{{font:600 13px/20px "Segoe UI Variable Text","Segoe UI",sans-serif}} .bar i{{flex:1}} .bar span{{width:38px;text-align:center;opacity:.75;font-size:11px}}
</style></head><body>
<h1>VietKB · logo v10 — keycap retro (V bật · E tắt)</h1>
<div class="row"><figure><div class="ck"><img src="keycap/_old-v-master.svg" width="420"></div><figcaption>1 · hai tờ giấy (hai đường viền lệch)</figcaption></figure>
<figure><div class="ck"><img src="keycap/_prev-v-master.svg" width="420"></div><figcaption>2 · lần trước: mặt + đế là hai hình bo góc chồng nhau</figcaption></figure>
<figure><div class="ck"><img src="keycap/v-master.svg" width="420"></div><figcaption>3 · MỚI: một đường viền ngoài, chia trong bằng một cạnh ngang ở 83 %</figcaption></figure></div>
<div class="row"><figure><div class="ck"><img src="keycap/v-master.svg" width="420"><img src="keycap/e-master.svg" width="420"></div><figcaption>420 px · nền bàn cờ</figcaption></figure>
<figure><div class="dk"><img src="keycap/v-master.svg" width="300"><img src="keycap/e-master.svg" width="300"></div><figcaption>300 px · nền #202020</figcaption></figure></div>
<div class="row"><figure><div class="dk"><img src="keycap/v-master.svg" width="420"><img src="keycap/e-master.svg" width="420"></div><figcaption>420 px · nền #202020</figcaption></figure>
<div><div class="row"><figure><div class="ck pad"><img src="keycap/v-master.svg" width="192"><img src="keycap/e-master.svg" width="192"></div><figcaption>192 px</figcaption></figure>
<figure><div class="ck pad"><img src="keycap/v-96.png"><img src="keycap/e-96.png"></div><figcaption>96 px (keycap phẳng)</figcaption></figure></div>
<div class="row">{sizes}</div></div></div>
<div class="row">{tray("#F3F3F3", "taskbar sáng #F3F3F3")}{tray("#202020", "taskbar tối #202020")}{tray("#808080", "xám giữa #808080")}</div>
<div class="row">{bar("#F7F7F5", "#1C1C1A", "thanh tiêu đề 480 px, sáng")}{bar("#272727", "#FFFFFF", "thanh tiêu đề 480 px, tối")}</div>
</body></html>'''
    (HERE / 'index.html').write_text(html, encoding='utf-8')
    subprocess.run([EDGE, '--headless=new', '--disable-gpu', '--force-device-scale-factor=1', '--hide-scrollbars',
                    '--window-size=1600,1980', f'--screenshot={HERE / "sheet.png"}', (HERE / 'index.html').as_uri()], capture_output=True)

if __name__ == '__main__':
    build(); sheet(); print('ok')
