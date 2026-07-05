$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location (Join-Path $Root "crates/vietkb-gui")

$KeyPath = Join-Path $env:USERPROFILE ".tauri/vietkb.key"
if (-not (Test-Path $KeyPath)) {
    throw "Missing updater private key: $KeyPath"
}

$env:TAURI_SIGNING_PRIVATE_KEY = $KeyPath
cargo tauri build
if ($LASTEXITCODE -ne 0) {
    throw "cargo tauri build failed with exit code $LASTEXITCODE"
}

Write-Host "Built Tauri bundles under: crates/vietkb-gui/target/release/bundle"
Write-Host "Upload installer, .sig, and latest.json to GitHub Releases."
