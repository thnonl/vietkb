# QA host E2E for VietKB 0.2.4 candidate (owner go-ahead 2026-10-30). Windows PowerShell 5.1 only:
#   powershell.exe -NoProfile -ExecutionPolicy Bypass -File E:\vietkb\docs\evkey-clone-work\qa\host-e2e\host-e2e.ps1
$ErrorActionPreference = 'Continue'
$root    = 'E:\vietkb\docs\evkey-clone-work\qa\host-e2e'
$shots   = Join-Path $root 'shots'
$scratch = 'C:\Users\namth\AppData\Local\Temp\claude\E--vietkb\9a85ac6a-ade5-453b-9010-44a303edbc97\scratchpad\host-e2e'
$exe     = 'E:\vietkb\target\release\vietkb.exe'
$cfg     = Join-Path $env:APPDATA 'VietKB\config.toml'
$runKey  = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run'
$logPath = Join-Path $root 'run-log.txt'
$logMd   = 'E:\vietkb\docs\evkey-clone-work\log.md'
$resPath = Join-Path $root 'results.tsv'
New-Item -ItemType Directory -Force -Path $shots, $scratch | Out-Null
Add-Type -AssemblyName System.Drawing, System.Windows.Forms, UIAutomationClient, UIAutomationTypes
Add-Type -Path (Join-Path $root 'QN.cs') -ReferencedAssemblies System.Drawing, System.Windows.Forms

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
function Rec([string]$id, [string]$app, [string]$keys, [string]$exp, [string]$act, [string]$verdict, [string]$note = '') {
    $o = [pscustomobject]@{ id = $id; app = $app; keys = $keys; expected = $exp; actual = $act; verdict = $verdict; note = $note }
    [void]$R.Add($o)
    L ('RESULT ' + $id + ' [' + $app + '] ' + $verdict + ' keys=[' + $keys + '] expected=[' + $exp + '] actual=[' + $act + '] ' + $note)
}
function Norm([string]$s) {
    if ($null -eq $s) { return '' }
    $t = $s.Normalize([Text.NormalizationForm]::FormC).Replace("`r`n", "`n").Replace("`r", "`n")
    return $t.TrimEnd("`n")
}
function Cmp([string]$id, [string]$app, [string]$keys, [string]$exp, [string]$act) {
    $e = Norm $exp; $a = Norm $act
    if ($e -ceq $a) { Rec $id $app $keys $e $a 'PASS' } else { Rec $id $app $keys $e $a 'FAIL' }
}

# One typing batch: focus guard + foreground assert + restore previous foreground afterwards.
function Batch([string]$name, [IntPtr]$h, [string]$spec, [bool]$restore = $true, [int]$delay = 35) {
    $prev = [QN]::Fg()
    $r = [QN]::Run($name, $h, $spec, $delay)
    L ('BATCH ' + $name + ' prevFg=' + $prev + ' -> ' + $r)
    if ($r -ne 'OK') { throw ('RUN-ABORT ' + $r) }
    if ($restore) { [void][QN]::RestoreFg($prev) }
}
function Click1([IntPtr]$h, [int]$x, [int]$y, [string]$name) {
    $prev = [QN]::Fg()
    $r = [QN]::Click($h, $x, $y, $name)
    L ('CLICK ' + $name + ' at ' + $x + ',' + $y + ' -> ' + $r)
    if ($r -ne 'OK') { throw ('RUN-ABORT ' + $r) }
}

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

function Phrases {
    return @(
        @{ id = 'A1'; keys = 'dduwowcj';                 exp = 'được' },
        @{ id = 'A2'; keys = 'tieesng vieetj';           exp = 'tiếng việt' },
        @{ id = 'A3'; keys = 'nghieeng';                 exp = 'nghiêng' },
        @{ id = 'A4'; keys = 'Tieesng Vieetj raast ddepj.'; exp = 'Tiếng Việt rất đẹp.' },
        @{ id = 'A5'; keys = ('dduwowcj ' * 10);         exp = ('được ' * 10) },
        @{ id = 'B1'; keys = 'VIEETJ';                   exp = 'VIỆT' },
        @{ id = 'B2'; keys = 'Vieejt';                   exp = 'Việt' },
        @{ id = 'B3'; keys = 'AAS DD';                   exp = 'Ấ Đ' }
    )
}

function TypePhrases([IntPtr]$h, [string]$app) {
    foreach ($p in (Phrases)) {
        Batch ('type-' + $p.id + '-' + $app) $h ($p.keys + '{ESC}{ENTER}') $true 35
    }
}

function CheckPhrases([string]$app, [string]$actual) {
    $lines = (Norm $actual).Split("`n")
    $items = Phrases
    for ($k = 0; $k -lt $items.Count; $k++) {
        $it = $items[$k]
        $act = if ($k -lt $lines.Count) { $lines[$k] } else { '<missing line>' }
        Cmp $it.id $app $it.keys $it.exp $act
    }
    $a5 = if ($lines.Count -gt 4) { $lines[4] } else { '' }
    $toks = @($a5.Split(' ') | Where-Object { $_ -ne '' })
    $wrong = @($toks | Where-Object { $_ -cne 'được' }).Count + [Math]::Max(0, 10 - $toks.Count)
    $verdict = if ($wrong -eq 0) { 'PASS' } else { 'FAIL' }
    Rec 'A5x10' $app 'dduwowcj x10 (per-token count)' '10 of 10 correct' ((10 - $wrong).ToString() + ' of 10 correct') $verdict
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
        $w = [QN]::FindWin($sub, $pid0)
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
$initFg = [QN]::Fg()
$overall = 'UNSET'
$RunStart = (Get-Date).ToUniversalTime()
L ('START host E2E 0.2.4; initFg=' + $initFg + '; scratch=' + $scratch)
Remove-Item -LiteralPath (Join-Path $root 'run-done.marker') -ErrorAction SilentlyContinue

try {
    # ---- 0. pre-flight: owner processes
    # Only EVKey, Unikey, the installed VietKB (or a stray vietkb.exe) is a conflict. Dev cargo/rustc/vkb_windows test binaries are ignored.
    $owner = @(Get-Process | Where-Object { $_.ProcessName -match 'EVKey|UniKey|^vietkb$' -or ($_.Path -and $_.Path -like "$env:LOCALAPPDATA\VietKB\*") })
    foreach ($o in $owner) { L ('PREFLIGHT found ' + $o.ProcessName + ' pid=' + $o.Id + ' path=' + $o.Path) }
    if ($owner.Count -gt 0) { throw 'PREFLIGHT-HOLD owner IME or VietKB running' }
    L ('PREFLIGHT ok idleMs=' + [QN]::IdleMs())
    # Owner approval 2026-10-30: wait for >=60 s GetLastInputInfo idle, polling up to 5 min, before anything is armed or changed.
    $idleOk = $false
    for ($t = 0; $t -lt 150; $t++) {
        if ([QN]::IdleMs() -ge 60000) { $idleOk = $true; break }
        Start-Sleep -Seconds 2
    }
    if (-not $idleOk) { throw 'HOLD(idle): owner input kept idle below 60 s for 5 min; nothing was changed' }
    L ('IDLE ok idleMs=' + [QN]::IdleMs())
    [QN]::StartWatcher()
    [QN]::MarkInjected()

    # ---- 1. backup config + Run key + clipboard
    Copy-Item -LiteralPath $cfg -Destination $cfgBak -Force
    $origHash = (Get-FileHash -LiteralPath $cfg -Algorithm SHA256).Hash
    $bakHash = (Get-FileHash -LiteralPath $cfgBak -Algorithm SHA256).Hash
    L ('CONFIG backup hash=' + $bakHash + ' orig=' + $origHash + ' equal=' + ($origHash -eq $bakHash))
    $origRun = (Get-ItemProperty -Path $runKey -Name VietKB -ErrorAction SilentlyContinue).VietKB
    L ('RUNKEY VietKB before=' + $origRun)
    $clipHadText = [System.Windows.Forms.Clipboard]::ContainsText()
    if ($clipHadText) { $clipText = [System.Windows.Forms.Clipboard]::GetText() }
    L ('CLIPBOARD had-text=' + $clipHadText + ' formats=' + (([System.Windows.Forms.Clipboard]::GetDataObject()).GetFormats() -join ','))

    # ---- 2. set method telex (only change; restored byte-identical afterwards)
    $txt = [IO.File]::ReadAllText($cfg)
    $cnt = ([regex]::Matches($txt, 'method = "vni"')).Count
    if ($cnt -ne 1) { throw ('CONFIG unexpected method count ' + $cnt) }
    $new = $txt.Replace('method = "vni"', 'method = "telex"')
    [IO.File]::WriteAllText($cfg, $new, (New-Object Text.UTF8Encoding($false)))
    L 'CONFIG set [input] method = "telex" for this run (restored from backup at teardown)'

    # re-check owner processes immediately before launching the candidate
    $owner2 = @(Get-Process | Where-Object { $_.ProcessName -match 'EVKey|UniKey|^vietkb$' -or ($_.Path -and $_.Path -like "$env:LOCALAPPDATA\VietKB\*") })
    if ($owner2.Count -gt 0) { throw 'HOLD(owner): owner IME or VietKB appeared before launch' }

    # ---- 3. launch the candidate build (only this process is ever stopped by QA)
    $vk = Start-Process -FilePath $exe -PassThru
    $vkPid = $vk.Id
    Start-Sleep -Seconds 5
    $vkp = Get-Process -Id $vkPid -ErrorAction SilentlyContinue
    if (-not $vkp) { throw 'BLOCKED(redirected-or-exited): launched candidate exited' }
    L ('LAUNCH vietkb pid=' + $vkPid + ' path=' + $vkp.Path)
    if ($vkp.Path -ine $exe) { throw 'BLOCKED(redirected): process path is not the target build' }
    $inst = @(Get-Process | Where-Object { $_.Path -and $_.Path -like "$env:LOCALAPPDATA\VietKB\*" })
    if ($inst.Count -gt 0) { throw 'BLOCKED(redirected): installed VietKB appeared' }

    # ---- 4. Notepad
    $idN = Get-Random -Maximum 99999
    $fileN = Join-Path $scratch ('qa-notepad-' + $idN + '.txt')
    Set-Content -LiteralPath $fileN -Value '' -Encoding UTF8
    $np = Start-Process -FilePath 'notepad.exe' -ArgumentList ('"' + $fileN + '"') -PassThru
    $hN = WaitWin ('qa-notepad-' + $idN) 0 15
    if ($hN -eq [IntPtr]::Zero) { throw 'SETUP Notepad window not found' }
    $notePid = [int][QN]::PidOf($hN)
    [QN]::Place($hN, 2660, 100, 960, 700)
    if (-not [QN]::Force($hN)) { throw 'FOCUS Notepad window not foreground after Force' }
    L ('NOTEPAD hwnd=' + $hN + ' pid=' + $notePid + ' rect=' + [QN]::RectOf($hN))
    Batch 'clear-notepad' $hN '{CTRL+A}{DEL}'
    TypePhrases $hN 'Notepad'
    $actN = ReadField $hN 'Notepad'
    CheckPhrases 'Notepad' $actN
    [QN]::Capture(2560, 0, 1920, 1080, (Join-Path $shots 'notepad-A-B.png'))

    # C: click ends the word (D-E2E-02)
    Batch 'clear-C' $hN '{CTRL+A}{DEL}'
    Batch 'C-type' $hN 'xyz{ENTER}vie' $false
    $rc = EditorRect $hN
    if ($rc) { $cx = [int]($rc.Left + 300); $cy = [int]($rc.Top + 12) } else { $cx = 2660 + 300; $cy = 100 + 90 }
    Click1 $hN $cx $cy 'C-click-line1-end'
    Batch 'C-et' $hN 'et' $true
    [QN]::Capture(2560, 0, 1920, 1080, (Join-Path $shots 'notepad-C.png'))
    $actC = ReadField $hN 'Notepad'
    Cmp 'C1' 'Notepad' 'xyz{ENTER}vie | click end of line 1 | et' ("xyzet`nvie") $actC

    # E: restore word Ctrl+Shift+Z
    Batch 'clear-E' $hN '{CTRL+A}{DEL}'
    Batch 'E-type-restore' $hN 'vieejt{CTRL+SHIFT+Z}'
    $actE = ReadField $hN 'Notepad'
    Cmp 'E1' 'Notepad' 'vieejt then Ctrl+Shift+Z' 'vieejt' $actE

    # H: settings window on DISPLAY1 + tray screenshot
    Batch 'H-open-settings' $hN '{CTRL+ALT+S}' $true
    Start-Sleep -Seconds 3
    $hS = WaitWin '' $vkPid 10
    if ($hS -eq [IntPtr]::Zero) {
        Rec 'H1' 'VietKB' 'Ctrl+Alt+S' 'settings window visible' '<not found>' 'FAIL' 'no visible window of the candidate pid after Ctrl+Alt+S'
    } else {
        Rec 'H1' 'VietKB' 'Ctrl+Alt+S' 'settings window visible' ('visible hwnd=' + $hS + ' rect=' + [QN]::RectOf($hS)) 'PASS'
        $r0 = [QN]::RectArr($hS)
        if ($r0[0] -lt 2560) { [QN]::Place($hS, 2660 + 960, 100, 900, 640) }
        Start-Sleep -Seconds 2
        $r1 = [QN]::RectArr($hS)
        $onD1 = ($r1[0] -ge 2560 -and $r1[1] -ge 0 -and $r1[2] -le 4480 -and $r1[3] -le 1080)
        Rec 'H2' 'VietKB' 'settings window placement' 'settings on DISPLAY1' ('rect=' + ($r1 -join ',')) $(if ($onD1) { 'PASS' } else { 'FAIL' })
        [QN]::Capture(2560, 0, 1920, 1080, (Join-Path $shots 'H-settings-display1.png'))
        [QN]::CloseWin($hS)
    }
    [QN]::Capture(1600, 1100, 448, 52, (Join-Path $shots 'H-tray-primary.png'))
    Rec 'H3' 'Shell' 'tray area' 'screenshot of tray icon area (primary taskbar)' 'shots\H-tray-primary.png' 'PASS' 'owner to view; icon presence not auto-checked'
    [void][QN]::RestoreFg($initFg)

    # ---- 5. VS Code (scratch instance, own user-data-dir)
    $idV = Get-Random -Maximum 99999
    $fileV = Join-Path $scratch ('qa-vscode-' + $idV + '.txt')
    Set-Content -LiteralPath $fileV -Value '' -Encoding UTF8
    $vsud = Join-Path $scratch 'vscode-ud'
    $codeExe = 'C:\Users\namth\AppData\Local\Programs\Microsoft VS Code\Code.exe'
    $vs = Start-Process -FilePath $codeExe -ArgumentList @('--user-data-dir', $vsud, '--new-window', '--disable-workspace-trust', '--skip-release-notes', '--skip-welcome', $fileV) -PassThru
    $codePid = $vs.Id
    $hV = WaitWin ('qa-vscode-' + $idV) 0 40
    if ($hV -eq [IntPtr]::Zero) { throw 'SETUP VS Code window not found' }
    [QN]::Place($hV, 2660, 100, 960, 700)
    Start-Sleep -Seconds 4
    if (-not [QN]::Force($hV)) { throw 'FOCUS VS Code window not foreground after Force' }
    L ('VSCODE hwnd=' + $hV + ' pid=' + $codePid + ' rect=' + [QN]::RectOf($hV))
    Batch 'clear-vscode' $hV '{CTRL+A}{DEL}'
    TypePhrases $hV 'VS Code'
    # D: F2 then F1 (VietKB disable/enable hotkeys in the live config), then baatj
    Batch 'D-F2-F1-baatj' $hV '{F2}{F1}baatj'
    [QN]::Capture(2560, 0, 1920, 1080, (Join-Path $shots 'vscode-D-after-F1.png'))
    $actV = ReadField $hV 'VS Code'
    $linesV = (Norm $actV).Split("`n")
    CheckPhrases 'VS Code' $actV
    $dLine = if ($linesV.Count -ge 9) { $linesV[8] } else { '<missing>' }
    Cmp 'D1' 'VS Code' '{F2}{F1}baatj' 'bật' $dLine
    [void][QN]::RestoreFg($initFg)

    # ---- 6. Chrome (scratch profile, local textarea page)
    $htmlPath = Join-Path $scratch 'qa-textarea.html'
    Set-Content -LiteralPath $htmlPath -Encoding UTF8 -Value '<!doctype html><html><head><meta charset="utf-8"><title>QA textarea</title></head><body style="margin:0"><textarea id="t" autofocus style="width:100%;height:100vh;font-size:20px"></textarea></body></html>'
    $cud = Join-Path $scratch 'chrome-ud'
    $chromeExe = 'C:\Program Files\Google\Chrome\Application\chrome.exe'
    $ch = Start-Process -FilePath $chromeExe -ArgumentList @(('--user-data-dir=' + $cud), '--new-window', '--no-first-run', '--no-default-browser-check', '--window-position=2660,100', '--window-size=960,700', ('file:///' + $htmlPath.Replace('\', '/'))) -PassThru
    $chromePid = $ch.Id
    $hC = WaitWin 'QA textarea' ([uint32]$chromePid) 30
    if ($hC -eq [IntPtr]::Zero) { $hC = WaitWin 'QA textarea' 0 5 }
    if ($hC -eq [IntPtr]::Zero) { throw 'SETUP Chrome window not found' }
    [QN]::Place($hC, 2660, 100, 960, 700)
    Start-Sleep -Seconds 2
    if (-not [QN]::Force($hC)) { throw 'FOCUS Chrome window not foreground after Force' }
    L ('CHROME hwnd=' + $hC + ' pid=' + $chromePid + ' rect=' + [QN]::RectOf($hC))
    Batch 'clear-chrome' $hC '{CTRL+A}{DEL}'
    TypePhrases $hC 'Chrome'
    $actCh = ReadField $hC 'Chrome'
    CheckPhrases 'Chrome' $actCh
    [QN]::Capture(2560, 0, 1920, 1080, (Join-Path $shots 'chrome-end.png'))
    [void][QN]::RestoreFg($initFg)
    $overall = 'RAN'
}
catch {
    $overall = 'ABORTED: ' + $_.Exception.Message
    L ('ABORT ' + $_.Exception.Message)
}
finally {
    # ---- teardown: only processes this run launched
    L 'TEARDOWN begin'
    if ($chromePid -ne 0) { Kill-Tree $chromePid 'QA host E2E teardown (Chrome scratch profile, launched by QA)' 'chrome-ud' }
    if ($codePid -ne 0) { Kill-Tree $codePid 'QA host E2E teardown (VS Code scratch profile, launched by QA)' 'vscode-ud' }
    if ($notePid -ne 0) {
        $pn = Get-Process -Id $notePid -ErrorAction SilentlyContinue
        if ($pn -and $pn.ProcessName -ieq 'notepad') { KillLog 'notepad' $notePid 'QA host E2E teardown (scratch file only, launched by QA)'; Stop-Process -Id $notePid -Force -ErrorAction SilentlyContinue }
    }
    if ($vkPid -ne 0) {
        $pv = Get-Process -Id $vkPid -ErrorAction SilentlyContinue
        if ($pv -and $pv.Path -ieq $exe) { Kill-Tree $vkPid ('QA host E2E teardown: owned target build path=' + $exe + ' (launched by QA)') '' }
    }
    Start-Sleep -Seconds 2
    $left = @(Get-Process | Where-Object { $_.Path -and ($_.Path -ieq $exe) })
    L ('TEARDOWN candidate processes left=' + $left.Count)
    [QN]::StopWatcher()

    # restore config byte-identical
    Copy-Item -LiteralPath $cfgBak -Destination $cfg -Force
    $afterHash = (Get-FileHash -LiteralPath $cfg -Algorithm SHA256).Hash
    L ('CONFIG restored hash=' + $afterHash + ' expected=' + $origHash + ' identical=' + ($afterHash -eq $origHash))

    # Run key
    $nowRun = (Get-ItemProperty -Path $runKey -Name VietKB -ErrorAction SilentlyContinue).VietKB
    L ('RUNKEY VietKB after=' + $nowRun + ' unchanged=' + ($nowRun -ceq $origRun))
    if (($nowRun -cne $origRun) -and ($null -ne $origRun)) { Set-ItemProperty -Path $runKey -Name VietKB -Value $origRun -Type String; L 'RUNKEY VietKB restored to original value' }
    $finalRun = (Get-ItemProperty -Path $runKey -Name VietKB -ErrorAction SilentlyContinue).VietKB
    L ('RUNKEY VietKB final byte-identical=' + ($finalRun -ceq $origRun) + ' value=' + $finalRun)

    # clipboard
    if ($clipHadText) { Set-Clipboard -Value $clipText } else { [System.Windows.Forms.Clipboard]::Clear() }
    L ('CLIPBOARD restored (had-text=' + $clipHadText + ')')
    [void][QN]::RestoreFg($initFg)
    L ('FOREGROUND restored to ' + $initFg + ' now=' + [QN]::Fg())

    $R | Export-Csv -LiteralPath $resPath -NoTypeInformation -Delimiter "`t" -Encoding UTF8
    L ('OVERALL ' + $overall)
    L 'END'
    Set-Content -LiteralPath (Join-Path $root 'run-done.marker') -Value ($overall + ' ' + (Get-Date).ToUniversalTime().ToString('o')) -Encoding UTF8
}
