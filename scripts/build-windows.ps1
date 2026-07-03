$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root
cargo build --release --target-dir target/windows -p vietkb-gui -p vietkb-cli
New-Item -ItemType Directory -Force dist/windows | Out-Null
Copy-Item target/windows/release/vietkb-gui.exe dist/windows/ -Force
Copy-Item target/windows/release/vietkb-cli.exe dist/windows/ -Force
Write-Host "Built: dist/windows"
