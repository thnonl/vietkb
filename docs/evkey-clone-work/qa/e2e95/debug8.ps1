. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
$w = Get-TopWindow 'scratch-np.txt'; [void](Repair-Notepad 'scratch-np.txt'); $hwnd = $w.Hwnd
[void][QAInput]::Focus($hwnd); Start-Sleep -Milliseconds 150
$el = Get-EditElement $hwnd; [void](Click-Element $el)
Clear-TextArea $hwnd; Start-Sleep -Milliseconds 250
[QAInput]::TypeAscii('ba6t5/ta6t5', 150)
Start-Sleep -Milliseconds 900
$u = Get-UiaText $hwnd
Write-Host "RESULT=[$(([string]$u.Text).TrimEnd("`r","`n"))]"
$k = Join-Path $env:APPDATA 'vietkb\keys.log'
if (Test-Path $k) { Write-Host '--- keys.log ---'; Get-Content $k -Tail 40 } else { Write-Host 'no keys.log' }
