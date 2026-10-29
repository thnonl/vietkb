param([switch]$NoShutdown)
# Proves the About window's update button installs in place, silently, and reopens
# the app: install an older build, run it, then hand the new setup to the same
# batch the updater writes (`<setup> /S /UPDATE` + `start vietkb.exe`).
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
$ErrorActionPreference = 'Stop'
$result = 'C:\share\update-check\inplace.txt'
function Log($kind, $message) { Add-Content -Path $result -Value "$kind $message" }

try {
    $version = $env:VERSION
    $previous = $env:PREVIOUS
    if (-not $version) { throw 'VERSION not set' }
    if (-not $previous) { throw 'PREVIOUS not set' }

    $headers = @{ 'User-Agent' = 'vietkb-update-check'; 'Accept' = 'application/vnd.github+json' }
    function Get-Setup($tag) {
        $rel = Invoke-RestMethod "https://api.github.com/repos/thnonl/vietkb/releases/tags/$tag" -Headers $headers
        $exe = $rel.assets | Where-Object { $_.name -like '*x64-setup.exe' } | Select-Object -First 1
        if (-not $exe) { throw "no Windows x64 setup in $tag" }
        return $exe
    }

    $old = Get-Setup "v$previous"
    $new = Get-Setup "v$version"
    $oldFile = Join-Path $env:TEMP 'vietkb-old.exe'
    $newFile = Join-Path $env:TEMP 'vietkb-new.exe'
    Invoke-WebRequest $old.browser_download_url -OutFile $oldFile
    Invoke-WebRequest $new.browser_download_url -OutFile $newFile
    Log PASS "downloaded v$previous and v$version setups"

    # 1. The older build, installed the way a user installs it.
    Start-Process $oldFile -ArgumentList '/S' -Wait
    $exe = Join-Path $env:LOCALAPPDATA 'VietKB\vietkb.exe'
    if (-not (Test-Path $exe)) { throw 'the previous build did not install' }
    $dir = Split-Path $exe -Parent
    Log PASS "installed v$((Get-Item $exe).VersionInfo.FileVersion)"

    # 2. Two sentinels. An uninstall/reinstall cycle removes the install directory,
    #    so either one surviving is proof the run was an in-place update.
    Set-Content -Path (Join-Path $dir 'inplace-sentinel.txt') -Value 'keep me'
    $cfgDir = Join-Path $env:APPDATA 'VietKB'
    New-Item -ItemType Directory -Force -Path $cfgDir | Out-Null
    Set-Content -Path (Join-Path $cfgDir 'inplace-marker.txt') -Value 'keep me'
    Log PASS 'wrote sentinel + config marker'

    # 3. Run it, so the installer has a live process to close.
    $before = Start-Process $exe -PassThru
    Start-Sleep -Seconds 5
    if (-not (Get-Process -Id $before.Id -ErrorAction SilentlyContinue)) { throw 'the app is not running before the update' }
    Log PASS "app running before update (pid $($before.Id))"

    # 4. The same command line the updater's batch runs, but timed so a hung
    #    installer is reported instead of blocking the whole sandbox.
    $sw = [Diagnostics.Stopwatch]::StartNew()
    $setup = Start-Process $newFile -ArgumentList '/S', '/UPDATE' -PassThru
    if (-not $setup.WaitForExit(240000)) {
        try { Stop-Process -Id $setup.Id -Force } catch {}
        Log FAIL 'setup /S /UPDATE did not finish in 240 s'
    } else {
        Log PASS "setup /S /UPDATE exit $($setup.ExitCode) in $([math]::Round($sw.Elapsed.TotalSeconds, 1)) s"
    }
    $sw.Stop()
    Start-Process $exe | Out-Null

    # 5. What the update has to leave behind.
    $after = (Get-Item $exe).VersionInfo.FileVersion
    if ($after.StartsWith($version)) { Log PASS "FileVersion now $after" } else { Log FAIL "FileVersion $after != $version" }

    if (Test-Path (Join-Path $dir 'inplace-sentinel.txt')) {
        Log PASS 'install directory kept its sentinel — the old uninstaller never ran'
    } else { Log FAIL 'sentinel gone: the old build was uninstalled first' }

    if (Test-Path (Join-Path $cfgDir 'inplace-marker.txt')) {
        Log PASS 'config directory kept its marker'
    } else { Log FAIL 'config marker gone' }

    if (-not (Get-Process -Id $before.Id -ErrorAction SilentlyContinue)) {
        Log PASS 'the installer closed the old process'
    } else { Log FAIL 'the old process survived the update' }

    Start-Sleep -Seconds 3
    $deadline = (Get-Date).AddSeconds(30)
    $running = @(Get-Process vietkb -ErrorAction SilentlyContinue)
    while ($running.Count -eq 0 -and (Get-Date) -lt $deadline) {
        Start-Sleep -Seconds 2
        $running = @(Get-Process vietkb -ErrorAction SilentlyContinue)
    }
    if ($running.Count -gt 0) { Log PASS "app reopened after the update (pid $($running[0].Id))" } else { Log FAIL 'app did not come back' }
    if ($running.Count -gt 1) { Log FAIL "$($running.Count) app processes after the update" }
    elseif ($running.Count -eq 1) { Log PASS 'exactly one app process' }

    $entry = Get-ItemProperty 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*' -ErrorAction SilentlyContinue |
        Where-Object { $_.DisplayName -like 'VietKB*' } | Select-Object -First 1
    if ($entry.DisplayVersion -eq $version) { Log PASS "registry DisplayVersion $($entry.DisplayVersion)" }
    else { Log FAIL "registry DisplayVersion $($entry.DisplayVersion) != $version" }

    $running | Stop-Process -Force
} catch {
    Log FAIL $_.Exception.Message
} finally {
    Add-Content -Path $result -Value 'DONE'
    if (-not $NoShutdown) { shutdown.exe /s /t 5 }
}
