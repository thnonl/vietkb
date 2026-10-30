# lab-matrix9.ps1 — can a 3-deep non-ASCII run be made reliable?
. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
public static class LabB {
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
  public static void PushU(List<INPUT> a, char c) { a.Add(Mk(0, c, UNICODE)); a.Add(Mk(0, c, UNICODE | KEYUP)); }
  public static void PushVk(List<INPUT> a, char c) {
    short m = VkKeyScanW(c); byte vk = (byte)(m & 0xFF); byte st = (byte)((m >> 8) & 0xFF);
    if ((st & 1) != 0) a.Add(Mk(0x10, (ushort)MapVirtualKeyW(0x10, 0), 0));
    a.Add(Mk(vk, (ushort)MapVirtualKeyW(vk, 0), 0)); a.Add(Mk(vk, (ushort)MapVirtualKeyW(vk, 0), KEYUP));
    if ((st & 1) != 0) a.Add(Mk(0x10, (ushort)MapVirtualKeyW(0x10, 0), KEYUP)); }
  public static void Sep(List<INPUT> a) { ushort s = (ushort)MapVirtualKeyW(0x10, 0); a.Add(Mk(0x10, s, 0)); a.Add(Mk(0x10, s, KEYUP)); }
  public static uint SendAll(List<INPUT> a) { var arr = a.ToArray(); return SendInput((uint)arr.Length, arr, Marshal.SizeOf(typeof(INPUT))); }
  public static uint SendOne(INPUT i) { INPUT[] one = { i }; return SendInput(1, one, Marshal.SizeOf(typeof(INPUT))); }
}
'@
$w = Wait-Window 'scratch-lab.txt' 20; $hwnd = $w.Hwnd
function Prep {
  [void](Repair-Notepad 'scratch-lab.txt'); [void][QAInput]::Focus($hwnd); Start-Sleep -Milliseconds 120
  $el = Get-EditElement $hwnd
  if ($el) { [void](Click-Element $el) } else { Click-Center $hwnd }
  Start-Sleep -Milliseconds 120; Clear-TextArea $hwnd; Start-Sleep -Milliseconds 150 }
function Read2 { return ([string](Read-Text $hwnd).Text).TrimEnd("`r", "`n") }

$ins = 'được'; $del = 3; $prefix = 'bật/ta'; $exp = 'bật/' + $ins

function Trial($label, [scriptblock]$build, [string]$delivery, [int]$n) {
  $ok = 0; $bad = @()
  for ($i = 0; $i -lt $n; $i++) {
    Prep
    [QAInput]::TypeAscii($prefix, 60); Start-Sleep -Milliseconds 250
    $l = New-Object 'System.Collections.Generic.List[LabB+INPUT]'
    [LabB]::Bs($l, $del)
    & $build $l
    if ($delivery -eq 'one') { foreach ($e in $l) { [void][LabB]::SendOne($e) } } else { [void][LabB]::SendAll($l) }
    Start-Sleep -Milliseconds 500
    $t = Read2
    if ($t -ceq $exp) { $ok++ } else { $bad += $t }
  }
  "{0,-40} {1}/{2}  exp=[{3}]  got={4}" -f $label, $ok, $n, $exp, (($bad | Select-Object -Unique) -join ' , ')
}

Trial 'plain mixed' {
  param($l) [LabB]::PushVk($l,'d'); [LabB]::PushU($l,[char]0x0111); [LabB]::PushU($l,[char]0x01B0); [LabB]::PushU($l,[char]0x1EE3); [LabB]::PushVk($l,'c') } all 5

Trial 'shift separator between unicode chars' {
  param($l) [LabB]::PushVk($l,'d')
  [LabB]::PushU($l,[char]0x0111); [LabB]::Sep($l); [LabB]::PushU($l,[char]0x01B0); [LabB]::Sep($l); [LabB]::PushU($l,[char]0x1EE3)
  [LabB]::PushVk($l,'c') } all 5

Trial 'one SendInput per event' {
  param($l) [LabB]::PushVk($l,'d'); [LabB]::PushU($l,[char]0x0111); [LabB]::PushU($l,[char]0x01B0); [LabB]::PushU($l,[char]0x1EE3); [LabB]::PushVk($l,'c') } one 5

Trial 'ASCII separators do not exist here; unicode pairs' {
  param($l) [LabB]::PushVk($l,'d'); [LabB]::PushU($l,[char]0x0111); [LabB]::PushU($l,[char]0x01B0); [LabB]::PushU($l,[char]0x1EE3); [LabB]::PushVk($l,'c') } all 3
