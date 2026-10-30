. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
$w = Get-TopWindow 'scratch-np.txt'; [void](Repair-Notepad 'scratch-np.txt'); $hwnd = $w.Hwnd
function Show($label) {
  $u = Get-UiaText $hwnd
  Write-Host ("{0,-28} -> [{1}]" -f $label, ([string]$u.Text).TrimEnd("`r", "`n"))
}
function Reset2 { [void][QAInput]::Focus($hwnd); Start-Sleep -Milliseconds 150; Clear-TextArea $hwnd; Start-Sleep -Milliseconds 200 }

Reset2; [QAInput]::TypeAscii('ta6t5', 90); Start-Sleep -Milliseconds 600; Show 'ta6t5 @90'
Reset2; [QAInput]::TypeAscii('ba6t5', 90); Start-Sleep -Milliseconds 1200; [QAInput]::TypeAscii('/ta6t5', 90); Start-Sleep -Milliseconds 600; Show 'ba6t5 +1200ms /ta6t5 @90'
Reset2; [QAInput]::TypeAscii('ba6t5/', 300); Start-Sleep -Milliseconds 600; [QAInput]::TypeAscii('ta6t5', 300); Start-Sleep -Milliseconds 600; Show 'ba6t5/ @300 pause ta6t5 @300'
Reset2; [QAInput]::TypeAscii('ba6t5/ta6t5', 300); Start-Sleep -Milliseconds 600; Show 'ba6t5/ta6t5 @300'
Reset2; [QAInput]::TypeAscii('ba6t5', 90); Start-Sleep -Milliseconds 400; [QAInput]::TypeAscii('/', 90); Start-Sleep -Milliseconds 400; [QAInput]::TypeAscii('ta6t5', 90); Start-Sleep -Milliseconds 600; Show 'ba6t5 p400 / p400 ta6t5 @90'
Reset2; [QAInput]::TypeAscii('m01 ', 90); Start-Sleep -Milliseconds 700; Show 'm01 space'
