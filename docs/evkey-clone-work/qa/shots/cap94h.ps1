# QA round 94 — part H: tray flyout shot, quit via the tray, verify no process.
$ErrorActionPreference = 'Continue'
. 'E:\vietkb\docs\evkey-clone-work\qa\lib.ps1'
Add-Type -AssemblyName System.Drawing
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class Cap94H {
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
  [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr v);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("dwmapi.dll")] public static extern int DwmGetWindowAttribute(IntPtr h, int attr, out RECT r, int cb);
  public static int[] Frame(IntPtr h) {
    RECT r;
    if (DwmGetWindowAttribute(h, 9, out r, Marshal.SizeOf(typeof(RECT))) != 0 || r.Right - r.Left <= 0) { GetWindowRect(h, out r); }
    return new int[] { r.Left, r.Top, r.Right - r.Left, r.Bottom - r.Top };
  }
}
"@ -ErrorAction SilentlyContinue
[void][Cap94H]::SetProcessDpiAwarenessContext([IntPtr](-4))
$OutDir = 'E:\vietkb\docs\evkey-clone-work\qa\shots'

$icon = Get-TrayIconElement
if ($icon) {
  "icon: $($icon.Current.Name)"
  $fly = $null
  foreach ($w in [System.Windows.Automation.AutomationElement]::RootElement.FindAll([System.Windows.Automation.TreeScope]::Children, [System.Windows.Automation.Condition]::TrueCondition)) {
    if ($w.Current.ClassName -eq 'TopLevelWindowForOverflowXamlIsland' -and -not $w.Current.IsOffscreen) { $fly = $w }
  }
  if ($fly) {
    [void][Cap94H]::SetCursorPos(200, 200)
    Start-Sleep -Milliseconds 500
    $h = [IntPtr]$fly.Current.NativeWindowHandle
    $f = [Cap94H]::Frame($h)
    $bmp = New-Object System.Drawing.Bitmap($f[2], $f[3])
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.CopyFromScreen($f[0], $f[1], 0, 0, $bmp.Size)
    $bmp.Save((Join-Path $OutDir 'tray-flyout.png'), [System.Drawing.Imaging.ImageFormat]::Png)
    $g.Dispose(); $bmp.Dispose()
    "SHOT tray-flyout.png $($f[2])x$($f[3]) at $($f[0]),$($f[1])"
  } else { "flyout: NOT FOUND" }
} else { "icon: NOT FOUND" }

# quit through the tray menu
$menu = Open-TrayMenu
"menu=$menu"
if ($menu -and $menu -ne [IntPtr]::Zero) {
  $ok = Invoke-MenuItem $menu 'Thoát VietKB'
  "quit invoked=$ok"
}
Start-Sleep -Milliseconds 1500
$left = @(Get-Process vietkb -ErrorAction SilentlyContinue)
"vietkb-processes=$($left.Count)"
if ($left.Count -gt 0) { $left | ForEach-Object { "LEFT pid=$($_.Id) path=$($_.Path)" } }
