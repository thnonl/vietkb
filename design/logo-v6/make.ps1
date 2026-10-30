# VietKB keycap icons (v6) — modelled on a real keycap photo:
#   * the lettered top face is a rounded square;
#   * the base is wider and lower than the face;
#   * the front and right walls are trapezoids that flare out from the face edge
#     to the base edge, so the sides lean away instead of standing level;
#   * one flat colour per wall (no diagonal wedge), soft shadow underneath.
# Letters are the approved paths from design/logo11, unchanged.

$ErrorActionPreference = 'Stop'
$out = $PSScriptRoot
$src = Join-Path (Split-Path -Parent $PSScriptRoot) 'logo11\keycap'
$edge = 'C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe'
$sizes = 16, 20, 24, 32, 48, 64, 128, 256

$variants = @(
  @{ name = 'v'; master = 'v-master.svg'
     face = '#FFEBDD'; left = '#E0B08A'; front = '#C08055'; right = '#A25F33' },
  @{ name = 'e'; master = 'e-master.svg'
     face = '#E0F1FD'; left = '#A9CBEA'; front = '#7FA9D9'; right = '#547FA9' }
)

# Geometry in the 512 viewBox.
# Left band 12 px, right band 48 px, bottom band 56 px: light comes from the
# left, so the right side is the thickest and the darkest.
$fx = 36; $fy = 24; $fw = 404; $fh = 408; $fr = 48      # top face
$bx = 24; $by = 24; $bw = 464; $bh = 464; $br = 90      # base

Add-Type -AssemblyName System.Drawing

function Body([hashtable]$v, [string]$letters) {
  $fx2 = $fx + $fw; $fy2 = $fy + $fh
  $bx2 = $bx + $bw; $by2 = $by + $bh
  @"
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" width="512" height="512">
  <defs>
    <clipPath id="base"><rect x="$bx" y="$by" width="$bw" height="$bh" rx="$br"/></clipPath>
    <filter id="sh" x="-30%" y="-70%" width="160%" height="250%"><feGaussianBlur stdDeviation="14"/></filter>
  </defs>
  <ellipse cx="256" cy="500" rx="196" ry="11" fill="#1C1C1A" opacity="0.20" filter="url(#sh)"/>
  <g clip-path="url(#base)">
    <rect x="$bx" y="$by" width="$bw" height="$bh" fill="$($v.front)"/>
    <polygon points="$bx,$by $fx,$fy $fx,$fy2 $bx,$by2" fill="$($v.left)"/>
    <polygon points="$fx2,$fy $bx2,$by $bx2,$by2 $fx2,$fy2" fill="$($v.right)"/>
    <polygon points="$fx,$fy2 $fx2,$fy2 $bx2,$by2 $bx,$by2" fill="$($v.front)"/>
    <path d="M$($fx + $fr) $fy H$($fx2 - $fr) A$fr $fr 0 0 1 $fx2 $($fy + $fr) V$fy2 H$fx V$($fy + $fr) A$fr $fr 0 0 1 $($fx + $fr) $fy Z" fill="$($v.face)"/>
  </g>
  $letters
</svg>
"@
}

function Render([string]$svgPath, [int]$size, [string]$pngPath) {
  $big = Join-Path $env:TEMP "vkb-logo6-$size-big.png"
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
  $srcImg = [System.Drawing.Image]::FromFile($big)
  $bmp = New-Object System.Drawing.Bitmap $size, $size
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.InterpolationMode = 'HighQualityBicubic'
  $g.PixelOffsetMode = 'HighQuality'
  $g.Clear([System.Drawing.Color]::Transparent)
  $w = [int]($size * 1.12)
  $off = [int](($size - $w) / 2)
  $g.DrawImage($srcImg, $off, $off, $w, $w)
  $g.Dispose(); $srcImg.Dispose()
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
  if ($v.name -eq 'v') {
    $letters = $letters.Replace('fill="#E4507C"', 'fill="#F06D95"')
    $letters = $letters.Replace('fill="#A84A00"', 'fill="#D97A1C"')
  }
  $svg = Body $v $letters
  $svgPath = Join-Path $out "$($v.name).svg"
  Set-Content $svgPath -Value $svg -Encoding UTF8
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


