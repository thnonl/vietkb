. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
Add-Type -Namespace T -Name K2 -MemberDefinition @'
[DllImport("user32.dll")] public static extern uint SendInput(uint n, INPUT[] p, int cb);
[StructLayout(LayoutKind.Sequential)] public struct KEYBDINPUT { public ushort wVk; public ushort wScan; public uint dwFlags; public uint time; public IntPtr extra; }
[StructLayout(LayoutKind.Sequential)] public struct MOUSEINPUT { public int dx; public int dy; public uint data; public uint flags; public uint time; public IntPtr extra; }
[StructLayout(LayoutKind.Sequential)] public struct HARDINPUT { public uint msg; public ushort p1; public ushort p2; }
[StructLayout(LayoutKind.Explicit)] public struct InputUnion { [FieldOffset(0)] public MOUSEINPUT mi; [FieldOffset(0)] public KEYBDINPUT ki; [FieldOffset(0)] public HARDINPUT hi; }
[StructLayout(LayoutKind.Sequential)] public struct INPUT { public uint type; public InputUnion u; }
public static void Tap(ushort vk, bool ctrl, bool shift, bool alt) {
  INPUT[] a = new INPUT[6]; int i = 0;
  if (ctrl) { a[i].type = 1; a[i].u.ki.wVk = 0x11; i++; }
  if (shift) { a[i].type = 1; a[i].u.ki.wVk = 0x10; i++; }
  if (alt) { a[i].type = 1; a[i].u.ki.wVk = 0x12; i++; }
  a[i].type = 1; a[i].u.ki.wVk = vk; i++;
  a[i].type = 1; a[i].u.ki.wVk = vk; a[i].u.ki.dwFlags = 2; i++;
  if (alt) { a[i].type = 1; a[i].u.ki.wVk = 0x12; a[i].u.ki.dwFlags = 2; i++; }
  if (shift) { a[i].type = 1; a[i].u.ki.wVk = 0x10; a[i].u.ki.dwFlags = 2; i++; }
  if (ctrl) { a[i].type = 1; a[i].u.ki.wVk = 0x11; a[i].u.ki.dwFlags = 2; i++; }
  SendInput((uint)i, a, Marshal.SizeOf(typeof(INPUT)));
}
'@
$vs = Get-TopWindow 'scratch-vsc.txt'
if ($vs) {
  [void][QAInput]::Focus($vs.Hwnd); Start-Sleep -Milliseconds 400
  [void][T.K2]::Tap(0x57, $true, $true, $false)   # Ctrl+Shift+W
  Start-Sleep -Milliseconds 1200
  # a "Save?" dialog may appear: press Right then Enter? safer: press 'n' if a Don't Save button is focused; try Escape first
  $still = Get-TopWindow 'scratch-vsc.txt'
  if ($still) {
    [void][QAInput]::Focus($still.Hwnd); Start-Sleep -Milliseconds 300
    [void][T.K2]::Tap(0x57, $false, $false, $false)   # Ctrl+W close the dirty editor
    Start-Sleep -Milliseconds 900
    $dlg = $null
    foreach ($h in [QAInput]::TopWindows()) { $t = [QAInput]::Title($h); if ($t -match 'Save|Lưu') { $dlg = $h } }
    if ($dlg) { [void][QAInput]::Focus($dlg); [void][T.K2]::Tap(0x4E, $false, $false, $false); Start-Sleep -Milliseconds 600 }  # "N"/Don't Save
    Start-Sleep -Milliseconds 800
    if (Get-TopWindow 'scratch-vsc.txt') { [void][T.K2]::Tap(0x73, $false, $false, $true); Start-Sleep -Milliseconds 800 }
  }
  Write-Host "vscode_window_gone=$(-not [bool](Get-TopWindow 'scratch-vsc.txt'))"
}
$con = Get-TopWindow 'VKB soak console'
if ($con) {
  [void][QAInput]::Focus($con.Hwnd); Start-Sleep -Milliseconds 400
  [QAInput]::TypeAscii('exit', 60); Start-Sleep -Milliseconds 200
  [QAInput]::Combo(0x0D, $false, $false, $false); Start-Sleep -Milliseconds 1200
  if (Get-TopWindow 'VKB soak console') {
    $c2 = Get-TopWindow 'VKB soak console'; [void][QAInput]::Focus($c2.Hwnd); Start-Sleep -Milliseconds 300
    [void][T.K2]::Tap(0x73, $false, $false, $true); Start-Sleep -Milliseconds 800
  }
  Write-Host "console_gone=$(-not [bool](Get-TopWindow 'VKB soak console'))"
}
Get-Process vietkb -ErrorAction SilentlyContinue | Select-Object Id
Write-Host 'CLEANUP-DONE'
