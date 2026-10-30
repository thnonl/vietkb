# D-E2E-02 re-test: `restore_word` (Ctrl+Shift+Z) and the modifier-only toggle.
# Route: real SendInput -> low-level hook -> foreground app; text read with UIA
# (Notepad) or the saved file (VS Code, Monaco exposes no document text).
param(
  [ValidateSet('notepad', 'vscode')][string]$App,
  [string]$Tag = 'owner'
)
. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
$dir = 'E:\vietkb\docs\evkey-clone-work\qa\e2e95'
$tsv = Join-Path $dir "restore-$Tag-$App.tsv"
if (-not (Test-Path $tsv)) {
  Set-Content -Encoding UTF8 $tsv "tag`tapp`tid`tkeystrokes`texpected`tactual`tvia`tverdict"
}
$global:KeyDelay = 55

$cfgText = Get-Content "$env:APPDATA\vietkb\config.toml" -Raw
$toggleCfg = ([regex]::Match($cfgText, '(?m)^toggle\s*=\s*"([^"]*)"')).Groups[1].Value
$settingsCfg = ([regex]::Match($cfgText, '(?m)^open_settings\s*=\s*"([^"]*)"')).Groups[1].Value
$modeCfg = ([regex]::Match($cfgText, '(?m)^toggle_mode\s*=\s*"([^"]*)"')).Groups[1].Value
Write-Host "CONFIG toggle=[$toggleCfg] open_settings=[$settingsCfg] toggle_mode=[$modeCfg]"

function Row($id, $keys, $expected, $actual, $via, $verdict) {
  Add-Content -Encoding UTF8 $tsv ("{0}`t{1}`t{2}`t{3}`t{4}`t{5}`t{6}`t{7}" -f $Tag, $App, $id, $keys, $expected, $actual, $via, $verdict)
  Write-Host ("{0,-26} keys=[{1}] exp=[{2}] act=[{3}] {4} via={5}" -f $id, $keys, $expected, $actual, $verdict, $via)
}

# ---- app setup (same shape as run-typing.ps1) ----
$hwnd = [IntPtr]::Zero
switch ($App) {
  'notepad' {
    Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force
    Start-Sleep -Milliseconds 600
    $f = Join-Path $dir 'scratch-np.txt'
    Remove-Item $f -ErrorAction SilentlyContinue; New-Item -ItemType File -Path $f | Out-Null
    Start-Process notepad.exe -ArgumentList $f | Out-Null
    $w = Wait-Window 'scratch-np.txt' 30
    if (-not $w) { Write-Host 'SETUP-FAIL notepad window'; exit 1 }
    $hwnd = $w.Hwnd
  }
  'vscode' {
    $f = Join-Path $dir 'scratch-vsc.txt'
    Remove-Item $f -ErrorAction SilentlyContinue; New-Item -ItemType File -Path $f | Out-Null
    $code = "$env:LOCALAPPDATA\Programs\Microsoft VS Code\Code.exe"
    Start-Process $code -ArgumentList @('-n', $f) | Out-Null
    $w = Wait-Window 'scratch-vsc.txt' 40
    if (-not $w) { Write-Host 'SETUP-FAIL vscode window'; exit 1 }
    $hwnd = $w.Hwnd
  }
}
Write-Host "APP=$App hwnd=$hwnd"

function FocusTarget {
  if ($App -eq 'notepad') { [void](Repair-Notepad 'scratch-np.txt') }
  [void][QAInput]::Focus($hwnd)
  Start-Sleep -Milliseconds 150
  if ($App -ne 'vscode') {
    $el = Get-EditElement $hwnd
    if ($el) { [void](Click-Element $el) } else { Click-Center $hwnd }
  } else {
    Click-Center $hwnd
  }
  Start-Sleep -Milliseconds 150
  return ([QAInput]::GetForegroundWindow() -eq $hwnd)
}

function Read-Actual {
  if ($App -eq 'vscode') {
    [QAInput]::Combo(0x53, $false, $true, $false)  # Ctrl+S
    Start-Sleep -Milliseconds 400
    $p = Join-Path $dir 'scratch-vsc.txt'
    $t = [System.IO.File]::ReadAllText($p, [System.Text.Encoding]::UTF8)
    return [pscustomobject]@{ Text = $t; Via = 'saved-file' }
  }
  # Notepad restores its previous session tabs; read the one document that holds
  # the warm-up marker instead of the first Document in the UIA tree.
  if ($script:DocEl) {
    try {
      $tp = $script:DocEl.GetCurrentPattern([System.Windows.Automation.TextPattern]::Pattern)
      return [pscustomobject]@{ Text = $tp.DocumentRange.GetText(-1); Via = 'TextPattern/pinned' }
    } catch {}
  }
  $r = Read-Text $hwnd
  if ($null -eq $r.Text) { $r.Text = '' }
  return $r
}

function Find-OurDocument([string]$needle) {
  try { $root = [System.Windows.Automation.AutomationElement]::FromHandle($hwnd) } catch { return $null }
  if (-not $root) { return $null }
  foreach ($el in $root.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)) {
    $ct = ''
    try { $ct = $el.Current.ControlType.ProgrammaticName } catch { continue }
    if ($ct -notmatch 'Document|Edit') { continue }
    try {
      $tp = $el.GetCurrentPattern([System.Windows.Automation.TextPattern]::Pattern)
      $t = $tp.DocumentRange.GetText(-1)
      if ($t -like "*$needle*") { return $el }
    } catch {}
  }
  return $null
}

# warmup
$script:DocEl = $null
$warm = $false
for ($i = 0; $i -lt 12; $i++) {
  Start-Sleep -Milliseconds 700
  $focused = FocusTarget
  if (-not $focused) { continue }
  Clear-TextArea $hwnd
  [QAInput]::TypeAscii('warm', $global:KeyDelay)
  Start-Sleep -Milliseconds 350
  if ($App -eq 'notepad') {
    $script:DocEl = Find-OurDocument 'warm'
    if ($script:DocEl) { $warm = $true; break }
  } else {
    $a = Read-Actual
    if ([string]$a.Text -like '*warm*') { $warm = $true; break }
  }
}
if (-not $warm) { Write-Host 'WARMUP-FAIL'; exit 2 }
Clear-TextArea $hwnd
Write-Host "WARMUP ok pinned=$([bool]$script:DocEl)"

# ---- ordered chord helper: full control over release order ----
Add-Type @'
using System;
using System.Runtime.InteropServices;
using System.Threading;
public static class QAChord {
  [DllImport("user32.dll", SetLastError = true)] static extern uint SendInput(uint n, INPUT[] p, int cb);
  [DllImport("user32.dll")] static extern uint MapVirtualKeyW(uint code, uint type);
  [StructLayout(LayoutKind.Sequential)] struct KEYBDINPUT { public ushort wVk; public ushort wScan; public uint dwFlags; public uint time; public IntPtr extra; }
  [StructLayout(LayoutKind.Sequential)] struct MOUSEINPUT { public int dx; public int dy; public uint data; public uint flags; public uint time; public IntPtr extra; }
  [StructLayout(LayoutKind.Sequential)] struct HARDINPUT { public uint msg; public ushort p1; public ushort p2; }
  [StructLayout(LayoutKind.Explicit)] struct InputUnion {
    [FieldOffset(0)] public MOUSEINPUT mi;
    [FieldOffset(0)] public KEYBDINPUT ki;
    [FieldOffset(0)] public HARDINPUT hi;
  }
  [StructLayout(LayoutKind.Sequential)] struct INPUT { public uint type; public InputUnion u; }

  public static int Failures = 0;
  static void Raw(ushort vk, bool up) {
    INPUT[] a = new INPUT[1];
    a[0].type = 1;
    a[0].u.ki.wVk = vk;
    a[0].u.ki.wScan = (ushort)MapVirtualKeyW(vk, 0);
    a[0].u.ki.dwFlags = up ? 2u : 0u;
    if (SendInput(1, a, Marshal.SizeOf(typeof(INPUT))) != 1) Failures++;
  }
  // press order: Ctrl, Shift, key. Release order given by `order` (letters C, S, K).
  public static void Chord(ushort key, string order) {
    Raw(0x11, false);
    Raw(0x10, false);
    if (key != 0) Raw(key, false);
    Thread.Sleep(60);
    foreach (char c in order.ToUpperInvariant()) {
      if (c == 'K' && key != 0) Raw(key, true);
      if (c == 'S') Raw(0x10, true);
      if (c == 'C') Raw(0x11, true);
      Thread.Sleep(25);
    }
  }
  // tap a modifier-only chord: down Ctrl, down Shift, up Shift, up Ctrl
  public static void TapCtrlShift(int holdMs) {
    Raw(0x11, false);
    Thread.Sleep(30);
    Raw(0x10, false);
    Thread.Sleep(holdMs);
    Raw(0x10, true);
    Thread.Sleep(30);
    Raw(0x11, true);
  }
  public static void TapAltShift(int holdMs) {
    Raw(0x12, false);
    Thread.Sleep(30);
    Raw(0x10, false);
    Thread.Sleep(holdMs);
    Raw(0x10, true);
    Thread.Sleep(30);
    Raw(0x12, true);
  }
}
'@

function TypeAndRead([string]$text) {
  $focused = FocusTarget
  Clear-TextArea $hwnd
  [QAInput]::TypeAscii($text, $global:KeyDelay)
  Start-Sleep -Milliseconds 400
  $a = Read-Actual
  return [pscustomobject]@{ Text = ([string]$a.Text).TrimEnd("`r", "`n"); Via = $a.Via; Focused = $focused }
}

# Make sure Vietnamese is ON before a row that expects composition.
function Ensure-On {
  $r = TypeAndRead 'ba6t5'
  if ($r.Text -ceq 'bật') { return }
  for ($i = 0; $i -lt 4; $i++) {
    if ($toggleCfg -eq 'Ctrl+Shift') { [QAChord]::TapCtrlShift(90) } else { [QAChord]::TapAltShift(90) }
    Start-Sleep -Milliseconds 400
    $r = TypeAndRead 'ba6t5'
    if ($r.Text -ceq 'bật') { Write-Host "ensure-on: ON after $($i+1) tap(s)"; return }
  }
  Write-Host "ensure-on: FAILED last=[$($r.Text)]"
}

# hook-side evidence: every fired hotkey writes `stage=hotkey`
$logPath = "$env:APPDATA\vietkb\keys.log"
function HotkeyCount {
  if (-not (Test-Path $logPath)) { return 0 }
  return (Select-String -Path $logPath -Pattern 'stage=hotkey' | Measure-Object).Count
}
$hkBefore = HotkeyCount

# R1: the defect row — mis-typed/transformed word, then Ctrl+Shift+Z, text back
Ensure-On
$focused = FocusTarget
Clear-TextArea $hwnd
[QAInput]::TypeAscii('ba6t5', $global:KeyDelay)
Start-Sleep -Milliseconds 300
$before = Read-Actual; $beforeTxt = ([string]$before.Text).TrimEnd("`r", "`n")
Row 'R0-typed' 'ba6t5' 'bật' $beforeTxt $before.Via $(if ($beforeTxt -ceq 'bật') { 'PASS' } else { 'FAIL' })

[QAChord]::Chord(0x5A, 'KSC')
Start-Sleep -Milliseconds 450
$a = Read-Actual; $t = ([string]$a.Text).TrimEnd("`r", "`n")
Row 'R1-restore' 'ba6t5 Ctrl+Shift+Z' 'ba6t5' $t $a.Via $(if ($t -ceq 'ba6t5') { 'PASS' } else { 'FAIL' })

# R2: the chord must not have toggled Vietnamese — typing must still transform
$r = TypeAndRead 'ba6t5'
Row 'R2-still-on' 'ba6t5 after the chord' 'bật' $r.Text $r.Via $(if ($r.Text -ceq 'bật') { 'PASS' } else { 'FAIL' })

# R3/R4/R5: release order must not double-fire or lose the restore
foreach ($order in @('SKC', 'CSK', 'CKS')) {
  $focused = FocusTarget
  Clear-TextArea $hwnd
  [QAInput]::TypeAscii('ba6t5', $global:KeyDelay)
  Start-Sleep -Milliseconds 250
  [QAChord]::Chord(0x5A, $order)
  Start-Sleep -Milliseconds 450
  $a = Read-Actual; $t = ([string]$a.Text).TrimEnd("`r", "`n")
  $id = "R-order-$order"
  Row $id "ba6t5 Ctrl+Shift+Z release=$order" 'ba6t5' $t $a.Via $(if ($t -ceq 'ba6t5') { 'PASS' } else { 'FAIL' })
}

# R6: modifier-only toggle alone still flips exactly once per press, when the
# configured toggle is the chord under test (Ctrl+Shift) or Alt+Shift.
if ($toggleCfg -eq 'Ctrl+Shift' -or $toggleCfg -eq 'Alt+Shift') {
  $focused = FocusTarget
  # ensure ON
  $r = TypeAndRead 'ba6t5'
  if ($r.Text -cne 'bật') {
    if ($toggleCfg -eq 'Ctrl+Shift') { [QAChord]::TapCtrlShift(90) } else { [QAChord]::TapAltShift(90) }
    Start-Sleep -Milliseconds 400
  }
  if ($toggleCfg -eq 'Ctrl+Shift') { [QAChord]::TapCtrlShift(90) } else { [QAChord]::TapAltShift(90) }
  Start-Sleep -Milliseconds 450
  $r1 = TypeAndRead 'ba6t5'
  Row 'R6-toggle-off' "$toggleCfg tap 1" 'ba6t5' $r1.Text $r1.Via $(if ($r1.Text -ceq 'ba6t5') { 'PASS' } else { 'FAIL' })
  if ($toggleCfg -eq 'Ctrl+Shift') { [QAChord]::TapCtrlShift(90) } else { [QAChord]::TapAltShift(90) }
  Start-Sleep -Milliseconds 450
  $r2 = TypeAndRead 'ba6t5'
  Row 'R7-toggle-on' "$toggleCfg tap 2" 'bật' $r2.Text $r2.Via $(if ($r2.Text -ceq 'bật') { 'PASS' } else { 'FAIL' })
}

# R8: a third combo sharing the modifier set (open_settings) must still fire and
# must not be swallowed by the toggle.
if ($settingsCfg -eq 'Ctrl+Shift+S') {
  $focused = FocusTarget
  Clear-TextArea $hwnd
  [QAChord]::Chord(0x53, 'KSC')
  Start-Sleep -Milliseconds 900
  $win = Get-TopWindow 'VietKB' 0
  $titles = @()
  foreach ($h in [QAInput]::TopWindows()) {
    $tt = [QAInput]::Title($h)
    if ($tt) { $titles += $tt }
  }
  $hit = $titles | Where-Object { $_ -like '*VietKB*' } | Select-Object -First 3
  Row 'R8-settings-chord' 'Ctrl+Shift+S' 'VietKB settings window' (($hit -join ' | ') -replace "`t", ' ') 'window titles' $(if ($hit) { 'PASS' } else { 'FAIL' })
  foreach ($h in [QAInput]::TopWindows()) {
    if (([QAInput]::Title($h)) -like '*VietKB*') { [void][QAInput]::Focus($h); Start-Sleep -Milliseconds 200; [QAInput]::Combo(0x1B, $false, $false, $false); Start-Sleep -Milliseconds 300 }
  }
}

Clear-TextArea $hwnd
$hkAfter = HotkeyCount
Write-Host "HOOK hotkey-fires-in-log=$($hkAfter - $hkBefore) sendinput-failures=$([QAChord]::Failures)"
if ($hkAfter -gt $hkBefore) {
  Select-String -Path $logPath -Pattern 'stage=hotkey' | Select-Object -Last 3 | ForEach-Object { Write-Host "  $($_.Line)" }
}
Write-Host "RESTORE-DONE $App $Tag"
