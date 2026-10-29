# Builds the production `VietKB` channel.
#
# Produces the shipped app: default identity, no dev overlay, and no extra
# feature on top of `custom-protocol`. The side-by-side dev channel stays
# untouched and is built by `build-dev.ps1`.
#
# The production build uses the default cargo target dir (`target`) and must
# never write to `target-dev`, so it can never overwrite a dev binary and a
# dev build can never masquerade as the production installer.

$ErrorActionPreference = 'Continue'

$root = Split-Path -Parent $PSScriptRoot
$tauri = Join-Path $root 'ui\node_modules\.bin\tauri.cmd'
$targetDir = Join-Path $root 'target'
$bundleDir = Join-Path $targetDir 'release\bundle\nsis'

if (-not (Test-Path $tauri)) {
    Write-Error "tauri CLI not found at $tauri (run: npm --prefix ui install)"
    exit 1
}

# Drop a previous production installer so a failed build can never report a
# stale path.
if (Test-Path $bundleDir) {
    Get-ChildItem $bundleDir -Filter 'VietKB_*setup.exe' | Remove-Item -Force
}

Push-Location (Join-Path $root 'crates\vkb-app')
try {
    & $tauri build --features custom-protocol 2>&1 |
        ForEach-Object { Write-Host $_ }
    $status = $LASTEXITCODE
} finally {
    Pop-Location
}

if ($status -ne 0) {
    Write-Error "tauri build failed with exit code $status"
    exit $status
}

$exe = Join-Path $targetDir 'release\vietkb.exe'
$setup = Get-ChildItem $bundleDir -Filter 'VietKB_*setup.exe' -ErrorAction SilentlyContinue |
    Sort-Object LastWriteTime -Descending |
    Select-Object -First 1

if (-not $setup) {
    Write-Error "installer not found under $bundleDir"
    exit 1
}

Write-Host ''
Write-Host "Prod exe:       $exe"
Write-Host "Prod installer: $($setup.FullName)  ($([math]::Round($setup.Length / 1MB, 2)) MB)"
