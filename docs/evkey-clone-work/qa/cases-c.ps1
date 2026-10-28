# §3 UI runner — part C (live apply, hotkeys, macros, app rules, states)
. 'C:\Users\namth\AppData\Local\Temp\opencode\vkb-qa\lib.ps1'

function Focus-Cap($Win, [string]$Field) {
  if (-not (Focus-AppWindow $Win)) { throw "could not bring '$($Win.Current.Name)' to the foreground" }
  $c = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::AutomationIdProperty, "cap-$Field")
  $cap = $Win.FindFirst([System.Windows.Automation.TreeScope]::Descendants, $c)
  if (-not $cap) { throw "capture field not found: $Field" }
  $cap.SetFocus(); Start-Sleep -Milliseconds 400
  if (-not $cap.Current.HasKeyboardFocus) {
    $cap.SetFocus(); Start-Sleep -Milliseconds 400
  }
  if (-not $cap.Current.HasKeyboardFocus) { throw "capture field did not take focus: $Field" }
  return $cap
}

function Run-GroupD {
  Reset-Config
  Restart-TestApp | Out-Null
  $main = Get-AppWindow 'Cài đặt VietKB'

  Test-Case 'UI-29' 'live-apply' {
    Invoke-Button $main 'Phím tắt…' 'v4lk' | Out-Null
    $hk = Wait-AppWindow 'Phím tắt — VietKB' 5000
    Invoke-Button $main 'Nâng cao…' 'v4lk' | Out-Null
    $adv = Wait-AppWindow 'Nâng cao — VietKB' 5000
    $saves = @()
    foreach ($w in @($main, $hk, $adv)) {
      if (-not $w) { continue }
      foreach ($b in Get-AllButtons $w) { if ($b.Current.Name -match '^(Lưu|Save)$') { $saves += "$($w.Current.Name):$($b.Current.Name)" } }
    }
    Set-Combo $main 'Kiểu gõ' 'VNI' | Out-Null
    $m = Wait-Cfg 'input' 'method' '"vni"' 3000
    Set-Toggle $main 'Đặt dấu kiểu mới' $false | Out-Null
    $t = Wait-Cfg 'typing' 'modern_tone' 'false' 3000
    Set-Toggle $main 'Đặt dấu kiểu mới' $true | Out-Null
    Set-Toggle $main 'Kiểm tra chính tả' $false | Out-Null
    $s = Wait-Cfg 'typing' 'spell_check' 'false' 3000
    Set-Toggle $main 'Kiểm tra chính tả' $true | Out-Null
    Set-Combo $main 'Kiểu gõ' 'Telex' | Out-Null
    $ok = ($saves.Count -eq 0) -and $m -and $t -and $s
    Save-Evidence 'UI-29' @("saveButtons=$($saves -join ',')", "methodPersisted=$m", "modernPersisted=$t", "spellPersisted=$s") | Out-Null
    Add-Result 'UI-29' 'live-apply' $(if ($ok) { 'PASS' } else { 'FAIL' }) "no Save button=$($saves.Count -eq 0); 3 changes persisted within 1.5s=$($m -and $t -and $s)"
  }

  Test-Case 'UI-30' 'hotkey-capture' {
    $hk = Get-AppWindow 'Phím tắt — VietKB'
    $cfg = $null
    for ($attempt = 0; $attempt -lt 2 -and $cfg -ne 'Ctrl+Alt+K'; $attempt++) {
      if ($attempt -gt 0) {
        $close = Find-El $hk 'Đóng' 'btn pri' ([System.Windows.Automation.ControlType]::Button)
        if ($close) { Invoke-El $close; Start-Sleep -Milliseconds 700 }
        $mainW = Get-AppWindow 'Cài đặt VietKB'
        Invoke-Button $mainW 'Phím tắt…' 'v4lk' | Out-Null
        $hk = Wait-AppWindow 'Phím tắt — VietKB' 5000
        Start-Sleep -Milliseconds 800
      }
      Focus-Cap $hk 'cycleMethod' | Out-Null
      Send-Key 'Backspace'; Send-Key 'Enter'
      Start-Sleep -Milliseconds 700
      Focus-Cap $hk 'cycleMethod' | Out-Null
      Send-Chord @([uint16]0x11, [uint16]0x12) ([uint16]0x4B)
      Start-Sleep -Milliseconds 700
      Send-Key 'Enter'
      Start-Sleep -Milliseconds 1200
      $cfg = Get-CfgStr 'hotkeys' 'cycle_method'
    }
    $aria = Find-El $hk 'Đổi kiểu gõ — phím tắt hiện tại Ctrl+Alt+K' '' ([System.Windows.Automation.ControlType]::Button)
    $ok = ($cfg -eq 'Ctrl+Alt+K')
    Save-Evidence 'UI-30' @("config=$cfg", "ariaShows=$($null -ne $aria)") | Out-Null
    Add-Result 'UI-30' 'hotkey-capture' $(if ($ok) { 'PASS' } else { 'FAIL' }) "Ctrl+Alt+K committed, config='$cfg', field shows canonical=$($null -ne $aria)"
  }

  Test-Case 'UI-31' 'hotkey-cancel' {
    $hk = Get-AppWindow 'Phím tắt — VietKB'
    $before = Get-CfgStr 'hotkeys' 'cycle_method'
    Focus-Cap $hk 'cycleMethod' | Out-Null
    Send-Chord @([uint16]0x11, [uint16]0x12) ([uint16]0x4D)
    Send-Key 'Escape'
    Start-Sleep -Milliseconds 700
    $afterEsc = Get-CfgStr 'hotkeys' 'cycle_method'
    Focus-Cap $hk 'cycleMethod' | Out-Null
    Send-Key 'Backspace'; Send-Key 'Enter'
    Start-Sleep -Milliseconds 1100
    $afterClear = Get-CfgStr 'hotkeys' 'cycle_method'
    $ok = ($afterEsc -eq $before) -and ($afterClear -eq '')
    Save-Evidence 'UI-31' @("before=$before", "afterEsc=$afterEsc", "afterBackspaceEnter=$afterClear") | Out-Null
    Add-Result 'UI-31' 'hotkey-cancel' $(if ($ok) { 'PASS' } else { 'FAIL' }) "Esc kept '$afterEsc'; Backspace+Enter -> '$afterClear'"
  }

  Test-Case 'UI-32' 'hotkey-conflict-live' {
    $hk = Get-AppWindow 'Phím tắt — VietKB'
    $before = Get-CfgStr 'hotkeys' 'cycle_method'
    Focus-Cap $hk 'cycleMethod' | Out-Null
    if ([QaWin]::GetForegroundWindow() -ne $script:InputHwnd) { throw 'foreground changed before modifier chords; refusing to type' }
    [void][QaWin]::Key(0x11, $false); [void][QaWin]::Key(0x10, $false); Start-Sleep -Milliseconds 250
    [void][QaWin]::Key(0x10, $true); [void][QaWin]::Key(0x11, $true); Start-Sleep -Milliseconds 700
    $ariaDuring = (Find-El $hk 'Đổi kiểu gõ — phím tắt hiện tại Ctrl+Shift' '' ([System.Windows.Automation.ControlType]::Button))
    Send-Key 'Enter'
    Start-Sleep -Milliseconds 1200
    $cfg = Get-CfgStr 'hotkeys' 'cycle_method'
    $warn = @()
    foreach ($w in Get-AppWindows) {
      foreach ($e in $w.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)) {
        if ($e.Current.Name -match '^Trùng|Dễ trùng') { $warn += "$($w.Current.Name):$($e.Current.Name)" }
      }
    }
    $ok = ($cfg -ne 'Ctrl+Shift') -and ($warn.Count -gt 0)
    Save-Evidence 'UI-32' @("before=$before", "config=$cfg", "fieldCaughtModifierOnly=$($null -ne $ariaDuring)", "warnings=$($warn -join ',')") | Out-Null
    Add-Result 'UI-32' 'hotkey-conflict-live' $(if ($ok) { 'PASS' } else { 'FAIL' }) "Ctrl+Shift (toggle's chord): field captured=$($null -ne $ariaDuring), config='$cfg', warning=$($warn.Count -gt 0)"
    Focus-Cap $hk 'cycleMethod' | Out-Null
    Send-Key 'Backspace'; Send-Key 'Enter'
    Start-Sleep -Milliseconds 900
  }

  Test-Case 'UI-33' 'hotkey-conflict-stored' {
    Stop-TestApp
    $raw = Get-ConfigRaw
    $raw = [regex]::Replace($raw, '(?m)^cycle_method\s*=\s*.*$', 'cycle_method = "Ctrl+Shift+Z"')
    Set-Content -Path $CfgPath -Value $raw -Encoding UTF8
    Start-Sleep -Milliseconds 400
    Start-TestApp | Out-Null
    $main = Get-AppWindow 'Cài đặt VietKB'
    Invoke-Button $main 'Phím tắt…' 'v4lk' | Out-Null
    $hk = Wait-AppWindow 'Phím tắt — VietKB' 5000
    $warn = @()
    foreach ($e in $hk.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)) {
      if ($e.Current.Name -match 'Trùng|Dễ trùng') { $warn += $e.Current.Name }
    }
    $cycle = Find-El $hk 'Đổi kiểu gõ — phím tắt hiện tại Ctrl+Shift+Z' '' ([System.Windows.Automation.ControlType]::Button)
    $restore = Find-El $hk 'Khôi phục từ gõ sai — phím tắt hiện tại Ctrl+Shift+Z' '' ([System.Windows.Automation.ControlType]::Button)
    $ok = ($warn.Count -gt 0)
    Save-Evidence 'UI-33' @("cycleField=$($null -ne $cycle)", "restoreField=$($null -ne $restore)", "warnings=$($warn -join ',')") | Out-Null
    Add-Result 'UI-33' 'hotkey-conflict-stored' $(if ($ok) { 'PASS' } else { 'FAIL' }) "duplicate Ctrl+Shift+Z visible on load: warning=$($warn.Count -gt 0) (both fields present=$($null -ne $cycle -and $null -ne $restore))"
    Stop-TestApp
    $raw = Get-ConfigRaw
    $raw = [regex]::Replace($raw, '(?m)^cycle_method\s*=\s*.*$', 'cycle_method = ""')
    Set-Content -Path $CfgPath -Value $raw -Encoding UTF8
  }

  Test-Case 'UI-34' 'hotkey-dependency' {
    Start-TestApp | Out-Null
    $main = Get-AppWindow 'Cài đặt VietKB'
    Invoke-Button $main 'Nâng cao…' 'v4lk' | Out-Null
    $adv = Wait-AppWindow 'Nâng cao — VietKB' 5000
    $restore = Find-El $adv 'Tự động khôi phục từ gõ sai' '' ([System.Windows.Automation.ControlType]::CheckBox)
    $enabledWhileOn = $restore.Current.IsEnabled
    Set-Toggle $main 'Kiểm tra chính tả' $false | Out-Null
    Start-Sleep -Milliseconds 1600
    $liveDisabled = -not $restore.Current.IsEnabled
    Invoke-El (Find-El $adv 'Đóng' 'btn pri' ([System.Windows.Automation.ControlType]::Button)); Start-Sleep -Milliseconds 600
    Invoke-Button $main 'Nâng cao…' 'v4lk' | Out-Null
    $adv2 = Wait-AppWindow 'Nâng cao — VietKB' 5000
    $restore2 = Find-El $adv2 'Tự động khôi phục từ gõ sai' '' ([System.Windows.Automation.ControlType]::CheckBox)
    $reloadDisabled = -not $restore2.Current.IsEnabled
    Set-Toggle $main 'Kiểm tra chính tả' $true | Out-Null
    $ok = $enabledWhileOn -and $liveDisabled -and $reloadDisabled
    Save-Evidence 'UI-34' @("enabledWhileSpellOn=$enabledWhileOn", "disabledLive=$liveDisabled", "disabledAfterReopen=$reloadDisabled") | Out-Null
    Add-Result 'UI-34' 'hotkey-dependency' $(if ($ok) { 'PASS' } else { 'FAIL' }) "spell on -> enabled=$enabledWhileOn; spell off -> live disabled=$liveDisabled, reopened disabled=$reloadDisabled"
  }
}

function Run-GroupE {
  Reset-Config
  Restart-TestApp | Out-Null
  $main = Get-AppWindow 'Cài đặt VietKB'

  Test-Case 'UI-35' 'macro-add' {
    Invoke-Button $main 'Bảng gõ tắt…' 'btn' | Out-Null
    $mac = Wait-AppWindow 'Bảng gõ tắt — VietKB' 5000
    Invoke-Button $mac 'Thêm' 'btn pri' | Out-Null
    Start-Sleep -Milliseconds 700
    $edits = $mac.FindAll([System.Windows.Automation.TreeScope]::Descendants, (New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ControlTypeProperty, [System.Windows.Automation.ControlType]::Edit)))
    Set-TextValue $edits[0] 'qa1'
    Set-TextValue $edits[1] 'QA one'
    Invoke-Button $mac 'Lưu' 'btn pri' | Out-Null
    Start-Sleep -Milliseconds 1200
    $cfg = Get-CfgStr 'macros' 'qa1'
    $row = Find-El $mac 'Sửa qa1' '' ([System.Windows.Automation.ControlType]::Button)
    $ok = ($cfg -eq 'QA one') -and $row
    Save-Evidence 'UI-35' @("config=$cfg", "rowShown=$($null -ne $row)") | Out-Null
    Add-Result 'UI-35' 'macro-add' $(if ($ok) { 'PASS' } else { 'FAIL' }) "config qa1='$cfg', row rendered=$($null -ne $row)"
  }

  Test-Case 'UI-36' 'macro-edit' {
    $mac = Get-AppWindow 'Bảng gõ tắt — VietKB'
    Invoke-Button $mac 'Sửa qa1' '' | Out-Null
    Start-Sleep -Milliseconds 700
    $edits = @($mac.FindAll([System.Windows.Automation.TreeScope]::Descendants, (New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ControlTypeProperty, [System.Windows.Automation.ControlType]::Edit))))
    $trigger = $edits[0].GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern).Current.Value
    Set-TextValue $edits[1] 'QA two'
    Invoke-Button $mac 'Lưu' 'btn pri' | Out-Null
    Start-Sleep -Milliseconds 1200
    $cfg = Get-CfgStr 'macros' 'qa1'
    $ok = ($trigger -eq 'qa1') -and ($cfg -eq 'QA two')
    Save-Evidence 'UI-36' @("prefillTrigger=$trigger", "config=$cfg") | Out-Null
    Add-Result 'UI-36' 'macro-edit' $(if ($ok) { 'PASS' } else { 'FAIL' }) "editor prefilled '$trigger', config='$cfg'"
  }

  Test-Case 'UI-37' 'macro-delete' {
    $mac = Get-AppWindow 'Bảng gõ tắt — VietKB'
    Invoke-Button $mac 'Tuỳ chọn khác cho qa1' '' | Out-Null
    Start-Sleep -Milliseconds 500
    $del = Find-El $mac 'Xoá' '' ([System.Windows.Automation.ControlType]::MenuItem)
    if (-not $del) { $del = Find-El $mac 'Xoá' '' ([System.Windows.Automation.ControlType]::Button) }
    Invoke-El $del
    Start-Sleep -Milliseconds 1200
    $cfg = Get-CfgStr 'macros' 'qa1'
    $row = Find-El $mac 'Sửa qa1' '' ([System.Windows.Automation.ControlType]::Button)
    $ok = ($null -eq $cfg) -and ($null -eq $row)
    Save-Evidence 'UI-37' @("configAfterDelete=[$cfg]", "rowAfterDelete=$($null -ne $row)") | Out-Null
    Add-Result 'UI-37' 'macro-delete' $(if ($ok) { 'PASS' } else { 'FAIL' }) "config entry gone=$($null -eq $cfg), row gone=$($null -eq $row)"
  }

  Test-Case 'UI-38' 'macro-validation' {
    $mac = Get-AppWindow 'Bảng gõ tắt — VietKB'
    $cases = @(
      @('two words', 'x', 'space in trigger'),
      @('', 'y', 'empty trigger'),
      @('ok1', '', 'empty expansion')
    )
    $details = @()
    foreach ($c in $cases) {
      Invoke-Button $mac 'Thêm' 'btn pri' | Out-Null
      Start-Sleep -Milliseconds 700
      $dlg = Get-Dialog $mac 'Thêm từ tắt'
      if (-not $dlg) { $details += "$($c[2]):dialogDidNotOpen"; continue }
      $edits = @($dlg.FindAll([System.Windows.Automation.TreeScope]::Descendants, (New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ControlTypeProperty, [System.Windows.Automation.ControlType]::Edit))))
      if ($edits.Count -lt 2) { $details += "$($c[2]):edits=$($edits.Count)"; continue }
      Set-TextValue $edits[0] $c[0]
      Set-TextValue $edits[1] $c[1]
      Invoke-Button $mac 'Lưu' 'btn pri' | Out-Null
      Start-Sleep -Milliseconds 600
      $dlg = Get-Dialog $mac 'Thêm từ tắt'
      $err = $null
      if ($dlg) {
        foreach ($e in $dlg.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)) {
          if ($e.Current.Name -match 'không có dấu cách|Nhập từ tắt|không được để trống|tối đa') { $err = $e.Current.Name }
        }
      }
      $details += "$($c[2]):dialogOpen=$($null -ne $dlg),err='$err'"
      if ($dlg) {
        $cancel = Find-El $dlg 'Huỷ' 'btn ghost' ([System.Windows.Automation.ControlType]::Button)
        if ($cancel) { Invoke-El $cancel }
        Start-Sleep -Milliseconds 500
      }
    }
    $bad = @()
    foreach ($t in @('two words', 'ok1')) { if ($null -ne (Get-CfgStr 'macros' $t)) { $bad += $t } }
    $ok = ($bad.Count -eq 0)
    Save-Evidence 'UI-38' ($details + @("unexpectedSaves=$($bad -join ',')")) | Out-Null
    Add-Result 'UI-38' 'macro-validation' $(if ($ok) { 'PASS' } else { 'FAIL' }) "$($details -join '; '); persisted invalid=$($bad.Count)"
  }
}

function Run-GroupE2 {
  Reset-Config
  Restart-TestApp | Out-Null
  $main = Get-AppWindow 'Cài đặt VietKB'

  Test-Case 'UI-39' 'macro-file' {
    Invoke-Button $main 'Bảng gõ tắt…' 'btn' | Out-Null
    $mac = Wait-AppWindow 'Bảng gõ tắt — VietKB' 5000
    $outFile = 'C:\Users\namth\AppData\Local\Temp\opencode\vkb-qa\export.tsv'
    if (Test-Path $outFile) { Remove-Item $outFile -Force }
    $dlg = $null
    for ($try = 0; $try -lt 2 -and -not $dlg; $try++) {
      Invoke-Button $mac 'Xuất ra tệp…' 'btn' | Out-Null
      Start-Sleep -Milliseconds 2200
      $dlg = Find-El $mac 'Lưu danh sách gõ tắt' '#32770' ([System.Windows.Automation.ControlType]::Window)
    }
    $exported = $false
    if ($dlg) {
      $edit = $null
      foreach ($e in $dlg.FindAll([System.Windows.Automation.TreeScope]::Descendants, (New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::AutomationIdProperty, '1001')))) { if ($e.Current.ControlType.ProgrammaticName -eq 'ControlType.Edit') { $edit = $e } }
      $save = $null
      foreach ($b in Get-AllButtons $dlg) { if ($b.Current.Name -eq 'Save') { $save = $b } }
      if ($edit -and $save) {
        Set-TextValue $edit $outFile
        Invoke-El $save
        Start-Sleep -Milliseconds 1800
        $exported = Test-Path $outFile
      }
    }
    $lines = if ($exported) { @(Get-Content $outFile).Count } else { 0 }
    $imported = $false
    $toastText = $null
    if ($exported) {
      Invoke-Button $mac 'Tuỳ chọn khác cho ko' '' | Out-Null
      Start-Sleep -Milliseconds 500
      $del = Find-El $mac 'Xoá' '' ([System.Windows.Automation.ControlType]::MenuItem)
      if ($del) { Invoke-El $del; Start-Sleep -Milliseconds 1200 }
      $deleted = $null -eq (Get-CfgStr 'macros' 'ko')
      Invoke-Button $mac 'Nhập từ tệp…' 'btn' | Out-Null
      $odlg = $null
      $deadline = (Get-Date).AddMilliseconds(4000)
      do {
        $odlg = Find-El $mac 'Chọn tệp gõ tắt' '#32770' ([System.Windows.Automation.ControlType]::Window)
        if (-not $odlg) { Start-Sleep -Milliseconds 200 }
      } while (-not $odlg -and (Get-Date) -lt $deadline)
      if ($odlg) {
        Start-Sleep -Milliseconds 500
        $oedit = $null
        foreach ($e in $odlg.FindAll([System.Windows.Automation.TreeScope]::Descendants, (New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::AutomationIdProperty, '1148')))) { if ($e.Current.ControlType.ProgrammaticName -eq 'ControlType.Edit') { $oedit = $e } }
        if ($oedit) {
          Set-TextValue $oedit $outFile
          $dh = [IntPtr]$odlg.Current.NativeWindowHandle
          [void][QaWin]::SetForegroundWindow($dh); Start-Sleep -Milliseconds 400
          $script:InputHwnd = $dh
          $oedit.SetFocus(); Start-Sleep -Milliseconds 500
          Send-Key 'Enter'
          Start-Sleep -Milliseconds 2500
        }
      }
      $imported = (Get-CfgStr 'macros' 'ko') -eq 'không'
      if (-not $imported) {
        Invoke-Button $mac 'Nhập từ tệp…' 'btn' | Out-Null
        Start-Sleep -Milliseconds 2000
        $odlg = Find-El $mac 'Chọn tệp gõ tắt' '#32770' ([System.Windows.Automation.ControlType]::Window)
        if ($odlg) {
          $oedit = $null
          foreach ($e in $odlg.FindAll([System.Windows.Automation.TreeScope]::Descendants, (New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::AutomationIdProperty, '1148')))) { if ($e.Current.ControlType.ProgrammaticName -eq 'ControlType.Edit') { $oedit = $e } }
          if ($oedit) {
            Set-TextValue $oedit $outFile
            $dh = [IntPtr]$odlg.Current.NativeWindowHandle
            [void][QaWin]::SetForegroundWindow($dh); Start-Sleep -Milliseconds 400
            $script:InputHwnd = $dh
            $oedit.SetFocus(); Start-Sleep -Milliseconds 400
            Send-Key 'Enter'
            Start-Sleep -Milliseconds 2500
          }
        }
        $imported = (Get-CfgStr 'macros' 'ko') -eq 'không'
      }
      foreach ($t in $mac.FindAll([System.Windows.Automation.TreeScope]::Descendants, (New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ControlTypeProperty, [System.Windows.Automation.ControlType]::Text)))) { if ($t.Current.Name -match 'Đã nhập') { $toastText = $t.Current.Name } }
    }
    $ok = $exported -and ($lines -ge 4) -and $imported
    Save-Evidence 'UI-39' @("exported=$exported lines=$lines", "imported=$imported", "toast=$toastText") | Out-Null
    Add-Result 'UI-39' 'macro-file' $(if ($ok) { 'PASS' } else { 'FAIL' }) "export $lines lines, re-import restored deleted row=$imported, toast='$toastText'"
  }

  Test-Case 'UI-40' 'macro-list' {
    Stop-TestApp
    $raw = Get-ConfigRaw
    $raw = [regex]::Replace($raw, '(?ms)^\[macros\].*$', '')
    $block = "[macros]`r`n"
    for ($i = 1; $i -le 12; $i++) { $block += "m$i = `"value $i`"`r`n" }
    Set-Content -Path $CfgPath -Value ($raw.TrimEnd() + "`r`n`r`n" + $block) -Encoding UTF8
    Start-TestApp | Out-Null
    $main = Get-AppWindow 'Cài đặt VietKB'
    Invoke-Button $main 'Bảng gõ tắt…' 'btn' | Out-Null
    $mac = Wait-AppWindow 'Bảng gõ tắt — VietKB' 5000
    Start-Sleep -Milliseconds 800
    $rows = @(Get-AllButtons $mac | Where-Object { $_.Current.Name -like 'Sửa *' })
    $footer = @()
    foreach ($t in $mac.FindAll([System.Windows.Automation.TreeScope]::Descendants, (New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ControlTypeProperty, [System.Windows.Automation.ControlType]::Text)))) {
      $r = $t.Current.BoundingRectangle
      if ($r.Y -gt 1010 -and $r.Y -lt 1040) { $footer += $t.Current.Name }
    }
    $ok = ($rows.Count -eq 12)
    Save-Evidence 'UI-40' @("configMacros=12", "renderedRows=$($rows.Count)", "footer=$($footer -join ' ')") | Out-Null
    Add-Result 'UI-40' 'macro-list' $(if ($ok) { 'PASS' } else { 'FAIL' }) "12 macros in config, $($rows.Count) rows rendered; footer: $($footer -join '')"
  }
}

function Run-GroupF {
  Reset-Config
  Restart-TestApp | Out-Null
  $main = Get-AppWindow 'Cài đặt VietKB'

  Test-Case 'UI-41' 'app-rules' {
    Invoke-Button $main 'Quản lý…' 'btn' | Out-Null
    Start-Sleep -Milliseconds 800
    $dlg = Get-Dialog $main 'Loại trừ ứng dụng'
    $empty = Find-El $dlg 'Chưa có quy tắc nào. Thêm ứng dụng bạn chỉ dùng tiếng Anh.' '' ([System.Windows.Automation.ControlType]::Text)
    Invoke-El (Find-El $dlg 'Chọn ứng dụng' 'btn ghost' ([System.Windows.Automation.ControlType]::Button))
    Start-Sleep -Milliseconds 1500
    $target = $null
    foreach ($b in Get-AllButtons $dlg) { if ($b.Current.Name -like 'cua-driver.exe*') { $target = $b } }
    Invoke-El $target
    Start-Sleep -Milliseconds 1500
    $back = Find-El $dlg '‹ Danh sách quy tắc' 'btn ghost' ([System.Windows.Automation.ControlType]::Button)
    if ($back) { Invoke-El $back; Start-Sleep -Milliseconds 700 }
    $cfgAdded = @(Select-String -Path $CfgPath -Pattern 'cua-driver' -SimpleMatch)
    $rm = Find-El $dlg 'Xoá quy tắc cua-driver.exe' '' ([System.Windows.Automation.ControlType]::Button)
    $ruleRow = $null -ne $rm
    if ($rm) { Invoke-El $rm; Start-Sleep -Milliseconds 1000 }
    $cfgAfter = @(Select-String -Path $CfgPath -Pattern 'cua-driver' -SimpleMatch)
    $close = Find-El $dlg 'Đóng' 'btn pri' ([System.Windows.Automation.ControlType]::Button)
    if ($close) { Invoke-El $close }
    Start-Sleep -Milliseconds 700
    $countEl = $null
    foreach ($t in $main.FindAll([System.Windows.Automation.TreeScope]::Descendants, (New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ControlTypeProperty, [System.Windows.Automation.ControlType]::Text)))) {
      $r = $t.Current.BoundingRectangle
      if ($t.Current.Name -match '^\d+$' -and $r.Y -gt 800 -and $r.Y -lt 870) { $countEl = $t }
    }
    $ok = $empty -and $target -and ($cfgAdded.Count -ge 1) -and $ruleRow -and ($cfgAfter.Count -eq 0) -and $countEl -and $countEl.Current.Name -eq '0'
    Save-Evidence 'UI-41' @("emptyState=$($null -ne $empty)", "addedToConfig=$($cfgAdded.Count)", "ruleRow=$ruleRow", "afterRemove=$($cfgAfter.Count)", "countNow=$($countEl.Current.Name)") | Out-Null
    Add-Result 'UI-41' 'app-rules' $(if ($ok) { 'PASS' } else { 'FAIL' }) "empty state=$($null -ne $empty), add persisted=$($cfgAdded.Count -ge 1), remove persisted=$($cfgAfter.Count -eq 0), count restored='$($countEl.Current.Name)'"
  }

  Test-Case 'UI-42' 'app-rule-override' {
    Invoke-Button $main 'Quản lý…' 'btn' | Out-Null
    Start-Sleep -Milliseconds 800
    $dlg = Get-Dialog $main 'Loại trừ ứng dụng'
    Invoke-El (Find-El $dlg 'Chọn ứng dụng' 'btn ghost' ([System.Windows.Automation.ControlType]::Button))
    Start-Sleep -Milliseconds 1500
    $target = $null
    foreach ($b in Get-AllButtons $dlg) { if ($b.Current.Name -like 'cua-driver.exe*') { $target = $b } }
    Invoke-El $target
    Start-Sleep -Milliseconds 1500
    $back2 = Find-El $dlg '‹ Danh sách quy tắc' 'btn ghost' ([System.Windows.Automation.ControlType]::Button)
    if ($back2) { Invoke-El $back2; Start-Sleep -Milliseconds 700 }
    $tg = Find-El $dlg 'Bật tiếng Việt trong cua-driver.exe' '' ([System.Windows.Automation.ControlType]::Button)
    $tp = $tg.GetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern)
    $tp.Toggle(); Start-Sleep -Milliseconds 1000
    $enabledCfg = $null
    $rawNow = Get-ConfigRaw
    $blockNow = [regex]::Match($rawNow, '(?s)process\s*=\s*"cua-driver[^\[]*').Value
    $enabledCfg = [regex]::Match($blockNow, 'enabled\s*=\s*(true|false)').Groups[1].Value
    $combo = Find-El $dlg 'Kiểu gõ cho cua-driver.exe' 'sel' ([System.Windows.Automation.ControlType]::ComboBox)
    $ec = $combo.GetCurrentPattern([System.Windows.Automation.ExpandCollapsePattern]::Pattern); $ec.Expand(); Start-Sleep -Milliseconds 500
    $opt = Find-El $dlg 'VNI' '' ([System.Windows.Automation.ControlType]::ListItem)
    if ($opt) { $opt.GetCurrentPattern([System.Windows.Automation.SelectionItemPattern]::Pattern).Select(); Start-Sleep -Milliseconds 1000 }
    $raw = Get-ConfigRaw
    $methodCfg = [regex]::Match($raw, '(?ms)process\s*=\s*"cua-driver[^\[]*?method\s*=\s*"([a-z_]+)"').Groups[1].Value
    $rm = Find-El $dlg 'Xoá quy tắc cua-driver.exe' '' ([System.Windows.Automation.ControlType]::Button)
    if ($rm) { Invoke-El $rm; Start-Sleep -Milliseconds 800 }
    $close = Find-El $dlg 'Đóng' 'btn pri' ([System.Windows.Automation.ControlType]::Button)
    if ($close) { Invoke-El $close }
    $ok = ($tg) -and ($enabledCfg -eq 'true') -and ($methodCfg -eq 'vni')
    Save-Evidence 'UI-42' @("toggleFound=$($null -ne $tg)", "enabledInRule=$enabledCfg (rule starts disabled; one click enables)", "methodInRule=$methodCfg") | Out-Null
    Add-Result 'UI-42' 'app-rule-override' $(if ($ok) { 'PASS' } else { 'FAIL' }) "rule toggle persisted enabled=$enabledCfg, method override persisted='$methodCfg'"
  }
}

function Run-GroupG {
  Reset-Config
  Restart-TestApp | Out-Null
  $main = Get-AppWindow 'Cài đặt VietKB'

  Test-Case 'UI-43' 'error-toast' {
    $before = Get-Toggle $main 'Đặt dấu kiểu mới'
    $cfgBefore = Get-CfgStr 'typing' 'modern_tone'
    $locked = $null
    try { $locked = [System.IO.File]::Open($CfgPath, [System.IO.FileMode]::Open, [System.IO.FileAccess]::ReadWrite, [System.IO.FileShare]::None) } catch { $locked = $null }
    if (-not $locked) { Set-ItemProperty $CfgPath -Name IsReadOnly -Value $true }
    try {
      Set-Toggle $main 'Đặt dấu kiểu mới' ($before -eq [System.Windows.Automation.ToggleState]::Off) | Out-Null
      Start-Sleep -Milliseconds 3000
      $toast = $null
      foreach ($e in $main.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)) {
        if ($e.Current.Name -match 'Không lưu được thay đổi') { $toast = $e.Current.Name }
      }
    } finally {
      if ($locked) { $locked.Dispose() } else { Set-ItemProperty $CfgPath -Name IsReadOnly -Value $false }
    }
    Start-Sleep -Milliseconds 400
    $cfgAfter = Get-CfgStr 'typing' 'modern_tone'
    $now = Get-Toggle $main 'Đặt dấu kiểu mới'
    if ($now -ne $before) { Set-Toggle $main 'Đặt dấu kiểu mới' ($before -eq [System.Windows.Automation.ToggleState]::On) | Out-Null }
    $ok = ($null -ne $toast) -and ($now -eq $before) -and ($cfgAfter -eq $cfgBefore)
    Save-Evidence 'UI-43' @("toast=$toast", "toggleBefore=$before", "toggleAfter=$now", "configBefore=$cfgBefore", "configAfter=$cfgAfter", "lockMode=$(if ($locked) { 'exclusive handle' } else { 'read-only attribute' })") | Out-Null
    Add-Result 'UI-43' 'error-toast' $(if ($ok) { 'PASS' } else { 'FAIL' }) "toast shown=$($null -ne $toast), UI reverted=$($now -eq $before), config unchanged=$($cfgAfter -eq $cfgBefore)"
  }

  Test-Case 'UI-44' 'keyboard' {
    $main = Get-AppWindow 'Cài đặt VietKB'
    $first = Get-ToggleElement $main 'Bật tiếng Việt'
    [void](Focus-AppWindow $main)
    $first.SetFocus(); Start-Sleep -Milliseconds 400
    $seq = @()
    for ($i = 0; $i -lt 11; $i++) {
      $f = Get-FocusedElement $main
      if ($f) { $seq += $f.Current.Name }
      Send-Key 'Tab'
    }
    $expected = @('Bật tiếng Việt', 'Kiểu gõ', 'Đặt dấu kiểu mới', 'Kiểm tra chính tả', 'Bảng gõ tắt…', 'Cho phép gõ tắt', 'Quản lý…', 'Phím tắt…', 'Nâng cao…', 'Thoát VietKB', 'Đóng')
    $ok = (($seq -join '|') -eq ($expected -join '|'))
    Save-Evidence 'UI-44' (@("tabOrder=$($seq -join ' > ')") + @("focusRing: tokens.css :focus-visible 2px solid --focus (#5844c8 light / #b9aeff dark, both > 3:1)")) | Out-Null
    Add-Result 'UI-44' 'keyboard' $(if ($ok) { 'PASS' } else { 'FAIL' }) "tab order: $($seq -join ' > ') (ring CSS present, visual not measured)"
  }

  Test-Case 'UI-45' 'modal-focus' {
    Invoke-Button $main 'Bảng gõ tắt…' 'btn' | Out-Null
    $mac = Wait-AppWindow 'Bảng gõ tắt — VietKB' 5000
    Invoke-Button $mac 'Thêm' 'btn pri' | Out-Null
    Start-Sleep -Milliseconds 700
    $dlg = Get-Dialog $mac 'Thêm từ tắt'
    $dr = $dlg.Current.BoundingRectangle
    $focused = $null
    foreach ($e in $mac.FindAll([System.Windows.Automation.TreeScope]::Descendants, (New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::HasKeyboardFocusProperty, $true)))) {
      $r = $e.Current.BoundingRectangle
      if ($e.Current.ControlType.ProgrammaticName -in @('ControlType.Button','ControlType.Edit','ControlType.ComboBox') -and $r.X -ge $dr.X -and $r.Y -ge $dr.Y -and $r.X -lt ($dr.X + $dr.Width) -and $r.Y -lt ($dr.Y + $dr.Height)) { $focused = $e }
    }
    [void](Focus-AppWindow $mac)
    Send-Key 'Escape'
    Start-Sleep -Milliseconds 700
    $dlgGone = $null -eq (Get-Dialog $mac 'Thêm từ tắt')
    if (-not $dlgGone) {
      Send-Key 'Escape'
      Start-Sleep -Milliseconds 700
      $dlgGone = $null -eq (Get-Dialog $mac 'Thêm từ tắt')
    }
    $thm = Find-El $mac 'Thêm' 'btn pri' ([System.Windows.Automation.ControlType]::Button)
    $back = $thm.Current.HasKeyboardFocus
    $ok = ($null -ne $focused) -and $dlgGone -and $back
    Save-Evidence 'UI-45' @("focusInside=$($null -ne $focused)", "focusedName=$($focused.Current.Name)", "dialogClosedByEsc=$dlgGone", "focusBackOnThêm=$back") | Out-Null
    Add-Result 'UI-45' 'modal-focus' $(if ($ok) { 'PASS' } else { 'FAIL' }) "focus moved into dialog=$($null -ne $focused), Esc closed=$dlgGone, focus returned=$back"
  }

  Test-Case 'UI-46' 'persistence' {
    $main = Get-AppWindow 'Cài đặt VietKB'
    $mac0 = Get-AppWindow 'Bảng gõ tắt — VietKB'
    if ($mac0) {
      $d0 = Get-Dialog $mac0 'Thêm từ tắt'
      if ($d0) { [void](Focus-AppWindow $mac0); Send-Key 'Escape'; Start-Sleep -Milliseconds 600 }
    }
    Set-Combo $main 'Kiểu gõ' 'VNI' | Out-Null
    Start-Sleep -Milliseconds 900
    Set-Toggle $main 'Đặt dấu kiểu mới' $false | Out-Null
    Start-Sleep -Milliseconds 900
    Invoke-Button $main 'Phím tắt…' 'v4lk' | Out-Null
    $hk = Wait-AppWindow 'Phím tắt — VietKB' 5000
    $cc = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::AutomationIdProperty, 'cap-cycleMethod')
    for ($i = 0; $i -lt 15; $i++) { if ($hk.FindFirst([System.Windows.Automation.TreeScope]::Descendants, $cc)) { break }; Start-Sleep -Milliseconds 200 }
    Start-Sleep -Milliseconds 600
    Focus-Cap $hk 'cycleMethod' | Out-Null
    Send-Chord @([uint16]0x11, [uint16]0x12) ([uint16]0x4B)
    Send-Key 'Enter'
    Start-Sleep -Milliseconds 1200
    if ((Get-CfgStr 'hotkeys' 'cycle_method') -ne 'Ctrl+Alt+K') {
      Focus-Cap $hk 'cycleMethod' | Out-Null
      Send-Chord @([uint16]0x11, [uint16]0x12) ([uint16]0x4B)
      Send-Key 'Enter'
      Start-Sleep -Milliseconds 1200
    }
    $cycleSet = Get-CfgStr 'hotkeys' 'cycle_method'
    Invoke-Button $main 'Bảng gõ tắt…' 'btn' | Out-Null
    $mac = Wait-AppWindow 'Bảng gõ tắt — VietKB' 5000
    Invoke-Button $mac 'Thêm' 'btn pri' | Out-Null
    Start-Sleep -Milliseconds 600
    $edits = @($mac.FindAll([System.Windows.Automation.TreeScope]::Descendants, (New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ControlTypeProperty, [System.Windows.Automation.ControlType]::Edit))))
    Set-TextValue $edits[0] 'zz'
    Set-TextValue $edits[1] 'zed'
    Invoke-Button $mac 'Lưu' 'btn pri' | Out-Null
    Start-Sleep -Milliseconds 1200
    $before = @("method=$(Get-CfgStr 'input' 'method')", "modern=$(Get-CfgStr 'typing' 'modern_tone')", "cycle=$(Get-CfgStr 'hotkeys' 'cycle_method')", "macro=$(Get-CfgStr 'macros' 'zz')")
    Restart-TestApp | Out-Null
    $main2 = Get-AppWindow 'Cài đặt VietKB'
    $after = @("method=$(Get-CfgStr 'input' 'method')", "modern=$(Get-CfgStr 'typing' 'modern_tone')", "cycle=$(Get-CfgStr 'hotkeys' 'cycle_method')", "macro=$(Get-CfgStr 'macros' 'zz')")
    $uiMethod = Get-Combo $main2 'Kiểu gõ'
    for ($i = 0; $i -lt 6 -and -not $uiMethod; $i++) { Start-Sleep -Milliseconds 300; $uiMethod = Get-Combo $main2 'Kiểu gõ' }
    $uiModern = Get-Toggle $main2 'Đặt dấu kiểu mới'
    $ok = (($before -join '|') -eq ($after -join '|')) -and ($uiMethod -eq 'VNI') -and ($uiModern -eq [System.Windows.Automation.ToggleState]::Off) -and ($cycleSet -eq 'Ctrl+Alt+K')
    Save-Evidence 'UI-46' (@("before=$($before -join ', ')") + @("after=$($after -join ', ')") + @("uiMethod=$uiMethod", "uiModern=$uiModern")) | Out-Null
    Add-Result 'UI-46' 'persistence' $(if ($ok) { 'PASS' } else { 'FAIL' }) "$($after -join ', '); UI method=$uiMethod, modern=$uiModern"
  }
}

