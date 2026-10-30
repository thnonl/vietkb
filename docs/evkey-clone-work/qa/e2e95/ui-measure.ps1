. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
Add-Type -AssemblyName System.Drawing
$w = Get-TopWindow 'Cài đặt VietKB'
Write-Host "hwnd=$($w.Hwnd)"
$r = New-Object QAInput+RECT; [void][QAInput]::GetWindowRect($w.Hwnd, [ref]$r)
$c = New-Object QAInput+RECT; [void][QAInput]::GetClientRect($w.Hwnd, [ref]$c)
$dpi = [QAInput]::GetDpiForWindow($w.Hwnd)
Write-Host ("outer={0}x{1} at {2},{3} client={4}x{5} dpi={6} scale={7}" -f ($r.R-$r.L), ($r.B-$r.T), $r.L, $r.T, $c.R, $c.B, $dpi, ($dpi/96))
[void][QAInput]::Focus($w.Hwnd); Start-Sleep -Milliseconds 400
$bmp = New-Object System.Drawing.Bitmap ($r.R-$r.L), ($r.B-$r.T)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.CopyFromScreen($r.L, $r.T, 0, 0, $bmp.Size)
$bmp.Save('E:\vietkb\docs\evkey-clone-work\qa\e2e95\ui-main.png')
$g.Dispose(); $bmp.Dispose()
Write-Host 'shot saved'
