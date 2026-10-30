. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
$w = Get-TopWindow 'scratch-np.txt'; [void](Repair-Notepad 'scratch-np.txt'); $hwnd = $w.Hwnd
[void][QAInput]::Focus($hwnd); Start-Sleep -Milliseconds 200
$el = Get-EditElement $hwnd; [void](Click-Element $el)
Write-Host "fg=$([QAInput]::GetForegroundWindow()) hwnd=$hwnd"
[QAInput]::TypeAscii('hello', 100)
Start-Sleep -Milliseconds 800
$u = Get-UiaText $hwnd
Write-Host "uia=[$($u.Text)]"
$c = Get-ClipboardText $hwnd
Write-Host "clip=[$($c.Text)]"
# now clear and type again
Clear-TextArea $hwnd
Start-Sleep -Milliseconds 300
$c = Get-ClipboardText $hwnd
Write-Host "after_clear_clip=[$($c.Text)]"
[QAInput]::TypeAscii('world ta6t5', 100)
Start-Sleep -Milliseconds 800
$u = Get-UiaText $hwnd
Write-Host "uia2=[$($u.Text)]"
$c = Get-ClipboardText $hwnd
Write-Host "clip2=[$($c.Text)]"
