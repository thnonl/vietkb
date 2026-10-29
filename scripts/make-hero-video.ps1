# Renders design/hero-motion/scene.html to site/assets/hero-motion.mp4 (+ poster).
# Requires Edge and ffmpeg on PATH (or in the WinGet package folder).
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$scene = Join-Path $root 'design\hero-motion\scene.html'
$frames = Join-Path $root 'design\hero-motion\_frames'
$edge = 'C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe'
$fps = 25
$n = 150
Remove-Item $frames -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force $frames | Out-Null
$prof = Join-Path $env:TEMP ('edge-hero-' + [guid]::NewGuid().ToString('N').Substring(0,6))
for ($i = 0; $i -lt $n; $i++) {
  $t = [math]::Round($i / $fps, 3)
  $out = Join-Path $frames ('f{0:d3}.png' -f $i)
  $args = @('--headless','--disable-gpu','--hide-scrollbars',"--user-data-dir=$prof",'--window-size=1280,720','--virtual-time-budget=300',"--screenshot=$out","file:///$($scene -replace '\\','/')?t=$t")
  Start-Process $edge -ArgumentList $args -Wait -WindowStyle Hidden
}
$mp4 = Join-Path $root 'site\assets\hero-motion.mp4'
& ffmpeg -y -framerate $fps -i (Join-Path $frames 'f%03d.png') -c:v libx264 -pix_fmt yuv420p -crf 23 -preset slow -movflags +faststart $mp4
& ffmpeg -y -i (Join-Path $frames 'f090.png') -q:v 3 (Join-Path $root 'site\assets\hero-poster.jpg')
Write-Host "mp4: $mp4"
