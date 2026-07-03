$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

$TargetGuiExe = Join-Path $Root "target/windows/release/vietkb-gui.exe"
$DistGuiExe = Join-Path $Root "dist/windows/vietkb-gui.exe"
$RunningGui = Get-Process -Name "vietkb-gui" -ErrorAction SilentlyContinue | Where-Object {
    $_.Path -eq $TargetGuiExe -or $_.Path -eq $DistGuiExe
}
if ($RunningGui) {
    $ProcessList = ($RunningGui | ForEach-Object { "PID $($_.Id): $($_.Path)" }) -join "; "
    throw "vietkb-gui.exe is running ($ProcessList). Close it before building."
}

cargo build --release --target-dir target/windows -p vietkb-gui
if ($LASTEXITCODE -ne 0) {
    throw "cargo build failed with exit code $LASTEXITCODE"
}

Remove-Item dist/windows -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force dist/windows | Out-Null
Copy-Item target/windows/release/vietkb-gui.exe dist/windows/ -Force
Write-Host "Built: dist/windows"
