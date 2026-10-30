# QA host E2E v2 for VietKB 0.2.4 REBUILT candidate (0D39D42F…, D-E2E-06 fix main 32e6f06). Owner-approved 2026-10-30.
# Windows PowerShell 5.1 only:
#   powershell.exe -NoProfile -ExecutionPolicy Bypass -File E:\vietkb\docs\evkey-clone-work\qa\host-e2e\host-e2e2.ps1
# Physical input: ONLY our WH_KEYBOARD_LL / WH_MOUSE_LL watchers (injected flags). GetLastInputInfo: pre-run 60 s idle gate only.
$ErrorActionPreference = 'Continue'
$root    = 'E:\vietkb\docs\evkey-clone-work\qa\host-e2e'
$shots   = Join-Path $root 'shots2'
$scratch = 'C:\Users\namth\AppData\Local\Temp\claude\E--vietkb\9a85ac6a-ade5-453b-9010-44a303edbc97\scratchpad\host-e2e2'
$exe     = 'E:\vietkb\target\release\vietkb.exe'
$setup   = 'E:\vietkb\target\release\bundle\nsis\VietKB_0.2.4_x64-setup.exe'
$exeSha  = '0D39D42F5F66CA4CE40D22068FA31DE940E32D811DD5F393C79FD775FADFEB92'
$setupSha = 'CD45BD84E26DC3A46A4DF51CD96465A31932DEBE310741797AAE190D3812DCBC'
$cfg     = Join-Path $env:APPDATA 'VietKB\config.toml'
$runKey  = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run'
$logPath = Join-Path $root 'run2-log.txt'
$logMd   = 'E:\vietkb\docs\evkey-clone-work\log.md'
$resPath = Join-Path $root 'results2.tsv'
$donePath = Join-Path $root 'run2-done.marker'
New-Item -ItemType Directory -Force -Path $shots, $scratch | Out-Null
Add-Type -AssemblyName System.Drawing, System.Windows.Forms, UIAutomationClient, UIAutomationTypes
Add-Type -Path (Join-Path $root 'QN2.cs') -ReferencedAssemblies System.Drawing, System.Windows.Forms
Remove-Item -LiteralPath $donePath -ErrorAction SilentlyContinue

# Logging: Write-Host only (never the pipeline), so function return values stay clean.
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

# One batch: guard (5 s own quiet + watchers), foreground assert, optional restore of previous foreground.
function Batch([string]$name, [IntPtr]$h, [string]$spec, [bool]$restore = $true, [int]$delay = 35) {
    $prev = [QN2]::Fg()
    $r = [QN2]::Run($name, $h, $spec, $delay)
    L ('BATCH ' + $name + ' prevFg=' + $prev + ' -> ' + $r)
    if ($r -ne 'OK') { throw ('RUN-ABORT ' + $r) }
    if ($restore) { [void][QN2]::RestoreFg($prev) }
}
function Click1([IntPtr]$h, [int]$x, [int]$y, [string]$name) {
    $r = [QN2]::Click($h, $x, $y, $name)
    L ('CLICK ' + $name + ' at ' + $x + ',' + $y + ' -> ' + $r)
    if ($r -ne 'OK') { throw ('RUN-ABORT ' + $r) }
}

# Verdict text comes from the clipboard copy taken AFTER all log writes for this step (no log lines in the copy path).
function ReadField([IntPtr]$h, [string]$app) {
    $mark = 'QAMARK' + [guid]::NewGuid().ToString('N')
    Set-Clipboard -Value $mark
    Batch ('copy-' + $app) $h '{ESC}{CTRL+A}{CTRL+C}{CTRL+END}' $true 35
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
        @{ id = 'A1'; spec = 'dduwowcj';                     exp = 'được';         chk = $false; rec = $false },
        @{ id = 'A2'; spec = 'tieesng vieetj';               exp = 'tiếng việt';   chk = $false; rec = $false },
        @{ id = 'A3'; spec = 'nghieeng';                     exp = 'nghiêng';      chk = $false; rec = $false },
        @{ id = 'A4'; spec = 'Tieesng Vieetj raast ddepj.';  exp = 'Tiếng Việt rất đẹp.'; chk = $false; rec = $false },
        @{ id = 'A5'; spec = ('dduwowcj ' * 10);             exp = ('được ' * 10); chk = $false; rec = $false },
        @{ id = 'B1'; spec = 'VIEETJ';                       exp = 'VIỆT';         chk = $true;  rec = $false },
        @{ id = 'B2'; spec = 'Vieejt';                       exp = 'Việt';         chk = $true;  rec = $false },
        @{ id = 'B4'; spec = '{SHIFTDOWN}VIEETJ{SHIFTUP}';   exp = 'VIỆT';         chk = $true;  rec = $false },
        @{ id = 'B5'; spec = 'duong{SHIFT+J}';               exp = 'dượng';        chk = $true;  rec = $true },
        @{ id = 'B6'; spec = '{CAPS_ON}vieejt{CAPS_OFF}';    exp = 'VIỆT';         chk = $true;  rec = $false }
    )
}

function RunRows([IntPtr]$h, [string]$app) {
    Batch ('clear-' + $app) $h '{CTRL+A}{DEL}' $true 35
    $spec = ''
    foreach ($row in (Rows)) {
        $spec += $row.spec
        if ($row.chk) { $spec += '{CHK:' + $row.id + '}' }
        $spec += '{ESC}{ENTER}'
    }
    [QN2]::ClearChecks()
    Batch ('rows-' + $app) $h $spec $true 35
    $chk = [string[]]([QN2]::Checks.ToArray())
    $txt = ReadField $h $app
    [QN2]::Capture(2560, 0, 1920, 1080, (Join-Path $shots ($app + '-rows.png')))
    $lines = (Norm $txt).Split("`n")
    $rowsList = Rows
    for ($k = 0; $k -lt $rowsList.Count; $k++) {
        $row = $rowsList[$k]
        $act = if ($k -lt $lines.Count) { $lines[$k] } else { '<missing line>' }
        if ($row.rec) {
            $note = if ((Norm $act) -ceq (Norm $row.exp)) { 'same as expected' } else { 'DIFFERS from dượng (recorded, not a pass/fail row)' }
            Rec $app $row.id $row.spec $row.exp $act 'RECORD' $note
        } else {
            Cmp $app $row.id $row.spec $row.exp $act
        }
    }
    # A5 per-token failure count
    $a5 = if ($lines.Count -gt 4) { $lines[4] } else { '' }
    $toks = @($a5.Split(' ') | Where-Object { $_ -ne '' })
    $wrong = @($toks | Where-Object { $_ -cne 'được' }).Count + [Math]::Max(0, 10 - $toks.Count)
    $verdictA5 = if ($wrong -eq 0) { 'PASS' } else { 'FAIL' }
    Rec $app 'A5x10' 'dduwowcj x10' '10 of 10 correct' ((10 - $wrong).ToString() + ' of 10 correct; failures=' + $wrong) $verdictA5
    # Shift-stuck check after every B row (GetAsyncKeyState, recorded from inside the batch)
    foreach ($c in $chk) {
        L ('SHIFTCHK [' + $app + '] ' + $c)
        $parts = $c.Split('|')
        $rid = $parts[0].Trim()
        $stuck = ($c -match 'DOWN')
        $v = if ($stuck) { 'FAIL' } else { 'PASS' }
        Rec $app ('SHIFT-' + $rid) 'after row' 'no Shift stuck down; caps as set' $parts[1].Trim() ($parts[1].Trim() + ' | ' + $parts[2].Trim()) $v
    }
}

function EditorRect([IntPtr]$h) {
    try {
        $rootEl = [System.Windows.Automation.AutomationElement]::FromHandle($h)
        $c1 = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ControlTypeProperty, [System.Windows.Automation.ControlType]::Document)
        $c2 = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ControlTypeProperty, [System.Windows.Automation.ControlType]::Edit)
        $cond = New-Object System.Windows.Automation.OrCondition($c1, $c2)
        $el = $rootEl.FindFirst([System.Windows.Automation.TreeScope]::Descendants, $cond)
        if ($el) { return $el.Current.BoundingRectangle }
    } catch { L ('UIA error: ' + $_.Exception.Message) }
    return $null
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
$vkPid = 0; $notePid = 0; $chromePid = 0; $codePid = 0
$cfgBak = Join-Path $scratch 'config.backup.toml'
$clipText = $null; $clipHadText = $false
$origRun = $null
$initFg = [QN2]::Fg()
$capsInit = [QN2]::CapsOn()
$cur0 = [System.Windows.Forms.Cursor]::Position
$overall = 'UNSET'
L ('START host E2E v2 (rebuilt candidate); initFg=' + $initFg + ' capsLockInit=' + $capsInit + ' cursor=' + $cur0.X + ',' + $cur0.Y)

try {
    # ---- 0. hashes
    $hE = (Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash
    $hS = (Get-FileHash -LiteralPath $setup -Algorithm SHA256).Hash
    L ('HASH exe=' + $hE + ' match=' + ($hE -eq $exeSha) + ' | installer=' + $hS + ' match=' + ($hS -eq $setupSha))
    if (($hE -ne $exeSha) -or ($hS -ne $setupSha)) { throw 'HASH-MISMATCH: build is not the approved rebuilt candidate' }

    # ---- 1. owner processes (EVKey, Unikey, installed VietKB). cargo/rustc/vkb_windows test binaries are ignored.
    $owner = @(Get-Process | Where-Object { $_.ProcessName -match 'EVKey|UniKey|^vietkb$' -or ($_.Path -and $_.Path -like "$env:LOCALAPPDATA\VietKB\*") })
    foreach ($o in $owner) { L ('PREFLIGHT owner process ' + $o.ProcessName + ' pid=' + $o.Id + ' path=' + $o.Path) }
    if ($owner.Count -gt 0) { throw 'HOLD(owner): EVKey, Unikey or installed VietKB running' }
    L 'PREFLIGHT ok (no EVKey, Unikey or installed VietKB)'

    # ---- 2. pre-run idle gate: GetLastInputInfo >= 60 s, poll up to 5 min. Not used after this point.
    $idleOk = $false
    for ($t = 0; $t -lt 150; $t++) {
        if ([QN2]::IdleMs() -ge 60000) { $idleOk = $true; break }
        Start-Sleep -Seconds 2
    }
    if (-not $idleOk) { throw 'HOLD(idle): owner input kept idle below 60 s for 5 min; nothing was changed' }
    L ('IDLE ok idleMs=' + [QN2]::IdleMs())

    # ---- 3. arm physical-input watchers (keyboard + mouse, injected flags) and our quiet timer
    [QN2]::StartWatcher()
    [QN2]::MarkInjected()
    L 'WATCHERS armed: WH_KEYBOARD_LL (0x10) and WH_MOUSE_LL (0x01)'

    # ---- 4. backups: config, Run key, clipboard
    Copy-Item -LiteralPath $cfg -Destination $cfgBak -Force
    $origHash = (Get-FileHash -LiteralPath $cfg -Algorithm SHA256).Hash
    $bakHash = (Get-FileHash -LiteralPath $cfgBak -Algorithm SHA256).Hash
    L ('CONFIG backup hash=' + $bakHash + ' orig=' + $origHash + ' equal=' + ($origHash -eq $bakHash))
    $origRun = (Get-ItemProperty -Path $runKey -Name VietKB -ErrorAction SilentlyContinue).VietKB
    L ('RUNKEY VietKB before=' + $origRun)
    $clipHadText = [System.Windows.Forms.Clipboard]::ContainsText()
    if ($clipHadText) { $clipText = [System.Windows.Forms.Clipboard]::GetText() }
    L ('CLIPBOARD had-text=' + $clipHadText)

    # ---- 5. Notepad (before the candidate is launched)
    $idN = Get-Random -Maximum 99999
    $fileN = Join-Path $scratch ('qa-notepad-' + $idN + '.txt')
    Set-Content -LiteralPath $fileN -Value '' -Encoding UTF8
    $np = Start-Process -FilePath 'notepad.exe' -ArgumentList ('"' + $fileN + '"') -PassThru
    $hN = WaitWin ('qa-notepad-' + $idN) 0 15
    if ($hN -eq [IntPtr]::Zero) { throw 'SETUP Notepad window not found' }
    $notePid = [int][QN2]::PidOf($hN)
    [QN2]::Place($hN, 2660, 100, 960, 700)
    if (-not [QN2]::Force($hN)) { throw 'FOCUS Notepad window not foreground after Force' }
    L ('NOTEPAD hwnd=' + $hN + ' pid=' + $notePid + ' rect=' + [QN2]::RectOf($hN))

    # ---- 6. set method telex (only change; restored byte-identical at teardown)
    $txt = [IO.File]::ReadAllText($cfg)
    $cnt = ([regex]::Matches($txt, 'method = "vni"')).Count
    if ($cnt -ne 1) { throw ('CONFIG unexpected method count ' + $cnt) }
    [IO.File]::WriteAllText($cfg, $txt.Replace('method = "vni"', 'method = "telex"'), (New-Object Text.UTF8Encoding($false)))
    L 'CONFIG set [input] method = "telex" for this run'

    # ---- 7. owner re-check right before launching the candidate, then launch (only this pid is ever stopped)
    $owner2 = @(Get-Process | Where-Object { $_.ProcessName -match 'EVKey|UniKey|^vietkb$' -or ($_.Path -and $_.Path -like "$env:LOCALAPPDATA\VietKB\*") })
    if ($owner2.Count -gt 0) { throw 'HOLD(owner): owner IME or VietKB appeared before launch' }
    $vk = Start-Process -FilePath $exe -PassThru
    $vkPid = $vk.Id
    Start-Sleep -Seconds 5
    $vkp = Get-Process -Id $vkPid -ErrorAction SilentlyContinue
    if (-not $vkp) { throw 'BLOCKED(redirected-or-exited): launched candidate exited' }
    L ('LAUNCH vietkb pid=' + $vkPid + ' path=' + $vkp.Path)
    if ($vkp.Path -ine $exe) { throw 'BLOCKED(redirected): process path is not the target build' }
    $inst = @(Get-Process | Where-Object { $_.Path -and $_.Path -like "$env:LOCALAPPDATA\VietKB\*" })
    if ($inst.Count -gt 0) { throw 'BLOCKED(redirected): installed VietKB appeared' }
    L ('RUNKEY VietKB after launch=' + (Get-ItemProperty -Path $runKey -Name VietKB -ErrorAction SilentlyContinue).VietKB)

    # ---- 8. Notepad: A1-A5, B1, B2, B4, B5, B6 (shift checks), C, E, H
    RunRows $hN 'Notepad'

    Batch 'clear-C' $hN '{CTRL+A}{DEL}' $true 35
    Batch 'C-type' $hN 'xyz{ENTER}vie' $false 35
    $rc = EditorRect $hN
    if ($rc) { $cx = [int]($rc.Left + 300); $cy = [int]($rc.Top + 12) } else { $cx = 2660 + 300; $cy = 100 + 90 }
    Click1 $hN $cx $cy 'C-click-line1-end'
    Batch 'C-et' $hN 'et' $true 35
    [QN2]::Capture(2560, 0, 1920, 1080, (Join-Path $shots 'notepad-C-run2.png'))
    $actC = ReadField $hN 'Notepad'
    Cmp 'Notepad' 'C1' 'xyz{ENTER}vie | click end of line 1 | et' ("xyzet`nvie") $actC

    Batch 'clear-E' $hN '{CTRL+A}{DEL}' $true 35
    Batch 'E-type-restore' $hN 'vieejt{CTRL+SHIFT+Z}' $true 35
    $actE = ReadField $hN 'Notepad'
    Cmp 'Notepad' 'E1' 'vieejt then Ctrl+Shift+Z' 'vieejt' $actE

    Batch 'H-open-settings' $hN '{CTRL+ALT+S}' $true 35
    Start-Sleep -Seconds 3
    $hS = WaitWin '' $vkPid 10
    if ($hS -eq [IntPtr]::Zero) {
        Rec 'VietKB' 'H1' 'Ctrl+Alt+S' 'settings window visible' '<not found>' 'FAIL' 'no visible window of candidate pid'
    } else {
        Rec 'VietKB' 'H1' 'Ctrl+Alt+S' 'settings window visible' ('hwnd=' + $hS + ' rect=' + [QN2]::RectOf($hS)) 'PASS'
        Start-Sleep -Seconds 2
        [QN2]::Capture(2560, 0, 1920, 1080, (Join-Path $shots 'H-settings-display1-run2.png'))
        [QN2]::CloseWin($hS)
    }
    [QN2]::Capture(1600, 1100, 448, 52, (Join-Path $shots 'H-tray-primary-run2.png'))
    Rec 'Shell' 'H3' 'tray area' 'screenshot of tray icon area' 'shots2\H-tray-primary-run2.png' 'CAPTURED' 'owner to view'
    [void][QN2]::RestoreFg($initFg)

    # ---- 9. VS Code (scratch instance, own user-data-dir): rows, then D (F2, F1, baatj)
    $idV = Get-Random -Maximum 99999
    $fileV = Join-Path $scratch ('qa-vscode-' + $idV + '.txt')
    Set-Content -LiteralPath $fileV -Value '' -Encoding UTF8
    $vsud = Join-Path $scratch 'vscode-ud'
    $codeExe = 'C:\Users\namth\AppData\Local\Programs\Microsoft VS Code\Code.exe'
    $vs = Start-Process -FilePath $codeExe -ArgumentList @('--user-data-dir', $vsud, '--new-window', '--disable-workspace-trust', '--skip-release-notes', '--skip-welcome', $fileV) -PassThru
    $codePid = $vs.Id
    $hV = WaitWin ('qa-vscode-' + $idV) 0 40
    if ($hV -eq [IntPtr]::Zero) { throw 'SETUP VS Code window not found' }
    [QN2]::Place($hV, 2660, 100, 960, 700)
    Start-Sleep -Seconds 4
    if (-not [QN2]::Force($hV)) { throw 'FOCUS VS Code window not foreground after Force' }
    L ('VSCODE hwnd=' + $hV + ' pid=' + $codePid + ' rect=' + [QN2]::RectOf($hV))
    RunRows $hV 'VS Code'
    Batch 'D-F2-F1-baatj' $hV '{F2}{F1}baatj' $true 35
    [QN2]::Capture(2560, 0, 1920, 1080, (Join-Path $shots 'vscode-D-after-F1-run2.png'))
    $actD = ReadField $hV 'VS Code'
    $linesD = (Norm $actD).Split("`n")
    $dLine = if ($linesD.Count -ge 11) { $linesD[10] } else { '<missing>' }
    Cmp 'VS Code' 'D1' '{F2}{F1}baatj' 'bật' $dLine
    [void][QN2]::RestoreFg($initFg)

    # ---- 10. Chrome (scratch profile, local textarea): rows only
    $htmlPath = Join-Path $scratch 'qa-textarea.html'
    Set-Content -LiteralPath $htmlPath -Encoding UTF8 -Value '<!doctype html><html><head><meta charset="utf-8"><title>QA textarea</title></head><body style="margin:0"><textarea id="t" autofocus style="width:100%;height:100vh;font-size:20px"></textarea></body></html>'
    $cud = Join-Path $scratch 'chrome-ud'
    $chromeExe = 'C:\Program Files\Google\Chrome\Application\chrome.exe'
    $ch = Start-Process -FilePath $chromeExe -ArgumentList @(('--user-data-dir=' + $cud), '--new-window', '--no-first-run', '--no-default-browser-check', '--window-position=2660,100', '--window-size=960,700', ('file:///' + $htmlPath.Replace('\', '/'))) -PassThru
    $chromePid = $ch.Id
    $hC = WaitWin 'QA textarea' ([uint32]$chromePid) 30
    if ($hC -eq [IntPtr]::Zero) { $hC = WaitWin 'QA textarea' 0 5 }
    if ($hC -eq [IntPtr]::Zero) { throw 'SETUP Chrome window not found' }
    [QN2]::Place($hC, 2660, 100, 960, 700)
    Start-Sleep -Seconds 2
    if (-not [QN2]::Force($hC)) { throw 'FOCUS Chrome window not foreground after Force' }
    L ('CHROME hwnd=' + $hC + ' pid=' + $chromePid + ' rect=' + [QN2]::RectOf($hC))
    RunRows $hC 'Chrome'
    [void][QN2]::RestoreFg($initFg)
    $overall = 'RAN'
}
catch {
    $overall = 'ABORTED: ' + $_.Exception.Message
    L ('ABORT ' + $_.Exception.Message)
}
finally {
    L 'TEARDOWN begin'
    if ($chromePid -ne 0) { Kill-Tree $chromePid 'QA host E2E v2 teardown (Chrome scratch profile, launched by QA)' 'chrome-ud' }
    if ($codePid -ne 0) { Kill-Tree $codePid 'QA host E2E v2 teardown (VS Code scratch profile, launched by QA)' 'vscode-ud' }
    if ($notePid -ne 0) {
        $pn = Get-Process -Id $notePid -ErrorAction SilentlyContinue
        if ($pn -and $pn.ProcessName -ieq 'notepad') { KillLog 'notepad' $notePid 'QA host E2E v2 teardown (scratch file only, launched by QA)'; Stop-Process -Id $notePid -Force -ErrorAction SilentlyContinue }
    }
    if ($vkPid -ne 0) {
        $pv = Get-Process -Id $vkPid -ErrorAction SilentlyContinue
        if ($pv -and $pv.Path -ieq $exe) { Kill-Tree $vkPid ('QA host E2E v2 teardown: owned rebuilt target build path=' + $exe + ' (launched by QA)') '' }
    }
    Start-Sleep -Seconds 2
    $left = @(Get-Process | Where-Object { $_.Path -and ($_.Path -ieq $exe) })
    L ('TEARDOWN candidate processes left=' + $left.Count)
    [QN2]::StopWatcher()

    # Caps Lock back to its starting state, and verify
    [QN2]::SetCapsState($capsInit)
    L ('CAPSLOCK restored to ' + ($(if ($capsInit) { 'on' } else { 'off' })) + ' now=' + ($(if ([QN2]::CapsOn()) { 'on' } else { 'off' })) + ' identical=' + ([QN2]::CapsOn() -eq $capsInit))

    # Cursor back to starting position
    [System.Windows.Forms.Cursor]::Position = New-Object System.Drawing.Point($cur0.X, $cur0.Y)
    L ('CURSOR restored to ' + $cur0.X + ',' + $cur0.Y)

    # config byte-identical
    if (Test-Path -LiteralPath $cfgBak) {
        Copy-Item -LiteralPath $cfgBak -Destination $cfg -Force
        $afterHash = (Get-FileHash -LiteralPath $cfg -Algorithm SHA256).Hash
        L ('CONFIG restored hash=' + $afterHash + ' expected=' + $origHash + ' identical=' + ($afterHash -eq $origHash))
    }

    # Run key: restore byte-identical if the candidate rewrote it
    $nowRun = (Get-ItemProperty -Path $runKey -Name VietKB -ErrorAction SilentlyContinue).VietKB
    L ('RUNKEY VietKB after=' + $nowRun + ' unchanged=' + ($nowRun -ceq $origRun))
    if (($nowRun -cne $origRun) -and ($null -ne $origRun)) { Set-ItemProperty -Path $runKey -Name VietKB -Value $origRun -Type String; L 'RUNKEY VietKB restored to original value' }
    $finalRun = (Get-ItemProperty -Path $runKey -Name VietKB -ErrorAction SilentlyContinue).VietKB
    L ('RUNKEY VietKB final byte-identical=' + ($finalRun -ceq $origRun) + ' value=' + $finalRun)

    # clipboard
    if ($clipHadText) { Set-Clipboard -Value $clipText } else { [System.Windows.Forms.Clipboard]::Clear() }
    L ('CLIPBOARD restored (had-text=' + $clipHadText + ')')
    [void][QN2]::RestoreFg($initFg)
    L ('FOREGROUND restored to ' + $initFg + ' now=' + [QN2]::Fg())

    $R | Export-Csv -LiteralPath $resPath -NoTypeInformation -Delimiter "`t" -Encoding UTF8
    L ('OVERALL ' + $overall)
    L 'END'
    Set-Content -LiteralPath $donePath -Value ($overall + ' ' + (Get-Date).ToUniversalTime().ToString('o')) -Encoding UTF8
}
