# VietKB logo v8 — two masters per letter: a 512-unit large master and a separately drawn 16 px optical size.
# Run: python design/logo8/_build.py
import math, pathlib, subprocess
from PIL import Image

HERE = pathlib.Path(__file__).resolve().parent
EDGE = r'C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe'
PAL = {
    'd3': dict(name='Tím/Ngọc', light=('#492684', '#108265'), dark=('#BFA9E6', '#15AD87')),
    'd4': dict(name='Đất/Trời', light=('#5D301F', '#3B6CDF'), dark=('#DBA794', '#6F93E8')),
}

def rounded(pts, radii):
    """Closed path through pts with each corner rounded by its radius (quadratic, tangent-length r)."""
    n, out = len(pts), []
    for i in range(n):
        p0, p1, p2 = pts[i - 1], pts[i], pts[(i + 1) % n]
        r = radii[i]
        if r <= 0:
            out.append(('L', p1)); continue
        def toward(a, b, d):
            dx, dy = b[0] - a[0], b[1] - a[1]; L = math.hypot(dx, dy); d = min(d, L / 2)
            return (a[0] + dx / L * d, a[1] + dy / L * d)
        out.append(('L', toward(p1, p0, r))); out.append(('Q', p1, toward(p1, p2, r)))
    d = f'M{out[0][1][0]:.1f} {out[0][1][1]:.1f}'
    for seg in out[1:] + out[:1]:
        if seg[0] == 'L': d += f'L{seg[1][0]:.1f} {seg[1][1]:.1f}'
        else: d += f'Q{seg[1][0]:.1f} {seg[1][1]:.1f} {seg[2][0]:.1f} {seg[2][1]:.1f}'
    return d + 'Z'

# ---------------------------------------------------------------- 512 master
# Heavy downstroke: top cut rising to the right (brush entry, same angle on every terminal), tapering from
# ~113 to ~95 units perpendicular; its bottom-right corner IS the join point.
H_TL, H_TR, H_BR, H_BL = (56, 68), (184, 44), (318.4, 448), (216.8, 448)
# Thin upstroke: ~71 units perpendicular, same top cut; its right edge ends exactly at H_BR and its left
# edge ends exactly on the heavy stroke's right edge (computed), so nothing overshoots and nothing gaps.
T_TL, T_TR = (402, 62), (478, 46)
def thin_join():
    sh = (H_BR[0] - H_TR[0]) / (H_BR[1] - H_TR[1])           # heavy right edge dx/dy
    st = (T_TR[0] - H_BR[0]) / (H_BR[1] - T_TR[1])           # thin right edge dx/dy (going down-left)
    # thin left edge: x = T_TL.x - (y - T_TL.y)*st ; heavy right: x = H_TR.x + (y - H_TR.y)*sh
    y = (T_TL[0] + T_TL[1] * st - H_TR[0] + H_TR[1] * sh) / (sh + st)
    return (H_TR[0] + (y - H_TR[1]) * sh, y)
J = thin_join()
V_MASTER = ('<path fill="{b}" d="' + rounded([T_TL, T_TR, H_BR, (H_BR[0] - 14, H_BR[1] - 2), (J[0] - 12, J[1] + 4)], [14, 14, 0, 0, 0]) + '"/>'
            '<path fill="{a}" d="' + rounded([H_TL, H_TR, H_BR, H_BL], [14, 14, 0, 10]) + '"/>')
# E: stem 104 wide, bars 72 thick (= thin stroke), equal 100-unit counters, bar ends cut at the same angle as V tops.
E_PTS = [(96, 48), (440, 48), (428, 120), (200, 120), (200, 220), (400, 220), (388, 292), (200, 292),
         (200, 392), (452, 392), (440, 464), (96, 464)]
E_R = [14, 14, 14, 8, 8, 14, 14, 8, 8, 14, 14, 14]
E_MASTER = '<path fill="{b}" d="' + rounded(E_PTS, E_R) + '"/>'

# ---------------------------------------------------------------- 16 px optical size (redrawn, not scaled)
# Flat tops, no corner rounding, heavier: heavy 3.7 -> 3.1 px perpendicular, thin 2.5 px; interior >= 2 px at y=8.
V16 = ('<polygon fill="{b}" points="11.9,1.5 14.6,1.5 9.5,14.5 8.6,14.5 7.9,11"/>'
       '<polygon fill="{a}" points="1,1.5 5,1.5 9.5,14.5 6.2,14.5"/>')
E16 = '<polygon fill="{b}" points="2.5,1.5 13.2,1.5 13.2,4 5.8,4 5.8,6.75 12.2,6.75 12.2,9.25 5.8,9.25 5.8,12 13.5,12 13.5,14.5 2.5,14.5"/>'

def svg(vb, body): return f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {vb} {vb}" width="{vb}" height="{vb}">{body}</svg>'

def build():
    for k, p in PAL.items():
        d = HERE / k
        d.mkdir(exist_ok=True)
        for var, (a, b) in (('', p['light']), ('-dark', p['dark'])):
            (d / f'v-master{var}.svg').write_text(svg(512, V_MASTER.format(a=a, b=b)), encoding='utf-8')
            (d / f'e-master{var}.svg').write_text(svg(512, E_MASTER.format(b=b)), encoding='utf-8')
            (d / f'v-16{var}.svg').write_text(svg(16, V16.format(a=a, b=b)), encoding='utf-8')
            (d / f'e-16{var}.svg').write_text(svg(16, E16.format(b=b)), encoding='utf-8')
        jobs = []
        for l in 've':
            for var in ('', '-dark'):
                for sz in (512, 256, 128, 96, 64):
                    jobs.append((f'{l}-{sz}{var}.png', f'{l}-master{var}.svg', sz))
                for sz in (16, 20, 32):
                    jobs.append((f'{l}-{sz}{var}.png', f'{l}-16{var}.svg', sz))
        x, y, rowh, pos = 0, 0, 0, []
        for fn, src, sz in jobs:
            if x + sz > 1600:
                x, y, rowh = 0, y + rowh + 8, 0
            pos.append((fn, src, x, y, sz)); x += sz + 8; rowh = max(rowh, sz)
        page, shot = d / '_r.html', d / '_r.png'
        page.write_text('<!doctype html><html><body style="margin:0;background:transparent">' + ''.join(
            f'<img src="{src}" style="position:absolute;left:{px}px;top:{py}px;width:{sz}px;height:{sz}px">' for _, src, px, py, sz in pos)
            + '</body></html>', encoding='utf-8')
        subprocess.run([EDGE, '--headless=new', '--disable-gpu', '--force-device-scale-factor=1', '--hide-scrollbars',
                        '--default-background-color=00000000', f'--window-size=1610,{y + rowh + 16}', f'--screenshot={shot}', page.as_uri()],
                       capture_output=True)
        im = Image.open(shot).convert('RGBA')
        for fn, _, px, py, sz in pos:
            im.crop((px, py, px + sz, py + sz)).save(d / fn)
        page.unlink(); shot.unlink()
        (d / '_notes.md').write_text(f'''# {k} — {p["name"]} · master + optical size

Colours — light taskbar: heavy `{p["light"][0]}`, thin/E `{p["light"][1]}`; dark taskbar: heavy `{p["dark"][0]}`, thin/E `{p["dark"][1]}`.

**Master (512 units, used ≥ 64 px).**
- Heavy downstroke: 113 → 95 units perpendicular (taper), top cut rising to the right at ~11°, 14-unit corner radii, 10-unit radius on the foot's outer corner.
- Thin upstroke: ~71 units perpendicular (heavy : thin ≈ 1.45 : 1), same top cut and radii.
- Join: the thin stroke's right edge ends exactly at the heavy stroke's bottom-right corner ({H_BR[0]}, {H_BR[1]}), and its left edge ends on the heavy's right edge at ({J[0]:.1f}, {J[1]:.1f}). The thin shape sits under the heavy one with a 12-unit hidden overlap, so there is no gap, no seam and no cap below the point.
- E: 104-unit stem, three 72-unit bars (= thin stroke), two equal 100-unit counters, bar ends cut at the V's terminal angle; outer corners 14, inner 8. One single path, so the thickness is exactly even.

**16 px optical size (redrawn).**
- Thickened: heavy 3.7 → 3.1 px (master scaled would be 3.5 → 3.0), thin 2.5 px (scaled 2.2), E stem 3.3 px, bars 2.5 px.
- Removed: the slanted terminal cut and all corner radii (flat tops, square corners), because they need antialiasing to read.
- Adjusted join: the thin stroke's left edge meets the heavy stroke at y ≈ 10.9 instead of the scaled ≈ 11, keeping the interior ≥ 2 px open at mid-height.
- 20 px and 32 px are rendered from the 16 px drawing; a separate 20 grid was not needed, because the 16 drawing has no half-pixel features that misalign at 1.25×.

Stroke widths by size: 512 → 113/71 units · 256 → 56/35 px · 128 → 28/18 px · 64 → 14/9 px (master) · 32 → 7.4/5 px · 20 → 4.6/3.1 px · 16 → 3.7/2.5 px (optical size).
''', encoding='utf-8')

def sheet():
    def block(k, p):
        rows = ''.join(f'<figure><div class="ck"><img src="{k}/v-{s}.png" width="{s}" height="{s}"><img src="{k}/e-{s}.png" width="{s}" height="{s}"></div><figcaption>{s} px</figcaption></figure>'
                       for s in (96, 32, 20, 16))
        return f'''<div class="blk"><h2>{k} · {p["name"]} <code>{p["light"][0]} × {p["light"][1]}</code></h2>
<div class="big ck"><img src="{k}/v-master.svg" width="420" height="420"><img src="{k}/e-master.svg" width="420" height="420"></div>
<p class="cap">Bản gốc 512 hiển thị ở 420 px, nền bàn cờ</p>
<div class="mid"><figure><div class="ck"><img src="{k}/v-master.svg" width="192"><img src="{k}/e-master.svg" width="192"></div><figcaption>192 px</figcaption></figure>
<figure><div class="ckd"><img src="{k}/v-master-dark.svg" width="192"><img src="{k}/e-master-dark.svg" width="192"></div><figcaption>192 px · bản taskbar tối</figcaption></figure></div>
<div class="row">{rows}</div>
<div class="tray"><div class="tl"><img class="px" src="{k}/v-16.png"><img class="px" src="{k}/e-16.png"></div><div class="td"><img class="px" src="{k}/v-16-dark.png"><img class="px" src="{k}/e-16-dark.png"></div></div>
<p class="cap">Bản 16 px vẽ riêng, phóng 8 lần pixel gốc · taskbar sáng #F3F3F3 / tối #202020</p></div>'''
    html = f'''<!doctype html><html lang="vi"><head><meta charset="utf-8"><title>VietKB — logo v8, bản gốc + bản 16 px</title>
<style>
body{{margin:0;padding:20px 24px;background:#E7E4DE;font:400 13px/18px "Segoe UI Variable Text","Segoe UI",sans-serif;color:#1C1C1A;width:1552px}}
h1{{font:600 22px/30px "Segoe UI Variable Display","Segoe UI",sans-serif;margin:0 0 10px}}
.wrap{{display:flex;gap:16px}} .blk{{flex:1;background:#FAF9F6;border-radius:12px;box-shadow:0 0 0 1px #E0DBD2;padding:12px}}
h2{{margin:0 0 8px;font:600 15px/22px "Segoe UI Variable Text","Segoe UI",sans-serif}} h2 code{{font:500 12px/16px "Cascadia Mono",Consolas,monospace;color:#57524A;margin-left:8px}}
.ck{{background:repeating-conic-gradient(#FFFFFF 0 25%,#E9E9E9 0 50%) 0 0/24px 24px;display:flex;gap:0;width:fit-content}}
.ckd{{background:repeating-conic-gradient(#202020 0 25%,#2B2B2B 0 50%) 0 0/24px 24px;display:flex;width:fit-content}}
.big{{border-radius:8px;overflow:hidden}} .cap{{margin:4px 0 10px;color:#6B655B;font-size:12px}}
.mid{{display:flex;gap:12px}} .row{{display:flex;gap:14px;align-items:flex-end;margin-top:10px}}
.row .ck{{padding:6px;gap:8px}} figure{{margin:0}} figcaption{{font:500 11.5px/16px "Segoe UI Variable Small","Segoe UI",sans-serif;color:#6B655B;margin-top:3px}}
.tray{{display:flex;gap:10px;margin-top:12px}} .tl,.td{{display:flex;gap:8px;padding:8px;border-radius:8px}} .tl{{background:#F3F3F3}} .td{{background:#202020}}
.px{{width:128px;height:128px;image-rendering:pixelated}}
</style></head><body>
<h1>VietKB · logo v8 — bản gốc 512 + bản 16 px vẽ riêng (d3 · d4)</h1>
<div class="wrap">{block("d3", PAL["d3"])}{block("d4", PAL["d4"])}</div>
</body></html>'''
    (HERE / 'index.html').write_text(html, encoding='utf-8')
    subprocess.run([EDGE, '--headless=new', '--disable-gpu', '--force-device-scale-factor=1', '--hide-scrollbars',
                    '--window-size=1600,1500', f'--screenshot={HERE / "sheet.png"}', (HERE / 'index.html').as_uri()], capture_output=True)

if __name__ == '__main__':
    build(); sheet(); print('join', J)
