. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
$dir = 'E:\vietkb\docs\evkey-clone-work\qa\e2e95'
$out = Join-Path $dir 'tray-results.txt'
Add-Content -Encoding UTF8 $out 'TRAY (overflow icon, left-click toggle)'
function W($s) { Add-Content -Encoding UTF8 $out $s; Write-Host $s }
Add-Type -Namespace W -Name K5 -MemberDefinition @'
[DllImport("user32.dll")] public static extern uint SendInput(uint n, INPUT[] p, int cb);
[StructLayout(LayoutKind.Sequential)] public struct KEYBDINPUT { public ushort wVk; public ushort wScan; public uint dwFlags; public uint time; public IntPtr extra; }
[StructLayout(LayoutKind.Sequential)] public struct MOUSEINPUT { public int dx; public int dy; public uint data; public uint flags; public uint time; public IntPtr extra; }
[StructLayout(LayoutKind.Sequential)] public struct HARDINPUT { public uint msg; public ushort p1; public ushort p2; }
[StructLayout(LayoutKind.Explicit)] public struct InputUnion { [FieldOffset(0)] public MOUSEINPUT mi; [FieldOffset(0)] public KEYBDINPUT ki; [FieldOffset(0)] public HARDINPUT hi; }
[StructLayout(LayoutKind.Sequential)] public struct INPUT { public uint type; public InputUnion u; }
public static void Tap(ushort vk, bool win) {
  INPUT[] a = new INPUT[4]; int i = 0;
  if (win) { a[i].type = 1; a[i].u.ki.wVk = 0x5B; i++; }
  a[i].type = 1; a[i].u.ki.wVk = vk; i++;
  a[i].type = 1; a[i].u.ki.wVk = vk; a[i].u.ki.dwFlags = 2; i++;
  if (win) { a[i].type = 1; a[i].u.ki.wVk = 0x5B; a[i].u.ki.dwFlags = 2; i++; }
  SendInput((uint)i, a, Marshal.SizeOf(typeof(INPUT)));
}
'@

function OpenFlyout {
  [W.K5]::Tap(0x42, $true); Start-Sleep -Milliseconds 600
  for ($i = 0; $i -lt 8; $i++) {
    $n = ''; try { $n = [System.Windows.Automation.AutomationElement]::FocusedElement.Current.Name } catch {}
    if ($n -match 'Hidden Icons') { break }
    [QAInput]::Combo(0x27, $false, $false, $false); Start-Sleep -Milliseconds 250
  }
  [QAInput]::Combo(0x0D, $false, $false, $false); Start-Sleep -Milliseconds 1100
}
function GetIcon {
  foreach ($h in [QAInput]::TopWindows()) {
    try {
      $root = [System.Windows.Automation.AutomationElement]::FromHandle($h)
      foreach ($e in $root.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)) {
        try {
          $r = $e.Current.BoundingRectangle
          if ($e.Current.Name -like 'VietKB ·*' -and $r.Width -gt 5) { return $e }
        } catch {}
      }
    } catch {}
  }
  return $null
}
function ProbeState($label) {
  $ch = Get-TopWindow 'VietKB E2E probe'
  [void][QAInput]::Focus($ch.Hwnd); Start-Sleep -Milliseconds 250
  $el = Get-EditElement $ch.Hwnd; if ($el) { [void](Click-Element $el) }
  Clear-TextArea $ch.Hwnd
  [QAInput]::TypeAscii('ba6t5', 120); Start-Sleep -Milliseconds 600
  $t = ([string](Get-UiaText $ch.Hwnd).Text).TrimEnd("`r", "`n")
  $st = if ($t -match 'ậ') { 'ON' } elseif ($t -ceq 'ba6t5') { 'OFF' } else { 'OTHER' }
  W ("{0,-26} actual=[{1}] state={2}" -f $label, $t, $st)
  return $st
}

OpenFlyout
$icon = GetIcon
if (-not $icon) { W 'icon=NOT_FOUND'; W 'TRAY-DONE'; exit 0 }
$nameA = $icon.Current.Name
$r = $icon.Current.BoundingRectangle
[void][QAInput]::SetCursorPos([int]($r.X + $r.Width / 2), [int]($r.Y + $r.Height / 2))
Start-Sleep -Milliseconds 200
[QAInput]::mouse_event(0x0002, 0, 0, 0, [IntPtr]::Zero)
[QAInput]::mouse_event(0x0004, 0, 0, 0, [IntPtr]::Zero)
Start-Sleep -Milliseconds 1600
$stateB = ProbeState 'after tray left-click'
OpenFlyout
$icon2 = GetIcon
$nameB = if ($icon2) { $icon2.Current.Name } else { 'gone' }
W "icon_name_before=[$nameA]"
W "icon_name_after =[$nameB]"
W "tray_route_state_flipped=$screen"
# restore ON through the tray again
if ($icon2) {
  $r2 = $icon2.Current.BoundingRectangle
  [void][QAInput]::SetCursorPos([int]($r2.X + $r2.Width / 2), [int]($r2.Y + $r2.Height / 2))
  Start-Sleep -Milliseconds 200
  [QAInput]::mouse_event(0x0002, 0, 0, 0, [IntPtr]::Zero)
  [QAInput]::mouse_event(0x0004, 0, 0, 0, [IntPtr]::Zero)
  Start-Sleep -Milliseconds 1600
  $stateC = ProbeState 'after second tray left-click'
  OpenFlyout
  $icon3 = GetIcon
  W "icon_name_final=[$(if($icon3){$icon3.Current.Name}else{'gone'})] final_state=$stateC"
  [QAInput]::Combo(0x1B, $false, $false, $false)
}
W 'TRAY-DONE'
