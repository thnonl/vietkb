# lab-matrix.ps1 — pin the exact rule behind the Notepad corruption.
# No app involved: prefix typed with real keystrokes, then ONE injected batch.
. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
public static class Lab3 {
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
    INPUT i = new INPUT(); i.type = 1; i.u.ki.wVk = vk; i.u.ki.wScan = scan; i.u.ki.dwFlags = flags; i.u.ki.extra = MAGIC; return i;
  }
  public static void Bs(List<INPUT> a, int n) { for (int k = 0; k < n; k++) { a.Add(Mk(0x08, 0, 0)); a.Add(Mk(0x08, 0, KEYUP)); } }
  public static void BsScan(List<INPUT> a, int n) { ushort s = (ushort)MapVirtualKeyW(0x08, 0); for (int k = 0; k < n; k++) { a.Add(Mk(0x08, s, 0)); a.Add(Mk(0x08, s, KEYUP)); } }
  public static void U(List<INPUT> a, char c) { a.Add(Mk(0, c, UNICODE)); a.Add(Mk(0, c, UNICODE | KEYUP)); }
  public static void UTagless(List<INPUT> a, char c) { INPUT d = Mk(0, c, UNICODE); d.u.ki.extra = IntPtr.Zero; INPUT u = Mk(0, c, UNICODE|KEYUP); u.u.ki.extra = IntPtr.Zero; a.Add(d); a.Add(u); }
  public static void Vk(List<INPUT> a, char c) {
    short m = VkKeyScanW(c); byte vk = (byte)(m & 0xFF); byte st = (byte)((m >> 8) & 0xFF); bool shift = (st & 1) != 0;
    if (shift) a.Add(Mk(0x10, (ushort)MapVirtualKeyW(0x10, 0), 0));
    a.Add(Mk(vk, (ushort)MapVirtualKeyW(vk, 0), 0)); a.Add(Mk(vk, (ushort)MapVirtualKeyW(vk, 0), KEYUP));
    if (shift) a.Add(Mk(0x10, (ushort)MapVirtualKeyW(0x10, 0), KEYUP));
  }
  public static string Send(List<INPUT> a) {
    var arr = a.ToArray(); uint ok = SendInput((uint)arr.Length, arr, Marshal.SizeOf(typeof(INPUT))); return ok + "/" + arr.Length;
  }
  public static string SendSplitAfterDeletes(List<INPUT> a, int deletes) {
    // first the backspace block, then the text block, as two SendInput calls
    uint ok = 0;
    int cut = deletes * 2;
    INPUT[] first = a.GetRange(0, cut).ToArray();
    INPUT[] second = a.GetRange(cut, a.Count - cut).ToArray();
    if (first.Length > 0) ok += SendInput((uint)first.Length, first, Marshal.SizeOf(typeof(INPUT)));
    System.Threading.Thread.Sleep(30);
    if (second.Length > 0) ok += SendInput((uint)second.Length, second, Marshal.SizeOf(typeof(INPUT)));
    return ok + "/" + a.Count;
  }
  public static string Describe(List<INPUT> a) {
    var s = new System.Text.StringBuilder();
    foreach (var i in a) {
      string f = i.u.ki.dwFlags == UNICODE ? "U" : i.u.ki.dwFlags == (UNICODE|KEYUP) ? "u" : i.u.ki.dwFlags == KEYUP ? "k^" : "k";
      s.Append(f).Append(':').Append(i.u.ki.wVk.ToString("X2")).Append('/').Append(i.u.ki.wScan.ToString("X4")).Append(' ');
    }
    return s.ToString().Trim();
  }
}
'@

if (-not (Get-Process notepad -ErrorAction SilentlyContinue)) {
  $f = 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\scratch-lab.txt'
  Remove-Item $f -ErrorAction SilentlyContinue; New-Item -ItemType File -Path $f | Out-Null
  Start-Process notepad.exe -ArgumentList $f | Out-Null
  Start-Sleep -Milliseconds 1500
}
$w = Wait-Window 'scratch-lab.txt' 20
$hwnd = $w.Hwnd

function Prep {
  [void](Repair-Notepad 'scratch-lab.txt'); [void][QAInput]::Focus($hwnd); Start-Sleep -Milliseconds 120
  $el = Get-EditElement $hwnd
  if ($el) { [void](Click-Element $el) } else { Click-Center $hwnd }
  Start-Sleep -Milliseconds 120; Clear-TextArea $hwnd; Start-Sleep -Milliseconds 120
}
function Read2 { $t = [string](Read-Text $hwnd).Text; return $t.TrimEnd("`r", "`n") }

function Case($name, [string]$prefix, [scriptblock]$mk, [int]$splitAfter = -1) {
  Prep
  if ($prefix) { [QAInput]::TypeAscii($prefix, 60); Start-Sleep -Milliseconds 250 }
  $l = New-Object 'System.Collections.Generic.List[Lab3+INPUT]'
  & $mk $l
  $res = if ($splitAfter -ge 0) { [Lab3]::SendSplitAfterDeletes($l, $splitAfter) } else { [Lab3]::Send($l) }
  Start-Sleep -Milliseconds 550
  $txt = Read2
  "{0,-30} prefix=[{1,-8}] send={2,-6} text=[{3}]" -f $name, $prefix, $res, $txt
}

"--- del=0 (no backspaces) ---"
Case 'd0-u-t+a'      'bật/' { param($l) [Lab3]::U($l,'t'); [Lab3]::U($l,[char]0x0103) }
Case 'd0-u-t+x'      'bật/' { param($l) [Lab3]::U($l,'t'); [Lab3]::U($l,'x') }
"--- del=1 ---"
Case 'd1-u-t+a'      'bật/ta' { param($l) [Lab3]::Bs($l,1); [Lab3]::U($l,'t'); [Lab3]::U($l,[char]0x0103) }
Case 'd1-u-t+x'      'bật/ta' { param($l) [Lab3]::Bs($l,1); [Lab3]::U($l,'t'); [Lab3]::U($l,'x') }
"--- del=2 ---"
Case 'd2-u-t+a'      'bật/ta' { param($l) [Lab3]::Bs($l,2); [Lab3]::U($l,'t'); [Lab3]::U($l,[char]0x0103) }
Case 'd2-u-t+x'      'bật/ta' { param($l) [Lab3]::Bs($l,2); [Lab3]::U($l,'t'); [Lab3]::U($l,'x') }
Case 'd2-u-t-only'   'bật/ta' { param($l) [Lab3]::Bs($l,2); [Lab3]::U($l,'t') }
Case 'd2-u-a-only'   'bật/ta' { param($l) [Lab3]::Bs($l,2); [Lab3]::U($l,[char]0x0103) }
Case 'd2-vk-t+a'     'bật/ta' { param($l) [Lab3]::Bs($l,2); [Lab3]::Vk($l,'t'); [Lab3]::U($l,[char]0x0103) }
Case 'd2-u-t+vk-a'   'bật/ta' { param($l) [Lab3]::Bs($l,2); [Lab3]::U($l,'t'); [Lab3]::Vk($l,'x') }
Case 'd2-vk-t+x'     'bật/ta' { param($l) [Lab3]::Bs($l,2); [Lab3]::Vk($l,'t'); [Lab3]::Vk($l,'x') }
Case 'd2-u-tagless-t+x' 'bật/ta' { param($l) [Lab3]::Bs($l,2); [Lab3]::UTagless($l,'t'); [Lab3]::UTagless($l,'x') }
Case 'd2-split-u-t+a' 'bật/ta' { param($l) [Lab3]::Bs($l,2); [Lab3]::U($l,'t'); [Lab3]::U($l,[char]0x0103) } 2
Case 'd2-u-3chars'   'bật/ta' { param($l) [Lab3]::Bs($l,2); [Lab3]::U($l,'t'); [Lab3]::U($l,[char]0x0103); [Lab3]::U($l,'n') }
"--- del=2 on pure ASCII doc (no diacritics anywhere) ---"
Case 'd2-ascii-doc'  'abcde'  { param($l) [Lab3]::Bs($l,2); [Lab3]::U($l,'t'); [Lab3]::U($l,'x') }
Case 'd2-ascii-doc-vk' 'abcde' { param($l) [Lab3]::Bs($l,2); [Lab3]::Vk($l,'t'); [Lab3]::Vk($l,'x') }
"--- del=3 ---"
Case 'd3-u-t+aa+t'   'bật/ta' { param($l) [Lab3]::Bs($l,3); [Lab3]::U($l,'t'); [Lab3]::U($l,[char]0x1EAF); [Lab3]::U($l,'t') }
Case 'd3-vk-t+u-aa+vk-t' 'bật/ta' { param($l) [Lab3]::Bs($l,3); [Lab3]::Vk($l,'t'); [Lab3]::U($l,[char]0x1EAF); [Lab3]::Vk($l,'t') }
