# lab-raw.ps1 — raw-array lab against the real Windows 11 Notepad, NO app involved.
# Types a prefix with ordinary keystrokes, then fires ONE SendInput batch that is
# byte-identical to what crates/vkb-windows/src/input.rs builds for a Replace, and
# reads the document back. Isolates "what the target does with the array".
. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
public static class Lab2 {
  [DllImport("user32.dll")] public static extern uint SendInput(uint n, INPUT[] p, int cb);
  [DllImport("user32.dll")] public static extern uint MapVirtualKeyW(uint code, uint type);
  [DllImport("user32.dll")] public static extern short VkKeyScanW(char c);
  [DllImport("user32.dll")] public static extern uint GetLastError();
  [StructLayout(LayoutKind.Sequential)] public struct KEYBDINPUT { public ushort wVk; public ushort wScan; public uint dwFlags; public uint time; public IntPtr extra; }
  [StructLayout(LayoutKind.Sequential)] public struct MOUSEINPUT { public int dx; public int dy; public uint data; public uint flags; public uint time; public IntPtr extra; }
  [StructLayout(LayoutKind.Explicit)] public struct InputUnion { [FieldOffset(0)] public MOUSEINPUT mi; [FieldOffset(0)] public KEYBDINPUT ki; }
  [StructLayout(LayoutKind.Sequential)] public struct INPUT { public uint type; public InputUnion u; }
  public const uint KEYUP = 0x0002, UNICODE = 0x0004;
  public static readonly IntPtr MAGIC = (IntPtr)0x5649454B;
  public static INPUT Mk(ushort vk, ushort scan, uint flags) {
    INPUT i = new INPUT(); i.type = 1; i.u.ki.wVk = vk; i.u.ki.wScan = scan; i.u.ki.dwFlags = flags; i.u.ki.extra = MAGIC; return i;
  }
  public static void BsProd(List<INPUT> a) { a.Add(Mk(0x08, 0, 0)); a.Add(Mk(0x08, 0, KEYUP)); }
  public static void BsScan(List<INPUT> a) { ushort s = (ushort)MapVirtualKeyW(0x08, 0); a.Add(Mk(0x08, s, 0)); a.Add(Mk(0x08, s, KEYUP)); }
  public static void BsPacket(List<INPUT> a) { a.Add(Mk(0, 8, UNICODE)); a.Add(Mk(0, 8, UNICODE | KEYUP)); }
  public static void U(List<INPUT> a, char c) { a.Add(Mk(0, c, UNICODE)); a.Add(Mk(0, c, UNICODE | KEYUP)); }
  public static void Real(List<INPUT> a, char c) {
    short m = VkKeyScanW(c); byte vk = (byte)(m & 0xFF); byte st = (byte)((m >> 8) & 0xFF);
    bool shift = (st & 1) != 0;
    if (shift) a.Add(Mk(0x10, (ushort)MapVirtualKeyW(0x10, 0), 0));
    a.Add(Mk(vk, (ushort)MapVirtualKeyW(vk, 0), 0));
    a.Add(Mk(vk, (ushort)MapVirtualKeyW(vk, 0), KEYUP));
    if (shift) a.Add(Mk(0x10, (ushort)MapVirtualKeyW(0x10, 0), KEYUP));
  }
  public static string Describe(List<INPUT> a) {
    var s = new System.Text.StringBuilder();
    foreach (var i in a) {
      string f = i.u.ki.dwFlags == UNICODE ? "U" : i.u.ki.dwFlags == (UNICODE|KEYUP) ? "u" : i.u.ki.dwFlags == KEYUP ? "k^" : "k";
      s.Append(f).Append(':').Append(i.u.ki.wVk.ToString("X2")).Append('/').Append(i.u.ki.wScan.ToString("X4")).Append(' ');
    }
    return s.ToString().Trim();
  }
  public static string Send(List<INPUT> a) {
    var arr = a.ToArray();
    uint ok = SendInput((uint)arr.Length, arr, Marshal.SizeOf(typeof(INPUT)));
    return ok.ToString() + "/" + arr.Length + (ok < arr.Length ? " err=" + GetLastError() : "");
  }
  public static string SendSplit(List<INPUT> a) {
    uint ok = 0;
    foreach (var i in a) { INPUT[] one = { i }; ok += SendInput(1, one, Marshal.SizeOf(typeof(INPUT))); }
    return ok.ToString() + "/" + a.Count;
  }
}
'@

Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Milliseconds 500
$dir = 'E:\vietkb\docs\evkey-clone-work\qa\e2e95'
$f = Join-Path $dir 'scratch-lab.txt'
Remove-Item $f -ErrorAction SilentlyContinue; New-Item -ItemType File -Path $f | Out-Null
Start-Process notepad.exe -ArgumentList $f | Out-Null
$w = Wait-Window 'scratch-lab.txt' 30
if (-not $w) { 'SETUP-FAIL'; exit 1 }
$hwnd = $w.Hwnd
Start-Sleep -Milliseconds 1200

function Prep {
  [void](Repair-Notepad 'scratch-lab.txt')
  [void][QAInput]::Focus($hwnd)
  Start-Sleep -Milliseconds 120
  $el = Get-EditElement $hwnd
  if ($el) { [void](Click-Element $el) } else { Click-Center $hwnd }
  Start-Sleep -Milliseconds 120
  Clear-TextArea $hwnd
  Start-Sleep -Milliseconds 100
}
function Read2 { $r = Read-Text $hwnd; $t = [string]$r.Text; return $t.TrimEnd("`r", "`n") }

$rows = @()
function Case($name, [string]$prefix, [scriptblock]$mk, [switch]$split) {
  Prep
  if ($prefix) { [QAInput]::TypeAscii($prefix, 60); Start-Sleep -Milliseconds 250 }
  $l = New-Object 'System.Collections.Generic.List[Lab2+INPUT]'
  & $mk $l
  $res = if ($split) { [Lab2]::SendSplit($l) } else { [Lab2]::Send($l) }
  Start-Sleep -Milliseconds 500
  $txt = Read2
  $row = [pscustomobject]@{ Case = $name; Send = $res; Array = [Lab2]::Describe($l); Text = $txt }
  $script:rows += $row
  "CASE {0}`n  prefix=[{1}] send={2}`n  array={3}`n  text=[{4}]" -f $name, $prefix, $res, $row.Array, $txt
}

# 1. production Replace{2,"tă"} on top of "bật/ta"
Case 'P1-replace2-t-a-breve' 'bật/ta' {
  param($l)
  [Lab2]::BsProd($l); [Lab2]::BsProd($l); [Lab2]::U($l, 't'); [Lab2]::U($l, [char]0x0103)
}
# 2. same, ASCII-only insert  (Replace{2,"tx"})
Case 'P2-replace2-ascii-insert' 'bật/ta' {
  param($l)
  [Lab2]::BsProd($l); [Lab2]::BsProd($l); [Lab2]::U($l, 't'); [Lab2]::U($l, 'x')
}
# 3. same, backspace with scancode
Case 'P3-backspace-scancode' 'bật/ta' {
  param($l)
  [Lab2]::BsScan($l); [Lab2]::BsScan($l); [Lab2]::U($l, 't'); [Lab2]::U($l, [char]0x0103)
}
# 4. deletes only, no insert
Case 'P4-deletes-only' 'bật/ta' {
  param($l)
  [Lab2]::BsProd($l); [Lab2]::BsProd($l)
}
# 5. insert only, non-ASCII, no deletes
Case 'P5-insert-only-nonascii' 'bật/ta' {
  param($l)
  [Lab2]::U($l, [char]0x0103)
}
# 6. exact whole-session production replay into an empty doc
Case 'P6-whole-session' '' {
  param($l)
  [Lab2]::Real($l, 'b'); [Lab2]::Real($l, 'a')
  [Lab2]::BsProd($l); [Lab2]::BsProd($l); [Lab2]::U($l, 'b'); [Lab2]::U($l, [char]0x00E2)
  [Lab2]::Real($l, 't')
  [Lab2]::BsProd($l); [Lab2]::BsProd($l); [Lab2]::BsProd($l); [Lab2]::U($l, 'b'); [Lab2]::U($l, [char]0x1EAD); [Lab2]::U($l, 't')
  [Lab2]::Real($l, '/'); [Lab2]::Real($l, 't'); [Lab2]::Real($l, 'a')
  [Lab2]::BsProd($l); [Lab2]::BsProd($l); [Lab2]::U($l, 't'); [Lab2]::U($l, [char]0x0103)
  [Lab2]::Real($l, 't')
  [Lab2]::BsProd($l); [Lab2]::BsProd($l); [Lab2]::BsProd($l); [Lab2]::U($l, 't'); [Lab2]::U($l, [char]0x1EAF); [Lab2]::U($l, 't')
}
# 7. whole session but every replace SPLIT into separate SendInput calls
Case 'P7-whole-session-split' '' -split {
  param($l)
  [Lab2]::Real($l, 'b'); [Lab2]::Real($l, 'a')
  [Lab2]::BsProd($l); [Lab2]::BsProd($l); [Lab2]::U($l, 'b'); [Lab2]::U($l, [char]0x00E2)
  [Lab2]::Real($l, 't')
  [Lab2]::BsProd($l); [Lab2]::BsProd($l); [Lab2]::BsProd($l); [Lab2]::U($l, 'b'); [Lab2]::U($l, [char]0x1EAD); [Lab2]::U($l, 't')
  [Lab2]::Real($l, '/'); [Lab2]::Real($l, 't'); [Lab2]::Real($l, 'a')
  [Lab2]::BsProd($l); [Lab2]::BsProd($l); [Lab2]::U($l, 't'); [Lab2]::U($l, [char]0x0103)
  [Lab2]::Real($l, 't')
  [Lab2]::BsProd($l); [Lab2]::BsProd($l); [Lab2]::BsProd($l); [Lab2]::U($l, 't'); [Lab2]::U($l, [char]0x1EAF); [Lab2]::U($l, 't')
}

'---- done, notepad left open for inspection ----'
