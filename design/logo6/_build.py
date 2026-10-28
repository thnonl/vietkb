# VietKB logo v6 — transparent Ink mark. Construction from logo4/5 C (heavy downstroke + 1.9 px upstroke), no tile,
# no black: both strokes coloured. Two taskbar variants per palette. E = whole letter in the accent (thin-stroke) colour.
# Run: python design/logo6/_build.py
import colorsys, pathlib, subprocess
from PIL import Image

HERE = pathlib.Path(__file__).resolve().parent
EDGE = r'C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe'
BG = {'tb-light': '#F3F3F3', 'tb-dark': '#202020', 'win-light': '#F7F7F5', 'win-dark': '#272727', 'grey': '#808080'}

C_V = ('<polygon fill="{a}" points="1.5,1.5 4.6,1.5 8.6,14.6 6.4,14.6"/>'
       '<path d="M7.6 14.8L13.6 1.7" fill="none" stroke="{b}" stroke-width="1.9" stroke-linecap="round"/>')
C_E = ('<polygon fill="{a}" points="3,1.4 5.4,1.4 5.4,14.6 3,14.6"/>'
       '<path d="M4.4 2.4Q8.5 1.6 12.6 1.9M4.4 8Q7.8 7.4 11 7.6M4.4 13.6Q8.6 13 13.2 12.9" fill="none" stroke="{b}" stroke-width="1.9" stroke-linecap="round"/>')

def lum_rgb(rgb):
    c = [x / 12.92 if x <= .04045 else ((x + .055) / 1.055) ** 2.4 for x in rgb]
    return .2126 * c[0] + .7152 * c[1] + .0722 * c[2]
def hexs(rgb):
    return '#%02X%02X%02X' % tuple(round(x * 255) for x in rgb)
def lum(h):
    return lum_rgb([int(h[i:i + 2], 16) / 255 for i in (1, 3, 5)])
def cr(a, b):
    x, y = sorted((lum(a), lum(b)), reverse=True)
    return (x + .05) / (y + .05)
def tone(h, s, target):
    lo, hi = 0.0, 1.0
    for _ in range(40):
        m = (lo + hi) / 2
        if lum_rgb(colorsys.hls_to_rgb(h / 360, m, s)) < target: lo = m
        else: hi = m
    return hexs(colorsys.hls_to_rgb(h / 360, (lo + hi) / 2, s))

# Rule: light-taskbar variant = heavy at L≈0.05, thin at L≈0.17 (both ≥ 3:1 on #F3F3F3, two clearly different tones);
#       dark-taskbar variant  = heavy at L≈0.55, thin at L≈0.27 (both ≥ 3:1 on #202020), same two hues lightened.
FAM = {  # name, heavy hue/sat, thin hue/sat, reason
    'p1': ('Mực tím', (258, .55), (250, .75), 'mực tím của giao diện, chỉnh độ sáng cho đạt'),
    'p2': ('Mực xanh ngọc', (178, .70), (168, .80), 'màu nước Việt'),
    'p3': ('Mực đỏ son', (2, .65), (8, .80), 'son mài, dấu đỏ'),
    'p4': ('Mực chàm', (232, .55), (224, .70), 'chàm nhuộm vải vùng cao'),
    'p5': ('Mực nâu đất', (14, .50), (20, .65), 'đất nung, ngói'),
}
PAL = {}
for k, (name, (hh, hs), (th, ts), why) in FAM.items():
    PAL[k] = dict(name=name, why=why,
                  light=(tone(hh, hs, .05), tone(th, ts, .17)),
                  dark=(tone(hh, hs, .55), tone(th, ts, .27)))

def svg16(body):
    return f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" width="16" height="16">{body}</svg>'

def numbers(p):
    (lh, lt), (dh, dt) = p['light'], p['dark']
    n = [('nặng / taskbar sáng', cr(lh, BG['tb-light'])), ('thanh / taskbar sáng', cr(lt, BG['tb-light'])),
         ('nặng(tối) / taskbar tối', cr(dh, BG['tb-dark'])), ('thanh(tối) / taskbar tối', cr(dt, BG['tb-dark'])),
         ('E / taskbar sáng', cr(lt, BG['tb-light'])), ('E(tối) / taskbar tối', cr(dt, BG['tb-dark'])),
         ('nặng / cửa sổ #F7F7F5', cr(lh, BG['win-light'])), ('thanh / cửa sổ #F7F7F5', cr(lt, BG['win-light'])),
         ('nặng(tối) / cửa sổ #272727', cr(dh, BG['win-dark'])), ('thanh(tối) / cửa sổ #272727', cr(dt, BG['win-dark'])),
         ('— chéo: nặng sáng / taskbar tối', cr(lh, BG['tb-dark'])), ('— chéo: nặng tối / taskbar sáng', cr(dh, BG['tb-light']))]
    return n

def write_render():
    nums = {}
    for k, p in PAL.items():
        d = HERE / k
        d.mkdir(exist_ok=True)
        (lh, lt), (dh, dt) = p['light'], p['dark']
        files = {'v.svg': svg16(C_V.format(a=lh, b=lt)), 'e.svg': svg16(C_E.format(a=lt, b=lt)),
                 'v-dark.svg': svg16(C_V.format(a=dh, b=dt)), 'e-dark.svg': svg16(C_E.format(a=dt, b=dt))}
        for fn, s in files.items():
            (d / fn).write_text(s, encoding='utf-8')
        n = numbers(p)
        nums[k] = n
        fails = [t for t, v in n if not t.startswith('—') and v < 3]
        table = '\n'.join(f'| {t} | {v:.2f} |' for t, v in n)
        (d / 'notes.md').write_text(
            f'# {k} — {p["name"]}\n\n{p["why"]}.\n\nTaskbar sáng: nét nặng `{lh}`, nét thanh `{lt}`, chữ E `{lt}`.  \n'
            f'Taskbar tối: nét nặng `{dh}`, nét thanh `{dt}`, chữ E `{dt}`.  \nNền trong suốt, không có ô.\n\n'
            f'| Cặp | Tỉ lệ |\n|---|---|\n{table}\n\nDưới 3:1 (khay 16 px): {", ".join(fails) if fails else "không có"}.\n'
            f'Hàng “chéo” cho thấy vì sao cần hai biến thể: nét nặng của bản sáng biến mất trên taskbar tối và ngược lại.\n',
            encoding='utf-8')
        jobs = []
        for l in 've':
            for sz in (16, 20, 32):
                jobs += [(f'{l}-{sz}.png', f'{l}.svg', sz), (f'{l}-{sz}-dark.png', f'{l}-dark.svg', sz)]
        x, pos = 0, []
        for fn, src, sz in jobs:
            pos.append((fn, src, x, sz)); x += sz + 8
        # title-bar strips (mark at 20 px beside the word, on the real window colours)
        strips = (f'<div style="position:absolute;left:0;top:40px;width:240px;height:36px;background:{BG["win-light"]};display:flex;align-items:center;gap:8px;padding-left:12px;'
                  f'font:600 13px/20px Segoe UI Variable Text,Segoe UI,sans-serif;color:#1C1C1A"><img src="v.svg" width="20" height="20">VietKB</div>'
                  f'<div style="position:absolute;left:0;top:80px;width:240px;height:36px;background:{BG["win-dark"]};display:flex;align-items:center;gap:8px;padding-left:12px;'
                  f'font:600 13px/20px Segoe UI Variable Text,Segoe UI,sans-serif;color:#FFFFFF"><img src="v-dark.svg" width="20" height="20">VietKB</div>')
        page, shot = d / '_r.html', d / '_r.png'
        page.write_text('<!doctype html><html><body style="margin:0;background:transparent">' + ''.join(
            f'<img src="{src}" style="position:absolute;left:{px}px;top:0;width:{sz}px;height:{sz}px">' for _, src, px, sz in pos)
            + strips + '</body></html>', encoding='utf-8')
        subprocess.run([EDGE, '--headless=new', '--disable-gpu', '--force-device-scale-factor=1', '--hide-scrollbars',
                        '--default-background-color=00000000', f'--window-size={max(x, 260) + 8},124', f'--screenshot={shot}', page.as_uri()],
                       capture_output=True)
        im = Image.open(shot).convert('RGBA')
        for fn, _, px, sz in pos:
            im.crop((px, 0, px + sz, sz)).save(d / fn)
        im.crop((0, 40, 240, 76)).save(d / 'titlebar-20.png')
        im.crop((0, 80, 240, 116)).save(d / 'titlebar-20-dark.png')
        page.unlink(); shot.unlink()
    return nums

def sheet(nums):
    grey = ''.join(f'''<div class="g"><b>{p["name"]}</b><div class="gr"><img class="px4" src="{k}/v-16.png"><img class="px4" src="{k}/e-16.png">
<img class="px4" src="{k}/v-16-dark.png"><img class="px4" src="{k}/e-16-dark.png"></div>
<div class="gr"><img src="{k}/v-32.png"><img src="{k}/v-32-dark.png"><img src="{k}/v-16.png"><img src="{k}/v-16-dark.png"></div></div>''' for k, p in PAL.items())
    rows = []
    for k, p in PAL.items():
        (lh, lt), (dh, dt) = p['light'], p['dark']
        n = dict(nums[k])
        show = [('nặng/sáng', n['nặng / taskbar sáng']), ('thanh/sáng', n['thanh / taskbar sáng']), ('nặng/tối', n['nặng(tối) / taskbar tối']),
                ('thanh/tối', n['thanh(tối) / taskbar tối']), ('E sáng', n['E / taskbar sáng']), ('E tối', n['E(tối) / taskbar tối']),
                ('#F7F7F5', min(n['nặng / cửa sổ #F7F7F5'], n['thanh / cửa sổ #F7F7F5'])), ('#272727', min(n['nặng(tối) / cửa sổ #272727'], n['thanh(tối) / cửa sổ #272727']))]
        cells = ''.join(f'<span class="{"bad" if v < 3 else ""}">{t} <b>{v:.2f}</b></span>' for t, v in show)
        sw = lambda c: f'<i style="background:{c}"></i><code>{c}</code>'
        rows.append(f'''<section><h2><span class="id">{k[1]}</span>{p["name"]}
<span class="pal">sáng {sw(lh)}{sw(lt)} · tối {sw(dh)}{sw(dt)}</span></h2>
<div class="r"><div class="tb l"><figure><img class="px" src="{k}/v-16.png"><figcaption>V · bật</figcaption></figure><figure><img class="px" src="{k}/e-16.png"><figcaption>E · tắt</figcaption></figure></div>
<div class="tb d"><figure><img class="px" src="{k}/v-16-dark.png"><figcaption>V · bật</figcaption></figure><figure><img class="px" src="{k}/e-16-dark.png"><figcaption>E · tắt</figcaption></figure></div>
<div class="col"><div class="sz l"><img src="{k}/v-20.png"><img src="{k}/v-32.png"><img src="{k}/e-20.png"></div><div class="sz d"><img src="{k}/v-20-dark.png"><img src="{k}/v-32-dark.png"><img src="{k}/e-20-dark.png"></div>
<img src="{k}/titlebar-20.png" width="240"><img src="{k}/titlebar-20-dark.png" width="240"><div class="nums">{cells}</div></div></div></section>''')
    html = f'''<!doctype html><html lang="vi"><head><meta charset="utf-8"><title>VietKB — logo Ink trong suốt (v6)</title>
<style>
body{{margin:0;padding:16px 24px;background:#E7E4DE;font:400 12px/17px "Segoe UI Variable Text","Segoe UI",sans-serif;color:#1C1C1A;width:1052px}}
h1{{font:600 20px/26px "Segoe UI Variable Display","Segoe UI",sans-serif;margin:0}} .lead{{margin:2px 0 6px;color:#57524A}}
.grey{{background:#808080;border-radius:10px;padding:8px 12px;display:flex;gap:12px;color:#fff}} .g b{{font:600 12px/16px "Segoe UI Variable Text","Segoe UI",sans-serif}}
.gr{{display:flex;gap:6px;align-items:center;margin-top:4px}} .px4{{width:32px;height:32px;image-rendering:pixelated}} .g{{flex:1}}
section{{background:#FAF9F6;border-radius:10px;box-shadow:0 0 0 1px #E0DBD2;padding:7px 12px 9px;margin-top:8px}}
h2{{margin:0 0 6px;font:600 14px/20px "Segoe UI Variable Text","Segoe UI",sans-serif;display:flex;gap:10px;align-items:center}}
.id{{display:inline-grid;place-items:center;width:20px;height:20px;border-radius:5px;background:#1C1C1A;color:#fff;font-size:11px}}
.pal{{display:flex;gap:4px;align-items:center;font-weight:400;color:#57524A;font-size:12px}} .pal i{{width:14px;height:14px;border-radius:3px;display:inline-block}}
.pal code{{font:500 11px/14px "Cascadia Mono",Consolas,monospace;margin-right:6px}}
.r{{display:flex;gap:10px;align-items:flex-start}}
.tb{{display:flex;gap:6px;padding:5px 6px 1px;border-radius:8px}} .tb.l{{background:#F3F3F3}} .tb.d{{background:#202020;color:#C5C5C5}}
figure{{margin:0;text-align:center}} figcaption{{font:500 11px/14px "Segoe UI Variable Small","Segoe UI",sans-serif;margin:1px 0 2px}}
.px{{width:128px;height:128px;image-rendering:pixelated;display:block}}
.col{{display:flex;flex-direction:column;gap:5px}} .sz{{display:flex;gap:10px;align-items:center;padding:4px 8px;border-radius:6px;width:240px}} .sz.l{{background:#F3F3F3}} .sz.d{{background:#202020}}
.nums{{display:grid;grid-template-columns:1fr 1fr;gap:0 10px;font:400 11px/15px "Segoe UI Variable Small","Segoe UI",sans-serif;color:#57524A;width:240px}}
.nums b{{color:#1C1C1A;font-variant-numeric:tabular-nums}} .nums .bad,.nums .bad b{{color:#B42318}}
</style></head><body>
<h1>VietKB · logo Ink, nền trong suốt — 5 bộ màu</h1>
<p class="lead">Không ô, không màu đen: cả hai nét V đều có màu. Hai biến thể taskbar (sáng / tối). E = cả chữ một màu nhấn (đối lập với V hai màu). Khay 16 px phóng 8 lần, pixel gốc.</p>
<div class="grey">{grey}</div>
<p class="lead" style="margin-top:4px">Trên: nền xám giữa #808080 (trường hợp xấu nhất cho dấu trong suốt) — mỗi bộ: V/E bản sáng, V/E bản tối (16 px ×2), rồi V 32 và 16 thật (sáng, tối).</p>
{"".join(rows)}
</body></html>'''
    (HERE / 'index.html').write_text(html, encoding='utf-8')
    subprocess.run([EDGE, '--headless=new', '--disable-gpu', '--force-device-scale-factor=1', '--hide-scrollbars',
                    '--window-size=1100,1700', f'--screenshot={HERE / "sheet.png"}', (HERE / 'index.html').as_uri()], capture_output=True)

if __name__ == '__main__':
    sheet(write_render())
    for k, p in PAL.items():
        print(k, p['name'], 'light', p['light'], 'dark', p['dark'])
