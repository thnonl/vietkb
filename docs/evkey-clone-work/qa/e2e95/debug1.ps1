. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
$w = Get-TopWindow 'scratch-np.txt'
Write-Host "window=$($w.Hwnd) title=[$($w.Title)] pid=$($w.Pid)"
$fg0 = [QAInput]::GetForegroundWindow()
Write-Host "fg_before=$fg0"
$ok = [QAInput]::Focus($w.Hwnd)
Write-Host "focus=$ok fg_after=$([QAInput]::GetForegroundWindow())"
Start-Sleep -Milliseconds 600
$el = Get-EditElement $w.Hwnd
Write-Host "edit_el=$($el.Current.ControlType.ProgrammaticName) rect=$($el.Current.BoundingRectangle)"
$clicked = Click-Element $el
Write-Host "clicked=$clicked fg=$([QAInput]::GetForegroundWindow())"
[QAInput]::TypeAscii('xyz', 80)
Start-Sleep -Milliseconds 600
$u = Get-UiaText $w.Hwnd
Write-Host "uia via=$($u.Via) text=[$($u.Text)]"
$c = Get-ClipboardText $w.Hwnd
Write-Host "clip=[$($c.Text)]"
Get-Content 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\app.log' -Tail 12
