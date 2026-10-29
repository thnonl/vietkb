$ErrorActionPreference = 'Continue'
# Entry point inside Windows Sandbox: runs the paragraph E2E against the dev build
# that the host repository folder exposes at C:\vkb.
$root = 'C:\vkb'
$here = Join-Path $root 'docs\evkey-clone-work\qa\e2e'
$log = Join-Path $here 'sandbox-e2e.log'
Set-Content -Encoding UTF8 $log 'SANDBOX-E2E-START'
Set-Content -Encoding UTF8 (Join-Path $here 'host-paragraph-results.txt') 'HOST-PARAGRAPH-E2E-START'

Add-Content -Encoding UTF8 $log ("os=" + [Environment]::OSVersion.VersionString)
Add-Content -Encoding UTF8 $log ("exe=" + (Test-Path (Join-Path $root 'target-dev\release\vietkb.exe')))

try {
    & powershell.exe -NoProfile -ExecutionPolicy Bypass -File (Join-Path $here 'host-e2e-paragraphs.ps1') -Only both *>> $log
} catch {
    Add-Content -Encoding UTF8 $log ("runner_error=" + $_.Exception.Message)
}

Add-Content -Encoding UTF8 $log 'SANDBOX-E2E-DONE'
Start-Sleep -Seconds 3
shutdown.exe /s /t 5
