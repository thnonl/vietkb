# verify-extra.ps1 — VS Code required rows (fresh session) + a classic Win32 EDIT
# (the Run dialog) as the legacy control. WordPad is absent on this Windows build.
. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class WinKey {
  [DllImport("user32.dll")] static extern uint SendInput(uint n, INPUT[] p, int cb);
  [StructLayout(LayoutKind.Sequential)] public struct KEYBDINPUT { public ushort wVk; public ushort wScan; public uint dwFlags; public uint time; public IntPtr extra; }
  [StructLayout(LayoutKind.Sequential)] public struct MOUSEINPUT { public int dx; public int dy; public uint data; public uint flags; public uint time; public IntPtr extra; }
  [StructLayout(LayoutKind.Explicit)] public struct InputUnion { [FieldOffset(0)] public MOUSEINPUT mi; [FieldOffset(0)] public KEYBDINPUT ki; }
  [StructLayout(LayoutKind.Sequential)] public struct INPUT { public uint type; public InputUnion u; }
  static INPUT Mk(ushort vk, uint flags) { INPUT i = new INPUT(); i.type = 1; i.u.ki.wVk = vk; i.u.ki.dwFlags = flags; return i; }
  public static void Down(ushort vk) { INPUT[] a = { Mk(vk, 0) }; SendInput(1, a, Marshal.SizeOf(typeof(INPUT))); }
  public static void Up(ushort vk) { INPUT[] a = { Mk(vk, 2) }; SendInput(1, a, Marshal.SizeOf(typeof(INPUT))); }
  public static void Chord(ushort mod, ushort key) { Down(mod); System.Threading.Thread.Sleep(20); Down(key); System.Threading.Thread.Sleep(20); Up(key); Up(mod); }
}
'@
$dir = 'E:\vietkb\docs\evkey-clone-work\qa\e2e95'
$out = @()

function Row($app, $id, $keys, $exp, $act) {
  $v = if ($act -ceq $exp) { 'PASS' } else { 'FAIL' }
  $script:out += [pscustomobject]@{ App = $app; Id = $id; Keys = $keys; Expected = $exp; Actual = $act; Verdict = $v }
  "{0,-8} {1,-22} keys=[{2,-22}] exp=[{3,-12}] act=[{4,-12}] {5}" -f $app, $id, $keys, $exp, $act, $v
}

# ---------------- VS Code ----------------
$f = Join-Path $dir 'scratch-vsc2.txt'
Remove-Item $f -ErrorAction SilentlyContinue; New-Item -ItemType File -Path $f | Out-Null
$code = "$env:LOCALAPPDATA\Programs\Microsoft VS Code\Code.exe"
Start-Process $code -ArgumentList @('-n', $f) | Out-Null
$w = Wait-Window 'scratch-vsc2.txt' 40
if (-not $w) { 'VSCODE-SETUP-FAIL' } else {
  $hwnd = $w.Hwnd
  Start-Sleep -Milliseconds 2500
  [void][QAInput]::Focus($hwnd); Start-Sleep -Milliseconds 200
  Click-Center $hwnd; Start-Sleep -Milliseconds 200
  [QAInput]::TypeAscii('warm', 55); Start-Sleep -Milliseconds 600
  [QAInput]::Combo(0x53, $false, $true, $false); Start-Sleep -Milliseconds 700
  $warm = [System.IO.File]::ReadAllText($f, [System.Text.Encoding]::UTF8)
  "vscode warm=[$warm]"
  [QAInput]::Combo(0x70, $false, $false, $false)  # F1 Vietnamese ON
  Start-Sleep -Milliseconds 400

  $cases = @(
    @('A1-bat-slash-tat',   'ba6t5/ta8t1', 'bật/tắt'),
    @('A2-bat-dot-tat',     'ba6t5.ta8t1', 'bật.tắt'),
    @('A4-bat-underscore',  'ba6t5_ta8t1', 'bật_tắt'),
    @('A5-windows-path',    'C:\Users\nam', 'C:\Users\nam'),
    @('B1-bs-tone',         'ba6t5',       'bậ'),
    @('C2-macro-m01',       'm01 ',        'mục 1 ')
  )
  foreach ($c in $cases) {
    [void][QAInput]::Focus($hwnd); Start-Sleep -Milliseconds 150
    Click-Center $hwnd; Start-Sleep -Milliseconds 150
    Clear-TextArea $hwnd; Start-Sleep -Milliseconds 150
    [QAInput]::TypeAscii($c[1], 55)
    Start-Sleep -Milliseconds 500
    [QAInput]::Combo(0x53, $false, $true, $false)
    Start-Sleep -Milliseconds 800
    $t = ([System.IO.File]::ReadAllText($f, [System.Text.Encoding]::UTF8)).TrimEnd("`r", "`n")
    Row 'vscode' $c[0] $c[1] $c[2] $t
  }
}

# ---------------- legacy Win32 EDIT: the Run dialog ----------------
[WinKey]::Chord(0x5B, 0x52)   # Win+R
Start-Sleep -Milliseconds 900
$run = Get-TopWindow 'Run' 10
if (-not $run) { $run = Get-TopWindow 'Chạy' 5 }
if (-not $run) {
  'LEGACY-SETUP-FAIL (Run dialog not found)'
} else {
  $rh = $run.Hwnd
  $root = [System.Windows.Automation.AutomationElement]::FromHandle($rh)
  $edit = $root.FindFirst([System.Windows.Automation.TreeScope]::Descendants,
    (New-Object System.Windows.Automation.PropertyCondition(
      [System.Windows.Automation.AutomationElement]::ControlTypeProperty,
      [System.Windows.Automation.ControlType]::Edit)))
  if (-not $edit) { 'LEGACY-SETUP-FAIL (no Edit in Run dialog)' }
  else {
    "legacy control: $($run.Title) / $($edit.Current.ClassName)"
    $vp = $edit.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern)
    $cases = @(
      @('L1-bat-slash-tat', 'ba6t5/ta8t1', 'bật/tắt'),
      @('L2-bat-dot-tat',   'ba6t5.ta8t1', 'bật.tắt'),
      @('L4-bat-underscore','ba6t5_ta8t1', 'bật_tắt'),
      @('L5-windows-path',  'C:\Users\nam', 'C:\Users\nam'),
      @('L6-bs-tone',       'ba6t5',       'bậ')
    )
    foreach ($c in $cases) {
      [void][QAInput]::Focus($rh); Start-Sleep -Milliseconds 150
      $vp.SetValue(''); Start-Sleep -Milliseconds 80
      [QAInput]::TypeAscii($c[1], 55)
      Start-Sleep -Milliseconds 450
      Row 'run-dialog' $c[0] $c[1] $c[2] ($vp.Current.Value)
    }
    [WinKey]::Down(0x1B); [WinKey]::Up(0x1B)   # Esc: close without running anything
    Start-Sleep -Milliseconds 400
  }
}
''
"--- summary: $((($out | Where-Object Verdict -eq 'PASS').Count))/$($out.Count) PASS"
