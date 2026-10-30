# QA host E2E v3: VS Code and Chrome rows only, rebuilt candidate 0D39D42F (owner-approved 2026-10-30).
# Windows PowerShell 5.1 only:
#   powershell.exe -NoProfile -ExecutionPolicy Bypass -File E:\vietkb\docs\evkey-clone-work\qa\host-e2e\host-e2e3.ps1
# Physical input: ONLY our WH_KEYBOARD_LL / WH_MOUSE_LL watchers (injected flags). GetLastInputInfo: pre-run 60 s idle gate only.
$ErrorActionPreference = 'Continue'
$root    = 'E:\vietkb\docs\evkey-clone-work\qa\host-e2e'
$shots   = Join-Path $root 'shots3'
$scratch = 'C:\Users\namth\AppData\Local\Temp\claude\E--vietkb\9a85ac6a-ade5-453b-9010-44a303edbc97\scratchpad\host-e2e3'
$exe     = 'E:\vietkb\target\release\vietkb.exe'
$setup   = 'E:\vietkb\target\release\bundle\nsis\VietKB_0.2.4_x64-setup.exe'
$exeSha  = '0D39D42F5F66CA4CE40D22068FA31DE940E32D811DD5F393C79FD775FADFEB92'
$setupSha = 'CD45BD84E26DC3A46A4DF51CD96465A31932DEBE310741797AAE190D3812DCBC'
$cfg     = Join-Path $env:APPDATA 'VietKB\config.toml'
$runKey  = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run'
$logPath = Join-Path $root 'run3-log.txt'
$logMd   = 'E:\vietkb\docs\evkey-clone-work\log.md'
$resPath = Join-Path $root 'results3.tsv'
$donePath = Join-Path $root 'run3-done.marker'
$codeExe = 'C:\Users\namth\AppData\Local\Programs\Microsoft VS Code\Code.exe'
$chromeExe = 'C:\Program Files\Google\Chrome\Application\chrome.exe'
New-Item -ItemType Directory -Force -Path $shots, $scratch | Out-Null
Add-Type -AssemblyName System.Drawing, System.Windows.Forms, UIAutomationClient, UIAutomationTypes
Add-Type -Path (Join-Path $root 'QN2.cs') -ReferencedAssemblies System.Drawing, System.Windows.Forms
Remove-Item -LiteralPath $donePath -ErrorAction SilentlyContinue

function L([string]$m) {
    $s = (Get-Date).ToUniversalTime().ToString('HH:mm:ss.fff') + ' ' + $m
    Add-Content -LiteralPath $logPath -Value $s -Encoding UTF8
    Write-Host $s
}
function KillLog([string]$name, [int]$procId, [string]$why) {
    $s = 'KILL ' + (Get-Date).ToUniversalTime().ToString('yyyy-MM-dd HH:mm:ss') + ' ' + $name + ' pid=' + $procId + ' reason=' + $why + ' (launched by QA host-e2e)'
    Add-Content -LiteralPath $logMd -Value $s -Encoding UTF8
    L $s
}

$R = New-Object System.Collections.ArrayList
function Rec([string]$app, [string]$id, [string]$keys, [string]$exp, [string]$act, [string]$verdict, [string]$note = '') {
    [void]$R.Add([pscustomobject]@{ app = $app; id = $id; keys = $keys; expected = $exp; actual = $act; verdict = $verdict; note = $note })
    L ('RESULT [' + $app + '] ' + $id + ' ' + $verdict + ' keys=[' + $keys + '] expected=[' + $exp + '] actual=[' + $act + '] ' + $note)
}
function Norm([string]$s) {
    if ($null -eq $s) { return '' }
    $t = $s.Normalize([Text.NormalizationForm]::FormC).Replace("`r`n", "`n").Replace("`r", "`n")
    return $t.TrimEnd("`n")
}
function Cmp([string]$app, [string]$id, [string]$keys, [string]$exp, [string]$act, [string]$note = '') {
    $e = Norm $exp; $a = Norm $act
    if ($e -ceq $a) { Rec $app $id $keys $e $a 'PASS' $note } else { Rec $app $id $keys $e $a 'FAIL' $note }
}

function WinTitle([IntPtr]$h) {
    try { $p = Get-Process -Id ([QN2]::PidOf($h)) -ErrorAction Stop; return [string]$p.MainWindowTitle } catch { return '<no-title>' }
}

# Settle: our window must be foreground, and its title must be stable for >=3 s. Up to 3 focus attempts;
# each failed attempt logs what took focus (process name, pid and title) before forcing focus back.
function Settle([IntPtr]$h, [string]$app) {
    for ($attempt = 1; $attempt -le 3; $attempt++) {
        $last = $null; $since = Get-Date; $deadline = (Get-Date).AddSeconds(40)
        while ((Get-Date) -lt $deadline) {
            $t = WinTitle $h
            if ($t -ne $last) { $last = $t; $since = Get-Date }
            elseif (((Get-Date) - $since).TotalSeconds -ge 3) { break }
            Start-Sleep -Milliseconds 500
        }
        L ('SETTLE [' + $app + '] attempt=' + $attempt + ' title=[' + $last + '] stableSecs=' + [math]::Round(((Get-Date) - $since).TotalSeconds, 1))
        if ([QN2]::Fg() -eq $h) { L ('SETTLE [' + $app + '] foreground is our window hwnd=' + $h); return }
        $fg = [QN2]::Fg()
        $fgPid = [int][QN2]::PidOf($fg)
        $fgProc = Get-Process -Id $fgPid -ErrorAction SilentlyContinue
        $fgName = if ($fgProc) { $fgProc.ProcessName } else { '<gone>' }
        $fgTitle = if ($fgProc) { [string]$fgProc.MainWindowTitle } else { '' }
        L ('FOCUS-TAKEN [' + $app + '] attempt=' + $attempt + ' by process=' + $fgName + ' pid=' + $fgPid + ' title=[' + $fgTitle + ']')
        [void][QN2]::Force($h)
        if ([QN2]::Fg() -eq $h) { L ('SETTLE [' + $app + '] refocused on attempt ' + $attempt); return }
    }
    throw ('FOCUS-ABORT [' + $app + '] foreground not ours after 3 attempts')
}

# One batch. A pre-batch focus failure triggers Settle and a retry (max 3). A mid-batch focus change aborts (no retry,
# so keys already sent are not repeated).
function Batch([string]$name, [IntPtr]$h, [string]$spec, [bool]$restore = $true, [int]$delay = 35, [string]$app = '') {
    $prev = [QN2]::Fg()
    for ($try = 1; $try -le 3; $try++) {
        $r = [QN2]::Run($name, $h, $spec, $delay)
        L ('BATCH ' + $name + ' try=' + $try + ' prevFg=' + $prev + ' -> ' + $r)
        if ($r -eq 'OK') { break }
        if ($r -like 'ABORT:test window not foreground*' -and $try -lt 3) { L ('FOCUS retry ' + $try + ' for ' + $name); Settle $h $app; continue }
        throw ('RUN-ABORT ' + $r)
    }
    if ($restore) { [void][QN2]::RestoreFg($prev) }
}
function Click1([IntPtr]$h, [int]$x, [int]$y, [string]$name) {
    $r = [QN2]::Click($h, $x, $y, $name)
    L ('CLICK ' + $name + ' at ' + $x + ',' + $y + ' -> ' + $r)
    if ($r -ne 'OK') { throw ('RUN-ABORT ' + $r) }
}

# Verdict text: clipboard copy after the batch; the copy path writes no log lines.
function ReadField([IntPtr]$h, [string]$app) {
    $mark = 'QAMARK' + [guid]::NewGuid().ToString('N')
    Set-Clipboard -Value $mark
    Batch ('copy-' + $app) $h '{ESC}{CTRL+A}{CTRL+C}{CTRL+END}' $true 35 $app
    $got = $null
    for ($i = 0; $i -lt 40; $i++) {
        $c = Get-Clipboard -Raw
        if ($c -is [string] -and $c -ne $mark) { $got = $c; break }
        Start-Sleep -Milliseconds 100
    }
    if ($null -eq $got) { return '<clipboard-no-change>' }
    return $got
}

function Rows {
    return @(
        @{ id = 'A1'; spec = 'dduwowcj';                     exp = 'được';                  chk = $false },
        @{ id = 'A2'; spec = 'tieesng vieetj';               exp = 'tiếng việt';            chk = $false },
        @{ id = 'A3'; spec = 'nghieeng';                     exp = 'nghiêng';               chk = $false },
        @{ id = 'A4'; spec = 'Tieesng Vieetj raast ddepj.';  exp = 'Tiếng Việt rất đẹp.';   chk = $false },
        @{ id = 'A5'; spec = ('dduwowcj ' * 10);             exp = ('được ' * 10);          chk = $false },
        @{ id = 'B1'; spec = 'VIEETJ';                       exp = 'VIỆT';                  chk = $true },
        @{ id = 'B2'; spec = 'Vieejt';                       exp = 'Việt';                  chk = $true },
        @{ id = 'B4'; spec = '{SHIFTDOWN}VIEETJ{SHIFTUP}';   exp = 'VIỆT';                  chk = $true },
        @{ id = 'B6'; spec = '{CAPS_ON}vieejt{CAPS_OFF}';    exp = 'VIỆT';                  chk = $true },
        @{ id = 'E';  spec = 'vieejt{CTRL+SHIFT+Z}';         exp = 'vieejt';                chk = $false }
    )
}

function RunRows([IntPtr]$h, [string]$app) {
    Batch ('clear-' + $app) $h '{CTRL+A}{DEL}' $true 35 $app
    $spec = ''
    $rowsList = Rows
    foreach ($row in $rowsList) {
        $spec += $row.spec
        if ($row.chk) { $spec += '{CHK:' + $row.id + '}' }
        $spec += '{ESC}{ENTER}'
    }
    [QN2]::ClearChecks()
    Batch ('rows-' + $app) $h $spec $true 35 $app
    $chk = [string[]]([QN2]::Checks.ToArray())
    $txt = ReadField $h $app
    [QN2]::Capture(2560, 0, 1920, 1080, (Join-Path $shots ($app + '-rows.png')))
    $lines = (Norm $txt).Split("`n")
    for ($k = 0; $k -lt $rowsList.Count; $k++) {
        $row = $rowsList[$k]
        $act = if ($k -lt $lines.Count) { $lines[$k] } else { '<missing line>' }
        Cmp $app $row.id $row.spec $row.exp $act
    }
    $a5 = if ($lines.Count -gt 4) { $lines[4] } else { '' }
    $toks = @($a5.Split(' ') | Where-Object { $_ -ne '' })
    $wrong = @($toks | Where-Object { $_ -cne 'được' }).Count + [Math]::Max(0, 10 - $toks.Count)
    Rec $app 'A5x10' 'dduwowcj x10' '10 of 10 correct' ((10 - $wrong).ToString() + ' of 10 correct; failures=' + $wrong) $(if ($wrong -eq 0) { 'PASS' } else { 'FAIL' })
    foreach ($c in $chk) {
        L ('SHIFTCHK [' + $app + '] ' + $c)
        $parts = $c.Split('|')
        $stuck = ($c -match 'DOWN')
        Rec $app ('SHIFT-' + $parts[0].Trim()) 'after row' 'no Shift stuck down' $parts[1].Trim() $(if ($stuck) { 'FAIL' } else { 'PASS' }) ($parts[2].Trim())
    }
    return $lines
}

function WaitWin([string]$sub, [uint32]$pid0, [int]$secs) {
    for ($i = 0; $i -lt ($secs * 5); $i++) {
        $w = [QN2]::FindWin($sub, $pid0)
        if ($w -ne [IntPtr]::Zero) { return $w }
        Start-Sleep -Milliseconds 200
    }
    return [IntPtr]::Zero
}

function Kill-Tree([int]$rootPid, [string]$why, [string]$cmdMatch) {
    $all = @(Get-CimInstance Win32_Process)
    $set = New-Object System.Collections.ArrayList
    $q = New-Object System.Collections.Queue
    $q.Enqueue($rootPid)
    while ($q.Count -gt 0) {
        $x = [int]$q.Dequeue()
        if ($set -contains $x) { continue }
        [void]$set.Add($x)
        foreach ($c in ($all | Where-Object { $_.ParentProcessId -eq $x })) { $q.Enqueue([int]$c.ProcessId) }
    }
    if ($cmdMatch) {
        foreach ($c in ($all | Where-Object { $_.CommandLine -and $_.CommandLine.Contains($cmdMatch) })) {
            if (-not ($set -contains [int]$c.ProcessId)) { [void]$set.Add([int]$c.ProcessId) }
        }
    }
    foreach ($x in $set) {
        $pr = Get-Process -Id $x -ErrorAction SilentlyContinue
        if ($pr) { KillLog $pr.ProcessName $x $why; Stop-Process -Id $x -Force -ErrorAction SilentlyContinue }
    }
}

# ------------------------------------------------------------------ run
$vkPid = 0; $codePid = 0; $chromePid = 0
$cfgBak = Join-Path $scratch 'config.backup.toml'
$clipText = $null; $clipHadText = $false
$origRun = $null
$initFg = [QN2]::Fg()
$capsInit = [QN2]::CapsOn()
$cur0 = [System.Windows.Forms.Cursor]::Position
$ownerCodeBefore = @(Get-Process Code -ErrorAction SilentlyContinue).Count
$overall = 'UNSET'
L ('START host E2E v3 (VS Code + Chrome) initFg=' + $initFg + ' capsInit=' + $capsInit + ' cursor=' + $cur0.X + ',' + $cur0.Y + ' owner-Code-procs-before=' + $ownerCodeBefore)

try {
    # ---- 0. hashes
    $hE = (Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash
    $hS = (Get-FileHash -LiteralPath $setup -Algorithm SHA256).Hash
    L ('HASH exe match=' + ($hE -eq $exeSha) + ' installer match=' + ($hS -eq $setupSha))
    if (($hE -ne $exeSha) -or ($hS -ne $setupSha)) { throw 'HASH-MISMATCH: not the approved rebuilt candidate' }

    # ---- 1. owner processes: EVKey, Unikey, installed VietKB (cargo/rustc/vkb_windows test binaries ignored)
    $owner = @(Get-Process | Where-Object { $_.ProcessName -match 'EVKey|UniKey|^vietkb$' -or ($_.Path -and $_.Path -like "$env:LOCALAPPDATA\VietKB\*") })
    if ($owner.Count -gt 0) { throw 'HOLD(owner): EVKey, Unikey or installed VietKB running' }
    L 'PREFLIGHT ok'

    # ---- 2. pre-run idle gate (GetLastInputInfo >= 60 s, up to 5 min). Not used after this point.
    $idleOk = $false
    for ($t = 0; $t -lt 150; $t++) {
        if ([QN2]::IdleMs() -ge 60000) { $idleOk = $true; break }
        Start-Sleep -Seconds 2
    }
    if (-not $idleOk) { throw 'HOLD(idle): owner input kept idle below 60 s for 5 min; nothing was changed' }
    L ('IDLE ok idleMs=' + [QN2]::IdleMs())

    # ---- 3. watchers (physical input detection) and our quiet timer
    [QN2]::StartWatcher()
    [QN2]::MarkInjected()
    L 'WATCHERS armed (WH_KEYBOARD_LL 0x10, WH_MOUSE_LL 0x01)'

    # ---- 4. backups: config, Run key, clipboard
    Copy-Item -LiteralPath $cfg -Destination $cfgBak -Force
    $origHash = (Get-FileHash -LiteralPath $cfg -Algorithm SHA256).Hash
    L ('CONFIG backup hash=' + $origHash)
    $origRun = (Get-ItemProperty -Path $runKey -Name VietKB -ErrorAction SilentlyContinue).VietKB
    L ('RUNKEY VietKB before=' + $origRun)
    $clipHadText = [System.Windows.Forms.Clipboard]::ContainsText()
    if ($clipHadText) { $clipText = [System.Windows.Forms.Clipboard]::GetText() }
    L ('CLIPBOARD had-text=' + $clipHadText)

    # ---- 5. method telex for this run (restored at teardown)
    $txt = [IO.File]::ReadAllText($cfg)
    if (([regex]::Matches($txt, 'method = "vni"')).Count -ne 1) { throw 'CONFIG unexpected method line' }
    [IO.File]::WriteAllText($cfg, $txt.Replace('method = "vni"', 'method = "telex"'), (New-Object Text.UTF8Encoding($false)))
    L 'CONFIG set [input] method = "telex" for this run'

    # ---- 6. launch the rebuilt candidate (only this pid is ever stopped by QA)
    $owner2 = @(Get-Process | Where-Object { $_.ProcessName -match 'EVKey|UniKey|^vietkb$' -or ($_.Path -and $_.Path -like "$env:LOCALAPPDATA\VietKB\*") })
    if ($owner2.Count -gt 0) { throw 'HOLD(owner): owner IME or VietKB appeared before launch' }
    $vk = Start-Process -FilePath $exe -PassThru
    $vkPid = $vk.Id
    Start-Sleep -Seconds 5
    $vkp = Get-Process -Id $vkPid -ErrorAction SilentlyContinue
    if (-not $vkp) { throw 'BLOCKED(redirected-or-exited): candidate exited' }
    if ($vkp.Path -ine $exe) { throw 'BLOCKED(redirected): path is not the target build' }
    L ('LAUNCH vietkb pid=' + $vkPid + ' path=' + $vkp.Path)

    # ---- 7. VS Code: isolated instance (own user-data-dir and extensions-dir, extensions disabled)
    $idV = Get-Random -Maximum 99999
    $fileV = Join-Path $scratch ('qa-vscode-' + $idV + '.txt')
    Set-Content -LiteralPath $fileV -Value '' -Encoding UTF8
    $vsud = Join-Path $scratch 'vscode-ud'
    $vsext = Join-Path $scratch 'vscode-ext'
    $vs = Start-Process -FilePath $codeExe -ArgumentList @('--new-window', '--disable-extensions', '--user-data-dir', $vsud, '--extensions-dir', $vsext, $fileV) -PassThru
    $codePid = $vs.Id
    L ('VSCODE launched pid=' + $codePid + ' flags=--new-window --disable-extensions --user-data-dir --extensions-dir (dropped, unsupported in this build: --disable-workspace-trust --skip-welcome --skip-release-notes; covered by settings.json)')
    $hV = WaitWin ('qa-vscode-' + $idV) ([uint32]$codePid) 60
    if ($hV -eq [IntPtr]::Zero) { $hV = WaitWin ('qa-vscode-' + $idV) 0 10 }
    if ($hV -eq [IntPtr]::Zero) { throw 'SETUP VS Code window not found' }
    [QN2]::Place($hV, 2660, 100, 960, 700)
    L ('VSCODE hwnd=' + $hV + ' rect=' + [QN2]::RectOf($hV))
    Settle $hV 'VS Code'
    RunRows $hV 'VS Code' | Out-Null
    # D: F2, F1, then baatj. The command palette must NOT open; the text must land in the editor.
    Batch 'D-F2-F1-baatj' $hV '{F2}{F1}baatj' $true 35 'VS Code'
    Start-Sleep -Milliseconds 800
    [QN2]::Capture(2560, 0, 1920, 1080, (Join-Path $shots 'vscode-D-after-F1.png'))
    $actD = ReadField $hV 'VS Code'
    $linesD = (Norm $actD).Split("`n")
    $dLine = if ($linesD.Count -ge 11) { $linesD[10] } else { '<missing>' }
    Cmp 'VS Code' 'D1' '{F2}{F1}baatj (palette must not open)' 'bật' $dLine
    [void][QN2]::RestoreFg($initFg)

    # ---- 8. Chrome: fresh user-data-dir, first-run and default-browser prompts off
    $htmlPath = Join-Path $scratch 'qa-textarea.html'
    Set-Content -LiteralPath $htmlPath -Encoding UTF8 -Value '<!doctype html><html><head><meta charset="utf-8"><title>QA textarea</title></head><body style="margin:0"><textarea id="t" autofocus style="width:100%;height:100vh;font-size:20px"></textarea></body></html>'
    $cud = Join-Path $scratch 'chrome-ud'
    $ch = Start-Process -FilePath $chromeExe -ArgumentList @(('--user-data-dir=' + $cud), '--new-window', '--no-first-run', '--no-default-browser-check', '--window-position=2660,100', '--window-size=960,700', ('file:///' + $htmlPath.Replace('\', '/'))) -PassThru
    $chromePid = $ch.Id
    $hC = WaitWin 'QA textarea' ([uint32]$chromePid) 40
    if ($hC -eq [IntPtr]::Zero) { $hC = WaitWin 'QA textarea' 0 5 }
    if ($hC -eq [IntPtr]::Zero) { throw 'SETUP Chrome window not found' }
    [QN2]::Place($hC, 2660, 100, 960, 700)
    L ('CHROME hwnd=' + $hC + ' pid=' + $chromePid + ' rect=' + [QN2]::RectOf($hC))
    Settle $hC 'Chrome'
    RunRows $hC 'Chrome' | Out-Null
    [void][QN2]::RestoreFg($initFg)
    $overall = 'RAN'
}
catch {
    $overall = 'ABORTED: ' + $_.Exception.Message
    L ('ABORT ' + $_.Exception.Message)
}
finally {
    L 'TEARDOWN begin'
    if ($chromePid -ne 0) { Kill-Tree $chromePid 'QA host E2E v3 teardown (Chrome scratch profile, launched by QA)' 'chrome-ud' }
    if ($codePid -ne 0) { Kill-Tree $codePid 'QA host E2E v3 teardown (VS Code isolated profile, launched by QA)' 'vscode-ud' }
    if ($vkPid -ne 0) {
        $pv = Get-Process -Id $vkPid -ErrorAction SilentlyContinue
        if ($pv -and $pv.Path -ieq $exe) { Kill-Tree $vkPid ('QA host E2E v3 teardown: rebuilt target build path=' + $exe + ' (launched by QA)') '' }
    }
    Start-Sleep -Seconds 2
    [QN2]::StopWatcher()
    L ('TEARDOWN candidate processes left=' + @(Get-Process | Where-Object { $_.Path -and ($_.Path -ieq $exe) }).Count)

    # scratch profiles: delete only our own folders
    foreach ($d in @('vscode-ud', 'vscode-ext', 'chrome-ud')) {
        $p = Join-Path $scratch $d
        if (Test-Path -LiteralPath $p) { Remove-Item -LiteralPath $p -Recurse -Force -ErrorAction SilentlyContinue; L ('DELETE scratch profile ' + $p + ' exists-after=' + (Test-Path -LiteralPath $p)) }
    }

    [QN2]::SetCapsState($capsInit)
    L ('CAPSLOCK restored to ' + $(if ($capsInit) { 'on' } else { 'off' }) + ' now=' + $(if ([QN2]::CapsOn()) { 'on' } else { 'off' }))
    [System.Windows.Forms.Cursor]::Position = New-Object System.Drawing.Point($cur0.X, $cur0.Y)
    L ('CURSOR restored to ' + $cur0.X + ',' + $cur0.Y)

    if (Test-Path -LiteralPath $cfgBak) {
        Copy-Item -LiteralPath $cfgBak -Destination $cfg -Force
        $afterHash = (Get-FileHash -LiteralPath $cfg -Algorithm SHA256).Hash
        L ('CONFIG restored hash=' + $afterHash + ' identical=' + ($afterHash -eq $origHash))
    }

    $nowRun = (Get-ItemProperty -Path $runKey -Name VietKB -ErrorAction SilentlyContinue).VietKB
    L ('RUNKEY VietKB after=' + $nowRun + ' unchanged=' + ($nowRun -ceq $origRun))
    if (($nowRun -cne $origRun) -and ($null -ne $origRun)) { Set-ItemProperty -Path $runKey -Name VietKB -Value $origRun -Type String; L 'RUNKEY VietKB restored to original value' }
    $finalRun = (Get-ItemProperty -Path $runKey -Name VietKB -ErrorAction SilentlyContinue).VietKB
    L ('RUNKEY VietKB final byte-identical=' + ($finalRun -ceq $origRun))

    if ($clipHadText) { Set-Clipboard -Value $clipText } else { [System.Windows.Forms.Clipboard]::Clear() }
    L ('CLIPBOARD restored (had-text=' + $clipHadText + ')')
    [void][QN2]::RestoreFg($initFg)
    L ('FOREGROUND restored to ' + $initFg + ' now=' + [QN2]::Fg())
    L ('OWNER VS Code processes before=' + $ownerCodeBefore + ' after=' + @(Get-Process Code -ErrorAction SilentlyContinue).Count + ' (owner windows not touched by QA)')

    $R | Export-Csv -LiteralPath $resPath -NoTypeInformation -Delimiter "`t" -Encoding UTF8
    L ('OVERALL ' + $overall)
    L 'END'
    Set-Content -LiteralPath $donePath -Value ($overall + ' ' + (Get-Date).ToUniversalTime().ToString('o')) -Encoding UTF8
}
