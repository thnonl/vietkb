# lab-matrix7.ps1 — reliability of each construction over N runs of the A1 replay.
. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
public static class Lab9 {
  [DllImport("user32.dll")] public static extern uint SendInput(uint n, INPUT[] p, int cb);
  [DllImport("user32.dll")] public static extern uint MapVirtualKeyW(uint code, uint type);
  [DllImport("user32.dll")] public static extern short VkKeyScanW(char c);
  [StructLayout(LayoutKind.Sequential)] public struct KEYBDINPUT { public ushort wVk; public ushort wScan; public uint dwFlags; public uint time; public IntPtr extra; }
  [StructLayout(LayoutKind.Sequential)] public struct MOUSEINPUT { public int dx; public int dy; public uint data; public uint flags; public uint time; public IntPtr extra; }
  [StructLayout(LayoutKind.Explicit)] public struct InputUnion { [FieldOffset(0)] public MOUSEINPUT mi; [FieldOffset(0)] public KEYBDINPUT ki; }
  [StructLayout(LayoutKind.Sequential)] public struct INPUT { public uint type; public InputUnion u; }
  public const uint KEYUP = 0x0002, UNICODE = 0x0004;
  public static readonly IntPtr MAGIC = (IntPtr)0x5649454B;
  public static ushort PacketVk = 0;
  public static bool StampTime = false;
  static INPUT Mk(ushort vk, ushort scan, uint flags) {
    INPUT i = new INPUT(); i.type = 1; i.u.ki.wVk = vk; i.u.ki.wScan = scan; i.u.ki.dwFlags = flags; i.u.ki.extra = MAGIC;
    if (StampTime) i.u.ki.time = (uint)Environment.TickCount; return i; }
  public static void Bs(List<INPUT> a, int n) { for (int k = 0; k < n; k++) { a.Add(Mk(0x08, 0, 0)); a.Add(Mk(0x08, 0, KEYUP)); } }
  public static void U(List<INPUT> a, char c) { a.Add(Mk(PacketVk, c, UNICODE)); a.Add(Mk(PacketVk, c, UNICODE | KEYUP)); }
  public static void Vk(List<INPUT> a, char c) {
    short m = VkKeyScanW(c); byte vk = (byte)(m & 0xFF); byte st = (byte)((m >> 8) & 0xFF);
    if ((st & 1) != 0) a.Add(Mk(0x10, (ushort)MapVirtualKeyW(0x10, 0), 0));
    a.Add(Mk(vk, (ushort)MapVirtualKeyW(vk, 0), 0)); a.Add(Mk(vk, (ushort)MapVirtualKeyW(vk, 0), KEYUP));
    if ((st & 1) != 0) a.Add(Mk(0x10, (ushort)MapVirtualKeyW(0x10, 0), KEYUP)); }
  public static uint SendAll(List<INPUT> a) { var arr = a.ToArray(); return SendInput((uint)arr.Length, arr, Marshal.SizeOf(typeof(INPUT))); }
  public static uint SendOne(INPUT i) { INPUT[] one = { i }; return SendInput(1, one, Marshal.SizeOf(typeof(INPUT))); }
  public static uint SendSplit(List<INPUT> a, int cut) {
    uint n = 0; if (cut > 0) { INPUT[] b = a.GetRange(0, cut).ToArray(); n += SendInput((uint)b.Length, b, Marshal.SizeOf(typeof(INPUT))); }
    if (cut < a.Count) { INPUT[] c = a.GetRange(cut, a.Count - cut).ToArray(); n += SendInput((uint)c.Length, c, Marshal.SizeOf(typeof(INPUT))); } return n; }
}
'@
$w = Wait-Window 'scratch-lab.txt' 20; $hwnd = $w.Hwnd
function Prep {
  [void](Repair-Notepad 'scratch-lab.txt'); [void][QAInput]::Focus($hwnd); Start-Sleep -Milliseconds 120
  $el = Get-EditElement $hwnd
  if ($el) { [void](Click-Element $el) } else { Click-Center $hwnd }
  Start-Sleep -Milliseconds 120; Clear-TextArea $hwnd; Start-Sleep -Milliseconds 150 }
function Read2 { return ([string](Read-Text $hwnd).Text).TrimEnd("`r", "`n") }

$script = @(
  @{k='vk';  ch='b' }, @{k='vk'; ch='a' }, @{k='rep'; del=2; ins='bâ' }, @{k='vk'; ch='t' },
  @{k='rep'; del=3; ins='bật' }, @{k='vk'; ch='/' }, @{k='vk'; ch='t' }, @{k='vk'; ch='a' },
  @{k='rep'; del=2; ins='tă' }, @{k='vk'; ch='t' }, @{k='rep'; del=3; ins='tắt' } )

function Replay([string]$shape, [string]$delivery) {
  Prep
  foreach ($s in $script) {
    $l = New-Object 'System.Collections.Generic.List[Lab9+INPUT]'
    if ($s.k -eq 'vk') { [Lab9]::Vk($l, [char][int]$s.ch[0]) }
    else {
      [Lab9]::Bs($l, $s.del)
      if ($shape -eq 'mixed') { foreach ($c in $s.ins.ToCharArray()) { if ([int]$c -lt 128 -and [Lab9]::VkKeyScanW($c) -ne -1) { [Lab9]::Vk($l,$c) } else { [Lab9]::U($l,$c) } } }
      else { foreach ($c in $s.ins.ToCharArray()) { [Lab9]::U($l, $c) } }
    }
    if ($delivery -eq 'per-event') { foreach ($e in $l) { [void][Lab9]::SendOne($e) } }
    elseif ($delivery -eq 'split') { [void][Lab9]::SendSplit($l, $s.del * 2) }
    else { [void][Lab9]::SendAll($l) }
    Start-Sleep -Milliseconds 55
  }
  Start-Sleep -Milliseconds 500
  return Read2
}

function Trial($label, $shape, $delivery, $packetVk, $stamp, $n) {
  [Lab9]::PacketVk = $packetVk; [Lab9]::StampTime = $stamp
  $ok = 0; $samples = @()
  for ($i = 0; $i -lt $n; $i++) {
    $t = Replay $shape $delivery
    if ($t -ceq 'bật/tắt') { $ok++ } else { $samples += $t }
  }
  "{0,-42} {1}/{2} correct   variants: {3}" -f $label, $ok, $n, (($samples | Select-Object -Unique) -join ' , ')
}

Trial 'unicode wVk=0, one call per burst'        'unicode' 'all'       0      $false 8
Trial 'unicode wVk=VK_PACKET(0xE7), one call'   'unicode' 'all'       0xE7   $false 8
Trial 'unicode wVk=0, time stamped'             'unicode' 'all'       0      $true  6
Trial 'unicode wVk=0, one call per event'       'unicode' 'per-event' 0      $false 6
Trial 'unicode wVk=0, delete|text split'        'unicode' 'split'     0      $false 6
Trial 'mixed (ASCII VK / rest unicode)'         'mixed'   'all'       0      $false 8
Trial 'mixed, delete|text split'                'mixed'   'split'     0      $false 6
