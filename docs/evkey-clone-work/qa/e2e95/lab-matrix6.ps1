# lab-matrix6.ps1 — which construction survives, and what the degeneration depends on.
. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
public static class Lab8 {
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
  public static void U(List<INPUT> a, char c) { a.Add(Mk(0, c, UNICODE)); a.Add(Mk(0, c, UNICODE | KEYUP)); }
  public static void Vk(List<INPUT> a, char c) {
    short m = VkKeyScanW(c); byte vk = (byte)(m & 0xFF); byte st = (byte)((m >> 8) & 0xFF);
    if ((st & 1) != 0) a.Add(Mk(0x10, (ushort)MapVirtualKeyW(0x10, 0), 0));
    a.Add(Mk(vk, (ushort)MapVirtualKeyW(vk, 0), 0)); a.Add(Mk(vk, (ushort)MapVirtualKeyW(vk, 0), KEYUP));
    if ((st & 1) != 0) a.Add(Mk(0x10, (ushort)MapVirtualKeyW(0x10, 0), KEYUP)); }
  public static uint SendAll(List<INPUT> a) { var arr = a.ToArray(); return SendInput((uint)arr.Length, arr, Marshal.SizeOf(typeof(INPUT))); }
  // mixed insert: ASCII -> VK/scancode, everything else -> one unicode packet each
  public static void InsertMixed(List<INPUT> a, string text) {
    foreach (char c in text) { if (VkKeyScanW(c) != -1 && c < 0x80) Vk(a, c); else U(a, c); } }
  public static void InsertUnicode(List<INPUT> a, string text) { foreach (char c in text) U(a, c); }
  public static uint SendTextOneByOne(string text) {
    uint n = 0; foreach (char c in text) { var l = new List<INPUT>(); U(l, c); n += SendAll(l); } return n; }
}
'@
$w = Wait-Window 'scratch-lab.txt' 20; $hwnd = $w.Hwnd
function Prep {
  [void](Repair-Notepad 'scratch-lab.txt'); [void][QAInput]::Focus($hwnd); Start-Sleep -Milliseconds 120
  $el = Get-EditElement $hwnd
  if ($el) { [void](Click-Element $el) } else { Click-Center $hwnd }
  Start-Sleep -Milliseconds 120; Clear-TextArea $hwnd; Start-Sleep -Milliseconds 150 }
function Read2 { return ([string](Read-Text $hwnd).Text).TrimEnd("`r", "`n") }

# A) full replay, Standard-mode shape (ASCII via VK, non-ASCII via unicode)
$script = @(
  @{k='vk';  ch='b' }, @{k='vk'; ch='a' }, @{k='rep'; del=2; ins='bâ' }, @{k='vk'; ch='t' },
  @{k='rep'; del=3; ins='bật' }, @{k='vk'; ch='/' }, @{k='vk'; ch='t' }, @{k='vk'; ch='a' },
  @{k='rep'; del=2; ins='tă' }, @{k='vk'; ch='t' }, @{k='rep'; del=3; ins='tắt' } )
foreach ($shape in 'unicode','mixed') {
  Prep
  foreach ($s in $script) {
    $l = New-Object 'System.Collections.Generic.List[Lab8+INPUT]'
    if ($s.k -eq 'vk') { [Lab8]::Vk($l, [char][int]$s.ch[0]) }
    else { [Lab8]::Bs($l, $s.del); if ($shape -eq 'mixed') { [Lab8]::InsertMixed($l, $s.ins) } else { [Lab8]::InsertUnicode($l, $s.ins) } }
    [void][Lab8]::SendAll($l); Start-Sleep -Milliseconds 55 }
  Start-Sleep -Milliseconds 500
  "A) full replay shape={0,-8} text=[{1}]" -f $shape, (Read2)
}
"---- targeted: doc 'bật/' then Replace{3,'được'} (3 consecutive non-ASCII) ----"
foreach ($shape in 'unicode','mixed') {
  Prep
  [QAInput]::TypeAscii('bật/', 60); Start-Sleep -Milliseconds 250
  $l = New-Object 'System.Collections.Generic.List[Lab8+INPUT]'
  [Lab8]::Bs($l, 3); if ($shape -eq 'mixed') { [Lab8]::InsertMixed($l,'được') } else { [Lab8]::InsertUnicode($l,'được') }
  [void][Lab8]::SendAll($l); Start-Sleep -Milliseconds 500
  "B) shape={0,-8} text=[{1}]  expect [bậtđượ...]" -f $shape, (Read2)
}
"---- does the *next* burst still degenerate? deletes burst, then text burst 200 ms later ----"
Prep
[QAInput]::TypeAscii('bật/ta', 60); Start-Sleep -Milliseconds 250
$l = New-Object 'System.Collections.Generic.List[Lab8+INPUT]'; [Lab8]::Bs($l,2); [void][Lab8]::SendAll($l)
Start-Sleep -Milliseconds 200
$l2 = New-Object 'System.Collections.Generic.List[Lab8+INPUT]'; [Lab8]::InsertUnicode($l2,'tx'); [void][Lab8]::SendAll($l2)
Start-Sleep -Milliseconds 400
"C) deletes then text 200 ms later: [{0}]  expect [bật/tx]" -f (Read2)
"---- non-ASCII in doc vs not (2 deletes, 2 unicode chars) ----"
foreach ($p in 'abcde', ([char]0x1EAD + 'bcde')) {
  Prep; if ($p) { [QAInput]::TypeAscii($p, 60); Start-Sleep -Milliseconds 250 }
  $l = New-Object 'System.Collections.Generic.List[Lab8+INPUT]'; [Lab8]::Bs($l,2); [Lab8]::InsertUnicode($l,'tx'); [void][Lab8]::SendAll($l)
  Start-Sleep -Milliseconds 400
  "D) doc=[{0,-6}] got=[{1}]" -f $p, (Read2)
}
