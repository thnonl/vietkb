. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
$dir = 'E:\vietkb\docs\evkey-clone-work\qa\e2e95'
$out = Join-Path $dir 'tray-results.txt'
Add-Content -Encoding UTF8 $out 'TRAY (right-click on the flyout icon)'
function W($s) { Add-Content -Encoding UTF8 $out $s; Write-Host $s }

# reopen the overflow flyout: Win+B then Enter on "Show Hidden Icons"
Add-Type -Namespace W -Name K3 -MemberDefinition @'
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
[W.K3]::Tap(0x42, $true); Start-Sleep -Milliseconds 600
for ($i = 0; $i -lt 8; $i++) {
  $n = ''; try { $n = [System.Windows.Automation.AutomationElement]::FocusedElement.Current.Name } catch {}
  if ($n -match 'Hidden Icons') { break }
  [QAInput]::Combo(0x27, $false, $false, $false); Start-Sleep -Milliseconds 250
}
[QAInput]::Combo(0x0D, $false, $false, $false); Start-Sleep -Milliseconds 1200

$icon = $null
foreach ($h in [QAInput]::TopWindows()) {
  try {
    $root = [System.Windows.Automation.AutomationElement]::FromHandle($h)
    foreach ($e in $root.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)) {
      try { if ($e.Current.Name -like 'VietKB ·*') { $icon = $e; break } } catch {}
    }
  } catch {}
  if ($icon) { break }
}
if (-not $icon) { W 'icon=NOT_FOUND'; W 'TRAY-DONE'; exit 0 }
W "icon_name=[$($icon.Current.Name)] rect=$($icon.Current.BoundingRectangle)"
$r = $icon.Current.BoundingRectangle
[void][QAInput]::SetCursorPos([int]($r.X + $r.Width / 2), [int]($r.Y + $r.Height / 2))
Start-Sleep -Milliseconds 200
[QAInput]::mouse_event(0x0008, 0, 0, 0, [IntPtr]::Zero)
[QAInput]::mouse_event(0x0010, 0, 0, 0, [IntPtr]::Zero)
Start-Sleep -Milliseconds 1500

$menus = @()
foreach ($h in [QAInput]::TopWindows()) {
  $t = [QAInput]::Title($h)
  $pn = (Get-Process -Id ([QAInput]::Pid($h)) -ErrorAction SilentlyContinue).ProcessName
  if ($pn -eq 'vietkb' -and $t) { W "vkb_window title=[$t]" }
  try {
    $root = [System.Windows.Automation.AutomationElement]::FromHandle($h)
    foreach ($e in $root.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)) {
      try {
        $ct = $e.Current.ControlType.ProgrammaticName
        if ($ct -match 'MenuItem') { $menus += [pscustomobject]@{ Win = $t; Pid = $pn; Name = $e.Current.Name; El = $e } }
      } catch {}
    }
  } catch {}
}
W "menu_items=$($menus.Count): $((($menus | ForEach-Object { "$($_.Name)@$($_.Pid)" }) -join ' | '))"
$tog = $menus | Where-Object { $_.Pid -eq (Get-VkbProcess).Id -and $_.Name -match 'tiếng Việt' } | Select-Object -First 1
if ($tog) {
  W "tray_item=[$($tog.Name)]"
  try { $tog.El.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke() } catch { [void](Click-Element $tog.El) }
  Start-Sleep -Milliseconds 1500
  $ch = Get-TopWindow 'VietKB E2E probe'
  [void][QAInput]::Focus($ch.Hwnd); Start-Sleep -Milliseconds 250
  $el = Get-EditElement $ch.Hwnd; if ($el) { [void](Click-Element $el) }
  Clear-TextArea $ch.Hwnd
  [QAInput]::TypeAscii('ba6t5', 120); Start-Sleep -Milliseconds 600
  $t = ([string](Get-UiaText $ch.Hwnd).Text).TrimEnd("`r", "`n")
  W "after_tray_toggle=[$t] state=$(if ($t -match 'ậ') { 'ON' } elseif ($t -ceq 'ba6t5') { 'OFF' } else { 'OTHER' })"
  # toggle back through the same route
  [W.K3]::Tap(0x42, $true); Start-Sleep -Milliseconds 500
  for ($i = 0; $i -lt 8; $i++) { $n=''; try { $n = [System.Windows.Automation.AutomationElement]::FocusedElement.Current.Name } catch {}; if ($n -match 'Hidden Icons') { break }; [QAInput]::Combo(0x27,$false,$false,$false); Start-Sleep -Milliseconds 250 }
  [QAInput]::Combo(0x0D, $false, $false, $false); Start-Sleep -Milliseconds 900
  foreach ($h in [QAInput]::TopWindows()) {
    try {
      $root = [System.Windows.Automation.AutomationElement]::FromHandle($h)
      foreach ($e in $root.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)) {
        try { if ($e.Current.Name -like 'VietKB ·*') { $icon = $e; break } } catch {}
      }
    } catch {}
  }
  if ($icon) {
    $r = $icon.Current.BoundingRectangle
    [void][QAInput]::SetCursorPos([int]($r.X + $r.Width / 2), [int]($r.Y + $r.Height / 2)); Start-Sleep -Milliseconds 150
    [QAInput]::mouse_event(0x0008,0,0,0,[IntPtr]::Zero); [QAInput]::mouse_event(0x0010,0,0,0,[IntPtr]::Zero)
    Start-Sleep -Milliseconds 1200
    $menus = @()
    foreach ($h2 in [QAInput]::TopWindows()) {
      try {
        $root2 = [System.Windows.Automation.AutomationElement]::FromHandle($h2)
        foreach ($e in $root2.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)) {
          try { if ($e.Current.ControlType.ProgrammaticName -match 'MenuItem' -and $e.Current.Name -match 'tiếng Việt') { $menus += $e } } catch {}
        }
      } catch {}
    }
    if ($menus.Count -gt 0) { try { $menus[0].GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke() } catch {} ; Start-Sleep -Milliseconds 800 }
    W "icon_name_after=[$((Find-ElementByName ([QAInput]::FindWindowW('Shell_TrayWnd',$null)) 'VietKB ·*').Current.Name)]"
  }
} else { W 'tray_toggle_item=NOT_FOUND'; [QAInput]::Combo(0x1B,$false,$false,$false) }
W 'TRAY-DONE'
