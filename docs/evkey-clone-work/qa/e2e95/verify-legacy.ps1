# verify-legacy.ps1 — classic Win32 EDIT (Character Map's "Characters to copy").
# WordPad is not installed on this Windows build; this is a plain RichEdit50W
# hosted by a legacy Win32 dialog, the control class VB6/Delphi apps embed.
. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using System.Text;
public static class EditText {
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern IntPtr SendMessageW(IntPtr h, uint msg, IntPtr w, StringBuilder l);
  public static string Get(IntPtr h, int len) {
    var sb = new StringBuilder(len);
    SendMessageW(h, 0x000D, (IntPtr)len, sb);   // WM_GETTEXT
    return sb.ToString();
  }
  public static void Clear(IntPtr h) { SendMessageW(h, 0x000C, IntPtr.Zero, null); } // WM_SETTEXT ""
}
'@
Get-Process charmap -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Milliseconds 400
Start-Process "$env:SystemRoot\System32\charmap.exe" | Out-Null
$w = Wait-Window 'Character Map' 20
if (-not $w) { 'LEGACY-SETUP-FAIL window'; exit 1 }
$hwnd = $w.Hwnd
Start-Sleep -Milliseconds 1200
$root = [System.Windows.Automation.AutomationElement]::FromHandle($hwnd)
$edit = $root.FindFirst([System.Windows.Automation.TreeScope]::Descendants,
  (New-Object System.Windows.Automation.PropertyCondition(
    [System.Windows.Automation.AutomationElement]::ControlTypeProperty,
    [System.Windows.Automation.ControlType]::Edit)))
if (-not $edit) { 'LEGACY-SETUP-FAIL edit'; exit 1 }
$eh = [IntPtr]$edit.Current.NativeWindowHandle
"legacy control: '$($w.Title)' class=$($edit.Current.ClassName) hwnd=$eh"

function Type-Into($keys) {
  [void][QAInput]::Focus($hwnd)
  Start-Sleep -Milliseconds 300
  [EditText]::Clear($eh)
  Start-Sleep -Milliseconds 200
  [QAInput]::TypeAscii($keys, 55)
  Start-Sleep -Milliseconds 900
  $a = ([EditText]::Get($eh, 512)).TrimEnd("`r", "`n")
  Start-Sleep -Milliseconds 250
  $b = ([EditText]::Get($eh, 512)).TrimEnd("`r", "`n")
  return $(if ($a.Length -ge $b.Length) { $a } else { $b })
}
# make sure Vietnamese is ON: probe, flip with F1 only if the probe is raw
$probe = Type-Into 'ba6t5'
if ($probe -ne 'bậ') {
  [QAInput]::Combo(0x70, $false, $false, $false)
  Start-Sleep -Milliseconds 400
  $probe = Type-Into 'ba6t5'
}
"probe -> [$probe]"

$cases = @(
  @('L1-bat-slash-tat',  'ba6t5/ta8t1', 'bật/tắt'),
  @('L2-bat-dot-tat',    'ba6t5.ta8t1', 'bật.tắt'),
  @('L4-bat-underscore', 'ba6t5_ta8t1', 'bật_tắt'),
  @('L5-windows-path',   'C:\Users\nam', 'C:\Users\nam'),
  @('L6-bs-tone',        'ba6t5',       'bậ'),
  @('L7-macro-m01',      'm01 ',        'mục 1 ')
)
foreach ($c in $cases) {
  $act = Type-Into $c[1]
  $v = if ($act -ceq $c[2]) { 'PASS' } else { 'FAIL' }
  "legacy   {0,-20} keys=[{1,-16}] exp=[{2,-10}] act=[{3,-10}] {4}" -f $c[0], $c[1], $c[2], $act, $v
}
Get-Process charmap -ErrorAction SilentlyContinue | Stop-Process -Force


