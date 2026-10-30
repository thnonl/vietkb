. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
$dir = 'E:\vietkb\docs\evkey-clone-work\qa\e2e95'
$out = Join-Path $dir 'tray-results.txt'
Set-Content -Encoding UTF8 $out 'TRAY (keyboard path: Win+B)'
function W($s) { Add-Content -Encoding UTF8 $out $s; Write-Host $s }

function FocusedName {
  try { return [System.Windows.Automation.AutomationElement]::FocusedElement.Current.Name } catch { return '' }
}
function FocusedType {
  try { return [System.Windows.Automation.AutomationElement]::FocusedElement.Current.ControlType.ProgrammaticName } catch { return '' }
}

# Win+B focuses the notification area
Add-Type -Namespace W -Name K -MemberDefinition @'
[DllImport("user32.dll")] public static extern uint SendInput(uint n, INPUT[] p, int cb);
[StructLayout(LayoutKind.Sequential)] public struct KEYBDINPUT { public ushort wVk; public ushort wScan; public uint dwFlags; public uint time; public IntPtr extra; }
[StructLayout(LayoutKind.Sequential)] public struct MOUSEINPUT { public int dx; public int dy; public uint data; public uint flags; public uint time; public IntPtr extra; }
[StructLayout(LayoutKind.Sequential)] public struct HARDINPUT { public uint msg; public ushort p1; public ushort p2; }
[StructLayout(LayoutKind.Explicit)] public struct InputUnion { [FieldOffset(0)] public MOUSEINPUT mi; [FieldOffset(0)] public KEYBDINPUT ki; [FieldOffset(0)] public HARDINPUT hi; }
[StructLayout(LayoutKind.Sequential)] public struct INPUT { public uint type; public InputUnion u; }
public static void Tap(ushort vk, bool win) {
  INPUT[] a = new INPUT[4];
  int i = 0;
  if (win) { a[i].type = 1; a[i].u.ki.wVk = 0x5B; i++; }
  a[i].type = 1; a[i].u.ki.wVk = vk; i++;
  a[i].type = 1; a[i].u.ki.wVk = vk; a[i].u.ki.dwFlags = 2; i++;
  if (win) { a[i].type = 1; a[i].u.ki.wVk = 0x5B; a[i].u.ki.dwFlags = 2; i++; }
  SendInput((uint)i, a, Marshal.SizeOf(typeof(INPUT)));
}
'@
[W.K]::Tap(0x42, $true)   # Win+B
Start-Sleep -Milliseconds 800
W "focused_after_win_b=[$(FocusedName)] type=[$(FocusedType)]"

$found = $false
for ($i = 0; $i -lt 14; $i++) {
  $n = FocusedName
  if ($n -match 'VietKB|vietkb') { W "focused icon=[$n] at step $i"; $found = $true; break }
  [QAInput]::Combo(0x25, $false, $false, $false)   # Left
  Start-Sleep -Milliseconds 350
  $n2 = FocusedName
  if ($n2 -and $n2 -ne $n) { W "step $i icon=[$n2]" }
  if ($n2 -match 'VietKB|vietkb') { W "focused icon=[$n2] at step $i"; $found = $true; break }
}
if (-not $found) {
  # try Right instead of Left from the start
  for ($i = 0; $i -lt 14; $i++) {
    [QAInput]::Combo(0x27, $false, $false, $false)
    Start-Sleep -Milliseconds 350
    $n2 = FocusedName
    if ($n2 -match 'VietKB|vietkb') { W "focused icon=[$n2] at right step $i"; $found = $true; break }
  }
}
if (-not $found) { W 'tray_icon=NOT_REACHABLE_BY_KEYBOARD'; W 'TRAY-DONE'; exit 0 }

# open its context menu
[W.K]::Tap(0x5D, $false)   # Apps key
Start-Sleep -Milliseconds 1200
$items = @()
foreach ($h in [QAInput]::TopWindows()) {
  try {
    $root = [System.Windows.Automation.AutomationElement]::FromHandle($h)
    foreach ($e in $root.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)) {
      try {
        if ($e.Current.ControlType.ProgrammaticName -match 'MenuItem') { $items += [pscustomobject]@{ Win = $h; Name = $e.Current.Name; El = $e } }
      } catch {}
    }
  } catch {}
}
W "menu_items=$($items.Count): $((($items | ForEach-Object { $_.Name }) -join ' | '))"
$toggle = $items | Where-Object { $_.Name -match 'tiếng Việt|Tiếng Việt' } | Select-Object -First 1
if ($toggle) {
  W "tray_menu_item=[$($toggle.Name)]"
  try { $toggle.El.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke() } catch { [void](Click-Element $toggle.El) }
  Start-Sleep -Milliseconds 1200
  W 'tray_toggle_invoked=true'
} else { W 'tray_toggle_item=NOT_FOUND'; [QAInput]::Combo(0x1B, $false, $false, $false) }
W 'TRAY-DONE'
