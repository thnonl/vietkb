# VietKB keycap icons — old approved art + three owner tweaks:
#   1. the dark outline around the whole icon is removed;
#   2. the lettered face gets its own thin outline;
#   3. both letters get the same heavy stroke weight used before, scaled so the
#      weight survives at 16 px.
# Everything else (wall, wedge, face size, colours) stays as in logo11.

$ErrorActionPreference = 'Stop'
$out = $PSScriptRoot
$src = Join-Path (Split-Path -Parent $PSScriptRoot) 'logo11\keycap'
$edge = 'C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe'
$sizes = 16, 20, 24, 32, 48, 64, 128, 256

$variants = @(
  @{ name = 'v'; master = 'v-master.svg'; border = '#C94810'; face = '#FFEBDD'; rim = '#FFF9F4'
     inks = @('#E4507C', '#E9861B'); recolor = @{ '#A84A00' = '#E9861B' } },
  @{ name = 'e'; master = 'e-master.svg'; border = '#1E72D2'; face = '#E0F1FD'; rim = '#F4FBFF'
     inks = @('#1C6CC6') }
)

function StrokeFor([int]$size) {
  switch ($size) {
    { $_ -le 20 } { 150 }
    { $_ -le 24 } { 130 }
    { $_ -le 32 } { 100 }
    { $_ -le 48 } { 74 }
    { $_ -le 64 } { 56 }
    default { 42 }
  }
}

Add-Type -AssemblyName System.Drawing

function Render([string]$svgPath, [int]$size, [string]$pngPath) {
  $big = Join-Path $env:TEMP "vkb-icon-old-$size.png"
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
  $w = [int]($size * 1.04)
  $off = [int](($size - $w) / 2)
  $g.DrawImage($srcImg, $off, $off, $w, $w)
  $g.Dispose(); $srcImg.Dispose()
  if (Test-Path $pngPath) { Remove-Item $pngPath -Force }
  $bmp.Save($pngPath, [System.Drawing.Imaging.ImageFormat]::Png)
  $bmp.Dispose()
  Remove-Item $big -Force
}

foreach ($v in $variants) {
  $svg = Get-Content (Join-Path $src $v.master) -Raw

  # 1. outer outline out
  $svg = $svg -replace "<path d=""M130\.0 16\.0[^>]*fill=""$([regex]::Escape($v.border))""/>", ''

  # 2. outline on the lettered face only
  $faceBorder = "<rect x=""34"" y=""24"" width=""416"" height=""416"" rx=""96"" fill=""none"" stroke=""$($v.rim)"" stroke-width=""7""/>"
  $svg = $svg.Replace('<line x1="130" y1="25.25" x2="354" y2="25.25"', "$faceBorder<line x1=""130"" y1=""25.25"" x2=""354"" y2=""25.25""")

  if ($v.recolor) { foreach ($k in $v.recolor.Keys) { $svg = $svg.Replace("fill=""$k""", "fill=""$($v.recolor[$k])""") } }
  # 3. heavier letter strokes
  foreach ($ink in $v.inks) {
    $svg = $svg.Replace(
      "fill=""$ink""",
      "fill=""$ink"" stroke=""$ink"" stroke-width=""42"" stroke-linejoin=""round"""
    )
  }

  foreach ($size in $sizes) {
    $stroke = StrokeFor $size
    $sized = $svg.Replace('stroke-width="42"', "stroke-width=""$stroke""")
    $sizedPath = Join-Path $out "$($v.name)-$size.svg"
    Set-Content $sizedPath -Value $sized -Encoding UTF8
    Render $sizedPath $size (Join-Path $out "$($v.name)-$size.png")
    Remove-Item $sizedPath -Force
  }
  Set-Content (Join-Path $out "$($v.name).svg") -Value $svg -Encoding UTF8
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


