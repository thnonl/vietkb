# Gate teardown: stop the app, close QA scratch windows, restore config byte-identical.
$dir = 'E:\vietkb\docs\evkey-clone-work\qa\e2e95'
$log = Join-Path $dir 'teardown.txt'
Set-Content -Encoding UTF8 $log 'TEARDOWN'
function W($s) { Add-Content -Encoding UTF8 $log $s; Write-Host $s }

# VS Code: close only the window QA opened (focused one with scratch-vsc.txt in the title)
$vs = Get-Process Code -ErrorAction SilentlyContinue
[void]$vs
$codeWin = $null
foreach ($p in Get-Process -ErrorAction SilentlyContinue) {
  if ($p.ProcessName -eq 'Code' -and $p.MainWindowTitle -like '*scratch-vsc.txt*') { $codeWin = $p; break }
}
if ($codeWin) {
  Add-Type -Namespace T -Name K -MemberDefinition @'
[DllImport("user32.dll")] public static extern uint SendInput(uint n, INPUT[] p, int cb);
[StructLayout(LayoutKind.Sequential)] public struct KEYBDINPUT { public ushort wVk; public ushort wScan; public uint dwFlags; public uint time; public IntPtr extra; }
[StructLayout(LayoutKind.Sequential)] public struct MOUSEINPUT { public int dx; public int dy; public uint data; public uint flags; public uint time; public IntPtr extra; }
[StructLayout(LayoutKind.Sequential)] public struct HARDINPUT { public uint msg; public ushort p1; public ushort p2; }
[StructLayout(LayoutKind.Explicit)] public struct InputUnion { [FieldOffset(0)] public MOUSEINPUT mi; [FieldOffset(0)] public KEYBDINPUT ki; [FieldOffset(0)] public HARDINPUT hi; }
[StructLayout(LayoutKind.Sequential)] public struct INPUT { public uint type; public InputUnion u; }
public static void Tap(ushort vk, bool ctrl, bool shift) {
  INPUT[] a = new INPUT[6]; int i = 0;
  if (ctrl) { a[i].type = 1; a[i].u.ki.wVk = 0x11; i++; }
  if (shift) { a[i].type = 1; a[i].u.ki.wVk = 0x10; i++; }
  a[i].type = 1; a[i].u.ki.wVk = vk; i++;
  a[i].type = 1; a[i].u.ki.wVk = vk; a[i].u.ki.dwFlags = 2; i++;
  if (shift) { a[i].type = 1; a[i].u.ki.wVk = 0x10; a[i].u.ki.dwFlags = 2; i++; }
  if (ctrl) { a[i].type = 1; a[i].u.ki.wVk = 0x11; a[i].u.ki.dwFlags = 2; i++; }
  SendInput((uint)i, a, Marshal.SizeOf(typeof(INPUT)));
}
'@
  [void][T.K]::Tap(0x57, $true, $true)   # Ctrl+Shift+W = close window
  Start-Sleep -Milliseconds 800
  W "vscode_closed=$(-not [bool](Get-Process -Id $codeWin.Id -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowTitle -like '*scratch-vsc.txt*' }))"
}

# our isolated Chrome profile
$mine = Get-CimInstance Win32_Process -Filter "Name='chrome.exe'" | Where-Object { $_.CommandLine -like '*vkbe2e-chrome*' }
foreach ($m in $mine) {
  if ($m.CommandLine -notlike '*--type=*') { Stop-Process -Id $m.ProcessId -Force -ErrorAction SilentlyContinue }
}
W "chrome_profile_procs_stopped=$(@($mine | Where-Object { $_.CommandLine -notlike '*--type=*' }).Count)"

# QA scratch apps
Get-Process notepad -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowTitle -like '*scratch-np.txt*' } | Stop-Process -Force -ErrorAction SilentlyContinue
Get-Process cmd -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowTitle -like '*VKB soak console*' } | Stop-Process -Force -ErrorAction SilentlyContinue

# app
Get-Process vietkb -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Milliseconds 900
W "vietkb_running=$([bool](Get-Process vietkb -ErrorAction SilentlyContinue))"

# config restore
$cfg = Join-Path $env:APPDATA 'vietkb\config.toml'
$bak = Join-Path $dir 'config.backup.toml'
Copy-Item $bak $cfg -Force
$h1 = (Get-FileHash $cfg -Algorithm SHA256).Hash
$h2 = (Get-FileHash $bak -Algorithm SHA256).Hash
$diff = Compare-Object (Get-Content $cfg) (Get-Content $bak)
W "config_sha=$h1 backup_sha=$h2 identical=$($h1 -eq $h2) compare_object_empty=$($diff.Count -eq 0)"
W "app_closed=$([bool](-not (Get-Process vietkb -ErrorAction SilentlyContinue)))"
W 'TEARDOWN-DONE'
