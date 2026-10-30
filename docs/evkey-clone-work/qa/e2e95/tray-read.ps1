. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
$tray = [QAInput]::FindWindowW('Shell_TrayWnd', $null)
Write-Host "Shell_TrayWnd=$tray"
$root = [System.Windows.Automation.AutomationElement]::FromHandle($tray)
$els = $root.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)
foreach ($e in $els) {
  try {
    $n = $e.Current.Name
    if ($n -match 'VietKB|vietkb') {
      Write-Host ("tray {0} name=[{1}] rect={2}" -f $e.Current.ControlType.ProgrammaticName, $n, $e.Current.BoundingRectangle)
    }
  } catch {}
}
# also any top-level window owned by vietkb that exposes a menu
foreach ($h in [QAInput]::TopWindows()) {
  $p = [QAInput]::Pid($h)
  $pn = (Get-Process -Id $p -ErrorAction SilentlyContinue).ProcessName
  if ($pn -eq 'vietkb') { Write-Host "vkb window $h class=[$(([QAInput]::Title($h)))]" }
}
