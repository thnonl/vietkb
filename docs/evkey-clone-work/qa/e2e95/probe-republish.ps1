# Probe: does the hotkey matcher survive a config republish (F1 toggle saves config)?
param([string]$App = 'notepad')
. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
$dir = 'E:\vietkb\docs\evkey-clone-work\qa\e2e95'
Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Milliseconds 500
$f = Join-Path $dir 'scratch-probe.txt'
Remove-Item $f -ErrorAction SilentlyContinue; New-Item -ItemType File -Path $f | Out-Null
Start-Process notepad.exe -ArgumentList $f | Out-Null
$w = Wait-Window 'scratch-probe.txt' 30
if (-not $w) { 'SETUP-FAIL'; exit 1 }
$hwnd = $w.Hwnd
Start-Sleep -Seconds 2
[void][QAInput]::Focus($hwnd)
[void](Repair-Notepad 'scratch-probe.txt')
[void][QAInput]::Focus($hwnd)
Start-Sleep -Milliseconds 400
$el = Get-EditElement $hwnd
if ($el) { [void](Click-Element $el) }
Start-Sleep -Milliseconds 300

function TypeAndRead {
  param([string]$t)
  [void][QAInput]::Focus($hwnd)
  [QAInput]::Combo(0x41, $false, $true, $false)
  [QAInput]::Combo(0x2E, $false, $false, $false)
  Start-Sleep -Milliseconds 80
  [QAInput]::TypeAscii($t, 55)
  Start-Sleep -Milliseconds 350
  $r = Read-Text $hwnd
  return ([string]$r.Text).TrimEnd("`r", "`n")
}

$log = "$env:APPDATA\vietkb\keys.log"
function HK { (Select-String -Path $log -Pattern 'stage=hotkey' | Measure-Object).Count }

$a = TypeAndRead 'ba6t5';      "step1 pre-F1        text=[$a] hotkeys=$(HK)"
[QAInput]::Combo(0x70, $false, $false, $false); Start-Sleep -Milliseconds 500
$b = TypeAndRead 'ba6t5';      "step2 after F1      text=[$b] hotkeys=$(HK)"
[QAInput]::Combo(0x70, $false, $false, $false); Start-Sleep -Milliseconds 500
$c = TypeAndRead 'ba6t5';      "step3 after F1 #2   text=[$c] hotkeys=$(HK)"
$z = ((Select-String -Path $log -Pattern 'note config-applied' | Measure-Object).Count); "config-applied notes=$z"
Select-String -Path $log -Pattern 'stage=hotkey|note config-applied' | Select-Object -Last 4 | ForEach-Object { "  " + $_.Line }
