#requires -version 5.1
<#
  run-host-e2e.ps1 - VietKB host typing/toggle gate, unattended, one pass.

  Why this exists: Windows Sandbox drops KEYEVENTF_UNICODE (96-sandbox-feasibility.md),
  so accented typing can only be graded on the host desktop. This script is the whole
  gate in one run: launch, type, read back, judge, tear down.

  Owner-time run (one command):
    powershell -NoProfile -ExecutionPolicy Bypass -File E:\vietkb\docs\evkey-clone-work\qa\e2e\run-host-e2e.ps1
    ... -Fast        # skip the 60 s soak
    ... -SelfCheck   # dry run: no app, no GUI; validates environment, corpus and helpers

  Judgement (91-round-plan-sound-cases.md "Correctness rule", owner 2026-10-30):
    PASS        the pinned composed Vietnamese appeared
    PASS(raw)   the literal raw keystrokes appeared instead (rule 4: also acceptable)
    QUALITY     a documented, owner-accepted loss (L1-L7 / raw-zone literal): the row is
                tagged with the loss and the app produced exactly the pinned text.
    READ-FAIL   the harness could not read the field (U+FFFC object replacement, empty
                after a retry at a longer settle, and the second reader also empty).
                Nothing is known about the product for that row: it is unmeasured, not
                broken, and is counted on its own line.
    MANUAL      a row this build cannot drive from a script; the owner does it by hand.
                The note says exactly what to do and how many seconds it takes.
    SKIP(unbound) the config has no chord for this group (hotkeys.<name> empty). Nothing
                was pressed and nothing is claimed: not a failure, not a pass.
    FAIL        anything else. Never ship on a FAIL.

  Verdict arithmetic: HOLD when FAIL > 0, else SHIP. QUALITY, MANUAL, SKIP(unbound) and
  READ-FAIL never block. A READ-FAIL still prints a CAVEAT line, because an unmeasured
  row is not proof.

  Method discipline: the corpus carries VNI rows and Telex rows. The script switches
  `input.method` through its supported surface (the config file, then an app restart),
  proves the switch with a typed probe plus a config read-back, runs that group, and
  restores the original method before the chord/toggle/soak blocks. config.toml is
  byte-compared against its backup at the very end.

  Hotkey discipline: no chord is written into this file. Every chord the run presses is
  parsed at run time out of the live config (hotkeys.toggle / restore_word /
  open_settings / enable / disable), with the product's own grammar
  (crates/vkb-config/src/hotkey.rs): case- and order-insensitive, modifier-only chords
  allowed, a bare key only for F1..F24. A binding that is empty or unparsable makes its
  whole group `SKIP(unbound)` instead of FAIL, and every parsed binding is printed in a
  `# CONFIG` block at the top of results-host.txt so the reader can check what was
  pressed against what the user has bound.

  Readers: UIA TextPattern first (ValuePattern is the built-in fallback), then after a
  longer settle, then the clipboard (Ctrl+A Ctrl+C, selection collapsed with End). A
  text of U+FFFC or an empty field aborts that reader.

  This file is ASCII-only on purpose: every Vietnamese string is built from codepoints,
  so a re-save without a BOM can never silently turn an expectation into mojibake.
  Assert-Encoding checks the file bytes and the built literals before anything runs.

  Guardrails (99-agent-guardrails.md, 00-charter.md section 6): the owner's installed build
  (%LOCALAPPDATA%\VietKB\vietkb.exe) is NEVER killed, restarted or relaunched. If it is
  alive, the run stops with HOLD(owner-quit) and asks the owner to quit it; with
  -OwnerQuit it waits up to 120 s and then aborts. Only a process THIS run launched, and
  only a vietkb.exe under E:\vietkb\target\, may be stopped, and every stop or refusal is
  logged as a KILL/REFUSED line in results-host.txt and log.md. A launch that Tauri
  single-instance redirects into the installed app is labelled BLOCKED(redirected) and is
  a void run, not a result.

  Everything this script opens it closes; everything it changes it restores. The process
  under test is quit through the product's own "Thoat han" button, by its own pid.
#>
param(
  [switch]$Fast,
  [switch]$SelfCheck,
  # 99-agent-guardrails.md: the owner's installed app is never killed. With -OwnerQuit the
  # script waits up to 120 s for the owner to close it themselves, then aborts.
  [switch]$OwnerQuit,
  [int]$SoakSeconds = 60,
  [int]$KeyDelayMs = 55,
  [string]$RepoRoot = 'E:\vietkb',
  # Identity of the build this harness was last validated against (the Dev's CH2 fix
  # build). Never a gate on its own: a newer build is legitimate, so a mismatch is a
  # NOTE plus a `# binary-id: DIFFERS` line, not a FAIL. Pass '' to skip the comparison.
  [string]$ExpectSha = '346299C0EA211DFB45CC58A449469C1E06024F3B0973001D0D7B59FEDB113374',
  [int]$ExpectBytes = 8390144
)

$ErrorActionPreference = 'Continue'
$script:StartTime = Get-Date
$script:KeyDelayMs = $KeyDelayMs
$script:Dir = if ($PSScriptRoot) { $PSScriptRoot } else { Split-Path -Parent $MyInvocation.MyCommand.Path }
$script:ScriptPath = if ($PSCommandPath) { $PSCommandPath } else { Join-Path $script:Dir 'run-host-e2e.ps1' }
$script:ResultsPath = Join-Path $script:Dir 'results-host.txt'
$script:AppLog = Join-Path $script:Dir 'app.log'
$script:Launcher = Join-Path $script:Dir 'launch-vietkb.cmd'
$script:ProbeHtml = Join-Path $script:Dir 'probe-host.html'
$script:Exe = Join-Path $RepoRoot 'target\release\vietkb.exe'
$script:TargetRoot = Join-Path $RepoRoot 'target'
$script:InstalledRoot = Join-Path $env:LOCALAPPDATA 'VietKB'
$script:SimExe = Join-Path $RepoRoot 'target\release\vkb.exe'
$script:ConfigPath = Join-Path $env:APPDATA 'vietkb\config.toml'
$script:ConfigBackup = Join-Path $script:Dir 'config.backup.toml'
$script:Lib95 = Join-Path $RepoRoot 'docs\evkey-clone-work\qa\e2e95\lib95.ps1'

# --------------------------------------------------------------- literals ----
# Built from codepoints (never typed as UTF-8 in this file) so encoding cannot lie.
function New-Text([int[]]$cp) { return [string]::new([char[]]$cp) }

$script:WinSettings = New-Text @(0x43, 0xE0, 0x69, 0x20, 0x111, 0x1EB7, 0x74, 0x20, 0x56, 0x69, 0x65, 0x74, 0x4B, 0x42)          # Cai dat VietKB
$script:SwEnabled   = New-Text @(0x42, 0x1EAD, 0x74, 0x20, 0x74, 0x69, 0x1EBF, 0x6E, 0x67, 0x20, 0x56, 0x69, 0x65, 0x74, 0x1EC7, 0x74) # Bat tieng Viet
$script:BtnClose    = New-Text @(0x110, 0xF3, 0x6E, 0x67)                                                                        # Dong
$script:BtnQuit     = New-Text @(0x54, 0x68, 0x6F, 0xE1, 0x74, 0x20, 0x68, 0x1EB3, 0x6E)                                         # Thoat han
$script:TrayToggle  = New-Text @(0x42, 0x1EAD, 0x74, 0x2F, 0x74, 0x1EAF, 0x74, 0x20, 0x74, 0x69, 0x1EBF, 0x6E, 0x67, 0x20, 0x56, 0x69, 0x65, 0x74, 0x1EC7, 0x74)
$script:TrayOpen    = New-Text @(0x4D, 0x1EDF, 0x20, 0x63, 0xE0, 0x69, 0x20, 0x111, 0x1EB7, 0x74, 0x2026)                        # Mo cai dat...
$script:TrayQuit    = New-Text @(0x54, 0x68, 0x6F, 0xE1, 0x74, 0x20, 0x56, 0x69, 0x65, 0x74, 0x4B, 0x42)                         # Thoat VietKB
$script:TrayMethod  = New-Text @(0x4B, 0x69, 0x1EC3, 0x75, 0x20, 0x67, 0xF5)                                                      # Kieu go
$script:OverflowVi  = New-Text @(0x42, 0x69, 0x1EC3, 0x75, 0x20, 0x74, 0x1B0, 0x1EE3, 0x6E, 0x67, 0x20, 0x1EA9, 0x6E)              # Bieu tuong an
$script:NotifyVi    = New-Text @(0x54, 0x68, 0xF4, 0x6E, 0x67, 0x20, 0x62, 0xE1, 0x6F)                                           # Thong bao
$script:OffTooltip  = New-Text @(0x56, 0x69, 0x65, 0x74, 0x4B, 0x42, 0x20, 0xB7, 0x20, 0x110, 0xE3, 0x20, 0x74, 0x1EAF, 0x74, 0x20, 0x74, 0x69, 0x1EBF, 0x6E, 0x67, 0x20, 0x56, 0x69, 0x1EC7, 0x74)

$script:V_BAT = 'b' + [char]0x1EAD + 't'                                        # bat (tone sac + nang)
$script:V_BA = 'b' + [char]0xE2                                                # ba (circumflex)
$script:V_TAT = 't' + [char]0x1EAF + 't'                                       # tat
$script:V_BA_NANG = 'b' + [char]0x1EAD                                         # ba + nang: B1/T13 after BS
$script:V_HA = 'H' + [char]0xE0
$script:V_NGUYEN = 'Nguy' + [char]0x1EC5 + 'n Thu ' + $script:V_HA
$script:V_MUC1 = 'm' + [char]0x1EE5 + 'c 1 '
$script:V_TAI = [string]([char]0x74 + [char]0x1EA3 + [char]0x69)
$script:V_UE = [string]([char]0xFA + [char]0x65)

# ---------------------------------------------------------------- buckets ----
$script:Counts = @{ PASS = 0; RAW = 0; QUALITY = 0; READFAIL = 0; FAIL = 0; MANUAL = 0; SKIP = 0; BLOCK = 0 }
$script:Notes = 0
$script:Started = @{ Procs = @(); ChromeProfiles = @() }
$script:Failures = New-Object System.Collections.ArrayList
$script:ManualRows = New-Object System.Collections.ArrayList
$script:ReadFails = New-Object System.Collections.ArrayList
$script:SkipRows = New-Object System.Collections.ArrayList
$script:Kills = New-Object System.Collections.ArrayList
$script:OwnedPids = New-Object System.Collections.ArrayList
$script:PrePids = @()
$script:TestPid = 0
$script:OwnerWasRunning = $false
$script:OwnerQuitByOwner = $false
$script:BlockVerdict = ''
$script:RunActive = $false
$script:Method = 'vni'
$script:BaseMethod = 'vni'
$script:GuardReported = $false
$script:FocusCount = 0
$script:MsaaOk = $false

# ---------------------------------------------------------------- output ----
function W([string]$s) { Write-Host $s }

function Section([string]$name) {
  $elapsed = [int]((Get-Date) - $script:StartTime).TotalSeconds
  W ''
  W ('=' * 78)
  W ("[{0,3}s] {1}" -f $elapsed, $name)
  W ('=' * 78)
}

function Esc([string]$s) {
  if ($null -eq $s) { return '' }
  return $s.Replace("`r`n", '\n').Replace("`n", '\n').Replace("`r", '\n').Replace("`t", '\t').Replace('|', '/')
}

# Codepoints for anything a reader must be shown byte-exactly (U+FFFC, controls).
function Show-Bytes([string]$s) {
  if ($null -eq $s) { return '(null)' }
  $sb = New-Object System.Text.StringBuilder
  foreach ($ch in $s.ToCharArray()) {
    $c = [int]$ch
    if ($c -lt 32 -or $c -eq 0xFFFC -or $c -gt 126) { [void]$sb.Append('<U+' + $c.ToString('X4') + '>') }
    else { [void]$sb.Append($ch) }
  }
  return $sb.ToString()
}

# One result row: app | id | keystrokes | expected | actual | raw | verdict | via | note
function Row {
  param(
    [string]$App, [string]$Id, [string]$Keys, [string]$Expected, [string]$Actual,
    [string]$Raw, [string]$Verdict, [string]$Via = '', [string]$Note = '',
    [switch]$NoCount
  )
  $line = "{0} | {1} | {2} | {3} | {4} | {5} | {6} | {7}" -f `
    (Esc $App), (Esc $Id), (Esc $Keys), (Esc $Expected), (Esc $Actual), (Esc $Raw), $Verdict, (Esc $Via)
  if ($Note) { $line += ' | ' + (Esc $Note) }
  Add-Content -Encoding UTF8 -Path $script:ResultsPath -Value $line
  W ("{0,-9} {1,-30} act=[{2,-30}] exp=[{3,-24}] {4}" -f $App, $Id, (Esc $Actual), (Esc $Expected), $Verdict)
  if ($NoCount) { return }
  switch ($Verdict) {
    'PASS' { $script:Counts.PASS++ }
    'PASS(raw)' { $script:Counts.RAW++ }
    'QUALITY' { $script:Counts.QUALITY++ }
    'READ-FAIL' {
      $script:Counts.READFAIL++
      [void]$script:ReadFails.Add("$App/$Id -> [$Actual]")
    }
    'MANUAL' {
      $script:Counts.MANUAL++
      [void]$script:ManualRows.Add("$App/$Id")
    }
    'SKIP(unbound)' {
      $script:Counts.SKIP++
      [void]$script:SkipRows.Add("$App/$Id")
    }
    'BLOCKED(redirected)' {
      $script:Counts.BLOCK++
      if (-not $script:BlockVerdict) { $script:BlockVerdict = 'BLOCKED(redirected)' }
      [void]$script:Failures.Add("$App/$Id -> blocked run: $Actual")
    }
    'HOLD(owner-quit)' {
      $script:Counts.BLOCK++
      if (-not $script:BlockVerdict) { $script:BlockVerdict = 'HOLD(owner-quit)' }
    }
    'HOLD(stale-target)' {
      $script:Counts.BLOCK++
      if (-not $script:BlockVerdict) { $script:BlockVerdict = 'HOLD(stale-target)' }
    }
    default {
      $script:Counts.FAIL++
      [void]$script:Failures.Add("$App/$Id -> [$Actual] want [$Expected] or [$Raw]")
    }
  }
}

function Note([string]$s) {
  $script:Notes++
  Add-Content -Encoding UTF8 -Path $script:ResultsPath -Value ('NOTE | ' + (Esc $s))
  W ('NOTE      ' + $s)
}

function Focus-Line([string]$app, [string]$where, [string]$was) {
  $script:FocusCount++
  $line = "{0} | {1} | {2} | was [{3}]" -f $app, $where, 're-focused', $was
  Add-Content -Encoding UTF8 -Path $script:ResultsPath -Value ('FOCUS | ' + (Esc $line))
  W ("FOCUS     {0,-8} {1} (foreground was [{2}])" -f $app, $where, (Esc $was))
}

# Verdict arithmetic, one place: QUALITY / MANUAL / SKIP(unbound) / READ-FAIL never block,
# a FAIL does, and a blocked run (redirected launch, owner app in the way) blocks with its
# own label so the report cannot mistake it for a result.
function Get-ShipVerdict($counts) {
  if ($counts.BLOCK -gt 0) {
    if ($script:BlockVerdict) { return $script:BlockVerdict }
    return 'BLOCKED(redirected)'
  }
  if ($counts.FAIL -gt 0) { return 'HOLD' }
  return 'SHIP'
}

function Get-BucketLine($counts) {
  return ("PASS {0} / PASS(raw) {1} / QUALITY {2} / READ-FAIL {3} / FAIL {4} / MANUAL {5} / SKIP {6} / BLOCKED {7}" -f `
      $counts.PASS, $counts.RAW, $counts.QUALITY, $counts.READFAIL, $counts.FAIL, $counts.MANUAL, $counts.SKIP, $counts.BLOCK)
}

# ---------------------------------------------------------------- guards ----
function Assert-Encoding {
  # The file is ASCII-only: every Vietnamese literal is built from codepoints. Two
  # checks: the bytes on disk, and the literals themselves (a bad edit that keeps the
  # file ASCII but drops a codepoint still fails here).
  $bad = @()
  try {
    $bytes = [System.IO.File]::ReadAllBytes($script:ScriptPath)
    for ($i = 0; $i -lt $bytes.Length; $i++) {
      if ($bytes[$i] -gt 127) { $bad += $i; if ($bad.Count -ge 5) { break } }
    }
  } catch {
    Write-Host "FATAL: cannot read own script ($($_.Exception.Message))" -ForegroundColor Red
    exit 9
  }
  if ($bad.Count -gt 0) {
    Write-Host ("FATAL encoding: non-ASCII byte(s) at offset {0} in {1}. Every Vietnamese literal must be built with New-Text." -f ($bad -join ','), $script:ScriptPath) -ForegroundColor Red
    exit 9
  }
  if ($script:V_BAT -ne (New-Text @(0x62, 0x1EAD, 0x74))) {
    Write-Host 'FATAL encoding: Vietnamese literals are mangled.' -ForegroundColor Red
    exit 9
  }
  if ($script:WinSettings -ne (New-Text @(0x43, 0xE0, 0x69, 0x20, 0x111, 0x1EB7, 0x74, 0x20, 0x56, 0x69, 0x65, 0x74, 0x4B, 0x42))) {
    Write-Host 'FATAL encoding: window title literal is mangled.' -ForegroundColor Red
    exit 9
  }
  if ($PSVersionTable.PSVersion.Major -lt 5) {
    Write-Host "FATAL need PowerShell 5.1+ (found $($PSVersionTable.PSVersion))" -ForegroundColor Red
    exit 9
  }
}

function Get-ConfigValue([string]$section, [string]$key) {
  if (-not (Test-Path $script:ConfigPath)) { return '' }
  $in = $false
  foreach ($line in Get-Content -Path $script:ConfigPath -Encoding UTF8) {
    if ($line -match '^\s*\[(.+)\]\s*$') { $in = ($matches[1] -eq $section); continue }
    if (-not $in) { continue }
    if ($line -match ('^\s*' + [regex]::Escape($key) + '\s*=\s*"?(.*?)"?\s*$')) { return $matches[1] }
  }
  return ''
}

# The supported surface for a method change without IPC: the config file, then the app
# reloads it on start. Rewrites exactly one line and proves the rest of the file is
# untouched, so a bad edit fails here rather than silently shipping a scrambled config.
function Remove-MethodLine([string]$text) {
  $nl = if ($text.Contains("`r`n")) { "`r`n" } else { "`n" }
  $kept = @($text -split [regex]::Escape($nl) | Where-Object { $_ -notmatch '^\s*method\s*=' })
  return ($kept -join $nl)
}

function Write-ConfigMethod([string]$method) {
  if (-not (Test-Path $script:ConfigPath)) { W 'config: missing, cannot switch method'; return $false }
  $before = [System.IO.File]::ReadAllText($script:ConfigPath, [System.Text.Encoding]::UTF8)
  $nl = if ($before.Contains("`r`n")) { "`r`n" } else { "`n" }
  $lines = $before -split [regex]::Escape($nl)
  $inInput = $false
  $hit = -1
  $old = ''
  for ($i = 0; $i -lt $lines.Count; $i++) {
    if ($lines[$i] -match '^\s*\[(.+)\]\s*$') { $inInput = ($matches[1] -eq 'input'); continue }
    if (-not $inInput) { continue }
    if ($lines[$i] -match '^\s*method\s*=') { $hit = $i; $old = $lines[$i]; break }
  }
  if ($hit -lt 0) { W 'config: no [input] method key found'; return $false }
  $lines[$hit] = 'method = "' + $method + '"'
  $after = $lines -join $nl
  [System.IO.File]::WriteAllText($script:ConfigPath, $after, (New-Object System.Text.UTF8Encoding($false)))

  # Two proofs: the value reads back, and every other line is byte-identical.
  $read = Get-ConfigValue 'input' 'method'
  if ($read -ne $method) { W ("config: read-back says method=`"{0}`" want `"{1}`"" -f $read, $method); return $false }
  if ((Remove-MethodLine $before) -cne (Remove-MethodLine $after)) {
    W 'config: the rewrite touched more than the method line'
    return $false
  }
  W ("config: [input] method line [{0}] -> [method = `"{1}`"] ({2} bytes)" -f $old.Trim(), $method, (New-Object System.Text.UTF8Encoding($false)).GetByteCount($after))
  return $true
}

# ------------------------------------------------------------- bindings -----
# Mirrors crates/vkb-config/src/hotkey.rs: case- and order-insensitive, `+`-separated,
# modifier-only chords allowed, a bare key only for F1..F24. Nothing here knows a chord
# value: every value comes from the live config at run time.
$script:ModVk = @{ Ctrl = 0x11; Alt = 0x12; Shift = 0x10; Win = 0x5B }

function Get-KeyVk([string]$token) {
  $u = $token.Trim().ToUpperInvariant()
  if (-not $u) { return $null }
  if ($u.Length -eq 1) {
    $v = [int][char]$u
    if ($v -ge 0x41 -and $v -le 0x5A) { return [uint16]$v }
    if ($v -ge 0x30 -and $v -le 0x39) { return [uint16]$v }
    $punctuation = @{ '`' = 0xC0; '-' = 0xBD; '=' = 0xBB; '+' = 0xBB; ',' = 0xBC; '.' = 0xBE; '/' = 0xBF; ';' = 0xBA; '[' = 0xDB; '\' = 0xDC; ']' = 0xDD; "'" = 0xDE }
    if ($punctuation.ContainsKey($u)) { return [uint16]$punctuation[$u] }
    return $null
  }
  if ($u -match '^F(\d{1,2})$') {
    $n = [int]$matches[1]
    if ($n -ge 1 -and $n -le 24) { return [uint16](0x70 + $n - 1) }
  }
  $named = @{
    'SPACE' = 0x20; 'SPACEBAR' = 0x20; 'ENTER' = 0x0D; 'RETURN' = 0x0D; 'TAB' = 0x09; 'ESC' = 0x1B; 'ESCAPE' = 0x1B
    'BACKSPACE' = 0x08; 'BKSP' = 0x08; 'DELETE' = 0x2E; 'DEL' = 0x2E; 'INSERT' = 0x2D; 'INS' = 0x2D
    'HOME' = 0x24; 'END' = 0x23; 'PAGEUP' = 0x21; 'PGUP' = 0x21; 'PAGEDOWN' = 0x22; 'PGDN' = 0x22
    'UP' = 0x26; 'DOWN' = 0x28; 'LEFT' = 0x25; 'RIGHT' = 0x27
    'BACKQUOTE' = 0xC0; 'MINUS' = 0xBD; 'EQUAL' = 0xBB; 'COMMA' = 0xBC; 'PERIOD' = 0xBE; 'SLASH' = 0xBF
    'SEMICOLON' = 0xBA; 'LBRACKET' = 0xDB; 'BACKSLASH' = 0xDC; 'RBRACKET' = 0xDD; 'QUOTE' = 0xDE
  }
  if ($named.ContainsKey($u)) { return [uint16]$named[$u] }
  return $null
}

function Parse-Hotkey([string]$spec) {
  $out = [pscustomobject]@{
    Raw = $spec; Empty = $true; Valid = $false
    Ctrl = $false; Alt = $false; Shift = $false; Win = $false
    Vk = [uint16]0; Key = ''; Reason = ''
  }
  $s = ''
  if ($null -ne $spec) { $s = $spec.Trim() }
  if (-not $s) { $out.Reason = 'empty (unbound)'; return $out }
  $out.Empty = $false
  $vk = [uint16]0
  $keyName = ''
  $sawKey = $false
  foreach ($raw in ($s -split '\+')) {
    $t = $raw.Trim()
    if (-not $t) { continue }          # "Ctrl++" ends on a literal plus
    $low = $t.ToLowerInvariant()
    if ($low -eq 'ctrl' -or $low -eq 'control') { $out.Ctrl = $true; continue }
    if ($low -eq 'alt') { $out.Alt = $true; continue }
    if ($low -eq 'shift') { $out.Shift = $true; continue }
    if ($low -eq 'win' -or $low -eq 'windows' -or $low -eq 'super' -or $low -eq 'meta') { $out.Win = $true; continue }
    if ($sawKey) { $out.Reason = "second key token: $t"; return $out }
    $v = Get-KeyVk $t
    if ($null -eq $v) { $out.Reason = "unknown key: $t"; return $out }
    $vk = [uint16]$v
    $keyName = $t.ToUpperInvariant()
    $sawKey = $true
  }
  $out.Vk = $vk
  if ($sawKey) { $out.Key = $keyName }
  $mods = $out.Ctrl -or $out.Alt -or $out.Shift -or $out.Win
  if (-not $mods -and -not $sawKey) { $out.Reason = 'no modifier and no key'; return $out }
  if (-not $mods -and -not ($keyName -match '^F\d{1,2}$')) { $out.Reason = 'a bare key must be F1..F24'; return $out }
  $out.Valid = $true
  return $out
}

function Get-BindingLabel($hk) {
  if (-not $hk -or $hk.Empty) { return '(unbound)' }
  if (-not $hk.Valid) { return ("(invalid: {0})" -f $hk.Reason) }
  $parts = New-Object System.Collections.ArrayList
  if ($hk.Ctrl) { [void]$parts.Add('Ctrl') }
  if ($hk.Alt) { [void]$parts.Add('Alt') }
  if ($hk.Shift) { [void]$parts.Add('Shift') }
  if ($hk.Win) { [void]$parts.Add('Win') }
  if ($hk.Vk -ne 0) { [void]$parts.Add($hk.Key) }
  return ($parts -join '+')
}

function Get-BindingDetail($hk) {
  $mods = New-Object System.Collections.ArrayList
  if ($hk.Ctrl) { [void]$mods.Add('ctrl') }
  if ($hk.Alt) { [void]$mods.Add('alt') }
  if ($hk.Shift) { [void]$mods.Add('shift') }
  if ($hk.Win) { [void]$mods.Add('win') }
  $m = if ($mods.Count -gt 0) { $mods -join '+' } else { '(none)' }
  $k = if ($hk.Vk -ne 0) { ("{0} (0x{1:X2})" -f $hk.Key, $hk.Vk) } else { '(none, modifier-only)' }
  $v = if ($hk.Valid) { 'ok' } elseif ($hk.Empty) { 'unbound' } else { 'invalid: ' + $hk.Reason }
  return ("mods={0,-14} key={1,-22} {2}" -f $m, $k, $v)
}

# The live bindings, parsed once, in one place. Every press in this script reads here.
function Get-Bindings {
  $b = [pscustomobject]@{
    Toggle     = Parse-Hotkey (Get-ConfigValue 'hotkeys' 'toggle')
    Restore    = Parse-Hotkey (Get-ConfigValue 'hotkeys' 'restore_word')
    Open       = Parse-Hotkey (Get-ConfigValue 'hotkeys' 'open_settings')
    Enable     = Parse-Hotkey (Get-ConfigValue 'hotkeys' 'enable')
    Disable    = Parse-Hotkey (Get-ConfigValue 'hotkeys' 'disable')
    ToggleMode = Get-ConfigValue 'hotkeys' 'toggle_mode'
  }
  return $b
}

# The CONFIG block: every binding the run will press, as the config has it, so a reader
# can check the evidence against the machine instead of trusting the script.
function Get-ConfigLines {
  $b = $script:Bind
  if (-not $b) { return @('# CONFIG unavailable (no bindings parsed)') }
  $rows = @(
    @('toggle', $b.Toggle), @('restore_word', $b.Restore), @('open_settings', $b.Open),
    @('enable', $b.Enable), @('disable', $b.Disable)
  )
  $out = New-Object System.Collections.ArrayList
  foreach ($r in $rows) {
    $hk = $r[1]
    [void]$out.Add(("# CONFIG {0,-13} raw=`"{1}`"  parsed={2,-16} {3}" -f $r[0], $hk.Raw, (Get-BindingLabel $hk), (Get-BindingDetail $hk)))
  }
  [void]$out.Add(("# CONFIG {0,-13} {1}  (the toggle chord vs a longer chord: the longer one wins)" -f 'toggle_mode', $b.ToggleMode))
  return @($out)
}

function Write-ConfigBlock {
  foreach ($line in (Get-ConfigLines)) {
    Add-Content -Encoding UTF8 -Path $script:ResultsPath -Value $line
    W $line
  }
}

# Release orders the platform's matcher has tests for: the key first, each modifier
# first, and the whole chord released after the modifiers.
function Get-ReleaseOrders($hk) {
  $orders = New-Object System.Collections.ArrayList
  if ($hk.Vk -ne 0) { [void]$orders.Add('key') }
  if ($hk.Shift) { [void]$orders.Add('shift') }
  if ($hk.Ctrl) { [void]$orders.Add('ctrl') }
  if ($hk.Alt) { [void]$orders.Add('alt') }
  [void]$orders.Add('both')
  return @($orders)
}

# Presses one parsed binding. order: "key" (key up first), "shift"/"ctrl"/"alt" (that
# modifier up first, key last), "both" (every modifier up, then the key).
function Send-Binding($hk, [string]$order = 'key', [switch]$ModifiersOnly) {
  if (-not $hk -or -not $hk.Valid) { return $false }
  $mods = New-Object System.Collections.ArrayList
  if ($hk.Ctrl) { [void]$mods.Add([int]$script:ModVk.Ctrl) }
  if ($hk.Alt) { [void]$mods.Add([int]$script:ModVk.Alt) }
  if ($hk.Shift) { [void]$mods.Add([int]$script:ModVk.Shift) }
  if ($hk.Win) { [void]$mods.Add([int]$script:ModVk.Win) }
  $keyVk = 0
  if (-not $ModifiersOnly) { $keyVk = [int]$hk.Vk }

  foreach ($m in $mods) { [HostKey]::Down([uint16]$m) }
  if ($keyVk -ne 0) { [HostKey]::Down([uint16]$keyVk) }
  Start-Sleep -Milliseconds 60

  if ($keyVk -ne 0 -and $order -eq 'key') { [HostKey]::Up([uint16]$keyVk) }
  $early = -1
  if ($order -eq 'shift' -and $hk.Shift) { $early = [int]$script:ModVk.Shift }
  elseif ($order -eq 'ctrl' -and $hk.Ctrl) { $early = [int]$script:ModVk.Ctrl }
  elseif ($order -eq 'alt' -and $hk.Alt) { $early = [int]$script:ModVk.Alt }
  if ($early -ge 0) { [HostKey]::Up([uint16]$early) }
  foreach ($m in $mods) { if ($m -ne $early) { [HostKey]::Up([uint16]$m) } }
  if ($keyVk -ne 0 -and $order -ne 'key') { [HostKey]::Up([uint16]$keyVk) }
  Start-Sleep -Milliseconds 40
  return $true
}

# Presses a single modifier alone (the partial chord a toggle must ignore).
function Send-SingleModifier([int]$modVk) {
  [HostKey]::Down([uint16]$modVk)
  Start-Sleep -Milliseconds 60
  [HostKey]::Up([uint16]$modVk)
  Start-Sleep -Milliseconds 40
}

# ------------------------------------------------------------- processes ----
# 99-agent-guardrails.md / 00-charter.md section 6: only a process THIS RUN launched, and
# only a vietkb.exe under E:\vietkb\target\, may ever be stopped. The owner's installed
# build (%LOCALAPPDATA%\VietKB\vietkb.exe) is off-limits: no kill, no restart, no relaunch.
function Get-VkbProcesses { return @(Get-Process -Name vietkb -ErrorAction SilentlyContinue) }

function Get-ProcPath($p) {
  if (-not $p) { return '' }
  try { return $p.Path } catch { return '' }
}

function Test-PathUnder([string]$path, [string]$root) {
  if (-not $path -or -not $root) { return $false }
  try {
    $p = [System.IO.Path]::GetFullPath($path).TrimEnd('\')
    $r = [System.IO.Path]::GetFullPath($root).TrimEnd('\')
  } catch { return $false }
  if ($p.Length -le $r.Length) { return $false }
  return $p.StartsWith($r + '\', [System.StringComparison]::OrdinalIgnoreCase)
}

function Get-InstalledInstances {
  return @(Get-VkbProcesses | Where-Object { Test-PathUnder (Get-ProcPath $_) $script:InstalledRoot })
}

function Get-TestInstances {
  return @(Get-VkbProcesses | Where-Object { Test-PathUnder (Get-ProcPath $_) $script:TargetRoot })
}

function Get-VietkbCount { return @(Get-VkbProcesses).Count }

function Format-Proc($p) {
  if (-not $p) { return '(none)' }
  $started = ''
  try { $started = '{0:yyyy-MM-dd HH:mm:ss}' -f $p.StartTime } catch { $started = '(unknown)' }
  return ("pid={0} path={1} started={2}" -f $p.Id, (Get-ProcPath $p), $started)
}

# The one place that decides whether a stop is allowed. Pure, so -SelfCheck can prove the
# rule without touching a live process.
function Test-KillAllowed([string]$name, [string]$path, [int]$procId) {
  if ($name -eq 'vietkb' -and -not (Test-PathUnder $path $script:TargetRoot)) {
    return ('refused: vietkb at {0} is outside {1} (the owner app is off-limits)' -f $path, $script:TargetRoot)
  }
  if ($script:OwnedPids -notcontains $procId) {
    return ('refused: pid {0} was not launched by this run' -f $procId)
  }
  return 'allowed'
}

# Writes the mandatory kill/refusal line to results-host.txt and, during a real run, log.md.
function Write-KillLog([string]$line) {
  [void]$script:Kills.Add($line)
  if ($script:ResultsPath) { Add-Content -Encoding UTF8 -Path $script:ResultsPath -Value $line -ErrorAction SilentlyContinue }
  if ($script:RunActive) {
    $logPath = Join-Path $RepoRoot 'docs\evkey-clone-work\log.md'
    Add-Content -Encoding UTF8 -Path $logPath -Value ($line + ' (host E2E harness)') -ErrorAction SilentlyContinue
  }
  W ("KILL/REFUSED: " + $line)
}

# Stops one process, if and only if the rule allows it. Always logs first.
function Stop-Owned([int]$procId, [string]$reason) {
  $p = Get-Process -Id $procId -ErrorAction SilentlyContinue
  if (-not $p) { return $false }
  $name = $p.ProcessName
  $path = Get-ProcPath $p
  $verdict = Test-KillAllowed $name $path $procId
  $flag = if ($verdict -eq 'allowed') { 'KILL' } else { 'REFUSED' }
  $line = ('{0} {1} pid={2} path={3} reason={4}' -f $flag, (Get-Date -Format 'yyyy-MM-dd HH:mm:ss'), $procId, $path, $(if ($verdict -eq 'allowed') { $reason } else { $verdict + '; wanted: ' + $reason }))
  Write-KillLog $line
  if ($verdict -ne 'allowed') { return $false }
  try {
    Stop-Process -Id $procId -Force -ErrorAction Stop
    Start-Sleep -Milliseconds 200
    return $true
  } catch {
    W ("stop failed: {0}" -f $_.Exception.Message)
    return $false
  }
}

function Get-VietkbPid { return [int]$script:TestPid }

# The owner closes their own app; the script only watches. Used by -OwnerQuit.
function Wait-OwnerQuit([int]$seconds = 120) {
  W ("waiting up to {0} s for the owner to close the installed VietKB (this script will not kill it)" -f $seconds)
  $end = (Get-Date).AddSeconds($seconds)
  $last = -1
  while ((Get-Date) -lt $end) {
    $left = [int](($end - (Get-Date)).TotalSeconds)
    if ($left -ne $last -and ($left % 15 -eq 0)) { $last = $left; W ("  {0,3}s left - waiting for the owner" -f $left) }
    if ((Get-InstalledInstances).Count -eq 0 -and (Get-TestInstances | Where-Object { $script:OwnedPids -notcontains $_.Id }).Count -eq 0) { return $true }
    Start-Sleep -Milliseconds 1000
  }
  return (((Get-InstalledInstances).Count -eq 0) -and ((Get-TestInstances | Where-Object { $script:OwnedPids -notcontains $_.Id }).Count -eq 0))
}

# Records a blocking outcome and the label the summary must carry.
function Set-Block([string]$verdict, [string]$why) {
  if (-not $script:BlockVerdict) { $script:BlockVerdict = $verdict }
  Row -App 'runtime' -Id ('BLOCK-' + ($verdict -replace '[^A-Za-z0-9]+', '-').Trim('-')) -Keys 'process guard' -Expected 'the build under test, and the owner app untouched' -Actual $why -Raw '(process table)' -Verdict $verdict -Via 'pid+path'
}

# A second instance takes over the hook and every row then fails for the wrong
# reason. Report it once, as a FAIL, instead of letting the matrix look broken.
# Nothing is killed: a process this run did not launch belongs to the owner.
function Guard-Single {
  $test = Get-Process -Id $script:TestPid -ErrorAction SilentlyContinue
  $all = Get-VkbProcesses
  if ($test -and $all.Count -eq 1) { return $true }
  if (-not $script:GuardReported) {
    $script:GuardReported = $true
    $others = @($all | Where-Object { $_.Id -ne $script:TestPid } | ForEach-Object { Format-Proc $_ })
    Row -App 'runtime' -Id 'GUARD-single-instance' -Keys 'Get-Process vietkb (pid+path)' -Expected 'exactly 1: the process under test' -Actual ("test pid={0} alive={1}; others: {2}" -f $script:TestPid, [bool]$test, $(if ($others.Count -gt 0) { $others -join '; ' } else { 'none' })) -Raw '(process table)' -Verdict 'FAIL' -Via 'pid+path' -Note 'another vietkb is alive during the run (owner app or a manual launch). It is NOT killed; close it and re-run.'
  }
  return $false
}

# ---------------------------------------------------------------- launch ----
function Write-Launcher {
  # windows-detached-launch: tiny .cmd, redirected stdin/stdout/stderr, explicit
  # markers, launched through COM ShellExecute. The console is hidden (nShowCmd=0)
  # so no stray console can steal the foreground from the typing targets; the app
  # itself is a tray-resident GUI process and shows its own windows.
  $lines = @(
    '@echo off',
    "cd /d `"$RepoRoot`"",
    'set RUST_LOG=vkb=debug',
    'set VKB_DEBUG_KEYS=1',
    'set RUST_BACKTRACE=1',
    ('echo ==== LAUNCH %DATE% %TIME% ====>> "{0}"' -f $script:AppLog),
    ('"{0}" <nul >> "{1}" 2>&1' -f $script:Exe, $script:AppLog),
    ('echo ==== EXIT %ERRORLEVEL% %DATE% %TIME% ====>> "{0}"' -f $script:AppLog)
  )
  Set-Content -Path $script:Launcher -Value $lines -Encoding ASCII
}

function Start-App {
  Write-Launcher
  $script:PrePids = @(Get-VkbProcesses | ForEach-Object { $_.Id })
  $shell = New-Object -ComObject Shell.Application
  $shell.ShellExecute($script:Launcher, '', $RepoRoot, 'open', 0) | Out-Null
  W ("launched: {0} (ShellExecute, hidden console) -> {1}" -f $script:Launcher, $script:Exe)
  W ("pids alive before the launch: {0}" -f $(if ($script:PrePids.Count -gt 0) { $script:PrePids -join ',' } else { 'none' }))
}

# Waits for a process that is (a) under E:\vietkb\target\ and (b) new since the launch.
# Anything else means Tauri single-instance sent the launch to the owner's app.
function Wait-TestApp([int]$timeoutSec = 25) {
  $end = (Get-Date).AddSeconds($timeoutSec)
  while ((Get-Date) -lt $end) {
    foreach ($p in (Get-VkbProcesses)) {
      $path = Get-ProcPath $p
      if (-not (Test-PathUnder $path $script:TargetRoot)) { continue }
      if ($script:PrePids -contains $p.Id) { continue }
      Start-Sleep -Milliseconds 900
      W ("ready: test instance {0}" -f (Format-Proc $p))
      return $p
    }
    Start-Sleep -Milliseconds 250
  }
  return $null
}

# Launch + identity check. A redirect into the installed build is a blocked run, not a
# result: nothing is graded, and the owner's app is not touched.
function Start-AppVerified {
  Start-App
  $p = Wait-TestApp 25
  if ($p) {
    [void]$script:OwnedPids.Add([int]$p.Id)
    $script:TestPid = [int]$p.Id
    $path = Get-ProcPath $p
    if (-not (Test-PathUnder $path $script:TargetRoot)) {
      Set-Block 'BLOCKED(redirected)' ("the new process is outside target\: " + (Format-Proc $p))
      return $null
    }
    W ("identity check ok: pid={0} path={1} (target\release)" -f $p.Id, $path)
    return $p
  }
  $installed = Get-InstalledInstances
  if ($installed.Count -gt 0) {
    $script:TestPid = 0
    Set-Block 'BLOCKED(redirected)' ("no process under target\ came up; Tauri single-instance handed the launch to the installed build: " + (($installed | ForEach-Object { Format-Proc $_ }) -join '; '))
    return $null
  }
  $script:TestPid = 0
  return $null
}

# ----------------------------------------------------- native window layer ----
if (-not ('VkbWin' -as [type])) {
  Add-Type @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;

public static class VkbWin {
  [DllImport("user32.dll")] static extern bool EnumWindows(EnumProc cb, IntPtr p);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetClassNameW(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetWindowTextW(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] static extern bool IsWindow(IntPtr h);
  [DllImport("user32.dll")] static extern bool ShowWindow(IntPtr h, int cmd);
  [DllImport("user32.dll")] static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] static extern bool BringWindowToTop(IntPtr h);
  [DllImport("user32.dll")] static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll")] static extern bool IsIconic(IntPtr h);
  [DllImport("kernel32.dll")] static extern uint GetCurrentThreadId();
  [DllImport("user32.dll")] static extern bool AttachThreadInput(uint a, uint b, bool f);
  [DllImport("user32.dll")] static extern bool PostMessageW(IntPtr h, uint msg, IntPtr w, IntPtr l);
  [DllImport("oleacc.dll")] static extern int AccessibleObjectFromWindow(IntPtr h, uint id, ref Guid iid, [MarshalAs(UnmanagedType.Interface)] out object acc);

  public delegate bool EnumProc(IntPtr h, IntPtr p);

  public static IntPtr[] All() {
    List<IntPtr> list = new List<IntPtr>();
    EnumWindows(delegate(IntPtr h, IntPtr p) { list.Add(h); return true; }, IntPtr.Zero);
    return list.ToArray();
  }
  public static string Class(IntPtr h) {
    StringBuilder sb = new StringBuilder(256);
    GetClassNameW(h, sb, 256);
    return sb.ToString();
  }
  public static string Title(IntPtr h) {
    StringBuilder sb = new StringBuilder(512);
    GetWindowTextW(h, sb, 512);
    return sb.ToString();
  }
  public static bool Visible(IntPtr h) { return IsWindowVisible(h); }
  public static bool Alive(IntPtr h) { return IsWindow(h); }
  public static uint Pid(IntPtr h) { uint p = 0; GetWindowThreadProcessId(h, out p); return p; }
  public static IntPtr Foreground() { return GetForegroundWindow(); }
  public static string ForegroundTitle() { return Title(GetForegroundWindow()); }

  // Show + foreground, with the AttachThreadInput dance the foreground lock needs.
  public static bool Front(IntPtr h) {
    if (!IsWindow(h)) { return false; }
    if (IsIconic(h)) { ShowWindow(h, 9); }
    ShowWindow(h, 5);
    if (GetForegroundWindow() == h) { return true; }
    uint p1 = 0; uint p2 = 0;
    GetWindowThreadProcessId(h, out p1);
    GetWindowThreadProcessId(GetForegroundWindow(), out p2);
    uint me = GetCurrentThreadId();
    AttachThreadInput(me, p2, true);
    AttachThreadInput(me, p1, true);
    BringWindowToTop(h);
    SetForegroundWindow(h);
    AttachThreadInput(me, p1, false);
    AttachThreadInput(me, p2, false);
    System.Threading.Thread.Sleep(140);
    return GetForegroundWindow() == h;
  }

  public static void Close(IntPtr h) { PostMessageW(h, 0x0010, IntPtr.Zero, IntPtr.Zero); }

  // MSAA, only for a classic Win32 popup menu that UIA exposes thinly.
  public static object Acc(IntPtr h) {
    Guid iid = new Guid("618736E0-3C3D-11CF-810C-00AA00389B71");
    object acc = null;
    if (AccessibleObjectFromWindow(h, 0xFFFFFFFC, ref iid, out acc) == 0) { return acc; }
    acc = null;
    if (AccessibleObjectFromWindow(h, 0x00000000, ref iid, out acc) == 0) { return acc; }
    return null;
  }
}
'@
}

function Get-AppWindows([int]$procId) {
  $out = New-Object System.Collections.ArrayList
  foreach ($h in [VkbWin]::All()) {
    if ([VkbWin]::Pid($h) -ne [uint32]$procId) { continue }
    [void]$out.Add([pscustomobject]@{
        Hwnd    = $h
        Title   = [VkbWin]::Title($h)
        Class   = [VkbWin]::Class($h)
        Visible = [VkbWin]::Visible($h)
      })
  }
  return $out
}

function Get-ForegroundInfo {
  $h = [VkbWin]::Foreground()
  return [pscustomobject]@{ Hwnd = $h; Title = [VkbWin]::Title($h); Class = [VkbWin]::Class($h) }
}

# --------------------------------------------------------------- targets ----
function Start-Notepad {
  $f = Join-Path $script:Dir 'scratch-np.txt'
  Remove-Item $f -ErrorAction SilentlyContinue
  New-Item -ItemType File -Path $f | Out-Null
  $p = Start-Process notepad.exe -ArgumentList $f -PassThru
  $script:Started.Procs += $p.Id
  $w = Wait-Window 'scratch-np.txt' 30
  if (-not $w) { return $null }
  return [pscustomobject]@{ App = 'notepad'; Hwnd = $w.Hwnd; Pid = $w.Pid; Via = 'TextPattern'; File = $f; Marker = 'scratch-np.txt' }
}

function Find-Chrome {
  foreach ($c in @(
      (Join-Path $env:ProgramFiles 'Google\Chrome\Application\chrome.exe'),
      (Join-Path ${env:ProgramFiles(x86)} 'Google\Chrome\Application\chrome.exe'),
      (Join-Path $env:LOCALAPPDATA 'Google\Chrome\Application\chrome.exe'))) {
    if ($c -and (Test-Path $c)) { return $c }
  }
  return $null
}

function Write-ProbeHtml {
  $html = @'
<!doctype html><html lang="vi"><head><meta charset="utf-8"><title>VietKB E2E probe</title>
<style>html,body{margin:0;height:100%;background:#101418;color:#e6edf3;font:16px/1.5 Consolas,monospace}
textarea{box-sizing:border-box;width:100%;height:100%;padding:14px;border:0;outline:0;background:#101418;color:#e6edf3;font:inherit;resize:none}</style>
</head><body><textarea id="t" autofocus spellcheck="false"></textarea>
<script>var t=document.getElementById('t');t.focus();setInterval(function(){if(document.activeElement!==t){t.focus()}},400);</script>
</body></html>
'@
  Set-Content -Path $script:ProbeHtml -Value $html -Encoding UTF8
}

function Start-ChromeProbe {
  $chrome = Find-Chrome
  if (-not $chrome) { return $null }
  Write-ProbeHtml
  $prof = Join-Path $env:TEMP 'vkb-host-e2e-chrome'
  Remove-Item $prof -Recurse -Force -ErrorAction SilentlyContinue
  $script:Started.ChromeProfiles += $prof

  # Local page, no network: a data: URL first (nothing on disk), with the file:
  # fallback only because some Chrome builds refuse data: in --app mode.
  $body = Get-Content -Path $script:ProbeHtml -Raw -Encoding UTF8
  $dataUrl = 'data:text/html;charset=utf-8,' + [uri]::EscapeDataString($body)
  $common = @(
    "--user-data-dir=`"$prof`"", '--no-first-run', '--no-default-browser-check',
    '--force-renderer-accessibility', '--disable-features=Translate'
  )
  $p = Start-Process $chrome -ArgumentList ($common + "--app=$dataUrl") -PassThru
  $script:Started.Procs += $p.Id
  $w = Wait-Window 'VietKB E2E probe' 15
  if (-not $w) {
    W 'chrome: data: probe did not open; falling back to the local file'
    $url = ([uri]$script:ProbeHtml).AbsoluteUri
    $p = Start-Process $chrome -ArgumentList ($common + "--app=$url") -PassThru
    $script:Started.Procs += $p.Id
    $w = Wait-Window 'VietKB E2E probe' 20
    if (-not $w) { return $null }
    return [pscustomobject]@{ App = 'chrome'; Hwnd = $w.Hwnd; Pid = $w.Pid; Via = 'TextPattern'; File = $null; Page = 'file'; Marker = 'VietKB E2E probe' }
  }
  return [pscustomobject]@{ App = 'chrome'; Hwnd = $w.Hwnd; Pid = $w.Pid; Via = 'TextPattern'; File = $null; Page = 'data'; Marker = 'VietKB E2E probe' }
}

function Start-VsCode {
  $code = Join-Path $env:LOCALAPPDATA 'Programs\Microsoft VS Code\Code.exe'
  if (-not (Test-Path $code)) { return $null }
  $f = Join-Path $script:Dir 'scratch-vsc.txt'
  Remove-Item $f -ErrorAction SilentlyContinue
  New-Item -ItemType File -Path $f | Out-Null
  $p = Start-Process $code -ArgumentList @('-n', $f) -PassThru
  $script:Started.Procs += $p.Id
  $w = Wait-Window 'scratch-vsc.txt' 40
  if (-not $w) { return $null }
  return [pscustomobject]@{ App = 'vscode'; Hwnd = $w.Hwnd; Pid = $w.Pid; Via = 'saved-file'; File = $f; Marker = 'scratch-vsc.txt' }
}

# A target window can die or be replaced (the app restart is unaffected, but an app
# may recreate its frame). Re-resolve by marker before a group of rows.
function Repair-Targets($targets) {
  foreach ($t in $targets) {
    if ([VkbWin]::Alive($t.Hwnd)) { continue }
    $w = Wait-Window $t.Marker 8
    if ($w) {
      $t.Hwnd = $w.Hwnd
      $t.Pid = $w.Pid
      W ("targets: {0} re-resolved hwnd={1}" -f $t.App, $w.Hwnd)
    } else {
      W ("targets: {0} window is gone" -f $t.App)
    }
  }
}

# ------------------------------------------------------------ typing core ---
function Assert-Focus($t, [string]$where) {
  # Foreground discipline: every typing row proves its target is the foreground window
  # and re-takes it when something (a hotkey opening Help, a toast, a flyout) stole it.
  for ($i = 1; $i -le 3; $i++) {
    if ([VkbWin]::Foreground() -eq $t.Hwnd) { return $true }
    $was = [VkbWin]::ForegroundTitle()
    [void][VkbWin]::Front($t.Hwnd)
    Start-Sleep -Milliseconds 90
    if ($t.App -eq 'vscode') {
      Click-Center $t.Hwnd
    } else {
      $el = Get-EditElement $t.Hwnd
      if ($el) { [void](Click-Element $el) } else { Click-Center $t.Hwnd }
    }
    Start-Sleep -Milliseconds 130
    if ([VkbWin]::Foreground() -eq $t.Hwnd) { Focus-Line $t.App $where $was; return $true }
  }
  W ("FOCUS     {0,-8} CANNOT take the foreground at {1}" -f $t.App, $where)
  return $false
}

# Readers. Primary is UIA (TextPattern, ValuePattern built in); the saved file for
# VS Code; the clipboard is the second reader and never the first.
function Read-Primary($t) {
  if ($t.App -eq 'vscode') {
    [QAInput]::Combo(0x53, $false, $true, $false)   # Ctrl+S
    Start-Sleep -Milliseconds 450
    if (Test-Path $t.File) {
      return [pscustomobject]@{ Text = [System.IO.File]::ReadAllText($t.File, [System.Text.Encoding]::UTF8); Via = 'saved-file' }
    }
    return [pscustomobject]@{ Text = ''; Via = 'saved-file(missing)' }
  }
  $u = Get-UiaText $t.Hwnd
  if ($null -eq $u.Text) { $u.Text = '' }
  $via = if ($u.Via -and $u.Via -ne 'none') { $u.Via } else { 'uia(none)' }
  return [pscustomobject]@{ Text = $u.Text; Via = $via }
}

function Read-Secondary($t) {
  $r = Get-ClipboardText $t.Hwnd
  [QAInput]::Combo(0x23, $false, $false, $false)   # End: collapse the selection
  Start-Sleep -Milliseconds 80
  if ($null -eq $r.Text) { $r.Text = '' }
  return [pscustomobject]@{ Text = $r.Text; Via = 'clipboard' }
}

# A text is unusable when it carries the object-replacement char (Chrome's renderer
# does this while a composition is open) or when a field we typed into reads empty.
function Test-ReadSane([string]$text) {
  if ($null -eq $text) { return $false }
  if ($text.IndexOf([char]0xFFFC) -ge 0) { return $false }
  if ($text.Length -eq 0) { return $false }
  return $true
}

function Read-Actual($t) {
  $tried = New-Object System.Collections.ArrayList
  $r = Read-Primary $t
  [void]$tried.Add($r.Via)
  if (Test-ReadSane $r.Text) { return [pscustomobject]@{ Text = $r.Text; Via = $r.Via; Sane = $true; Tried = ($tried -join ',') } }

  Start-Sleep -Milliseconds 700
  $r2 = Read-Primary $t
  [void]$tried.Add($r2.Via + '+settle')
  if (Test-ReadSane $r2.Text) { return [pscustomobject]@{ Text = $r2.Text; Via = $r2.Via; Sane = $true; Tried = ($tried -join ',') } }

  $r3 = Read-Secondary $t
  [void]$tried.Add($r3.Via)
  $sane = Test-ReadSane $r3.Text
  return [pscustomobject]@{ Text = $r3.Text; Via = $r3.Via; Sane = $sane; Tried = ($tried -join ','); Bytes = (Show-Bytes $r3.Text) }
}

function Normalize([string]$s) {
  if ($null -eq $s) { return '' }
  return $s.Replace("`r`n", "`n").Replace("`r", "`n").TrimEnd("`n")
}

# Clear before every row, then verify it is empty: a row that starts on top of the
# previous row's text can never be graded against an absolute expectation.
function Clear-Field($t) {
  for ($i = 1; $i -le 3; $i++) {
    [void](Assert-Focus $t 'clear')
    [QAInput]::Combo(0x41, $false, $true, $false)    # Ctrl+A
    Start-Sleep -Milliseconds 70
    [QAInput]::Combo(0x2E, $false, $false, $false)   # Delete
    Start-Sleep -Milliseconds 90
    $r = Read-Primary $t
    if ($null -ne $r.Text -and (Normalize $r.Text) -ceq '') { return $true }
    Start-Sleep -Milliseconds 260
  }
  return $false
}

function Do-Action($t, [string]$a) {
  switch ($a) {
    'BS'      { [QAInput]::Combo(0x08, $false, $false, $false); Start-Sleep -Milliseconds 90 }
    'ENTER'   { [QAInput]::Combo(0x0D, $false, $false, $false); Start-Sleep -Milliseconds 150 }
    'TAB'     {
      [QAInput]::Combo(0x09, $false, $false, $false); Start-Sleep -Milliseconds 150
      if ($t.App -eq 'chrome') { $el = Get-EditElement $t.Hwnd; if ($el) { [void](Click-Element $el) } }
      if ($t.App -eq 'vscode') { Click-Center $t.Hwnd }
    }
    'ENABLE'  { [void](Send-Binding $script:Bind.Enable 'key'); Start-Sleep -Milliseconds 250 }
    'DISABLE' { [void](Send-Binding $script:Bind.Disable 'key'); Start-Sleep -Milliseconds 250 }
    'RESTORE' { [void](Send-Binding $script:Bind.Restore 'key'); Start-Sleep -Milliseconds 250 }
    default   { W "unknown action $a" }
  }
}

function Get-Verdict($c, [string]$act) {
  # The round's correctness rule, in one place. A row declared as a documented loss
  # is graded against its pinned acceptance text first (that is what makes L7 read as
  # QUALITY rather than as a silent PASS); any other row is PASS on the composed text
  # or on the literal raw keystrokes, and FAIL on anything else.
  if ($c.loss) {
    if ($act -ceq $c.lossExpected) { return 'QUALITY' }
    if ($act -ceq $c.composed) { return 'PASS' }
    if ($act -ceq $c.raw) { return 'PASS(raw)' }
    return 'FAIL'
  }
  if ($act -ceq $c.composed) { return 'PASS' }
  if ($act -ceq $c.raw) { return 'PASS(raw)' }
  return 'FAIL'
}

function Invoke-CaseOnce($t, $c, [int]$delay) {
  if (-not (Assert-Focus $t 'row-start')) { return $null }
  if (-not (Clear-Field $t)) {
    return [pscustomobject]@{ Text = ''; Via = 'clear'; Sane = $false; Tried = 'clear-field'; Cleared = $false }
  }
  foreach ($a in $c.pre) { Do-Action $t $a }
  if ($c.keys) {
    # F1/F2 are also application keys (Notepad Help, VS Code palette): re-take the
    # target after the pre-actions, never before them.
    [void](Assert-Focus $t 'before-keys')
    [QAInput]::TypeAscii($c.keys, $delay)
  }
  foreach ($a in $c.mid) { Do-Action $t $a }
  if ($c.keys2) {
    [void](Assert-Focus $t 'before-keys2')
    [QAInput]::TypeAscii($c.keys2, $delay)
  }
  Start-Sleep -Milliseconds (400 + $delay)
  [void](Assert-Focus $t 'before-read')
  $a = Read-Actual $t
  return [pscustomobject]@{
    Text = (Normalize ([string]$a.Text)); Via = $(if ($a.Via) { $a.Via } else { $t.Via })
    Sane = $a.Sane; Tried = $a.Tried; Bytes = $a.Bytes; Cleared = $true
  }
}

function Run-Case($t, $c) {
  # A row whose action needs an unbound chord cannot run: say so, do not grade it.
  $map = @{ enable = $script:Bind.Enable; disable = $script:Bind.Disable; restore_word = $script:Bind.Restore }
  $needed = New-Object System.Collections.ArrayList
  if ($c.pre -contains 'ENABLE') { [void]$needed.Add('enable') }
  if ($c.pre -contains 'DISABLE') { [void]$needed.Add('disable') }
  if ($c.mid -contains 'RESTORE') { [void]$needed.Add('restore_word') }
  foreach ($name in $needed) {
    if (-not $map[$name].Valid) {
      Row -App $t.App -Id $c.id -Keys $c.KeysLabel -Expected $c.composed -Actual ("not pressed: config hotkeys.{0} is {1}" -f $name, (Get-BindingDetail $map[$name])) -Raw $c.raw -Verdict 'SKIP(unbound)' -Via 'config binding' -Note 'the row needs a bound hotkey; nothing was typed or graded'
      return
    }
  }
  $r = Invoke-CaseOnce $t $c $script:KeyDelayMs
  if (-not $r) {
    Row -App $t.App -Id $c.id -Keys $c.KeysLabel -Expected $c.composed -Actual '(no focus)' -Raw $c.raw -Verdict 'FAIL' -Via 'none' -Note 'foreground guard: the target window could not be made foreground'
    return
  }
  if ($r.Cleared -eq $false) {
    Row -App $t.App -Id $c.id -Keys $c.KeysLabel -Expected $c.composed -Actual '(field did not clear)' -Raw $c.raw -Verdict 'READ-FAIL' -Via 'clear-field' -Note 'the field still held text after Ctrl+A Delete x3, so no absolute expectation can be graded'
    return
  }
  if (-not $r.Sane) {
    Row -App $t.App -Id $c.id -Keys $c.KeysLabel -Expected $c.composed -Actual ('(read-fail) ' + $(if ($r.Bytes) { $r.Bytes } else { '<empty>' })) -Raw $c.raw -Verdict 'READ-FAIL' -Via $r.Via -Note ("readers tried: {0}; U+FFFC or an empty field from every reader" -f $r.Tried)
    return
  }
  $v = Get-Verdict $c $r.Text
  $note = ''

  # A 55 ms row that fails is usually the hook's re-injection racing the read, not a
  # text defect (95 measured the same sequences clean at 120-700 ms). One retry at a
  # human speed keeps the gate honest without burning the owner's window.
  if ($v -eq 'FAIL') {
    $r2 = Invoke-CaseOnce $t $c 130
    if ($r2 -and $r2.Sane) {
      $v2 = Get-Verdict $c $r2.Text
      if ($v2 -ne 'FAIL') { $r = $r2; $v = $v2; $note = 'retried at 130 ms' }
    }
  }
  if ($v -eq 'PASS(raw)') { $note = Join-Note 'literal raw keystrokes (rule 4)' $note }
  elseif ($v -eq 'QUALITY' -or $v -eq 'FAIL') { $note = Join-Note $c.loss $note }
  Row -App $t.App -Id $c.id -Keys $c.KeysLabel -Expected $c.composed -Actual $r.Text -Raw $c.raw -Verdict $v -Via $r.Via -Note $note
}

function Join-Note([string]$a, [string]$b) {
  if ($a -and $b) { return "$a; $b" }
  return ($a + $b)
}

function Get-StateKeys([string]$method) {
  if ($method -eq 'telex') { return 'baajt' }
  return 'ba6t5'
}

function Get-State($t) {
  # Observer: type one word that only transforms when the engine is ON and the method
  # matches, so "ON" also proves which method the hook is using.
  if (-not (Assert-Focus $t 'state-probe')) { return 'NOFOCUS' }
  if (-not (Clear-Field $t)) { return 'NOFOCUS' }
  $keys = Get-StateKeys $script:Method
  [QAInput]::TypeAscii($keys, 90)
  Start-Sleep -Milliseconds 450
  $r = Read-Actual $t
  if (-not $r.Sane) { return 'READFAIL' }
  $txt = Normalize ([string]$r.Text)
  if ($txt -ceq $script:V_BAT) { return 'ON' }
  if ($txt -ceq $keys) { return 'OFF' }
  return "OTHER[$txt]"
}

function Ensure-On($t) {
  for ($i = 0; $i -lt 3; $i++) {
    $st = Get-State $t
    if ($st -eq 'ON') { return $true }
    [void](Send-Binding $script:Bind.Enable 'key')
    Start-Sleep -Milliseconds 450
  }
  return ((Get-State $t) -eq 'ON')
}

# --------------------------------------------------------------- corpus -----
# 35 rows per app: the 19 rows of 95-e2e-report.md A (VNI, the owner's method) and the
# 16-row Telex mirror (the same families plus the two documented losses and the
# forward-slash path). Every expectation is pinned offline with `vkb simulate` on this
# tree - -SelfCheck re-pins all of them - so a mismatch in an app is a transport loss.
# `method` on each row is what the script switches the engine to before typing it.
function Add-CorpusRow {
  param($List, [string]$Id, [string]$Method, [string[]]$Pre, [string]$Keys, [string[]]$Mid, [string]$Keys2,
    [string]$Composed, [string]$Raw, [string]$Label, [string]$Loss = '', [string]$LossExpected = '',
    [switch]$NoPin)
  [void]$List.Add([pscustomobject]@{
      id = $Id; method = $Method; pre = @($Pre); keys = $Keys; mid = @($Mid); keys2 = $Keys2
      composed = $Composed; raw = $Raw; KeysLabel = $Label; loss = $Loss; lossExpected = $LossExpected
      noPin = [bool]$NoPin
    })
}

function Get-Corpus {
  $BAT = $script:V_BAT
  $TAT = $script:V_TAT
  $rows = New-Object System.Collections.ArrayList

  # --- 95 A, VNI (owner's method) ---
  Add-CorpusRow $rows 'A1-bat-slash-tat'        'vni' @() 'ba6t5/ta8t1'  @() '' "$BAT/$TAT"          'ba6t5/ta8t1'  'ba6t5/ta8t1'
  Add-CorpusRow $rows 'A2-bat-dot-tat'          'vni' @() 'ba6t5.ta8t1'  @() '' "$BAT.$TAT"          'ba6t5.ta8t1'  'ba6t5.ta8t1'
  Add-CorpusRow $rows 'A3-bat-question-tat'     'vni' @() 'ba6t5?ta8t1'  @() '' ($BAT + '?' + $TAT)  'ba6t5?ta8t1'  'ba6t5?ta8t1'
  Add-CorpusRow $rows 'A4-bat-underscore-tat'   'vni' @() 'ba6t5_ta8t1'  @() '' ($BAT + '_' + $TAT)  'ba6t5_ta8t1'  'ba6t5_ta8t1'
  Add-CorpusRow $rows 'A5-windows-path'         'vni' @() 'C:\Users\nam' @() '' 'C:\Users\nam'       'C:\Users\nam' 'C:\Users\nam'
  Add-CorpusRow $rows 'A6-bat-space-tat'        'vni' @() 'ba6t5 ta8t1'  @() '' "$BAT $TAT"          'ba6t5 ta8t1'  'ba6t5 ta8t1'
  Add-CorpusRow $rows 'A7-nguyen-thu-ha'        'vni' @() 'Nguye6n4 Thu Ha2' @() '' $script:V_NGUYEN 'Nguye6n4 Thu Ha2' 'Nguye6n4 Thu Ha2'
  Add-CorpusRow $rows 'A8-url-tai'              'vni' @() 'https://vietkb.dev/ta3i' @() '' ('https://vietkb.dev/' + $script:V_TAI) 'https://vietkb.dev/ta3i' 'https://vietkb.dev/ta3i' `
    'raw-zone: the URL tail stays literal (spec A9 / L1, owner-accepted)' 'https://vietkb.dev/ta3i'
  Add-CorpusRow $rows 'A9-a-slash-b'            'vni' @() 'a/b' @() '' 'a/b' 'a/b' 'a/b'
  Add-CorpusRow $rows 'A10-2-slash-3'           'vni' @() '2/3' @() '' '2/3' '2/3' '2/3'
  Add-CorpusRow $rows 'A11-percent'             'vni' @() '100%/50%' @() '' '100%/50%' '100%/50%' '100%/50%'
  # BS after a toned word removes the final letter and keeps the tone: "bat" + nang.
  Add-CorpusRow $rows 'B1-bs-tone'              'vni' @() 'ba6t5' @('BS') '' $script:V_BA_NANG 'ba6t' 'ba6t5 + BS'
  Add-CorpusRow $rows 'B2-bs-horn'              'vni' @() 'ba6t' @('BS') '' $script:V_BA 'ba6' 'ba6t + BS'
  Add-CorpusRow $rows 'C1-restore-word'         'vni' @() 'ba6t5' @('RESTORE') '' 'ba6t5' 'ba6t5' 'ba6t5 + restore chord' -NoPin
  Add-CorpusRow $rows 'C2-macro-m01'            'vni' @() 'm01 ' @() '' $script:V_MUC1 'm01 ' 'm01 ' -NoPin
  Add-CorpusRow $rows 'D1-enter'                'vni' @() 'ba6t5' @('ENTER') 'ba6t5' "$BAT`n$BAT" "ba6t5`nba6t5" 'ba6t5 Enter ba6t5'
  Add-CorpusRow $rows 'D2-tab'                  'vni' @() 'ba6t5' @('TAB') 'ba6t5' "$BAT`t$BAT" "ba6t5`tba6t5" 'ba6t5 Tab ba6t5'
  Add-CorpusRow $rows 'E1-off-f2'               'vni' @('DISABLE') 'ba6t5' @() '' 'ba6t5' 'ba6t5' 'disable hotkey then ba6t5' -NoPin
  Add-CorpusRow $rows 'E2-on-f1'                'vni' @('ENABLE') 'ba6t5' @() '' $BAT 'ba6t5' 'enable hotkey then ba6t5' -NoPin

  # --- Telex mirror + the pinned losses ---
  Add-CorpusRow $rows 'T1-telex-bat-slash-tat'      'telex' @() 'baajt/tawts'  @() '' "$BAT/$TAT"      'baajt/tawts'  'baajt/tawts'
  Add-CorpusRow $rows 'T2-telex-bat-dot-tat'        'telex' @() 'baajt.tawts'  @() '' "$BAT.$TAT"      'baajt.tawts'  'baajt.tawts'
  Add-CorpusRow $rows 'T3-telex-bat-question-tat'   'telex' @() 'baajt?tawts'  @() '' ($BAT + '?' + $TAT) 'baajt?tawts'  'baajt?tawts'
  Add-CorpusRow $rows 'T4-telex-bat-underscore-tat' 'telex' @() 'baajt_tawts'  @() '' ($BAT + '_' + $TAT) 'baajt_tawts' 'baajt_tawts'
  Add-CorpusRow $rows 'T5-telex-bat-space-tat'      'telex' @() 'baajt tawts'  @() '' "$BAT $TAT"      'baajt tawts'  'baajt tawts'
  Add-CorpusRow $rows 'T6-telex-nguyen-thu-ha'      'telex' @() 'Nguyeexn Thu Haf' @() '' $script:V_NGUYEN 'Nguyeexn Thu Haf' 'Nguyeexn Thu Haf'
  Add-CorpusRow $rows 'T7-telex-path-forward'       'telex' @() 'C:/Users/nam' @() '' 'C:/Users/nam'   'C:/Users/nam' 'C:/Users/nam'
  Add-CorpusRow $rows 'T8-telex-a-slash-b'          'telex' @() 'a/b' @() '' 'a/b' 'a/b' 'a/b'
  Add-CorpusRow $rows 'T9-telex-2-slash-3'          'telex' @() '2/3' @() '' '2/3' '2/3' '2/3'
  Add-CorpusRow $rows 'T10-telex-percent'           'telex' @() '100%/50%' @() '' '100%/50%' '100%/50%' '100%/50%'
  Add-CorpusRow $rows 'T11-telex-url'               'telex' @() 'https://vietkb.dev/taif' @() '' ('https://vietkb.dev/' + $script:V_TAI) 'https://vietkb.dev/taif' 'https://vietkb.dev/taif' `
    'raw-zone: the URL tail stays literal (spec A9 / L1, owner-accepted)' 'https://vietkb.dev/taif'
  Add-CorpusRow $rows 'T12-telex-l7-digit-run'      'telex' @() '10/users' @() '' ('10/' + $script:V_UE) '10/users' '10/users' `
    'L7: a Telex digit run ends, so the opener composes (spec A9 6b, owner-accepted)' ('10/' + $script:V_UE)
  Add-CorpusRow $rows 'T13-telex-bs-tone'           'telex' @() 'baajt' @('BS') '' $script:V_BA_NANG 'baaj' 'baajt + BS'
  Add-CorpusRow $rows 'T14-telex-macro-m01'         'telex' @() 'm01 ' @() '' $script:V_MUC1 'm01 ' 'm01 ' -NoPin
  Add-CorpusRow $rows 'T15-telex-double-slash'      'telex' @() 'baajt//tawts' @() '' "$BAT//$TAT"    'baajt//tawts' 'baajt//tawts'
  return $rows
}

# Replays a corpus row through the CLI engine (no GUI, no hook) so -SelfCheck can prove
# every expectation against the tree under test.
function Get-PinnedText($c, [string]$method) {
  $bs = [string][char]0x08
  $cr = [string][char]0x0D
  $tab = [string][char]0x09
  $keys = $c.keys
  foreach ($a in $c.mid) {
    if ($a -eq 'BS') { $keys += $bs }
    elseif ($a -eq 'ENTER') { $keys += $cr }
    elseif ($a -eq 'TAB') { $keys += $tab }
  }
  if ($c.keys2) { $keys += $c.keys2 }
  $out = & $script:SimExe simulate --method $method $keys 2>&1
  return (Normalize (($out | Out-String)))
}

# ---------------------------------------------------------- method switch ----
# Supported surface: the config file, then the app reloads it on start. No IPC, no
# hidden window, nothing the product does not already do for a user who edits it.
function Confirm-Method([string]$method, $observer) {
  $cfg = Get-ConfigValue 'input' 'method'
  if ($cfg -ne $method) { return "CONFIG[$cfg]" }
  $saved = $script:Method
  $script:Method = $method
  $st = Get-State $observer
  $script:Method = $saved
  if ($st -eq 'ON') { return 'OK' }
  return $st
}

function Switch-Method([string]$method, $observer) {
  Section ("METHOD SWITCH -> {0} (config input.method + app reload)" -f $method)
  $was = $script:Method
  $quit = Quit-App
  if (-not $quit) {
    # The only process this may touch is the one this run started, under target\.
    W ("method switch: pid={0} did not exit through its button; stopping the test instance it started" -f $script:TestPid)
    [void](Stop-Owned $script:TestPid 'method switch: test instance did not quit via its own button')
  }
  Start-Sleep -Milliseconds 500
  $id = ("H1-method-switch-{0}" -f $method)
  if (Get-Process -Id $script:TestPid -ErrorAction SilentlyContinue) {
    Row -App 'harness' -Id $id -Keys 'config + restart' -Expected 'the test instance is gone' -Actual ("pid={0} still alive (not killed: the rule needs a logged, owned stop)" -f $script:TestPid) -Raw '(pid+path)' -Verdict 'FAIL' -Via 'pid+path'
    return $false
  }
  if (-not (Write-ConfigMethod $method)) {
    Row -App 'harness' -Id $id -Keys 'config + restart' -Expected "config method=$method" -Actual 'config write/read-back failed' -Raw '(config)' -Verdict 'FAIL' -Via 'config file'
    return $false
  }
  $appProc = Start-AppVerified
  if (-not $appProc) {
    if ($script:BlockVerdict) { return $false }
    Row -App 'harness' -Id $id -Keys 'config + restart' -Expected 'the build under test is running' -Actual 'no process under target\ after 25 s' -Raw '(pid+path)' -Verdict 'FAIL' -Via 'pid+path'
    return $false
  }
  Start-Sleep -Milliseconds 900
  $probe = Confirm-Method $method $observer
  $ok = ($probe -eq 'OK')
  if ($ok) { $script:Method = $method }
  Row -App 'harness' -Id $id -Keys 'config input.method then restart' `
    -Expected ("engine {0}: {1} -> {2}" -f $method, (Get-StateKeys $method), $script:V_BAT) `
    -Actual ("status read-back {0} (config method={1})" -f $probe, (Get-ConfigValue 'input' 'method')) `
    -Raw '(config + typed probe)' -Verdict $(if ($ok) { 'PASS' } else { 'FAIL' }) -Via 'config file + typed probe'
  if (-not $ok) { $script:Method = $was }
  return $ok
}

# ------------------------------------------------------------- settings -----
function Get-SettingsWindow([int]$procId, [switch]$AllowUntitled) {
  $best = $null
  foreach ($w in (Get-AppWindows $procId)) {
    if ($w.Title -ceq $script:WinSettings) {
      if ($w.Visible) { return $w }
      if (-not $best) { $best = $w }
      continue
    }
    if ($AllowUntitled -and -not $best -and $w.Title -and $w.Class -match 'Tauri|WebView|Chrome_WidgetWin') { $best = $w }
  }
  return $best
}

function Open-SettingsWindow([int]$waitSec = 5) {
  # Find the real top-level window by pid + class, not by a title guess on a stale
  # handle: the product destroys this window on close and rebuilds it on demand, so a
  # chord that lands during the rebuild gets nothing.
  $procId = Get-VietkbPid
  if ($procId -eq 0) { return $null }
  $chord = $script:Bind.Open
  for ($attempt = 1; $attempt -le 3; $attempt++) {
    $w = Get-SettingsWindow $procId
    if ($w) { break }
    if ($chord.Valid) {
      [void](Send-Binding $chord 'key')
    } else {
      W ("settings: config hotkeys.open_settings is {0}; no chord to press" -f (Get-BindingDetail $chord))
      break
    }
    $end = (Get-Date).AddSeconds($waitSec)
    while ((Get-Date) -lt $end) {
      $w = Get-SettingsWindow $procId
      if ($w) { break }
      Start-Sleep -Milliseconds 250
    }
    if ($w) {
      if ($attempt -gt 1) { W ("settings: opened on chord attempt {0}" -f $attempt) }
      break
    }
    # Built but not shown (the product shows it after centring): bring it forward.
    $hidden = Get-SettingsWindow $procId -AllowUntitled
    if ($hidden) {
      W ("settings: found window hwnd={0} class={1} visible={2}; bringing it forward" -f $hidden.Hwnd, $hidden.Class, $hidden.Visible)
      [void][VkbWin]::Front($hidden.Hwnd)
      Start-Sleep -Milliseconds 400
      $w = Get-SettingsWindow $procId
      if ($w) { break }
    }
    W ("settings: chord `"{0}`" attempt {1} produced no window" -f $chord, $attempt)
  }
  if (-not $w) { return $null }
  [void][VkbWin]::Front($w.Hwnd)
  Start-Sleep -Milliseconds 250
  return (Get-SettingsWindow $procId)
}

function Close-SettingsWindow {
  $procId = Get-VietkbPid
  if ($procId -eq 0) { return $false }
  $w = Get-SettingsWindow $procId
  if (-not $w) { return $true }
  [void][VkbWin]::Front($w.Hwnd)
  Start-Sleep -Milliseconds 120
  [QAInput]::Combo(0x1B, $false, $false, $false)   # Esc
  Start-Sleep -Milliseconds 500
  if (-not (Get-SettingsWindow $procId)) { return $true }
  [void](Invoke-Named $w.Hwnd $script:BtnClose 'Button')
  Start-Sleep -Milliseconds 500
  if (-not (Get-SettingsWindow $procId)) { return $true }
  [VkbWin]::Close($w.Hwnd)
  Start-Sleep -Milliseconds 600
  return (-not [bool](Get-SettingsWindow $procId))
}

# ------------------------------------------------------------- chords -------
# Each key is its own SendInput call so the hook sees the transitions of a real
# keyboard, including the explicit release order under test.
if (-not ('HostKey' -as [type])) {
  Add-Type @'
using System;
using System.Runtime.InteropServices;
using System.Threading;

public static class HostKey {
  [DllImport("user32.dll")] static extern uint SendInput(uint n, INPUT[] p, int cb);
  [DllImport("user32.dll")] static extern uint MapVirtualKeyW(uint code, uint type);

  [StructLayout(LayoutKind.Sequential)] public struct KEYBDINPUT { public ushort wVk; public ushort wScan; public uint dwFlags; public uint time; public IntPtr extra; }
  [StructLayout(LayoutKind.Sequential)] public struct MOUSEINPUT { public int dx; public int dy; public uint data; public uint flags; public uint time; public IntPtr extra; }
  [StructLayout(LayoutKind.Sequential)] public struct HARDINPUT { public uint msg; public ushort p1; public ushort p2; }
  [StructLayout(LayoutKind.Explicit)] public struct InputUnion {
    [FieldOffset(0)] public MOUSEINPUT mi;
    [FieldOffset(0)] public KEYBDINPUT ki;
    [FieldOffset(0)] public HARDINPUT hi;
  }
  [StructLayout(LayoutKind.Sequential)] public struct INPUT { public uint type; public InputUnion u; }

  static void Ev(ushort vk, bool up) {
    INPUT[] a = new INPUT[1];
    a[0].type = 1;
    a[0].u.ki.wVk = vk;
    a[0].u.ki.wScan = (ushort)MapVirtualKeyW(vk, 0);
    a[0].u.ki.dwFlags = up ? 2u : 0u;
    SendInput(1, a, Marshal.SizeOf(typeof(INPUT)));
  }
  public static void Down(ushort vk) { Ev(vk, false); Thread.Sleep(25); }
  public static void Up(ushort vk) { Ev(vk, true); Thread.Sleep(25); }
}
'@
}

function Run-Chords($t) {
  $BAT = $script:V_BAT
  $toggle = $script:Bind.Toggle
  $restore = $script:Bind.Restore
  $open = $script:Bind.Open
  $toggleLabel = Get-BindingLabel $toggle
  $restoreLabel = Get-BindingLabel $restore
  $openLabel = Get-BindingLabel $open

  # --- CH1: the toggle chord flips exactly once, whatever the release order ---
  if (-not $toggle.Valid) {
    foreach ($o in @('both')) {
      Row -App 'chord' -Id "CH1-toggle-once/release-$o" -Keys $toggleLabel -Expected 'one flip per press' -Actual ("not pressed: config hotkeys.toggle is {0}" -f (Get-BindingDetail $toggle)) -Raw '(config binding)' -Verdict 'SKIP(unbound)' -Via 'config binding' -Note 'no toggle chord is bound, so the route was not exercised'
    }
  } else {
    foreach ($o in (Get-ReleaseOrders $toggle)) {
      $before = Get-State $t
      [void](Send-Binding $toggle $o)
      Start-Sleep -Milliseconds 550
      $after = Get-State $t
      $flipped = ($before -ne $after) -and ($before -in @('ON', 'OFF')) -and ($after -in @('ON', 'OFF'))
      $v = if ($flipped) { 'PASS' } else { if ($before -eq 'READFAIL' -or $after -eq 'READFAIL') { 'READ-FAIL' } else { 'FAIL' } }
      Row -App 'chord' -Id "CH1-toggle-once/release-$o" -Keys ("{0}, release {1}" -f $toggleLabel, $o) -Expected "one flip of $before" -Actual "state $before -> $after" -Raw ('(config hotkeys.toggle={0}, mode={1})' -f $toggle.Raw, $script:Bind.ToggleMode) -Verdict $v -Via 'typed probe'
      if (-not $flipped -and $after -in @('ON', 'OFF')) {
        [void](Send-Binding $toggle 'both'); Start-Sleep -Milliseconds 450
      }
    }
  }

  # --- CH2: the restore chord puts the raw keystrokes back and must not toggle ---
  if (-not $restore.Valid) {
    Row -App 'chord' -Id 'CH2-restore-word/release-key' -Keys $restoreLabel -Expected "$BAT -> raw keystrokes" -Actual ("not pressed: config hotkeys.restore_word is {0}" -f (Get-BindingDetail $restore)) -Raw '(config binding)' -Verdict 'SKIP(unbound)' -Via 'config binding' -Note 'no restore chord is bound, so the route was not exercised'
  } else {
    $typedKeys = Get-StateKeys $script:Method
    foreach ($o in (Get-ReleaseOrders $restore)) {
      $st = Get-State $t
      [void](Assert-Focus $t 'chord-restore')
      if (-not (Clear-Field $t)) { Row -App 'chord' -Id "CH2-restore-word/release-$o" -Keys ("{0} then {1}" -f $typedKeys, $restoreLabel) -Expected "$BAT -> $typedKeys" -Actual '(field did not clear)' -Raw $typedKeys -Verdict 'READ-FAIL' -Via 'clear-field'; continue }
      [QAInput]::TypeAscii($typedKeys, 90)
      Start-Sleep -Milliseconds 420
      $rTyped = Read-Actual $t
      $typed = Normalize ([string]$rTyped.Text)
      [void](Send-Binding $restore $o)
      Start-Sleep -Milliseconds 650
      $rAct = Read-Actual $t
      $act = Normalize ([string]$rAct.Text)
      $stAfter = Get-State $t
      $ok = ($typed -ceq $BAT) -and ($act -ceq $typedKeys) -and ($stAfter -eq $st)
      $v = if ($ok) { 'PASS' } else { if (-not $rAct.Sane) { 'READ-FAIL' } else { 'FAIL' } }
      $via = if ($rAct.Sane) { 'typed probe' } else { $rAct.Via }
      $shown = if ($rAct.Sane) { $act } else { '(read-fail) ' + (Show-Bytes $rAct.Text) }
      Row -App 'chord' -Id "CH2-restore-word/release-$o" -Keys ("{0} then {1}, release {2}" -f $typedKeys, $restoreLabel, $o) -Expected "$BAT -> $typedKeys, state stays $st" -Actual "[$typed] -> [$shown], state $stAfter" -Raw $typedKeys -Verdict $v -Via $via
    }
  }

  # --- CH3: the settings chord, in each release order the platform promises ---
  if (-not $open.Valid) {
    Row -App 'chord' -Id 'CH3-open-settings/release-key' -Keys $openLabel -Expected "$($script:WinSettings) window" -Actual ("not pressed: config hotkeys.open_settings is {0}" -f (Get-BindingDetail $open)) -Raw '(config binding)' -Verdict 'SKIP(unbound)' -Via 'config binding' -Note 'no settings chord is bound, so the window route was not exercised'
  } else {
    foreach ($o in (Get-ReleaseOrders $open)) {
      [void](Close-SettingsWindow)
      Start-Sleep -Milliseconds 300
      [void](Send-Binding $open $o)
      $w = $null
      $end = (Get-Date).AddSeconds(6)
      while ((Get-Date) -lt $end -and -not $w) { $w = Get-SettingsWindow (Get-VietkbPid); if (-not $w) { Start-Sleep -Milliseconds 250 } }
      $v = if ($w) { 'PASS' } else { 'FAIL' }
      Row -App 'chord' -Id "CH3-open-settings/release-$o" -Keys ("{0} (config open_settings, release {1})" -f $openLabel, $o) -Expected "$($script:WinSettings) window" -Actual $(if ($w) { 'window opened (pid+class scan)' } else { 'no window' }) -Raw ('(open_settings={0})' -f $open.Raw) -Verdict $v -Via 'top-level window scan'
      if ($w) { [void][VkbWin]::Front($w.Hwnd); Start-Sleep -Milliseconds 200 }
    }
  }

  [void](Close-SettingsWindow)
  Note ("the round brief names its own settings key; the run pressed only the binding in the live config (open_settings=`"{0}`"). A mismatch between the brief and the shipped default is a contract question for the Lead, not a pass/fail here." -f $open.Raw)
}

# -------------------------------------------------------------- toggles -----
function SoundErrorCount {
  if (-not (Test-Path $script:AppLog)) { return -1 }
  return @(Select-String -Path $script:AppLog -Pattern 'toggle sound could not be played' -AllMatches -ErrorAction SilentlyContinue).Count
}

function Get-TrayIcon {
  foreach ($h in [VkbWin]::All()) {
    if (-not [VkbWin]::Visible($h)) { continue }
    try {
      $root = [System.Windows.Automation.AutomationElement]::FromHandle($h)
      if (-not $root) { continue }
      foreach ($e in $root.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)) {
        try {
          $r = $e.Current.BoundingRectangle
          if ($e.Current.Name -like 'VietKB *' -and $r.Width -gt 5) { return $e }
        } catch {}
      }
    } catch {}
  }
  return $null
}

if (-not ('TrayKey' -as [type])) {
  Add-Type -Namespace TrayKey -Name K -MemberDefinition @'
[StructLayout(LayoutKind.Sequential)] public struct KEYBDINPUT { public ushort wVk; public ushort wScan; public uint dwFlags; public uint time; public IntPtr extra; }
[StructLayout(LayoutKind.Sequential)] public struct MOUSEINPUT { public int dx; public int dy; public uint data; public uint flags; public uint time; public IntPtr extra; }
[StructLayout(LayoutKind.Sequential)] public struct HARDINPUT { public uint msg; public ushort p1; public ushort p2; }
[StructLayout(LayoutKind.Explicit)] public struct InputUnion { [FieldOffset(0)] public MOUSEINPUT mi; [FieldOffset(0)] public KEYBDINPUT ki; [FieldOffset(0)] public HARDINPUT hi; }
[StructLayout(LayoutKind.Sequential)] public struct INPUT { public uint type; public InputUnion u; }
[DllImport("user32.dll")] public static extern uint SendInput(uint n, INPUT[] p, int cb);
[DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
[DllImport("user32.dll")] public static extern void mouse_event(uint flags, int dx, int dy, uint data, IntPtr extra);
public static void Tap(ushort vk, bool win) {
  INPUT[] a = new INPUT[4]; int i = 0;
  if (win) { a[i].type = 1; a[i].u.ki.wVk = 0x5B; i++; }
  a[i].type = 1; a[i].u.ki.wVk = vk; i++;
  a[i].type = 1; a[i].u.ki.wVk = vk; a[i].u.ki.dwFlags = 2; i++;
  if (win) { a[i].type = 1; a[i].u.ki.wVk = 0x5B; a[i].u.ki.dwFlags = 2; i++; }
  SendInput((uint)i, a, Marshal.SizeOf(typeof(INPUT)));
}
'@
}

function Open-TrayFlyout {
  [TrayKey.K]::Tap(0x42, $true)   # Win+B: notification area
  Start-Sleep -Milliseconds 600
  for ($i = 0; $i -lt 8; $i++) {
    $n = ''
    try { $n = [System.Windows.Automation.AutomationElement]::FocusedElement.Current.Name } catch {}
    if ($n -match ('Hidden Icons|' + [regex]::Escape($script:OverflowVi) + '|' + [regex]::Escape($script:NotifyVi))) { break }
    [QAInput]::Combo(0x27, $false, $false, $false)
    Start-Sleep -Milliseconds 250
  }
  [QAInput]::Combo(0x0D, $false, $false, $false)
  Start-Sleep -Milliseconds 1000
  # Wait for the icon element itself, not for a fixed delay.
  for ($i = 0; $i -lt 10; $i++) {
    if (Get-TrayIcon) { return $true }
    Start-Sleep -Milliseconds 200
  }
  return $false
}

function Invoke-TrayClick($icon, [switch]$Right) {
  if (-not $icon) { return $false }
  $r = $icon.Current.BoundingRectangle
  [void][TrayKey.K]::SetCursorPos([int]($r.X + $r.Width / 2), [int]($r.Y + $r.Height / 2))
  Start-Sleep -Milliseconds 180
  if ($Right) {
    [TrayKey.K]::mouse_event(0x0008, 0, 0, 0, [IntPtr]::Zero)   # RIGHTDOWN
    [TrayKey.K]::mouse_event(0x0010, 0, 0, 0, [IntPtr]::Zero)   # RIGHTUP
  } else {
    [TrayKey.K]::mouse_event(0x0002, 0, 0, 0, [IntPtr]::Zero)   # LEFT DOWN
    [TrayKey.K]::mouse_event(0x0004, 0, 0, 0, [IntPtr]::Zero)   # LEFT UP
  }
  Start-Sleep -Milliseconds 900
  return $true
}

# The tray menu is a classic Win32 popup (class #32768) built by the menu crate: it is
# its own top-level window, not part of the flyout. Find it by class, then read its
# items through UIA (which proxies the menu's MSAA tree) with a raw MSAA walk behind it.
function Wait-TrayMenu([int]$timeoutMs = 2500) {
  $end = (Get-Date).AddMilliseconds($timeoutMs)
  while ((Get-Date) -lt $end) {
    foreach ($h in [VkbWin]::All()) {
      if (-not [VkbWin]::Visible($h)) { continue }
      $cls = [VkbWin]::Class($h)
      if ($cls -eq '#32768' -or $cls -eq 'ForegroundMenu' -or $cls -eq 'VkbTrayMenu') {
        return [pscustomobject]@{ Hwnd = $h; Class = $cls }
      }
    }
    Start-Sleep -Milliseconds 150
  }
  return $null
}

function Read-MenuItems([IntPtr]$hMenu) {
  $names = New-Object System.Collections.ArrayList
  $states = New-Object System.Collections.ArrayList
  try {
    $root = [System.Windows.Automation.AutomationElement]::FromHandle($hMenu)
    if ($root) {
      $all = $root.FindAll([System.Windows.Automation.TreeScope]::Subtree, [System.Windows.Automation.Condition]::TrueCondition)
      foreach ($e in $all) {
        try {
          if ($e.Current.ControlType.ProgrammaticName -ne 'ControlType.MenuItem') { continue }
          $n = $e.Current.Name
          if (-not $n) { continue }
          [void]$names.Add($n)
          $s = ''
          try { $tp = $e.GetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern); if ($tp) { $s = $tp.Current.ToggleState.ToString() } } catch {}
          [void]$states.Add(('{0}={1}' -f $n, $s))
        } catch {}
      }
    }
  } catch {}
  if ($names.Count -eq 0) {
    # MSAA straight from oleacc: the menu's IAccessible children carry the item text.
    try {
      $acc = [VkbWin]::Acc($hMenu)
      if ($acc) {
        $count = 0
        try { $count = [int]$acc.accChildCount } catch { $count = 0 }
        for ($i = 0; $i -lt $count; $i++) {
          $n = ''
          try { $n = [string]$acc.get_accName($i) } catch { try { $n = [string]$acc.accName($i) } catch { $n = '' } }
          if ($n) { [void]$names.Add($n); [void]$states.Add(('{0}=' -f $n)) }
        }
      }
    } catch {}
  }
  return [pscustomobject]@{ Names = @($names); States = @($states) }
}

function Run-Toggles($t) {
  $snd0 = SoundErrorCount
  $toggle = $script:Bind.Toggle
  $enable = $script:Bind.Enable
  $disable = $script:Bind.Disable
  $toggleLabel = Get-BindingLabel $toggle
  $enableLabel = Get-BindingLabel $enable
  $disableLabel = Get-BindingLabel $disable

  # route: the two bound enable/disable keys
  if (-not ($enable.Valid -and $disable.Valid)) {
    Row -App 'toggle' -Id 'RT1-disable-then-enable' -Keys ("{0}, {1}" -f $disableLabel, $enableLabel) -Expected 'OFF then ON' -Actual ("not pressed: enable={0}; disable={1}" -f (Get-BindingDetail $enable), (Get-BindingDetail $disable)) -Raw '(config binding)' -Verdict 'SKIP(unbound)' -Via 'config binding' -Note 'a missing enable/disable key leaves the route unexercised'
  } else {
    $before = Get-State $t
    [void](Send-Binding $disable 'key'); Start-Sleep -Milliseconds 500
    $off = Get-State $t
    [void](Send-Binding $enable 'key'); Start-Sleep -Milliseconds 500
    $on = Get-State $t
    $v = if ($off -eq 'OFF' -and $on -eq 'ON') { 'PASS' } elseif ($off -eq 'READFAIL' -or $on -eq 'READFAIL' -or $before -eq 'READFAIL') { 'READ-FAIL' } else { 'FAIL' }
    Row -App 'toggle' -Id 'RT1-disable-then-enable' -Keys ("{0}, {1}" -f $disableLabel, $enableLabel) -Expected "$before -> OFF -> ON" -Actual "$before -> $off -> $on" -Raw ('(enable={0} disable={1})' -f $enable.Raw, $disable.Raw) -Verdict $v -Via 'typed probe'
  }

  # route: the toggle chord. A partial press must not fire; each full press must flip
  # exactly once. Both facts come from the bound chord, never from a written-in value.
  if (-not $toggle.Valid) {
    Row -App 'toggle' -Id 'RT2-toggle-partial-then-full' -Keys $toggleLabel -Expected 'partial press does nothing, full press flips' -Actual ("not pressed: config hotkeys.toggle is {0}" -f (Get-BindingDetail $toggle)) -Raw '(config binding)' -Verdict 'SKIP(unbound)' -Via 'config binding' -Note 'no toggle chord is bound, so the route was not exercised'
  } else {
    $st0 = Get-State $t
    $partialNote = ''
    $stPartial = $st0
    if ($toggle.Vk -ne 0) {
      # every modifier of the chord, without the key
      [void](Send-Binding $toggle 'key' -ModifiersOnly)
      Start-Sleep -Milliseconds 650
      $stPartial = Get-State $t
    } elseif (([int]$toggle.Ctrl + [int]$toggle.Alt + [int]$toggle.Shift + [int]$toggle.Win) -gt 1) {
      # modifier-only chord: one modifier alone must not fire
      $firstMod = if ($toggle.Ctrl) { 0x11 } elseif ($toggle.Alt) { 0x12 } elseif ($toggle.Shift) { 0x10 } else { 0x5B }
      Send-SingleModifier $firstMod
      Start-Sleep -Milliseconds 650
      $stPartial = Get-State $t
    } else {
      $partialNote = 'single-modifier chord: no partial press exists'
    }
    [void](Send-Binding $toggle 'both'); Start-Sleep -Milliseconds 650
    $tap1 = Get-State $t
    [void](Send-Binding $toggle 'both'); Start-Sleep -Milliseconds 650
    $tap2 = Get-State $t
    $ok = ($stPartial -eq $st0) -and ($tap1 -ne $st0) -and ($tap2 -eq $st0)
    $v = if ($ok) { 'PASS' } elseif (@($st0, $stPartial, $tap1, $tap2) -contains 'READFAIL') { 'READ-FAIL' } else { 'FAIL' }
    $note = if ($partialNote) { $partialNote }
    Row -App 'toggle' -Id 'RT2-toggle-partial-then-full' -Keys ("{0} partial, then x2" -f $toggleLabel) -Expected "$st0 -> !$st0 -> $st0 (partial: $st0)" -Actual "$st0 -> $tap1 -> $tap2 (partial: $stPartial)" -Raw ('(toggle={0} mode={1})' -f $toggle.Raw, $script:Bind.ToggleMode) -Verdict $v -Via 'typed probe' -Note $note
  }

  # route: UI switch. Locate the top-level window by pid, bring it forward, then flip
  # the switch by its accessible name and read the state back.
  $w = Open-SettingsWindow
  if (-not $w) {
    Row -App 'toggle' -Id 'RT3-ui-switch' -Keys 'open panel, switch by name' -Expected 'TogglePattern flips the state, engine follows' -Actual 'no settings window after 3 chord attempts + hidden-window scan' -Raw '(ui)' -Verdict 'FAIL' -Via 'pid+class window scan' -Note 'Open-SettingsWindow tried the config chord 3x and enumerated the app pid windows including invisible ones'
  } else {
    $sw = $null
    for ($i = 0; $i -lt 12; $i++) {
      $sw = Find-ElementByName $w.Hwnd $script:SwEnabled ''
      if (-not $sw) { $sw = Find-ElementByName $w.Hwnd $script:SwEnabled 'Button|CheckBox|Custom' }
      if ($sw) { break }
      Start-Sleep -Milliseconds 400
    }
    if (-not $sw) {
      Row -App 'toggle' -Id 'RT3-ui-switch' -Keys $script:SwEnabled -Expected 'TogglePattern flips the state' -Actual 'switch not found by accessible name in the UIA tree' -Raw '(ui)' -Verdict 'FAIL' -Via "UIA in hwnd=$($w.Hwnd)"
    } else {
      $mode = 'TogglePattern'
      $tp = $null
      try { $tp = $sw.GetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern) } catch { $tp = $null }
      if (-not $tp) {
        $mode = 'InvokePattern'
        try {
          $ip = $sw.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern)
          if (-not $ip) { $mode = 'click' }
        } catch { $mode = 'click' }
      }
      $readState = {
        try {
          if ($tp) { return [string]$tp.Current.ToggleState }
          $hit = Find-ElementByName $w.Hwnd $script:SwEnabled ''
          if ($hit) {
            try { $t2 = $hit.GetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern); if ($t2) { return [string]$t2.Current.ToggleState } } catch {}
          }
        } catch {}
        return 'unknown'
      }
      # The flip is also defensive: a WebView re-render can invalidate the element the
      # patterns were fetched from, so an exception falls through to InvokePattern, then
      # to a real click on the rect.
      $flipSwitch = {
        try { if ($tp) { $tp.Toggle(); return 'TogglePattern' } } catch {}
        $done = $false
        try { $ip2 = $sw.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern); if ($ip2) { $ip2.Invoke(); $done = $true } } catch {}
        if (-not $done) { [void](Click-Element $sw) }
        return 'Invoke/click'
      }
      $s1 = & $readState
      $st1 = Get-State $t
      $used = & $flipSwitch
      Start-Sleep -Milliseconds 900
      $s2 = & $readState
      $st2 = Get-State $t
      [void](& $flipSwitch)
      Start-Sleep -Milliseconds 900
      $s3 = & $readState
      $st3 = Get-State $t
      $flipped = ($st1 -ne $st2) -and ($st2 -ne $st3) -and ($st1 -in @('ON', 'OFF'))
      $elemReadable = ($s1 -ne 'unknown')
      $elemFlipped = $elemReadable -and ($s1 -ne $s2) -and ($s2 -ne $s3)
      # When the switch exposes a state it must agree with the engine. When it exposes
      # none, the engine probe is the read-back (that is the whole point of the row).
      $ok = $flipped -and ($elemFlipped -or -not $elemReadable)
      $v = if ($ok) { 'PASS' } elseif (@($st1, $st2, $st3) -contains 'READFAIL') { 'READ-FAIL' } else { 'FAIL' }
      Row -App 'toggle' -Id 'RT3-ui-switch' -Keys $script:SwEnabled -Expected 'switch state and engine state flip together' -Actual ("element {0}->{1}->{2}; engine {3} -> {4} -> {5}" -f $s1, $s2, $s3, $st1, $st2, $st3) -Raw '(ui)' -Verdict $v -Via ("$used + typed probe; hwnd=" + $w.Hwnd)
    }
    [void](Close-SettingsWindow)
  }

  # route: tray. Two facts, two rows: the left click is the affordance (graded on the
  # engine), the right-click menu is the surface (graded on its readable item text).
  [void](Open-TrayFlyout)
  $icon = Get-TrayIcon
  if (-not $icon) {
    Row -App 'toggle' -Id 'RT4-tray-left-click' -Keys 'tray left click' -Expected 'engine state flips' -Actual 'VietKB icon not found in the overflow flyout' -Raw '(tray)' -Verdict 'FAIL' -Via 'UIA'
  } else {
    $stA = Get-State $t
    [void](Invoke-TrayClick $icon)
    $stB = Get-State $t
    if ($stA -eq $stB) {
      # One retry: a click that lands while the flyout is closing is swallowed.
      [void](Open-TrayFlyout)
      $icon2 = Get-TrayIcon
      if ($icon2) { [void](Invoke-TrayClick $icon2); $stB = Get-State $t }
    }
    $ok = ($stA -ne $stB) -and ($stA -in @('ON', 'OFF')) -and ($stB -in @('ON', 'OFF'))
    $v = if ($ok) { 'PASS' } elseif (@($stA, $stB) -contains 'READFAIL') { 'READ-FAIL' } else { 'FAIL' }
    Row -App 'toggle' -Id 'RT4-tray-left-click' -Keys 'tray left click' -Expected "$stA -> !$stA" -Actual "engine $stA -> $stB" -Raw '(tray)' -Verdict $v -Via 'UIA icon rect + typed probe'

    # Right-click opens the menu: read the item text, do not infer it from the tooltip.
    $menu = $null
    for ($i = 0; $i -lt 2 -and -not $menu; $i++) {
      if ($i -gt 0) { [void](Open-TrayFlyout) }
      $iconM = Get-TrayIcon
      if (-not $iconM) { break }
      [void](Invoke-TrayClick $iconM -Right)
      $menu = Wait-TrayMenu 2500
    }
    if (-not $menu) {
      Row -App 'toggle' -Id 'RT4-tray-menu' -Keys 'tray right click' -Expected 'menu window (#32768) with readable items' -Actual 'no popup menu window appeared' -Raw '(tray)' -Verdict 'MANUAL' -Via 'top-level window scan' -Note 'OWNER, ~20 s: right-click the VietKB tray icon and confirm the menu reads "Bat/tat tieng Viet" (checked = ON), "Kieu go" and "Thoat VietKB". The harness could not find or read the popup on this build.'
    } else {
      $items = Read-MenuItems $menu.Hwnd
      [QAInput]::Combo(0x1B, $false, $false, $false)   # Esc: dismiss the menu
      Start-Sleep -Milliseconds 400
      $hasToggle = @($items.Names | Where-Object { $_ -eq $script:TrayToggle }).Count -gt 0
      $hasQuit = @($items.Names | Where-Object { $_ -eq $script:TrayQuit }).Count -gt 0
      $hasMethod = @($items.Names | Where-Object { $_ -eq $script:TrayMethod }).Count -gt 0
      if ($items.Names.Count -eq 0) {
        Row -App 'toggle' -Id 'RT4-tray-menu' -Keys 'tray right click' -Expected 'menu items readable' -Actual ("menu window class={0} hwnd={1}, no item text from UIA or MSAA" -f $menu.Class, $menu.Hwnd) -Raw '(tray)' -Verdict 'MANUAL' -Via 'UIA + MSAA' -Note 'OWNER, ~20 s: right-click the VietKB tray icon and confirm the four items and their checkmarks. The popup exists but exposes no item text to either reader on this build.'
      } else {
        $ok2 = $hasToggle -and $hasQuit -and $hasMethod
        $v2 = if ($ok2) { 'PASS' } else { 'FAIL' }
        Row -App 'toggle' -Id 'RT4-tray-menu' -Keys 'tray right click' -Expected 'items: toggle / method / open / quit' -Actual (($items.States -join ' ; ')) -Raw '(tray)' -Verdict $v2 -Via ("UIA/MSAA menu; class=" + $menu.Class)
      }
    }
    [QAInput]::Combo(0x1B, $false, $false, $false)
    Start-Sleep -Milliseconds 300
    # Leave the engine ON for the soak and the teardown.
    if ((Get-State $t) -ne 'ON') { [void](Ensure-On $t) }
  }

  $snd1 = SoundErrorCount
  Note ("sound path: the one-shot 'toggle sound could not be played' line appears $snd1 time(s) in app.log (was $snd0). " +
    'Absence is consistent with a healthy path but is NOT proof of sound: this is a GUI-subsystem process and tracing may never reach the redirected log (95-e2e-report.md O2). ' +
    'Audibility of the go_moc pair is the OWNER check, not this script.')
}

# ---------------------------------------------------------------- soak ------
function Run-Soak($t, [int]$seconds) {
  $sentence = if ($script:Method -eq 'telex') { 'tieengs Vieetj coos daaus khoong' } else { 'tie6ng1 Vie6t5 co1 da6u1 kho6ng' }
  $bursts = 0
  $end = (Get-Date).AddSeconds($seconds)
  while ((Get-Date) -lt $end) {
    if (-not (Get-Process -Id $script:TestPid -ErrorAction SilentlyContinue)) { break }
    if (Assert-Focus $t 'soak') { Clear-Field $t | Out-Null; [QAInput]::TypeAscii($sentence, 25); $bursts++ }
    Start-Sleep -Milliseconds 600
  }
  $mem = Get-VkbMemory
  $canary = Get-State $t
  $alive = @(Get-TestInstances).Count
  Row -App 'soak' -Id "SO1-$($seconds)s" -Keys "$bursts bursts" -Expected 'test instance alive, hook transforming' -Actual "test-path procs=$alive canary=$canary mem=$($mem.AppMB)MB total=$($mem.TotalMB)MB" -Raw '(soak)' -Verdict $(if ($alive -eq 1 -and $canary -eq 'ON') { 'PASS' } elseif ($canary -eq 'READFAIL') { 'READ-FAIL' } else { 'FAIL' }) -Via 'typed probe + pid+path'
}

# ------------------------------------------------------------- teardown -----
function Quit-App {
  # The product's own quit, on the process under test only: the footer button by
  # accessible name, then a 5 s exit budget checked against that pid.
  $w = Open-SettingsWindow
  if (-not $w) {
    W ("teardown: no settings window for pid={0}, so no Thoat han button this time" -f $script:TestPid)
    return $false
  }
  $pressed = $false
  for ($i = 0; $i -lt 6; $i++) {
    if (Invoke-Named $w.Hwnd $script:BtnQuit '') { $pressed = $true; break }
    Start-Sleep -Milliseconds 400
  }
  if (-not $pressed) {
    # Last resort inside the window: a real click on the footer button.
    $el = Find-ElementByName $w.Hwnd $script:BtnQuit ''
    if ($el) { $pressed = [bool](Click-Element $el) }
  }
  if ($pressed) { W 'teardown: pressed "Thoat han" (InvokePattern/click)' }
  $end = (Get-Date).AddSeconds(5)
  while ((Get-Date) -lt $end) {
    if (-not (Get-Process -Id $script:TestPid -ErrorAction SilentlyContinue)) { return $true }
    Start-Sleep -Milliseconds 200
  }
  return $false
}

function Close-Scratch {
  # Only the scratch processes this run launched; each stop is a logged KILL line.
  foreach ($id in $script:Started.Procs) {
    if ($script:OwnedPids -notcontains $id) { continue }
    $p = Get-Process -Id $id -ErrorAction SilentlyContinue
    if (-not $p) { continue }
    if ($p.ProcessName -in @('notepad', 'Code', 'chrome')) { [void](Stop-Owned $id 'scratch target window closed at teardown') }
  }
  Start-Sleep -Milliseconds 400
  # Chrome children of a profile we created go with their parent; anything left is
  # reported, never killed (it could be the owner's own browser).
  foreach ($prof in $script:Started.ChromeProfiles) {
    $left = @(Get-CimInstance Win32_Process -Filter "Name='chrome.exe'" -ErrorAction SilentlyContinue |
        Where-Object { $_.CommandLine -and $_.CommandLine.Contains($prof) })
    if ($left.Count -gt 0) {
      Note ("{0} chrome child process(es) from the probe profile are still alive; left alone (only processes this run launched are stopped)" -f $left.Count)
    }
  }
}

function Restore-Config {
  if (-not (Test-Path $script:ConfigBackup)) { Note 'config backup missing - nothing to restore'; return $false }
  Copy-Item -Path $script:ConfigBackup -Destination $script:ConfigPath -Force
  $a = [System.IO.File]::ReadAllBytes($script:ConfigBackup)
  $b = [System.IO.File]::ReadAllBytes($script:ConfigPath)
  $diff = @(Compare-Object $a $b)
  $sha = (Get-FileHash $script:ConfigPath -Algorithm SHA256).Hash
  W ("config restored: {0} bytes, SHA-256 {1}, Compare-Object diff {2}" -f $b.Length, $sha, $diff.Count)
  return ($diff.Count -eq 0)
}

# -------------------------------------------------------------- sections ----
function Invoke-SelfCheck {
  Section 'SELF-CHECK (no app, no GUI)'
  Assert-Encoding
  $bad = 0

  W ("powershell      : {0}" -f $PSVersionTable.PSVersion)
  W ("script dir      : {0}" -f $script:Dir)
  W ("repo root       : {0}" -f $RepoRoot)
  W ("script encoding : ASCII-only, codepoint literals verified")

  if (Test-Path $script:Lib95) { W "harness lib95   : OK ($script:Lib95)" }
  else { W "harness lib95   : MISSING ($script:Lib95)"; $bad++ }

  try {
    Add-Type -AssemblyName UIAutomationClient
    Add-Type -AssemblyName UIAutomationTypes
    Add-Type -AssemblyName System.Windows.Forms
    $null = [System.Windows.Automation.TextPattern]::Pattern
    $null = [System.Windows.Automation.TogglePattern]::Pattern
    $null = [System.Windows.Automation.InvokePattern]::Pattern
    $null = [System.Windows.Automation.ValuePattern]::Pattern
    W 'UIAutomation     : OK (TextPattern, TogglePattern, InvokePattern, ValuePattern resolved)'
  } catch {
    W "UIAutomation     : FAIL ($($_.Exception.Message))"; $bad++
  }
  try { Add-Type -AssemblyName Accessibility; $script:MsaaOk = $true; W 'MSAA            : OK (Accessibility loaded for the classic tray menu)' }
  catch { W "MSAA            : missing ($($_.Exception.Message)) - a menu with no UIA text is graded MANUAL"; $bad++ }
  try { $null = [VkbWin]::All().Length; W 'window layer    : OK (pid/class enumeration, show, MSAA bridge)' }
  catch { W "window layer    : FAIL ($($_.Exception.Message))"; $bad++ }

  # Reader ladder: the classifier must reject exactly what it claims to.
  $cases = @(
    @('', 'empty', $false), @([string][char]0xFFFC, 'U+FFFC', $false), @('b', 'one char', $true), @($script:V_BAT, 'composed', $true),
    @('ba6t5', 'raw', $true), @((([string][char]0xFFFC) + 'x'), 'FFFC then text', $false)
  )
  $readerBad = 0
  foreach ($c in $cases) {
    $got = Test-ReadSane $c[0]
    if ($got -ne $c[2]) { W ("reader classifier: {0} -> {1}, want {2}" -f $c[1], $got, $c[2]); $readerBad++ }
  }
  if ($readerBad -eq 0) { W 'reader ladder   : OK (empty and U+FFFC rejected; text accepted; 2nd reader = clipboard)' } else { $bad++ }
  $bytesShown = Show-Bytes ([string][char]0xFFFC + 'a')
  if ($bytesShown -ne '<U+FFFC>a') { W "reader escape   : FAIL ($bytesShown)"; $bad++ }

  # Bindings: parsed from the live config, printed for the reader, and every non-empty
  # hotkey string in [hotkeys] must parse through the product's own grammar.
  $script:Bind = Get-Bindings
  W ''
  W 'CONFIG (what this run would press, from the live config)'
  foreach ($line in (Get-ConfigLines)) { W ("  {0}" -f $line) }
  $bindKeys = @('enable', 'disable', 'toggle', 'restore_word', 'open_settings', 'cycle_method', 'app_exclude', 'macro_panel', 'macro_toggle')
  $bindBad = 0
  foreach ($k in $bindKeys) {
    $raw = Get-ConfigValue 'hotkeys' $k
    if (-not $raw) { W ("binding {0,-13}: empty (its group would report SKIP(unbound))" -f $k); continue }
    $hk = Parse-Hotkey $raw
    if (-not $hk.Valid) { W ("binding {0,-13}: FAIL raw=`"{1}`" {2}" -f $k, $raw, $hk.Reason); $bindBad++ }
    elseif ($hk.Vk -eq 0 -and -not ($hk.Ctrl -or $hk.Alt -or $hk.Shift -or $hk.Win)) { W ("binding {0,-13}: FAIL zero chord" -f $k); $bindBad++ }
  }
  foreach ($need in @('Toggle', 'Restore', 'Open', 'Enable', 'Disable')) {
    $hk = $script:Bind.$need
    if ($hk.Empty) { W ("binding {0,-13}: unbound -> that group reports SKIP(unbound)" -f $need) }
    elseif (-not $hk.Valid) { W ("binding {0,-13}: FAIL {1}" -f $need, $hk.Reason); $bindBad++ }
  }
  if ($bindBad -eq 0) {
    W 'bindings        : OK (every non-empty hotkey in the config parses to a nonzero chord)'
    W ("release orders  : toggle={0} restore={1} open={2}" -f `
        ((Get-ReleaseOrders $script:Bind.Toggle) -join ','), ((Get-ReleaseOrders $script:Bind.Restore) -join ','), ((Get-ReleaseOrders $script:Bind.Open) -join ','))
  } else { $bad++ }

  # No chord may be written into the script: it would silently disagree with the config
  # (that is exactly what made CH1 press the wrong chord in the first host run). Comments
  # are stripped first, so the docstring may still name a chord as prose.
  $raw = [System.IO.File]::ReadAllText($script:ScriptPath)
  $body = [regex]::Replace($raw, '(?s)<#.*?#>', '')
  $body = [regex]::Replace($body, '(?m)^\s*#.*$', '')
  $body = [regex]::Replace($body, '(?m)\s+#[^\r\n]*$', '')
  $hard = @([regex]::Matches($body, 'Ctrl\+Shift|Alt\+Shift'))
  if ($hard.Count -eq 0) { W 'no hardcoded chord: OK (no toggle/restore chord literal in code; every press reads hotkeys.*)' }
  else {
    W ("no hardcoded chord: FAIL ({0} occurrence(s) in code)" -f $hard.Count)
    foreach ($h in ($hard | Select-Object -First 3)) {
      $idx = $h.Index
      $from = [Math]::Max(0, $idx - 40)
      W ("  - ...{0}..." -f $body.Substring($from, [Math]::Min(90, $body.Length - $from)).Replace("`r", ' ').Replace("`n", ' '))
    }
    $bad++
  }

  # ---- guardrails: 99-agent-guardrails.md (the owner's app is untouchable) ----
  $gBad = 0
  $astRef = $null
  $parseErrors = $null
  $scriptAst = [System.Management.Automation.Language.Parser]::ParseFile($script:ScriptPath, [ref]$astRef, [ref]$parseErrors)
  $savedOwned = @($script:OwnedPids)
  $stopBody = ($scriptAst.FindAll({ param($n) $n -is [System.Management.Automation.Language.FunctionDefinitionAst] -and $n.Name -eq 'Stop-Owned' }, $true) | Select-Object -First 1)
  $waitBody = ($scriptAst.FindAll({ param($n) $n -is [System.Management.Automation.Language.FunctionDefinitionAst] -and $n.Name -eq 'Wait-OwnerQuit' }, $true) | Select-Object -First 1)
  # The one place that decides whether a stop is allowed, exercised with synthetic input.
  $script:OwnedPids = New-Object System.Collections.ArrayList
  [void]$script:OwnedPids.Add(4242)
  $installedPath = Join-Path $script:InstalledRoot 'vietkb.exe'
  $targetPath = Join-Path $script:TargetRoot 'release\vietkb.exe'
  $cases = @(
    @('vietkb', $installedPath, 4242, $false, 'installed app'),
    @('vietkb', $targetPath, 9999, $false, 'target build this run did not launch'),
    @('vietkb', $targetPath, 4242, $true, 'owned build under target\'),
    @('notepad', "$env:WINDIR\System32\notepad.exe", 4242, $true, 'scratch app this run launched'),
    @('notepad', "$env:WINDIR\System32\notepad.exe", 7777, $false, 'an app this run did not launch')
  )
  foreach ($c in $cases) {
    $verdict = Test-KillAllowed $c[0] $c[1] $c[2]
    $allowed = ($verdict -eq 'allowed')
    if ($allowed -ne $c[3]) { W ("kill rule        : FAIL {0} -> {1} (want allowed={2})" -f $c[4], $verdict, $c[3]); $gBad++ }
  }
  if ($gBad -eq 0) { W 'kill rule       : OK (installed app and unowned processes refused; owned target build and scratch apps allowed)' }

  # No stop by name, and exactly one Stop-Process call in the whole file (the helper).
  # The AST is used so the checker's own message strings cannot be mistaken for calls.
  $stopCmds = @($scriptAst.FindAll({ param($n) $n -is [System.Management.Automation.Language.CommandAst] -and $n.GetCommandName() -eq 'Stop-Process' }, $true))
  $cmdTexts = @($stopCmds | ForEach-Object { $_.Extent.Text })
  $byName = @($cmdTexts | Where-Object { $_ -match '-Name\b' })
  $byId = @($cmdTexts | Where-Object { $_ -match '-Id\b' })
  $inHelper = ($stopCmds.Count -eq 1) -and ($cmdTexts.Count -eq 1) -and $stopBody -and ($stopBody.Extent.Text.Contains($cmdTexts[0]))
  if ($stopCmds.Count -eq 1 -and $byId.Count -eq 1 -and $byName.Count -eq 0 -and $inHelper) {
    W 'no kill by name : OK (exactly one stop call, -Id only, inside Stop-Owned)'
  } else {
    W ("no kill by name : FAIL (calls={0} byId={1} byName={2} insideHelper={3})" -f $stopCmds.Count, $byId.Count, $byName.Count, [bool]$inHelper); $gBad++
  }

  # Every kill path is logged before the stop, and the owner-quit wait cannot kill at all.
  if ($stopBody) {
    $t = $stopBody.Extent.Text
    $logAt = $t.IndexOf('Write-KillLog')
    $killAt = $t.IndexOf('Stop-Process')
    if ($logAt -ge 0 -and $killAt -gt $logAt) { W 'kill logging    : OK (KILL/REFUSED line written before the stop, in Stop-Owned)' }
    else { W 'kill logging    : FAIL (the stop is not preceded by Write-KillLog)'; $gBad++ }
  } else { W 'kill logging    : FAIL (Stop-Owned not found)'; $gBad++ }
  if ($waitBody) {
    $w = $waitBody.Extent.Text
    if ($w -notmatch 'Stop-Owned|Stop-Process') { W 'owner wait      : OK (Wait-OwnerQuit watches only; it has no stop call)' }
    else { W 'owner wait      : FAIL (the owner-quit wait contains a stop)'; $gBad++ }
  } else { W 'owner wait      : FAIL (Wait-OwnerQuit not found)'; $gBad++ }

  # The two blocking paths must refuse to score the run.
  $probeFile = Join-Path $env:TEMP ('vkb-block-' + [guid]::NewGuid().ToString('N') + '.txt')
  $savedResults2 = $script:ResultsPath
  $savedCounts2 = $script:Counts.Clone()
  $savedBlock2 = $script:BlockVerdict
  $savedFail2 = $script:Failures
  try {
    Set-Content $probeFile '# probe' -Encoding UTF8
    $script:ResultsPath = $probeFile
    $script:Counts = @{ PASS = 0; RAW = 0; QUALITY = 0; READFAIL = 0; FAIL = 0; MANUAL = 0; SKIP = 0; BLOCK = 0 }
    $script:BlockVerdict = ''
    $script:Failures = New-Object System.Collections.ArrayList
    Set-Block 'BLOCKED(redirected)' 'probe: launch landed on the installed app'
    $redir = (Get-ShipVerdict $script:Counts)
    $script:Counts = @{ PASS = 0; RAW = 0; QUALITY = 0; READFAIL = 0; FAIL = 0; MANUAL = 0; SKIP = 0; BLOCK = 0 }
    $script:BlockVerdict = ''
    $script:Failures = New-Object System.Collections.ArrayList
    Set-Block 'HOLD(owner-quit)' 'probe: the owner did not quit within 120 s'
    $ownerHold = (Get-ShipVerdict $script:Counts)
    if ($redir -eq 'BLOCKED(redirected)' -and $ownerHold -eq 'HOLD(owner-quit)') {
      W ("blocked run     : OK (redirect -> {0}; owner-quit timeout -> {1}; both block SHIP, neither kills)" -f $redir, $ownerHold)
    } else { W ("blocked run     : FAIL (redirect={0} ownerHold={1})" -f $redir, $ownerHold); $gBad++ }
  } finally {
    $script:ResultsPath = $savedResults2
    $script:Counts = $savedCounts2
    $script:BlockVerdict = $savedBlock2
    $script:Failures = $savedFail2
    $script:OwnedPids = New-Object System.Collections.ArrayList
    foreach ($x in $savedOwned) { [void]$script:OwnedPids.Add($x) }
    Remove-Item $probeFile -Force -ErrorAction SilentlyContinue
  }
  if ($gBad -eq 0) { W 'guardrails      : OK (owner app untouchable, every stop logged and owned, blocks are labelled)' } else { $bad++ }

  # Verdict arithmetic: FAIL is the only blocker, SKIP is silent, a block always holds.
  $a1 = @{ PASS = 94; RAW = 16; QUALITY = 6; READFAIL = 0; FAIL = 0; MANUAL = 0; SKIP = 0; BLOCK = 0 }
  $a2 = @{ PASS = 100; RAW = 0; QUALITY = 0; READFAIL = 3; FAIL = 0; MANUAL = 2; SKIP = 0; BLOCK = 0 }
  $a3 = @{ PASS = 100; RAW = 0; QUALITY = 0; READFAIL = 0; FAIL = 1; MANUAL = 0; SKIP = 0; BLOCK = 0 }
  $a4 = @{ PASS = 90; RAW = 0; QUALITY = 0; READFAIL = 0; FAIL = 0; MANUAL = 0; SKIP = 12; BLOCK = 0 }
  $a5 = @{ PASS = 40; RAW = 0; QUALITY = 0; READFAIL = 0; FAIL = 0; MANUAL = 0; SKIP = 0; BLOCK = 1 }
  $okA = ((Get-ShipVerdict $a1) -eq 'SHIP') -and ((Get-ShipVerdict $a2) -eq 'SHIP') -and ((Get-ShipVerdict $a3) -eq 'HOLD') -and ((Get-ShipVerdict $a4) -eq 'SHIP')
  if ($okA) { W 'verdict math    : OK (QUALITY/MANUAL/READ-FAIL/SKIP never block, one FAIL holds)' } else { W 'verdict math    : FAIL'; $bad++ }
  if ((Get-ShipVerdict $a5) -ne 'SHIP') { W 'blocked arith   : OK (a blocked row can never read SHIP)' } else { W 'blocked arith   : FAIL'; $bad++ }
  $line = Get-BucketLine $a1
  if ($line -match 'READ-FAIL' -and $line -match 'SKIP' -and $line -match 'BLOCKED') { W ("bucket line     : {0}" -f $line) } else { W 'bucket line     : FAIL'; $bad++ }
  # The SKIP path itself: an unbound group must produce SKIP(unbound), not PASS/FAIL.
  $before = @{ PASS = 0; RAW = 0; QUALITY = 0; READFAIL = 0; FAIL = 0; MANUAL = 0; SKIP = 0; BLOCK = 0 }
  $script:Counts = $before.Clone()
  $script:SkipRows = New-Object System.Collections.ArrayList
  $script:Failures = New-Object System.Collections.ArrayList
  $probe = Join-Path $env:TEMP ('vkb-skip-' + [guid]::NewGuid().ToString('N') + '.txt')
  $savedResults = $script:ResultsPath
  $script:ResultsPath = $probe
  try {
    Row -App 'chord' -Id 'SKIP-probe' -Keys '(unbound)' -Expected 'nothing pressed' -Actual 'empty binding' -Raw '(config)' -Verdict 'SKIP(unbound)' -Via 'config binding'
    $skipOk = ($script:Counts.SKIP -eq 1) -and ($script:Counts.FAIL -eq 0) -and ($script:Failures.Count -eq 0) -and ((Get-ShipVerdict $script:Counts) -eq 'SHIP')
    if ($skipOk) { W 'skip path       : OK (SKIP(unbound) counted on its own line and never blocks)' } else { W 'skip path       : FAIL'; $bad++ }
  } finally {
    $script:ResultsPath = $savedResults
    Remove-Item $probe -Force -ErrorAction SilentlyContinue
    $script:Counts = $before
  }

  foreach ($cmd in @('Get-Process', 'Get-FileHash', 'Compare-Object', 'Get-CimInstance', 'Select-String')) {
    if (Get-Command $cmd -ErrorAction SilentlyContinue) { W ("cmd {0,-15}: OK" -f $cmd) }
    else { W ("cmd {0,-15}: MISSING" -f $cmd); $bad++ }
  }

  if (Test-Path $script:Exe) {
    $f = Get-Item $script:Exe
    W ("binary          : {0} B  {1}  mtime {2:yyyy-MM-dd HH:mm:ss}" -f $f.Length, (Get-FileHash $f.FullName -Algorithm SHA256).Hash, $f.LastWriteTime)
  } else { W "binary          : MISSING ($script:Exe)"; $bad++ }
  if (Test-Path $script:SimExe) { W ("simulate cli    : OK ($script:SimExe, pins the corpus offline)") }
  else { W "simulate cli    : MISSING ($script:SimExe) - corpus pinning skipped"; }

  if (Test-Path $script:ConfigPath) {
    W ("config          : {0} B  SHA {1}" -f (Get-Item $script:ConfigPath).Length, (Get-FileHash $script:ConfigPath -Algorithm SHA256).Hash)
    W ("config keys     : method={0} open_settings={1} toggle={2} restore_word={3} enable={4} disable={5}" -f `
        (Get-ConfigValue 'input' 'method'), (Get-ConfigValue 'hotkeys' 'open_settings'), (Get-ConfigValue 'hotkeys' 'toggle'), `
        (Get-ConfigValue 'hotkeys' 'restore_word'), (Get-ConfigValue 'hotkeys' 'enable'), (Get-ConfigValue 'hotkeys' 'disable'))
    W ("config enabled  : {0}" -f (Get-ConfigValue 'input' 'enabled'))
    # Method-switch surface: rewrite a COPY of the live config and prove the round-trip.
    $tmp = Join-Path $env:TEMP ('vkb-selfcheck-' + [guid]::NewGuid().ToString('N') + '.toml')
    $savedPath = $script:ConfigPath
    try {
      Copy-Item -Path $script:ConfigPath -Destination $tmp -Force
      $origText = [System.IO.File]::ReadAllText($tmp, [System.Text.Encoding]::UTF8)
      $origMethod = Get-ConfigValue 'input' 'method'
      $script:ConfigPath = $tmp
      $written = Write-ConfigMethod 'telex'
      $readBack = Get-ConfigValue 'input' 'method'
      $othersOk = ((Remove-MethodLine $origText) -ceq (Remove-MethodLine ([System.IO.File]::ReadAllText($tmp, [System.Text.Encoding]::UTF8))))
      $backWritten = Write-ConfigMethod $origMethod
      $backRead = Get-ConfigValue 'input' 'method'
      $byteSame = ((Remove-MethodLine $origText) -ceq (Remove-MethodLine ([System.IO.File]::ReadAllText($tmp, [System.Text.Encoding]::UTF8)))) -and ($backRead -eq $origMethod)
      if ($written -and $readBack -eq 'telex' -and $othersOk -and $backWritten -and $byteSame) {
        W ("method switch   : OK (config edit + read-back, every other line untouched, restore back to `"{0}`")" -f $origMethod)
      } else {
        W ("method switch   : FAIL (write={0} read-back={1} othersUntouched={2} restore={3} back={4})" -f $written, $readBack, $othersOk, $backWritten, $backRead); $bad++
      }
    } catch {
      W "method switch   : FAIL ($($_.Exception.Message))"; $bad++
    } finally {
      $script:ConfigPath = $savedPath
      Remove-Item $tmp -Force -ErrorAction SilentlyContinue
    }
  } else { W "config          : MISSING ($script:ConfigPath) - the app will write defaults"; $bad++ }

  $chrome = Find-Chrome; if ($chrome) { W "chrome          : $chrome" } else { W 'chrome          : MISSING (chrome rows skipped; observer falls back to notepad)' }
  $code = Join-Path $env:LOCALAPPDATA 'Programs\Microsoft VS Code\Code.exe'
  if (Test-Path $code) { W "vscode          : $code" } else { W 'vscode          : not installed (optional, rows skipped)' }
  if (Test-Path "$env:WINDIR\System32\notepad.exe") { W 'notepad         : present' } else { W 'notepad         : missing'; $bad++ }

  Write-ProbeHtml
  $html = Get-Content $script:ProbeHtml -Raw -Encoding UTF8
  if ($html -match 'VietKB E2E probe' -and $html -match '<textarea') { W 'probe page      : OK (local data:/file: page with a textarea)' } else { W 'probe page      : FAIL'; $bad++ }

  $corpus = Get-Corpus
  W ("corpus          : {0} rows ({1} vni + {2} telex)" -f $corpus.Count, @($corpus | Where-Object { $_.method -eq 'vni' }).Count, @($corpus | Where-Object { $_.method -eq 'telex' }).Count)
  $dup = $corpus | ForEach-Object { $_.id } | Group-Object | Where-Object { $_.Count -gt 1 }
  if ($dup) { W ("corpus dup ids  : {0}" -f ($dup.Name -join ', ')); $bad++ }
  $missingComposed = @($corpus | Where-Object { -not $_.composed })
  if ($missingComposed.Count -gt 0) { W ("corpus rows without an expectation: {0}" -f $missingComposed.Count); $bad++ }
  $unknownMethod = @($corpus | Where-Object { $_.method -notin @('vni', 'telex') })
  if ($unknownMethod.Count -gt 0) { W ("corpus rows with an unknown method: {0}" -f (@($unknownMethod | ForEach-Object { $_.id }) -join ', ')); $bad++ }
  foreach ($c in ($corpus | Select-Object -First 3)) {
    W ("  sample        : {0,-26} method={1,-5} keys=[{2}] composed=[{3}] raw=[{4}]" -f $c.id, $c.method, $c.KeysLabel, $c.composed, $c.raw)
  }

  # Row budget of the next run, so a miscount is caught here and not in the owner window.
  # The chord count is derived from the bindings, because release orders follow them.
  $perApp = $corpus.Count
  $chordRows = (Get-ReleaseOrders $script:Bind.Toggle).Count + (Get-ReleaseOrders $script:Bind.Restore).Count + (Get-ReleaseOrders $script:Bind.Open).Count
  $fixed = $chordRows + 5 + 3   # chords + toggles (RT1, RT2, RT3, RT4 x2) + teardown (TE1-TE3)
  $harness = 2            # H1-method-switch per group that needs a restart
  $soak = if ($Fast) { 0 } else { 1 }
  $rowsNext = 3 * $perApp + $fixed + $harness + $soak
  W ("next run budget : 3 apps x {0} = {1} typing rows + {2} chords + 8 toggle/teardown + {3} method + {4} soak = {5} rows" -f `
      $perApp, (3 * $perApp), $chordRows, $harness, $soak, $rowsNext)
  if ($perApp -lt 30) { W "corpus          : only $perApp rows per app"; $bad++ }

  # A macro row is config-driven, so it is pinned against the live [macros] table
  # instead of the engine (vkb simulate never reads the config).
  $macroRows = @($corpus | Where-Object { $_.id -match 'macro' })
  $macroBad = 0
  foreach ($m in $macroRows) {
    $trigger = ([string]$m.keys).Trim()
    $expansion = Get-ConfigValue 'macros' $trigger
    $want = $expansion + ' '
    if (-not $expansion) { W ("macro row {0}: config has no [macros] {1}" -f $m.id, $trigger); $macroBad++ }
    elseif ($want -cne $m.composed) { W ("macro row {0}: config {1}=[{2}] + space = [{3}] but the row expects [{4}]" -f $m.id, $trigger, (Show-Bytes $expansion), (Show-Bytes $want), (Show-Bytes $m.composed)); $macroBad++ }
  }
  if ($macroBad -eq 0) { W ("macro rows      : OK ({0} rows match the live [macros] table)" -f $macroRows.Count) } else { $bad++ }

  # Pin every expectation against the tree under test (offline, no GUI).
  if (Test-Path $script:SimExe) {
    $pinned = 0
    $mismatch = New-Object System.Collections.ArrayList
    foreach ($c in $corpus) {
      if ($c.noPin) { continue }
      $pinTarget = if ($c.loss) { $c.lossExpected } else { $c.composed }
      if ($c.id -eq 'D2-tab') { $pinTarget = "$($script:V_BAT)`t$($script:V_BAT)" }
      $got = Get-PinnedText $c $c.method
      $pinned++
      if ($got -cne $pinTarget) { [void]$mismatch.Add(("{0}: pinned=[{1}] engine=[{2}]" -f $c.id, (Show-Bytes $pinTarget), (Show-Bytes $got))) }
    }
    if ($mismatch.Count -eq 0) {
      W ("corpus pin      : OK ({0} rows replayed through vkb simulate, every expectation matched)" -f $pinned)
    } else {
      W ("corpus pin      : {0} MISMATCH(ES) - the expectation is wrong, not the app" -f $mismatch.Count)
      foreach ($m in $mismatch) { W ("  - {0}" -f $m) }
      $bad++
    }
  }

  if (Test-Path $script:AppLog) { W ("app.log         : {0} B (truncated at launch)" -f (Get-Item $script:AppLog).Length) }
  W ("vietkb running  : {0} process(es); installed={1}, under target\={2}" -f (Get-VietkbCount), (Get-InstalledInstances).Count, (Get-TestInstances).Count)
  W ("owner app       : {0}" -f $(if ((Get-InstalledInstances).Count -gt 0) { 'RUNNING - a real run reports HOLD(owner-quit) and never kills it (add -OwnerQuit to wait)' } else { 'not running' }))
  W ("guardrail probe : kill rule / kill logging / no name kill / owner wait / blocked verdicts (see below)")
  W ("MANUAL rows     : RT4-tray-menu may report MANUAL (owner check, ~20 s); nothing else is manual")
  W ("SKIP rows       : a group whose hotkeys.<name> is empty reports SKIP(unbound); with the live config: chords {0}, toggle routes {1}" -f `
      ((Get-ReleaseOrders $script:Bind.Toggle).Count + (Get-ReleaseOrders $script:Bind.Restore).Count + (Get-ReleaseOrders $script:Bind.Open).Count), `
      $(if ($script:Bind.Toggle.Valid -and $script:Bind.Enable.Valid -and $script:Bind.Disable.Valid) { 'all bound' } else { 'some unbound' }))
  if ($ExpectSha -or $ExpectBytes -gt 0) {
    if (Test-Path $script:Exe) {
      $actual = (Get-FileHash $script:Exe -Algorithm SHA256).Hash
      $actualLen = (Get-Item $script:Exe).Length
      $lenOk = ($ExpectBytes -le 0) -or ($actualLen -eq $ExpectBytes)
      $shaOk = (-not $ExpectSha) -or ($actual -eq $ExpectSha.ToUpperInvariant())
      if ($shaOk -and $lenOk) { W ("binary-id      : MATCH ({0} B, {1})" -f $actualLen, $actual) }
      else { W ("binary-id      : DIFFERS (expected {0} B / {1}, found {2} B / {3}) - recorded, not a gate" -f $ExpectBytes, $ExpectSha, $actualLen, $actual) }
    }
  } else { W 'binary-id      : no expected identity (comparison skipped)' }

  W ''
  $verdict = if ($bad -eq 0) { 'OK' } else { "$bad problem(s)" }
  W "SELF-CHECK $verdict"
  if ($bad -eq 0) { exit 0 } else { exit 1 }
}

# ============================================================== main ========
# A terminating error must not leave the test instance (and its keyboard hook) behind.
trap {
  W ("ABORT: {0}" -f $_.Exception.Message)
  if ($script:TestPid -and (Get-Process -Id $script:TestPid -ErrorAction SilentlyContinue)) {
    [void](Stop-Owned $script:TestPid 'abort: the run failed, so the instance it launched must not keep the hook installed')
  }
  exit 8
}

Assert-Encoding
if (-not (Test-Path $script:Lib95)) {
  Write-Host "FATAL harness missing: $script:Lib95" -ForegroundColor Red
  exit 2
}
. $script:Lib95

if ($SelfCheck) { Invoke-SelfCheck }

# Bindings come from the live config, parsed once. Nothing below writes a chord value.
$script:Bind = Get-Bindings

Set-Content -Path $script:ResultsPath -Encoding UTF8 -Value @(
  '# VietKB host E2E - results-host.txt',
  ("# generated: {0:yyyy-MM-dd HH:mm:ss}  host: {1}  ps: {2}" -f (Get-Date), $env:COMPUTERNAME, $PSVersionTable.PSVersion),
  '# fields: app | id | keystrokes | expected | actual | raw-keystrokes | verdict | via | note',
  '# judgement (91-round-plan-sound-cases.md, owner 2026-10-30): PASS = pinned composed text; PASS(raw) = literal',
  '#   raw keystrokes (rule 4); QUALITY = documented, owner-accepted loss (L1-L7 / raw-zone) producing the pinned',
  '#   text; READ-FAIL = the harness could not read the field (unmeasured, not broken); MANUAL = owner does it by',
  '#   hand; SKIP(unbound) = the config binds no chord for that group, so nothing was pressed;',
  '#   BLOCKED(redirected) / HOLD(owner-quit) = the run could not be graded (Tauri single-instance sent',
  '#   the launch to the installed build, or the owner app is in the way); both block SHIP.',
  '#   HOLD when FAIL > 0 or anything blocked, else SHIP.',
  '# guardrails (99-agent-guardrails.md): the installed app in %LOCALAPPDATA%\VietKB is never killed,',
  '#   restarted or relaunched; only processes this run launched under E:\vietkb\target may be stopped,',
  '#   and every stop/refusal is logged below as KILL/REFUSED <time> pid= path= reason=.',
  '# every row clears the field first (Ctrl+A, Delete, verified empty), so expectations are absolute.',
  '# chords are read from the live config at run time; the CONFIG block below lists exactly what was pressed.',
  '# app labels: notepad | chrome | vscode | chord | toggle | soak | teardown | harness | runtime'
)
$script:RunActive = $true

Section 'PRECONDITIONS (guardrails: installed app untouchable)'
# 1) The owner's installed build. Never stopped, never restarted: the run aborts or waits.
$script:InstalledApp = Get-InstalledInstances
if ($script:InstalledApp.Count -gt 0) {
  $script:OwnerWasRunning = $true
  foreach ($p in $script:InstalledApp) { W ("owner app running: {0}" -f (Format-Proc $p)) }
  Add-Content -Encoding UTF8 -Path $script:ResultsPath -Value ("# owner-app: RUNNING at start ({0}) - off-limits, never killed" -f (($script:InstalledApp | ForEach-Object { Format-Proc $_ }) -join '; '))
  if ($OwnerQuit) {
    if (Wait-OwnerQuit 120) {
      $script:OwnerQuitByOwner = $true
      Note 'the owner quit their installed VietKB during the -OwnerQuit wait. It was not killed and it will NOT be relaunched by this script: start your IME again when you are done with this window.'
      Add-Content -Encoding UTF8 -Path $script:ResultsPath -Value '# owner-app: the owner quit it during the -OwnerQuit wait; not killed, not relaunched'
    } else {
      Set-Block 'HOLD(owner-quit)' ("installed VietKB still running after the 120 s -OwnerQuit wait: " + (($script:InstalledApp | ForEach-Object { Format-Proc $_ }) -join '; '))
      W 'HOLD(owner-quit): installed VietKB is still running. The script will not kill it. Quit it and re-run.'
      Add-Content -Encoding UTF8 -Path $script:ResultsPath -Value 'HOLD(owner-quit)'
      exit 7
    }
  } else {
    Set-Block 'HOLD(owner-quit)' ("installed VietKB is running: " + (($script:InstalledApp | ForEach-Object { Format-Proc $_ }) -join '; '))
    W 'HOLD(owner-quit): installed VietKB is running, owner must quit it - the script will not kill it. Re-run with -OwnerQuit to wait for the owner.'
    Add-Content -Encoding UTF8 -Path $script:ResultsPath -Value 'HOLD(owner-quit)'
    exit 7
  }
} else {
  W 'owner app: not running (nothing to leave alone; it will be left absent)'
  Add-Content -Encoding UTF8 -Path $script:ResultsPath -Value '# owner-app: not running at start (left absent)'
}

# 2) A leftover build under target\ from an earlier run: also not killed, because this run
#    did not launch it.
$stale = @(Get-TestInstances)
if ($stale.Count -gt 0) {
  foreach ($p in $stale) { W ("stale test build: {0}" -f (Format-Proc $p)) }
  if ($OwnerQuit) {
    if (Wait-OwnerQuit 120) { Note 'the stale target build was closed during the -OwnerQuit wait; nothing was killed by this script' }
    else {
      Set-Block 'HOLD(stale-target)' ("a target build from an earlier run is still alive: " + (($stale | ForEach-Object { Format-Proc $_ }) -join '; '))
      Exit 7
    }
  } else {
    Set-Block 'HOLD(stale-target)' ("a target build from an earlier run is still alive: " + (($stale | ForEach-Object { Format-Proc $_ }) -join '; ') + '. This run did not launch it, so it will not stop it. Close it (or re-run with -OwnerQuit to wait).')
    W 'HOLD(stale-target): close the leftover target build yourself, or re-run with -OwnerQuit.'
    Add-Content -Encoding UTF8 -Path $script:ResultsPath -Value 'HOLD(stale-target)'
    exit 7
  }
}

if (Test-Path $script:ConfigPath) {
  Copy-Item -Path $script:ConfigPath -Destination $script:ConfigBackup -Force
  $cfgBytes = [System.IO.File]::ReadAllBytes($script:ConfigPath)
  $cfgSha = (Get-FileHash $script:ConfigPath -Algorithm SHA256).Hash
  W ("config backup: {0} B, SHA-256 {1} -> {2}" -f $cfgBytes.Length, $cfgSha, $script:ConfigBackup)
  Add-Content -Encoding UTF8 -Path $script:ResultsPath -Value ("# config: {0} B  SHA-256 {1}  (backed up, then restored and byte-compared at the end)" -f $cfgBytes.Length, $cfgSha)
  $script:Method = (Get-ConfigValue 'input' 'method')
  if (-not $script:Method) { $script:Method = 'vni' }
  $script:BaseMethod = $script:Method
  Add-Content -Encoding UTF8 -Path $script:ResultsPath -Value ("# method: {0} (the run switches to the other method for its group, then restores this one)" -f $script:Method)
  Write-ConfigBlock
} else {
  W 'config.toml not present; the app will write defaults (nothing to restore)'
  Add-Content -Encoding UTF8 -Path $script:ResultsPath -Value '# config: absent at start (nothing to restore)'
  $script:Method = 'vni'
  $script:BaseMethod = 'vni'
  Write-ConfigBlock
}

if (Test-Path $script:Exe) {
  $x = Get-Item $script:Exe
  $xSha = (Get-FileHash $x.FullName -Algorithm SHA256).Hash
  W ("binary: {0} B ({1:N2} MiB)  SHA-256 {2}  mtime {3:yyyy-MM-dd HH:mm:ss}" -f $x.Length, ($x.Length / 1MB), $xSha, $x.LastWriteTime)
  Add-Content -Encoding UTF8 -Path $script:ResultsPath -Value ("# binary: {0}  {1} B  SHA-256 {2}" -f $x.FullName, $x.Length, $xSha)
  $idOk = ((-not $ExpectSha) -or ($xSha -eq $ExpectSha.ToUpperInvariant())) -and (($ExpectBytes -le 0) -or ($x.Length -eq $ExpectBytes))
  $idLine = if (-not $ExpectSha -and $ExpectBytes -le 0) { 'no expected identity given; recorded only' }
            elseif ($idOk) { ("MATCH (the validated fix build: {0} B, {1})" -f $ExpectBytes, $ExpectSha) }
            else { ("DIFFERS from the validated fix build ({0} B, {1}); graded {2} B, {3}" -f $ExpectBytes, $ExpectSha, $x.Length, $xSha) }
  W ("binary-id: {0}" -f $idLine)
  Add-Content -Encoding UTF8 -Path $script:ResultsPath -Value ("# binary-id: {0}  expected={1} B / {2}" -f $idLine, $ExpectBytes, $(if ($ExpectSha) { $ExpectSha } else { '(none)' }))
  if (-not $idOk) {
    Note ("the graded binary is not the build this harness was validated against (expected {0} B / {1}, found {2} B / {3}). That is legitimate after a rebuild, but the report must name the build the rows describe." -f $ExpectBytes, $ExpectSha, $x.Length, $xSha)
  }
} else { W "FATAL binary missing: $script:Exe"; exit 4 }
Remove-Item $script:AppLog -ErrorAction SilentlyContinue

Section 'LAUNCH (detached: .cmd + ShellExecute, hidden console)'
$appProc = Start-AppVerified
if (-not $appProc) {
  if ($script:BlockVerdict) {
    # Nothing to close: the launch never produced a process under target\. The installed
    # app that took the redirect is the owner's and is left strictly alone.
    W ("BLOCKED: {0} - nothing was graded and the owner app was not touched" -f $script:BlockVerdict)
    Add-Content -Encoding UTF8 -Path $script:ResultsPath -Value ''
    Add-Content -Encoding UTF8 -Path $script:ResultsPath -Value (Get-BucketLine $script:Counts)
    Add-Content -Encoding UTF8 -Path $script:ResultsPath -Value $script:BlockVerdict
    W (Get-BucketLine $script:Counts)
    W $script:BlockVerdict
    exit 7
  }
  W 'FATAL app did not start (no process under target\ and no installed instance)'; exit 5
}
$script:Started.Procs += $appProc.Id

Section 'TARGETS'
$targets = New-Object System.Collections.ArrayList
$np = Start-Notepad
if ($np) { [void]$targets.Add($np); W "notepad: hwnd=$($np.Hwnd) pid=$($np.Pid)" } else { W 'notepad: FAILED to open' }
$ch = Start-ChromeProbe
if ($ch) { [void]$targets.Add($ch); W "chrome: hwnd=$($ch.Hwnd) pid=$($ch.Pid) page=$($ch.Page)" } else { W 'chrome: not available' }
$vs = Start-VsCode
if ($vs) { [void]$targets.Add($vs); W "vscode: hwnd=$($vs.Hwnd) pid=$($vs.Pid)" } else { W 'vscode: not available (optional)' }
if ($targets.Count -eq 0) { W 'FATAL no target window'; exit 6 }

$observer = if ($ch) { $ch } else { $np }
if ($vs) { Note 'vscode rows are read through Ctrl+S + the saved file (Monaco exposes no document text over UIA), as in 95-e2e-report.md' }

Section 'READINESS / WARM-UP'
foreach ($t in $targets) {
  $warm = $false
  for ($i = 0; $i -lt 10; $i++) {
    Start-Sleep -Milliseconds 500
    if (-not (Assert-Focus $t 'warm-up')) { continue }
    if (-not (Clear-Field $t)) { continue }
    [QAInput]::TypeAscii('warm', $script:KeyDelayMs)
    Start-Sleep -Milliseconds 350
    $r = Read-Actual $t
    if ($r.Sane -and ([string]$r.Text) -like '*warm*') { $warm = $true; W "warm-up $($t.App): ok after $($i + 1) tries"; break }
  }
  if (-not $warm) { W "warm-up $($t.App): FAILED (its rows will report no focus or READ-FAIL)" }
  [void](Clear-Field $t)
}

# --- typing matrix: one method group at a time, each row under its own method ---
$corpus = Get-Corpus
# The three hotkey rows carry the bound chord in their keystroke column, so a reader can
# see what was pressed without consulting the config block.
foreach ($c in $corpus) {
  if ($c.id -eq 'C1-restore-word') { $c.KeysLabel = 'ba6t5 + ' + (Get-BindingLabel $script:Bind.Restore) }
  elseif ($c.id -eq 'E1-off-f2') { $c.KeysLabel = (Get-BindingLabel $script:Bind.Disable) + ' then ba6t5' }
  elseif ($c.id -eq 'E2-on-f1') { $c.KeysLabel = (Get-BindingLabel $script:Bind.Enable) + ' then ba6t5' }
}
$other = if ($script:BaseMethod -eq 'vni') { 'telex' } else { 'vni' }
$groups = New-Object System.Collections.ArrayList
[void]$groups.Add([pscustomobject]@{ Method = $script:BaseMethod; Switched = $false })
[void]$groups.Add([pscustomobject]@{ Method = $other; Switched = $true })

$baseProbe = Confirm-Method $script:BaseMethod $observer
if ($baseProbe -ne 'OK') {
  W ("warning: method probe for `"{0}`" did not read ON ({1}); rows still run" -f $script:BaseMethod, $baseProbe)
}

foreach ($g in $groups) {
  $rows = @($corpus | Where-Object { $_.method -eq $g.Method })
  Section ("TYPING MATRIX - {0} ({1} rows x {2} apps)" -f $g.Method, $rows.Count, $targets.Count)
  $active = $true
  if ($g.Switched) {
    $active = Switch-Method $g.Method $observer
  } elseif ($script:Method -ne $g.Method) {
    $active = Switch-Method $g.Method $observer
  }
  if (-not $active) {
    Note ("method `"{0}`" could not be made active; its {1} rows per app are NOT measured (they are reported as READ-FAIL, and the switch row is a FAIL)" -f $g.Method, $rows.Count)
    foreach ($t in $targets) {
      foreach ($c in $rows) {
        Row -App $t.App -Id $c.id -Keys $c.KeysLabel -Expected $c.composed -Actual '(not run: method switch failed)' -Raw $c.raw -Verdict 'READ-FAIL' -Via 'none' -Note ("method {0} was not active" -f $g.Method)
      }
    }
    continue
  }
  foreach ($t in $targets) {
    W ''
    W "--- $($t.App) [$($g.Method)] ---"
    [void](Guard-Single)
    Repair-Targets $targets
    if (-not (Ensure-On $observer)) { W 'warning: engine could not be switched ON before this app' }
    foreach ($c in $rows) {
      # Tab row: a browser textarea does not insert a tab, VS Code replaces it with
      # editor.insertSpaces; only an editor inserts the real tab. The per-app override
      # is part of the row's absolute expectation, like every other row.
      if ($c.id -eq 'D2-tab') {
        if ($t.App -eq 'chrome') { $c.composed = "$($script:V_BAT)$($script:V_BAT)"; $c.raw = 'ba6t5ba6t5' }
        elseif ($t.App -eq 'vscode') { $c.composed = "$($script:V_BAT) $($script:V_BAT)"; $c.raw = 'ba6t5 ba6t5' }
        else { $c.composed = "$($script:V_BAT)`t$($script:V_BAT)"; $c.raw = "ba6t5`tba6t5" }
      }
      Run-Case $t $c
    }
    if (-not (Ensure-On $observer)) { W 'warning: engine left OFF after this app' }
  }
}

# Back to the host's own method before the chord, toggle and soak blocks.
if ($script:Method -ne $script:BaseMethod) {
  if (-not (Switch-Method $script:BaseMethod $observer)) {
    Note ("could not restore the original method `"{0}`"; the chord/toggle rows below were typed with `"{1}`" and their state probes use that method" -f $script:BaseMethod, $script:Method)
  }
}
if (-not (Get-Process -Id $script:TestPid -ErrorAction SilentlyContinue)) {
  Note 'the test instance is not running after the typing matrix (a method switch failed); relaunching it so the chord and toggle blocks still measure something'
  $again = Start-AppVerified
  if (-not $again) {
    Note 'the relaunch produced no process under target\; the chord and toggle blocks below have no engine to drive'
  } else {
    $script:Started.Procs += $again.Id
    $script:Method = (Get-ConfigValue 'input' 'method')
    if (-not $script:Method) { $script:Method = $script:BaseMethod }
  }
}
Repair-Targets $targets

Section 'CHORDS (3 chords x 3 release orders)'
if (-not (Ensure-On $observer)) { W 'warning: engine not ON before the chord block' }
Run-Chords $observer

Section 'TOGGLE ROUTES + TRAY + SOUND PATH'
Run-Toggles $observer

if ($Fast) {
  Section 'SOAK (skipped: -Fast)'
  Add-Content -Encoding UTF8 -Path $script:ResultsPath -Value 'NOTE | soak skipped (-Fast)'
} else {
  Section "SOAK ($SoakSeconds s; -Fast skips)"
  Run-Soak $observer $SoakSeconds
}

Section 'TEARDOWN'
$clean = Quit-App
$quitPath = 'button'
if (-not $clean) {
  W ("teardown: pid={0} did not end through 'Thoat han' within 5 s; stopping that one process (logged)" -f $script:TestPid)
  if (Get-Process -Id $script:TestPid -ErrorAction SilentlyContinue) {
    [void](Stop-Owned $script:TestPid 'teardown: the test instance did not exit through its own quit button')
    $quitPath = 'owned stop'
  }
}
Start-Sleep -Milliseconds 600
$procsAfter = @(Get-TestInstances).Count
$ownerAfter = @(Get-InstalledInstances).Count
Close-Scratch
$cfgOk = Restore-Config

Row -App 'teardown' -Id 'TE1-quit-via-button' -Keys $script:BtnQuit -Expected 'the test instance exits through its own quit button within 5 s' -Actual $(if ($clean) { 'exited via button' } else { "owned stop of pid=$($script:TestPid) ($quitPath)" }) -Raw '(quit, pid+path)' -Verdict $(if ($clean) { 'PASS' } else { 'FAIL' }) -Via 'InvokePattern on the footer button, test pid only'
Row -App 'teardown' -Id 'TE2-zero-processes' -Keys 'processes under E:\vietkb\target' -Expected '0' -Actual "$procsAfter" -Raw '(pid+path)' -Verdict $(if ($procsAfter -eq 0) { 'PASS' } else { 'FAIL' }) -Via 'pid+path (the owner app is not counted)'
Row -App 'teardown' -Id 'TE3-config-restored' -Keys 'config.toml' -Expected 'byte-identical to the backup' -Actual $(if ($cfgOk) { 'Compare-Object empty' } else { 'Compare-Object not empty' }) -Raw (Get-FileHash $script:ConfigPath -Algorithm SHA256).Hash -Verdict $(if ($cfgOk) { 'PASS' } else { 'FAIL' }) -Via 'Compare-Object'

if ($script:OwnerWasRunning) {
  if ($script:OwnerQuitByOwner) {
    Note 'the run began with the owner''s installed VietKB alive; the OWNER quit it during -OwnerQuit. The script did not kill it and did NOT relaunch it: start your IME again when convenient.'
  } else {
    Note 'the owner''s installed VietKB was alive at start and was never touched by this run.'
  }
} elseif ($ownerAfter -gt 0) {
  Note ("the installed VietKB is running again (%1 processes) - the owner or autostart restarted it; the script neither started nor touched it." -replace '%1', $ownerAfter)
} else {
  Note 'the installed VietKB was absent at start and is absent at the end: unchanged, and nothing was left running by this script.'
}

Section 'SUMMARY'
$bucketLine = Get-BucketLine $script:Counts
$verdict = Get-ShipVerdict $script:Counts
Add-Content -Encoding UTF8 -Path $script:ResultsPath -Value ''
Add-Content -Encoding UTF8 -Path $script:ResultsPath -Value $bucketLine
Add-Content -Encoding UTF8 -Path $script:ResultsPath -Value $verdict
if ($script:Counts.READFAIL -gt 0) {
  Add-Content -Encoding UTF8 -Path $script:ResultsPath -Value ("CAVEAT | {0} row(s) were never measured (READ-FAIL); re-run before treating this as evidence" -f $script:Counts.READFAIL)
}
if ($script:Counts.MANUAL -gt 0) {
  Add-Content -Encoding UTF8 -Path $script:ResultsPath -Value ("MANUAL | {0}" -f (($script:ManualRows) -join ', '))
}
if ($script:Counts.SKIP -gt 0) {
  Add-Content -Encoding UTF8 -Path $script:ResultsPath -Value ("SKIP | {0} group(s) had no bound chord: {1}" -f $script:Counts.SKIP, (($script:SkipRows) -join ', '))
}
if ($script:Counts.BLOCK -gt 0) {
  Add-Content -Encoding UTF8 -Path $script:ResultsPath -Value ("BLOCKED | {0} row(s): {1}" -f $script:Counts.BLOCK, $script:BlockVerdict)
}
Add-Content -Encoding UTF8 -Path $script:ResultsPath -Value ("# stops: {0} (KILL/REFUSED lines above; only processes this run launched)" -f $script:Kills.Count)

$total = ((Get-Date) - $script:StartTime).TotalMinutes
W ''
W ("{0}   ({1:N1} min, {2} notes, {3} FOCUS lines, {4} stop(s))" -f $bucketLine, $total, $script:Notes, $script:FocusCount, $script:Kills.Count)
if ($script:Failures.Count -gt 0) {
  W 'failures:'
  foreach ($f in $script:Failures) { W "  - $f" }
}
if ($script:Counts.READFAIL -gt 0) {
  W ("CAVEAT: {0} unmeasured row(s) - {1}" -f $script:Counts.READFAIL, (($script:ReadFails | Select-Object -First 6) -join '; '))
}
if ($script:Counts.MANUAL -gt 0) { W ("MANUAL: " + (($script:ManualRows) -join ', ') + " - the results file says what to do and how long it takes") }
if ($script:Counts.SKIP -gt 0) { W ("SKIP: " + (($script:SkipRows) -join ', ') + " - no chord bound in the config, nothing pressed") }
if ($script:Counts.BLOCK -gt 0) { W ("BLOCKED: " + $script:BlockVerdict + " - the run is void, nothing is graded") }
W ("owner app: {0}" -f $(if ($script:OwnerWasRunning) { 'was running at start, never killed, not relaunched' } else { 'absent at start, left absent' }))
W ("config: {0}   test-path processes left: {1}   results: {2}" -f $(if ($cfgOk) { 'restored byte-identical' } else { 'NOT restored' }), $procsAfter, $script:ResultsPath)
W $verdict
if ($verdict -eq 'SHIP' -and $cfgOk -and $procsAfter -eq 0) { exit 0 } else { exit 1 }
