param([switch]$NoShutdown)
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
$ErrorActionPreference = 'Stop'
$result = 'C:\share\update-check\result.txt'
function Log($kind, $message) { Add-Content -Path $result -Value "$kind $message" }
# NSIS writes InstallLocation/UninstallString with surrounding quotes; strip them before use.
function Clean-Path($value) { return ([string]$value).Trim().Trim('"').Trim() }
$roots = @('HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*','HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*','HKLM:\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*')
try {
    $headers = @{ 'User-Agent' = 'vietkb-update-check'; 'Accept' = 'application/vnd.github+json' }
    if ($env:VERSION) { $version = $env:VERSION } else { $version = (Invoke-RestMethod 'https://api.github.com/repos/thnonl/vietkb/releases/latest' -Headers $headers).tag_name.TrimStart('v') }
    $release = Invoke-RestMethod 'https://api.github.com/repos/thnonl/vietkb/releases/latest' -Headers $headers
    if ($release.tag_name -ne "v$version") { throw "release tag $($release.tag_name) != v$version" }; Log PASS "release tag v$version"
    $asset = $release.assets | Where-Object { $_.name -like '*x64-setup.exe' } | Select-Object -First 1
    if (-not $asset) { throw 'x64 setup asset missing' }; Log PASS 'x64 setup asset found'
    $download = Join-Path $env:TEMP 'vietkb-update.exe'
    Invoke-WebRequest $asset.browser_download_url -Headers $headers -OutFile $download
    if ((Get-Item $download).Length -ne $asset.size) { throw 'download size mismatch' }; Log PASS "asset size $($asset.size)"
    Log PASS "SHA256 $((Get-FileHash $download -Algorithm SHA256).Hash)"
    function Find-App {
        $roots = @('HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*','HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*','HKLM:\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*')
        foreach ($root in $roots) {
            $e = Get-ItemProperty $root -ErrorAction SilentlyContinue | Where-Object { $_.DisplayName -like 'VietKB*' -and $_.InstallLocation } | Select-Object -First 1
            if ($e) { return (Join-Path (Clean-Path $e.InstallLocation) 'vietkb.exe') }
        }
        foreach ($p in @((Join-Path $env:LOCALAPPDATA 'VietKB\vietkb.exe'),(Join-Path $env:ProgramFiles 'VietKB\vietkb.exe'))) { if (Test-Path $p) { return $p } }; return $null
    }
    $releases = Invoke-RestMethod 'https://api.github.com/repos/thnonl/vietkb/releases?per_page=30' -Headers $headers
    $previous = $releases | Where-Object { $_.tag_name -ne "v$version" -and ($_.assets | Where-Object { $_.name -like '*x64-setup.exe' }) } | Select-Object -First 1
    if ($previous) { $old = $previous.assets | Where-Object { $_.name -like '*x64-setup.exe' } | Select-Object -First 1; $oldFile = Join-Path $env:TEMP 'vietkb-previous.exe'; Invoke-WebRequest $old.browser_download_url -OutFile $oldFile; Start-Process $oldFile -ArgumentList '/S' -Wait; $oldExe = Find-App; if ($oldExe -and (Test-Path $oldExe)) { Log PASS "previous version $((Get-Item $oldExe).VersionInfo.FileVersion)" } else { Log SKIP 'previous version' } } else { Log SKIP 'previous version unavailable' }
    Start-Process $download -ArgumentList '/S' -Wait
    $exe = Find-App
    if (-not $exe -or -not (Test-Path $exe)) { throw 'installed exe not found' }
    $installDir = Split-Path $exe -Parent; $fileVersion = (Get-Item $exe).VersionInfo.FileVersion
    if (-not $fileVersion.StartsWith($version)) { throw "FileVersion $fileVersion != $version" }; Log PASS "installed FileVersion $fileVersion"
    $uninstallEntry = Get-ItemProperty $roots -ErrorAction SilentlyContinue | Where-Object { $_.DisplayName -like 'VietKB*' } | Select-Object -First 1
    if ($uninstallEntry.UninstallString) {
        $raw = [string]$uninstallEntry.UninstallString
        $uninstaller = if ($raw -match '^\s*"([^"]+)"') { $matches[1] } else { ($raw -split '\s+')[0] }
    } else { $uninstaller = Join-Path $env:LOCALAPPDATA 'VietKB\uninstall.exe' }
    if (-not (Test-Path $uninstaller)) { throw 'uninstaller missing' }; Start-Process $uninstaller -ArgumentList '/S' -Wait
    $uninstallDeadline = (Get-Date).AddSeconds(60)
    while ((Test-Path $installDir) -and (Get-Date) -lt $uninstallDeadline) { Start-Sleep -Seconds 2 }
    if (Test-Path $installDir) { throw "install directory remains: $installDir" }
    $run = Get-ItemProperty 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run' -Name VietKB -ErrorAction SilentlyContinue
    if ($run) { throw 'VietKB Run value remains' }; Log PASS 'uninstalled; install directory and Run value gone'
} catch { Log FAIL $_.Exception.Message } finally {
    Add-Content -Path $result -Value 'DONE'
    if (-not $NoShutdown) { shutdown.exe /s /t 5 }
}
