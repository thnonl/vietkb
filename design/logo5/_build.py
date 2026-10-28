# VietKB logo C (Ink) in six palettes. Construction is copied verbatim from design/logo4 (concept C):
# heavy ink downstroke + 1.9 px accent upstroke on a 16 px grid, paper tile, drawn wordmark. Only colours change.
# Run: python design/logo5/_build.py
import pathlib, subprocess
from PIL import Image

HERE = pathlib.Path(__file__).resolve().parent
EDGE = r'C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe'
INK, PAPER_UI, PAPER_TRAY = '#1C1C1A', '#F7F7F5', '#F2F2F5'
OFF_L, OFF_D = '#6B6B70', '#A8A8A8'

# ---- construction (unchanged from logo4 C)
C_V = ('<polygon fill="{a}" points="1.5,1.5 4.6,1.5 8.6,14.6 6.4,14.6"/>'
       '<path d="M7.6 14.8L13.6 1.7" fill="none" stroke="{b}" stroke-width="1.9" stroke-linecap="round"/>')
C_E = ('<polygon fill="{a}" points="3,1.4 5.4,1.4 5.4,14.6 3,14.6"/>'
       '<path d="M4.4 2.4Q8.5 1.6 12.6 1.9M4.4 8Q7.8 7.4 11 7.6M4.4 13.6Q8.6 13 13.2 12.9" fill="none" stroke="{b}" stroke-width="1.9" stroke-linecap="round"/>')
def B_PATH(x, top=5, mid=11.6, base=19):
    return f'M{x} {base}V{top}h4.5a3.3 3.3 0 0 1 0 {mid-top:.1f}H{x}m0 0h5.4a3.7 3.7 0 0 1 0 {base-mid:.1f}H{x}'
WORD = (f'<g fill="none" stroke="{{c}}" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">'
        f'<path d="M2 5L8 19L14 5"/><path d="M18.5 10V19"/><path d="M22.5 14.5H31.5A4.5 4.5 0 1 0 30.2 17.8"/>'
        f'<path d="M35.5 6V16.5Q35.5 19 38.1 19M33 10H38.1"/><path d="M42.5 5V19M51.5 5L43.1 13.4M46.3 10.8L52 19"/>'
        f'<path d="{B_PATH(55.5)}"/></g><circle cx="18.5" cy="6" r="1.9" fill="{{c}}"/>')
WORD_W = 66

# ---- palettes: light accent (paper tile, light UI, light taskbar) / dark accent (ink tile, dark UI, dark taskbar)
# Light accents p2-p6 are tuned to relative luminance ~0.15 and dark accents to ~0.245: the only band where the
# accent clears 3:1 against BOTH the ink stroke and the paper (and 4.5:1 on paper for UI text use).
PAL = {
    'p1': ('Mực tím', '#5844C8', '#8B7BFF', 'Mốc so sánh: mực tím đang dùng trong giao diện.'),
    'p2': ('Mực xanh ngọc', '#0E7A6B', '#119986', 'Màu nước Việt — ngọc bích, vịnh, ruộng bậc thang mùa nước.'),
    'p3': ('Mực đỏ son', '#CD2D21', '#E35C52', 'Son mài, câu đối, dấu đỏ trên tranh in — Tết và chữ viết tay.'),
    'p4': ('Mực chàm', '#4863D1', '#6D83DA', 'Chàm nhuộm vải vùng cao phía Bắc; xanh tin cậy, gần mực tím nhưng lạnh hơn.'),
    'p5': ('Mực nâu đất', '#AE502E', '#CF6C49', 'Đất nung, ngói đỏ — ấm, trầm, không ồn.'),
    'p6': ('Mực vàng kim', '#8A660B', '#AE800E', 'Mực đen ánh vàng kim — sang nhất và rủi ro nhất: vàng tối dễ thành màu bùn.'),
}

def lum(h):
    c = [int(h[i:i + 2], 16) / 255 for i in (1, 3, 5)]
    c = [x / 12.92 if x <= .04045 else ((x + .055) / 1.055) ** 2.4 for x in c]
    return .2126 * c[0] + .7152 * c[1] + .0722 * c[2]
def cr(a, b):
    x, y = sorted((lum(a), lum(b)), reverse=True)
    return (x + .05) / (y + .05)

def svg16(body):
    return f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" width="16" height="16">{body}</svg>'
def tile(acc, dark=False):
    bg, fg, line = (INK, PAPER_TRAY, '#3A3A3A') if dark else (PAPER_UI, INK, '#D9D4CA')
    return (f'<rect x="0.25" y="0.25" width="15.5" height="15.5" rx="3.3" fill="{bg}" stroke="{line}" stroke-width="0.5"/>'
            f'<g transform="translate(1.6 1.6) scale(.8)">{C_V.replace("{a}", fg).replace("{b}", acc)}</g>')
def lockup(acc, dark=False):
    w = 34 + WORD_W
    word_c = PAPER_TRAY if dark else INK
    return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} 24" width="{w}" height="24">'
            f'<g transform="scale(1.5)">{tile(acc, dark)}</g><g transform="translate(34 0)">{WORD.replace("{c}", word_c)}</g></svg>')

def numbers(lt, dk):
    n = {
        'ink / tile sáng': cr(INK, PAPER_UI), 'accent / tile sáng': cr(lt, PAPER_UI), 'accent / ink': cr(lt, INK),
        'giấy / tile tối': cr(PAPER_TRAY, INK), 'accent tối / tile tối': cr(dk, INK), 'accent tối / nét giấy': cr(dk, PAPER_TRAY),
        'accent / taskbar sáng #F3F3F3': cr(lt, '#F3F3F3'), 'accent tối / taskbar tối #202020': cr(dk, '#202020'),
        'UI: accent chữ trên #F7F7F5 (≥4.5)': cr(lt, PAPER_UI), 'UI: chữ trắng trên nút accent (≥4.5)': cr('#FFFFFF', lt),
        'UI: accent tối / #202020 (≥3)': cr(dk, '#202020'), 'UI: accent tối / #272727 (≥3)': cr(dk, '#272727'),
        'UI: chữ ink trên nút accent tối (≥4.5)': cr(INK, dk),
    }
    return n

def write_and_render():
    out = {}
    for key, (name, lt, dk, why) in PAL.items():
        d = HERE / key
        d.mkdir(exist_ok=True)
        files = {
            'lockup.svg': lockup(lt), 'lockup-dark.svg': lockup(dk, True), 'tile.svg': svg16(tile(lt)), 'tile-dark.svg': svg16(tile(dk, True)),
            'tray-v.svg': svg16(C_V.replace('{a}', INK).replace('{b}', lt)),
            'tray-e.svg': svg16(C_E.replace('{a}', OFF_L).replace('{b}', OFF_L)),
            'tray-v-dark.svg': svg16(C_V.replace('{a}', PAPER_TRAY).replace('{b}', dk)),
            'tray-e-dark.svg': svg16(C_E.replace('{a}', OFF_D).replace('{b}', OFF_D)),
        }
        for fn, body in files.items():
            (d / fn).write_text(body, encoding='utf-8')
        n = numbers(lt, dk)
        out[key] = n
        flags = [k for k, v in n.items() if (('≥4.5' in k and v < 4.5) or ('≥3' in k and v < 3) or
                 (k in ('accent / tile sáng', 'accent / ink', 'accent tối / tile tối', 'accent tối / nét giấy',
                        'accent / taskbar sáng #F3F3F3', 'accent tối / taskbar tối #202020') and v < 3))]
        table = '\n'.join(f'| {k} | {v:.2f} |' for k, v in n.items())
        ui = ('Không cần đổi token giao diện (đây là màu hiện tại).' if key == 'p1' else
              f'Cần đổi token: `--acc`/`--focus`/`--rail` sáng = `{lt}`, `--acc-text` sáng = `{lt}`; tối: `--acc`/`--rail` = `{dk}`, '
              f'chữ trên nền accent tối = `#1C1C1A`; các tông nhạt (`--acc-tint`, `--acc-tint-2`, `--acc-line`) suy lại từ cùng sắc độ.')
        (d / 'notes.md').write_text(
            f'# {key} — {name}\n\n{why}\n\nBảng màu: mực `{INK}` · giấy (ô sáng) `{PAPER_UI}` · nhấn sáng `{lt}` · nhấn tối `{dk}` · '
            f'ô tối nền mực `{INK}` với nét giấy `{PAPER_TRAY}` · tắt (E) xám `{OFF_L}` / `{OFF_D}`.\n\n'
            f'| Cặp | Tỉ lệ |\n|---|---|\n{table}\n\n'
            f'Không đạt: {", ".join(flags) if flags else "không có"}.\n\nGiao diện: {ui}\n', encoding='utf-8')
        jobs = [('lockup-480.png', 'lockup.svg', 480, round(480 * 24 / (34 + WORD_W))),
                ('tile-128.png', 'tile.svg', 128, 128), ('tile-64.png', 'tile.svg', 64, 64), ('tile-32.png', 'tile.svg', 32, 32),
                ('titlebar-20.png', 'tile.svg', 20, 20), ('titlebar-20-dark.png', 'tile-dark.svg', 20, 20)]
        for l in 've':
            for sz in (16, 20):
                jobs += [(f'tray-{l}-{sz}.png', f'tray-{l}.svg', sz, sz), (f'tray-{l}-{sz}-dark.png', f'tray-{l}-dark.svg', sz, sz)]
        x, pos = 0, []
        for fn, src, w, h in jobs:
            pos.append((fn, src, x, w, h)); x += w + 8
        page, shot = d / '_r.html', d / '_r.png'
        page.write_text('<!doctype html><html><body style="margin:0;background:transparent">' + ''.join(
            f'<img src="{src}" style="position:absolute;left:{px}px;top:0;width:{w}px;height:{h}px">' for _, src, px, w, h in pos)
            + '</body></html>', encoding='utf-8')
        subprocess.run([EDGE, '--headless=new', '--disable-gpu', '--force-device-scale-factor=1', '--hide-scrollbars',
                        '--default-background-color=00000000', f'--window-size={x + 8},136', f'--screenshot={shot}', page.as_uri()],
                       capture_output=True)
        im = Image.open(shot).convert('RGBA')
        for fn, _, px, w, h in pos:
            im.crop((px, 0, px + w, h)).save(d / fn)
        page.unlink(); shot.unlink()
    return out

def sheet(nums):
    rows = []
    for key, (name, lt, dk, why) in PAL.items():
        n = nums[key]
        keyn = [('accent / giấy', n['accent / tile sáng']), ('accent / mực', n['accent / ink']), ('tối / mực', n['accent tối / tile tối']),
                ('tối / nét giấy', n['accent tối / nét giấy']), ('UI chữ trên giấy', n['UI: accent chữ trên #F7F7F5 (≥4.5)']),
                ('UI tối / #272727', n['UI: accent tối / #272727 (≥3)'])]
        lim = [3, 3, 3, 3, 4.5, 3]
        cells = ''.join(f'<span class="{"bad" if v < l else ""}">{t} <b>{v:.2f}</b></span>' for (t, v), l in zip(keyn, lim))
        rows.append(f'''<section><h2><span class="id">{key[1]}</span>{name} <code>{lt} / {dk}</code> <small>{why}</small></h2>
<div class="r1"><div class="lk"><img src="{key}/lockup.svg" style="height:64px"></div><div class="lk d"><img src="{key}/lockup-dark.svg" style="height:40px"></div>
<div class="bars"><div class="bar l"><img src="{key}/titlebar-20.png"><b>VietKB</b><i></i><span>–</span><span>✕</span></div>
<div class="bar d"><img src="{key}/titlebar-20-dark.png"><b>VietKB</b><i></i><span>–</span><span>✕</span></div></div></div>
<div class="r2"><div class="tiles"><img src="{key}/tile-128.png" width="128" height="128"><div><img src="{key}/tile-64.png"><img src="{key}/tile-32.png"></div></div>
<div class="tb l"><figure><img class="px" src="{key}/tray-v-16.png"><figcaption>V · bật</figcaption></figure><figure><img class="px" src="{key}/tray-e-16.png"><figcaption>E · tắt</figcaption></figure></div>
<div class="tb d"><figure><img class="px" src="{key}/tray-v-16-dark.png"><figcaption>V · bật</figcaption></figure><figure><img class="px" src="{key}/tray-e-16-dark.png"><figcaption>E · tắt</figcaption></figure></div>
<div class="nums">{cells}<div class="real"><span class="rl"><img src="{key}/tray-v-16.png"><img src="{key}/tray-e-16.png"><img src="{key}/tray-v-20.png"></span><span class="rd"><img src="{key}/tray-v-16-dark.png"><img src="{key}/tray-e-16-dark.png"><img src="{key}/tray-v-20-dark.png"></span></div></div>
</div></section>''')
    html = f'''<!doctype html><html lang="vi"><head><meta charset="utf-8"><title>VietKB — logo C, 6 bộ màu</title>
<style>
body{{margin:0;padding:16px 24px;background:#E7E4DE;font:400 12px/17px "Segoe UI Variable Text","Segoe UI",sans-serif;color:#1C1C1A;width:1052px}}
h1{{font:600 20px/26px "Segoe UI Variable Display","Segoe UI",sans-serif;margin:0}} .lead{{margin:2px 0 2px;color:#57524A}}
section{{background:#FAF9F6;border-radius:10px;box-shadow:0 0 0 1px #E0DBD2;padding:7px 12px 9px;margin-top:8px}}
h2{{margin:0 0 6px;font:600 14px/20px "Segoe UI Variable Text","Segoe UI",sans-serif;display:flex;gap:8px;align-items:center}}
h2 code{{font:500 11.5px/16px "Cascadia Mono",Consolas,monospace;color:#57524A}} h2 small{{font-weight:400;color:#6B655B}}
.id{{display:inline-grid;place-items:center;width:20px;height:20px;border-radius:5px;background:#1C1C1A;color:#fff;font-size:11px}}
.r1,.r2{{display:flex;gap:12px;align-items:center}} .r2{{margin-top:6px;align-items:flex-start}}
.lk{{height:76px;width:300px;display:flex;align-items:center;justify-content:center;background:#F7F7F5;border-radius:8px;box-shadow:inset 0 0 0 1px #E8E5DF}}
.lk.d{{background:#1C1C1A;width:200px}}
.bars{{display:flex;flex-direction:column;gap:6px}} .bar{{width:300px;height:34px;display:flex;align-items:center;gap:8px;padding-left:12px;border-radius:8px 8px 0 0;box-shadow:0 0 0 1px rgba(0,0,0,.08)}}
.bar.l{{background:#F2F0EB}} .bar.d{{background:#202020;color:#fff}} .bar b{{font:600 13px/20px "Segoe UI Variable Text","Segoe UI",sans-serif}} .bar i{{flex:1}} .bar span{{width:36px;text-align:center;opacity:.7;font-size:11px}}
.tiles{{display:flex;gap:8px;align-items:flex-end}} .tiles div{{display:flex;flex-direction:column;gap:6px;align-items:center}}
.tb{{display:flex;gap:6px;padding:5px 6px 1px;border-radius:8px}} .tb.l{{background:#F3F3F3}} .tb.d{{background:#202020;color:#C5C5C5}}
figure{{margin:0;text-align:center}} figcaption{{font:500 11px/14px "Segoe UI Variable Small","Segoe UI",sans-serif;margin:1px 0 2px}}
.px{{width:128px;height:128px;image-rendering:pixelated;display:block}}
.nums{{display:flex;flex-direction:column;gap:1px;font:400 11.5px/16px "Segoe UI Variable Small","Segoe UI",sans-serif;color:#57524A}}
.nums b{{color:#1C1C1A;font-variant-numeric:tabular-nums}} .nums .bad b{{color:#B42318}} .nums .bad{{color:#B42318}}
.real{{display:flex;gap:6px;margin-top:4px}} .real span{{display:flex;gap:6px;align-items:center;padding:4px 6px;border-radius:6px}} .rl{{background:#F3F3F3}} .rd{{background:#202020}}
</style></head><body>
<h1>VietKB · logo C (Ink) — 6 bộ màu</h1>
<p class="lead">Cùng cấu trúc, chỉ đổi màu. Khay 16 px phóng 8 lần (pixel gốc). Số đỏ = dưới ngưỡng (3:1 cho nét/biên, 4.5:1 cho chữ UI).</p>
{"".join(rows)}
</body></html>'''
    (HERE / 'index.html').write_text(html, encoding='utf-8')
    subprocess.run([EDGE, '--headless=new', '--disable-gpu', '--force-device-scale-factor=1', '--hide-scrollbars',
                    '--window-size=1100,1900', f'--screenshot={HERE / "sheet.png"}', (HERE / 'index.html').as_uri()], capture_output=True)

if __name__ == '__main__':
    sheet(write_and_render()); print('ok')
