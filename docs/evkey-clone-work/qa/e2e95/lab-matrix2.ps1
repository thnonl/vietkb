# lab-matrix2.ps1 — narrow the trigger + find the safe encoding.
. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
public static class Lab4 {
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
  public static INPUT BsDown() { return Mk(0x08, 0, 0); }
  public static INPUT BsUp() { return Mk(0x08, 0, KEYUP); }
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
  public static string Describe(List<INPUT> a) {
    var s = new System.Text.StringBuilder();
    foreach (var i in a) {
      string f = i.u.ki.dwFlags == UNICODE ? "U" : i.u.ki.dwFlags == (UNICODE|KEYUP) ? "u" : i.u.ki.dwFlags == KEYUP ? "k^" : "k";
      s.Append(f).Append(':').Append(i.u.ki.wVk.ToString("X2")).Append('/').Append(i.u.ki.wScan.ToString("X4")).Append(' '); }
    return s.ToString().Trim(); }
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

function Case($name, [string]$prefix, [scriptblock]$mk, [string]$how) {
  Prep
  if ($prefix) { [QAInput]::TypeAscii($prefix, 60); Start-Sleep -Milliseconds 250 }
  $l = New-Object 'System.Collections.Generic.List[Lab4+INPUT]'
  & $mk $l
  switch ($how) {
    'all'   { [void][Lab4]::SendAll($l) }
    'one'   { foreach ($e in $l) { [void][Lab4]::SendOne($e); Start-Sleep -Milliseconds 2 } }
    'split' { [void][Lab4]::SendBlock($l, 0, 4); Start-Sleep -Milliseconds 30; [void][Lab4]::SendBlock($l, 4, $l.Count - 4) }
    'charbychar' { [void][Lab4]::SendBlock($l, 0, 4); Start-Sleep -Milliseconds 20
                   for ($i = 4; $i -lt $l.Count; $i += 2) { [void][Lab4]::SendBlock($l, $i, 2); Start-Sleep -Milliseconds 15 } }
  }
  Start-Sleep -Milliseconds 550
  "{0,-28} prefix=[{1,-8}] how={2,-10} text=[{3}]" -f $name, $prefix, $how, (Read2)
}

"--- consecutive unicode packets after 2 backspaces: how many, and does splitting help ---"
Case 'd2-u3-all'        'bật/ta' { param($l) [Lab4]::U($l,'t'); [Lab4]::U($l,[char]0x0103); [Lab4]::U($l,'n') } all
Case 'd2-u3-split'      'bật/ta' { param($l) [Lab4]::U($l,'t'); [Lab4]::U($l,[char]0x0103); [Lab4]::U($l,'n') } split
Case 'd2-u3-charbychar' 'bật/ta' { param($l) [Lab4]::U($l,'t'); [Lab4]::U($l,[char]0x0103); [Lab4]::U($l,'n') } charbychar
Case 'd2-u3-one'        'bật/ta' { param($l) [Lab4]::U($l,'t'); [Lab4]::U($l,[char]0x0103); [Lab4]::U($l,'n') } one
Case 'd2-u2-u2all'      'bật/ta' { param($l) [Lab4]::U($l,'t'); [Lab4]::U($l,[char]0x0103) } all
"--- is a non-ASCII char anywhere in the document enough? ---"
Case 'ascii-doc-u2'     'abcde'  { param($l) [Lab4]::U($l,'t'); [Lab4]::U($l,[char]0x0103) } all
Case 'ascii-doc-u3'     'abcde'  { param($l) [Lab4]::U($l,'t'); [Lab4]::U($l,[char]0x0103); [Lab4]::U($l,'n') } all
Case 'diacritic-then-ab' ([char]0x1EAD+' ab') { param($l) [Lab4]::U($l,'t'); [Lab4]::U($l,'x') } all
Case 'diacritic-then-ab-u3' ([char]0x1EAD+' ab') { param($l) [Lab4]::U($l,'t'); [Lab4]::U($l,'x'); [Lab4]::U($l,'y') } all
"--- del=1 and del=4 ---"
Case 'd1-u2'            'bật/ta' { param($l) [Lab4]::U($l,'t'); [Lab4]::U($l,'x') } all
Case 'd4-u2'            'bật/ta' { param($l) [Lab4]::U($l,'t'); [Lab4]::U($l,'x') } all
"--- mixed (Standard-mode shape: VK around a single unicode char) ---"
Case 'std-shape-d2'     'bật/ta' { param($l) [Lab4]::Vk($l,'t'); [Lab4]::U($l,[char]0x0103) } all
Case 'std-shape-d3'     'bật/ta' { param($l) [Lab4]::Vk($l,'t'); [Lab4]::U($l,[char]0x1EAF); [Lab4]::Vk($l,'t') } all
Case 'two-diacritics-consecutive' 'bật/ta' { param($l) [Lab4]::Vk($l,'d'); [Lab4]::U($l,[char]0x0111); [Lab4]::U($l,[char]0x01B0); [Lab4]::Vk($l,'c') } all
"--- wordpad / legacy presence ---"
(Test-Path "$env:ProgramFiles\Windows NT\Accessories\wordpad.exe")

