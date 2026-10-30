. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
$dir = 'E:\vietkb\docs\evkey-clone-work\qa\e2e95'
$out = Join-Path $dir 'ui-probe.txt'
Set-Content -Encoding UTF8 $out 'UI-PROBE'
function W($s) { Add-Content -Encoding UTF8 $out $s; Write-Host $s }

$app = Get-VkbProcess
if (-not $app) { W 'app=NO_PROCESS'; exit 1 }
W "app pid=$($app.Id)"
# open settings: Ctrl+Alt+S
$fg = Get-TopWindow 'scratch-np.txt'
if ($fg) { [void][QAInput]::Focus($fg.Hwnd); Start-Sleep -Milliseconds 200 }
[QAInput]::Combo(0x53, $false, $true, $true)
Start-Sleep -Milliseconds 1500
$w = $null
for ($i = 0; $i -lt 20; $i++) {
  foreach ($h in [QAInput]::TopWindows()) {
    $pn = (Get-Process -Id ([QAInput]::Pid($h)) -ErrorAction SilentlyContinue).ProcessName
    if ($pn -eq 'vietkb' -and [QAInput]::Title($h)) { $w = [pscustomobject]@{ Hwnd = $h; Title = [QAInput]::Title($h) }; break }
  }
  if ($w) { break }
  Start-Sleep -Milliseconds 400
}
if (-not $w) { W 'settings=NO_WINDOW'; exit 1 }
W "settings hwnd=$($w.Hwnd) title=[$($w.Title)]"
$root = [System.Windows.Automation.AutomationElement]::FromHandle($w.Hwnd)
$all = $root.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)
W "descendants=$($all.Count)"
foreach ($el in $all) {
  try {
    $ct = $el.Current.ControlType.ProgrammaticName
    $n = $el.Current.Name
    $aid = $el.Current.AutomationId
    $r = $el.Current.BoundingRectangle
    if ($n -or $ct -match 'Tab|Button|CheckBox|Edit|Document') {
      W ("{0,-24} name=[{1}] auto=[{2}] rect={3},{4},{5},{6}" -f $ct, $n, $aid, [int]$r.X, [int]$r.Y, [int]$r.Width, [int]$r.Height)
    }
  } catch {}
}
W 'UI-PROBE-DONE'
