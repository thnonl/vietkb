# Generates the VietKB keycap icon set: vector SVG per variant, PNGs at every
# size Windows asks for, a multi-size icon.ico, and a contact sheet.
#
# Deterministic: no external assets, no manual steps. Rasterises with headless
# Edge, which is installed on every Windows machine that has WebView2.

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$out = $PSScriptRoot
$edge = 'C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe'
New-Item -ItemType Directory -Force $out | Out-Null

$sizes = 16, 20, 24, 32, 48, 64, 128, 256

$variants = @(
  @{ name = 'v'; letter = 'V'; face = '#FBE3EA'; wall = '#8E2F4D'; ink = '#E4507C'; ink2 = '#A84A00' },
  @{ name = 'e'; letter = 'E'; face = '#DCE9F7'; wall = '#123F74'; ink = '#1C6CC6'; ink2 = $null }
)

function Svg([int]$size, [hashtable]$v) {
  $pad = [math]::Max(1, [math]::Round($size * 0.055))
  $inner = $size - 2 * $pad
  $wall = [math]::Round($inner * 0.135)
  $face = $inner - $wall
  $radius = [math]::Round($inner * 0.13)
  $shadow = [math]::Round($size * 0.045)
  $glyph = if ($size -le 32) { [math]::Round($face * 0.74) } else { [math]::Round($face * 0.62) }
  $highlight = [math]::Round($face * 0.18)

  # Two-tone V: the lower half of the strokes uses the darker orange.
  $inkPaint = if ($v.ink2) {
    @"
    <linearGradient id="ink" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0.5" stop-color="$($v.ink)"/>
      <stop offset="0.5" stop-color="$($v.ink2)"/>
    </linearGradient>
"@
  } else {
    "<solidColor id=""ink"" color=""$($v.ink)""/>"
  }
  $inkRef = if ($v.ink2) { 'url(#ink)' } else { $v.ink }

@"
<svg xmlns="http://www.w3.org/2000/svg" width="$size" height="$size" viewBox="0 0 $size $size">
  <defs>
    $inkPaint
    <linearGradient id="gloss" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#ffffff" stop-opacity="0.22"/>
      <stop offset="1" stop-color="#ffffff" stop-opacity="0"/>
    </linearGradient>
    <filter id="drop" x="-30%" y="-30%" width="160%" height="170%">
      <feGaussianBlur in="SourceAlpha" stdDeviation="$([math]::Max(1,$shadow / 2))"/>
      <feOffset dx="0" dy="$shadow"/>
      <feComponentTransfer><feFuncA type="linear" slope="0.25"/></feComponentTransfer>
      <feMerge><feMergeNode/><feMergeNode in="SourceGraphic"/></feMerge>
    </filter>
  </defs>
  <g filter="url(#drop)">
    <!-- side wall: the silhouette drawn one wall-thickness down and right -->
    <rect x="$($pad + $wall)" y="$($pad + $wall)" width="$face" height="$face" rx="$radius" ry="$radius" fill="$($v.wall)"/>
    <!-- face plate -->
    <rect x="$pad" y="$pad" width="$face" height="$face" rx="$radius" ry="$radius" fill="$($v.face)"/>
    <!-- convex highlight across the top -->
    <rect x="$pad" y="$pad" width="$face" height="$highlight" rx="$radius" ry="$radius" fill="url(#gloss)"/>
    <text x="$($pad + $face / 2)" y="$($pad + $face / 2)" text-anchor="middle" dominant-baseline="central"
          font-family="Segoe UI Semibold, Segoe UI, Arial" font-weight="600"
          font-size="$glyph" fill="$inkRef">$($v.letter)</text>
  </g>
</svg>
"@
}

function Render([string]$svgPath, [int]$size, [string]$pngPath) {
  if (Test-Path $pngPath) { Remove-Item $pngPath -Force }
  $edgeArgs = @(
    '--headless', '--disable-gpu', '--hide-scrollbars', '--no-first-run',
    '--default-background-color=00000000',
    "--window-size=$size,$size",
    "--screenshot=$pngPath",
    "file:///$($svgPath -replace '\\','/')"
  )
  $p = Start-Process -FilePath $edge -ArgumentList $edgeArgs -Wait -PassThru -WindowStyle Hidden
  if (-not (Test-Path $pngPath)) { throw "Edge produced no PNG for $pngPath" }
}

foreach ($v in $variants) {
  $svgPath = Join-Path $out "$($v.name).svg"
  Set-Content -Path $svgPath -Value (Svg 256 $v) -Encoding UTF8
  foreach ($size in $sizes) {
    # The SVG is written per size so the glyph scales like a font hint and the
    # screenshot is exactly <size> pixels.
    Set-Content -Path $svgPath -Value (Svg $size $v) -Encoding UTF8
    Render $svgPath $size (Join-Path $out "$($v.name)-$size.png")
  }
  Set-Content -Path $svgPath -Value (Svg 256 $v) -Encoding UTF8
}

# --- icon.ico: PNG-compressed frames, hand-assembled -------------------------
$icoFrames = 16, 24, 32, 48, 64, 128, 256
$ms = New-Object System.IO.MemoryStream
$bw = New-Object System.IO.BinaryWriter($ms)
$bw.Write([uint16]0)          # reserved
$bw.Write([uint16]1)          # type: icon
$bw.Write([uint16]$icoFrames.Count)
$payloads = @()
$offset = 6 + 16 * $icoFrames.Count
foreach ($s in $icoFrames) {
  $png = Join-Path $out "v-$s.png"
  $bytes = [System.IO.File]::ReadAllBytes($png)
  $payloads += ,$bytes
  $dim = if ($s -ge 256) { 0 } else { $s }
  $bw.Write([byte]$dim); $bw.Write([byte]$dim)
  $bw.Write([byte]0); $bw.Write([byte]0)      # palette, reserved
  $bw.Write([uint16]1); $bw.Write([uint16]32) # planes, bpp
  $bw.Write([uint32]$bytes.Length)
  $bw.Write([uint32]$offset)
  $offset += $bytes.Length
}
foreach ($p in $payloads) { $bw.Write($p) }
$bw.Flush()
[System.IO.File]::WriteAllBytes((Join-Path $out 'icon.ico'), $ms.ToArray())
$bw.Dispose(); $ms.Dispose()

# --- contact sheet ----------------------------------------------------------
Add-Type -AssemblyName System.Drawing
$sheet = New-Object System.Drawing.Bitmap 1180, 620
$g = [System.Drawing.Graphics]::FromImage($sheet)
$g.Clear([System.Drawing.Color]::FromArgb(238, 238, 240))
$g.InterpolationMode = 'NearestNeighbor'
$font = New-Object System.Drawing.Font('Segoe UI', 9)
$brush = [System.Drawing.Brushes]::Black
$x = 20
foreach ($s in 16, 24, 32, 48, 128, 256) {
  foreach ($name in 'v', 'e') {
    $img = [System.Drawing.Image]::FromFile((Join-Path $out "$name-$s.png"))
    $row = if ($name -eq 'v') { 20 } else { 330 }
    $g.DrawImage($img, $x, $row, $s, $s)
    $g.DrawString("$name $s", $font, $brush, $x, ($row + $s + 6))
    $img.Dispose()
  }
  $x += [math]::Max($s, 60) + 24
}
$sheet.Save((Join-Path $out 'compare.png'), [System.Drawing.Imaging.ImageFormat]::Png)
$g.Dispose(); $sheet.Dispose()

Get-ChildItem $out | Sort-Object Name | Select-Object Name, Length | Format-Table -AutoSize



