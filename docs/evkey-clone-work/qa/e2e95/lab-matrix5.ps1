# lab-matrix5.ps1 — does inter-burst spacing change the outcome?
# Replays the real engine decisions for `ba6t5/ta8t1` either as ONE burst or as
# per-keystroke bursts (what actually happens when a human types).
. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
public static class Lab7 {
  [DllImport("user32.dll")] public static extern uint SendInput(uint n, INPUT[] p, int cb);
  [DllImport("user32.dll")] public static extern uint MapVirtualKeyW(uint code, uint type);
  [DllImport("user32.dll")] public static extern short VkKeyScanW(char c);
  [StructLayout(LayoutKind.Sequential)] public struct KEYBDINPUT { public ushort wVk; public ushort wScan; public uint dwFlags; public uint time; public IntPtr extra; }
  [StructLayout(LayoutKind.Sequential)] public struct MOUSEINPUT { public int dx; public int dy; public uint data; public uint flags; public uint time; public IntPtr extra; }
  [StructLayout(LayoutKind.Explicit)] public struct InputUnion { [FieldOffset(0)] public MOUSEINPUT mi; [FieldOffset(0)] public KEYBDINPUT ki; }
  [StructLayout(LayoutKind.Sequential)] public struct INPUT { public uint type; public InputUnion u; }
  public const uint KEYUP = 0x0002, UNICODE = 0x0004;
  public static readonly IntPtr MAGIC = (IntPtr)0x5649454B;
  public static int LastCount;
  static INPUT Mk(ushort vk, ushort scan, uint flags) {
    INPUT i = new INPUT(); i.type = 1; i.u.ki.wVk = vk; i.u.ki.wScan = scan; i.u.ki.dwFlags = flags; i.u.ki.extra = MAGIC; return i; }
  public static void Bs(List<INPUT> a, int n) { for (int k = 0; k < n; k++) { a.Add(Mk(0x08, 0, 0)); a.Add(Mk(0x08, 0, KEYUP)); } }
  public static void U(List<INPUT> a, char c) { a.Add(Mk(0, c, UNICODE)); a.Add(Mk(0, c, UNICODE | KEYUP)); }
  public static void Vk(List<INPUT> a, char c) {
    short m = VkKeyScanW(c); byte vk = (byte)(m & 0xFF); byte st = (byte)((m >> 8) & 0xFF);
    if ((st & 1) != 0) a.Add(Mk(0x10, (ushort)MapVirtualKeyW(0x10, 0), 0));
    a.Add(Mk(vk, (ushort)MapVirtualKeyW(vk, 0), 0)); a.Add(Mk(vk, (ushort)MapVirtualKeyW(vk, 0), KEYUP));
    if ((st & 1) != 0) a.Add(Mk(0x10, (ushort)MapVirtualKeyW(0x10, 0), KEYUP)); }
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

# the engine's real decision stream for `ba6t5/ta8t1` (from keys.log)
$script = @(
  @{kind='vk';  ch='b' },
  @{kind='vk';  ch='a' },
  @{kind='rep'; del=2; ins='bâ' },
  @{kind='vk';  ch='t' },
  @{kind='rep'; del=3; ins='bật' },
  @{kind='vk';  ch='/' },
  @{kind='vk';  ch='t' },
  @{kind='vk';  ch='a' },
  @{kind='rep'; del=2; ins='tă' },
  @{kind='vk';  ch='t' },
  @{kind='rep'; del=3; ins='tắt' }
)

function Play($gapMs, $label) {
  Prep
  foreach ($step in $script) {
    $l = New-Object 'System.Collections.Generic.List[Lab7+INPUT]'
    if ($step.kind -eq 'vk') { [Lab7]::Vk($l, [char][int]$step.ch[0]) }
    else { [Lab7]::Bs($l, $step.del); foreach ($c in $step.ins.ToCharArray()) { [Lab7]::U($l, $c) } }
    [void][Lab7]::SendAll($l)
    Start-Sleep -Milliseconds $gapMs
  }
  Start-Sleep -Milliseconds 500
  "{0,-34} gap={1,-4} text=[{2}]  {3}" -f $label, $gapMs, (Read2), $(if ((Read2) -ceq 'bật/tắt') {'OK'} else {'BROKEN'})
}

Play 0    'one continuous burst'
Play 55   '55 ms between bursts (harness speed)'
Play 150  '150 ms between bursts'
Play 400  '400 ms between bursts'
