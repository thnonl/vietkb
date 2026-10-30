# verify-legacy2.ps1 — the Run dialog's EDIT via plain Win32 (no UIA, no focus races).
. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using System.Text;
public static class Win32Edit {
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern IntPtr FindWindowW(string cls, string name);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern IntPtr FindWindowExW(IntPtr p, IntPtr after, string cls, string name);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern IntPtr SendMessageW(IntPtr h, uint msg, IntPtr w, StringBuilder l);
  [DllImport("user32.dll")] static extern uint SendInput(uint n, INPUT[] p, int cb);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern int GetClassNameW(IntPtr h, StringBuilder s, int n);
  [StructLayout(LayoutKind.Sequential)] public struct KEYBDINPUT { public ushort wVk; public ushort wScan; public uint dwFlags; public uint time; public IntPtr extra; }
  [StructLayout(LayoutKind.Sequential)] public struct MOUSEINPUT { public int dx; public int dy; public uint data; public uint flags; public uint time; public IntPtr extra; }
  [StructLayout(LayoutKind.Explicit)] public struct InputUnion { [FieldOffset(0)] public MOUSEINPUT mi; [FieldOffset(0)] public KEYBDINPUT ki; }
  [StructLayout(LayoutKind.Sequential)] public struct INPUT { public uint type; public InputUnion u; }
  static INPUT Mk(ushort vk, uint flags) { INPUT i = new INPUT(); i.type = 1; i.u.ki.wVk = vk; i.u.ki.dwFlags = flags; return i; }
  public static void Down(ushort vk) { INPUT[] a = { Mk(vk, 0) }; SendInput(1, a, Marshal.SizeOf(typeof(INPUT))); }
  public static void Up(ushort vk) { INPUT[] a = { Mk(vk, 2) }; SendInput(1, a, Marshal.SizeOf(typeof(INPUT))); }
  public static void Chord(ushort mod, ushort key) { Down(mod); System.Threading.Thread.Sleep(25); Down(key); System.Threading.Thread.Sleep(25); Up(key); Up(mod); }
  public static string Text(IntPtr h) { var sb = new StringBuilder(1024); SendMessageW(h, 0x000D, (IntPtr)1024, sb); return sb.ToString(); }
  public static void SetText(IntPtr h, string s) { SendMessageW(h, 0x000C, IntPtr.Zero, null); }
  public static string Cls(IntPtr h) { var sb = new StringBuilder(64); GetClassNameW(h, sb, 64); return sb.ToString(); }
}
'@

[Win32Edit]::Chord(0x5B, 0x52)   # Win+R
Start-Sleep -Milliseconds 1200
$dlg = [Win32Edit]::FindWindowW('#32770', $null)
$edit = [IntPtr]::Zero
if ($dlg -ne [IntPtr]::Zero) {
  $edit = [Win32Edit]::FindWindowExW($dlg, [IntPtr]::Zero, 'Edit', $null)
  "run dialog hwnd=$dlg edit=$edit class=$([Win32Edit]::Cls($edit))"
}
if ($edit -eq [IntPtr]::Zero) {
  # the Run dialog's field is a ComboBox holding an Edit; walk children
  $combo = [Win32Edit]::FindWindowExW($dlg, [IntPtr]::Zero, 'ComboBox', $null)
  "combo=$combo"
  if ($combo -ne [IntPtr]::Zero) { $edit = [Win32Edit]::FindWindowExW($combo, [IntPtr]::Zero, 'Edit', $null); "combo edit=$edit" }
}
if ($edit -eq [IntPtr]::Zero) { 'LEGACY-SETUP-FAIL'; exit 1 }

[void][QAInput]::Focus($dlg)
Start-Sleep -Milliseconds 250
function Probe {
  [Win32Edit]::SetText($edit, '')
  Start-Sleep -Milliseconds 120
  [QAInput]::TypeAscii('ba6t5', 55)
  Start-Sleep -Milliseconds 350
  return [Win32Edit]::Text($edit)
}
$p = Probe
if ($p -ne 'bậ') { [QAInput]::Combo(0x70, $false, $false, $false); Start-Sleep -Milliseconds 350; $p = Probe }
"probe -> [$p]"

$cases = @(
  @('L1-bat-slash-tat',  'ba6t5/ta8t1', 'bật/tắt'),
  @('L2-bat-dot-tat',    'ba6t5.ta8t1', 'bật.tắt'),
  @('L4-bat-underscore', 'ba6t5_ta8t1', 'bật_tắt'),
  @('L5-windows-path',   'C:\Users\nam', 'C:\Users\nam'),
  @('L6-bs-tone',        'ba6t5',       'bậ'),
  @('L7-macro-m01',      'm01 ',        'mục 1 ')
)
foreach ($c in $cases) {
  [void][QAInput]::Focus($dlg); Start-Sleep -Milliseconds 150
  [Win32Edit]::SetText($edit, '')
  Start-Sleep -Milliseconds 120
  [QAInput]::TypeAscii($c[1], 55)
  Start-Sleep -Milliseconds 400
  $act = ([Win32Edit]::Text($edit)).TrimEnd("`r", "`n")
  $v = if ($act -ceq $c[2]) { 'PASS' } else { 'FAIL' }
  "legacy   {0,-20} keys=[{1,-16}] exp=[{2,-10}] act=[{3,-10}] {4}" -f $c[0], $c[1], $c[2], $act, $v
}
[Win32Edit]::Down(0x1B); [Win32Edit]::Up(0x1B)   # Esc: nothing is executed
