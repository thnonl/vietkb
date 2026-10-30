param([ValidateSet('notepad','chrome','vscode')][string]$App)
. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
$dir = 'E:\vietkb\docs\evkey-clone-work\qa\e2e95'
$tsv = Join-Path $dir "typing-$App.tsv"
Set-Content -Encoding UTF8 $tsv "app`tid`tkeys`texpected`tactual`tvia`tfocus`tverdict"
$global:KeyDelay = 55
function Row($id, $keys, $expected, $actual, $via, $focused, $verdict) {
  $exp = $expected.Replace("`t", '\t').Replace("`r`n", '\n').Replace("`n", '\n').Replace("`r", '\n')
  $act = $actual.Replace("`t", '\t').Replace("`r`n", '\n').Replace("`n", '\n').Replace("`r", '\n')
  Add-Content -Encoding UTF8 $tsv ("{0}`t{1}`t{2}`t{3}`t{4}`t{5}`t{6}`t{7}" -f $App, $id, $keys, $exp, $act, $via, $focused, $verdict)
  Write-Host ("{0,-22} keys=[{1}] exp=[{2}] act=[{3}] {4} fg={5}" -f $id, $keys, $exp, $act, $verdict, $focused)
}

# ---------- app setup ----------
$hwnd = [IntPtr]::Zero
$needWarmup = $true
switch ($App) {
  'notepad' {
    Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force
    Start-Sleep -Milliseconds 600
    $f = Join-Path $dir 'scratch-np.txt'
    Remove-Item $f -ErrorAction SilentlyContinue; New-Item -ItemType File -Path $f | Out-Null
    Start-Process notepad.exe -ArgumentList $f | Out-Null
    $w = Wait-Window 'scratch-np.txt' 30
    if (-not $w) { Write-Host 'SETUP-FAIL notepad window'; exit 1 }
    $hwnd = $w.Hwnd
  }
  'chrome' {
    $prof = Join-Path $env:TEMP 'vkbe2e-chrome2'
    $url = 'file:///E:/vietkb/docs/evkey-clone-work/qa/e2e95/probe.html'
    Start-Process 'C:\Program Files\Google\Chrome\Application\chrome.exe' -ArgumentList @(
      "--user-data-dir=$prof", '--no-first-run', '--no-default-browser-check',
      '--force-renderer-accessibility', "--app=$url") | Out-Null
    $w = Wait-Window 'VietKB E2E probe' 40
    if (-not $w) { Write-Host 'SETUP-FAIL chrome window'; exit 1 }
    $hwnd = $w.Hwnd
  }
  'vscode' {
    $f = Join-Path $dir 'scratch-vsc.txt'
    Remove-Item $f -ErrorAction SilentlyContinue; New-Item -ItemType File -Path $f | Out-Null
    $code = "$env:LOCALAPPDATA\Programs\Microsoft VS Code\Code.exe"
    Start-Process $code -ArgumentList @('-n', $f) | Out-Null
    $w = Wait-Window 'scratch-vsc.txt' 40
    if (-not $w) { Write-Host 'SETUP-FAIL vscode window'; exit 1 }
    $hwnd = $w.Hwnd
  }
}
Write-Host "APP=$App hwnd=$hwnd"

function FocusTarget {
  if ($App -eq 'notepad') { [void](Repair-Notepad 'scratch-np.txt') }
  [void][QAInput]::Focus($hwnd)
  Start-Sleep -Milliseconds 150
  if ($App -ne 'vscode') {
    $el = Get-EditElement $hwnd
    if ($el) { [void](Click-Element $el) } else { Click-Center $hwnd }
  } else {
    Click-Center $hwnd
  }
  Start-Sleep -Milliseconds 150
  return ([QAInput]::GetForegroundWindow() -eq $hwnd)
}

function Read-Actual {
  if ($App -eq 'vscode') {
    [QAInput]::Combo(0x53, $false, $true, $false)  # Ctrl+S
    Start-Sleep -Milliseconds 400
    $p = Join-Path $dir 'scratch-vsc.txt'
    $t = [System.IO.File]::ReadAllText($p, [System.Text.Encoding]::UTF8)
    return [pscustomobject]@{ Text = $t; Via = 'saved-file' }
  }
  $r = Read-Text $hwnd
  if ($null -eq $r.Text) { $r.Text = '' }
  return $r
}

# warm up: the modern Notepad/Chrome/VS Code window exists before it accepts input
$warm = $false
for ($i = 0; $i -lt 12; $i++) {
  Start-Sleep -Milliseconds 700
  $focused = FocusTarget
  if (-not $focused) { Write-Host "warmup: window not focused (try $($i+1))"; continue }
  Clear-TextArea $hwnd
  [QAInput]::TypeAscii('warm', $global:KeyDelay)
  Start-Sleep -Milliseconds 350
  $a = Read-Actual
  if ([string]$a.Text -like '*warm*') { $warm = $true; Write-Host "warmup ok after $($i+1) tries fg=$focused"; break }
}
if (-not $warm) { Write-Host 'WARMUP-FAIL'; exit 2 }
Clear-TextArea $hwnd
if ($App -eq 'chrome' -or $App -eq 'vscode') {
  # refuse to type into a window we did not verify
  $fg = [QAInput]::GetForegroundWindow()
  if ($fg -ne $hwnd) { Write-Host 'GUARD-FAIL foreground drifted'; exit 3 }
}

# Vietnamese ON
[QAInput]::Combo(0x70, $false, $false, $false)  # F1
Start-Sleep -Milliseconds 300

function DoCase($id, $keys, $expected, [int]$bs = 0) {
  $focused = FocusTarget
  Clear-TextArea $hwnd
  [QAInput]::TypeAscii($keys, $global:KeyDelay)
  if ($bs -gt 0) { for ($i = 0; $i -lt $bs; $i++) { [QAInput]::Combo(0x08, $false, $false, $false); Start-Sleep -Milliseconds 90 } }
  Start-Sleep -Milliseconds 400
  $a = Read-Actual
  $txt = ([string]$a.Text).TrimEnd("`r", "`n")
  $v = if ($txt -ceq $expected) { 'PASS' } else { 'FAIL' }
  Row $id $keys $expected $txt $a.Via $focused $v
  return $txt
}

function DoCaseSeq($id, $script, $expected, $keysLabel) {
  $focused = FocusTarget
  Clear-TextArea $hwnd
  & $script
  Start-Sleep -Milliseconds 400
  $a = Read-Actual
  $txt = ([string]$a.Text).TrimEnd("`r", "`n")
  $txtN = $txt -replace "`r`n", "`n"
  $expN = $expected -replace "`r`n", "`n"
  $v = if ($txtN -ceq $expN) { 'PASS' } else { 'FAIL' }
  Row $id $keysLabel $expected $txt $a.Via $focused $v
}

$cases = @(
  @('A1-bat-slash-tat',      'ba6t5/ta8t1',             'bật/tắt'),
  @('A2-bat-dot-tat',        'ba6t5.ta8t1',             'bật.tắt'),
  @('A3-bat-question-tat',   'ba6t5?ta8t1',             'bật?tắt'),
  @('A4-bat-underscore-tat', 'ba6t5_ta8t1',             'bật_tắt'),
  @('A5-windows-path',       'C:\Users\nam',            'C:\Users\nam'),
  @('A6-bat-space-tat',      'ba6t5 ta8t1',             'bật tắt'),
  @('A7-nguyen-thu-ha',      'Nguye6n4 Thu Ha2',        'Nguyễn Thu Hà'),
  @('A8-url-tai',            'https://vietkb.dev/ta3i', 'https://vietkb.dev/tải'),
  @('A9-a-slash-b',          'a/b',                     'a/b'),
  @('A10-2-slash-3',         '2/3',                     '2/3'),
  @('A11-percent',           '100%/50%',                '100%/50%')
)
foreach ($c in $cases) {
  $delay = $global:KeyDelay
  $txt = DoCase $c[0] $c[1] $c[2]
}

DoCase 'B1-bs-tone' 'ba6t5' 'bậ' 1
DoCase 'B2-bs-horn' 'ba6t' 'bâ' 1

# restore-word hotkey: transformed word back to the raw keystrokes
DoCaseSeq 'C1-restore-word' { [QAInput]::TypeAscii('ba6t5', $global:KeyDelay); Start-Sleep -Milliseconds 250; [QAInput]::Combo(0x5A, $true, $true, $false) } 'ba6t5' 'ba6t5 Ctrl+Shift+Z'

# macro: m01 + space -> "mục 1 "
DoCase 'C2-macro-m01' 'm01 ' 'mục 1 '

# boundaries
$d1 = DoCaseSeq 'D1-enter' { [QAInput]::TypeAscii('ba6t5', $global:KeyDelay); [QAInput]::Combo(0x0D, $false, $false, $false); Start-Sleep -Milliseconds 200; [QAInput]::TypeAscii('ba6t5', $global:KeyDelay) } ("bật`r`nbật") 'ba6t5 Enter ba6t5'

# Tab does not insert in a Chrome textarea (browser moves focus); Tab inserts a tab/indent in editors.
$tabExpect = if ($App -eq 'chrome') { 'bậtbật' } else { "bật`tbật" }
$d2 = DoCaseSeq 'D2-tab' { [QAInput]::TypeAscii('ba6t5', $global:KeyDelay); [QAInput]::Combo(0x09, $false, $false, $false); Start-Sleep -Milliseconds 200; [QAInput]::TypeAscii('ba6t5', $global:KeyDelay) } $tabExpect 'ba6t5 Tab ba6t5'

# OFF: ASCII must stay ASCII
$focused = FocusTarget
Clear-TextArea $hwnd
[QAInput]::Combo(0x71, $false, $false, $false)  # F2
Start-Sleep -Milliseconds 450
[QAInput]::TypeAscii('ba6t5', $global:KeyDelay)
Start-Sleep -Milliseconds 350
$a = Read-Actual; $t = ([string]$a.Text).TrimEnd("`r", "`n")
Row 'E1-off-f2' 'ba6t5 after F2' 'ba6t5' $t $a.Via $focused $(if ($t -ceq 'ba6t5') { 'PASS' } else { 'FAIL' })

[QAInput]::Combo(0x70, $false, $false, $false)  # F1
Start-Sleep -Milliseconds 450
Clear-TextArea $hwnd
[QAInput]::TypeAscii('ba6t5', $global:KeyDelay)
Start-Sleep -Milliseconds 350
$a = Read-Actual; $t = ([string]$a.Text).TrimEnd("`r", "`n")
Row 'E2-on-f1' 'ba6t5 after F1' 'bật' $t $a.Via $focused $(if ($t -ceq 'bật') { 'PASS' } else { 'FAIL' })

Clear-TextArea $hwnd
Write-Host "TYPING-DONE $App"




