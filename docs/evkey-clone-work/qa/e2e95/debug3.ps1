. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
$dir = 'E:\vietkb\docs\evkey-clone-work\qa\e2e95'
$w = Get-TopWindow 'scratch-np.txt'
[void](Repair-Notepad 'scratch-np.txt')
$hwnd = $w.Hwnd
function TType($delay, $keys) {
  [void][QAInput]::Focus($hwnd); Start-Sleep -Milliseconds 150
  Clear-TextArea $hwnd
  [QAInput]::TypeAscii($keys, $delay)
  Start-Sleep -Milliseconds 600
  $u = Get-UiaText $hwnd
  $t = ([string]$u.Text).TrimEnd("`r", "`n")
  Write-Host ("delay={0,4} keys=[{1}] -> [{2}]" -f $delay, $keys, $t)
}
foreach ($d in 40, 60, 90, 120, 160) {
  TType $d 'ba6t5/ta6t5'
  TType $d 'ba6t5 ta6t5'
  TType $d 'm01 '
}

