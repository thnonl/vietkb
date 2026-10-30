# lab-matrix4.ps1 — find a construction that is reliably correct.
. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
public static class Lab6 {
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
  public static void Bs(List<INPUT> a, int n) { for (int k = 0; k < n; k++) { a.Add(Mk(0x08, 0, 0)); a.Add(Mk(0x08, 0, KEYUP)); } }
  public static void BsPacket(List<INPUT> a, int n) { for (int k = 0; k < n; k++) { a.Add(Mk(0, 8, UNICODE)); a.Add(Mk(0, 8, UNICODE|KEYUP)); } }
  public static void U(List<INPUT> a, char c) { a.Add(Mk(0, c, UNICODE)); a.Add(Mk(0, c, UNICODE | KEYUP)); }
  public static void UDownOnly(List<INPUT> a, char c) { a.Add(Mk(0, c, UNICODE)); }
  public static void Shift(List<INPUT> a) { ushort s = (ushort)MapVirtualKeyW(0x10, 0); a.Add(Mk(0x10, s, 0)); a.Add(Mk(0x10, s, KEYUP)); }
  public static void Ctrl(List<INPUT> a) { ushort s = (ushort)MapVirtualKeyW(0x11, 0); a.Add(Mk(0x11, s, 0)); a.Add(Mk(0x11, s, KEYUP)); }
  public static void Vk(List<INPUT> a, char c) {
    short m = VkKeyScanW(c); byte vk = (byte)(m & 0xFF); byte st = (byte)((m >> 8) & 0xFF);
    if ((st & 1) != 0) a.Add(Mk(0x10, (ushort)MapVirtualKeyW(0x10, 0), 0));
    a.Add(Mk(vk, (ushort)MapVirtualKeyW(vk, 0), 0)); a.Add(Mk(vk, (ushort)MapVirtualKeyW(vk, 0), KEYUP));
    if ((st & 1) != 0) a.Add(Mk(0x10, (ushort)MapVirtualKeyW(0x10, 0), KEYUP)); }
  public static uint SendAll(List<INPUT> a) { var arr = a.ToArray(); return SendInput((uint)arr.Length, arr, Marshal.SizeOf(typeof(INPUT))); }
  public static uint SendBlock(List<INPUT> a, int from, int count) {
    if (count <= 0) return 0; INPUT[] b = a.GetRange(from, count).ToArray();
    return SendInput((uint)b.Length, b, Marshal.SizeOf(typeof(INPUT))); }
  public static uint SendOne(INPUT i) { INPUT[] one = { i }; return SendInput(1, one, Marshal.SizeOf(typeof(INPUT))); }
}
'@
$w = Wait-Window 'scratch-lab.txt' 20; $hwnd = $w.Hwnd
function Prep {
  [void](Repair-Notepad 'scratch-lab.txt'); [void][QAInput]::Focus($hwnd); Start-Sleep -Milliseconds 120
  $el = Get-EditElement $hwnd
  if ($el) { [void](Click-Element $el) } else { Click-Center $hwnd }
  Start-Sleep -Milliseconds 120; Clear-TextArea $hwnd; Start-Sleep -Milliseconds 120 }
function Read2 { return ([string](Read-Text $hwnd).Text).TrimEnd("`r", "`n") }
function Case($name, [string]$prefix, [scriptblock]$mk, [string]$how, [string]$expect) {
  Prep
  if ($prefix) { [QAInput]::TypeAscii($prefix, 60); Start-Sleep -Milliseconds 250 }
  $l = New-Object 'System.Collections.Generic.List[Lab6+INPUT]'
  & $mk $l
  switch ($how) {
    'all'  { [void][Lab6]::SendAll($l) }
    'gap2' { [void][Lab6]::SendBlock($l,0,2); Start-Sleep -Milliseconds 2; [void][Lab6]::SendBlock($l,2,$l.Count-2) }
    'gap10'{ [void][Lab6]::SendBlock($l,0,2); Start-Sleep -Milliseconds 10; [void][Lab6]::SendBlock($l,2,$l.Count-2) }
    'gap2after' { [void][Lab6]::SendBlock($l,0,4); Start-Sleep -Milliseconds 2; [void][Lab6]::SendBlock($l,4,$l.Count-4) }
  }
  Start-Sleep -Milliseconds 550
  $r = Read2
  "{0,-30} how={1,-9} exp=[{2,-9}] got=[{3}] {4}" -f $name, $how, $expect, $r, $(if ($r -ceq $expect) {'OK'} else {'BROKEN'})
}

"--- 1. unicode run WITHOUT keyups (down events only) ---"
Case 'u-run-noup-del2' 'bật/ta' { param($l) [Lab6]::Bs($l,2); [Lab6]::UDownOnly($l,'t'); [Lab6]::UDownOnly($l,'x') } all 'bật/tx'
"--- 2. a modifier pair between unicode chars ---"
Case 'u-run-shift-between-del2' 'bật/ta' { param($l) [Lab6]::Bs($l,2); [Lab6]::U($l,'t'); [Lab6]::Shift($l); [Lab6]::U($l,'x') } all 'bật/tx'
Case 'u-run-ctrl-between-del2' 'bật/ta' { param($l) [Lab6]::Bs($l,2); [Lab6]::U($l,'t'); [Lab6]::Ctrl($l); [Lab6]::U($l,'x') } all 'bật/tx'
"--- 3. Standard-mode shape: VK around a 2- and 3-deep unicode run ---"
Case 'std-shape-run2-del2' 'bật/ta' { param($l) [Lab6]::Bs($l,2); [Lab6]::Vk($l,'d'); [Lab6]::U($l,[char]0x0111); [Lab6]::U($l,[char]0x01B0); [Lab6]::Vk($l,'c') } all 'bật/đưc'
Case 'std-shape-run3-del3' 'bật/ta' { param($l) [Lab6]::Bs($l,3); [Lab6]::Vk($l,'d'); [Lab6]::U($l,[char]0x0111); [Lab6]::U($l,[char]0x01B0); [Lab6]::U($l,[char]0x1EE3); [Lab6]::Vk($l,'c') } all 'bậtđược'
"--- 4. unicode backspace packets instead of VK_BACK ---"
Case 'u-bs-packet-run2' 'bật/ta' { param($l) [Lab6]::BsPacket($l,2); [Lab6]::U($l,'t'); [Lab6]::U($l,'x') } all 'bật/tx'
"--- 5. time gaps ---"
Case 'gap2-before-run'  'bật/ta' { param($l) [Lab6]::Bs($l,2); [Lab6]::U($l,'t'); [Lab6]::U($l,'x') } gap2 'bật/tx'
Case 'gap10-before-run' 'bật/ta' { param($l) [Lab6]::Bs($l,2); [Lab6]::U($l,'t'); [Lab6]::U($l,'x') } gap10 'bật/tx'
Case 'gap2-run3'        'bật/ta' { param($l) [Lab6]::Bs($l,2); [Lab6]::U($l,'t'); [Lab6]::U($l,[char]0x0103); [Lab6]::U($l,'n') } gap2after 'bật/tăn'
"--- 6. control: run with NO deletes, and run after ONE delete ---"
Case 'u-run2-del0' 'bật/ta' { param($l) [Lab6]::U($l,'t'); [Lab6]::U($l,'x') } all 'bật/tatx'
Case 'u-run2-del1' 'bật/ta' { param($l) [Lab6]::Bs($l,1); [Lab6]::U($l,'t'); [Lab6]::U($l,'x') } all 'bật/ttx'
Case 'u-run2-del2-then-vk' 'bật/ta' { param($l) [Lab6]::Bs($l,2); [Lab6]::U($l,'t'); [Lab6]::U($l,[char]0x0103); [Lab6]::Vk($l,'n') } all 'bật/tăn'
