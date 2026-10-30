. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
$dir = 'E:\vietkb\docs\evkey-clone-work\qa\e2e95'
$out = Join-Path $dir 'toggle-results.txt'
Set-Content -Encoding UTF8 $out 'TOGGLE-E2E (observer: Chrome textarea)'
function W($s) { Add-Content -Encoding UTF8 $out $s; Write-Host $s }
function SoundErrors {
  $p = Join-Path $dir 'app.log'
  if (-not (Test-Path $p)) { return -1 }
  return @(Select-String -Path $p -Pattern 'toggle sound could not be played' -AllMatches).Count
}

$ch = Get-TopWindow 'VietKB E2E probe'
if (-not $ch) { W 'chrome=NO_WINDOW'; exit 1 }
$hwnd = $ch.Hwnd

function Observe($label) {
  $ok = $false
  for ($i = 0; $i -lt 4; $i++) {
    [void][QAInput]::Focus($hwnd); Start-Sleep -Milliseconds 250
    $el = Get-EditElement $hwnd
    if ($el) { [void](Click-Element $el) }
    if ([QAInput]::GetForegroundWindow() -eq $hwnd) { $ok = $true; break }
    Start-Sleep -Milliseconds 300
  }
  Clear-TextArea $hwnd
  [QAInput]::TypeAscii('ba6t5', 120)
  Start-Sleep -Milliseconds 600
  $u = Get-UiaText $hwnd
  $t = ([string]$u.Text).TrimEnd("`r", "`n")
  $state = if ($t -match 'ậ') { 'ON' } elseif ($t -ceq 'ba6t5') { 'OFF' } else { "OTHER($t)" }
  W ("{0,-36} focused={1} actual=[{2}] state={3}" -f $label, $ok, $t, $state)
  return $state
}

W "sound_error_lines_before=$(SoundErrors)"
W "initial_state=$(Observe 'initial')"

W '--- route A: hotkey F2 / F1 ---'
[QAInput]::Combo(0x71, $false, $false, $false); Start-Sleep -Milliseconds 600
$a1 = Observe 'after F2'
[QAInput]::Combo(0x70, $false, $false, $false); Start-Sleep -Milliseconds 600
$a2 = Observe 'after F1'
W "route_F2F1_ok=$($a1 -eq 'OFF' -and $a2 -eq 'ON')"

W '--- route B: Alt+Shift (tap) ---'
[QAInput]::Combo(0x10, $false, $false, $true); Start-Sleep -Milliseconds 600
$b1 = Observe 'after Alt+Shift #1'
[QAInput]::Combo(0x10, $false, $false, $true); Start-Sleep -Milliseconds 600
$b2 = Observe 'after Alt+Shift #2'
W "route_AltShift_ok=$($b1 -ne $b2 -and $b2 -eq 'ON')"

W '--- route C: UI switch in the settings window ---'
[QAInput]::Combo(0x53, $false, $true, $true); Start-Sleep -Milliseconds 1400
$sw = $null; $win = $null
for ($i = 0; $i -lt 10; $i++) {
  $win = Get-TopWindow 'Cài đặt VietKB'
  if ($win) { $sw = Find-ElementByName $win.Hwnd 'Bật tiếng Việt' 'Button'; if ($sw) { break } }
  Start-Sleep -Milliseconds 400
}
if (-not $sw) { W 'ui_switch=NOT_FOUND' } else {
  $tp = $sw.GetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern)
  W "ui_switch_state_before=$($tp.Current.ToggleState)"
  $tp.Toggle(); Start-Sleep -Milliseconds 800
  W "ui_switch_state_after=$($tp.Current.ToggleState)"
  $c1 = Observe 'after UI switch #1'
  $tp.Toggle(); Start-Sleep -Milliseconds 800
  W "ui_switch_state_now=$($tp.Current.ToggleState)"
  $c2 = Observe 'after UI switch #2'
  W "route_UI_ok=$($c1 -eq 'OFF' -and $c2 -eq 'ON')"
  # window geometry for section C
  $r = New-Object QAInput+RECT; [void][QAInput]::GetWindowRect($win.Hwnd, [ref]$r)
  $cr = New-Object QAInput+RECT; [void][QAInput]::GetClientRect($win.Hwnd, [ref]$cr)
  W ("settings_window outer={0}x{1} client={2}x{3} dpi={4}" -f ($r.R-$r.L), ($r.B-$r.T), $cr.R, $cr.B, [QAInput]::GetDpiForWindow($win.Hwnd))
  # buttons present
  foreach ($n in 'Phím tắt…', 'Nâng cao…', 'Bảng gõ tắt…', 'Giới thiệu') {
    $e = Find-ElementByName $win.Hwnd $n
    W ("button[{0}] found={1}" -f $n, [bool]$e)
  }
}
W "sound_error_lines_after=$(SoundErrors)"
W 'TOGGLE-DONE'
