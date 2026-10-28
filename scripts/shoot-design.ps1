# Screenshot the design mocks at true window size
# Usage:  powershell -ExecutionPolicy Bypass -File scripts\shoot-design.ps1
param(
    [string]$Source = "E:\vietkb\design",
    [string]$OutDir = "E:\vietkb\design\_shots",
    [int]$Width = 1040,
    [int]$Height = 720
)

$edge = "C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe"
if (-not (Test-Path $edge)) { throw "Edge not found at $edge" }

New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

$files = Get-ChildItem -Path $Source -Recurse -Filter *.html |
    Where-Object { $_.FullName -notlike "*\_shots\*" }

foreach ($file in $files) {
    $rel = $file.FullName.Substring($Source.TrimEnd('\').Length).TrimStart('\')
    $name = ($rel -replace '[\\/]', '_') -replace '\.html$', ''
    $png = Join-Path $OutDir "$name.png"
    $uri = "file:///" + ($file.FullName -replace '\\', '/')

    & $edge --headless=new --disable-gpu --hide-scrollbars --no-first-run `
        --force-device-scale-factor=1 --window-size="$Width,$Height" `
        --screenshot="$png" $uri 2>$null | Out-Null

    if (Test-Path $png) {
        $kb = [math]::Round((Get-Item $png).Length / 1KB)
        "{0,-44} {1,6} KB" -f $name, $kb
    } else {
        "{0,-44} FAILED" -f $name
    }
}

"shots in: $OutDir"
