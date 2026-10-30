. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
$w = Get-TopWindow 'scratch-np.txt'; [void](Repair-Notepad 'scratch-np.txt'); $hwnd = $w.Hwnd
function RunSeq($delay) {
  [void][QAInput]::Focus($hwnd); Start-Sleep -Milliseconds 150
  $el = Get-EditElement $hwnd; [void](Click-Element $el)
  Clear-TextArea $hwnd; Start-Sleep -Milliseconds 250
  [QAInput]::TypeAscii('ba6t5/ta6t5', $delay)
  Start-Sleep -Milliseconds 900
  $u = Get-UiaText $hwnd
  Write-Host ("notepad delay={0,4} -> [{1}]" -f $delay, ([string]$u.Text).TrimEnd("`r", "`n"))
}
foreach ($d in 120, 250, 400, 700) { RunSeq $d }
# also: type word, pause, type separator, pause, type word (human-like)
[void][QAInput]::Focus($hwnd); Start-Sleep -Milliseconds 150; Clear-TextArea $hwnd; Start-Sleep -Milliseconds 250
[QAInput]::TypeAscii('ba6t5', 120); Start-Sleep -Milliseconds 700
[QAInput]::TypeAscii('/', 120); Start-Sleep -Milliseconds 700
[QAInput]::TypeAscii('ta6t5', 120); Start-Sleep -Milliseconds 900
$u = Get-UiaText $hwnd
Write-Host "notepad segmented -> [$(([string]$u.Text).TrimEnd("`r","`n"))]"
