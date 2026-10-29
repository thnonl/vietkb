$ErrorActionPreference = 'Continue'
$Out = 'E:\vietkb\sandbox-share\e2e\host-results.txt'
New-Item -ItemType Directory -Force (Split-Path $Out) | Out-Null
Set-Content -Encoding UTF8 $Out 'HOST-E2E-START'
function Log($s) { Add-Content -Encoding UTF8 $Out $s; Write-Host $s }

$prod = Join-Path $env:APPDATA 'VietKB'
$prodBefore = @()
$hashBefore = 'absent'
if (Test-Path $prod) {
  $prodBefore = @(Get-ChildItem $prod -File -Recurse -Force | ForEach-Object { $_.FullName.Substring($prod.Length) } | Sort-Object)
  $cf = Join-Path $prod 'config.toml'
  if (Test-Path $cf) { $hashBefore = (Get-FileHash $cf -Algorithm SHA256).Hash }
}
Log "prod_before files=$($prodBefore -join ',') config=$hashBefore"

Get-Process vietkb -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Milliseconds 500

Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public class H {
  [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr v);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint p);
  [DllImport("user32.dll")] public static extern bool AttachThreadInput(uint a, uint b, bool f);
  [DllImport("kernel32.dll")] public static extern uint GetCurrentThreadId();
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool BringWindowToTop(IntPtr h);
}
'@
try { [void][H]::SetProcessDpiAwarenessContext([IntPtr](-4)) } catch {}

$form = New-Object Windows.Forms.Form
$form.Text = 'VietKB host probe'
$form.StartPosition = 'Manual'
$form.Location = New-Object Drawing.Point 120, 120
$form.Size = New-Object Drawing.Size 700, 160
$tb = New-Object Windows.Forms.TextBox
$tb.Multiline = $true
$tb.Dock = 'Fill'
$tb.Font = New-Object Drawing.Font('Segoe UI', 16)
$form.Controls.Add($tb)
$form.Show()
$form.Activate()
$form.ActiveControl = $tb
[void]$tb.Focus()
[Windows.Forms.Application]::DoEvents()

function Pump([int]$ms) {
  $end = (Get-Date).AddMilliseconds($ms)
  while ((Get-Date) -lt $end) { [Windows.Forms.Application]::DoEvents(); Start-Sleep -Milliseconds 10 }
}

function FocusForm {
  $fg = [H]::GetForegroundWindow()
  if ($fg -ne $form.Handle) {
    $t1 = [uint32]0; $t2 = [uint32]0
    [void][H]::GetWindowThreadProcessId($form.Handle, [ref]$t1)
    [void][H]::GetWindowThreadProcessId($fg, [ref]$t2)
    $me = [H]::GetCurrentThreadId()
    [void][H]::AttachThreadInput($me, $t2, $true)
    [void][H]::AttachThreadInput($me, $t1, $true)
    [void][H]::BringWindowToTop($form.Handle)
    [void][H]::SetForegroundWindow($form.Handle)
    [void][H]::AttachThreadInput($me, $t1, $false)
    [void][H]::AttachThreadInput($me, $t2, $false)
    Pump 150
  }
  $form.ActiveControl = $tb
  [void]$tb.Focus()
  Pump 80
  $script:LastFocus = "fg_match=$([H]::GetForegroundWindow() -eq $form.Handle) tb_focused=$($tb.Focused)"
  return ([H]::GetForegroundWindow() -eq $form.Handle)
}

function TypeText([string]$s) {
  foreach ($ch in $s.ToCharArray()) {
    if ($ch -eq ' ') { [Windows.Forms.SendKeys]::SendWait(' ') }
    else { [Windows.Forms.SendKeys]::SendWait([string]$ch) }
    Pump 60
  }
}
function Backspace([int]$n) { 1..$n | ForEach-Object { [Windows.Forms.SendKeys]::SendWait('{BS}'); Pump 60 } }
function Press([string]$name) { [Windows.Forms.SendKeys]::SendWait("{$name}"); Pump 150 }

function Case([string]$id, [string]$expected, [scriptblock]$act) {
  [Windows.Forms.SendKeys]::SendWait('{ESC}')
  Pump 100
  $tb.Clear(); Pump 120
  $afterClear = $tb.Text
  $focused = FocusForm; Pump 100
  Log "focus $id $script:LastFocus after_clear=[$afterClear]"
  & $act
  Pump 400
  $actual = $tb.Text
  $ok = ($actual -ceq $expected)
  if ($ok) { Log ("CASE {0} expected=[{1}] actual=[{2}] PASS" -f $id, $expected, $actual) }
  else { Log ("CASE {0} expected=[{1}] actual=[{2}] FAIL" -f $id, $expected, $actual) }
}

function Start-App {
  $env:VKB_DEBUG_KEYS = '1'
  Start-Process 'E:\vietkb\target-dev\release\vietkb.exe' | Out-Null
  $end = (Get-Date).AddSeconds(20)
  while ((Get-Date) -lt $end) {
    Pump 200
    $p = Get-Process vietkb -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($p) { Log "app_wait=PASS pid=$($p.Id)"; Pump 1500; return $p }
  }
  Log 'app_wait=FAIL'
  return $null
}

$app = Start-App
if (-not $app) { Log 'HOST-E2E-DONE'; return }

Stop-Process -Name vietkb -Force -ErrorAction SilentlyContinue
Pump 500
$cfgDir = Join-Path $env:APPDATA 'VietKB Dev'
New-Item -ItemType Directory -Force $cfgDir | Out-Null
Set-Content -Encoding UTF8 (Join-Path $cfgDir 'config.toml') "[input]`r`nmethod = `"telex`"`r`n"
Log 'config_telex_written'
$app = Start-App
if (-not $app) { Log 'HOST-E2E-DONE'; return }

Log 'PHASE Telex'
Case 'T1-tieengs' 'tiếng' { TypeText 'tieengs' }
Case 'T2-backspace' 'ti' { TypeText 'tieengs'; Backspace 3 }
Case 'T3-reopen' 'Thá' { TypeText 'Tha'; TypeText ' '; Backspace 1; TypeText 's' }
Case 'T4-horn' 'thương' { TypeText 'thuowng' }
Case 'T5-af' 'à' { TypeText 'af' }
Case 'T6-f2-off' 'tieengs' { Press 'F2'; TypeText 'tieengs' }
Case 'T7-f1-on' 'tiếng' { Press 'F1'; TypeText 'tieengs' }

Stop-Process -Name vietkb -Force -ErrorAction SilentlyContinue
Pump 500
$cfgDir = Join-Path $env:APPDATA 'VietKB Dev'
New-Item -ItemType Directory -Force $cfgDir | Out-Null
Set-Content -Encoding UTF8 (Join-Path $cfgDir 'config.toml') "[input]`r`nmethod = `"vni`"`r`n"
Log 'config_vni_written'
$app = Start-App
if (-not $app) { Log 'HOST-E2E-DONE'; return }

Log 'PHASE VNI'
Case 'V1-tone' 'Thá' { TypeText 'Tha'; TypeText ' '; Backspace 1; TypeText '1' }
Case 'V2-auto-horn' 'thương' { TypeText 'thu7ong' }
Case 'V3-cluster-horn' 'thương' { TypeText 'thuo7ng' }
Case 'V4-quo7' 'quơ' { TypeText 'quo7' }
Case 'V5-tie6ng1' 'tiếng' { TypeText 'tie6ng1' }

Stop-Process -Name vietkb -Force -ErrorAction SilentlyContinue
$keys = Join-Path $env:APPDATA 'VietKB Dev\keys.log'
if (Test-Path $keys) {
  $tail = Get-Content $keys -Tail 200
  Set-Content -Encoding UTF8 'E:\vietkb\sandbox-share\e2e\host-keys.log' $tail
  Log 'keys_log=copied'
}

$prodAfter = @()
$hashAfter = 'absent'
if (Test-Path $prod) {
  $prodAfter = @(Get-ChildItem $prod -File -Recurse -Force | ForEach-Object { $_.FullName.Substring($prod.Length) } | Sort-Object)
  $cf = Join-Path $prod 'config.toml'
  if (Test-Path $cf) { $hashAfter = (Get-FileHash $cf -Algorithm SHA256).Hash }
}
$isolated = ((Compare-Object $prodBefore $prodAfter).Count -eq 0) -and ($hashBefore -eq $hashAfter)
Log "PROFILE_ISOLATED=$isolated"
Log 'HOST-E2E-DONE'
$form.Close()
