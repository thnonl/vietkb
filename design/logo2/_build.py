# VietKB logotype directions A–D (lettering). Drawn letterforms (monoline strokes), no fonts.
# Writes SVGs, renders PNGs via headless Edge (transparent), builds index.html + sheet.png.
# Run: python design/logo2/_build.py
import pathlib, subprocess
from PIL import Image

HERE = pathlib.Path(__file__).resolve().parent
EDGE = r'C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe'
ACC, ACC_D, INK, PAPER = '#5844C8', '#6B57D8', '#1C1C1A', '#F7F7F5'
OFF_L, OFF_D = '#3A362F', '#DCD9E3'
SW = 3.0   # wordmark stroke on a 24-unit cap box (cap height 14-16) -> ~0.2 em, no hairlines

# Each part: (svg element with {c} for colour, is_accent). Strokes use the group's stroke settings;
# dots are filled circles.
B_UP_LO = lambda x, top=4, mid=10.5, base=18: (f'M{x} {base}V{top}h4.5a3.25 3.25 0 0 1 0 {mid-top}H{x}'
                                              f'm0 0h5.5a3.75 3.75 0 0 1 0 {base-mid}H{x}')
DIRS = {
    'A': dict(
        spell='VKB', idea='Hình học, chữ V mang dấu nặng',
        derive='Dạng gọn là chữ V có dấu nặng, lấy nguyên từ chữ đầu của wordmark.',
        word=[('<path d="M2 4L8 18L14 4"/>', 0), ('<circle cx="8" cy="22" r="1.9" fill="{c}" stroke="none"/>', 1),
              ('<path d="M18 4V18M27 4L18.6 12.4M21.8 9.8L27.5 18"/>', 0), (f'<path d="{B_UP_LO(31)}"/>', 0)],
        wbox=(0, 0, 42, 24),
        mark=[('<path d="M2 4L8 18L14 4"/>', 0), ('<circle cx="8" cy="22" r="1.9" fill="{c}" stroke="none"/>', 1)],
        mbox=(0.5, 2.5, 15, 21.5)),
    'B': dict(
        spell='ViKB', idea='Chữ thường i, chấm thành dấu sắc',
        derive='Dạng gọn là “Ví”: hai chữ đầu của wordmark, chấm i là dấu sắc.',
        word=[('<path d="M2 4L8 18L14 4"/>', 0), ('<path d="M18 10V18"/>', 0), ('<path d="M17.6 7L20.2 3.4"/>', 1),
              ('<path d="M24 4V18M33 4L24.6 12.4M27.8 9.8L33.5 18"/>', 0), (f'<path d="{B_UP_LO(37)}"/>', 0)],
        wbox=(0, 0, 48, 24),
        mark=[('<path d="M2 4L8 18L14 4"/>', 0), ('<path d="M18 10V18"/>', 0), ('<path d="M17.6 7L20.2 3.4"/>', 1)],
        mbox=(0.5, 1.9, 21.7, 16.6)),
    'C': dict(
        spell='Viet', idea='Cả chữ Viet, V có móc',
        derive='Dạng gọn là chữ V đầu wordmark, cánh trái có móc (dấu hỏi) liền nét.',
        word=[('<path d="M4 6L9.5 19L15 4"/>', 0), ('<path d="M4 6C2.9 4.4 3.4 2 5.3 2C6.9 2 7.6 3.7 6.6 4.9"/>', 1),
              ('<path d="M19 10V19"/>', 0), ('<circle cx="19" cy="6" r="1.9" fill="{c}" stroke="none"/>', 0),
              ('<path d="M23 14.5H32A4.5 4.5 0 1 0 30.7 17.8"/>', 0), ('<path d="M36 6V16.5Q36 19 38.6 19M33.5 10H38.6"/>', 0)],
        wbox=(0, 0, 41, 24),
        mark=[('<path d="M4 6L9.5 19L15 4"/>', 0), ('<path d="M4 6C2.9 4.4 3.4 2 5.3 2C6.9 2 7.6 3.7 6.6 4.9"/>', 1)],
        mbox=(1.5, 0.5, 15, 20.5)),
    'D': dict(
        spell='VKB', idea='Ba chữ chung nét, B có gạch đ',
        derive='Dạng gọn chính là khối chữ ghép: cánh phải của V là thân K, đầu nhánh K là thân B.',
        word=[('<path d="M2 4L7 18L12 4"/>', 0), ('<path d="M9.5 11L17 4M9.5 11L17 18"/>', 0),
              ('<path d="M17 18V4h4a3.25 3.25 0 0 1 0 6.5H17m0 0h5a3.75 3.75 0 0 1 0 7.5H17"/>', 0),
              ('<path d="M14.6 8.2H19.4"/>', 1)],
        wbox=(0, 1, 27.5, 20),
        mark=None, mbox=(0.5, 2.5, 27, 17.5)),
}

# Tray (owner decision): V = Vietnamese on, E = off. Drawn on a 16 px grid, 2 px round-capped strokes, the
# same monoline construction and terminals as the wordmark; each direction keeps its V signature if it survives.
E16 = '<path d="M4 2.5V13.5M4 2.5H12M4 8H11M4 13.5H12"/>'
V16 = {
    'A': '<path d="M3 2.5L8 13.5L13 2.5"/>',  # the nặng dot fuses with the V point into a Y at 16 px -> plain V
    'B': '<path d="M3 2.5L8 13.5L13 2.5"/>',
    'C': '<path d="M3 2.5L8 13.5L13 2.5"/>',  # the hook blurs into a blob at 16 px -> plain V in the tray
    'D': '<path d="M3 2.5L8 13.5L13 2.5"/>',
}

def tray_svg(body, c):
    return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" width="16" height="16">'
            f'<g fill="none" stroke="{c}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">{body.replace("{c}", c)}</g></svg>')

def parts_svg(parts, main, accent, sw, transform=''):
    g = ''.join(p.replace('{c}', accent if a else main) if 'fill=' in p
                else f'<g stroke="{accent if a else main}">{p}</g>' for p, a in parts)
    t = f' transform="{transform}"' if transform else ''
    return f'<g fill="none" stroke-width="{sw:.3f}" stroke-linecap="round" stroke-linejoin="round"{t}>{g}</g>'

def wordmark(d, main, accent):
    x, y, w, h = d['wbox']
    return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="{x} {y} {w} {h}" width="{w}" height="{h}">'
            f'{parts_svg(d["word"], main, accent, SW)}</svg>')

def compact(d, bg, fg, accent, target_sw, field=17, radius=6, tile=True):
    parts = d['mark'] or d['word']
    x0, y0, w, h = d['mbox']
    s = min(field / w, field / h)
    tx, ty = 12 - (x0 + w / 2) * s, 12 - (y0 + h / 2) * s
    rect = f'<rect width="24" height="24" rx="{radius}" fill="{bg}"/>' if tile else ''
    return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">{rect}'
            f'{parts_svg(parts, fg, accent, target_sw / s, f"translate({tx:.3f} {ty:.3f}) scale({s:.4f})")}</svg>')

def write_svgs():
    for key, d in DIRS.items():
        p = HERE / key
        p.mkdir(exist_ok=True)
        (p / 'wordmark.svg').write_text(wordmark(d, INK, ACC), encoding='utf-8')
        (p / 'wordmark-dark.svg').write_text(wordmark(d, PAPER, ACC_D), encoding='utf-8')
        (p / 'tile.svg').write_text(compact(d, ACC, '#FFFFFF', '#FFFFFF', 2.4), encoding='utf-8')
        (p / 'tile-dark.svg').write_text(compact(d, ACC_D, '#FFFFFF', '#FFFFFF', 2.4), encoding='utf-8')
        for old in ('tray-on.svg', 'tray-off.svg', 'tray-off-dark.svg', 'tray-on.png', 'tray-off.png', 'tray-off-dark.png',
                    'tray-on-20.png', 'tray-off-20.png'):
            (p / old).unlink(missing_ok=True)
        # tray letters: ink on a light taskbar, paper on a dark one; state is the letter shape, not colour
        for letter, body in (('v', V16[key]), ('e', E16)):
            (p / f'tray-{letter}.svg').write_text(tray_svg(body, INK), encoding='utf-8')
            (p / f'tray-{letter}-dark.svg').write_text(tray_svg(body, PAPER), encoding='utf-8')

def wm_size(d, width):
    _, _, w, h = d['wbox']
    return width, round(width * h / w)

def render():
    for key, d in DIRS.items():
        ww, wh = wm_size(d, 480)
        jobs = [('wordmark-480.png', 'wordmark.svg', ww, wh), ('tile-128.png', 'tile.svg', 128, 128),
                ('tile-64.png', 'tile.svg', 64, 64), ('tile-32.png', 'tile.svg', 32, 32),
                ('titlebar-20.png', 'tile.svg', 20, 20)]
        for letter in 've':
            for sz in (16, 20, 32):
                jobs.append((f'tray-{letter}-{sz}.png', f'tray-{letter}.svg', sz, sz))
            jobs.append((f'tray-{letter}-dark-16.png', f'tray-{letter}-dark.svg', 16, 16))
            jobs.append((f'tray-{letter}-dark-20.png', f'tray-{letter}-dark.svg', 20, 20))
            jobs.append((f'tray-{letter}-dark-32.png', f'tray-{letter}-dark.svg', 32, 32))
        x, pos = 0, []
        for fn, src, w, h in jobs:
            pos.append((fn, src, x, w, h)); x += w + 8
        imgs = ''.join(f'<img src="{src}" style="position:absolute;left:{px}px;top:0;width:{w}px;height:{h}px">' for _, src, px, w, h in pos)
        page, shot = HERE / key / '_r.html', HERE / key / '_r.png'
        page.write_text(f'<!doctype html><html><body style="margin:0;background:transparent">{imgs}</body></html>', encoding='utf-8')
        subprocess.run([EDGE, '--headless=new', '--disable-gpu', '--force-device-scale-factor=1', '--hide-scrollbars',
                        '--default-background-color=00000000', f'--window-size={x + 8},{max(h for *_, h in pos) + 8}',
                        f'--screenshot={shot}', page.as_uri()], capture_output=True)
        im = Image.open(shot).convert('RGBA')
        for fn, _, px, w, h in pos:
            im.crop((px, 0, px + w, h)).save(HERE / key / fn)
        page.unlink(); shot.unlink()

def sheet():
    secs = []
    for key, d in DIRS.items():
        ww, wh = wm_size(d, 480)
        ratio = d['wbox'][2] / d['wbox'][3]
        secs.append(f'''<section><h2><span class="id">{key}</span><b>{d["spell"]}</b> <small>{d["idea"]} · tỉ lệ {ratio:.2f}:1 · {d["derive"]}</small></h2>
<div class="cells">
  <div class="col">
    <div class="paper wm"><img src="{key}/wordmark.svg" style="height:104px;width:auto"></div>
    <div class="ink wm2"><img src="{key}/wordmark-dark.svg" style="height:32px;width:auto"></div>
    <div class="tb l"><img src="{key}/titlebar-20.png"><b>VietKB</b><i></i><span>–</span><span>✕</span></div>
    <div class="tb d"><img src="{key}/tile-dark.svg" width="20" height="20"><b>VietKB</b><i></i><span>–</span><span>✕</span></div>
  </div>
  <figure><img src="{key}/tile-128.png" width="128" height="128"><div class="row"><img src="{key}/tile-32.png"><img src="{key}/titlebar-20.png"><img src="{key}/tray-v-16.png"><img src="{key}/tray-e-16.png"></div><figcaption>ô 128 · 32 · 20<br>khay V · E 16</figcaption></figure>
  <div class="trays"><div class="lab"><span>bật (V)</span><span>tắt (E)</span></div>
    <div class="tray l"><img class="px" src="{key}/tray-v-16.png"><img class="px" src="{key}/tray-e-16.png"></div>
    <div class="tray d"><img class="px" src="{key}/tray-v-dark-16.png"><img class="px" src="{key}/tray-e-dark-16.png"></div></div>
</div></section>''')
    html = f'''<!doctype html><html lang="vi"><head><meta charset="utf-8"><title>VietKB — logo chữ (v2)</title>
<style>
body{{margin:0;padding:20px 28px;background:#E7E4DE;font:400 13px/20px "Segoe UI Variable Text","Segoe UI",sans-serif;color:#1C1C1A;width:1044px}}
h1{{font:600 22px/30px "Segoe UI Variable Display","Segoe UI",sans-serif;margin:0}}
.lead{{margin:2px 0 6px;color:#57524A}}
section{{background:#FAF9F6;border-radius:10px;box-shadow:0 0 0 1px #E0DBD2;padding:8px 16px 10px;margin-top:8px}}
h2{{margin:0;font:600 15px/22px "Segoe UI Variable Text","Segoe UI",sans-serif;display:flex;gap:8px;align-items:baseline}}
h2 small{{font-weight:400;color:#6B655B}} .id{{display:inline-grid;place-items:center;width:22px;height:22px;border-radius:6px;background:#1C1C1A;color:#fff;font-size:12px}}
.der{{margin:0 0 8px;color:#57524A;font-size:12px}}
.cells{{display:flex;gap:18px;align-items:flex-start;margin-top:6px}} figure{{margin:0}} .col{{width:480px;display:flex;flex-direction:column;gap:6px}}
figcaption{{font:400 11.5px/16px "Segoe UI Variable Small","Segoe UI",sans-serif;color:#6B655B;margin-top:4px}}
.paper{{background:#F7F7F5;border-radius:8px;box-shadow:inset 0 0 0 1px #E8E5DF;display:flex;align-items:center;justify-content:center;padding:10px}}
.wm{{width:480px;height:124px}} .ink{{background:#17161C;border-radius:8px;display:flex;align-items:center;justify-content:center}} .wm2{{width:480px;height:40px}}
.row{{display:flex;gap:10px;align-items:flex-end;margin-top:8px}}
.tray{{display:flex;gap:8px;padding:4px;border-radius:8px}} .tray.l{{background:#F3F3F3}} .tray.d{{background:#202020;margin-top:4px}} .lab{{display:flex;gap:8px;padding:0 6px;font:500 12px/16px "Segoe UI Variable Small","Segoe UI",sans-serif;color:#57524A}} .lab span{{width:128px;text-align:center}}
.px{{width:128px;height:128px;image-rendering:pixelated;display:block}} .tray p{{margin:2px 0 0;text-align:center;font:500 11.5px/16px "Segoe UI Variable Small","Segoe UI",sans-serif}} .tray.l p{{color:#57524A}} .tray.d p{{color:#C9C6D2}}
.bars{{display:flex;gap:16px;margin-top:10px}}
.tb{{width:480px;height:34px;display:flex;align-items:center;gap:8px;padding-left:16px;border-radius:8px 8px 0 0;box-shadow:0 0 0 1px rgba(0,0,0,.08)}}
.tb.l{{background:#F2F0EB;color:#1C1A16}} .tb.d{{background:#111015;color:#EEECF3}} .tb b{{font:600 13px/20px "Segoe UI Variable Text","Segoe UI",sans-serif}}
.tb i{{flex:1}} .tb span{{width:44px;text-align:center;opacity:.7;font-size:11px}}
</style></head><body>
<h1>VietKB · logo chữ — 4 hướng</h1>
<p class="lead">Wordmark vẽ bằng nét · dạng gọn suy ra từ wordmark · khay (đã chốt): <b>V</b> = bật, <b>E</b> = tắt, 16 px phóng 8 lần.</p>
{"".join(secs)}
</body></html>'''
    (HERE / 'index.html').write_text(html, encoding='utf-8')
    subprocess.run([EDGE, '--headless=new', '--disable-gpu', '--force-device-scale-factor=1', '--hide-scrollbars',
                    '--window-size=1100,1500', f'--screenshot={HERE / "sheet.png"}', (HERE / 'index.html').as_uri()], capture_output=True)

if __name__ == '__main__':
    write_svgs(); render(); sheet(); print('ok')
