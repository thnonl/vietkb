# VietKB logo v7 — duotone, bolder transparent Ink mark. Heavy downstroke in hue A, thin upstroke in hue B,
# E solid in hue B. Light- and dark-taskbar variants. Run: python design/logo7/_build.py
import colorsys, pathlib, subprocess
from PIL import Image

HERE = pathlib.Path(__file__).resolve().parent
EDGE = r'C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe'
BG = {'tbl': '#F3F3F3', 'tbd': '#202020', 'wl': '#F7F7F5', 'wd': '#272727'}

# Construction at 16 px (scales linearly to 20/32/128):
#  heavy downstroke: polygon, horizontal width 3.9 at the top and 3.4 at the foot -> 3.6 -> 3.2 px measured perpendicular
#  thin upstroke: 2.4 px round-capped line drawn *under* the heavy stroke so the join is the heavy stroke's clean edge
#  interior of the V: >= 2.5 px open at mid-height (y = 8)
HEAVY_PTS = '1,1.5 4.9,1.5 9.2,14.5 5.8,14.5'
V = ('<path d="M8 13.6L14 1.9" fill="none" stroke="{b}" stroke-width="2.4" stroke-linecap="round"/>'
     f'<polygon fill="{{a}}" points="{HEAVY_PTS}"/>')
E = ('<polygon fill="{b}" points="2.5,1.3 5.7,1.3 5.7,14.7 2.5,14.7"/>'
     '<path d="M4.6 2.6Q8.8 1.9 13 2.2M4.6 8Q8 7.5 11.4 7.7M4.6 13.4Q8.8 12.9 13.4 12.8" fill="none" stroke="{b}" stroke-width="2.4" stroke-linecap="round"/>')
# previous (logo6) thin construction, for the weight comparison row
V_OLD = ('<polygon fill="{a}" points="1.5,1.5 4.6,1.5 8.6,14.6 6.4,14.6"/>'
         '<path d="M7.6 14.8L13.6 1.7" fill="none" stroke="{b}" stroke-width="1.9" stroke-linecap="round"/>')

def lum_rgb(rgb):
    c = [x / 12.92 if x <= .04045 else ((x + .055) / 1.055) ** 2.4 for x in rgb]
    return .2126 * c[0] + .7152 * c[1] + .0722 * c[2]
def hexs(rgb): return '#%02X%02X%02X' % tuple(round(x * 255) for x in rgb)
def lum(h): return lum_rgb([int(h[i:i + 2], 16) / 255 for i in (1, 3, 5)])
def cr(a, b):
    x, y = sorted((lum(a), lum(b)), reverse=True); return (x + .05) / (y + .05)
def tone(h, s, t):
    lo, hi = 0.0, 1.0
    for _ in range(40):
        m = (lo + hi) / 2
        if lum_rgb(colorsys.hls_to_rgb(h / 360, m, s)) < t: lo = m
        else: hi = m
    return hexs(colorsys.hls_to_rgb(h / 360, (lo + hi) / 2, s))

# name, logic, heavy (hue, sat), thin (hue, sat), thin light-target L, thin dark-target L
FAM = {
    'd1': ('Lạnh/Ấm', 'chàm đậm × đỏ son — lạnh đối ấm kinh điển', (230, .55), (4, .78), .17, .30),
    'd2': ('Rêu/Vàng', 'xanh rêu đậm × vàng nghệ — món ăn, vải dệt', (182, .65), (40, .92), .22, .42),
    'd3': ('Tím/Ngọc', 'tím đậm × ngọc — cặp lạnh nhưng rõ hai sắc', (262, .55), (165, .78), .17, .32),
    'd4': ('Đất/Trời', 'nâu đất × chàm sáng — đất đối trời', (16, .50), (222, .72), .17, .30),
    'd5': ('Mực/Nắng', 'mực xanh than × cam nắng — năng lượng cao nhất', (225, .60), (26, .92), .20, .38),
}
PAL = {}
for k, (name, logic, (hh, hs), (th, ts), tl, td) in FAM.items():
    PAL[k] = dict(name=name, logic=logic, light=(tone(hh, hs, .045), tone(th, ts, tl)), dark=(tone(hh, hs, .45), tone(th, ts, td)))

def svg16(body): return f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" width="16" height="16">{body}</svg>'

def numbers(p):
    (lh, lt), (dh, dt) = p['light'], p['dark']
    return [('nặng / taskbar sáng', cr(lh, BG['tbl'])), ('thanh / taskbar sáng', cr(lt, BG['tbl'])), ('E / taskbar sáng', cr(lt, BG['tbl'])),
            ('nặng / taskbar tối', cr(dh, BG['tbd'])), ('thanh / taskbar tối', cr(dt, BG['tbd'])), ('E / taskbar tối', cr(dt, BG['tbd'])),
            ('nặng / #F7F7F5', cr(lh, BG['wl'])), ('thanh / #F7F7F5', cr(lt, BG['wl'])),
            ('nặng / #272727', cr(dh, BG['wd'])), ('thanh / #272727', cr(dt, BG['wd'])),
            ('nặng ↔ thanh (bản sáng, chỉ tham khảo)', cr(lh, lt)), ('nặng ↔ thanh (bản tối, chỉ tham khảo)', cr(dh, dt))]

def build():
    nums = {}
    for k, p in PAL.items():
        d = HERE / k
        d.mkdir(exist_ok=True)
        (lh, lt), (dh, dt) = p['light'], p['dark']
        files = {'v.svg': V.format(a=lh, b=lt), 'e.svg': E.format(b=lt), 'v-dark.svg': V.format(a=dh, b=dt), 'e-dark.svg': E.format(b=dt),
                 '_old.svg': V_OLD.format(a=lh, b=lt), '_old-dark.svg': V_OLD.format(a=dh, b=dt)}
        for fn, body in files.items():
            (d / fn).write_text(svg16(body), encoding='utf-8')
        n = numbers(p); nums[k] = n
        fails = [t for t, v in n if 'tham khảo' not in t and v < 3]
        (d / 'notes.md').write_text(
            f'# {k} — {p["name"]}\n\n{p["logic"]}.\n\nTaskbar sáng: nét nặng `{lh}`, nét thanh và E `{lt}`.  \n'
            f'Taskbar tối: nét nặng `{dh}`, nét thanh và E `{dt}`.  \n'
            f'Nét ở 16 px: nặng 3.6 → 3.2 px (vuông góc), thanh 2.4 px; lòng chữ V mở 2.5 px ở giữa chiều cao.\n\n'
            '| Cặp | Tỉ lệ |\n|---|---|\n' + '\n'.join(f'| {t} | {v:.2f} |' for t, v in n) +
            f'\n\nDưới 3:1: {", ".join(fails) if fails else "không có"}.\n', encoding='utf-8')
        jobs = []
        for l in 've':
            for sz in (16, 20, 32):
                jobs += [(f'{l}-{sz}.png', f'{l}.svg', sz), (f'{l}-{sz}-dark.png', f'{l}-dark.svg', sz)]
        jobs += [('_old-16.png', '_old.svg', 16), ('_old-16-dark.png', '_old-dark.svg', 16)]
        x, pos = 0, []
        for fn, src, sz in jobs:
            pos.append((fn, src, x, sz)); x += sz + 8
        strip = lambda top, bg, fg, src: (f'<div style="position:absolute;left:0;top:{top}px;width:240px;height:36px;background:{bg};display:flex;align-items:center;gap:8px;'
                                          f'padding-left:12px;font:600 13px/20px Segoe UI Variable Text,Segoe UI,sans-serif;color:{fg}"><img src="{src}" width="20" height="20">VietKB</div>')
        page, shot = d / '_r.html', d / '_r.png'
        page.write_text('<!doctype html><html><body style="margin:0;background:transparent">' + ''.join(
            f'<img src="{src}" style="position:absolute;left:{px}px;top:0;width:{sz}px;height:{sz}px">' for _, src, px, sz in pos)
            + strip(40, BG['wl'], '#1C1C1A', 'v.svg') + strip(80, BG['wd'], '#FFFFFF', 'v-dark.svg') + '</body></html>', encoding='utf-8')
        subprocess.run([EDGE, '--headless=new', '--disable-gpu', '--force-device-scale-factor=1', '--hide-scrollbars',
                        '--default-background-color=00000000', f'--window-size={max(x, 260) + 8},124', f'--screenshot={shot}', page.as_uri()], capture_output=True)
        im = Image.open(shot).convert('RGBA')
        for fn, _, px, sz in pos:
            im.crop((px, 0, px + sz, sz)).save(d / fn)
        im.crop((0, 40, 240, 76)).save(d / 'titlebar-20.png'); im.crop((0, 80, 240, 116)).save(d / 'titlebar-20-dark.png')
        page.unlink(); shot.unlink()
    return nums

def sheet(nums):
    grey = ''.join(f'<div class="g"><b>{p["name"]}</b><div class="gr"><img class="x2" src="{k}/v-16.png"><img class="x2" src="{k}/e-16.png"><img class="x2" src="{k}/v-16-dark.png"><img class="x2" src="{k}/e-16-dark.png"></div>'
                   f'<div class="gr"><img src="{k}/v-32.png"><img src="{k}/v-32-dark.png"><img src="{k}/v-16.png"><img src="{k}/v-16-dark.png"></div></div>' for k, p in PAL.items())
    cmp_ = ''.join(f'<div class="cm"><b>{p["name"]}</b><div class="gr l"><img class="x4" src="{k}/_old-16.png"><img class="x4" src="{k}/v-16.png"><img src="{k}/_old-16.png"><img src="{k}/v-16.png"></div>'
                   f'<div class="gr d"><img class="x4" src="{k}/_old-16-dark.png"><img class="x4" src="{k}/v-16-dark.png"><img src="{k}/_old-16-dark.png"><img src="{k}/v-16-dark.png"></div></div>' for k, p in PAL.items())
    rows = []
    for k, p in PAL.items():
        (lh, lt), (dh, dt) = p['light'], p['dark']
        n = dict(nums[k])
        keys = ['nặng / taskbar sáng', 'thanh / taskbar sáng', 'nặng / taskbar tối', 'thanh / taskbar tối', 'nặng / #F7F7F5', 'thanh / #F7F7F5', 'nặng / #272727', 'thanh / #272727']
        cells = ''.join(f'<span class="{"bad" if n[t] < 3 else ""}">{t.replace("taskbar ", "tb ")} <b>{n[t]:.2f}</b></span>' for t in keys)
        sw = lambda c: f'<i style="background:{c}"></i><code>{c}</code>'
        rows.append(f'''<section><h2><span class="id">{k[1]}</span>{p["name"]} <small>{p["logic"]}</small>
<span class="pal">sáng {sw(lh)}{sw(lt)} · tối {sw(dh)}{sw(dt)}</span></h2>
<div class="r"><div class="tb l"><figure><img class="px" src="{k}/v-16.png"><figcaption>V · bật</figcaption></figure><figure><img class="px" src="{k}/e-16.png"><figcaption>E · tắt</figcaption></figure></div>
<div class="tb d"><figure><img class="px" src="{k}/v-16-dark.png"><figcaption>V · bật</figcaption></figure><figure><img class="px" src="{k}/e-16-dark.png"><figcaption>E · tắt</figcaption></figure></div>
<div class="col"><div class="sz l"><img src="{k}/v-20.png"><img src="{k}/v-32.png"><img src="{k}/e-20.png"><img src="{k}/e-32.png"></div>
<div class="sz d"><img src="{k}/v-20-dark.png"><img src="{k}/v-32-dark.png"><img src="{k}/e-20-dark.png"><img src="{k}/e-32-dark.png"></div>
<img src="{k}/titlebar-20.png" width="240"><img src="{k}/titlebar-20-dark.png" width="240"><div class="nums">{cells}</div></div></div></section>''')
    html = f'''<!doctype html><html lang="vi"><head><meta charset="utf-8"><title>VietKB — logo Ink hai sắc, nét đậm (v7)</title>
<style>
body{{margin:0;padding:14px 24px;background:#E7E4DE;font:400 12px/17px "Segoe UI Variable Text","Segoe UI",sans-serif;color:#1C1C1A;width:1052px}}
h1{{font:600 20px/26px "Segoe UI Variable Display","Segoe UI",sans-serif;margin:0}} .lead{{margin:2px 0 5px;color:#57524A}}
.grey{{background:#808080;border-radius:10px;padding:6px 12px;display:flex;gap:12px;color:#fff}} .g,.cm{{flex:1}} .g b,.cm b{{font:600 12px/16px "Segoe UI Variable Text","Segoe UI",sans-serif}}
.gr{{display:flex;gap:6px;align-items:center;margin-top:3px}} .x2{{width:32px;height:32px;image-rendering:pixelated}} .x4{{width:64px;height:64px;image-rendering:pixelated}}
.cmp{{display:flex;gap:10px;background:#FAF9F6;border-radius:10px;box-shadow:0 0 0 1px #E0DBD2;padding:6px 12px;margin-top:6px}}
.cm .gr{{padding:3px 6px;border-radius:6px}} .gr.l{{background:#F3F3F3}} .gr.d{{background:#202020}}
section{{background:#FAF9F6;border-radius:10px;box-shadow:0 0 0 1px #E0DBD2;padding:6px 12px 8px;margin-top:7px}}
h2{{margin:0 0 5px;font:600 14px/20px "Segoe UI Variable Text","Segoe UI",sans-serif;display:flex;gap:10px;align-items:center}} h2 small{{font-weight:400;color:#6B655B;font-size:12px}}
.id{{display:inline-grid;place-items:center;width:20px;height:20px;border-radius:5px;background:#1C1C1A;color:#fff;font-size:11px}}
.pal{{display:flex;gap:4px;align-items:center;font-weight:400;color:#57524A;font-size:12px;margin-left:auto}} .pal i{{width:13px;height:13px;border-radius:3px;display:inline-block}}
.pal code{{font:500 11px/14px "Cascadia Mono",Consolas,monospace;margin-right:5px}}
.r{{display:flex;gap:10px;align-items:flex-start}} .tb{{display:flex;gap:6px;padding:5px 6px 1px;border-radius:8px}} .tb.l{{background:#F3F3F3}} .tb.d{{background:#202020;color:#C5C5C5}}
figure{{margin:0;text-align:center}} figcaption{{font:500 11px/14px "Segoe UI Variable Small","Segoe UI",sans-serif;margin:1px 0 2px}}
.px{{width:128px;height:128px;image-rendering:pixelated;display:block}}
.col{{display:flex;flex-direction:column;gap:4px}} .sz{{display:flex;gap:10px;align-items:center;padding:3px 8px;border-radius:6px;width:240px}} .sz.l{{background:#F3F3F3}} .sz.d{{background:#202020}}
.nums{{display:grid;grid-template-columns:1fr 1fr;gap:0 8px;font:400 11px/14px "Segoe UI Variable Small","Segoe UI",sans-serif;color:#57524A;width:250px}}
.nums b{{color:#1C1C1A;font-variant-numeric:tabular-nums}} .nums .bad,.nums .bad b{{color:#B42318}}
</style></head><body>
<h1>VietKB · logo Ink hai sắc, nét đậm hơn — 5 bộ màu</h1>
<p class="lead">Nét nặng sắc A, nét thanh sắc B; E đặc sắc B. Ở 16 px: nét nặng 3.6→3.2 px, nét thanh 2.4 px, lòng V mở 2.5 px. Khay 16 px phóng 8 lần, pixel gốc.</p>
<div class="grey">{grey}</div>
<div class="cmp">{cmp_}</div>
<p class="lead" style="margin:3px 0 0">Trên: nền xám #808080 (xấu nhất cho dấu trong suốt) · Dưới: bản mảnh v6 (trái) và bản đậm v7 (phải), ×4 rồi 16 px thật.</p>
{"".join(rows)}
</body></html>'''
    (HERE / 'index.html').write_text(html, encoding='utf-8')
    subprocess.run([EDGE, '--headless=new', '--disable-gpu', '--force-device-scale-factor=1', '--hide-scrollbars',
                    '--window-size=1100,1800', f'--screenshot={HERE / "sheet.png"}', (HERE / 'index.html').as_uri()], capture_output=True)

if __name__ == '__main__':
    sheet(build())
    for k, p in PAL.items():
        print(k, p['name'], p['light'], p['dark'])
