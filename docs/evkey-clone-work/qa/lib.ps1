# Shared UIA helpers for the §3 UI run. Dot-source from a case script.
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes

$script:CfgPath = Join-Path $env:APPDATA 'VietKB\config.toml'
$script:EvidenceDir = 'C:\Users\namth\AppData\Local\Temp\opencode\vkb-qa\evidence'
if (-not (Test-Path $script:EvidenceDir)) { New-Item -ItemType Directory -Path $script:EvidenceDir | Out-Null }

function Get-AppProcess {
  $p = Get-Process vietkb -ErrorAction SilentlyContinue | Select-Object -First 1
  if (-not $p) { throw 'vietkb is not running' }
  return $p
}

function Get-AppWindows {
  $p = Get-AppProcess
  $cond = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ProcessIdProperty, $p.Id)
  return [System.Windows.Automation.AutomationElement]::RootElement.FindAll([System.Windows.Automation.TreeScope]::Children, $cond)
}

function Get-AppWindow([string]$Title) {
  foreach ($w in Get-AppWindows) { if ($w.Current.Name -eq $Title) { return $w } }
  return $null
}

function Wait-AppWindow([string]$Title, [int]$TimeoutMs = 5000) {
  $deadline = (Get-Date).AddMilliseconds($TimeoutMs)
  do {
    $w = Get-AppWindow $Title
    if ($w) { return $w }
    Start-Sleep -Milliseconds 100
  } while ((Get-Date) -lt $deadline)
  return $null
}

function New-AndCondition([object[]]$Conds) {
  $list = @($Conds | Where-Object { $null -ne $_ })
  if ($list.Count -eq 0) { return [System.Windows.Automation.Condition]::TrueCondition }
  if ($list.Count -eq 1) { return $list[0] }
  return New-Object System.Windows.Automation.AndCondition($list)
}

function Find-El($Root, [string]$Name, [string]$Class = '', $Type = $null) {
  $conds = @()
  if ($Name -ne '') { $conds += New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::NameProperty, $Name) }
  if ($Class -ne '') { $conds += New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ClassNameProperty, $Class) }
  if ($Type) { $conds += New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ControlTypeProperty, $Type) }
  return $Root.FindFirst([System.Windows.Automation.TreeScope]::Descendants, (New-AndCondition $conds))
}

function Find-Els($Root, [string]$Name, [string]$Class = '', $Type = $null) {
  $conds = @()
  if ($Name -ne '') { $conds += New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::NameProperty, $Name) }
  if ($Class -ne '') { $conds += New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ClassNameProperty, $Class) }
  if ($Type) { $conds += New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ControlTypeProperty, $Type) }
  return $Root.FindAll([System.Windows.Automation.TreeScope]::Descendants, (New-AndCondition $conds))
}

function Get-AllButtons($Root) {
  $c = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ControlTypeProperty, [System.Windows.Automation.ControlType]::Button)
  return $Root.FindAll([System.Windows.Automation.TreeScope]::Descendants, $c)
}

function Invoke-El($El) {
  $p = $El.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern)
  $p.Invoke()
  Start-Sleep -Milliseconds 450
}

function Invoke-Button($Root, [string]$Name, [string]$Class = '') {
  $el = Find-El $Root $Name $Class ([System.Windows.Automation.ControlType]::Button)
  if (-not $el) { throw "button not found: $Name class='$Class'" }
  Invoke-El $el
  return $el
}

function Get-ToggleElement($Root, [string]$Name) {
  foreach ($el in (Find-Els $Root $Name '' ([System.Windows.Automation.ControlType]::Button))) {
    if ($el.Current.ClassName -match '(^|\s)tg(\s|$)') { return $el }
  }
  return $null
}

function Toggle-El($El) {
  $p = $El.GetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern)
  $before = $p.Current.ToggleState
  $p.Toggle()
  Start-Sleep -Milliseconds 450
  return @($before, $p.Current.ToggleState)
}

function Set-Toggle($Root, [string]$Name, [bool]$On) {
  $el = Get-ToggleElement $Root $Name
  if (-not $el) { throw "toggle not found: $Name" }
  $p = $el.GetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern)
  $isOn = $p.Current.ToggleState -eq [System.Windows.Automation.ToggleState]::On
  if ($isOn -ne $On) { $p.Toggle(); Start-Sleep -Milliseconds 450 }
  return $p.Current.ToggleState
}

function Get-Toggle($Root, [string]$Name) {
  $el = Get-ToggleElement $Root $Name
  if (-not $el) { return $null }
  return $el.GetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern).Current.ToggleState
}

function Set-Combo($Root, [string]$Name, [string]$Option) {
  $combo = Find-El $Root $Name 'sel' ([System.Windows.Automation.ControlType]::ComboBox)
  if (-not $combo) { throw "combo not found: $Name" }
  $ec = $combo.GetCurrentPattern([System.Windows.Automation.ExpandCollapsePattern]::Pattern)
  $ec.Expand()
  Start-Sleep -Milliseconds 500
  $item = Find-El $Root $Option '' ([System.Windows.Automation.ControlType]::ListItem)
  if (-not $item) { $ec.Collapse(); throw "option not found: $Option in $Name" }
  $item.GetCurrentPattern([System.Windows.Automation.SelectionItemPattern]::Pattern).Select()
  Start-Sleep -Milliseconds 600
  try { if ($ec.Current.ExpandCollapseState -eq [System.Windows.Automation.ExpandCollapseState]::Expanded) { $ec.Collapse() } } catch {}
  Start-Sleep -Milliseconds 300
  return (Find-El $Root $Name 'sel' ([System.Windows.Automation.ControlType]::ComboBox)).GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern).Current.Value
}

function Get-Combo($Root, [string]$Name) {
  $combo = Find-El $Root $Name 'sel' ([System.Windows.Automation.ControlType]::ComboBox)
  if (-not $combo) { return $null }
  return $combo.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern).Current.Value
}

function Get-ComboOptions($Root, [string]$Name) {
  $combo = Find-El $Root $Name 'sel' ([System.Windows.Automation.ControlType]::ComboBox)
  if (-not $combo) { return @() }
  $ec = $combo.GetCurrentPattern([System.Windows.Automation.ExpandCollapsePattern]::Pattern)
  $ec.Expand()
  Start-Sleep -Milliseconds 500
  $items = @()
  $c = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ControlTypeProperty, [System.Windows.Automation.ControlType]::ListItem)
  foreach ($i in $Root.FindAll([System.Windows.Automation.TreeScope]::Descendants, $c)) { $items += $i.Current.Name }
  try { $ec.Collapse(); Start-Sleep -Milliseconds 250 } catch {}
  return $items
}

function Set-TextValue($El, [string]$Value) {
  $vp = $El.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern)
  $vp.SetValue($Value)
  Start-Sleep -Milliseconds 250
}

function Get-Focused($Root) {
  $c = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::HasKeyboardFocusProperty, $true)
  return $Root.FindAll([System.Windows.Automation.TreeScope]::Descendants, $c)
}

function Get-ConfigRaw { if (Test-Path $script:CfgPath) { return Get-Content -Raw $script:CfgPath } else { return '' } }

function Get-Cfg([string]$Section, [string]$Key) {
  $raw = Get-ConfigRaw
  $inSection = $false
  foreach ($line in ($raw -split "`r?`n")) {
    if ($line -match '^\s*\[(.+)\]\s*$') { $inSection = ($matches[1] -eq $Section); continue }
    if ($inSection -and $line -match "^\s*$([regex]::Escape($Key))\s*=\s*(.*)$") { return $matches[1].Trim() }
  }
  return $null
}

function Wait-Cfg([string]$Section, [string]$Key, [string]$Expected, [int]$TimeoutMs = 3000) {
  $deadline = (Get-Date).AddMilliseconds($TimeoutMs)
  do {
    $v = Get-Cfg $Section $Key
    if ($null -ne $v -and $v -eq $Expected) { return $true }
    Start-Sleep -Milliseconds 100
  } while ((Get-Date) -lt $deadline)
  return $false
}

function Wait-True([scriptblock]$Probe, [int]$TimeoutMs = 3000) {
  $deadline = (Get-Date).AddMilliseconds($TimeoutMs)
  do {
    if (& $Probe) { return $true }
    Start-Sleep -Milliseconds 100
  } while ((Get-Date) -lt $deadline)
  return $false
}

function Save-Evidence([string]$Id, [string[]]$Lines) {  $path = Join-Path $script:EvidenceDir "$Id.txt"
  $Lines | Set-Content -Encoding UTF8 $path
  return $path
}

function Dump-Tree($Root, [string]$Id) {
  $lines = New-Object System.Collections.Generic.List[string]
  $walker = [System.Windows.Automation.TreeWalker]::ControlViewWalker
  function Walk($el, $d) {
    $ai = $el.Current
    $lines.Add(('  ' * $d) + "[$($ai.ControlType.ProgrammaticName)] name='$($ai.Name)' class='$($ai.ClassName)' en=$($ai.IsEnabled) off=$($ai.IsOffscreen) rect=$($ai.BoundingRectangle)")
    $ch = $walker.GetFirstChild($el); $k = 0
    while ($null -ne $ch -and $k -lt 200) { Walk $ch ($d + 1); $ch = $walker.GetNextSibling($ch); $k++ }
  }
  Walk $Root 0
  return Save-Evidence $Id $lines
}

# ---------------------------------------------------------------- lifecycle
$script:ExePath = 'E:\vietkb\target\release\vietkb.exe'
$script:TestPid = $null
$script:LaunchCount = 0
$script:AppLog = $null
$script:InputHwnd = $null

function Start-TestApp([int]$WaitMs = 12000) {
  $existing = @(Get-Process vietkb -ErrorAction SilentlyContinue)
  foreach ($e in $existing) { $script:TestPid = $e.Id }
  if ($existing.Count -eq 0) {
    $script:LaunchCount++
    $script:AppLog = Join-Path 'C:\Users\namth\AppData\Local\Temp\opencode\vkb-qa' ("app-$($script:LaunchCount).log")
    Remove-Item $script:AppLog -ErrorAction SilentlyContinue
    $p = Start-Process -FilePath $script:ExePath -PassThru -RedirectStandardOutput $script:AppLog
    $script:TestPid = $p.Id
  }
  $deadline = (Get-Date).AddMilliseconds($WaitMs)
  do {
    $w = Get-AppWindow 'Cài đặt VietKB'
    if ($w) {
      # The shell renders before `app.settings` resolves; wait for a real control.
      $ready = (Get-Date).AddSeconds(8)
      do {
        if (Get-ToggleElement $w 'Bật tiếng Việt') { Start-Sleep -Milliseconds 300; return Get-AppWindow 'Cài đặt VietKB' }
        Start-Sleep -Milliseconds 150
      } while ((Get-Date) -lt $ready)
      throw 'settings UI did not render (no Bật tiếng Việt toggle)'
    }
    Start-Sleep -Milliseconds 150
  } while ((Get-Date) -lt $deadline)
  throw 'app window did not appear'
}

function Stop-TestApp {
  if ($script:TestPid) {
    Stop-Process -Id $script:TestPid -Force -ErrorAction SilentlyContinue
  } else {
    Get-Process vietkb -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
  }
  Start-Sleep -Milliseconds 900
  $script:TestPid = $null
}

function Restart-TestApp {
  Stop-TestApp
  Start-Sleep -Milliseconds 600
  return Start-TestApp
}

function Get-TrayIconElement {
  $root = [System.Windows.Automation.AutomationElement]::RootElement
  $oc = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ClassNameProperty, 'TopLevelWindowForOverflowXamlIsland')
  $w = $root.FindFirst([System.Windows.Automation.TreeScope]::Children, $oc)
  if (-not $w) {
    # open the overflow flyout
    $tc = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ClassNameProperty, 'Shell_TrayWnd')
    $tray = $root.FindFirst([System.Windows.Automation.TreeScope]::Children, $tc)
    $all = $tray.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)
    foreach ($e in $all) {
      if ($e.Current.ControlType.ProgrammaticName -eq 'ControlType.Button' -and $e.Current.Name -match 'Hidden Icons') {
        $e.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
        Start-Sleep -Milliseconds 900
        break
      }
    }
    $w = $root.FindFirst([System.Windows.Automation.TreeScope]::Children, $oc)
  }
  if (-not $w) { return $null }
  $btnCond = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ControlTypeProperty, [System.Windows.Automation.ControlType]::Button)
  foreach ($b in $w.FindAll([System.Windows.Automation.TreeScope]::Descendants, $btnCond)) {
    if ($b.Current.Name -match '^VietKB') { return $b }
  }
  return $null
}

function Open-TrayMenu {
  [void](Close-AllMenus)
  $icon = Get-TrayIconElement
  if (-not $icon) { throw 'VietKB tray icon not found' }
  $icon.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
  Start-Sleep -Milliseconds 1100
  $p = Get-AppProcess
  $fallback = [IntPtr]::Zero
  foreach ($w in [System.Windows.Automation.AutomationElement]::RootElement.FindAll([System.Windows.Automation.TreeScope]::Children, [System.Windows.Automation.Condition]::TrueCondition)) {
    if ($w.Current.ClassName -ne '#32768' -or $w.Current.IsOffscreen) { continue }
    $h = [IntPtr]$w.Current.NativeWindowHandle
    if ($w.Current.ProcessId -eq $p.Id) {
      $lines = Get-MenuItemLines $h
      if (@($lines | Where-Object { $_ -match '\|Thoát VietKB\|' }).Count -gt 0) { return $h }
      if ($fallback -eq [IntPtr]::Zero) { $fallback = $h }
    }
  }
  return $fallback
}

function Close-TrayMenu([IntPtr]$MenuHwnd) {
  if ($MenuHwnd -eq [IntPtr]::Zero) { return }
  Add-Type @"
using System;
using System.Runtime.InteropServices;
public class MenuKill {
  [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint m, IntPtr w, IntPtr l);
}
"@ -ErrorAction SilentlyContinue
  [void][MenuKill]::PostMessage($MenuHwnd, 0x0100, [IntPtr]0x1B, [IntPtr]::Zero)
  [void][MenuKill]::PostMessage($MenuHwnd, 0x0101, [IntPtr]0x1B, [IntPtr]::Zero)
  Start-Sleep -Milliseconds 300
}

# ---------------------------------------------------------------- input / win32
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class QaWin {
  [StructLayout(LayoutKind.Sequential)] public struct KEYBDINPUT { public ushort wVk; public ushort wScan; public uint dwFlags; public uint time; public IntPtr dwExtraInfo; }
  [StructLayout(LayoutKind.Explicit, Size=40)] public struct INPUT { [FieldOffset(0)] public uint type; [FieldOffset(8)] public KEYBDINPUT ki; }
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
  [DllImport("user32.dll")] public static extern uint SendInput(uint n, INPUT[] inputs, int cb);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern int GetWindowLong(IntPtr h, int idx);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr h);
  public static uint Key(ushort vk, bool up) { INPUT[] i = new INPUT[1]; i[0].type=1; i[0].ki.wVk=vk; i[0].ki.dwFlags = up?2u:0u; return SendInput(1,i,40); }
  public static void Chord(ushort[] mods, ushort key) { foreach (var m in mods) Key(m,false); Key(key,false); Key(key,true); for (int i=mods.Length-1;i>=0;i--) Key(mods[i],true); }
  public static double Scale(IntPtr h) { return GetDpiForWindow(h) / 96.0; }
}
"@

function Send-Key([string]$Name) {
  if ($script:InputHwnd) {
    if ([QaWin]::GetForegroundWindow() -ne $script:InputHwnd) {
      [void][QaWin]::SetForegroundWindow($script:InputHwnd)
      Start-Sleep -Milliseconds 300
      if ([QaWin]::GetForegroundWindow() -ne $script:InputHwnd) { throw 'foreground changed before Send-Key; refusing to type' }
    }
  }
  $map = @{ Tab = 0x09; Enter = 0x0D; Escape = 0x1B; Backspace = 0x08; Space = 0x20; Down = 0x28; Up = 0x26; Right = 0x27; Left = 0x25 }
  $vk = [uint16]$map[$Name]
  [void][QaWin]::Key($vk, $false)
  [void][QaWin]::Key($vk, $true)
  Start-Sleep -Milliseconds 350
}

function Send-Chord([uint16[]]$Mods, [uint16]$Key) {
  if ($script:InputHwnd -and [QaWin]::GetForegroundWindow() -ne $script:InputHwnd) {
    [void][QaWin]::SetForegroundWindow($script:InputHwnd)
    Start-Sleep -Milliseconds 300
    if ([QaWin]::GetForegroundWindow() -ne $script:InputHwnd) { throw 'foreground changed before Send-Chord; refusing to type' }
  }
  [QaWin]::Chord($Mods, $Key)
  Start-Sleep -Milliseconds 500
}

function Focus-AppWindow($Win) {
  $h = [IntPtr]$Win.Current.NativeWindowHandle
  for ($i = 0; $i -lt 4; $i++) {
    [void][QaWin]::SetForegroundWindow($h)
    Start-Sleep -Milliseconds 250
    if ([QaWin]::GetForegroundWindow() -eq $h) { $script:InputHwnd = $h; return $true }
  }
  return $false
}

function Get-Scale($Win) { return [QaWin]::Scale([IntPtr]$Win.Current.NativeWindowHandle) }

function Get-FocusedElement($Root) {
  $cond = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::HasKeyboardFocusProperty, $true)
  $found = $Root.FindAll([System.Windows.Automation.TreeScope]::Descendants, $cond)
  foreach ($e in $found) {
    if ($e.Current.ControlType.ProgrammaticName -in @('ControlType.Button','ControlType.ComboBox','ControlType.Edit','ControlType.CheckBox')) { return $e }
  }
  return $null
}

function Get-ScrollBars($Root) {
  $c = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ControlTypeProperty, [System.Windows.Automation.ControlType]::ScrollBar)
  return $Root.FindAll([System.Windows.Automation.TreeScope]::Descendants, $c)
}

function Get-Dialog($Root, [string]$Title) {
  return Find-El $Root $Title 'sheetdlg' ([System.Windows.Automation.ControlType]::Window)
}

function Get-CfgStr([string]$Section, [string]$Key) {
  $v = Get-Cfg $Section $Key
  if ($null -eq $v) { return $null }
  return $v.Trim('"')
}

$script:Results = New-Object System.Collections.Generic.List[object]
function Add-Result([string]$Id, [string]$Area, [string]$Status, [string]$Detail, [string]$EvidenceId = '') {
  $script:Results.Add([pscustomobject]@{ Id = $Id; Area = $Area; Status = $Status; Detail = $Detail; EvidenceId = $(if ($EvidenceId) { $EvidenceId } else { $Id }) })
  $alive = @(Get-Process vietkb -ErrorAction SilentlyContinue).Count
  Write-Output ("{0,-7} {1,-5} proc={2} {3}" -f $Id, $Status, $alive, $Detail)
}

# ---------------------------------------------------------------- MSAA menus
Add-Type -AssemblyName Accessibility
Add-Type -ReferencedAssemblies Accessibility -TypeDefinition @"
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using Accessibility;
public static class QaMenu {
  [DllImport("oleacc.dll")] static extern int AccessibleObjectFromWindow(IntPtr hwnd, uint dwId, ref Guid riid, [MarshalAs(UnmanagedType.Interface)] out object ppvObject);
  static IAccessible Get(IntPtr hwnd) {
    Guid iid = new Guid("618736E0-3C3D-11CF-810C-00AA00389B71");
    object obj; AccessibleObjectFromWindow(hwnd, 0xFFFFFFFC, ref iid, out obj);
    return (IAccessible)obj;
  }
  public static string[] Describe(IntPtr hwnd) {
    var lines = new List<string>();
    IAccessible acc = Get(hwnd);
    if (acc == null) return lines.ToArray();
    int count = 0; try { count = acc.accChildCount; } catch {}
    for (int i = 0; i <= count; i++) {
      try {
        string name = acc.get_accName(i); object role = acc.get_accRole(i); object state = acc.get_accState(i); string def = null;
        try { def = acc.get_accDefaultAction(i); } catch {}
        lines.Add(i + "|" + name + "|" + role + "|" + state + "|" + def);
      } catch (Exception e) { lines.Add(i + "|ERR|" + e.Message); }
    }
    return lines.ToArray();
  }
  public static bool DoDefault(IntPtr hwnd, string name) {
    IAccessible acc = Get(hwnd);
    if (acc == null) return false;
    int count = acc.accChildCount;
    for (int i = 0; i <= count; i++) {
      string n = null; try { n = acc.get_accName(i); } catch { continue; }
      if (n == name) { acc.accDoDefaultAction(i); return true; }
    }
    return false;
  }
}
"@

function Get-MenuPopups {
  if (-not ('QaMenuEnum' -as [type])) {
  Add-Type @"
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;
public static class QaMenuEnum {
  [DllImport("user32.dll")] static extern bool EnumWindows(EnumProc lpEnumFunc, IntPtr lParam);
  [DllImport("user32.dll")] static extern bool EnumChildWindows(IntPtr hWnd, EnumProc lpEnumFunc, IntPtr lParam);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetClassName(IntPtr hWnd, StringBuilder sb, int max);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hWnd, out RECT r);
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
  delegate bool EnumProc(IntPtr hWnd, IntPtr lParam);
  static string Cls(IntPtr h) { var sb = new StringBuilder(64); GetClassName(h, sb, 64); return sb.ToString(); }
  static void Walk(IntPtr h, List<IntPtr> acc) {
    if (Cls(h) == "#32768") acc.Add(h);
    EnumChildWindows(h, (c, l) => { Walk(c, acc); return true; }, IntPtr.Zero);
  }
  public static IntPtr[] All() {
    var acc = new List<IntPtr>();
    EnumWindows((h, l) => { Walk(h, acc); return true; }, IntPtr.Zero);
    return acc.ToArray();
  }
  public static int[] Rect(IntPtr h) { RECT r; GetWindowRect(h, out r); return new int[] { r.Left, r.Top, r.Right - r.Left, r.Bottom - r.Top }; }
}
"@ -ErrorAction SilentlyContinue
  }
  $list = @()
  foreach ($h in [QaMenuEnum]::All()) {
    $r = [QaMenuEnum]::Rect($h)
    $list += [pscustomobject]@{ Hwnd = $h; Rect = "$($r[0]),$($r[1]),$($r[2])x$($r[3])" }
  }
  return $list
}

function Get-MenuItemLines([IntPtr]$Hwnd) { return [QaMenu]::Describe($Hwnd) }

function Get-MenuItemState([IntPtr]$Hwnd, [string]$Name) {
  foreach ($line in [QaMenu]::Describe($Hwnd)) {
    $parts = $line -split '\|'
    if ($parts.Count -ge 4 -and $parts[1] -eq $Name) { return $parts[3] }
  }
  return $null
}

function Invoke-MenuItem([IntPtr]$Hwnd, [string]$Name) {
  $ok = [QaMenu]::DoDefault($Hwnd, $Name)
  Start-Sleep -Milliseconds 900
  return $ok
}

function Close-AllMenus {
  Add-Type @"
using System;
using System.Runtime.InteropServices;
public class QaMenuClose {
  [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint m, IntPtr w, IntPtr l);
}
"@ -ErrorAction SilentlyContinue
  $root = [System.Windows.Automation.AutomationElement]::RootElement
  $count = 0
  foreach ($w in $root.FindAll([System.Windows.Automation.TreeScope]::Children, [System.Windows.Automation.Condition]::TrueCondition)) {
    if ($w.Current.ClassName -eq '#32768') {
      [void][QaMenuClose]::PostMessage([IntPtr]$w.Current.NativeWindowHandle, 0x001F, [IntPtr]::Zero, [IntPtr]::Zero)
      $count++
    }
  }
  if ($count -gt 0) { Start-Sleep -Milliseconds 350 }
  return $count
}

