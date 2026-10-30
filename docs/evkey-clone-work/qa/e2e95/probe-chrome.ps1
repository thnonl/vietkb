. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
# stop only OUR isolated-profile chrome
$mine = Get-CimInstance Win32_Process -Filter "Name='chrome.exe'" | Where-Object { $_.CommandLine -like '*vkbe2e-chrome*' }
foreach ($m in $mine) { if ($m.ParentProcessId -eq 0 -or $m.CommandLine -notlike '*--type=*') { Stop-Process -Id $m.ProcessId -Force -ErrorAction SilentlyContinue } }
Start-Sleep -Milliseconds 800
$prof = Join-Path $env:TEMP 'vkbe2e-chrome2'
$url = 'file:///E:/vietkb/docs/evkey-clone-work/qa/e2e95/probe.html'
Start-Process 'C:\Program Files\Google\Chrome\Application\chrome.exe' -ArgumentList @(
  "--user-data-dir=$prof", '--no-first-run', '--no-default-browser-check',
  '--force-renderer-accessibility', "--app=$url") | Out-Null
$w = $null
for ($i = 0; $i -lt 40; $i++) {
  Start-Sleep -Milliseconds 500
  $w = Get-TopWindow 'VietKB E2E probe'
  if ($w) { break }
}
if (-not $w) { Write-Host 'NO_WINDOW'; exit 1 }
Write-Host "win=$($w.Hwnd) title=[$($w.Title)] pid=$($w.Pid)"
[void][QAInput]::Focus($w.Hwnd); Start-Sleep -Milliseconds 600
$el = Get-EditElement $w.Hwnd
if ($el) { Write-Host "edit=$($el.Current.ControlType.ProgrammaticName) rect=$($el.Current.BoundingRectangle)"; [void](Click-Element $el) }
else { Write-Host 'no edit element'; Click-Center $w.Hwnd }
Start-Sleep -Milliseconds 400
$u = Get-UiaText $w.Hwnd
Write-Host "before via=$($u.Via) text=[$($u.Text)] fg=$([QAInput]::GetForegroundWindow()) want=$($w.Hwnd)"
[QAInput]::TypeAscii('ba6t5/ta6t5', 120)
Start-Sleep -Milliseconds 800
$u = Get-UiaText $w.Hwnd
Write-Host "after1 via=$($u.Via) text=[$(([string]$u.Text).TrimEnd("`r","`n"))]"
Clear-TextArea $w.Hwnd
Start-Sleep -Milliseconds 300
[QAInput]::TypeAscii('Nguye6n4 Thu Ha2', 120)
Start-Sleep -Milliseconds 800
$u = Get-UiaText $w.Hwnd
Write-Host "after2 via=$($u.Via) text=[$(([string]$u.Text).TrimEnd("`r","`n"))]"
