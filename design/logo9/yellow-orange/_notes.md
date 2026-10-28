# Logo v9 — vàng + cam / xanh trời (owner palette)

V = bật: heavy stroke **vàng**, thin stroke **cam**. E = tắt: solid **xanh trời nhạt**. Transparent background.

## Two lightness variants of the same hues

| Variant | Role | Hex | #F3F3F3 (taskbar sáng) | #F7F7F5 (cửa sổ sáng) | #202020 (taskbar tối) | #272727 (cửa sổ tối) | #808080 (xám giữa (tham khảo)) |
|---|---|---|---|---|---|---|---|
| light | heavy | `#AD8400` | 3.11 | 3.22 | 4.72 | 4.32 | 1.14 |
| light | thin | `#CF5605` | 3.80 | 3.93 | 3.87 | 3.55 | 1.07 |
| light | e | `#1192D3` | 3.11 | 3.22 | 4.71 | 4.32 | 1.14 |
| dark | heavy | `#FACC15` | 1.38 | 1.43 | 10.64 | 9.75 | 2.58 |
| dark | thin | `#FB923C` | 2.04 | 2.11 | 7.20 | 6.60 | 1.75 |
| dark | e | `#7DD3FC` | 1.50 | 1.55 | 9.77 | 8.96 | 2.37 |

- **Dark-taskbar / dark-window variant**: the owner's own bright colours (`#FACC15` vàng, `#FB923C` cam, `#7DD3FC` xanh trời); all ≥ 6.6 : 1 on `#202020` and ≥ 6.4 : 1 on `#272727`.
- **Light-taskbar / light-window variant — a compromise.** None of the three bright colours can be used on a light background: `#FACC15` is 1.38 : 1 on `#F3F3F3`, `#7DD3FC` 1.50 : 1, `#FB923C` 2.04 : 1. To reach 3 : 1 they were darkened along the same hue to `#AD8400` (vàng → **dark gold / mustard**, no longer reads as "yellow"), `#CF5605` (cam → burnt orange, still orange) and `#1192D3` (xanh trời nhạt → **medium sky blue**, no longer "light"). The light variant keeps the hue relationships (yellow-orange vs blue) but loses the brightness the owner chose; it is the cost of being visible on `#F3F3F3`.
- On mid-grey `#808080` no colour of either set reaches 3 : 1 (1.1–2.6) — a transparent mark cannot be legible on every mid-tone; the two-variant rule assumes a light or a dark taskbar.

## Two-size rule numbers
- **Master (512 units, ≥ 64 px)**: heavy 113 → 95 units perpendicular, thin ~71 units, all terminals cut at the same ~11° rise with 8–14-unit radii; join = thin right edge ends exactly at the heavy foot corner, thin left edge ends on the heavy right edge, thin drawn underneath (no cap, no gap). E: 104-unit stem, 72-unit bars, equal 100-unit counters, one path. Both letters optically centred in the 512 box.
- **16 px optical size (redrawn)**: glyph 13 px tall on whole pixels (y 1–14). Heavy 3.55 → 3.2 px, thin 2.4 px, flat tops and square corners (terminal cut and radii removed), interior 2.0 px open at y = 8. E: 3 px stem and bars (thicker than the master ratio) with two 2 px counters, every edge on a whole pixel. 20 / 32 px are rendered from the 16 drawing.
