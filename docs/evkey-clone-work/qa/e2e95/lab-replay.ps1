# lab-replay.ps1 — in-process injection lab.
# Builds the EXACT INPUT arrays crates/vkb-windows/src/input.rs emits for
# `ba6t5/ta8t1` under the shipped default (send_standard = false), sends them to a
# real RichEdit control, and logs every message the control receives.
# Purpose: separate "array construction" from "target app behaviour".
param([switch]$Keep)
Add-Type -AssemblyName System.Windows.Forms, System.Drawing
Add-Type -ReferencedAssemblies 'System.Windows.Forms','System.Drawing' -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Windows.Forms;

public static class Lab {
  [DllImport("user32.dll")] public static extern uint SendInput(uint n, INPUT[] p, int cb);
  [DllImport("user32.dll")] public static extern uint MapVirtualKeyW(uint code, uint type);
  [DllImport("user32.dll")] public static extern short VkKeyScanW(char c);

  [StructLayout(LayoutKind.Sequential)] public struct KEYBDINPUT { public ushort wVk; public ushort wScan; public uint dwFlags; public uint time; public IntPtr extra; }
  [StructLayout(LayoutKind.Sequential)] public struct MOUSEINPUT { public int dx; public int dy; public uint data; public uint flags; public uint time; public IntPtr extra; }
  [StructLayout(LayoutKind.Explicit)] public struct InputUnion { [FieldOffset(0)] public MOUSEINPUT mi; [FieldOffset(0)] public KEYBDINPUT ki; }
  [StructLayout(LayoutKind.Sequential)] public struct INPUT { public uint type; public InputUnion u; }

  public const uint KEYUP = 0x0002;
  public const uint UNICODE = 0x0004;
  public static readonly IntPtr MAGIC = (IntPtr)0x5649454B;

  // production: Unicode-mode backspace -> wVk=VK_BACK, wScan=0, flags=0
  public static void BackUnicode(List<INPUT> a) {
    a.Add(Mk(0x08, 0, 0)); a.Add(Mk(0x08, 0, KEYUP));
  }
  // variant: backspace with a scancode (what SendMode::Standard does)
  public static void BackScan(List<INPUT> a) { ushort s = (ushort)MapVirtualKeyW(0x08, 0); a.Add(Mk(0x08, s, 0)); a.Add(Mk(0x08, s, KEYUP)); }
  // variant: backspace as pure unicode packet (U+0008)
  public static void BackPacket(List<INPUT> a) { a.Add(Mk(0, 8, UNICODE)); a.Add(Mk(0, 8, UNICODE | KEYUP)); }

  public static void U(List<INPUT> a, char c) { a.Add(Mk(0, c, UNICODE)); a.Add(Mk(0, c, UNICODE | KEYUP)); }
  // a real, human-style ASCII keystroke (VK + scancode), like QAInput.TypeAscii
  public static void Real(List<INPUT> a, char c) {
    short m = VkKeyScanW(c); byte vk = (byte)(m & 0xFF); byte st = (byte)((m >> 8) & 0xFF);
    bool shift = (st & 1) != 0;
    if (shift) { a.Add(Mk(0x10, (ushort)MapVirtualKeyW(0x10, 0), 0)); }
    a.Add(Mk(vk, (ushort)MapVirtualKeyW(vk, 0), 0));
    a.Add(Mk(vk, (ushort)MapVirtualKeyW(vk, 0), KEYUP));
    if (shift) { a.Add(Mk(0x10, (ushort)MapVirtualKeyW(0x10, 0), KEYUP)); }
  }

  public static INPUT Mk(ushort vk, ushort scan, uint flags) {
    INPUT i = new INPUT(); i.type = 1; i.u.ki.wVk = vk; i.u.ki.wScan = scan; i.u.ki.dwFlags = flags; i.u.ki.extra = MAGIC; return i;
  }
  public static string Describe(INPUT[] a) {
    var s = new System.Text.StringBuilder();
    foreach (var i in a) {
      string f = i.u.ki.dwFlags == UNICODE ? "U" : i.u.ki.dwFlags == (UNICODE|KEYUP) ? "u" : i.u.ki.dwFlags == KEYUP ? "k^" : "k";
      s.Append(f).Append(':').Append(i.u.ki.wVk.ToString("X2")).Append('/').Append(i.u.ki.wScan.ToString("X4")).Append(' ');
    }
    return s.ToString().Trim();
  }
  public static uint Send(INPUT[] a) { return SendInput((uint)a.Length, a, Marshal.SizeOf(typeof(INPUT))); }

  // variant: batch vs per-event delivery (each event its own SendInput + 0 ms)
  public static uint SendSplit(List<INPUT> a, int gapMs) {
    uint ok = 0;
    foreach (var i in a) { INPUT[] one = { i }; ok += SendInput(1, one, Marshal.SizeOf(typeof(INPUT))); if (gapMs > 0) System.Threading.Thread.Sleep(gapMs); }
    return ok;
  }
}

public class SpyEdit : RichTextBox {
  public List<string> Log = new List<string>();
  protected override void WndProc(ref Message m) {
    int msg = m.Msg;
    if (msg == 0x100 || msg == 0x101 || msg == 0x102 || msg == 0x109 || msg == 0x286 || msg == 0x010D) {
      int vk = (int)m.WParam;
      int scan = (int)((long)m.LParam >> 16) & 0xFF;
      int flags = (int)((long)m.LParam) & 0xFFFF;
      string tag = msg == 0x100 ? "KD" : msg == 0x101 ? "KU" : msg == 0x102 ? "CH" : msg == 0x109 ? "UNI" : msg == 0x286 ? "ICOMP" : "ISTART";
      string ch = (msg == 0x102 || msg == 0x109) && vk > 0x1F ? ((char)vk).ToString() : "";
      Log.Add(tag + " vk=" + vk.ToString("X2") + " scan=" + scan.ToString("X2") + " ch=[" + ch + "]");
    }
    base.WndProc(ref m);
  }
}
'@

$form = New-Object System.Windows.Forms.Form
$form.Text = 'VietKB input lab'
$form.Width = 620; $form.Height = 260
$edit = New-Object SpyEdit
$edit.Multiline = $true; $edit.Dock = 'Fill'; $edit.Font = New-Object System.Drawing.Font('Consolas', 14)
$form.Controls.Add($edit)
$form.Add_Shown({ $form.Activate(); $edit.Focus() | Out-Null })
[void]$form.Show()
$form.Activate(); $edit.Focus() | Out-Null
Start-Sleep -Milliseconds 600

function Run-Case($name, [scriptblock]$build) {
  $edit.Clear(); $edit.Log.Clear()
  [void]$form.Activate(); $edit.Focus() | Out-Null
  Start-Sleep -Milliseconds 120
  $list = New-Object 'System.Collections.Generic.List[Lab+INPUT]'
  & $build $list
  $arr = $list.ToArray()
  $sent = [Lab]::Send([Lab+INPUT[]]$arr)
  Start-Sleep -Milliseconds 400
  [System.Windows.Forms.Application]::DoEvents()
  Start-Sleep -Milliseconds 150
  [pscustomobject]@{
    Case = $name
    Sent = "$sent/$($arr.Length)"
    Array = [Lab]::Describe([Lab+INPUT[]]$arr)
    Text = $edit.Text
    Msgs = ($edit.Log -join ' | ')
  }
}

$rows = @()
# --- the whole A1 session, production-exact (Unicode mode) -------------------
$rows += Run-Case 'A1-production-exact' {
  param($l)
  [Lab]::Real($l, 'b'); [Lab]::Real($l, 'a')
  [Lab]::BackUnicode($l); [Lab]::BackUnicode($l); [Lab]::U($l, 'b'); [Lab]::U($l, [char]0x00E2)   # Replace{2,"bâ"}
  [Lab]::Real($l, 't')
  [Lab]::BackUnicode($l); [Lab]::BackUnicode($l); [Lab]::BackUnicode($l)
  [Lab]::U($l, 'b'); [Lab]::U($l, [char]0x1EAD); [Lab]::U($l, 't')                                  # Replace{3,"bật"}
  [Lab]::Real($l, '/'); [Lab]::Real($l, 't'); [Lab]::Real($l, 'a')
  [Lab]::BackUnicode($l); [Lab]::BackUnicode($l); [Lab]::U($l, 't'); [Lab]::U($l, [char]0x0103)     # Replace{2,"tă"}
  [Lab]::Real($l, 't')
  [Lab]::BackUnicode($l); [Lab]::BackUnicode($l); [Lab]::BackUnicode($l)
  [Lab]::U($l, 't'); [Lab]::U($l, [char]0x1EAF); [Lab]::U($l, 't')                                  # Replace{3,"tắt"}
}
# --- second syllable only, from "bật/ta" ------------------------------------
$rows += Run-Case 'A1-second-syllable-only' {
  param($l)
  foreach ($c in 'b', [char]0x1EAD, 't', '/', 't', 'a') { [Lab]::Real($l, $c) }
  [Lab]::BackUnicode($l); [Lab]::BackUnicode($l); [Lab]::U($l, 't'); [Lab]::U($l, [char]0x0103)
  [Lab]::Real($l, 't')
  [Lab]::BackUnicode($l); [Lab]::BackUnicode($l); [Lab]::BackUnicode($l); [Lab]::U($l, 't'); [Lab]::U($l, [char]0x1EAF); [Lab]::U($l, 't')
}
# --- is it the scancode on the backspace? ----------------------------------
$rows += Run-Case 'backspace-with-scancode' {
  param($l)
  foreach ($c in 'b', [char]0x1EAD, 't', '/', 't', 'a') { [Lab]::Real($l, $c) }
  [Lab]::BackScan($l); [Lab]::BackScan($l); [Lab]::U($l, 't'); [Lab]::U($l, [char]0x0103)
  [Lab]::Real($l, 't')
  [Lab]::BackScan($l); [Lab]::BackScan($l); [Lab]::BackScan($l); [Lab]::U($l, 't'); [Lab]::U($l, [char]0x1EAF); [Lab]::U($l, 't')
}
# --- is it the pure-unicode backspace packet? ------------------------------
$rows += Run-Case 'backspace-as-unicode-packet' {
  param($l)
  foreach ($c in 'b', [char]0x1EAD, 't', '/', 't', 'a') { [Lab]::Real($l, $c) }
  [Lab]::BackPacket($l); [Lab]::BackPacket($l); [Lab]::U($l, 't'); [Lab]::U($l, [char]0x0103)
  [Lab]::Real($l, 't')
  [Lab]::BackPacket($l); [Lab]::BackPacket($l); [Lab]::BackPacket($l); [Lab]::U($l, 't'); [Lab]::U($l, [char]0x1EAF); [Lab]::U($l, 't')
}

$rows | Format-List | Out-String -Width 400
$rows | ForEach-Object { "CASE {0}`n  sent={1}`n  text=[{2}]`n  array={3}" -f $_.Case, $_.Sent, $_.Text, $_.Array }

if (-not $Keep) { $form.Close() }

