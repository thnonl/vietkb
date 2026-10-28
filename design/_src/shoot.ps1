$ErrorActionPreference = 'Continue'
$root = Split-Path -Parent $PSScriptRoot
$out = Join-Path $PSScriptRoot 'shots'
New-Item -ItemType Directory -Force $out | Out-Null
$edge = 'C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe'
$names = if ($args.Count) { $args } else { 'go','macro','hotkey','system' }
foreach ($t in 'light','dark') {
  foreach ($n in $names) {
    $f = Join-Path $root "$t\$n.html"
    $png = Join-Path $out "$t-$n.png"
    $url = 'file:///' + ($f -replace '\\','/')
    & $edge --headless=new --disable-gpu --force-device-scale-factor=1 --window-size=720,560 "--screenshot=$png" $url 2>$null | Out-Null
  }
}
Get-ChildItem $out | Select-Object Name, Length
