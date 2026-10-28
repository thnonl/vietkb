$ErrorActionPreference = 'Continue'
$root = Split-Path -Parent $PSScriptRoot
$out = Join-Path $PSScriptRoot 'shots'
New-Item -ItemType Directory -Force $out | Out-Null
$edge = 'C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe'
$sizes = @{ quick = '480,384'; hotkeys = '520,460'; advanced = '560,496'; tray = '440,360' }
$names = if ($args.Count) { $args } else { 'quick','hotkeys','advanced','tray' }
foreach ($t in 'light','dark') {
  foreach ($n in $names) {
    $f = Join-Path $root "$t\$n.html"
    $png = Join-Path $out "$t-$n.png"
    $url = 'file:///' + ($f -replace '\\','/')
    & $edge --headless=new --disable-gpu --force-device-scale-factor=1 "--window-size=$($sizes[$n])" "--screenshot=$png" $url 2>$null | Out-Null
  }
}
Get-ChildItem $out | Select-Object Name, Length
