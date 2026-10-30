# QA round 94 — part G: PrintWindow shots of the two secondary windows + tray menu.
$ErrorActionPreference = 'Continue'
. 'E:\vietkb\docs\evkey-clone-work\qa\lib.ps1'
Add-Type -AssemblyName System.Drawing
Add-Type @"
using System;
using System.Runtime.InteropServices;
using System.Text;
public class Cap94G {
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
  [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr v);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr hdc, uint flags);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("dwmapi.dll")] public static extern int DwmGetWindowAttribute(IntPtr h, int attr, out RECT r, int cb);
  [DllImport("user32.dll")] static extern bool EnumWindows(EnumProc cb, IntPtr l);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowTextW(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetClassName(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint dx, uint dy, uint d, IntPtr e);
  public static void RClick(int x, int y) { SetCursorPos(x, y); System.Threading.Thread.Sleep(120); mouse_event(0x0008,0,0,0,IntPtr.Zero); mouse_event(0x0010,0,0,0,IntPtr.Zero); }
  delegate bool EnumProc(IntPtr h, IntPtr l);
  public static int[] Frame(IntPtr h) {
    RECT r;
    if (DwmGetWindowAttribute(h, 9, out r, Marshal.SizeOf(typeof(RECT))) != 0 || r.Right - r.Left <= 0) { GetWindowRect(h, out r); }
    return new int[] { r.Left, r.Top, r.Right - r.Left, r.Bottom - r.Top };
  }
  public static IntPtr Find(string title, uint pid, string cls) {
    IntPtr found = IntPtr.Zero;
    EnumWindows((h, l) => {
      var t = new StringBuilder(256); GetWindowTextW(h, t, 256);
      var c = new StringBuilder(64); GetClassName(h, c, 64);
      uint p; GetWindowThreadProcessId(h, out p);
      if (p == pid && t.ToString() == title && (cls == null || c.ToString() == cls)) { found = h; return false; }
      return true;
    }, IntPtr.Zero);
    return found;
  }
}
"@ -ErrorAction SilentlyContinue
[void][Cap94G]::SetProcessDpiAwarenessContext([IntPtr](-4))
$OutDir = 'E:\vietkb\docs\evkey-clone-work\qa\shots'
$vp = [uint32](Get-Process vietkb).Id

function Shot-Print([IntPtr]$h, [string]$Name) {
  $f = [Cap94G]::Frame($h)
  $bmp = New-Object System.Drawing.Bitmap($f[2], $f[3])
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $hdc = $g.GetHdc()
  $ok = [Cap94G]::PrintWindow($h, $hdc, 2)
  $g.ReleaseHdc($hdc)
  $g.Dispose()
  $bmp.Save((Join-Path $OutDir $Name), [System.Drawing.Imaging.ImageFormat]::Png)
  $bmp.Dispose()
  return "$Name $($f[2])x$($f[3]) pw=$ok"
}

$adv = [Cap94G]::Find('Nâng cao — VietKB', $vp, 'Tauri Window')
$hk = [Cap94G]::Find('Phím tắt — VietKB', $vp, 'Tauri Window')
"advanced hwnd=$adv ; hotkeys hwnd=$hk"
if ($adv -ne [IntPtr]::Zero) { "SHOT $(Shot-Print $adv 'advanced.png')" }
if ($hk -ne [IntPtr]::Zero) { "SHOT $(Shot-Print $hk 'hotkeys.png')" }

# ---- tray menu, right-click, cursor parked, capture ------------------
$icon = Get-TrayIconElement
if ($icon) {
  $ir = $icon.Current.BoundingRectangle
  [Cap94G]::RClick([int]($ir.X + $ir.Width / 2), [int]($ir.Y + $ir.Height / 2))
  Start-Sleep -Milliseconds 900
  [void][Cap94G]::SetCursorPos(200, 200)
  Start-Sleep -Milliseconds 700
  $menu = [Cap94G]::Find('', $vp, '#32768')
  "menu hwnd=$menu"
  if ($menu -ne [IntPtr]::Zero) {
    "items: $((Get-MenuItemLines $menu) -join ' / ')"
    $f = [Cap94G]::Frame($menu)
    $bmp = New-Object System.Drawing.Bitmap($f[2], $f[3])
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.CopyFromScreen($f[0], $f[1], 0, 0, $bmp.Size)
    $bmp.Save((Join-Path $OutDir 'tray.png'), [System.Drawing.Imaging.ImageFormat]::Png)
    $g.Dispose(); $bmp.Dispose()
    "SHOT tray.png $($f[2])x$($f[3]) at $($f[0]),$($f[1])"
    [void](Close-TrayMenu $menu)
  } else { "tray-menu: NOT FOUND" }
} else { "tray-icon: NOT FOUND" }
