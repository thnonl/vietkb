# lab-matrix8.ps1 — mixed encoding: does a run of consecutive NON-ASCII still collapse?
. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
public static class LabA {
  [DllImport("user32.dll")] public static extern uint SendInput(uint n, INPUT[] p, int cb);
  [DllImport("user32.dll")] public static extern uint MapVirtualKeyW(uint code, uint type);
  [DllImport("user32.dll")] public static extern short VkKeyScanW(char c);
  [StructLayout(LayoutKind.Sequential)] public struct KEYBDINPUT { public ushort wVk; public ushort wScan; public uint dwFlags; public uint time; public IntPtr extra; }
  [StructLayout(LayoutKind.Sequential)] public struct MOUSEINPUT { public int dx; public int dy; public uint data; public uint flags; public uint time; public IntPtr extra; }
  [StructLayout(LayoutKind.Explicit)] public struct InputUnion { [FieldOffset(0)] public MOUSEINPUT mi; [FieldOffset(0)] public KEYBDINPUT ki; }
  [StructLayout(LayoutKind.Sequential)] public struct INPUT { public uint type; public InputUnion u; }
  public const uint KEYUP = 0x0002, UNICODE = 0x0004;
  public static readonly IntPtr MAGIC = (IntPtr)0x5649454B;
  public static INPUT Mk(ushort vk, ushort scan, uint flags) {
    INPUT i = new INPUT(); i.type = 1; i.u.ki.wVk = vk; i.u.ki.wScan = scan; i.u.ki.dwFlags = flags; i.u.ki.extra = MAGIC; return i; }
  public static ushort ScanFor(char c) { short m = VkKeyScanW(c); return (ushort)MapVirtualKeyW((uint)((byte)(m & 0xFF)), 0); }
  public static void PushVk(List<INPUT> a, char c) {
    short m = VkKeyScanW(c); byte vk = (byte)(m & 0xFF); byte st = (byte)((m >> 8) & 0xFF);
    if ((st & 1) != 0) a.Add(Mk(0x10, (ushort)MapVirtualKeyW(0x10, 0), 0));
    a.Add(Mk(vk, (ushort)MapVirtualKeyW(vk, 0), 0)); a.Add(Mk(vk, (ushort)MapVirtualKeyW(vk, 0), KEYUP));
    if ((st & 1) != 0) a.Add(Mk(0x10, (ushort)MapVirtualKeyW(0x10, 0), KEYUP)); }
  public static void PushU(List<INPUT> a, char c) { a.Add(Mk(0, c, UNICODE)); a.Add(Mk(0, c, UNICODE | KEYUP)); }
  public static void Bs(List<INPUT> a, int n) { for (int k = 0; k < n; k++) { a.Add(Mk(0x08, 0, 0)); a.Add(Mk(0x08, 0, KEYUP)); } }
  public static uint SendAll(List<INPUT> a) { var arr = a.ToArray(); return SendInput((uint)arr.Length, arr, Marshal.SizeOf(typeof(INPUT))); }
}
'@
$w = Wait-Window 'scratch-lab.txt' 20; $hwnd = $w.Hwnd
function Prep {
  [void](Repair-Notepad 'scratch-lab.txt'); [void][QAInput]::Focus($hwnd); Start-Sleep -Milliseconds 120
  $el = Get-EditElement $hwnd
  if ($el) { [void](Click-Element $el) } else { Click-Center $hwnd }
  Start-Sleep -Milliseconds 120; Clear-TextArea $hwnd; Start-Sleep -Milliseconds 150 }
function Read2 { return ([string](Read-Text $hwnd).Text).TrimEnd("`r", "`n") }

function Trial($label, [string]$prefix, [int]$del, [string]$ins, [int]$n) {
  $exp = $prefix.Substring(0, [Math]::Max(0, $prefix.Length - $del)) + $ins
  $ok = 0; $bad = @()
  for ($i = 0; $i -lt $n; $i++) {
    Prep
    if ($prefix) { [QAInput]::TypeAscii($prefix, 60); Start-Sleep -Milliseconds 250 }
    $l = New-Object 'System.Collections.Generic.List[LabA+INPUT]'
    [LabA]::Bs($l, $del)
    foreach ($c in $ins.ToCharArray()) { if ([int]$c -lt 128 -and [LabA]::VkKeyScanW($c) -ne -1) { [LabA]::PushVk($l,$c) } else { [LabA]::PushU($l,$c) } }
    [void][LabA]::SendAll($l)
    Start-Sleep -Milliseconds 500
    $t = Read2
    if ($t -ceq $exp) { $ok++ } else { $bad += $t }
  }
  "{0,-34} {1}/{2}  exp=[{3}]  got={4}" -f $label, $ok, $n, $exp, (($bad | Select-Object -Unique) -join ' , ')
}

Trial 'replace2 tan (1 non-ascii)'   'bật/ta' 2 'tă'   5
Trial 'replace3 duoc (3 non-ascii)'  'bật/ta' 3 'được' 5
Trial 'replace2 đư (2 non-ascii)'    'abc'    2 'đư'   5
Trial 'replace4 Nguyễn'              'ta'     2 'Nguyễn' 5
Trial 'replace1 ư (1 non-ascii)'     'abc'    1 'ư'    5
