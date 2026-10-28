# d4 — Đất/Trời · master + optical size

Colours — light taskbar: heavy `#5D301F`, thin/E `#3B6CDF`; dark taskbar: heavy `#DBA794`, thin/E `#6F93E8`.

**Master (512 units, used ≥ 64 px).**
- Heavy downstroke: 113 → 95 units perpendicular (taper), top cut rising to the right at ~11°, 14-unit corner radii, 10-unit radius on the foot's outer corner.
- Thin upstroke: ~71 units perpendicular (heavy : thin ≈ 1.45 : 1), same top cut and radii.
- Join: the thin stroke's right edge ends exactly at the heavy stroke's bottom-right corner (318.4, 448), and its left edge ends on the heavy's right edge at (286.6, 352.6). The thin shape sits under the heavy one with a 12-unit hidden overlap, so there is no gap, no seam and no cap below the point.
- E: 104-unit stem, three 72-unit bars (= thin stroke), two equal 100-unit counters, bar ends cut at the V's terminal angle; outer corners 14, inner 8. One single path, so the thickness is exactly even.

**16 px optical size (redrawn).**
- Thickened: heavy 3.7 → 3.1 px (master scaled would be 3.5 → 3.0), thin 2.5 px (scaled 2.2), E stem 3.3 px, bars 2.5 px.
- Removed: the slanted terminal cut and all corner radii (flat tops, square corners), because they need antialiasing to read.
- Adjusted join: the thin stroke's left edge meets the heavy stroke at y ≈ 10.9 instead of the scaled ≈ 11, keeping the interior ≥ 2 px open at mid-height.
- 20 px and 32 px are rendered from the 16 px drawing; a separate 20 grid was not needed, because the 16 drawing has no half-pixel features that misalign at 1.25×.

Stroke widths by size: 512 → 113/71 units · 256 → 56/35 px · 128 → 28/18 px · 64 → 14/9 px (master) · 32 → 7.4/5 px · 20 → 4.6/3.1 px · 16 → 3.7/2.5 px (optical size).
