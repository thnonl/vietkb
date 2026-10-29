# Runs the paragraph typing E2E inside Windows Sandbox and waits for its log.
#
#   powershell -NoProfile -ExecutionPolicy Bypass -File scripts\run-sandbox-e2e.ps1
#   ... -TimeoutMinutes 45
param(
    [int]$TimeoutMinutes = 45
)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$e2e = Join-Path $root 'docs\evkey-clone-work\qa\e2e'
$log = Join-Path $e2e 'sandbox-e2e.log'
$results = Join-Path $e2e 'host-paragraph-results.txt'
$wsb = Join-Path $env:TEMP 'vietkb-e2e.wsb'
Remove-Item $log, $results -Force -ErrorAction SilentlyContinue

@"
<Configuration>
  <Networking>Enable</Networking>
  <MappedFolders>
    <MappedFolder><HostFolder>$root</HostFolder><SandboxFolder>C:\vkb</SandboxFolder><ReadOnly>false</ReadOnly></MappedFolder>
  </MappedFolders>
  <LogonCommand><Command>powershell.exe -NoProfile -ExecutionPolicy Bypass -File C:\vkb\docs\evkey-clone-work\qa\e2e\sandbox-e2e.ps1</Command></LogonCommand>
</Configuration>
"@ | Set-Content -Encoding UTF8 $wsb

Start-Process WindowsSandbox.exe -ArgumentList "`"$wsb`""

$deadline = (Get-Date).AddMinutes($TimeoutMinutes)
while ((Get-Date) -lt $deadline) {
    if (Test-Path $log) {
        $text = Get-Content $log -Raw -Encoding UTF8
        if ($text -match 'SANDBOX-E2E-DONE') { break }
    }
    Start-Sleep -Seconds 10
}

if (-not (Test-Path $log)) { throw 'sandbox log was never written' }
Get-Content $log -Encoding UTF8 | Select-String -Pattern 'SANDBOX-E2E|app_ready|E2E_PASS|CASE .*FAIL|first_diff|runner_error|os=' | ForEach-Object { $_.Line }
if (-not (Get-Content $log -Raw -Encoding UTF8).Contains('SANDBOX-E2E-DONE')) { throw 'sandbox E2E timed out' }
