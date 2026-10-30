. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
$dir = 'E:\vietkb\docs\evkey-clone-work\qa\e2e95'
$out = Join-Path $dir 'ui-results.txt'
function W($s) { Add-Content -Encoding UTF8 $out $s; Write-Host $s }
Add-Type -AssemblyName System.Drawing

$ch = Get-TopWindow 'VietKB E2E probe'
[void][QAInput]::Focus($ch.Hwnd); Start-Sleep -Milliseconds 300
[QAInput]::Combo(0x53, $false, $true, $true); Start-Sleep -Milliseconds 1500
$main = Get-TopWindow 'Cài đặt VietKB'
if (-not $main) { W 'about: main window missing'; exit 1 }
[void][QAInput]::Focus($main.Hwnd); Start-Sleep -Milliseconds 400
$adv = Find-ElementByName $main.Hwnd 'Nâng cao…' 'Button'
$adv.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
Start-Sleep -Milliseconds 1200
$advWin = Get-TopWindow 'Nâng cao'
if (-not $advWin) { W 'about: advanced missing'; exit 1 }
$gi = Find-ElementByName $advWin.Hwnd 'Giới thiệu…' 'Button'
if (-not $gi) { $gi = Find-ElementByName $advWin.Hwnd 'Giới thiệu' 'Button' }
if (-not $gi) { W 'about: button not found'; exit 1 }
W "about button=[$($gi.Current.Name)]"
$gi.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
Start-Sleep -Milliseconds 1500
$aboutWin = $null
foreach ($h in [QAInput]::TopWindows()) {
  $pn = (Get-Process -Id ([QAInput]::Pid($h)) -ErrorAction SilentlyContinue).ProcessName
  if ($pn -eq 'vietkb' -and [QAInput]::Title($h) -match 'Giới thiệu|VietKB') { W "vkb window title=[$([QAInput]::Title($h))]"; if ([QAInput]::Title($h) -notmatch 'Cài đặt|Nâng cao') { $aboutWin = $h } }
}
if ($aboutWin) {
  $r = New-Object QAInput+RECT; [void][QAInput]::GetWindowRect($aboutWin, [ref]$r)
  $bmp = New-Object System.Drawing.Bitmap ($r.R - $r.L), ($r.B - $r.T)
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.CopyFromScreen($r.L, $r.T, 0, 0, $bmp.Size)
  $bmp.Save((Join-Path $dir 'ui-5-about.png')); $g.Dispose(); $bmp.Dispose()
  $txt = (Get-UiaText $aboutWin).Text
  W "about_text=[$($txt -replace "`r?`n", ' | ')]"
  [void][QAInput]::Focus($aboutWin); Start-Sleep -Milliseconds 300
  [QAInput]::Combo(0x1B, $false, $false, $false); Start-Sleep -Milliseconds 600
  $still = $false
  foreach ($h in [QAInput]::TopWindows()) { if ([QAInput]::Title($h) -match 'Giới thiệu') { $still = $true } }
  W "about_page_closed_via_esc=$(-not $still)"
} else { W 'about window not found among vietkb windows' }
# close the two settings windows we opened
$advWin = Get-TopWindow 'Nâng cao'
if ($advWin) { [void][QAInput]::Focus($advWin.Hwnd); [QAInput]::Combo(0x1B, $false, $false, $false); Start-Sleep -Milliseconds 400 }
$main = Get-TopWindow 'Cài đặt VietKB'
if ($main) { [void][QAInput]::Focus($main.Hwnd); [QAInput]::Combo(0x1B, $false, $false, $false); Start-Sleep -Milliseconds 400 }
W 'ABOUT-DONE'
