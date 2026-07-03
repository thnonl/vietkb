$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$InstallDir = Join-Path $env:LOCALAPPDATA "VietKB"
& (Join-Path $PSScriptRoot "build-windows.ps1")
New-Item -ItemType Directory -Force $InstallDir | Out-Null
Copy-Item (Join-Path $Root "dist/windows/*") $InstallDir -Force
$Shortcut = Join-Path ([Environment]::GetFolderPath("Programs")) "VietKB.lnk"
$Wsh = New-Object -ComObject WScript.Shell
$Link = $Wsh.CreateShortcut($Shortcut)
$Link.TargetPath = Join-Path $InstallDir "vietkb-gui.exe"
$Link.WorkingDirectory = $InstallDir
$Link.Save()
Write-Host "Installed VietKB to $InstallDir"
