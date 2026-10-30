. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
$w = Get-TopWindow 'scratch-np.txt'; [void](Repair-Notepad 'scratch-np.txt'); $hwnd = $w.Hwnd
$tests = @('ta6t5', 'tat5', 'ba6t5', 'ta6t', 'Nguye6n4', 'Thu Ha2', 'm01 ', 'tie6ng1', 'ba6t5/ta6t5')
foreach ($k in $tests) {
  [void][QAInput]::Focus($hwnd); Start-Sleep -Milliseconds 120
  $el = Get-EditElement $hwnd; [void](Click-Element $el)
  Clear-TextArea $hwnd; Start-Sleep -Milliseconds 250
  [QAInput]::TypeAscii($k, 120)
  Start-Sleep -Milliseconds 900
  $u = Get-UiaText $hwnd
  Write-Host ("keys=[{0,-14}] -> [{1}]" -f $k, ([string]$u.Text).TrimEnd("`r", "`n"))
}
