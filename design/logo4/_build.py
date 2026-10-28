# VietKB colour identity systems A–D: logo (lockup + tile) first, tray V/E derived from it.
# All marks are paths on a 16 px grid (integer coords where possible); the wordmark letters are drawn strokes.
# Run: python design/logo4/_build.py
import pathlib, subprocess
from PIL import Image

HERE = pathlib.Path(__file__).resolve().parent
EDGE = r'C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe'
INK, PAPER, ACC, ACC_D = '#1C1C1A', '#F2F2F5', '#5844C8', '#8B7BFF'

# --- wordmark "VietKB": monoline strokes, cap 5..19 on a 24-unit line, round ends (same family as logo2)
def B_PATH(x, top=5, mid=11.6, base=19):
    return (f'M{x} {base}V{top}h4.5a3.3 3.3 0 0 1 0 {mid-top:.1f}H{x}m0 0h5.4a3.7 3.7 0 0 1 0 {base-mid:.1f}H{x}')
WORD = (f'<g fill="none" stroke="{{c}}" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">'
        f'<path d="M2 5L8 19L14 5"/><path d="M18.5 10V19"/>'
        f'<path d="M22.5 14.5H31.5A4.5 4.5 0 1 0 30.2 17.8"/><path d="M35.5 6V16.5Q35.5 19 38.1 19M33 10H38.1"/>'
        f'<path d="M42.5 5V19M51.5 5L43.1 13.4M46.3 10.8L52 19"/><path d="{B_PATH(55.5)}"/></g>'
        f'<circle cx="18.5" cy="6" r="1.9" fill="{{c}}"/>')
WORD_W = 66

# --- marks (16 grid). Each concept: on (V) and off (E) bodies for light and dark taskbars, plus the app tile.
V_HEAVY_IN_TILE = '3.5,3 6.2,3 8,9 9.8,3 12.5,3 9.3,13 6.7,13'
E_IN_TILE = '<rect x="5" y="3" width="2" height="10"/><rect x="5" y="3" width="6" height="2"/><rect x="5" y="7" width="5" height="2"/><rect x="5" y="11" width="6" height="2"/>'
TILE = '<rect x="1" y="1" width="14" height="14" rx="3" fill="{t}"/>'

C_V = ('<polygon fill="{a}" points="1.5,1.5 4.6,1.5 8.6,14.6 6.4,14.6"/>'
       '<path d="M7.6 14.8L13.6 1.7" fill="none" stroke="{b}" stroke-width="1.9" stroke-linecap="round"/>')
C_E = ('<polygon fill="{a}" points="3,1.4 5.4,1.4 5.4,14.6 3,14.6"/>'
       '<path d="M4.4 2.4Q8.5 1.6 12.6 1.9M4.4 8Q7.8 7.4 11 7.6M4.4 13.6Q8.6 13 13.2 12.9" fill="none" stroke="{b}" stroke-width="1.9" stroke-linecap="round"/>')

CONCEPTS = {
    'A': dict(
        name='Tile', style='ô vuông bo góc màu mực tím, chữ khối trắng',
        palette=[(ACC, 'ô (sáng)'), (ACC_D, 'ô (tối)'), ('#FFFFFF', 'chữ V trên ô'), ('#9A97A3', 'ô tắt, xám nhạt')],
        on=lambda ink, dark: TILE.replace('{t}', ACC_D if dark else ACC) + f'<polygon fill="{INK if dark else "#FFFFFF"}" points="{V_HEAVY_IN_TILE}"/>',
        off=lambda ink, dark: TILE.replace('{t}', '#55545B' if dark else '#B4B1BC') + f'<g fill="{"#E6E6EA" if dark else "#FFFFFF"}">{E_IN_TILE}</g>',
        tile=lambda: TILE.replace('{t}', ACC) + f'<polygon fill="#FFFFFF" points="{V_HEAVY_IN_TILE}"/>',
        note=('Ý tưởng: một ô vuông bo góc màu mực tím chứa chữ V khối trắng; khay chính là ô đó thu nhỏ. '
              'Bảng màu: #5844C8 (ô, nền sáng), #8B7BFF (ô trên taskbar tối, chữ đổi sang mực #1C1C1A để đạt 5.18:1), '
              'trắng (chữ), xám #B4B1BC / #55545B (ô khi tắt). Khay suy ra trực tiếp: cùng ô, cùng chữ; tắt = chữ E trên ô xám — '
              'khác bằng hình chữ, màu xám chỉ củng cố. Ở 16 px: ô chiếm 14 px, chữ 10 px, nét 2 px; cần 2 biến thể taskbar (sáng/tối).')),
    'B': dict(
        name='Ligature', style='chữ V phá vỡ cạnh trên của ô, nửa trong nửa ngoài',
        palette=[(ACC, 'ô + phần V nhô ra (sáng)'), (ACC_D, 'cùng vai trò trên nền tối'), ('#8A8794', 'trạng thái tắt')],
        on=lambda ink, dark: (f'<path fill="{ACC_D if dark else ACC}" fill-rule="evenodd" d="M3.5 5H12.5A2.5 2.5 0 0 1 15 7.5V12.5A2.5 2.5 0 0 1 12.5 15H3.5A2.5 2.5 0 0 1 1 12.5V7.5A2.5 2.5 0 0 1 3.5 5Z'
                              f'M2 1H4.8L8 9.6L11.2 1H14L9.3 13H6.7Z"/>'),
        off=lambda ink, dark: (f'<path fill="{"#A6A4AD" if dark else "#8A8794"}" fill-rule="evenodd" d="M3.5 1H8.5A2.5 2.5 0 0 1 11 3.5V12.5A2.5 2.5 0 0 1 8.5 15H3.5A2.5 2.5 0 0 1 1 12.5V3.5A2.5 2.5 0 0 1 3.5 1Z'
                               f'M4 3H15V5H6V7H14V9H6V11H15V13H4Z"/>'),
        tile=None,
        note=('Ý tưởng: chữ V khoét vào một ô thấp và hai cánh nhô ra khỏi cạnh trên — chữ “phá ô”. Không dùng màu phụ: '
              '#5844C8 / #8B7BFF và một xám #8A8794 / #A6A4AD cho trạng thái tắt. Phần chữ nằm trong ô là lỗ trong suốt nên '
              'dùng được trên mọi taskbar; chỉ đổi tông mực theo nền. Khay là chính dấu đó; tắt = chữ E khoét vào ô hẹp, '
              'ba thanh ngang nhô ra cạnh phải, màu xám. Ở 16 px: cánh V nhô 4 px; chữ E nhô ra dễ thành răng lược.')),
    'C': dict(
        name='Ink', style='chữ V bút lông hai mực: nét đậm mực đen, nét thanh mực tím',
        palette=[(INK, 'nét đậm (sáng)'), (PAPER, 'nét đậm (tối) / nền ô'), (ACC, 'nét thanh (sáng)'), (ACC_D, 'nét thanh (tối)')],
        on=lambda ink, dark: C_V.replace('{a}', PAPER if dark else INK).replace('{b}', ACC_D if dark else ACC),
        off=lambda ink, dark: C_E.replace('{a}', '#A8A8A8' if dark else '#6B6B70').replace('{b}', '#A8A8A8' if dark else '#6B6B70'),
        tile=lambda: '<rect x="0.25" y="0.25" width="15.5" height="15.5" rx="3.3" fill="#F7F7F5" stroke="#D9D4CA" stroke-width="0.5"/><g transform="translate(1.6 1.6) scale(.8)">'
                     + C_V.replace('{a}', INK).replace('{b}', ACC) + '</g>',
        note=('Ý tưởng: chữ V viết bằng bút lông hai mực — nét xuống đậm màu mực, nét lên thanh màu mực tím, nét lên vượt nhẹ '
              'qua chỗ gặp như khi viết tay. Bảng màu: #1C1C1A / #F2F2F5 (mực theo nền), #5844C8 / #8B7BFF (nét tím), ô ứng '
              'dụng nền giấy #F7F7F5. Khay dùng đúng hai mực ở 16 px; tắt = chữ E cùng nét bút nhưng một màu xám #6B6B70 / '
              '#A8A8A8 (mất màu tím = tắt). Ở 16 px: nét thanh 1.9 px — mảnh nhất cho phép; cần 2 biến thể taskbar.')),
    'D': dict(
        name='Monogram', style='chữ V khối + một dấu sắc màu tím',
        palette=[(INK, 'chữ V (sáng) / nền ô'), (PAPER, 'chữ V (tối)'), (ACC_D, 'dấu sắc (trên ô tối và taskbar tối)'), (ACC, 'dấu sắc (taskbar sáng)')],
        on=lambda ink, dark: (f'<polygon fill="{PAPER if dark else INK}" points="2,5 4.8,5 8,12.3 11.2,5 14,5 9.6,15 6.4,15"/>'
                              f'<path d="M10.4 3.4L13.2 1.3" stroke="{ACC_D if dark else ACC}" stroke-width="2.2" stroke-linecap="round"/>'),
        off=lambda ink, dark: (f'<g fill="{"#C5C5C5" if dark else "#57524A"}"><rect x="3" y="5" width="3" height="10"/>'
                               f'<rect x="3" y="5" width="10" height="2"/><rect x="3" y="9" width="9" height="2"/><rect x="3" y="13" width="10" height="2"/></g>'),
        tile=lambda: f'<rect width="16" height="16" rx="3.5" fill="{INK}"/><g transform="translate(2 1.6) scale(.75)">'
                     f'<polygon fill="#FFFFFF" points="2,5 4.8,5 8,12.3 11.2,5 14,5 9.6,15 6.4,15"/>'
                     f'<path d="M10.4 3.4L13.2 1.3" stroke="{ACC_D}" stroke-width="2.2" stroke-linecap="round"/></g>',
        note=('Ý tưởng: chữ V khối kèm một dấu sắc màu tím — dấu thanh là thứ làm chữ thành tiếng Việt. Bảng màu: mực '
              '#1C1C1A / #F2F2F5, dấu sắc #5844C8 / #8B7BFF, ô ứng dụng nền mực với chữ trắng. Khay: bật = V + dấu sắc; '
              'tắt = chữ E khối, không có dấu, màu xám #57524A / #C5C5C5 (quy tắc: bỏ dấu = tắt tiếng Việt). Ở 16 px dấu sắc '
              'là một nét 2.2 px dài 3.5 px, cách đỉnh chữ 0.7 px; nếu khay ở 100 % mà dính vào chữ thì bỏ dấu, giữ V.')),
}

def svg16(body):
    return f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" width="16" height="16">{body}</svg>'

def mark_for_lockup(k):
    c = CONCEPTS[k]
    return c['tile']() if c['tile'] else c['on'](INK, False)

def lockup(k):
    w = 24 + 10 + WORD_W
    return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} 24" width="{w}" height="24">'
            f'<g transform="scale(1.5)">{mark_for_lockup(k)}</g><g transform="translate(34 0)">{WORD.replace("{c}", INK)}</g></svg>')

def write():
    for k, c in CONCEPTS.items():
        d = HERE / k
        d.mkdir(exist_ok=True)
        (d / 'lockup.svg').write_text(lockup(k), encoding='utf-8')
        (d / 'tile.svg').write_text(svg16(mark_for_lockup(k)), encoding='utf-8')
        (d / 'tray-v.svg').write_text(svg16(c['on'](INK, False)), encoding='utf-8')
        (d / 'tray-e.svg').write_text(svg16(c['off'](INK, False)), encoding='utf-8')
        (d / 'tray-v-dark.svg').write_text(svg16(c['on'](PAPER, True)), encoding='utf-8')
        (d / 'tray-e-dark.svg').write_text(svg16(c['off'](PAPER, True)), encoding='utf-8')
        (d / 'notes.md').write_text(f'# {k} — {c["name"]}\n\n{c["note"]}\n', encoding='utf-8')

def render():
    for k in CONCEPTS:
        d = HERE / k
        lw = 480; lh = round(480 * 24 / (34 + WORD_W))
        jobs = [('lockup-480.png', 'lockup.svg', lw, lh), ('tile-128.png', 'tile.svg', 128, 128), ('tile-64.png', 'tile.svg', 64, 64),
                ('tile-32.png', 'tile.svg', 32, 32), ('titlebar-20.png', 'tile.svg', 20, 20),
                ('tray-v-16.png', 'tray-v.svg', 16, 16), ('tray-e-16.png', 'tray-e.svg', 16, 16),
                ('tray-v-20.png', 'tray-v.svg', 20, 20), ('tray-e-20.png', 'tray-e.svg', 20, 20),
                ('tray-v-16-dark.png', 'tray-v-dark.svg', 16, 16), ('tray-e-16-dark.png', 'tray-e-dark.svg', 16, 16),
                ('tray-v-20-dark.png', 'tray-v-dark.svg', 20, 20), ('tray-e-20-dark.png', 'tray-e-dark.svg', 20, 20)]
        x, pos = 0, []
        for fn, src, w, h in jobs:
            pos.append((fn, src, x, w, h)); x += w + 8
        page, shot = d / '_r.html', d / '_r.png'
        page.write_text('<!doctype html><html><body style="margin:0;background:transparent">' + ''.join(
            f'<img src="{src}" style="position:absolute;left:{px}px;top:0;width:{w}px;height:{h}px">' for _, src, px, w, h in pos)
            + '</body></html>', encoding='utf-8')
        subprocess.run([EDGE, '--headless=new', '--disable-gpu', '--force-device-scale-factor=1', '--hide-scrollbars',
                        '--default-background-color=00000000', f'--window-size={x + 8},{lh + 8}', f'--screenshot={shot}', page.as_uri()],
                       capture_output=True)
        im = Image.open(shot).convert('RGBA')
        for fn, _, px, w, h in pos:
            im.crop((px, 0, px + w, h)).save(d / fn)
        page.unlink(); shot.unlink()

def sheet():
    secs = []
    for k, c in CONCEPTS.items():
        chips = ''.join(f'<span class="chip"><i style="background:{h}"></i>{h} <small>{u}</small></span>' for h, u in c['palette'])
        secs.append(f'''<section><h2><span class="id">{k}</span>{c["name"]} <small>{c["style"]}</small></h2>
<div class="r1">
  <div class="lk"><img src="{k}/lockup-480.png" width="480"></div>
  <div class="tiles"><img src="{k}/tile-128.png" width="128" height="128"><img src="{k}/tile-64.png"><img src="{k}/tile-32.png"></div>
  <div class="pal">{chips}</div>
</div>
<div class="r2">
  <div class="trays"><div class="tb l"><figure><img class="px" src="{k}/tray-v-16.png"><figcaption>V · bật</figcaption></figure><figure><img class="px" src="{k}/tray-e-16.png"><figcaption>E · tắt</figcaption></figure></div>
  <div class="tb d"><figure><img class="px" src="{k}/tray-v-16-dark.png"><figcaption>V · bật</figcaption></figure><figure><img class="px" src="{k}/tray-e-16-dark.png"><figcaption>E · tắt</figcaption></figure></div></div>
  <div class="ctx">
    <div class="bar l"><img src="{k}/titlebar-20.png"><b>VietKB</b><i></i><span>–</span><span>✕</span></div>
    <div class="bar d"><img src="{k}/titlebar-20.png"><b>VietKB</b><i></i><span>–</span><span>✕</span></div>
    <div class="real"><span class="rl"><img src="{k}/tray-v-16.png"><img src="{k}/tray-e-16.png"><img src="{k}/tray-v-20.png"><img src="{k}/tray-e-20.png"></span>
    <span class="rd"><img src="{k}/tray-v-16-dark.png"><img src="{k}/tray-e-16-dark.png"><img src="{k}/tray-v-20-dark.png"><img src="{k}/tray-e-20-dark.png"></span><small>khay thật 16 · 20 px</small></div>
  </div>
</div></section>''')
    html = f'''<!doctype html><html lang="vi"><head><meta charset="utf-8"><title>VietKB — logo màu v4</title>
<style>
body{{margin:0;padding:18px 24px;background:#E7E4DE;font:400 12.5px/18px "Segoe UI Variable Text","Segoe UI",sans-serif;color:#1C1C1A;width:1052px}}
h1{{font:600 21px/28px "Segoe UI Variable Display","Segoe UI",sans-serif;margin:0}} .lead{{margin:2px 0 4px;color:#57524A}}
section{{background:#FAF9F6;border-radius:10px;box-shadow:0 0 0 1px #E0DBD2;padding:8px 14px 10px;margin-top:8px}}
h2{{margin:0 0 6px;font:600 14px/20px "Segoe UI Variable Text","Segoe UI",sans-serif;display:flex;gap:8px;align-items:center}} h2 small{{font-weight:400;color:#6B655B}}
.id{{display:inline-grid;place-items:center;width:20px;height:20px;border-radius:5px;background:#1C1C1A;color:#fff;font-size:11px}}
.r1,.r2{{display:flex;gap:16px;align-items:flex-start}} .r2{{margin-top:8px}}
.lk{{width:480px;height:128px;display:flex;align-items:center;background:#F7F7F5;border-radius:8px;box-shadow:inset 0 0 0 1px #E8E5DF}}
.tiles{{display:flex;gap:10px;align-items:flex-end}}
.pal{{display:flex;flex-direction:column;gap:4px}} .chip{{display:flex;align-items:center;gap:6px;font:500 11.5px/16px "Cascadia Mono",Consolas,monospace}}
.chip i{{width:16px;height:16px;border-radius:4px;box-shadow:inset 0 0 0 1px rgba(0,0,0,.15)}} .chip small{{font:400 11.5px/16px "Segoe UI Variable Small","Segoe UI",sans-serif;color:#6B655B}}
.trays{{display:flex;gap:8px}} .tb{{display:flex;gap:6px;padding:6px 6px 2px;border-radius:8px}} .tb.l{{background:#F3F3F3}} .tb.d{{background:#202020;color:#C5C5C5}}
figure{{margin:0;text-align:center}} figcaption{{font:500 11px/14px "Segoe UI Variable Small","Segoe UI",sans-serif;margin:2px 0 3px}}
.px{{width:128px;height:128px;image-rendering:pixelated;display:block}}
.ctx{{display:flex;flex-direction:column;gap:6px}}
.bar{{width:420px;height:36px;display:flex;align-items:center;gap:8px;padding-left:14px;border-radius:8px 8px 0 0;box-shadow:0 0 0 1px rgba(0,0,0,.08)}}
.bar.l{{background:#F2F0EB}} .bar.d{{background:#202020;color:#fff}} .bar b{{font:600 13px/20px "Segoe UI Variable Text","Segoe UI",sans-serif}} .bar i{{flex:1}} .bar span{{width:40px;text-align:center;opacity:.7;font-size:11px}}
.real{{display:flex;align-items:center;gap:8px}} .real span{{display:flex;gap:8px;align-items:center;padding:6px 8px;border-radius:6px}}
.rl{{background:#F3F3F3}} .rd{{background:#202020}} .real small{{color:#6B655B}}
</style></head><body>
<h1>VietKB · logo màu — 4 hệ nhận diện</h1>
<p class="lead">Logo trước, khay suy ra sau. V = bật tiếng Việt, E = tắt (khác bằng hình chữ; màu chỉ củng cố). Khay 16 px phóng 8 lần, pixel gốc.</p>
{"".join(secs)}
</body></html>'''
    (HERE / 'index.html').write_text(html, encoding='utf-8')
    subprocess.run([EDGE, '--headless=new', '--disable-gpu', '--force-device-scale-factor=1', '--hide-scrollbars',
                    '--window-size=1100,1700', f'--screenshot={HERE / "sheet.png"}', (HERE / 'index.html').as_uri()], capture_output=True)

if __name__ == '__main__':
    write(); render(); sheet(); print('ok')
