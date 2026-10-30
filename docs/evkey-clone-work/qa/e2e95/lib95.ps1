# QA E2E harness helpers, gate 0.2.4 (docs/evkey-clone-work/95-e2e-report.md)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes

Add-Type @'
using System;
using System.Runtime.InteropServices;
using System.Threading;

public static class QAInput {
  [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr v);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool BringWindowToTop(IntPtr h);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int cmd);
  [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr h);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint p);
  [DllImport("kernel32.dll")] public static extern uint GetCurrentThreadId();
  [DllImport("user32.dll")] public static extern bool AttachThreadInput(uint a, uint b, bool f);
  [DllImport("user32.dll")] public static extern short VkKeyScanW(char c);
  [DllImport("user32.dll")] public static extern uint MapVirtualKeyW(uint code, uint type);
  [DllImport("user32.dll")] public static extern uint SendInput(uint n, INPUT[] p, int cb);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint flags, int dx, int dy, uint data, IntPtr extra);
  [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr h);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern IntPtr FindWindowW(string cls, string name);
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc cb, IntPtr p);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern int GetWindowTextW(IntPtr h, System.Text.StringBuilder s, int n);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);

  public delegate bool EnumProc(IntPtr h, IntPtr p);

  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  [StructLayout(LayoutKind.Sequential)] public struct KEYBDINPUT { public ushort wVk; public ushort wScan; public uint dwFlags; public uint time; public IntPtr extra; }
  [StructLayout(LayoutKind.Sequential)] public struct MOUSEINPUT { public int dx; public int dy; public uint data; public uint flags; public uint time; public IntPtr extra; }
  [StructLayout(LayoutKind.Sequential)] public struct HARDINPUT { public uint msg; public ushort p1; public ushort p2; }
  [StructLayout(LayoutKind.Explicit)] public struct InputUnion {
    [FieldOffset(0)] public MOUSEINPUT mi;
    [FieldOffset(0)] public KEYBDINPUT ki;
    [FieldOffset(0)] public HARDINPUT hi;
  }
  [StructLayout(LayoutKind.Sequential)] public struct INPUT { public uint type; public InputUnion u; }

  const uint KEYEVENTF_KEYUP = 0x0002;
  const uint KEYEVENTF_UNICODE = 0x0004;
  const uint KEYEVENTF_SCANCODE = 0x0008;

  static void Raw(ushort vk, bool up) {
    INPUT[] a = new INPUT[1];
    a[0].type = 1;
    a[0].u.ki.wVk = vk;
    a[0].u.ki.wScan = (ushort)MapVirtualKeyW(vk, 0);
    a[0].u.ki.dwFlags = up ? KEYEVENTF_KEYUP : 0;
    SendInput(1, a, Marshal.SizeOf(typeof(INPUT)));
  }
  public static void KeyVk(ushort vk, bool shift, bool ctrl, bool alt) {
    if (alt) Raw(0x12, false);
    if (ctrl) Raw(0x11, false);
    if (shift) Raw(0x10, false);
    Raw(vk, false);
    Thread.Sleep(8);
    Raw(vk, true);
    if (shift) Raw(0x10, true);
    if (ctrl) Raw(0x11, true);
    if (alt) Raw(0x12, true);
  }
  public static void Combo(ushort vk, bool shift, bool ctrl, bool alt) { KeyVk(vk, shift, ctrl, alt); }

  /// types ASCII through the real VK path so the low-level hook sees ordinary keystrokes
  public static void TypeAscii(string s, int delayMs) {
    foreach (char c in s) {
      short m = VkKeyScanW(c);
      if (m == -1) {
        INPUT[] a = new INPUT[2];
        a[0].type = 1; a[0].u.ki.wVk = 0; a[0].u.ki.wScan = c; a[0].u.ki.dwFlags = KEYEVENTF_UNICODE;
        a[1].type = 1; a[1].u.ki.wVk = 0; a[1].u.ki.wScan = c; a[1].u.ki.dwFlags = KEYEVENTF_UNICODE | KEYEVENTF_KEYUP;
        SendInput(2, a, Marshal.SizeOf(typeof(INPUT)));
      } else {
        byte vk = (byte)(m & 0xFF);
        byte st = (byte)((m >> 8) & 0xFF);
        KeyVk((ushort)vk, (st & 1) != 0, (st & 2) != 0, (st & 4) != 0);
      }
      if (delayMs > 0) Thread.Sleep(delayMs);
    }
  }

  public static bool Focus(IntPtr h) {
    if (IsIconic(h)) ShowWindow(h, 9); // SW_RESTORE
    IntPtr fg = GetForegroundWindow();
    if (fg == h) return true;
    uint p1 = 0, p2 = 0;
    GetWindowThreadProcessId(h, out p1);
    GetWindowThreadProcessId(fg, out p2);
    uint me = GetCurrentThreadId();
    AttachThreadInput(me, p2, true);
    AttachThreadInput(me, p1, true);
    BringWindowToTop(h);
    SetForegroundWindow(h);
    AttachThreadInput(me, p1, false);
    AttachThreadInput(me, p2, false);
    Thread.Sleep(120);
    return GetForegroundWindow() == h;
  }

  public static IntPtr[] TopWindows() {
    var list = new System.Collections.Generic.List<IntPtr>();
    EnumWindows((h, p) => { if (IsWindowVisible(h)) list.Add(h); return true; }, IntPtr.Zero);
    return list.ToArray();
  }
  public static string Title(IntPtr h) {
    var sb = new System.Text.StringBuilder(512);
    GetWindowTextW(h, sb, 512);
    return sb.ToString();
  }
  public static uint Pid(IntPtr h) { uint p = 0; GetWindowThreadProcessId(h, out p); return p; }
}
'@

try { [void][QAInput]::SetProcessDpiAwarenessContext([IntPtr](-4)) } catch {}

function Log($s) { Write-Host $s }

# ---- window discovery ----------------------------------------------------
function Get-TopWindow([string]$titleMatch, [int]$excludePid = 0) {
  foreach ($h in [QAInput]::TopWindows()) {
    $t = [QAInput]::Title($h)
    if ($t -and $t -like "*$titleMatch*") {
      if ($excludePid -ne 0 -and [QAInput]::Pid($h) -eq $excludePid) { continue }
      return [pscustomobject]@{ Hwnd = $h; Title = $t; Pid = [QAInput]::Pid($h) }
    }
  }
  return $null
}

# ---- UIA text read-back --------------------------------------------------
function Get-UiaText([IntPtr]$hwnd) {
  $out = [pscustomobject]@{ Text = $null; Via = 'none'; Element = '' }
  try { $root = [System.Windows.Automation.AutomationElement]::FromHandle($hwnd) } catch { return $out }
  if (-not $root) { return $out }
  $all = $null
  try { $all = $root.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition) } catch { return $out }
  $best = $null; $bestVia = ''; $bestScore = 0
  foreach ($el in $all) {
    $ct = ''
    try { $ct = $el.Current.ControlType.ProgrammaticName } catch {}
    $score = 1
    if ($ct -eq 'ControlType.Edit') { $score = 3 }
    if ($ct -eq 'ControlType.Document') { $score = 4 }
    $txt = $null; $via = ''
    try {
      $tp = $el.GetCurrentPattern([System.Windows.Automation.TextPattern]::Pattern)
      if ($tp) { $txt = $tp.DocumentRange.GetText(-1); $via = 'TextPattern' }
    } catch {}
    if ($null -eq $txt) {
      try {
        $vp = $el.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern)
        if ($vp) { $txt = $vp.Current.Value; $via = 'ValuePattern' }
      } catch {}
    }
    if ($null -ne $txt -and $score -gt $bestScore) { $best = $txt; $bestVia = "$via/$ct"; $bestScore = $score }
    if ($score -eq 4 -and $null -ne $txt) { break }
  }
  if ($null -ne $best) { $out.Text = $best; $out.Via = $bestVia }
  return $out
}

function Get-EditElement([IntPtr]$hwnd) {
  try { $root = [System.Windows.Automation.AutomationElement]::FromHandle($hwnd) } catch { return $null }
  if (-not $root) { return $null }
  $all = $root.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)
  $best = $null; $bestScore = 0
  foreach ($el in $all) {
    $ct = ''; try { $ct = $el.Current.ControlType.ProgrammaticName } catch { continue }
    $score = 0
    if ($ct -eq 'ControlType.Document') { $score = 4 }
    elseif ($ct -eq 'ControlType.Edit') { $score = 3 }
    elseif ($ct -eq 'ControlType.Pane' -and $el.Current.Name -match 'Text Editor') { $score = 2 }
    if ($score -gt $bestScore) {
      $has = $false
      try { [void]$el.GetCurrentPattern([System.Windows.Automation.TextPattern]::Pattern); $has = $true } catch {}
      if (-not $has) { try { [void]$el.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern); $has = $true } catch {} }
      if ($has) { $best = $el; $bestScore = $score }
    }
  }
  return $best
}

function Click-Element($el) {
  if (-not $el) { return $false }
  try {
    $r = $el.Current.BoundingRectangle
    if ($r.Width -le 1 -or $r.Height -le 1) { return $false }
    $x = [int]($r.X + $r.Width / 2); $y = [int]($r.Y + [Math]::Min(40, $r.Height / 2))
    [void][QAInput]::SetCursorPos($x, $y)
    Start-Sleep -Milliseconds 40
    [QAInput]::mouse_event(0x0002, 0, 0, 0, [IntPtr]::Zero)
    [QAInput]::mouse_event(0x0004, 0, 0, 0, [IntPtr]::Zero)
    Start-Sleep -Milliseconds 120
    return $true
  } catch { return $false }
}

function Click-Center([IntPtr]$hwnd) {
  $r = New-Object QAInput+RECT
  [void][QAInput]::GetClientRect($hwnd, [ref]$r)
  $w = New-Object QAInput+RECT
  [void][QAInput]::GetWindowRect($hwnd, [ref]$w)
  $x = [int](($w.L + $w.R) / 2); $y = [int]($w.T + ($w.B - $w.T) * 0.35)
  [void][QAInput]::SetCursorPos($x, $y)
  Start-Sleep -Milliseconds 40
  [QAInput]::mouse_event(0x0002, 0, 0, 0, [IntPtr]::Zero)
  [QAInput]::mouse_event(0x0004, 0, 0, 0, [IntPtr]::Zero)
  Start-Sleep -Milliseconds 120
}

function Get-BackupClipboard {
  try { return (Get-Clipboard -Raw -ErrorAction SilentlyContinue) } catch { return $null }
}

# clipboard read-back is a documented fallback only (UIA is the primary source)
function Get-ClipboardText([IntPtr]$hwnd) {
  [void][QAInput]::Focus($hwnd)
  [QAInput]::Combo(0x41, $false, $true, $false)   # Ctrl+A
  Start-Sleep -Milliseconds 80
  [QAInput]::Combo(0x43, $false, $true, $false)   # Ctrl+C
  Start-Sleep -Milliseconds 200
  $t = ''
  try { $t = Get-Clipboard -Raw -ErrorAction SilentlyContinue } catch {}
  if ($null -eq $t) { $t = '' }
  return [pscustomobject]@{ Text = $t; Via = 'Clipboard' }
}

function Clear-TextArea([IntPtr]$hwnd) {
  [void][QAInput]::Focus($hwnd)
  [QAInput]::Combo(0x41, $false, $true, $false)
  [QAInput]::Combo(0x2E, $false, $false, $false)  # Delete
  Start-Sleep -Milliseconds 60
}

function Read-Text([IntPtr]$hwnd, [switch]$ForceClipboard) {
  if (-not $ForceClipboard) {
    $u = Get-UiaText $hwnd
    if ($null -ne $u.Text -and $u.Via -ne 'none') { return $u }
  }
  return (Get-ClipboardText $hwnd)
}

# ---- app processes -------------------------------------------------------
function Get-VkbProcess { Get-Process vietkb -ErrorAction SilentlyContinue | Select-Object -First 1 }

function Get-VkbChildren {
  $app = Get-VkbProcess
  if (-not $app) { return @() }
  $all = Get-CimInstance Win32_Process -Filter "Name='msedgewebview2.exe'" -ErrorAction SilentlyContinue
  $byParent = @{}
  foreach ($p in $all) { if (-not $byParent.ContainsKey([int]$p.ParentProcessId)) { $byParent[[int]$p.ParentProcessId] = @() }; $byParent[[int]$p.ParentProcessId] += $p }
  $result = @(); $queue = @([int]$app.Id)
  while ($queue.Count -gt 0) {
    $cur = $queue[0]; $queue = @($queue | Select-Object -Skip 1)
    if ($byParent.ContainsKey($cur)) { foreach ($c in $byParent[$cur]) { $result += $c; $queue += [int]$c.ProcessId } }
  }
  return $result
}

function Get-VkbMemory {
  $app = Get-VkbProcess
  if (-not $app) { return [pscustomobject]@{ AppMB = 0; TotalMB = 0; WebViewCount = 0 } }
  $appMB = [math]::Round($app.WorkingSet64 / 1MB, 1)
  $kids = Get-VkbChildren
  $kidMB = 0.0
  foreach ($k in $kids) { $kidMB += [math]::Round($k.WorkingSetSize / 1MB, 1) }
  return [pscustomobject]@{ AppMB = $appMB; TotalMB = [math]::Round($kidMB + $appMB, 1); WebViewCount = @($kids).Count }
}

function Find-ElementByName([IntPtr]$hwnd, [string]$name, [string]$typePattern = '') {
  try { $root = [System.Windows.Automation.AutomationElement]::FromHandle($hwnd) } catch { return $null }
  if (-not $root) { return $null }
  $all = $root.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)
  foreach ($el in $all) {
    try {
      if ($el.Current.Name -ne $name) { continue }
      if ($typePattern -and $el.Current.ControlType.ProgrammaticName -notmatch $typePattern) { continue }
      return $el
    } catch {}
  }
  return $null
}

# Clicks (or invokes) an element by accessible name; returns $true when the action ran.
function Invoke-Named([IntPtr]$hwnd, [string]$name, [string]$typePattern = '') {
  $el = Find-ElementByName $hwnd $name $typePattern
  if (-not $el) { return $false }
  try {
    $ip = $el.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern)
    if ($ip) { $ip.Invoke(); Start-Sleep -Milliseconds 250; return $true }
  } catch {}
  try {
    $sp = $el.GetCurrentPattern([System.Windows.Automation.SelectionItemPattern]::Pattern)
    if ($sp) { $sp.Select(); Start-Sleep -Milliseconds 200; return $true }
  } catch {}
  return (Click-Element $el)
}

# Notepad (Win11) restores its previous session and may raise a modal on top;
# dismiss "Keep unsaved changes?" by discarding only our scratch file, then
# make sure our scratch tab is the active document.
function Repair-Notepad([string]$scratchName) {
  for ($i = 0; $i -lt 6; $i++) {
    $w = Get-TopWindow $scratchName
    if (-not $w) { return $false }
    $modal = Find-ElementByName $w.Hwnd 'Keep unsaved changes?' 'Window'
    if ($modal) {
      Write-Host 'notepad: dismissing Keep-unsaved-changes modal'
      [void](Invoke-Named $w.Hwnd 'Reopen file' 'Button')
      Start-Sleep -Milliseconds 600
      continue
    }
    $tab = Find-ElementByName $w.Hwnd $scratchName 'Text|TabItem'
    if ($tab) { [void](Click-Element $tab); Start-Sleep -Milliseconds 300 }
    return $true
  }
  return $false
}

function Wait-Window([string]$titleMatch, [int]$timeoutSec = 20) {  $end = (Get-Date).AddSeconds($timeoutSec)
  while ((Get-Date) -lt $end) {
    $w = Get-TopWindow $titleMatch
    if ($w) { return $w }
    Start-Sleep -Milliseconds 250
  }
  return $null
}
