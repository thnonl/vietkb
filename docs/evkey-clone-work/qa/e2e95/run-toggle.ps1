. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
$dir = 'E:\vietkb\docs\evkey-clone-work\qa\e2e95'
$out = Join-Path $dir 'toggle-results.txt'
Set-Content -Encoding UTF8 $out 'TOGGLE-E2E'
function W($s) { Add-Content -Encoding UTF8 $out $s; Write-Host $s }
Add-Type -AssemblyName System.Drawing

function SoundErrors {
  $p = Join-Path $dir 'app.log'
  if (-not (Test-Path $p)) { return -1 }
  return @(Select-String -Path $p -Pattern 'toggle sound could not be played' -AllMatches).Count
}

$np = Get-TopWindow 'scratch-np.txt'
if (-not $np) { W 'notepad=NO_WINDOW'; exit 1 }
[void](Repair-Notepad 'scratch-np.txt')
$hwnd = $np.Hwnd

function TypeCheck($label) {
  [void][QAInput]::Focus($hwnd); Start-Sleep -Milliseconds 200
  Clear-TextArea $hwnd
  [QAInput]::TypeAscii('ba6t5', 120)
  Start-Sleep -Milliseconds 700
  $u = Get-UiaText $hwnd
  $t = ([string]$u.Text).TrimEnd("`r", "`n")
  W ("{0,-34} typed=[ba6t5] actual=[{1}] transforms={2}" -f $label, $t, ($t -match 'ậ'))
  return $t
}

W "sound_errors_at_start=$(SoundErrors)"
W "--- route: F2 (disable) / F1 (enable) ---"
[QAInput]::Combo(0x71, $false, $false, $false); Start-Sleep -Milliseconds 500
$t = TypeCheck 'after F2 (expect literal)'
[QAInput]::Combo(0x70, $false, $false, $false); Start-Sleep -Milliseconds 500
$t = TypeCheck 'after F1 (expect bật)'
W "sound_errors_after_f1f2=$(SoundErrors)"

W "--- route: Alt+Shift ---"
[QAInput]::Combo(0x12, $false, $false, $false)   # tap Alt only, sanity: must not toggle
Start-Sleep -Milliseconds 300
$t = TypeCheck 'after bare Alt (expect bật)'
# Alt+Shift tap: hold alt, press+release shift, release alt
[QAInput]::Combo(0x10, $false, $false, $true)
Start-Sleep -Milliseconds 500
$t1 = TypeCheck 'after Alt+Shift #1'
[QAInput]::Combo(0x10, $false, $false, $true)
Start-Sleep -Milliseconds 500
$t2 = TypeCheck 'after Alt+Shift #2'
W "alt_shift_flipped=$([bool](($t1 -match 'ậ') -xor ($t2 -match 'ậ')))"
W "sound_errors_after_altshift=$(SoundErrors)"
# leave Vietnamese enabled
if ($t2 -notmatch 'ậ') { [QAInput]::Combo(0x70, $false, $false, $false); Start-Sleep -Milliseconds 400 }

W "--- route: UI switch ---"
[QAInput]::Combo(0x53, $false, $true, $true)   # Ctrl+Alt+S
Start-Sleep -Milliseconds 1200
$sw = $null
for ($i = 0; $i -lt 10; $i++) {
  $w = Get-TopWindow 'Cài đặt VietKB'
  if ($w) { $sw = Find-ElementByName $w.Hwnd 'Bật tiếng Việt' 'Button'; if ($sw) { break } }
  Start-Sleep -Milliseconds 400
}
if (-not $sw) { W 'ui_switch=NOT_FOUND' } else {
  W ("ui_switch_found name=[{0}] rect={1}" -f $sw.Current.Name, $sw.Current.BoundingRectangle)
  try {
    $tp = $sw.GetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern)
    W "ui_switch_pattern=Toggle state=$($tp.Current.ToggleState)"
    $tp.Toggle(); Start-Sleep -Milliseconds 700
    W "ui_switch_state_after=$($tp.Current.ToggleState)"
  } catch {
    [void](Click-Element $sw); Start-Sleep -Milliseconds 700
    W 'ui_switch clicked (no TogglePattern)'
  }
  $st = $sw.GetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern).Current.ToggleState
  W "ui_switch_state_now=$st"
  $t = TypeCheck 'after UI switch #1'
  try { $sw.GetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern).Toggle(); Start-Sleep -Milliseconds 600 } catch {}
  $t = TypeCheck 'after UI switch #2'
  W "sound_errors_after_ui=$(SoundErrors)"
}
W 'TOGGLE-DONE'
