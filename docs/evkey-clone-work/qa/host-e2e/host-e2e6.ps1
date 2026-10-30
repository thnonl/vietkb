# QA host E2E v6: VS Code only, suggestions off, rebuilt candidate 0D39D42F (owner-approved 2026-10-30, final run).
# Windows PowerShell 5.1 only:
#   powershell.exe -NoProfile -ExecutionPolicy Bypass -File E:\vietkb\docs\evkey-clone-work\qa\host-e2e\host-e2e6.ps1
# Physical input: ONLY our WH_KEYBOARD_LL / WH_MOUSE_LL watchers (injected flags).
# GetLastInputInfo: pre-run 60 s idle gate and the 60 s idle wait before the click (no injection has happened yet at that point).
$ErrorActionPreference = 'Continue'
$root    = 'E:\vietkb\docs\evkey-clone-work\qa\host-e2e'
$shots   = Join-Path $root 'shots6'
$scratch = 'C:\Users\namth\AppData\Local\Temp\claude\E--vietkb\9a85ac6a-ade5-453b-9010-44a303edbc97\scratchpad\host-e2e6'
$exe     = 'E:\vietkb\target\release\vietkb.exe'
$setup   = 'E:\vietkb\target\release\bundle\nsis\VietKB_0.2.4_x64-setup.exe'
$exeSha  = '0D39D42F5F66CA4CE40D22068FA31DE940E32D811DD5F393C79FD775FADFEB92'
$setupSha = 'CD45BD84E26DC3A46A4DF51CD96465A31932DEBE310741797AAE190D3812DCBC'
$cfg     = Join-Path $env:APPDATA 'VietKB\config.toml'
$runKey  = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run'
$logPath = Join-Path $root 'run6-log.txt'
$logMd   = 'E:\vietkb\docs\evkey-clone-work\log.md'
$resPath = Join-Path $root 'results6.tsv'
$donePath = Join-Path $root 'run6-done.marker'
$codeExe = 'C:\Users\namth\AppData\Local\Programs\Microsoft VS Code\Code.exe'
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
function Rec([string]$id, [string]$keys, [string]$exp, [string]$act, [string]$verdict, [string]$note = '') {
    [void]$R.Add([pscustomobject]@{ app = 'VS Code'; id = $id; keys = $keys; expected = $exp; actual = $act; verdict = $verdict; note = $note })
    L ('RESULT [VS Code] ' + $id + ' ' + $verdict + ' keys=[' + $keys + '] expected=[' + $exp + '] actual=[' + $act + '] ' + $note)
}
function Norm([string]$s) {
    if ($null -eq $s) { return '' }
    $t = $s.Normalize([Text.NormalizationForm]::FormC).Replace("`r`n", "`n").Replace("`r", "`n")
    return $t.TrimEnd("`n")
}

function WinTitle([IntPtr]$h) {
    try { $p = Get-Process -Id ([QN2]::PidOf($h)) -ErrorAction Stop; return [string]$p.MainWindowTitle } catch { return '<no-title>' }
}

function Settle([IntPtr]$h) {
    for ($attempt = 1; $attempt -le 3; $attempt++) {
        $last = $null; $since = Get-Date; $deadline = (Get-Date).AddSeconds(40)
        while ((Get-Date) -lt $deadline) {
            $t = WinTitle $h
            if ($t -ne $last) { $last = $t; $since = Get-Date }
            elseif (((Get-Date) - $since).TotalSeconds -ge 3) { break }
            Start-Sleep -Milliseconds 500
        }
        L ('SETTLE attempt=' + $attempt + ' title=[' + $last + ']')
        if ([QN2]::Fg() -eq $h) { L 'SETTLE foreground is our window'; return }
        $fg = [QN2]::Fg()
        $fgPid = [int][QN2]::PidOf($fg)
        $fgProc = Get-Process -Id $fgPid -ErrorAction SilentlyContinue
        $fgName = if ($fgProc) { $fgProc.ProcessName } else { '<gone>' }
        L ('FOCUS-TAKEN by process=' + $fgName + ' pid=' + $fgPid + ' title=[' + $(if ($fgProc) { [string]$fgProc.MainWindowTitle } else { '' }) + ']')
        [void][QN2]::Force($h)
        if ([QN2]::Fg() -eq $h) { L 'SETTLE refocused'; return }
    }
    throw 'FOCUS-ABORT foreground not ours after 3 attempts'
}

function Batch([string]$name, [IntPtr]$h, [string]$spec, [bool]$restore = $true, [int]$delay = 35) {
    $prev = [QN2]::Fg()
    for ($try = 1; $try -le 3; $try++) {
        $r = [QN2]::Run($name, $h, $spec, $delay)
        L ('BATCH ' + $name + ' try=' + $try + ' prevFg=' + $prev + ' -> ' + $r)
        if ($r -eq 'OK') { break }
        if ($r -like 'ABORT:test window not foreground*' -and $try -lt 3) { L ('FOCUS retry ' + $try); Settle $h; continue }
        throw ('RUN-ABORT ' + $r)
    }
    if ($restore) { [void][QN2]::RestoreFg($prev) }
}

# ---- sentinel confirmation, no clipboard: (a) UIA TextPattern of the editor document; (b) pixel diff of the editor area.
function UiaDocText([IntPtr]$h) {
    try {
        $root2 = [System.Windows.Automation.AutomationElement]::FromHandle($h)
        $cond = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ControlTypeProperty, [System.Windows.Automation.ControlType]::Document)
        $doc = $root2.FindFirst([System.Windows.Automation.TreeScope]::Descendants, $cond)
        if ($null -eq $doc) { return $null }
        $tp = $null
        if ($doc.TryGetCurrentPattern([System.Windows.Automation.TextPattern]::Pattern, [ref]$tp)) {
            return [string]([System.Windows.Automation.TextPattern]$tp).DocumentRange.GetText(-1)
        }
        return $null
    } catch { return $null }
}
function Grab([int]$x, [int]$y, [int]$w, [int]$hh) {
    $b = New-Object System.Drawing.Bitmap $w, $hh
    $g = [System.Drawing.Graphics]::FromImage($b)
    $g.CopyFromScreen($x, $y, 0, 0, (New-Object System.Drawing.Size $w, $hh))
    $g.Dispose()
    return $b
}
function DiffCount($a, $b) {
    $n = 0
    for ($i = 0; $i -lt $a.Width; $i += 2) {
        for ($j = 0; $j -lt $a.Height; $j += 2) {
            if ($a.GetPixel($i, $j).ToArgb() -ne $b.GetPixel($i, $j).ToArgb()) { $n++ }
        }
    }
    return $n
}

# ---- verdict read: one Ctrl+A, Ctrl+C inside our window, poll the clipboard (max 4 s) for a value different from the mark.
function ReadVerdict([IntPtr]$h) {
    $mark = 'QAMARK' + [guid]::NewGuid().ToString('N')
    Set-Clipboard -Value $mark
    Batch 'verdict-select-copy' $h '{ESC}{CTRL+A}{CTRL+C}{CTRL+END}' $true 35
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
        @{ id = 'A1'; spec = 'dduwowcj';                     exp = 'được';         chk = $false },
        @{ id = 'A2'; spec = 'tieesng vieetj';               exp = 'tiếng việt';   chk = $false },
        @{ id = 'B1'; spec = 'VIEETJ';                       exp = 'VIỆT';         chk = $true },
        @{ id = 'B2'; spec = 'Vieejt';                       exp = 'Việt';         chk = $true },
        @{ id = 'B4'; spec = '{SHIFTDOWN}VIEETJ{SHIFTUP}';   exp = 'VIỆT';         chk = $true },
        @{ id = 'E';  spec = 'vieejt{CTRL+SHIFT+Z}';         exp = 'vieejt';       chk = $false },
        @{ id = 'D';  spec = '{F2}{F1}baatj';                exp = 'bật';          chk = $false }
    )
}

# Row matching by ORDER on the whole document: exact line searched forward from the last match; otherwise "present but not
# on its own line" (merged or shifted) or "missing". No fixed indices.
function MatchRows([string]$doc, $rowsList) {
    $lines = (Norm $doc).Split("`n")
    $p = 0
    foreach ($row in $rowsList) {
        $expCmp = (Norm $row.exp).TrimEnd(' ')
        $found = -1
        for ($i = $p; $i -lt $lines.Count; $i++) { if ($lines[$i].TrimEnd(' ') -ceq $expCmp) { $found = $i; break } }
        if ($found -ge 0) {
            Rec $row.id $row.spec $row.exp $lines[$found] 'PASS' ('own line ' + ($found + 1))
            $p = $found + 1
        } else {
            $where = -1
            for ($i = $p; $i -lt $lines.Count; $i++) { if ($lines[$i].Contains($expCmp)) { $where = $i; break } }
            if ($where -ge 0) {
                Rec $row.id $row.spec $row.exp $lines[$where] 'FAIL' ('present on line ' + ($where + 1) + ' but not as its own line (merged or shifted)')
                $p = $where + 1
            } else {
                Rec $row.id $row.spec $row.exp '<not in document>' 'FAIL' ('missing after line ' + $p)
            }
        }
    }
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
$vkPid = 0; $codePid = 0
$cfgBak = Join-Path $scratch 'config.backup.toml'
$clipText = $null; $clipHadText = $false
$origRun = $null
$initFg = [QN2]::Fg()
$capsInit = [QN2]::CapsOn()
$cur0 = [System.Windows.Forms.Cursor]::Position
$ownerCodeBefore = @(Get-Process Code -ErrorAction SilentlyContinue).Count
$overall = 'UNSET'
L ('START host E2E v6 (VS Code only, suggestions off, final) initFg=' + $initFg + ' cursor=' + $cur0.X + ',' + $cur0.Y + ' owner-Code-before=' + $ownerCodeBefore)

try {
    # ---- 0. hashes
    $hE = (Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash
    $hS = (Get-FileHash -LiteralPath $setup -Algorithm SHA256).Hash
    L ('HASH exe match=' + ($hE -eq $exeSha) + ' installer match=' + ($hS -eq $setupSha))
    if (($hE -ne $exeSha) -or ($hS -ne $setupSha)) { throw 'HASH-MISMATCH: not the approved rebuilt candidate' }

    # ---- 1. owner processes
    $owner = @(Get-Process | Where-Object { $_.ProcessName -match 'EVKey|UniKey|^vietkb$' -or ($_.Path -and $_.Path -like "$env:LOCALAPPDATA\VietKB\*") })
    if ($owner.Count -gt 0) { throw 'HOLD(owner): EVKey, Unikey or installed VietKB running' }
    L 'PREFLIGHT ok'

    # ---- 2. pre-run idle gate >= 60 s (up to 5 min)
    $idleOk = $false
    for ($t = 0; $t -lt 150; $t++) {
        if ([QN2]::IdleMs() -ge 60000) { $idleOk = $true; break }
        Start-Sleep -Seconds 2
    }
    if (-not $idleOk) { throw 'HOLD(idle): owner input kept idle below 60 s for 5 min; nothing was changed' }
    L ('IDLE ok idleMs=' + [QN2]::IdleMs())

    # ---- 3. watchers
    [QN2]::StartWatcher()
    [QN2]::MarkInjected()
    L 'WATCHERS armed (WH_KEYBOARD_LL 0x10, WH_MOUSE_LL 0x01)'

    # ---- 4. backups
    Copy-Item -LiteralPath $cfg -Destination $cfgBak -Force
    $origHash = (Get-FileHash -LiteralPath $cfg -Algorithm SHA256).Hash
    L ('CONFIG backup hash=' + $origHash)
    $origRun = (Get-ItemProperty -Path $runKey -Name VietKB -ErrorAction SilentlyContinue).VietKB
    L ('RUNKEY VietKB before=' + $origRun)
    $clipHadText = [System.Windows.Forms.Clipboard]::ContainsText()
    if ($clipHadText) { $clipText = [System.Windows.Forms.Clipboard]::GetText() }
    L ('CLIPBOARD had-text=' + $clipHadText)

    # ---- 5. telex for this run
    $txt = [IO.File]::ReadAllText($cfg)
    if (([regex]::Matches($txt, 'method = "vni"')).Count -ne 1) { throw 'CONFIG unexpected method line' }
    [IO.File]::WriteAllText($cfg, $txt.Replace('method = "vni"', 'method = "telex"'), (New-Object Text.UTF8Encoding($false)))
    L 'CONFIG set [input] method = "telex" for this run'

    # ---- 6. launch the candidate
    $owner2 = @(Get-Process | Where-Object { $_.ProcessName -match 'EVKey|UniKey|^vietkb$' -or ($_.Path -and $_.Path -like "$env:LOCALAPPDATA\VietKB\*") })
    if ($owner2.Count -gt 0) { throw 'HOLD(owner): owner IME or VietKB appeared before launch' }
    $vk = Start-Process -FilePath $exe -PassThru
    $vkPid = $vk.Id
    Start-Sleep -Seconds 5
    $vkp = Get-Process -Id $vkPid -ErrorAction SilentlyContinue
    if (-not $vkp) { throw 'BLOCKED(redirected-or-exited): candidate exited' }
    if ($vkp.Path -ine $exe) { throw 'BLOCKED(redirected): path is not the target build' }
    L ('LAUNCH vietkb pid=' + $vkPid + ' path=' + $vkp.Path)

    # ---- 7. VS Code isolated instance (empty single-line file; no Set-Content newline)
    $idV = Get-Random -Maximum 99999
    $fileV = Join-Path $scratch ('qa-vscode-' + $idV + '.txt')
    [IO.File]::WriteAllText($fileV, '')
    $vsud = Join-Path $scratch 'vscode-ud'
    $vsext = Join-Path $scratch 'vscode-ext'
    $vs = Start-Process -FilePath $codeExe -ArgumentList @('--new-window', '--disable-extensions', '--user-data-dir', $vsud, '--extensions-dir', $vsext, $fileV) -PassThru
    $codePid = $vs.Id
    L ('VSCODE launched pid=' + $codePid)
    $hV = WaitWin ('qa-vscode-' + $idV) ([uint32]$codePid) 60
    if ($hV -eq [IntPtr]::Zero) { $hV = WaitWin ('qa-vscode-' + $idV) 0 10 }
    if ($hV -eq [IntPtr]::Zero) { throw 'SETUP VS Code window not found' }
    [QN2]::Place($hV, 2660, 100, 960, 700)
    $rr = [QN2]::RectArr($hV)
    $onD1 = ($rr[0] -ge 2560 -and $rr[1] -ge 0 -and $rr[2] -le 4480 -and $rr[3] -le 1080)
    L ('VSCODE hwnd=' + $hV + ' rect=' + ($rr -join ',') + ' onDISPLAY1=' + $onD1)
    if (-not $onD1) { throw 'PLACEMENT: VS Code window not fully on DISPLAY1' }
    Settle $hV
    Start-Sleep -Milliseconds 800

    # ---- 8. >= 60 s physical idle immediately before the click (no injection has happened yet in this run)
    $idle2 = $false
    for ($t = 0; $t -lt 150; $t++) {
        if ([QN2]::Abort) { throw ('ABORT before click: ' + [QN2]::AbortReason) }
        if ([QN2]::IdleMs() -ge 60000) { $idle2 = $true; break }
        Start-Sleep -Seconds 2
    }
    if (-not $idle2) { throw 'HOLD(idle): no 60 s idle before the click' }
    L ('IDLE before click ok idleMs=' + [QN2]::IdleMs())

    # ---- 9. click on line 1 of OUR editor, then sentinel x; confirm via UIA TextPattern or pixel diff (no clipboard)
    $rr = [QN2]::RectArr($hV)
    $w = $rr[2] - $rr[0]; $hh = $rr[3] - $rr[1]
    $cx = [int]($rr[0] + $w * 0.25)
    $cy = [int]($rr[1] + $hh * 0.18)
    $rc = [QN2]::Click($hV, $cx, $cy, 'VSCODE-click-line1')
    L ('CLICK line 1 at ' + $cx + ',' + $cy + ' -> ' + $rc)
    if ($rc -ne 'OK') { throw ('RUN-ABORT ' + $rc) }
    Start-Sleep -Seconds 1
    $regX = [int]($rr[0] + $w * 0.16); $regY = [int]($rr[1] + $hh * 0.12)
    $regW = [int]($w * 0.40); $regH = [int]($hh * 0.16)
    $before = Grab $regX $regY $regW $regH
    Batch 'sentinel-x' $hV 'x' $false 35
    $sentinelOk = $false; $how = ''
    $uia = UiaDocText $hV
    if ($uia -ne $null -and ($uia -cmatch 'x')) { $sentinelOk = $true; $how = 'UIA TextPattern' }
    if (-not $sentinelOk) {
        for ($t = 0; $t -lt 8 -and -not $sentinelOk; $t++) {
            Start-Sleep -Milliseconds 500
            $after = Grab $regX $regY $regW $regH
            $d = DiffCount $before $after
            if ($d -gt 15) { $sentinelOk = $true; $how = ('pixel diff ' + $d + ' px in editor area') }
            $after.Dispose()
        }
    }
    $before.Dispose()
    L ('SENTINEL confirmed=' + $sentinelOk + ' via=' + $how + ' uiaText=[' + $(if ($uia -ne $null) { ($uia -replace "`r|`n", '|') } else { '<null>' }) + ']')
    if (-not $sentinelOk) { L 'SENTINEL not confirmed within 4 s: recorded; continuing (foreground guard still holds)' }
    Rec 'focus-sentinel' 'x' 'x in our editor' $how $(if ($sentinelOk) { 'PASS' } else { 'NOT-CONFIRMED' }) 'UIA or pixel diff, no clipboard'
    Batch 'sentinel-delete' $hV '{BACKSPACE}' $true 35
    Start-Sleep -Milliseconds 300

    # ---- 10. rows: A1, A2, B1, B2, B4, E, then D (F2, F1, baatj on its own line). Shift checks after B rows.
    Batch 'clear-vscode' $hV '{CTRL+A}{DEL}' $true 35
    $spec = ''
    $rowsList = Rows
    foreach ($row in $rowsList) {
        if ($row.id -eq 'D') { continue }
        $spec += $row.spec
        if ($row.chk) { $spec += '{CHK:' + $row.id + '}' }
        $spec += '{ESC}{ENTER}'
    }
    [QN2]::ClearChecks()
    Batch 'rows-vscode' $hV $spec $true 35
    $chk = [string[]]([QN2]::Checks.ToArray())
    foreach ($c in $chk) {
        L ('SHIFTCHK ' + $c)
        $parts = $c.Split('|')
        $stuck = ($c -match 'DOWN')
        Rec ('SHIFT-' + $parts[0].Trim()) 'after row' 'no Shift stuck down' $parts[1].Trim() $(if ($stuck) { 'FAIL' } else { 'PASS' }) ($parts[2].Trim())
    }

    # D: F2 (disable), F1 (enable), then baatj. Command palette must not open.
    Batch 'D-F2-F1-baatj' $hV '{F2}{F1}baatj' $true 35
    Start-Sleep -Milliseconds 800
    [QN2]::Capture(2560, 0, 1920, 1080, (Join-Path $shots 'vscode-v6-D-after-F1.png'))

    # ---- 11. verdict read: one Ctrl+A, Ctrl+C inside our window; screenshot; clipboard read
    $docFinal = ReadVerdict $hV
    [QN2]::Capture(2560, 0, 1920, 1080, (Join-Path $shots 'vscode-v6-final.png'))
    L ('VERDICT doc=[' + ((Norm $docFinal) -replace "`n", '|') + ']')
    if ($docFinal -eq '<clipboard-no-change>') { Rec 'verdict-read' 'Ctrl+A Ctrl+C' 'clipboard text' $docFinal 'NOT-READ' 'clipboard not updated within 4 s; screenshot shots6\vscode-v6-final.png is the evidence' }
    else { MatchRows $docFinal $rowsList }
    [void][QN2]::RestoreFg($initFg)
    $overall = 'RAN'
}
catch {
    $overall = 'ABORTED: ' + $_.Exception.Message
    L ('ABORT ' + $_.Exception.Message)
}
finally {
    L 'TEARDOWN begin'
    if ($codePid -ne 0) { Kill-Tree $codePid 'QA host E2E v6 teardown (VS Code isolated profile, launched by QA)' 'vscode-ud' }
    if ($vkPid -ne 0) {
        $pv = Get-Process -Id $vkPid -ErrorAction SilentlyContinue
        if ($pv -and $pv.Path -ieq $exe) { Kill-Tree $vkPid ('QA host E2E v6 teardown: rebuilt target build path=' + $exe + ' (launched by QA)') '' }
    }
    Start-Sleep -Seconds 2
    [QN2]::StopWatcher()
    L ('TEARDOWN candidate processes left=' + @(Get-Process | Where-Object { $_.Path -and ($_.Path -ieq $exe) }).Count)

    foreach ($d in @('vscode-ud', 'vscode-ext')) {
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
    L ('OWNER VS Code processes before=' + $ownerCodeBefore + ' after=' + @(Get-Process Code -ErrorAction SilentlyContinue).Count)

    $R | Export-Csv -LiteralPath $resPath -NoTypeInformation -Delimiter "`t" -Encoding UTF8
    L ('OVERALL ' + $overall)
    L 'END'
    Set-Content -LiteralPath $donePath -Value ($overall + ' ' + (Get-Date).ToUniversalTime().ToString('o')) -Encoding UTF8
}
