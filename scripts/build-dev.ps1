# Builds the side-by-side `VietKB Dev` channel.
#
# Produces the same app as the production build plus the `dev-channel` feature
# (own config dir, own `Run` value) and the dev identity overlay
# (`tauri.dev.conf.json`: productName `VietKB Dev`, identifier
# `vn.vietkb.app.dev`). The production build is untouched.
#
# The dev build gets its OWN cargo target dir (`target-dev`) so it can never
# overwrite `target\release\vietkb.exe` with a dev binary.

$ErrorActionPreference = 'Continue'

$root = Split-Path -Parent $PSScriptRoot
$tauri = Join-Path $root 'ui\node_modules\.bin\tauri.cmd'
$targetDir = Join-Path $root 'target-dev'
$bundleDir = Join-Path $targetDir 'release\bundle\nsis'

if (-not (Test-Path $tauri)) {
    Write-Error "tauri CLI not found at $tauri (run: npm --prefix ui install)"
    exit 1
}

# Drop a previous dev installer so a failed build can never report a stale path.
if (Test-Path $bundleDir) {
    Get-ChildItem $bundleDir -Filter 'VietKB Dev*setup.exe' | Remove-Item -Force
}

# `tauri build` has no `--target-dir`; cargo reads CARGO_TARGET_DIR, and the
# tauri CLI resolves the same directory for the bundle output.
$previousTargetDir = $env:CARGO_TARGET_DIR
$env:CARGO_TARGET_DIR = $targetDir

Push-Location (Join-Path $root 'crates\vkb-app')
try {
    & $tauri build --features custom-protocol,dev-channel --config tauri.dev.conf.json 2>&1 |
        ForEach-Object { Write-Host $_ }
    $status = $LASTEXITCODE
} finally {
    Pop-Location
    $env:CARGO_TARGET_DIR = $previousTargetDir
}

if ($status -ne 0) {
    Write-Error "tauri build failed with exit code $status"
    exit $status
}

$exe = Join-Path $targetDir 'release\vietkb.exe'
$setup = Get-ChildItem $bundleDir -Filter 'VietKB Dev*setup.exe' -ErrorAction SilentlyContinue |
    Sort-Object LastWriteTime -Descending |
    Select-Object -First 1

if (-not $setup) {
    Write-Error "installer not found under $bundleDir"
    exit 1
}

Write-Host ''
Write-Host "Dev exe:       $exe"
Write-Host "Dev installer: $($setup.FullName)  ($([math]::Round($setup.Length / 1MB, 2)) MB)"
