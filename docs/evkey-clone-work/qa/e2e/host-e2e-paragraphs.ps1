# End-to-end typing test on the real desktop.
#
# Types whole paragraphs (corpus/*.txt) plus keyboard-level edge cases through the
# hook into this process' own text box, then compares the text field with the
# authored paragraph. Keys come from encode-keys.ps1, so the same paragraph is
# typed once in Telex and once in VNI.
#
#   powershell -NoProfile -ExecutionPolicy Bypass -File host-e2e-paragraphs.ps1
#   ... -Only telex          # one method
#   ... -Corpus 01,02        # only those paragraph numbers
param(
    [ValidateSet('telex', 'vni', 'both')][string]$Only = 'both',
    [string[]]$Corpus = @(),
    [int]$KeyDelayMs = 45
)
$ErrorActionPreference = 'Continue'
$Corpus = @($Corpus | ForEach-Object { $_ -split ',' } | Where-Object { $_ })
$here = Split-Path -Parent $MyInvocation.MyCommand.Path
. (Join-Path $here 'encode-keys.ps1')

$Out = Join-Path $here 'host-paragraph-results.txt'
Set-Content -Encoding UTF8 $Out 'HOST-PARAGRAPH-E2E-START'
function Log($s) { Add-Content -Encoding UTF8 $Out $s; Write-Host $s }

$exe = Join-Path (Split-Path -Parent (Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $here)))) 'target-dev\release\vietkb.exe'
if (-not (Test-Path $exe)) { Log "FAIL missing $exe"; return }

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
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int cmd);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, int dx, int dy, uint d, IntPtr e);
}
'@
try { [void][H]::SetProcessDpiAwarenessContext([IntPtr](-4)) } catch {}

$form = New-Object Windows.Forms.Form
$form.Text = 'VietKB paragraph probe'
$form.StartPosition = 'Manual'
$form.Location = New-Object Drawing.Point 120, 120
$form.Size = New-Object Drawing.Size 760, 220
$tb = New-Object Windows.Forms.TextBox
$tb.Multiline = $true
$tb.AcceptsReturn = $true
$tb.ScrollBars = 'Vertical'
$tb.Dock = 'Fill'
$tb.Font = New-Object Drawing.Font('Segoe UI', 14)
$form.Controls.Add($tb)
$form.Show()
$form.TopMost = $true
$form.Activate()
$form.ActiveControl = $tb
[void]$tb.Focus()
[Windows.Forms.Application]::DoEvents()

function Pump([int]$ms) {
    $end = (Get-Date).AddMilliseconds($ms)
    while ((Get-Date) -lt $end) { [Windows.Forms.Application]::DoEvents(); Start-Sleep -Milliseconds 8 }
}

function FocusForm {
    for ($try = 0; $try -lt 4; $try++) {
        $fg = [H]::GetForegroundWindow()
        if ($fg -eq $form.Handle) { break }
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
        Pump 120
        if ([H]::GetForegroundWindow() -ne $form.Handle) {
            # Foreground lock: a real click on the form is the reliable way in.
            $pt = $form.PointToScreen((New-Object Drawing.Point 30, 30))
            [void][H]::SetCursorPos($pt.X, $pt.Y)
            [H]::mouse_event(0x02, 0, 0, 0, [IntPtr]::Zero)
            [H]::mouse_event(0x04, 0, 0, 0, [IntPtr]::Zero)
            Pump 150
        }
    }
    $form.TopMost = $true
    $form.Activate()
    $form.ActiveControl = $tb
    [void]$tb.Focus()
    Pump 60
    return ([H]::GetForegroundWindow() -eq $form.Handle)
}

# SendKeys treats these as operators; wrap them in braces to type them literally.
$special = '+^%~(){}[]'
function TypeText([string]$s) {
    foreach ($ch in $s.ToCharArray()) {
        $c = [string]$ch
        if ($special.Contains($c)) { [Windows.Forms.SendKeys]::SendWait('{' + $c + '}') }
        else { [Windows.Forms.SendKeys]::SendWait($c) }
        Pump $KeyDelayMs
    }
}
function Backspace([int]$n) { 1..$n | ForEach-Object { [Windows.Forms.SendKeys]::SendWait('{BS}'); Pump 60 } }
function Press([string]$name) { [Windows.Forms.SendKeys]::SendWait("{$name}"); Pump 150 }

# The app grabs the foreground for a moment after it starts; wait until the probe
# form has held it for a few consecutive samples before typing anything.
function Ensure-Foreground([int]$timeoutMs = 8000) {
    $end = (Get-Date).AddMilliseconds($timeoutMs)
    $stable = 0
    while ((Get-Date) -lt $end) {
        $null = FocusForm
        if ([H]::GetForegroundWindow() -eq $form.Handle) {
            $stable++
            if ($stable -ge 3) { return $true }
        } else { $stable = 0 }
        Pump 100
    }
    return $false
}

function Case([string]$id, [string]$expected, [scriptblock]$act) {
    [Windows.Forms.SendKeys]::SendWait('{ESC}')
    Pump 80
    $tb.Clear(); Pump 100
    $focused = Ensure-Foreground
    Log ("focus {0} fg_match={1} tb_focused={2}" -f $id, $focused, $tb.Focused)
    & $act
    Pump 500
    $actual = $tb.Text
    if ($actual -ceq $expected) {
        Log ("CASE {0} chars={1} PASS" -f $id, $expected.Length)
    } else {
        Log ("CASE {0} FAIL" -f $id)
        Log ("  expected=[{0}]" -f $expected)
        Log ("  actual  =[{0}]" -f $actual)
        $ew = $expected -split '(\s+)'
        $aw = $actual -split '(\s+)'
        for ($i = 0; $i -lt [Math]::Max($ew.Count, $aw.Count); $i++) {
            $a = if ($i -lt $ew.Count) { $ew[$i] } else { '<none>' }
            $b = if ($i -lt $aw.Count) { $aw[$i] } else { '<none>' }
            if ($a -cne $b) { Log ("  first_diff token#{0} expected=[{1}] actual=[{2}]" -f $i, $a, $b); break }
        }
    }
}

function Start-App([string]$method) {
    Get-Process vietkb -ErrorAction SilentlyContinue | Stop-Process -Force
    Pump 500
    $cfgDir = Join-Path $env:APPDATA 'VietKB Dev'
    New-Item -ItemType Directory -Force $cfgDir | Out-Null
    Set-Content -Encoding UTF8 (Join-Path $cfgDir 'config.toml') "[input]`r`nmethod = `"$method`"`r`n"
    Start-Process $exe | Out-Null
    $end = (Get-Date).AddSeconds(25)
    while ((Get-Date) -lt $end) {
        Pump 200
        $p = Get-Process vietkb -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($p) {
            Pump 1500
            # The app's panel would otherwise hold the foreground and our probe
            # form could never receive the keystrokes.
            if ($p.MainWindowHandle -ne [IntPtr]::Zero) { [void][H]::ShowWindow($p.MainWindowHandle, 6) }
            Pump 300
            # First paint of the webview can keep stealing focus for a while; do
            # not start typing until the probe form has really held it.
            $warmDeadline = (Get-Date).AddSeconds(60)
            $warm = $false
            while ((Get-Date) -lt $warmDeadline) {
                if (Ensure-Foreground 5000) { $warm = $true; break }
                Pump 500
            }
            Log "app_ready $method pid=$($p.Id) warmup=$warm"
            return $p
        }
    }
    Log "app_ready $method FAIL"
    return $null
}

$methods = if ($Only -eq 'both') { @('telex', 'vni') } else { @($Only) }
$files = Get-ChildItem (Join-Path $here 'corpus') -Filter *.txt | Sort-Object Name
if ($Corpus.Count -gt 0) { $files = $files | Where-Object { $Corpus -contains $_.BaseName.Substring(0, 2) } }

foreach ($m in $methods) {
    if (-not (Start-App $m)) { break }
    Log "PHASE $m paragraphs"
    foreach ($file in $files) {
        $expected = (Get-Content $file.FullName -Raw -Encoding UTF8).Trim()
        $keys = ConvertTo-Keys $expected $m
        Case ("PARA-{0}-{1}" -f $file.BaseName, $m) $expected { TypeText $keys }.GetNewClosure()
    }

    Log "PHASE $m edge cases"
    Case "E01-backspace-$m" 'ti' { TypeText $(if ($m -eq 'vni') { 'tie6ng1' } else { 'tieengs' }); Backspace 3 }.GetNewClosure()
    Case "E02-reopen-space-$m" 'Thá' { TypeText 'Tha'; TypeText ' '; Backspace 1; TypeText $(if ($m -eq 'vni') { '1' } else { 's' }) }.GetNewClosure()
    Case "E03-reopen-punct-$m" 'tiếng,việt' { TypeText $(if ($m -eq 'vni') { 'tie6ng1' } else { 'tieengs' }); TypeText ','; TypeText $(if ($m -eq 'vni') { 'vie6t5' } else { 'vieetj' }) }.GetNewClosure()
    Case "E04-english-$m" 'deadline xong' { TypeText 'deadline xong' }
    Case "E05-clear-$m" 'á' { Backspace 4; TypeText $(if ($m -eq 'vni') { 'a1' } else { 'as' }) }.GetNewClosure()
    Case "E06-enter-reset-$m" "tiếng`r`ná" { TypeText $(if ($m -eq 'vni') { 'tie6ng1' } else { 'tieengs' }); Press 'ENTER'; TypeText $(if ($m -eq 'vni') { 'a1' } else { 'as' }) }.GetNewClosure()
    Case "E07-f2-off-$m" $(if ($m -eq 'vni') { 'tie6ng1' } else { 'tieengs' }) {
        Press 'F2'; TypeText $(if ($m -eq 'vni') { 'tie6ng1' } else { 'tieengs' }); Press 'F1'
    }.GetNewClosure()
}

Get-Process vietkb -ErrorAction SilentlyContinue | Stop-Process -Force
$result = Get-Content $Out -Encoding UTF8
$fails = @($result | Where-Object { $_ -like 'CASE*FAIL*' }).Count
$passes = @($result | Where-Object { $_ -like 'CASE*PASS*' }).Count
Log "E2E_PASS=$passes E2E_FAIL=$fails"
Log 'HOST-PARAGRAPH-E2E-DONE'
$form.Close()
