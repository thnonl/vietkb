. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
$vs = Get-TopWindow 'scratch-vsc.txt'
if (-not $vs) { Write-Host 'closed'; exit 0 }
[void][QAInput]::Focus($vs.Hwnd); Start-Sleep -Milliseconds 400
$hit = $null
foreach ($e in [System.Windows.Automation.AutomationElement]::FromHandle($vs.Hwnd).FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)) {
  try {
    $n = $e.Current.Name
    if ($n -eq "Don't Save" -or $n -eq 'Không lưu') { $hit = $e; Write-Host "found [$n] rect=$($e.Current.BoundingRectangle)" }
  } catch {}
}
if ($hit) {
  [void](Click-Element $hit)
  Start-Sleep -Milliseconds 1500
} else { Write-Host 'dont-save button not exposed' }
Write-Host "vscode_gone=$(-not [bool](Get-TopWindow 'scratch-vsc.txt'))"
