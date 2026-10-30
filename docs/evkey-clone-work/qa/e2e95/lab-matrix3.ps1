# lab-matrix3.ps1 — correct predecessor: backspaces ARE in the batch.
. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
public static class Lab5 {
  [DllImport("user32.dll")] public static extern uint SendInput(uint n, INPUT[] p, int cb);
  [DllImport("user32.dll")] public static extern uint MapVirtualKeyW(uint code, uint type);
  [DllImport("user32.dll")] public static extern short VkKeyScanW(char c);
  [StructLayout(LayoutKind.Sequential)] public struct KEYBDINPUT { public ushort wVk; public ushort wScan; public uint dwFlags; public uint time; public IntPtr extra; }
  [StructLayout(LayoutKind.Sequential)] public struct MOUSEINPUT { public int dx; public int dy; public uint data; public uint flags; public uint time; public IntPtr extra; }
  [StructLayout(LayoutKind.Explicit)] public struct InputUnion { [FieldOffset(0)] public MOUSEINPUT mi; [FieldOffset(0)] public KEYBDINPUT ki; }
  [StructLayout(LayoutKind.Sequential)] public struct INPUT { public uint type; public InputUnion u; }
  public const uint KEYUP = 0x0002, UNICODE = 0x0004;
  public static readonly IntPtr MAGIC = (IntPtr)0x5649454B;
  static INPUT Mk(ushort vk, ushort scan, uint flags) {
    INPUT i = new INPUT(); i.type = 1; i.u.ki.wVk = vk; i.u.ki.wScan = scan; i.u.ki.dwFlags = flags; i.u.ki.extra = MAGIC; return i; }
  public static void Bs(List<INPUT> a, int n) { for (int k = 0; k < n; k++) { a.Add(Mk(0x08, 0, 0)); a.Add(Mk(0x08, 0, KEYUP)); } }
  public static void U(List<INPUT> a, char c) { a.Add(Mk(0, c, UNICODE)); a.Add(Mk(0, c, UNICODE | KEYUP)); }
  public static void Vk(List<INPUT> a, char c) {
    short m = VkKeyScanW(c); byte vk = (byte)(m & 0xFF); byte st = (byte)((m >> 8) & 0xFF);
    if ((st & 1) != 0) a.Add(Mk(0x10, (ushort)MapVirtualKeyW(0x10, 0), 0));
    a.Add(Mk(vk, (ushort)MapVirtualKeyW(vk, 0), 0)); a.Add(Mk(vk, (ushort)MapVirtualKeyW(vk, 0), KEYUP));
    if ((st & 1) != 0) a.Add(Mk(0x10, (ushort)MapVirtualKeyW(0x10, 0), KEYUP)); }
  public static uint SendOne(INPUT i) { INPUT[] one = { i }; return SendInput(1, one, Marshal.SizeOf(typeof(INPUT))); }
  public static uint SendAll(List<INPUT> a) { var arr = a.ToArray(); return SendInput((uint)arr.Length, arr, Marshal.SizeOf(typeof(INPUT))); }
  public static uint SendBlock(List<INPUT> a, int from, int count) {
    if (count <= 0) return 0;
    INPUT[] b = a.GetRange(from, count).ToArray();
    return SendInput((uint)b.Length, b, Marshal.SizeOf(typeof(INPUT))); }
}
'@
if (-not (Get-Process notepad -ErrorAction SilentlyContinue)) {
  $f = 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\scratch-lab.txt'
  Remove-Item $f -ErrorAction SilentlyContinue; New-Item -ItemType File -Path $f | Out-Null
  Start-Process notepad.exe -ArgumentList $f | Out-Null; Start-Sleep -Milliseconds 1500 }
$w = Wait-Window 'scratch-lab.txt' 20; $hwnd = $w.Hwnd
function Prep {
  [void](Repair-Notepad 'scratch-lab.txt'); [void][QAInput]::Focus($hwnd); Start-Sleep -Milliseconds 120
  $el = Get-EditElement $hwnd
  if ($el) { [void](Click-Element $el) } else { Click-Center $hwnd }
  Start-Sleep -Milliseconds 120; Clear-TextArea $hwnd; Start-Sleep -Milliseconds 120 }
function Read2 { return ([string](Read-Text $hwnd).Text).TrimEnd("`r", "`n") }

function Case($name, [string]$prefix, [int]$del, [string]$ins, [string]$how) {
  Prep
  if ($prefix) { [QAInput]::TypeAscii($prefix, 60); Start-Sleep -Milliseconds 250 }
  $l = New-Object 'System.Collections.Generic.List[Lab5+INPUT]'
  [Lab5]::Bs($l, $del)
  foreach ($c in $ins.ToCharArray()) { [Lab5]::U($l, $c) }
  $cut = $del * 2
  switch ($how) {
    'all'        { [void][Lab5]::SendAll($l) }
    'two-calls'  { [void][Lab5]::SendBlock($l, 0, $cut); Start-Sleep -Milliseconds 1; [void][Lab5]::SendBlock($l, $cut, $l.Count - $cut) }
    'per-event'  { foreach ($e in $l) { [void][Lab5]::SendOne($e) } }
    'after-del-1ms-split-text' { [void][Lab5]::SendBlock($l, 0, $cut); Start-Sleep -Milliseconds 1
                                 for ($i = $cut; $i -lt $l.Count; $i += 2) { [void][Lab5]::SendBlock($l, $i, 2) } }
  }
  Start-Sleep -Milliseconds 550
  $r = Read2
  $exp = $prefix.Substring(0, [Math]::Max(0, $prefix.Length - $del)) + $ins
  "{0,-24} del={1} ins=[{2,-4}] how={3,-24} exp=[{4,-10}] got=[{5}] {6}" -f $name, $del, $ins, $how, $exp, $r, $(if ($r -ceq $exp) {'OK'} else {'BROKEN'})
}

Case 'd2-tx'        'bật/ta' 2 'tx'  all
Case 'd2-tx-2calls' 'bật/ta' 2 'tx'  two-calls
Case 'd2-tx-perevt' 'bật/ta' 2 'tx'  per-event
Case 'd2-tan'       'bật/ta' 2 'tăn' all
Case 'd2-tan-2calls' 'bật/ta' 2 'tăn' two-calls
Case 'd2-tan-splittext' 'bật/ta' 2 'tăn' 'after-del-1ms-split-text'
Case 'd3-ta-t'      'bật/ta' 3 'tắt' all
Case 'd3-ta-t-2calls' 'bật/ta' 3 'tắt' two-calls
Case 'd3-duoc'      'bật/ta' 3 'được' all
Case 'd3-duoc-2calls' 'bật/ta' 3 'được' two-calls
Case 'd1-tx'        'bật/ta' 1 'tx'  all
Case 'd4-tx'        'bật/ta' 4 'tx'  all
Case 'd2-empty-insert' 'bật/ta' 2 '' all
Case 'd0-tx'        'bật/ta' 0 'tx'  all
