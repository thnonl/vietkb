. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
$dir = 'E:\vietkb\docs\evkey-clone-work\qa\e2e95'
$out = Join-Path $dir 'tray-results.txt'
Set-Content -Encoding UTF8 $out 'TRAY-E2E'
function W($s) { Add-Content -Encoding UTF8 $out $s; Write-Host $s }

$ch = Get-TopWindow 'VietKB E2E probe'
function Observe($label) {
  if ($ch) {
    [void][QAInput]::Focus($ch.Hwnd); Start-Sleep -Milliseconds 250
    $el = Get-EditElement $ch.Hwnd; if ($el) { [void](Click-Element $el) }
    Clear-TextArea $ch.Hwnd
    [QAInput]::TypeAscii('ba6t5', 120)
    Start-Sleep -Milliseconds 600
    $t = ([string](Get-UiaText $ch.Hwnd).Text).TrimEnd("`r", "`n")
    $st = if ($t -match 'ậ') { 'ON' } elseif ($t -ceq 'ba6t5') { 'OFF' } else { "OTHER($t)" }
    W ("{0,-30} actual=[{1}] state={2}" -f $label, $t, $st)
    return $st
  }
  return 'NO_OBSERVER'
}

$tray = [QAInput]::FindWindowW('Shell_TrayWnd', $null)
W "tray_hwnd=$tray"
$trayRoot = [System.Windows.Automation.AutomationElement]::FromHandle($tray)
$kids = $trayRoot.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)
$vkbBtn = $null; $overflowBtn = $null
foreach ($el in $kids) {
  try {
    $n = $el.Current.Name
    if ($n -match 'VietKB') { $vkbBtn = $el; W "tray_element name=[$n] type=$($el.Current.ControlType.ProgrammaticName) rect=$($el.Current.BoundingRectangle)" }
    if ($n -match 'hidden icon|biểu tượng ẩn') { $overflowBtn = $el; W "overflow_button name=[$n] rect=$($el.Current.BoundingRectangle)" }
  } catch {}
}
if (-not $vkbBtn -and $overflowBtn) {
  W 'vietkb icon not in the visible tray; opening the overflow flyout'
  [void](Click-Element $overflowBtn)
  Start-Sleep -Milliseconds 1200
  # overflow flyout is a separate top-level window
  foreach ($h in [QAInput]::TopWindows()) {
    $cls = ''
    $t = [QAInput]::Title($h)
    try {
      $root2 = [System.Windows.Automation.AutomationElement]::FromHandle($h)
      $els = $root2.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)
      foreach ($e in $els) {
        try {
          if ($e.Current.Name -match 'VietKB') {
            W "overflow element name=[$($e.Current.Name)] type=$($e.Current.ControlType.ProgrammaticName) rect=$($e.Current.BoundingRectangle) win=[$t]"
            $vkbBtn = $e
            break
          }
        } catch {}
      }
    } catch {}
    if ($vkbBtn) { break }
  }
}
if (-not $vkbBtn) { W 'tray_icon=NOT_FOUND'; W 'TRAY-DONE'; exit 0 }

W "icon_state_text=[$($vkbBtn.Current.Name)]"
$before = Observe 'before tray toggle'
# right click
$r = $vkbBtn.Current.BoundingRectangle
[void][QAInput]::SetCursorPos([int]($r.X + $r.Width / 2), [int]($r.Y + $r.Height / 2))
Start-Sleep -Milliseconds 150
[QAInput]::mouse_event(0x0008, 0, 0, 0, [IntPtr]::Zero)
[QAInput]::mouse_event(0x0010, 0, 0, 0, [IntPtr]::Zero)
Start-Sleep -Milliseconds 1200
# find the context menu: look for a window with a menu item naming the toggle
$menuItem = $null; $menuWin = $null
foreach ($h in [QAInput]::TopWindows()) {
  try {
    $root3 = [System.Windows.Automation.AutomationElement]::FromHandle($h)
    $els = $root3.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)
    foreach ($e in $els) {
      try {
        $n = $e.Current.Name
        if ($n -match '^(Bật|Tắt).{0,20}(tiếng Việt)?$' -and $e.Current.ControlType.ProgrammaticName -match 'MenuItem|Button|Text') {
          W "menu_candidate win=[$([QAInput]::Title($h))] name=[$n] type=$($e.Current.ControlType.ProgrammaticName)"
          if (-not $menuItem) { $menuItem = $e; $menuWin = $h }
        }
      } catch {}
    }
  } catch {}
}
if ($menuItem) {
  W "menu_open_title=[$([QAInput]::Title($menuWin))] chosen=[$($menuItem.Current.Name)]"
  try { $menuItem.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke() } catch { [void](Click-Element $menuItem) }
  Start-Sleep -Milliseconds 1200
  $after = Observe 'after tray toggle'
  W "tray_route_flipped=$($before -ne $after)"
  W "icon_state_text_after=[$((Find-ElementByName ([QAInput]::FindWindowW('Shell_TrayWnd', $null)) 'VietKB').Current.Name)]"
  # toggle back
  [void]$vkbBtn.Current.BoundingRectangle
} else {
  W 'menu=NOT_FOUND'
  [QAInput]::Combo(0x1B, $false, $false, $false)
}
W 'TRAY-DONE'
