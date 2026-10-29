param(
    [string]$Version,
    [string]$Previous,
    [string]$Script = 'in-sandbox.ps1',
    [string]$ResultName = 'result.txt',
    [bool]$Sandbox = $true,
    [bool]$KeepSandbox = $false
)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$share = Join-Path $root 'sandbox-share'
$check = Join-Path $share 'update-check'
New-Item -ItemType Directory -Force -Path $check | Out-Null
$result = Join-Path $check $ResultName
Remove-Item $result -Force -ErrorAction SilentlyContinue
$inner = Join-Path $check $Script
Copy-Item (Join-Path $PSScriptRoot "update-check\$Script") $inner -Force
if ($Version) { $env:VERSION = $Version } else { Remove-Item Env:\VERSION -ErrorAction SilentlyContinue }
if ($Previous) { $env:PREVIOUS = $Previous } else { Remove-Item Env:\PREVIOUS -ErrorAction SilentlyContinue }
if (-not $Sandbox) {
    & powershell.exe -NoProfile -ExecutionPolicy Bypass -File $inner -NoShutdown
} else {
    $wsb = Join-Path $root 'update-check.wsb'
    @"
<Configuration>
  <Networking>Enable</Networking>
  <MappedFolders><MappedFolder><HostFolder>$share</HostFolder><SandboxFolder>C:\share</SandboxFolder><ReadOnly>false</ReadOnly></MappedFolder></MappedFolders>
  <LogonCommand><Command>powershell.exe -NoProfile -ExecutionPolicy Bypass -File C:\share\update-check\launch.ps1</Command></LogonCommand>
</Configuration>
"@ | Set-Content -Encoding UTF8 $wsb
    $launch = @()
    if ($Version) { $launch += "`$env:VERSION='$Version'" }
    if ($Previous) { $launch += "`$env:PREVIOUS='$Previous'" }
    $launch += "powershell.exe -NoProfile -ExecutionPolicy Bypass -File C:\share\update-check\$Script"
    $launch | Set-Content -Encoding ASCII (Join-Path $check 'launch.ps1')
    Start-Process WindowsSandbox.exe -ArgumentList "`"$wsb`""
}
$deadline = (Get-Date).AddMinutes(15)
while ((Get-Date) -lt $deadline) {
    if ((Test-Path $result) -and (Select-String -Path $result -Pattern '^DONE$' -Quiet)) { break }
    Start-Sleep -Seconds 10
}
if (-not (Test-Path $result)) { throw 'Result file not created before timeout' }
Get-Content $result
if (-not (Select-String -Path $result -Pattern '^DONE$' -Quiet)) { throw 'Update verification timed out' }
if (Select-String -Path $result -Pattern '^FAIL') { exit 1 }
