# QA round 94 — screenshot capture for the reduced sound feature.
# Attaches to an already-running vietkb.exe; launches nothing, mutates nothing.
param(
  [string]$OutDir = 'E:\vietkb\docs\evkey-clone-work\qa\shots',
  [switch]$Only
)

$ErrorActionPreference = 'Stop'
. 'E:\vietkb\docs\evkey-clone-work\qa\lib.ps1'
Add-Type -AssemblyName System.Drawing
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class Cap94 {
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
  [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr v);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("dwmapi.dll")] public static extern int DwmGetWindowAttribute(IntPtr h, int attr, out RECT r, int cb);
  public static int[] Frame(IntPtr h) {
    RECT r;
    if (DwmGetWindowAttribute(h, 9, out r, Marshal.SizeOf(typeof(RECT))) != 0 || r.Right - r.Left <= 0) {
      GetWindowRect(h, out r);
    }
    return new int[] { r.Left, r.Top, r.Right - r.Left, r.Bottom - r.Top };
  }
}
"@ -ErrorAction SilentlyContinue
[void][Cap94]::SetProcessDpiAwarenessContext([IntPtr](-4))
if (-not (Test-Path $OutDir)) { New-Item -ItemType Directory -Path $OutDir -Force | Out-Null }

function Save-Shot([IntPtr]$h, [string]$Name) {
  [void][Cap94]::SetForegroundWindow($h)
  Start-Sleep -Milliseconds 550
  $r = [Cap94]::Frame($h)
  if ($r[2] -le 0 -or $r[3] -le 0) { throw "window $Name has no area: $($r -join ',')" }
  $bmp = New-Object System.Drawing.Bitmap($r[2], $r[3])
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.CopyFromScreen($r[0], $r[1], 0, 0, $bmp.Size)
  $path = Join-Path $OutDir $Name
  $bmp.Save($path, [System.Drawing.Imaging.ImageFormat]::Png)
  $g.Dispose(); $bmp.Dispose()
  [pscustomobject]@{ file = $path; w = $r[2]; h = $r[3]; x = $r[0]; y = $r[1] }
}

function Hwnd($el) { return [IntPtr]$el.Current.NativeWindowHandle }

$shots = @()

# ---- main window, Vietnamese ON -----------------------------------------
$main = Get-AppWindow 'Cài đặt VietKB'
if (-not $main) { throw 'main window not open' }
$tog = Get-Toggle $main 'Bật tiếng Việt'
$shots += Save-Shot (Hwnd $main) 'main-on.png'
"main-on: toggle=$tog"

# ---- toggle OFF through the UI switch, same window position -------------
if ($tog) {
  Set-Toggle $main 'Bật tiếng Việt' $false
  Start-Sleep -Milliseconds 800
  $shots += Save-Shot (Hwnd (Get-AppWindow 'Cài đặt VietKB')) 'main-off.png'
  "main-off: toggle=$(Get-Toggle (Get-AppWindow 'Cài đặt VietKB') 'Bật tiếng Việt')"
} else {
  "main-off: SKIP (toggle already off)"
}

# ---- advanced window (560 x 388 expected) ------------------------------
$main = Get-AppWindow 'Cài đặt VietKB'
Invoke-Button $main 'Nâng cao…' 'v4lk' | Out-Null
$adv = Wait-AppWindow 'Nâng cao — VietKB' 6000
if (-not $adv) { throw 'advanced window did not open' }
Start-Sleep -Milliseconds 700
$shots += Save-Shot (Hwnd $adv) 'advanced.png'
$scale = Get-Scale $adv
$r = $adv.Current.BoundingRectangle
"advanced: physical=$([int]$r.Width)x$([int]$r.Height) logical=$([math]::Round($r.Width/$scale))x$([math]::Round($r.Height/$scale)) scale=$scale"
$advRows = @(Find-Els $adv 'Âm thanh bật/tắt' '' $null).Count
$advPreview = @(Find-Els $adv 'Nghe thử' '' $null).Count
"advanced: soundRow=$advRows preview=$advPreview"
$closeAdv = Find-El $adv 'Đóng' 'btn pri' ([System.Windows.Automation.ControlType]::Button)
if ($closeAdv) { Invoke-El $closeAdv | Out-Null; Start-Sleep -Milliseconds 500 }

# ---- tray flyout + menu ------------------------------------------------
try {
  $icon = Get-TrayIconElement
  if (-not $icon) { throw 'VietKB tray icon not found in overflow' }
  # Capture the flyout (icon visible) before the menu opens over it.
  $fly = $icon
  for ($i = 0; $i -lt 4; $i++) { if ($fly.Current.NativeWindowHandle -ne 0) { break }; $fly = [System.Windows.Automation.TreeWalker]::ControlViewWalker.GetParent($fly) }
  $fh = [IntPtr]$fly.Current.NativeWindowHandle
  $shots += Save-Shot $fh 'tray.png'
  "tray: flyout class=$($fly.Current.ClassName) hwnd=$fh"
} catch {
  "tray: SKIP — $($_.Exception.Message)"
}

# ---- tray menu items (evidence, no shot of the popup attempted blindly) -
try {
  $menu = Open-TrayMenu
  if ($menu -and $menu -ne [IntPtr]::Zero) {
    $lines = Get-MenuItemLines $menu
    "tray-menu: $($lines -join ' / ')"
    $shots += Save-Shot $menu 'tray-menu.png'
  } else { "tray-menu: not found" }
} catch { "tray-menu: SKIP — $($_.Exception.Message)" }

$shots | ForEach-Object { "SHOT {0} {1}x{2} at {3},{4}" -f $_.file, $_.w, $_.h, $_.x, $_.y }
