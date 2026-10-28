# Logo v10 — retro keycap

| Tile | Face | Lip (2 steps darker, same hue) | Top highlight | Rim | Letter |
|---|---|---|---|---|---|
| V (bật) | `#FFEBDD` | `#F5C5A3` | `#FFF8F3` | `#C94810` | pink `#CF1A65` (heavy) + orange `#CF5605` (thin) |
| E (tắt) | `#E0F1FD` | `#A7D0F2` | `#F4FAFE` | `#1E72D2` | blue `#1C6CC6` (rim hue, one step darker) |

No gradient anywhere (face, lip, rim, highlight are flat colours); the only soft element is the contact shadow below the tile.

## Contrast
| Tile | Element | vs #F3F3F3 | vs #202020 | vs #808080 | vs own face |
|---|---|---|---|---|---|
| V | rim `#C94810` | 4.28 | 3.43 | 1.20 | 4.11 |
| V | face `#FFEBDD` | 1.04 | 14.10 | 3.42 | — |
| E | rim `#1E72D2` | 4.31 | 3.41 | 1.21 | 4.14 |
| E | face `#E0F1FD` | 1.04 | 14.10 | 3.42 | — |

| Letter stroke | vs tile face |
|---|---|
| V heavy (pink) `#CF1A65` | 4.56 |
| V thin (orange) `#CF5605` | 3.65 |
| E (blue) `#1C6CC6` | 4.53 |

- Rims clear 3:1 on both taskbars (≈ 4.3 / 3.4). Deep orange had to stay at relative luminance ≈ 0.17 for that: lighter fails on `#202020`, darker fails nothing but turns brown.
- V thin orange is 3.65:1 on the face (passes 3:1, not 4.5): reaching 4.5 would push it to the rim's rust and the two oranges would merge — kept as orange, noted as the weakest pair. Pink and blue clear 4.5.
- Mid-grey `#808080`: the rim fails (1.2) but the light face is 3.42:1 against it, so the tile still separates from mid-grey by its face — the opaque tile solves the case the transparent mark could not.
- **No dark variant**: the tile is opaque and its rim clears 3:1 on both taskbars, so one version serves light and dark.

## Sizes
- **512 / 256 (full keycap)**: ONE rounded rectangle 456 × 462 units (radius 88) is the whole key; its interior is divided by a straight horizontal edge at y 398 (83 % of the height): face above, front lip below (78 units: 64 px at 420, 29 px at 192) filled to the outer contour with square inner corners and one lighter tonal step (`lipt`) in its top 26 units; the 8-unit rim strokes the outer silhouette only; 4-unit highlight along the top inner edge; soft contact shadow (blur 9, 28 % ink) below the tile only. Letter at 56 % of the box.
- **48–128 (flat keycap)**: lip, highlight and shadow removed; rim thickened to 12 units so it stays ≥ 1.3 px at 48.
- **16 / 20 / 32 (redrawn 16 px tile)**: whole-pixel tile, radius 3.5, **rim 1 px** (a scaled rim would be 0.3 px), no lip, no highlight, no shadow; letter redrawn in a 10 px field — V heavy 2.7 → 2.3 px, thin 2.2 px, interior ~1.9 px at mid-height; E 2 px stem and bars with 2 px counters.
