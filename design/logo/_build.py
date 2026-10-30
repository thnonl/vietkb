# VietKB logo concepts A–D: writes SVG masters, renders PNGs through headless Edge (transparent background),
# and builds the contact sheet. Run: python design/logo/_build.py
import pathlib, subprocess
from PIL import Image

HERE = pathlib.Path(__file__).resolve().parent
EDGE = r'C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe'
INK, INK_DARK = '#5844C8', '#6B57D8'
OFF_LIGHT, OFF_DARK = '#3A362F', '#DCD9E3'   # tray "off" glyph on light / dark taskbar

# Each glyph is drawn with round-capped strokes on a 24 x 24 grid.
CONCEPTS = {
    'A': dict(name='Móc', idea='Dấu hỏi đặt trên dòng chữ',
              body='<path d="M9 8c0-2.2 1.3-4 3-4s3 1.3 3 3c0 1.5-1 2.3-2 2.9-.6.4-1 .8-1 1.6V13"/><path d="M6 19h12"/>'),
    'B': dict(name='Ơ', idea='Chữ ơ chỉ tiếng Việt có',
              body='<circle cx="11" cy="14" r="6"/><path d="M16.2 10.6C18 10.6 19 9.6 19 7.8"/>'),
    'C': dict(name='Đ', idea='Chữ đ có gạch ngang',
              body='<circle cx="10" cy="15" r="5"/><path d="M15 4v16"/><path d="M12 7h6"/>'),
    'D': dict(name='Con trỏ ngã', idea='Con trỏ gõ mang dấu ngã',
              body='<path d="M12 10v10"/><path d="M9.5 10h5M9.5 20h5"/><path d="M6.5 6.5c1.5-2 3.5-2 5.5-.5s4 1.5 5.5-.5"/>'),
}

def glyph_svg(body, color, stroke=2.0):
    return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">'
            f'<g fill="none" stroke="{color}" stroke-width="{stroke}" stroke-linecap="round" stroke-linejoin="round">{body}</g></svg>')

def tile_svg(body, bg=INK, fg='#FFFFFF', stroke=2.5, radius=6):
    # glyph scaled to 75 % and centred: inner 18 x 18 field inside the 24 tile
    return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">'
            f'<rect width="24" height="24" rx="{radius}" fill="{bg}"/>'
            f'<g transform="translate(3 3) scale(.75)" fill="none" stroke="{fg}" stroke-width="{stroke / .75:.3f}" '
            f'stroke-linecap="round" stroke-linejoin="round">{body}</g></svg>')

def write_svgs():
    for key, c in CONCEPTS.items():
        d = HERE / key
        d.mkdir(exist_ok=True)
        (d / 'logo.svg').write_text(glyph_svg(c['body'], INK), encoding='utf-8')
        (d / 'logo-dark.svg').write_text(glyph_svg(c['body'], '#8E7BEA'), encoding='utf-8')
        (d / 'tile.svg').write_text(tile_svg(c['body']), encoding='utf-8')
        (d / 'tile-dark.svg').write_text(tile_svg(c['body'], bg=INK_DARK), encoding='utf-8')
        # tray: ON = solid tile with the glyph knocked out; OFF = the bare glyph, heavier stroke, no tile.
        (d / 'tray-on.svg').write_text(tile_svg(c['body'], stroke=2.5, radius=5), encoding='utf-8')
        (d / 'tray-off.svg').write_text(glyph_svg(c['body'], OFF_LIGHT, stroke=2.5), encoding='utf-8')
        (d / 'tray-off-dark.svg').write_text(glyph_svg(c['body'], OFF_DARK, stroke=2.5), encoding='utf-8')

RENDERS = [('16.png', 'tile.svg', 16), ('20.png', 'tile.svg', 20), ('32.png', 'tile.svg', 32), ('64.png', 'tile.svg', 64),
           ('128.png', 'tile.svg', 128), ('256.png', 'tile.svg', 256), ('tray-on.png', 'tray-on.svg', 16),
           ('tray-off.png', 'tray-off.svg', 16), ('tray-off-dark.png', 'tray-off-dark.svg', 16),
           ('tray-on-20.png', 'tray-on.svg', 20), ('tray-off-20.png', 'tray-off.svg', 20)]

def render_pngs():
    # one transparent page per concept, each image at an integer position, then crop
    for key in CONCEPTS:
        x, items = 0, []
        for fn, src, s in RENDERS:
            items.append((fn, x, s))
            x += s + 8
        imgs = ''.join(f'<img src="{src}" style="position:absolute;left:{px}px;top:0;width:{s}px;height:{s}px">'
                       for (fn, src, s), (_, px, _) in zip(RENDERS, items))
        page = HERE / key / '_render.html'
        page.write_text(f'<!doctype html><html><body style="margin:0;background:transparent">{imgs}</body></html>', encoding='utf-8')
        shot = HERE / key / '_render.png'
        subprocess.run([EDGE, '--headless=new', '--disable-gpu', '--force-device-scale-factor=1', '--hide-scrollbars',
                        '--default-background-color=00000000', f'--window-size={x + 8},264', f'--screenshot={shot}',
                        page.as_uri()], capture_output=True)
        im = Image.open(shot).convert('RGBA')
        for fn, px, s in items:
            im.crop((px, 0, px + s, s)).save(HERE / key / fn)
        page.unlink(); shot.unlink()

def sheet():
    blocks = []
    for key, c in CONCEPTS.items():
        tb = lambda theme: (f'<div class="tb {theme}"><img src="{key}/{"tile-dark" if theme == "dark" else "tile"}.svg" width="20" height="20">'
                            f'<b>VietKB</b><span class="sp"></span><span class="wcb">&#x2013;</span><span class="wcb">&#x2715;</span></div>')
        blocks.append(f'''<section><h2><span class="id">{key}</span>{c["name"]} <small>{c["idea"]}</small></h2>
<div class="cells">
  <figure><div class="paper"><img src="{key}/logo.svg" width="128" height="128"></div><figcaption>chữ 128</figcaption></figure>
  <figure><div class="paper sm"><img src="{key}/logo.svg" width="32" height="32"><img src="{key}/logo.svg" width="16" height="16"></div>
    <div class="ink sm"><img src="{key}/logo-dark.svg" width="32" height="32"><img src="{key}/logo-dark.svg" width="16" height="16"></div><figcaption>32 · 16, giấy / mực</figcaption></figure>
  <figure><img src="{key}/128.png" width="128" height="128"><figcaption>ô ứng dụng 128</figcaption></figure>
  <figure><div class="row16"><img src="{key}/16.png"><img src="{key}/20.png"><img src="{key}/32.png"></div><figcaption>ô 16 · 20 · 32 (PNG)</figcaption></figure>
  <figure><div class="tray light"><img class="px" src="{key}/tray-on.png"><img class="px" src="{key}/tray-off.png"></div>
    <div class="tray dark"><img class="px" src="{key}/tray-on.png"><img class="px" src="{key}/tray-off-dark.png"></div>
    <figcaption>khay 16 px ×8: bật · tắt (taskbar sáng / tối)</figcaption></figure>
</div>
<div class="bars">{tb("light")}{tb("dark")}</div></section>''')
    html = f'''<!doctype html><html lang="vi"><head><meta charset="utf-8"><title>VietKB — 4 phương án logo</title>
<style>
body{{margin:0;padding:24px 28px;background:#E7E4DE;font:400 13px/20px "Segoe UI Variable Text","Segoe UI",sans-serif;color:#1C1A16;width:1044px}}
h1{{font:600 22px/30px "Segoe UI Variable Display","Segoe UI",sans-serif;margin:0 0 4px}}
.lead{{color:#57524A;margin:0 0 8px}}
section{{background:#FAF9F6;border-radius:10px;box-shadow:0 0 0 1px #E0DBD2;padding:12px 16px 14px;margin-top:12px}}
h2{{font:600 15px/22px "Segoe UI Variable Text","Segoe UI",sans-serif;margin:0 0 8px;display:flex;align-items:baseline;gap:8px}}
h2 small{{font-weight:400;color:#6B655B}} .id{{display:inline-grid;place-items:center;width:22px;height:22px;border-radius:6px;background:#1C1A16;color:#fff;font-size:12px}}
.cells{{display:flex;gap:16px;align-items:flex-start}} figure{{margin:0}}
figcaption{{font:400 11.5px/16px "Segoe UI Variable Small","Segoe UI",sans-serif;color:#6B655B;margin-top:4px;max-width:200px}}
.paper{{background:#F2F0EB;border-radius:8px;display:flex;gap:12px;align-items:center;justify-content:center}}
.ink{{background:#17161C;border-radius:8px;display:flex;gap:12px;align-items:center;justify-content:center;margin-top:6px}}
.sm{{width:96px;height:56px}}
.row16{{display:flex;gap:10px;align-items:flex-end;height:128px;padding-bottom:4px}}
.tray{{display:flex;gap:8px;padding:6px;border-radius:8px}} .tray.light{{background:#F3F3F3}} .tray.dark{{background:#202020;margin-top:6px}}
.px{{width:64px;height:64px;image-rendering:pixelated}}
.bars{{display:flex;gap:16px;margin-top:10px}}
.tb{{width:480px;height:36px;display:flex;align-items:center;gap:8px;padding-left:16px;border-radius:8px 8px 0 0;box-shadow:0 0 0 1px rgba(0,0,0,.08)}}
.tb.light{{background:#F2F0EB;color:#1C1A16}} .tb.dark{{background:#111015;color:#EEECF3}}
.tb b{{font:600 13px/20px "Segoe UI Variable Text","Segoe UI",sans-serif}} .tb .sp{{flex:1}}
.wcb{{width:44px;text-align:center;opacity:.7;font-size:11px}}
</style></head><body>
<h1>VietKB · 4 phương án logo</h1>
<p class="lead">Mỗi phương án: chữ ở 128 / 32 / 16 trên giấy sáng và mặt tối, ô ứng dụng, PNG nhỏ, hai trạng thái khay ở 16 px phóng 8 lần (pixel gốc), và thanh tiêu đề 480 px với logo 20 px.
Khay: <b>bật</b> = ô đặc, chữ khoét trắng · <b>tắt</b> = chỉ nét chữ, không có ô.</p>
{"".join(blocks)}
</body></html>'''
    (HERE / 'index.html').write_text(html, encoding='utf-8')

if __name__ == '__main__':
    write_svgs()
    render_pngs()
    sheet()
    print('ok')
