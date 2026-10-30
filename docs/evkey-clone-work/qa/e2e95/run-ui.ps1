. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
Add-Type -AssemblyName System.Drawing
$dir = 'E:\vietkb\docs\evkey-clone-work\qa\e2e95'
$out = Join-Path $dir 'ui-results.txt'
Set-Content -Encoding UTF8 $out 'UI-E2E'
function W($s) { Add-Content -Encoding UTF8 $out $s; Write-Host $s }

# dismiss any stray Notepad Save dialog
foreach ($h in [QAInput]::TopWindows()) {
  $t = [QAInput]::Title($h)
  if ($t -eq 'Save as' -or $t -eq 'Save As') {
    [void][QAInput]::Focus($h); Start-Sleep -Milliseconds 200
    [QAInput]::Combo(0x1B, $false, $false, $false); Start-Sleep -Milliseconds 400
    W "dismissed dialog [$t]"
  }
}

function Shot($name, $hwnd) {
  $r = New-Object QAInput+RECT
  [void][QAInput]::GetWindowRect($hwnd, [ref]$r)
  Start-Sleep -Milliseconds 250
  $bmp = New-Object System.Drawing.Bitmap ($r.R - $r.L), ($r.B - $r.T)
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.CopyFromScreen($r.L, $r.T, 0, 0, $bmp.Size)
  $bmp.Save((Join-Path $dir $name))
  $g.Dispose(); $bmp.Dispose()
}

function SizeOf($hwnd) {
  $r = New-Object QAInput+RECT; [void][QAInput]::GetWindowRect($hwnd, [ref]$r)
  $c = New-Object QAInput+RECT; [void][QAInput]::GetClientRect($hwnd, [ref]$c)
  return "outer=$($r.R-$r.L)x$($r.B-$r.T) client=$($c.R)x$($c.B) at $($r.L),$($r.T) dpi=$([QAInput]::GetDpiForWindow($hwnd))"
}

$main = Get-TopWindow 'Cài đặt VietKB'
if (-not $main) {
  $fgw = Get-TopWindow 'VietKB E2E probe'
  if ($fgw) { [void][QAInput]::Focus($fgw.Hwnd); Start-Sleep -Milliseconds 300 }
  [QAInput]::Combo(0x53, $false, $true, $true); Start-Sleep -Milliseconds 1500
  $main = Get-TopWindow 'Cài đặt VietKB'
}
if (-not $main) { W 'main=NO_WINDOW'; exit 1 }
[void][QAInput]::Focus($main.Hwnd); Start-Sleep -Milliseconds 500
W "main_window: $(SizeOf $main.Hwnd)"
Shot 'ui-1-main.png' $main.Hwnd

# no sound row anywhere in the main panel
$soundRow = Find-ElementByName $main.Hwnd 'Âm thanh bật/tắt'
W "sound_row_present=$([bool]$soundRow)"

# Nâng cao
$adv = Find-ElementByName $main.Hwnd 'Nâng cao…' 'Button'
if ($adv) {
  $adv.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
  Start-Sleep -Milliseconds 1200
  $advWin = $null
  for ($i = 0; $i -lt 12; $i++) {
    $advWin = Get-TopWindow 'Nâng cao'
    if ($advWin) { break }
    Start-Sleep -Milliseconds 400
  }
  if ($advWin) {
    W "advanced_window: $(SizeOf $advWin.Hwnd) title=[$($advWin.Title)]"
    Shot 'ui-2-advanced.png' $advWin.Hwnd
    $txt = (Get-UiaText $advWin.Hwnd).Text
    W "advanced_has_sound_row=$([bool]($txt -match 'Âm thanh'))"
    W "advanced_has_pack_select=$([bool]($txt -match 'Kiểu âm thanh|bộ âm thanh'))"
    # scroll/clipping check: last row + footer inside the client rect
    $root = [System.Windows.Automation.AutomationElement]::FromHandle($advWin.Hwnd)
    $c = New-Object QAInput+RECT; [void][QAInput]::GetClientRect($advWin.Hwnd, [ref]$c)
    $wr = New-Object QAInput+RECT; [void][QAInput]::GetWindowRect($advWin.Hwnd, [ref]$wr)
    $bottomClient = $wr.T + $c.B
    $maxBottom = 0; $maxName = ''
    foreach ($el in $root.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)) {
      try {
        $r = $el.Current.BoundingRectangle
        if ($r.Width -gt 0) { $b = $r.Y + $r.Height; if ($b -gt $maxBottom) { $maxBottom = $b; $maxName = $el.Current.Name } }
      } catch {}
    }
    W ("advanced_client_bottom={0} lowest_element_bottom={1} [{2}] clipped={3}" -f $bottomClient, [int]$maxBottom, $maxName, ($maxBottom -gt $bottomClient))
    [QAInput]::Combo(0x1B, $false, $false, $false); Start-Sleep -Milliseconds 700
    $still = Get-TopWindow 'Nâng cao'
    W "advanced_closed=$(-not [bool]$still)"
  } else { W 'advanced=NO_WINDOW' }
} else { W 'advanced_button=NOT_FOUND' }

# Phím tắt
[void][QAInput]::Focus($main.Hwnd); Start-Sleep -Milliseconds 300
$hk = Find-ElementByName $main.Hwnd 'Phím tắt…' 'Button'
if ($hk) {
  $hk.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
  Start-Sleep -Milliseconds 1200
  $hkWin = $null
  for ($i = 0; $i -lt 12; $i++) { $hkWin = Get-TopWindow 'Phím tắt'; if ($hkWin) { break }; Start-Sleep -Milliseconds 400 }
  W "hotkeys_window=$([bool]$hkWin) $(if($hkWin){SizeOf $hkWin.Hwnd})"
  if ($hkWin) { Shot 'ui-3-hotkeys.png' $hkWin.Hwnd; [QAInput]::Combo(0x1B, $false, $false, $false); Start-Sleep -Milliseconds 600; W "hotkeys_closed=$(-not [bool](Get-TopWindow 'Phím tắt'))" }
} else { W 'hotkeys_button=NOT_FOUND' }

# Bảng gõ tắt
[void][QAInput]::Focus($main.Hwnd); Start-Sleep -Milliseconds 300
$mf = Find-ElementByName $main.Hwnd 'Bảng gõ tắt…' 'Button'
if ($mf) {
  $mf.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
  Start-Sleep -Milliseconds 1200
  $mfWin = $null
  for ($i = 0; $i -lt 12; $i++) { $mfWin = Get-TopWindow 'Gõ tắt'; if ($mfWin) { break }; Start-Sleep -Milliseconds 400 }
  W "macros_window=$([bool]$mfWin) $(if($mfWin){SizeOf $mfWin.Hwnd})"
  if ($mfWin) { Shot 'ui-4-macros.png' $mfWin.Hwnd; [QAInput]::Combo(0x1B, $false, $false, $false); Start-Sleep -Milliseconds 600; W "macros_closed=$(-not [bool](Get-TopWindow 'Gõ tắt'))" }
} else { W 'macros_button=NOT_FOUND' }

# Giới thiệu — where is it?
$about = Find-ElementByName $main.Hwnd 'Giới thiệu'
W "about_on_main=$([bool]$about)"
W 'UI-DONE'
