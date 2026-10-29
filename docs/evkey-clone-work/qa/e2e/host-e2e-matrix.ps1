$ErrorActionPreference = 'Continue'
$Out = 'E:\vietkb\sandbox-share\e2e\matrix-results.txt'
Set-Content -Encoding UTF8 $Out 'MATRIX-START'
function Log($s) { Add-Content -Encoding UTF8 $Out $s; Write-Host $s }

$app = 'E:\vietkb\target-dev\release\vietkb.exe'
$cfgDir = Join-Path $env:APPDATA 'VietKB Dev'
$cfg = Join-Path $cfgDir 'config.toml'
New-Item -ItemType Directory -Force $cfgDir | Out-Null

Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public class M {
  [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr v);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint p);
  [DllImport("user32.dll")] public static extern bool AttachThreadInput(uint a, uint b, bool f);
  [DllImport("kernel32.dll")] public static extern uint GetCurrentThreadId();
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool BringWindowToTop(IntPtr h);
}
'@
try { [void][M]::SetProcessDpiAwarenessContext([IntPtr](-4)) } catch {}

$form = New-Object Windows.Forms.Form
$form.Text = 'VietKB matrix probe'
$form.StartPosition = 'Manual'
$form.Location = New-Object Drawing.Point 120, 120
$form.Size = New-Object Drawing.Size 760, 170
$tb = New-Object Windows.Forms.TextBox
$tb.Multiline = $true
$tb.Dock = 'Fill'
$tb.Font = New-Object Drawing.Font('Segoe UI', 16)
$form.Controls.Add($tb)
$form.Show(); $form.Activate(); $form.ActiveControl = $tb; [void]$tb.Focus()
[Windows.Forms.Application]::DoEvents()

function Pump([int]$ms) { $end = (Get-Date).AddMilliseconds($ms); while ((Get-Date) -lt $end) { [Windows.Forms.Application]::DoEvents(); Start-Sleep -Milliseconds 10 } }

function FocusForm {
  $fg = [M]::GetForegroundWindow()
  if ($fg -ne $form.Handle) {
    $t1 = [uint32]0; $t2 = [uint32]0
    [void][M]::GetWindowThreadProcessId($form.Handle, [ref]$t1)
    [void][M]::GetWindowThreadProcessId($fg, [ref]$t2)
    $me = [M]::GetCurrentThreadId()
    [void][M]::AttachThreadInput($me, $t2, $true); [void][M]::AttachThreadInput($me, $t1, $true)
    [void][M]::BringWindowToTop($form.Handle); [void][M]::SetForegroundWindow($form.Handle)
    [void][M]::AttachThreadInput($me, $t1, $false); [void][M]::AttachThreadInput($me, $t2, $false)
    Pump 150
  }
  $form.ActiveControl = $tb; [void]$tb.Focus(); Pump 80
  return ([M]::GetForegroundWindow() -eq $form.Handle)
}

function TypeText([string]$s) { foreach ($ch in $s.ToCharArray()) { if ($ch -eq ' ') { [Windows.Forms.SendKeys]::SendWait(' ') } else { [Windows.Forms.SendKeys]::SendWait([string]$ch) }; Pump 60 } }

function Write-Cfg([string]$method, [string]$typing) {
  $text = "[input]`r`nmethod = `"$method`"`r`n"
  if ($typing) { $text += "[typing]`r`n$typing" }
  $text += "[macros]`r`nvn = `"Việt Nam`"`r`nbtw = `"by the way`"`r`n"
  Set-Content -Encoding UTF8 $cfg $text
}

function Start-App {
  Get-Process vietkb -ErrorAction SilentlyContinue | Stop-Process -Force
  Pump 400
  $env:VKB_DEBUG_KEYS = '1'
  Start-Process $app | Out-Null
  $end = (Get-Date).AddSeconds(20)
  while ((Get-Date) -lt $end) { Pump 200; $p = Get-Process vietkb -ErrorAction SilentlyContinue | Select-Object -First 1; if ($p) { Pump 1200; return $true } }
  return $false
}

function Case([string]$id, [string]$method, [string]$typing, [string]$expected, [string]$keys) {
  Write-Cfg $method $typing
  if (-not (Start-App)) { Log "CASE $id expected=[$expected] actual=[<app did not start>] FAIL"; return }
  [Windows.Forms.SendKeys]::SendWait('{ESC}'); Pump 120
  $tb.Clear(); Pump 120
  if (-not (FocusForm)) { Log "CASE $id expected=[$expected] actual=[<no focus>] FAIL"; return }
  TypeText $keys
  Pump 500
  $actual = $tb.Text
  if ($actual -ceq $expected) { Log ("CASE {0} expected=[{1}] actual=[{2}] PASS" -f $id, $expected, $actual) }
  else { Log ("CASE {0} expected=[{1}] actual=[{2}] FAIL" -f $id, $expected, $actual) }
}

Case 'A1-modern-on' 'telex' "modern_tone = true`r`n" 'hòa' 'hoaf'
Case 'A2-modern-off' 'telex' "modern_tone = false`r`n" 'hoà' 'hoaf'
Case 'A3-free-on' 'telex' "free_tone = true`r`n" 'hoàn' 'hoafn'
Case 'A4-free-off' 'telex' "free_tone = false`r`n" 'hòan' 'hoafn'
Case 'A5-jfwz-on' 'telex' "jfwz_as_consonant = true`r`n" 'w' 'w'
Case 'A6-jfwz-off' 'telex' "jfwz_as_consonant = false`r`n" 'ư' 'w'
Case 'A7-autocap-on' 'telex' "auto_capitalize = true`r`n" 'tiếng. Việt' 'tieengs. vieetj'
Case 'A8-spell-off' 'telex' "spell_check = false`r`n" 'ád' 'asd'
Case 'A9-spell-on' 'telex' "spell_check = true`r`n" 'asd' 'asd'
Case 'A10-macro-on' 'telex' "macro_enabled = true`r`n" 'Việt Nam ' 'vn '
Case 'A11-macro-off' 'telex' "macro_enabled = false`r`n" 'vn ' 'vn '
Case 'A12-case-on' 'telex' "macro_case_follow = true`r`n" 'By the way ' 'Btw '
Case 'A13-case-off' 'telex' "macro_case_follow = false`r`n" 'by the way ' 'Btw '
Case 'V1-modern-on' 'vni' "modern_tone = true`r`n" 'hòa' 'hoa2'
Case 'V2-modern-off' 'vni' "modern_tone = false`r`n" 'hoà' 'hoa2'
Case 'V3-free-on' 'vni' "free_tone = true`r`n" 'hoàn' 'hoan2'
Case 'V4-free-off' 'vni' "free_tone = false`r`n" 'hòan' 'hoa2n'
Case 'V5-spell-off' 'vni' "spell_check = false`r`n" 'ád' 'a1d'
Case 'V6-macro' 'vni' "macro_enabled = true`r`n" 'Việt Nam ' 'vn '
Case 'V7-case-off' 'vni' "macro_case_follow = false`r`n" 'by the way ' 'Btw '

Get-Process vietkb -ErrorAction SilentlyContinue | Stop-Process -Force
Log 'MATRIX-DONE'
$form.Close()
