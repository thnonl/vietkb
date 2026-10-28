# WCAG 2.x contrast for every text/background pair used in the screens.
import re, sys, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).parent))
from build import LIGHT, DARK

def tok(src):
    return {k: v for k, v in re.findall(r'--([\w-]+):(#[0-9A-Fa-f]{6})', src)}

def lum(h):
    c = [int(h[i:i+2], 16) / 255 for i in (1, 3, 5)]
    c = [x / 12.92 if x <= 0.04045 else ((x + 0.055) / 1.055) ** 2.4 for x in c]
    return 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]

def cr(a, b):
    la, lb = sorted((lum(a), lum(b)), reverse=True)
    return (la + 0.05) / (lb + 0.05)

TEXT = [
    ('t1', 'surface', 'labels, titles'), ('t1', 'plane', 'selected nav'), ('t1', 'raised', 'table, controls'),
    ('t1', 'float', 'popover, toast'), ('t1', 'key-bot', 'keycap legend'), ('t1', 'acc-tint', 'selected row text'),
    ('t1', 'bad-tint', 'admin notice'), ('t1', 'hover', 'hovered row'),
    ('t2', 'surface', 'subtitles, strip chips'), ('t2', 'plane', 'nav items'), ('t2', 'raised', 'table header'),
    ('t2', 'sunken', 'segmented (off)'), ('t2', 'warn-tint', 'warning detail'),
    ('t3', 'surface', 'row descriptions'), ('t3', 'plane', 'nav counts, privacy'), ('t3', 'raised', 'paths, placeholder'),
    ('t3', 'hover', 'desc on hovered row'), ('t3', 'acc-tint', 'desc on capturing row'), ('t3', 'sunken', 'capture placeholder'),
    ('t3', 'float', 'popover meta'),
    ('acc-text', 'surface', 'links'), ('acc-text', 'acc-tint', 'highlighted picker exe'), ('acc-text', 'acc-tint-2', 'accent tag, byte chip'),
    ('acc-text', 'plane', 'selected nav icon'),
    ('on-acc', 'acc', 'primary button, checkbox tick'),
    ('ok', 'surface', 'live status'), ('ok', 'raised', '“Đã lưu”'),
    ('warn', 'warn-tint', 'recording chip, Trùng tag'), ('warn', 'surface', 'conflict message icon line'),
    ('bad', 'raised', 'danger button'), ('bad', 'surface', 'danger icon'),
]
TEXT += [('t1', 'raised', 'Sitka output in Thử gõ / empty demo'), ('t2', 'raised', 'tone pill example (off)'),
         ('acc-text', 'raised', 'tone pill name (on)'), ('t3', 'sunken', 'tone pill name (off)')]
NONTEXT = [
    ('acc', 'surface', 'toggle on track'), ('acc', 'raised', 'toggle on track (raised)'),
    ('focus', 'surface', 'focus ring'), ('focus', 'raised', 'focus ring on fields (Thử gõ)'),
    ('rail', 'acc-tint', 'selected-row rail on its tint'),
    ('ctl-line', 'raised', 'checkbox / radio border'), ('bad', 'surface', 'danger toggle on'),
    ('ctl-line', 'surface', 'toggle OFF outline vs surface'), ('ctl-line', 'hover', 'toggle OFF outline vs hovered row'),
    ('ctl-line', 'tg-off', 'toggle OFF outline vs its own track (info, inner edge)'),
    ('acc-tint', 'raised', 'selected-row tint vs table (luminance delta, info)'),
]

def report(name, src):
    t = tok(src)
    out = [f'### {name}', '', '| Foreground | Background | Use | Ratio | AA |', '|---|---|---|---|---|']
    for f, b, use in TEXT:
        r = cr(t[f], t[b]); out.append(f'| `{f}` {t[f]} | `{b}` {t[b]} | {use} | {r:.2f} | {"pass" if r >= 4.5 else "FAIL"} |')
    out += ['', '| Component | Against | Use | Ratio | 1.4.11 (3:1) |', '|---|---|---|---|---|']
    for f, b, use in NONTEXT:
        r = cr(t[f], t[b]); v = 'n/a' if 'info' in use else ('pass' if r >= 3 else 'below')
        out.append(f'| `{f}` {t[f]} | `{b}` {t[b]} | {use} | {r:.2f} | {v} |')
    return '\n'.join(out)

if __name__ == '__main__':
    print(report('Light', LIGHT)); print(); print(report('Dark', DARK))
