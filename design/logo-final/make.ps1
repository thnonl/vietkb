# VietKB keycap icons — final.
#
# The approved artwork from design/logo11/keycap/*-master.svg with exactly one
# change: the 1 px outline path is removed (the owner's diagnosis was that the
# dark outline wrapping the whole logo is what flattened it). Face, side wall,
# light wedge, shadow and letter paths are untouched.

$ErrorActionPreference = 'Stop'
$out = $PSScriptRoot
$src = Join-Path (Split-Path -Parent $PSScriptRoot) 'logo11\keycap'
$edge = 'C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe'
$sizes = 16, 20, 24, 32, 48, 64, 128, 256

$variants = @(
  @{ name = 'v'; master = 'v-master.svg'; border = '#C94810' },
  @{ name = 'e'; master = 'e-master.svg'; border = '#1E72D2' }
)

Add-Type -AssemblyName System.Drawing

function Render([string]$svgPath, [int]$size, [string]$pngPath) {
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
  $srcImg = [System.Drawing.Image]::FromFile($big)
  $bmp = New-Object System.Drawing.Bitmap $size, $size
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.InterpolationMode = 'HighQualityBicubic'
  $g.PixelOffsetMode = 'HighQuality'
  $g.Clear([System.Drawing.Color]::Transparent)
  $g.DrawImage($srcImg, 0, 0, $size, $size)
  $g.Dispose(); $srcImg.Dispose()
  if (Test-Path $pngPath) { Remove-Item $pngPath -Force }
  $bmp.Save($pngPath, [System.Drawing.Imaging.ImageFormat]::Png)
  $bmp.Dispose()
  Remove-Item $big -Force
}

foreach ($v in $variants) {
  $svg = Get-Content (Join-Path $src $v.master) -Raw
  $svg = $svg -replace "<path d=""M130\.0 16\.0[^>]*fill=""$([regex]::Escape($v.border))""/>", ''
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
