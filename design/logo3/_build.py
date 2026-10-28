# VietKB tray letter pairs, six styles: V = tiếng Việt on, E = off. Drawn on a 16 px grid as paths (no fonts).
# Writes s1..s6/{v,e}.svg, PNGs (light/dark taskbar ink), tile-128.png where the style supports a tile,
# then index.html + sheet.png. Run: python design/logo3/_build.py
import pathlib, subprocess
from PIL import Image

HERE = pathlib.Path(__file__).resolve().parent
EDGE = r'C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe'
INK_L, INK_D, ACC = '#1C1C1A', '#F2F2F5', '#5844C8'

def svg(body, c, extra=''):
    return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" width="16" height="16"{extra}>'
            f'{body.replace("{c}", c)}</svg>')

TILE = 'M4 1H12A3 3 0 0 1 15 4V12A3 3 0 0 1 12 15H4A3 3 0 0 1 1 12V4A3 3 0 0 1 4 1Z'
STYLES = {
    's1': dict(name='Grotesque',
               how='V và E cùng thân 2 px, đầu nét cắt vuông; khác nhau hoàn toàn bằng hình: chữ nhọn đáy vs. chữ có 3 thanh ngang.',
               v='<polygon fill="{c}" points="1.5,1 4.1,1 8,11.6 11.9,1 14.5,1 9.3,15 6.7,15"/>',
               e='<g fill="{c}"><rect x="3" y="1" width="2" height="14"/><rect x="3" y="1" width="10" height="2"/>'
                 '<rect x="3" y="7" width="9" height="2"/><rect x="3" y="13" width="10" height="2"/></g>',
               tile=True),
    's2': dict(name='Rounded humanist',
               how='Nét 2.2 px, đầu tròn, khớp tròn; khoảng mở rộng hơn; V/E khác bằng hình.',
               v='<path d="M2.6 2.1L8 13.9L13.4 2.1" fill="none" stroke="{c}" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/>',
               e='<path d="M4 2.1V13.9M4 2.1H12.4M4 8H11.2M4 13.9H12.6" fill="none" stroke="{c}" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/>',
               tile=True),
    's3': dict(name='Monoline outline',
               how='Một nét 1.5 px, không tô; V/E khác bằng hình. Mảnh nhất — vi phạm quy tắc thân ≥ 2 px theo định nghĩa.',
               v='<path d="M2.25 1.75L8 14.25L13.75 1.75" fill="none" stroke="{c}" stroke-width="1.5" stroke-linecap="square" stroke-linejoin="round"/>',
               e='<path d="M3.75 1.75V14.25M3.75 1.75H12.25M3.75 7.75H11.25M3.75 14.25H12.25" fill="none" stroke="{c}" stroke-width="1.5" stroke-linecap="square" stroke-linejoin="miter"/>',
               tile=False),
    's4': dict(name='Heavy',
               how='Thân 3 px, khối đặc; lòng chữ E giữ 2 px, khe V mở 6 px ở đỉnh; V/E khác bằng hình.',
               v='<polygon fill="{c}" points="1,1 5,1 8,9.1 11,1 15,1 10.2,14 5.8,14"/>',
               e='<g fill="{c}"><rect x="2" y="1" width="3" height="13"/><rect x="2" y="1" width="11" height="3"/>'
                 '<rect x="2" y="6" width="10" height="3"/><rect x="2" y="11" width="11" height="3"/></g>',
               tile=True),
    's5': dict(name='Negative space',
               how='V: ô vuông bo góc đặc, chữ V khoét rỗng. E: cùng ô nhưng chỉ có viền 1.5 px, chữ E đặc bên trong — bật/tắt khác bằng ô đặc vs. ô rỗng và bằng hình chữ.',
               v=f'<path fill="{{c}}" fill-rule="evenodd" d="{TILE}M3.6 3H6.1L8 8.9L9.9 3H12.4L9.2 13H6.8Z"/>',
               e='<rect x="1.75" y="1.75" width="12.5" height="12.5" rx="2.5" fill="none" stroke="{c}" stroke-width="1.5"/>'
                 '<g fill="{c}"><rect x="5" y="3.5" width="2" height="9"/><rect x="5" y="3.5" width="6" height="2"/>'
                 '<rect x="5" y="7" width="5" height="2"/><rect x="5" y="10.5" width="6" height="2"/></g>',
               tile=True),
    's6': dict(name='Ink / calligraphic',
               how='V: nét trái đậm như bút lông, nét phải 1.8 px vượt nhẹ qua chỗ gặp; E: thân 2.4 px, ba nét ngang vẽ tay hơi lên. Mảnh nhất 1.8 px; V/E khác bằng hình.',
               v='<polygon fill="{c}" points="1.5,1.5 4.6,1.5 8.6,14.6 6.4,14.6"/>'
                 '<path d="M7.6 14.8L13.6 1.7" fill="none" stroke="{c}" stroke-width="1.8" stroke-linecap="round"/>',
               e='<polygon fill="{c}" points="3,1.4 5.4,1.4 5.4,14.6 3,14.6"/>'
                 '<path d="M4.4 2.4Q8.5 1.6 12.6 1.9M4.4 8Q7.8 7.4 11 7.6M4.4 13.6Q8.6 13 13.2 12.9" fill="none" stroke="{c}" stroke-width="1.9" stroke-linecap="round"/>',
               tile=True),
}

def tile_svg(vbody):
    return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" width="16" height="16">'
            f'<rect width="16" height="16" rx="3.5" fill="{ACC}"/>'
            f'<g transform="translate(2.4 2.4) scale(.7)">{vbody.replace("{c}", "#FFFFFF")}</g></svg>')

def write():
    for key, s in STYLES.items():
        d = HERE / key
        d.mkdir(exist_ok=True)
        for letter in 've':
            (d / f'{letter}.svg').write_text(svg(s[letter], INK_L), encoding='utf-8')
            (d / f'_{letter}-dark.svg').write_text(svg(s[letter], INK_D), encoding='utf-8')
        if s['tile'] and key != 's5':
            (d / '_tile.svg').write_text(tile_svg(s['v']), encoding='utf-8')
        elif key == 's5':  # the style already is a tile: V knocked out of the accent square
            (d / '_tile.svg').write_text(svg(s['v'], ACC), encoding='utf-8')

def render():
    for key, s in STYLES.items():
        d = HERE / key
        jobs = []
        for letter in 've':
            for sz in (16, 20, 32):
                jobs.append((f'{letter}-{sz}.png', f'{letter}.svg', sz))
            for sz in (16, 20):
                jobs.append((f'{letter}-{sz}-dark.png', f'_{letter}-dark.svg', sz))
        if (d / '_tile.svg').exists():
            jobs.append(('tile-128.png', '_tile.svg', 128))
        x, pos = 0, []
        for fn, src, sz in jobs:
            pos.append((fn, src, x, sz)); x += sz + 8
        page, shot = d / '_r.html', d / '_r.png'
        page.write_text('<!doctype html><html><body style="margin:0;background:transparent">' + ''.join(
            f'<img src="{src}" style="position:absolute;left:{px}px;top:0;width:{sz}px;height:{sz}px">' for _, src, px, sz in pos)
            + '</body></html>', encoding='utf-8')
        subprocess.run([EDGE, '--headless=new', '--disable-gpu', '--force-device-scale-factor=1', '--hide-scrollbars',
                        '--default-background-color=00000000', f'--window-size={x + 8},140', f'--screenshot={shot}', page.as_uri()],
                       capture_output=True)
        im = Image.open(shot).convert('RGBA')
        for fn, _, px, sz in pos:
            im.crop((px, 0, px + sz, sz)).save(d / fn)
        page.unlink(); shot.unlink()

def sheet():
    rows = []
    for key, s in STYLES.items():
        tile = f'<img src="{key}/tile-128.png" width="96" height="96">' if (HERE / key / 'tile-128.png').exists() else '<span class="na">không có ô</span>'
        rows.append(f'''<section><h2><span class="id">{key[1]}</span>{s["name"]}</h2><p>{s["how"]}</p>
<div class="r">
  <div class="tb l"><figure><img class="px" src="{key}/v-16.png"><figcaption>V · bật</figcaption></figure><figure><img class="px" src="{key}/e-16.png"><figcaption>E · tắt</figcaption></figure></div>
  <div class="tb d"><figure><img class="px" src="{key}/v-16-dark.png"><figcaption>V · bật</figcaption></figure><figure><img class="px" src="{key}/e-16-dark.png"><figcaption>E · tắt</figcaption></figure></div>
  <div class="sz"><div class="b l"><img src="{key}/v-20.png"><img src="{key}/e-20.png"><img src="{key}/v-32.png"><img src="{key}/e-32.png"></div>
    <div class="b d"><img src="{key}/v-16-dark.png"><img src="{key}/e-16-dark.png"><img src="{key}/v-20-dark.png"><img src="{key}/e-20-dark.png"></div>
    <div class="b l"><img src="{key}/v-16.png"><img src="{key}/e-16.png"><span>16 thật</span></div><small>20 · 32 (sáng) · 16 · 20 (tối)</small></div>
  <div class="tile">{tile}</div>
</div></section>''')
    html = f'''<!doctype html><html lang="vi"><head><meta charset="utf-8"><title>VietKB — V/E cho khay, 6 phong cách</title>
<style>
body{{margin:0;padding:18px 24px;background:#E7E4DE;font:400 12.5px/18px "Segoe UI Variable Text","Segoe UI",sans-serif;color:#1C1C1A;width:1052px}}
h1{{font:600 21px/28px "Segoe UI Variable Display","Segoe UI",sans-serif;margin:0}}
.lead{{margin:2px 0 4px;color:#57524A}}
section{{background:#FAF9F6;border-radius:10px;box-shadow:0 0 0 1px #E0DBD2;padding:8px 14px 10px;margin-top:8px}}
h2{{margin:0;font:600 14px/20px "Segoe UI Variable Text","Segoe UI",sans-serif;display:flex;gap:8px;align-items:center}}
.id{{display:inline-grid;place-items:center;width:20px;height:20px;border-radius:5px;background:#1C1C1A;color:#fff;font-size:11px}}
section>p{{margin:2px 0 6px;color:#57524A;font-size:12px}}
.r{{display:flex;gap:14px;align-items:flex-start}}
.tb{{display:flex;gap:8px;padding:6px 8px 2px;border-radius:8px}} .tb.l{{background:#F3F3F3}} .tb.d{{background:#202020;color:#C5C5C5}}
figure{{margin:0;text-align:center}} figcaption{{font:500 11px/14px "Segoe UI Variable Small","Segoe UI",sans-serif;margin:2px 0 3px}}
.px{{width:128px;height:128px;image-rendering:pixelated;display:block}}
.sz{{display:flex;flex-direction:column;gap:6px}} .b{{display:flex;gap:10px;align-items:center;padding:6px 8px;border-radius:6px}}
.b.l{{background:#F3F3F3}} .b.d{{background:#202020}} .b span{{font-size:11px;color:#6B655B}} .sz small{{color:#6B655B}}
.tile{{width:96px;display:grid;place-items:center;height:128px}} .na{{color:#8A847A;font-size:11.5px}}
</style></head><body>
<h1>VietKB · khay V / E — 6 phong cách</h1>
<p class="lead">V = bật tiếng Việt, E = tắt. Một màu mực: <b>#1C1C1A</b> trên taskbar sáng, <b>#F2F2F5</b> trên taskbar tối. Ô 16 px phóng 8 lần (pixel gốc).</p>
{"".join(rows)}
</body></html>'''
    (HERE / 'index.html').write_text(html, encoding='utf-8')
    subprocess.run([EDGE, '--headless=new', '--disable-gpu', '--force-device-scale-factor=1', '--hide-scrollbars',
                    '--window-size=1100,1600', f'--screenshot={HERE / "sheet.png"}', (HERE / 'index.html').as_uri()], capture_output=True)

if __name__ == '__main__':
    write(); render(); sheet(); print('ok')
