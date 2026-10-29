# VietKB keycap icons v4.
#
# Starts from the approved letter art in design/logo11/keycap/*-master.svg and
# changes ONLY the block: the 1 px dark outline is removed and the side wall is
# darkened, so the shape reads as one solid keycap instead of two stacked
# plates. The letter paths are untouched.

$ErrorActionPreference = 'Stop'
$out = $PSScriptRoot
$src = Join-Path (Split-Path -Parent $PSScriptRoot) 'logo11\keycap'
$edge = 'C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe'
$sizes = 16, 20, 24, 32, 48, 64, 128, 256

$variants = @(
  @{ name = 'v'; master = 'v-master.svg'; border = '#C94810'; wall = '#E8BFA6'; wallDeep = '#B4693C'; face = '#FFEBDD'; faceLight = '#EFD2B8' },
  @{ name = 'e'; master = 'e-master.svg'; border = '#1E72D2'; wall = '#A8C9E2'; wallDeep = '#5B84AE'; face = '#E0F1FD'; faceLight = '#C3DCF2' }
)

function Render([string]$svgPath, [int]$size, [string]$pngPath) {
  if (Test-Path $pngPath) { Remove-Item $pngPath -Force }
  $edgeArgs = @(
    '--headless', '--disable-gpu', '--hide-scrollbars', '--no-first-run',
    '--default-background-color=00000000',
    "--window-size=$size,$size",
    "--screenshot=$pngPath",
    "file:///$($svgPath -replace '\\','/')"
  )
  Start-Process -FilePath $edge -ArgumentList $edgeArgs -Wait -WindowStyle Hidden | Out-Null
  if (-not (Test-Path $pngPath)) { throw "no PNG: $pngPath" }
}

foreach ($v in $variants) {
  $svg = Get-Content (Join-Path $src $v.master) -Raw
  # 1. drop the outline path (the owner's "dark border around the whole logo")
  $svg = $svg -replace "<path d=""M130\.0 16\.0[^>]*fill=""$([regex]::Escape($v.border))""/>", ''
  # 2. keep the master's block; give the wall a downward slope instead of a flat
  #    band: a diagonal gradient from the face edge (light) to the outer edge
  #    (deep) reads as a bevelled side.
  $grad = "<linearGradient id=""wgrad"" x1=""0"" y1=""0"" x2=""0.35"" y2=""1""><stop offset=""0"" stop-color=""$($v.faceLight)""/><stop offset=""1"" stop-color=""$($v.wallDeep)""/></linearGradient>"
  $svg = $svg.Replace('</defs>', "$grad</defs>")
  $svg = $svg.Replace("fill=""$($v.wall)""", 'fill="url(#wgrad)"')
  $svg = $svg.Replace('stroke-opacity=".08"', 'stroke-opacity=".14"')
  Set-Content (Join-Path $out "$($v.name).svg") -Value $svg -Encoding UTF8

  foreach ($size in $sizes) {
    $sized = $svg.Replace('viewBox="0 0 512 512" width="512" height="512"', "viewBox=""0 0 512 512"" width=""$size"" height=""$size""")
    $sizedPath = Join-Path $out "$($v.name)-$size.svg"
    Set-Content $sizedPath -Value $sized -Encoding UTF8
    Render $sizedPath $size (Join-Path $out "$($v.name)-$size.png")
    Remove-Item $sizedPath -Force
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






