# Renders design/hero-motion/scene.html frame by frame with headless Edge, then encodes with ffmpeg.
# Defaults (no parameters): 1280x720 -> site/assets/hero-motion.mp4 + site/assets/hero-poster.jpg (poster = frame 165).
# Store trailer: -Scale 1.5 -Out <mp4> -Poster <png> -Crf 18 gives 1920x1080 from the same 1280x720 layout.
# Requires Edge and ffmpeg on PATH (or in the WinGet package folder).
param(
  [double]$Scale = 1.0,        # 1.0 = 1280x720 (default), 1.5 = 1920x1080 (Edge device scale factor)
  [string]$Out = '',           # mp4 output; default site\assets\hero-motion.mp4
  [string]$Poster = '',        # poster output (.jpg or .png); default site\assets\hero-poster.jpg
  [int]$PosterFrame = 165,     # frame used for the poster (165 = 6.6 s)
  [int]$Crf = 23,              # x264 quality (23 default, 18 for crisp text)
  [switch]$SilentAudio,        # add a silent stereo AAC-LC track (48 kHz, 384 kbps)
  [switch]$Trailer             # Store trailer x264 settings: High profile, GOP 12, 2 B-frames, closed GOP
)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$scene = Join-Path $root 'design\hero-motion\scene.html'
$frames = Join-Path $root 'design\hero-motion\_frames'
$assets = Join-Path $root 'design\hero-motion\assets'
$edge = 'C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe'
$fps = 25
$duration = 30.72
$n = $fps * $duration
$inv = [Globalization.CultureInfo]::InvariantCulture

$mp4 = if ($Out) { [IO.Path]::GetFullPath($Out) } else { Join-Path $root 'site\assets\hero-motion.mp4' }
$posterPath = if ($Poster) { [IO.Path]::GetFullPath($Poster) } else { Join-Path $root 'site\assets\hero-poster.jpg' }
New-Item -ItemType Directory -Force (Split-Path -Parent $mp4) | Out-Null
New-Item -ItemType Directory -Force (Split-Path -Parent $posterPath) | Out-Null

New-Item -ItemType Directory -Force $assets | Out-Null
Copy-Item (Join-Path $root 'site\assets\logo-v.png') (Join-Path $assets 'logo-v.png') -Force
foreach ($f in 'settings-vi-dark.png', 'gotat-vi-light.png') {
  Copy-Item (Join-Path $root "site\assets\$f") (Join-Path $assets $f) -Force
}

Remove-Item $frames -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force $frames | Out-Null
$prof = Join-Path $env:TEMP ('edge-hero-' + [guid]::NewGuid().ToString('N').Substring(0,6))
$url = 'file:///' + ($scene -replace '\\', '/')
for ($i = 0; $i -lt $n; $i++) {
  $t = [math]::Round($i / $fps, 3)
  $out = Join-Path $frames ('f{0:d3}.png' -f $i)
  $edgeArgs = @('--headless', '--disable-gpu', '--hide-scrollbars', "--user-data-dir=$prof",
    '--window-size=1280,720', '--virtual-time-budget=300', "--screenshot=$out", "${url}?t=$t")
  if ($Scale -ne 1) { $edgeArgs += '--force-device-scale-factor=' + $Scale.ToString($inv) }
  Start-Process $edge -ArgumentList $edgeArgs -Wait -WindowStyle Hidden
}

$x264 = @()
if ($Trailer) { $x264 = @('-profile:v', 'high', '-g', '12', '-bf', '2', '-flags', '+cgop') }
if ($SilentAudio) {
  & ffmpeg -y -framerate $fps -i (Join-Path $frames 'f%03d.png') -f lavfi -i 'anullsrc=channel_layout=stereo:sample_rate=48000' `
    -map 0:v -map 1:a -c:v libx264 -pix_fmt yuv420p -crf $Crf -preset slow @x264 `
    -c:a aac -profile:a aac_low -b:a 384k -ar 48000 -ac 2 -shortest -movflags +faststart $mp4
} else {
  & ffmpeg -y -framerate $fps -i (Join-Path $frames 'f%03d.png') -c:v libx264 -pix_fmt yuv420p `
    -crf $Crf -preset slow @x264 -movflags +faststart $mp4
}

$posterFile = Join-Path $frames ('f{0:d3}.png' -f $PosterFrame)
if ($posterPath.EndsWith('.png')) {
  & ffmpeg -y -i $posterFile -update 1 $posterPath
} else {
  & ffmpeg -y -i $posterFile -update 1 -q:v 3 $posterPath
}
Write-Host "mp4: $mp4"
Write-Host "poster: $posterPath"
