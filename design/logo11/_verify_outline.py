# Verify the rendered matte keycap outline and report the shadow separately.
# Run: python design/logo11/_verify_outline.py
import math, pathlib, re, subprocess, sys
from PIL import Image, ImageFilter
from _build import T

HERE = pathlib.Path(__file__).resolve().parent
KC = HERE / 'keycap'
EDGE = r'C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe'
MASK_PX = 2048
RAYS = 64

def silhouette_mask(k):
    svg = (KC / f'{k}-master.svg').read_text(encoding='utf-8')
    inner = re.search(r'<clipPath id="s' + k + r'">(.*?)</clipPath>', svg, re.S).group(1)
    page = HERE / f'_mask{k}.html'
    shot = HERE / f'_mask{k}.png'
    page.write_text('<!doctype html><html><body style="margin:0"><svg xmlns="http://www.w3.org/2000/svg" '
                    f'viewBox="0 0 512 512" width="{MASK_PX}" height="{MASK_PX}"><g fill="#000">{inner}</g>'
                    '</svg></body></html>', encoding='utf-8')
    subprocess.run([EDGE, '--headless=new', '--disable-gpu', '--force-device-scale-factor=1', '--hide-scrollbars',
                    '--default-background-color=00000000', f'--window-size={MASK_PX},{MASK_PX}',
                    f'--screenshot={shot}', page.as_uri()], check=True, capture_output=True)
    alpha = Image.open(shot).convert('RGBA').getchannel('A')
    page.unlink(); shot.unlink()
    return alpha

def contour(alpha):
    px = alpha.load(); w, h = alpha.size
    coords = [(x, y) for y in range(0, h, 4) for x in range(0, w, 4) if px[x, y] > 127]
    cx = sum(x for x, _ in coords) / len(coords)
    cy = sum(y for _, y in coords) / len(coords)
    scale = 512 / w
    pts = []
    for i in range(RAYS):
        theta = 2 * math.pi * i / RAYS
        dx, dy = math.cos(theta), math.sin(theta)
        last = None
        distance = 0
        while distance < 2 * w:
            x, y = cx + dx * distance, cy + dy * distance
            if not (0 <= x < w and 0 <= y < h):
                break
            if px[int(x), int(y)] > 127:
                last = (x * scale, y * scale)
            distance += 1
        if last:
            pts.append((last, (cx * scale, cy * scale)))
    return pts

def is_shadow(r, g, b):
    return max(r, g, b) < 70

def outside_counts(k):
    alpha = silhouette_mask(k)
    inside = alpha.point(lambda a: 255 if a > 127 else 0)
    within2 = inside.filter(ImageFilter.MaxFilter(17))  # 8 px at 4x = 2 px
    a2 = within2.load()
    im = Image.open(KC / f'{k}-512.png').convert('RGBA')
    px = im.load(); w, h = im.size
    samples = dict(n=0, bad=0, shadow=0, shadow_max=0)
    global_out = dict(n=0, bad=0, shadow=0, shadow_max=0)

    def classify(bucket, x, y):
        r, g, b, a = px[x, y]
        if a == 0:
            return
        bucket['n'] += 1
        if is_shadow(r, g, b):
            bucket['shadow'] += 1
            bucket['shadow_max'] = max(bucket['shadow_max'], a)
        else:
            bucket['bad'] += 1

    for y in range(h):
        for x in range(w):
            if a2[x * 4 + 2, y * 4 + 2] == 0:
                classify(global_out, x, y)

    ap = alpha.load()
    for (x0, y0), (cx, cy) in contour(alpha):
        dx, dy = x0 - cx, y0 - cy
        length = math.hypot(dx, dy) or 1
        ux, uy = dx / length, dy / length
        for distance in (2.0, 2.5, 3.0, 3.5, 4.0):
            for tangent in (-1.5, -0.75, 0.0, 0.75, 1.5):
                x = int(round(x0 + ux * distance - uy * tangent))
                y = int(round(y0 + uy * distance + ux * tangent))
                if not (0 <= x < w and 0 <= y < h):
                    continue
                if a2[x * 4 + 2, y * 4 + 2] > 0:
                    continue
                samples['n'] += 1
                r, g, b, a = px[x, y]
                if a == 0:
                    continue
                if is_shadow(r, g, b):
                    samples['shadow'] += 1
                    samples['shadow_max'] = max(samples['shadow_max'], a)
                else:
                    samples['bad'] += 1
    return samples, global_out

def rendered_metrics(k):
    im = Image.open(KC / f'{k}-512.png').convert('RGBA')
    px = im.load(); w, h = im.size
    rgb = lambda hexcolor: tuple(int(hexcolor[i:i + 2], 16) for i in (1, 3, 5))
    palette = T[k]
    face = rgb(palette['face'])
    wall, band = rgb(palette['wall']), rgb(palette['band'])
    letter_colors = [rgb(palette['a']), rgb(palette['b'])]
    near = lambda p, t, tol: max(abs(p[i] - t[i]) for i in range(3)) <= tol
    face_xy, body_xy, letter_xy = [], [], []
    for y in range(h):
        for x in range(w):
            p = px[x, y]
            if p[3] > 250 and near(p, face, 8):
                face_xy.append((x, y))
            if p[3] > 200 and (near(p, wall, 8) or near(p, band, 8)):
                body_xy.append((x, y))
            if p[3] > 200 and any(near(p, color, 4) for color in letter_colors):
                letter_xy.append((x, y))
    x0, x1 = min(x for x, _ in face_xy), max(x for x, _ in face_xy) + 1
    y0, y1 = min(y for _, y in face_xy), max(y for _, y in face_xy) + 1
    fw, fh = x1 - x0, y1 - y0
    cx, cy = (x0 + x1 - 1) / 2, (y0 + y1 - 1) / 2
    lx0, lx1 = min(x for x, _ in letter_xy), max(x for x, _ in letter_xy)
    ly0, ly1 = min(y for _, y in letter_xy), max(y for _, y in letter_xy)
    letter_delta = ((lx0 + lx1) / 2 - cx, (ly0 + ly1) / 2 - cy)
    right_pixels = [x for x, y in body_xy if abs(y - cy) <= 1]
    bottom_pixels = [y for x, y in body_xy if abs(x - cx) <= 1]
    right = max(right_pixels) - (x1 - 1)
    bottom = max(bottom_pixels) - (y1 - 1)
    body_top = min(y for _, y in body_xy)
    return dict(bounds=(x0, y0, fw, fh), ratio=fw / fh, delta=letter_delta,
                overhang=(right, bottom), top_overhang=y0 - body_top)

def main():
    failed = False
    print('Outside samples at 2–4 px from the silhouette (drop shadow listed separately):')
    for k in 've':
        sampled, global_out = outside_counts(k)
        m = rendered_metrics(k)
        print(f'{k.upper()}: face {m["bounds"]}, ratio {m["ratio"]:.4f}; '
              f'letter centre Δ {m["delta"][0]:.2f}, {m["delta"][1]:.2f} units; '
              f'body overhang {m["overhang"][0] * 420 / 512:.1f} / {m["overhang"][1] * 420 / 512:.1f} px @420 '
              f'(top {m["top_overhang"]} px)')
        print(f'  2–4 px samples: {sampled["n"]} tested, {sampled["bad"]} non-shadow outside pixels, '
              f'{sampled["shadow"]} shadow pixels, max shadow alpha {sampled["shadow_max"]}')
        print(f'  all pixels >=2 px outside: {global_out["n"]} alpha-positive, '
              f'{global_out["bad"]} non-shadow, {global_out["shadow"]} shadow, '
              f'max shadow alpha {global_out["shadow_max"]}')
        if sampled['bad'] or global_out['bad']:
            failed = True
        if not (0.995 <= m['ratio'] <= 1.005):
            failed = True
        if max(abs(v) for v in m['delta']) > 2:
            failed = True
        if m['overhang'] != (30, 40) or m['top_overhang'] != 0:
            failed = True
    if failed:
        print('FAIL')
        return 1
    print('PASS')
    return 0

if __name__ == '__main__':
    sys.exit(main())
