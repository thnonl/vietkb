# Logo v11 — matte keycap

| Tile | Face | Right wall | Bottom band | Rim | Letter |
|---|---|---|---|---|---|
| V | `#FFEBDD` | `#F3D4C2` | `#E8BFA6` | `#C94810` | pink `#E4507C` + orange `#A84A00` |
| E | `#E0F1FD` | `#C6DFF1` | `#A8C9E2` | `#1E72D2` | `#1C6CC6` |

## Full-size geometry (512 units)

The face is an exact 416 × 416 rounded square with radius 96. Its letter is centred on the face. The body uses the
same rounded-square construction and offsets left / right / bottom / top by 2.5 / 30 / 40 / 0 units. The left
overhang is hidden beneath the rim, leaving only the right wall and bottom band visible. One 8-unit rim follows the
outside silhouette. No bevel or glossy highlight is used. A 2.5-unit inner top-edge shade uses the body-band colour at
8% opacity. The desk shadow uses 14-unit blur, 20 / 12-unit offset and 18% opacity, below the opaque body.

Rendered 512 px measurements:

| Tile | Face bounds | Ratio | Letter centre offset | Body overhang at 420 px (right / bottom) |
|---|---:|---:|---:|---:|
| V | 416 × 416 = 1.000 | 1.000 | 0.0, 0.5 units | 24.6, 32.8 px |
| E | 416 × 416 = 1.000 | 1.000 | -0.5, 0.0 units | 24.6, 32.8 px |

## Flat 16 / 20 / 32 px drawings

These stay flat: one rounded-square face, 1 px rim and letter only. No body, band, shade, bevel or shadow.

| Tile | Face | Rim | Letter |
|---|---|---|---|
| V | `#FFEBDD` | `#C94810` | pink `#E4507C` + orange `#A84A00` |
| E | `#E0F1FD` | `#1E72D2` | blue `#1C6CC6` |

## Contrast
| Tile | Element | vs #F3F3F3 | vs #202020 | vs #808080 |
|---|---|---:|---:|---:|
| V | rim `#C94810` | 4.28 | 3.43 | 1.20 |
| V | face `#FFEBDD` | 1.04 | 14.10 | 3.42 |
| E | rim `#1E72D2` | 4.31 | 3.41 | 1.21 |
| E | face `#E0F1FD` | 1.04 | 14.10 | 3.42 |

| Letter | vs face |
|---|---:|
| V pink `#E4507C` | 3.17 |
| V orange `#A84A00` | 4.99 |
| E blue `#1C6CC6` | 4.53 |
