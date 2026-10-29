# V tile — letter palette (light pink + dark orange)

Face `#FFEBDD`, rim `#C94810`, geometry, 512 master: unchanged.

## Old → new

| Stroke | Old | Luminance | vs face | Hue / sat | New | Luminance | vs face | Hue / sat |
|---|---|---:|---:|---|---|---:|---:|---|
| Heavy | `#CF1A65` | 0.1494 | 4.56 | 335° · 87 % magenta | `#E4507C` | 0.2369 | **3.17** | 342° · 65 % rose pink |
| Thin | `#CF5605` | 0.1993 | 3.65 | 24° · 98 % orange | `#A84A00` | 0.1322 | **4.99** | 26° · 100 % burnt orange |

## Four contrast ratios, both strokes vs face `#FFEBDD`

| Stroke | Old | New |
|---|---:|---:|
| Heavy pink | 4.56 | **3.17** |
| Thin orange | 3.65 | **4.99** |

Floor is 3.0:1 at 16 px — both new values clear it (pink margin +0.17, orange +1.99).

## Separation

- Luminance: pink 0.2369 / orange 0.1322 → **ratio 1.79×** (was 1.33×; target ≥ 1.6×).
- Pink ↔ orange contrast: 1.57:1; hue distance 342° → 26° = 44° through red.
- Same warm family: both keep the orange-red axis (24°/26° on the thin stroke) and the pink stays a red-based rose, so the V still reads as one letter.

## Why it reads at 16 px

The rose sits ~59 % higher in luminance than before and the orange ~34 % lower, so at 16 px the pair separates by lightness first (1.79×), with hue second — the heavy stroke is the light glyph part and the thin stroke the dark one, no muddy shared mid-tone. The lighter pink is not weaker than the dark orange: its 2.7→2.3 px heavy stroke still covers ~2× the solid core pixels of the 2.2 px thin stroke, so no width compensation was needed and the pink is not darkened back toward the floor.
