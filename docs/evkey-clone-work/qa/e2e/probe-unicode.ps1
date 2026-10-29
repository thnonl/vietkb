$ErrorActionPreference = 'Continue'
$Out = 'C:\share\e2e\probe-unicode.txt'
Set-Content -Encoding UTF8 $Out 'PROBE-START'
function Log($s) { Add-Content -Encoding UTF8 $Out $s }

Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public class U {
  [DllImport("user32.dll")] public static extern uint SendInput(uint n, INPUT[] i, int cb);
  [StructLayout(LayoutKind.Sequential)] public struct INPUT { public uint type; public KEYBDINPUT ki; public int pad; }
  [StructLayout(LayoutKind.Sequential)] public struct KEYBDINPUT { public ushort wVk; public ushort wScan; public uint dwFlags; public uint time; public UIntPtr dwExtraInfo; }
}
'@

$form = New-Object Windows.Forms.Form
$form.Text = 'unicode probe'
$form.StartPosition = 'Manual'
$form.Location = New-Object Drawing.Point 100, 100
$form.Size = New-Object Drawing.Size 600, 120
$tb = New-Object Windows.Forms.TextBox
$tb.Multiline = $true; $tb.Dock = 'Fill'; $tb.Font = New-Object Drawing.Font('Segoe UI', 16)
$form.Controls.Add($tb)
$form.Show(); $form.Activate(); $form.ActiveControl = $tb; [void]$tb.Focus()
[Windows.Forms.Application]::DoEvents()

function Pump([int]$ms) {
  $end = (Get-Date).AddMilliseconds($ms)
  while ((Get-Date) -lt $end) { [Windows.Forms.Application]::DoEvents(); Start-Sleep -Milliseconds 10 }
}

function Uni([int]$code) {
  $i = New-Object U+INPUT; $i.type = 1; $i.ki.wVk = 0; $i.ki.wScan = [uint16]$code; $i.ki.dwFlags = 4; $i.ki.dwExtraInfo = [UIntPtr]::Zero
  $a = [U]::SendInput(1, @($i), [Runtime.InteropServices.Marshal]::SizeOf([type][U+INPUT]))
  $j = New-Object U+INPUT; $j.type = 1; $j.ki.wVk = 0; $j.ki.wScan = [uint16]$code; $j.ki.dwFlags = 6; $j.ki.dwExtraInfo = [UIntPtr]::Zero
  $b = [U]::SendInput(1, @($j), [Runtime.InteropServices.Marshal]::SizeOf([type][U+INPUT]))
  Pump 200
  return "down=$a up=$b"
}

$tb.Clear(); $form.ActiveControl = $tb; [void]$tb.Focus(); Pump 100
$r1 = Uni 0x00EA
Log "ascii_probe send=[$r1] text=[$($tb.Text)]"
$tb.Clear(); Pump 100
$r2 = Uni 0x1EC7
Log "viet_probe send=[$r2] text=[$($tb.Text)]"

Log 'PROBE-DONE'
shutdown /s /t 5
