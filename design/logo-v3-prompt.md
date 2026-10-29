Design the VietKB application icon: a 3D keycap, as crisp vector art. Work ONLY inside `design/logo-v3/`. Do not modify any other directory.

## What it must look like
- The keycap FACE is a perfect square (equal width and height, front-facing, no perspective skew).
- 3D comes from a solid **side wall** on the bottom and right: 12–15% of the icon height, filled with the face colour darkened ~40%. The face plate sits on top, offset up-left, so the wall reads as a thick band.
- **No dark outline/stroke around the silhouette.** That is the main thing the owner rejected. The shape's edge must be the face colour meeting transparent, plus the shadow.
- Top highlight: a band across the top ~18% of the face, white at 18–22% alpha, fading to zero.
- Soft directional drop shadow: offset (0, 6% of height), ~10% blur, black 25%, transparent background.
- Corner radius: 12–14% of the icon size, same radius on the face and the wall.

## Colours
- Letter `V` (Vietnamese on): `#E4507C`, with the lower half of the strokes `#A84A00`.
- Letter `E` (off): `#1C6CC6`.
- Face: V `#FBE3EA`, E `#DCE9F7`.
- Letter-on-face WCAG contrast must be **>= 4.5:1**. Compute the ratio in a script and print it; if it is below 4.5, darken the letter (not the face) and recompute.

## Glyph sizing
- Cap height ≈ 62% of the face at 64 px and above; ≈ 72–78% of the face at 16–32 px (optical bump).
- Stroke width must stay >= 2 px at 16 px.

## Deliverables in `design/logo-v3/`
1. `v.svg` and `e.svg` — the vector source, no raster effects inside.
2. PNGs at **16, 20, 24, 32, 48, 64, 128, 256** for both letters, transparent outside the shadow. Rasterise with headless Edge, rendering each size at exactly that pixel size:
   `"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe" --headless --disable-gpu --hide-scrollbars --default-background-color=00000000 --window-size=<W>,<H> --screenshot=<out.png> file:///E:/vietkb/design/logo-v3/v.svg`
   Adjust the SVG's own width/height per size (or use `--force-device-scale-factor`) so the screenshot is exactly that many pixels and the keycap fills it edge to edge.
3. `icon.ico` containing 16, 24, 32, 48, 64, 128 and 256 px frames (hand-assemble the ICO container if needed; PNG-compressed frames are fine).
4. `compare.png` — contact sheet on a neutral grey: both letters at 16/24/32/48/128/256, with the pixel size labelled under each.
5. `README.md` with the exact commands to regenerate everything.

## Verification you must do before finishing
- Open the generated PNGs (you can view images) and look at 16 px, 48 px and 128 px.
- 48 px: the side wall must be obviously a solid block, not a 1 px edge.
- 16 px: both letters readable, not a solid blob, strokes not dropped.
- Print the computed contrast ratios in your final message.

Report the file list with byte sizes, the contrast ratios, and one line on how 16 px reads.
