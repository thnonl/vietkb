# VietKB keycap icons (v5).
#
# The block is a frustum, not a slab: the lettered face is smaller than the base
# and four bevelled facets slope out from the face edge to the base edge, so the
# sides lean away instead of sitting level with the face. Letters are the
# approved paths from design/logo11/keycap/*-master.svg, unchanged; only the
# wrapper around them is rebuilt (the old 1 px outline is gone).

$ErrorActionPreference = 'Stop'
$out = $PSScriptRoot
$src = Join-Path (Split-Path -Parent $PSScriptRoot) 'logo11\keycap'
$edge = 'C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe'
$sizes = 16, 20, 24, 32, 48, 64, 128, 256

$variants = @(
  @{ name = 'v'; master = 'v-master.svg'
     face = '#FFEBDD'; top = '#FFF7F0'; left = '#F8E2CE'; right = '#E3B896'; base = '#B4693C'; letter = '#E4507C' },
  @{ name = 'e'; master = 'e-master.svg'
     face = '#E0F1FD'; top = '#F3FAFF'; left = '#D2E8F9'; right = '#A8C9E2'; base = '#5B84AE'; letter = '#1C6CC6' }
)

$inset = 34          # how far the face is pulled in from the base edge
$radiusBase = 112
$radiusFace = 82

function Block([hashtable]$v) {
  $i = $script:inset
  $s = 512
  $f = $s - 2 * $i
  @"
  <clipPath id="base"><rect x="0" y="0" width="$s" height="$s" rx="$radiusBase"/></clipPath>
  <filter id="sh" x="-25%" y="-60%" width="150%" height="230%"><feGaussianBlur stdDeviation="16"/></filter>
"@
}

function Body([hashtable]$v, [string]$letters) {
  $i = $script:inset
  $s = 512
  $f = $s - 2 * $i
  @"
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" width="512" height="512">
  <defs>
    <clipPath id="base"><rect x="0" y="0" width="$s" height="$s" rx="$radiusBase"/></clipPath>
    <filter id="sh" x="-25%" y="-60%" width="150%" height="230%"><feGaussianBlur stdDeviation="16"/></filter>
  </defs>
  <ellipse cx="256" cy="496" rx="190" ry="12" fill="#1C1C1A" opacity="0.20" filter="url(#sh)"/>
  <g clip-path="url(#base)">
    <rect x="0" y="0" width="$s" height="$s" fill="$($v.base)"/>
    <polygon points="0,0 $s,0 $($s - $i),$i $i,$i" fill="$($v.top)"/>
    <polygon points="0,0 $i,$i $i,$($s - $i) 0,$s" fill="$($v.left)"/>
    <polygon points="$s,0 $($s - $i),$i $($s - $i),$($s - $i) $s,$s" fill="$($v.right)"/>
    <polygon points="0,$s $i,$($s - $i) $($s - $i),$($s - $i) $s,$s" fill="$($v.base)"/>
    <rect x="$i" y="$i" width="$f" height="$f" rx="$radiusFace" ry="$radiusFace" fill="$($v.face)"/>
  </g>
  $letters
</svg>
"@
}

Add-Type -AssemblyName System.Drawing

function Render([string]$svgPath, [int]$size, [string]$pngPath) {
  # Always rasterise at 512 (the SVG's own viewBox), then downsample: relying on
  # Edge to honour a rewritten width/height crops the drawing instead of scaling.
  $big = Join-Path $env:TEMP "vkb-logo-$size-big.png"
  if (Test-Path $big) { Remove-Item $big -Force }
  $edgeArgs = @(
    '--headless', '--disable-gpu', '--hide-scrollbars', '--no-first-run',
    '--default-background-color=00000000',
    '--window-size=512,512',
    "--screenshot=$big",
    "file:///$($svgPath -replace '\\','/')"
  )
  Start-Process -FilePath $edge -ArgumentList $edgeArgs -Wait -WindowStyle Hidden | Out-Null
  if (-not (Test-Path $big)) { throw "no PNG: $big" }
  $src = [System.Drawing.Image]::FromFile($big)
  $bmp = New-Object System.Drawing.Bitmap $size, $size
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.InterpolationMode = 'HighQualityBicubic'
  $g.PixelOffsetMode = 'HighQuality'
  $g.Clear([System.Drawing.Color]::Transparent)
  $g.DrawImage($src, 0, 0, $size, $size)
  $g.Dispose(); $src.Dispose()
  if (Test-Path $pngPath) { Remove-Item $pngPath -Force }
  $bmp.Save($pngPath, [System.Drawing.Imaging.ImageFormat]::Png)
  $bmp.Dispose()
  Remove-Item $big -Force
}

foreach ($v in $variants) {
  $master = Get-Content (Join-Path $src $v.master) -Raw
  $m = [regex]::Match($master, '<g transform="translate\(114\.0 104\.0\)[\s\S]*?</g></g>')
  if (-not $m.Success) { throw "letters not found in $($v.master)" }
  $letters = $m.Value

  $svg = Body $v $letters
  Set-Content (Join-Path $out "$($v.name).svg") -Value $svg -Encoding UTF8

  $svgPath = Join-Path $out "$($v.name).svg"
  foreach ($size in $sizes) {
    Render $svgPath $size (Join-Path $out "$($v.name)-$size.png")
  }
}

# --- icon.ico ---------------------------------------------------------------
$frames = 16, 24, 32, 48, 64, 128, 256
$ms = New-Object System.IO.MemoryStream
$bw = New-Object System.IO.BinaryWriter($ms)
$bw.Write([uint16]0); $bw.Write([uint16]1); $bw.Write([uint16]$frames.Count)
$offset = 6 + 16 * $frames.Count
$payloads = @()
foreach ($s in $frames) {
  $bytes = [System.IO.File]::ReadAllBytes((Join-Path $out "v-$s.png"))
  $payloads += ,$bytes
  $dim = if ($s -ge 256) { 0 } else { $s }
  $bw.Write([byte]$dim); $bw.Write([byte]$dim)
  $bw.Write([byte]0); $bw.Write([byte]0)
  $bw.Write([uint16]1); $bw.Write([uint16]32)
  $bw.Write([uint32]$bytes.Length); $bw.Write([uint32]$offset)
  $offset += $bytes.Length
}
foreach ($p in $payloads) { $bw.Write($p) }
$bw.Flush()
[System.IO.File]::WriteAllBytes((Join-Path $out 'icon.ico'), $ms.ToArray())
$bw.Dispose(); $ms.Dispose()

# --- contact sheet ----------------------------------------------------------
Add-Type -AssemblyName System.Drawing
$sheet = New-Object System.Drawing.Bitmap 1180, 700
$g = [System.Drawing.Graphics]::FromImage($sheet)
$g.Clear([System.Drawing.Color]::FromArgb(238, 238, 240))
$font = New-Object System.Drawing.Font('Segoe UI', 9)
$x = 20
foreach ($s in 16, 24, 32, 48, 128, 256) {
  foreach ($name in 'v', 'e') {
    $img = [System.Drawing.Image]::FromFile((Join-Path $out "$name-$s.png"))
    $row = if ($name -eq 'v') { 20 } else { 370 }
    $g.DrawImage($img, $x, $row, $s, $s)
    $g.DrawString("$name $s", $font, [System.Drawing.Brushes]::Black, $x, ($row + $s + 6))
    $img.Dispose()
  }
  $x += [math]::Max($s, 60) + 24
}
$sheet.Save((Join-Path $out 'compare.png'), [System.Drawing.Imaging.ImageFormat]::Png)
$g.Dispose(); $sheet.Dispose()

Get-ChildItem $out | Sort-Object Name | Select-Object Name, Length | Format-Table -AutoSize

