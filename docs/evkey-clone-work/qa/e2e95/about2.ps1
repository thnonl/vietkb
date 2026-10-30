. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
Add-Type -AssemblyName System.Drawing
$dir = 'E:\vietkb\docs\evkey-clone-work\qa\e2e95'
function W($s) { Write-Host $s; Add-Content -Encoding UTF8 (Join-Path $dir 'ui-results.txt') $s }

$w = Get-TopWindow 'Giới thiệu'
if (-not $w) { W 'about: no window'; exit 0 }
[void][QAInput]::Focus($w.Hwnd); Start-Sleep -Milliseconds 1500
$r = New-Object QAInput+RECT; [void][QAInput]::GetWindowRect($w.Hwnd, [ref]$r)
$c = New-Object QAInput+RECT; [void][QAInput]::GetClientRect($w.Hwnd, [ref]$c)
W "about outer=$($r.R-$r.L)x$($r.B-$r.T) client=$($c.R)x$($c.B) dpi=$([QAInput]::GetDpiForWindow($w.Hwnd))"
$root = [System.Windows.Automation.AutomationElement]::FromHandle($w.Hwnd)
foreach ($e in $root.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)) {
  try { $n = $e.Current.Name; if ($n) { W ("about_el {0} name=[{1}] rect={2}" -f $e.Current.ControlType.ProgrammaticName, $n, $e.Current.BoundingRectangle) } } catch {}
}
$bmp = New-Object System.Drawing.Bitmap ($r.R - $r.L), ($r.B - $r.T)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.CopyFromScreen($r.L, $r.T, 0, 0, $bmp.Size)
$bmp.Save((Join-Path $dir 'ui-6-about2.png')); $g.Dispose(); $bmp.Dispose()
# close with Alt+F4
[QAInput]::Combo(0x73, $false, $false, $true)
Start-Sleep -Milliseconds 1200
W "about_closed_after_altF4=$(-not [bool](Get-TopWindow 'Giới thiệu'))"
