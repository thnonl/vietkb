. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
$w = Get-TopWindow 'scratch-np.txt'
Write-Host "notepad=$($w.Hwnd) [$($w.Title)]"
$root = [System.Windows.Automation.AutomationElement]::FromHandle($w.Hwnd)
$kids = $root.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)
Write-Host "descendants=$($kids.Count)"
foreach ($el in $kids) {
  try {
    $ct = $el.Current.ControlType.ProgrammaticName
    $n = $el.Current.Name
    if ($ct -match 'Button|Window|Text|Document|Edit' -and ($n -or $ct -match 'Window|Document|Edit')) {
      Write-Host ("{0,-24} name=[{1}] auto=[{2}] rect={3}" -f $ct, $n, $el.Current.AutomationId, $el.Current.BoundingRectangle)
    }
  } catch {}
}
