. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
$w = Get-TopWindow 'scratch-np.txt'
Write-Host "win=$($w.Hwnd) [$($w.Title)]"
[void](Repair-Notepad 'scratch-np.txt')
$hwnd = $w.Hwnd
$fg = [QAInput]::GetForegroundWindow()
Write-Host "fg=$fg match=$($fg -eq $hwnd)"
$u = Get-UiaText $hwnd
Write-Host "before via=$($u.Via) text=[$($u.Text)]"
$el = Get-EditElement $hwnd
Write-Host "edit=$($el.Current.ControlType.ProgrammaticName) rect=$($el.Current.BoundingRectangle)"
[void](Click-Element $el)
Write-Host "fg_after_click=$([QAInput]::GetForegroundWindow())"
[QAInput]::TypeAscii('ba6t5/ta6t5', 90)
Start-Sleep -Milliseconds 700
$u = Get-UiaText $hwnd
Write-Host "after via=$($u.Via) text=[$($u.Text)]"
$c = Get-ClipboardText $hwnd
Write-Host "clip=[$($c.Text)]"
$modal = Find-ElementByName $hwnd 'Keep unsaved changes?' 'Window'
Write-Host "modal_now=$([bool]$modal)"
