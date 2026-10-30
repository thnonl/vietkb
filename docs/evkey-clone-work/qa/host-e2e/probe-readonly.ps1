# Read-only environment probe for the 0.2.4 host E2E. Sends no input, starts/stops nothing.
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
Add-Type -TypeDefinition @"
using System; using System.Runtime.InteropServices;
public static class NativeProbe {
  [StructLayout(LayoutKind.Sequential)] public struct LASTINPUTINFO { public uint cbSize; public uint dwTime; }
  [DllImport("user32.dll")] public static extern bool GetLastInputInfo(ref LASTINPUTINFO p);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern bool GetCursorPos(out POINT p);
  [StructLayout(LayoutKind.Sequential)] public struct POINT { public int X; public int Y; }
  [DllImport("user32.dll")] public static extern int GetDpiForWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern IntPtr GetDC(IntPtr h);
  [DllImport("gdi32.dll")] public static extern int GetDeviceCaps(IntPtr hdc, int idx);
  [DllImport("user32.dll")] public static extern int ReleaseDC(IntPtr h, IntPtr hdc);
}
"@
$li = New-Object NativeProbe+LASTINPUTINFO; $li.cbSize = [System.Runtime.InteropServices.Marshal]::SizeOf($li)
[void][NativeProbe]::GetLastInputInfo([ref]$li)
$idleMs = [Environment]::TickCount - $li.dwTime
"IDLE_MS=$idleMs"
$pt = New-Object NativeProbe+POINT; [void][NativeProbe]::GetCursorPos([ref]$pt); "CURSOR=$($pt.X),$($pt.Y)"
"FOREGROUND_HWND=$([NativeProbe]::GetForegroundWindow())"
$i = 0
foreach ($s in [System.Windows.Forms.Screen]::AllScreens) {
  $i++
  $hdc = [NativeProbe]::GetDC([IntPtr]::Zero)
  $dpi = [NativeProbe]::GetDeviceCaps($hdc, 88); [void][NativeProbe]::ReleaseDC([IntPtr]::Zero, $hdc)
  "MONITOR$i device=$($s.DeviceName) primary=$($s.Primary) bounds=$($s.Bounds) workingArea=$($s.WorkingArea) bpp=$($s.BitsPerPixel) logpixelsx=$dpi (scale=$([math]::Round($dpi/96*100))%)"
}
"HOOK_PROCESSES:"
Get-Process | Where-Object { $_.ProcessName -match 'vietkb|vkb|evkey|unikey|autohotkey|powertoys|keyboard' } | ForEach-Object { "  $($_.ProcessName) pid=$($_.Id) path=$($_.Path)" }
"INSTALLER_VERSION_RESOURCE:"
$v = (Get-Item 'E:\vietkb\target\release\bundle\nsis\VietKB_0.2.4_x64-setup.exe').VersionInfo
"  FileVersion=$($v.FileVersion) ProductVersion=$($v.ProductVersion) ProductName=$($v.ProductName) FileDescription=$($v.FileDescription)"
