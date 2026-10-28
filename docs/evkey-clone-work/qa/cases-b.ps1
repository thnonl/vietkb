# §3 UI runner — part B (secondary windows, tray, live apply, macros, rules, states)
. 'C:\Users\namth\AppData\Local\Temp\opencode\vkb-qa\lib.ps1'

function Run-GroupB {
  Reset-Config
  Restart-TestApp | Out-Null
  $main = Get-AppWindow 'Cài đặt VietKB'
  $scale = Get-Scale $main

  Test-Case 'UI-18' 'hotkeys-window' {
    Invoke-Button $main 'Phím tắt…' 'v4lk' | Out-Null
    $hk = Wait-AppWindow 'Phím tắt — VietKB' 5000
    $r = $hk.Current.BoundingRectangle
    $lw = [math]::Round($r.Width / (Get-Scale $hk)); $lh = [math]::Round($r.Height / (Get-Scale $hk))
    $caps = @()
    foreach ($b in Get-AllButtons $hk) { if ($b.Current.AutomationId -like 'cap-*') { $caps += "$($b.Current.AutomationId)=$($b.Current.Name)" } }
    $seg = @(Find-Els $hk 'Khi nhả phím' '' ([System.Windows.Automation.ControlType]::Button)).Count
    $ok = ($lw -eq 520 -and $lh -eq 460 -and $caps.Count -eq 4 -and $seg -eq 1)
    Save-Evidence 'UI-18' (@("logical=${lw}x${lh}") + $caps + @("segmented=$seg")) | Out-Null
    Add-Result 'UI-18' 'hotkeys-window' $(if ($ok) { 'PASS' } else { 'FAIL' }) "${lw}x${lh}, $($caps.Count) capture rows, segmented mode present=$($seg -eq 1)"
  }

  Test-Case 'UI-19' 'advanced-window' {
    $main = Get-AppWindow 'Cài đặt VietKB'
    Invoke-Button $main 'Nâng cao…' 'v4lk' | Out-Null
    $adv = Wait-AppWindow 'Nâng cao — VietKB' 5000
    $r = $adv.Current.BoundingRectangle
    $lw = [math]::Round($r.Width / (Get-Scale $adv)); $lh = [math]::Round($r.Height / (Get-Scale $adv))
    $cbs = @(Find-Els $adv '' '' ([System.Windows.Automation.ControlType]::CheckBox)).Count
    $combos = @(Find-Els $adv '' '' ([System.Windows.Automation.ControlType]::ComboBox)).Count
    $numb = $null
    foreach ($e in $adv.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)) {
      if ($e.Current.ControlType.ProgrammaticName -in @('ControlType.Edit','ControlType.Spinner')) { $numb = $e }
    }
    $sections = @(Find-Els $adv 'Gõ' '' ([System.Windows.Automation.ControlType]::Text)).Count + @(Find-Els $adv 'Tương thích' '' ([System.Windows.Automation.ControlType]::Text)).Count + @(Find-Els $adv 'Hệ thống' '' ([System.Windows.Automation.ControlType]::Text)).Count
    $ok = ($lw -eq 560 -and $lh -eq 496 -and $cbs -eq 15 -and $combos -eq 3 -and $numb -and $sections -eq 3)
    Save-Evidence 'UI-19' @("logical=${lw}x${lh}", "checkboxes=$cbs", "combos=$combos", "numberEdit=$($null -ne $numb)", "sections=$sections") | Out-Null
    Add-Result 'UI-19' 'advanced-window' $(if ($ok) { 'PASS' } else { 'FAIL' }) "${lw}x${lh}, checkboxes=$cbs, selects=$combos, delay input=$($null -ne $numb), sections=$sections"
  }

  Test-Case 'UI-20' 'macro-window' {
    $main = Get-AppWindow 'Cài đặt VietKB'
    Invoke-Button $main 'Bảng gõ tắt…' 'btn' | Out-Null
    $mac = Wait-AppWindow 'Bảng gõ tắt — VietKB' 5000
    $r = $mac.Current.BoundingRectangle
    $lw = [math]::Round($r.Width / (Get-Scale $mac)); $lh = [math]::Round($r.Height / (Get-Scale $mac))
    $search = $null
    foreach ($e in $mac.FindAll([System.Windows.Automation.TreeScope]::Descendants, (New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ControlTypeProperty, [System.Windows.Automation.ControlType]::Edit)))) { if ($e.Current.Name -match 'Tìm từ tắt') { $search = $e } }
    $btns = @(); foreach ($b in Get-AllButtons $mac) { if ($b.Current.Name -in @('Nhập từ tệp…', 'Xuất ra tệp…', 'Thêm')) { $btns += $b.Current.Name } }
    $rows = @(Get-AllButtons $mac | Where-Object { $_.Current.Name -like 'Sửa *' })
    $ok = ($lw -eq 720 -and $lh -eq 560 -and $search -and $btns.Count -eq 3 -and $rows.Count -ge 1)
    Save-Evidence 'UI-20' @("logical=${lw}x${lh}", "search=$($null -ne $search)", "toolbar=$($btns -join ',')", "rows=$($rows.Count)") | Out-Null
    Add-Result 'UI-20' 'macro-window' $(if ($ok) { 'PASS' } else { 'FAIL' }) "${lw}x${lh}, search=$($null -ne $search), toolbar=$($btns.Count)/3, rows=$($rows.Count)"
  }

  Test-Case 'UI-21' 'window-independence' {
    $main = Get-AppWindow 'Cài đặt VietKB'
    $hk = Get-AppWindow 'Phím tắt — VietKB'; $adv = Get-AppWindow 'Nâng cao — VietKB'; $mac = Get-AppWindow 'Bảng gõ tắt — VietKB'
    $allOpen = $hk -and $adv -and $mac
    $close = Find-El $hk 'Đóng' 'btn pri' ([System.Windows.Automation.ControlType]::Button)
    Invoke-El $close
    Start-Sleep -Milliseconds 700
    $hkGone = $null -eq (Get-AppWindow 'Phím tắt — VietKB')
    $othersStay = ($null -ne (Get-AppWindow 'Nâng cao — VietKB')) -and ($null -ne (Get-AppWindow 'Bảng gõ tắt — VietKB')) -and ($null -ne (Get-AppWindow 'Cài đặt VietKB'))
    $main2 = Get-AppWindow 'Cài đặt VietKB'
    Invoke-Button $main2 'Đóng, VietKB vẫn chạy' 'wbtn x' | Out-Null
    Start-Sleep -Milliseconds 900
    $mainGone = $null -eq (Get-AppWindow 'Cài đặt VietKB')
    $secStay = ($null -ne (Get-AppWindow 'Nâng cao — VietKB')) -and ($null -ne (Get-AppWindow 'Bảng gõ tắt — VietKB'))
    $alive = $null -ne (Get-Process vietkb -ErrorAction SilentlyContinue)
    $ok = $allOpen -and $hkGone -and $othersStay -and $mainGone -and $secStay -and $alive
    Save-Evidence 'UI-21' @("allOpen=$allOpen", "hotkeysClosed=$hkGone", "othersStay=$othersStay", "mainClosed=$mainGone", "secondariesStay=$secStay", "processAlive=$alive") | Out-Null
    Add-Result 'UI-21' 'window-independence' $(if ($ok) { 'PASS' } else { 'FAIL' }) "hotkeys closes alone=$hkGone; main closes with secondaries alive=$($mainGone -and $secStay); process alive=$alive"
  }

  Test-Case 'UI-22' 'reopen' {
    $icon = Get-TrayIconElement
    $mh = Open-TrayMenu
    if ($mh -ne [IntPtr]::Zero) { [void](Invoke-MenuItem $mh 'Mở cài đặt…') }
    Start-Sleep -Milliseconds 1200
    $main = Get-AppWindow 'Cài đặt VietKB'
    $advOpen = $null -ne (Get-AppWindow 'Nâng cao — VietKB')
    $adv = Get-AppWindow 'Nâng cao — VietKB'
    if ($adv) { Invoke-El (Find-El $adv 'Đóng' 'btn pri' ([System.Windows.Automation.ControlType]::Button)); Start-Sleep -Milliseconds 600 }
    Invoke-Button $main 'Nâng cao…' 'v4lk' | Out-Null
    $reopened = $null -ne (Wait-AppWindow 'Nâng cao — VietKB' 4000)
    $ok = $main -and $advOpen -and $reopened
    Save-Evidence 'UI-22' @("mainReopened=$($null -ne $main)", "advancedWasOpen=$advOpen", "advancedReopened=$reopened") | Out-Null
    Add-Result 'UI-22' 'reopen' $(if ($ok) { 'PASS' } else { 'FAIL' }) "main from tray=$($null -ne $main), closed secondary reopens=$reopened"
  }
}

function Run-GroupC {
  Reset-Config
  Restart-TestApp | Out-Null
  $main = Get-AppWindow 'Cài đặt VietKB'

  Test-Case 'UI-23' 'tray-menu' {
    $mh = Open-TrayMenu
    $lines = Get-MenuItemLines $mh
    $names = @(); foreach ($l in $lines) { $p = $l -split '\|'; if ($p[1] -ne '') { $names += $p[1] } }
    $expected = @('Context', 'Bật/tắt tiếng Việt', 'Kiểu gõ', 'Bảng mã', 'Mở cài đặt…', 'Phím tắt…', 'Thoát VietKB')
    $seps = @($lines | Where-Object { ($_ -split '\|')[2] -eq '21' }).Count
    $okTop = (($names -join '|') -eq ($expected -join '|')) -and $seps -eq 3
    [void](Invoke-MenuItem $mh 'Kiểu gõ')
    $methodPopup = $null
    $deadline = (Get-Date).AddMilliseconds(2500)
    do {
      foreach ($p in Get-MenuPopups) {
        $ls = Get-MenuItemLines ([IntPtr]$p.Hwnd)
        if (@($ls | Where-Object { $_ -match '\|Telex\|' }).Count -gt 0) { $methodPopup = $p }
      }
      if (-not $methodPopup) { Start-Sleep -Milliseconds 200 }
    } while (-not $methodPopup -and (Get-Date) -lt $deadline)
    $methodItems = @(); if ($methodPopup) { foreach ($l in (Get-MenuItemLines ([IntPtr]$methodPopup.Hwnd))) { $x = $l -split '\|'; if ($x[1] -ne '' -and $x[1] -ne 'Kiểu gõ' -and $x[1] -ne 'null') { $methodItems += "$($x[1])=$($x[3])" } } }
    $mh2 = Open-TrayMenu
    [void](Invoke-MenuItem $mh2 'Bảng mã')
    $charsetPopup = $null
    $deadline = (Get-Date).AddMilliseconds(2500)
    do {
      foreach ($p in Get-MenuPopups) {
        $ls = Get-MenuItemLines ([IntPtr]$p.Hwnd)
        if (@($ls | Where-Object { $_ -match '\|Unicode dựng sẵn\|' }).Count -gt 0) { $charsetPopup = $p }
      }
      if (-not $charsetPopup) { Start-Sleep -Milliseconds 200 }
    } while (-not $charsetPopup -and (Get-Date) -lt $deadline)
    $charsetItems = @(); if ($charsetPopup) { foreach ($l in (Get-MenuItemLines ([IntPtr]$charsetPopup.Hwnd))) { $x = $l -split '\|'; if ($x[1] -ne '' -and $x[1] -ne 'Bảng mã' -and $x[1] -ne 'null') { $charsetItems += "$($x[1])=$($x[3])" } } }
    $ok = $okTop -and ($methodItems.Count -eq 2) -and ($charsetItems.Count -eq 6)
    Save-Evidence 'UI-23' (@("top=$($names -join ' | ')", "separators=$seps", "methods=$($methodItems -join ',')", "charsets=$($charsetItems -join ',')")) | Out-Null
    Add-Result 'UI-23' 'tray-menu' $(if ($ok) { 'PASS' } else { 'FAIL' }) "items=$($names -join ', '); seps=$seps; methods=$($methodItems.Count); charsets=$($charsetItems.Count)"
    Close-TrayMenu ([IntPtr]::Zero)
  }

  Test-Case 'UI-24' 'tray-toggle' {
    $main = Get-AppWindow 'Cài đặt VietKB'
    $before = Get-CfgStr 'input' 'enabled'
    $mh = Open-TrayMenu
    [void](Invoke-MenuItem $mh 'Bật/tắt tiếng Việt')
    Start-Sleep -Milliseconds 900
    if ((Get-CfgStr 'input' 'enabled') -eq $before) {
      $mh = Open-TrayMenu
      [void](Invoke-MenuItem $mh 'Bật/tắt tiếng Việt')
    }
    $deadline = (Get-Date).AddMilliseconds(3000)
    $uiMatches = $false
    do {
      $after = Get-CfgStr 'input' 'enabled'
      $tg = Get-ToggleElement $main 'Bật tiếng Việt'
      if ($tg) {
        $ui = $tg.GetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern).Current.ToggleState
        $uiMatches = (($ui -eq [System.Windows.Automation.ToggleState]::On) -eq ($after -eq 'true'))
      }
      if (-not $uiMatches) { Start-Sleep -Milliseconds 200 }
    } while (-not $uiMatches -and (Get-Date) -lt $deadline)
    $after = Get-CfgStr 'input' 'enabled'
    $ui = (Get-ToggleElement $main 'Bật tiếng Việt').GetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern).Current.ToggleState
    $icon = Get-TrayIconElement
    $tooltip = $icon.Current.Name
    $ok = ($before -ne $after) -and $uiMatches -and ($tooltip -match 'VietKB ·')
    Save-Evidence 'UI-24' @("before=$before", "after=$after", "uiToggle=$ui", "tooltip=$tooltip") | Out-Null
    Add-Result 'UI-24' 'tray-toggle' $(if ($ok) { 'PASS' } else { 'FAIL' }) "enabled $before->$after, UI=$ui, tooltip='$tooltip'"
  }

  Test-Case 'UI-25' 'tray-method' {
    if ((Get-CfgStr 'input' 'enabled') -ne 'true') {
      $mh0 = Open-TrayMenu; [void](Invoke-MenuItem $mh0 'Bật/tắt tiếng Việt'); Start-Sleep -Milliseconds 1000
    }
    $pop = $null
    for ($attempt = 0; $attempt -lt 2 -and -not $pop; $attempt++) {
      $mh = Open-TrayMenu
      $top = @(); foreach ($l in (Get-MenuItemLines $mh)) { $p = $l -split '\|'; if ($p[1] -ne '') { $top += $p[1] } }
      if ($top -notcontains 'Kiểu gõ') { break }
      [void](Invoke-MenuItem $mh 'Kiểu gõ')
      $deadline = (Get-Date).AddMilliseconds(2500)
      do {
        foreach ($p in Get-MenuPopups) {
          $ls = Get-MenuItemLines ([IntPtr]$p.Hwnd)
          if (@($ls | Where-Object { $_ -match '\|VNI\|' }).Count -gt 0) { $pop = $p }
        }
        if (-not $pop) { Start-Sleep -Milliseconds 200 }
      } while (-not $pop -and (Get-Date) -lt $deadline)
    }
    if (-not $pop) {
      Add-Result 'UI-25' 'tray-method' 'FAIL' "Kiểu gõ submenu did not open (no popup with Telex/VNI/VIQR)"
    } else {
      $telexState = Get-MenuItemState ([IntPtr]$pop.Hwnd) 'Telex'
      [void](Invoke-MenuItem ([IntPtr]$pop.Hwnd) 'VNI')
      Start-Sleep -Milliseconds 1200
      $cfg = Get-CfgStr 'input' 'method'
      $main = Get-AppWindow 'Cài đặt VietKB'
      $ui = Get-Combo $main 'Kiểu gõ'
      $tooltip = (Get-TrayIconElement).Current.Name
      $ok = ($telexState -eq '16') -and ($cfg -eq 'vni') -and ($ui -eq 'VNI') -and ($tooltip -match 'VNI')
      Save-Evidence 'UI-25' @("telexState=$telexState", "config=$cfg", "mainCombo=$ui", "tooltip=$tooltip") | Out-Null
      Add-Result 'UI-25' 'tray-method' $(if ($ok) { 'PASS' } else { 'FAIL' }) "Telex checked=$($telexState -eq '16'), config=$cfg, UI=$ui, tooltip='$tooltip'"
    }
  }

  Test-Case 'UI-26' 'tray-charset' {
    $mh = Open-TrayMenu
    $top = @(); foreach ($l in (Get-MenuItemLines $mh)) { $p = $l -split '\|'; if ($p[1] -ne '') { $top += $p[1] } }
    if ($top -notcontains 'Bảng mã') {
      Add-Result 'UI-26' 'tray-charset' 'FAIL' "Bảng mã submenu absent from tray menu (items: $($top -join ', '))"
    } else {
      [void](Invoke-MenuItem $mh 'Bảng mã')
      $pop = $null
      foreach ($p in Get-MenuPopups) { $ls = Get-MenuItemLines ([IntPtr]$p.Hwnd); if (@($ls | Where-Object { $_ -match '\|Unicode dựng sẵn\|' }).Count -gt 0) { $pop = $p } }
      if (-not $pop) {
        Add-Result 'UI-26' 'tray-charset' 'FAIL' 'Bảng mã submenu did not open'
      } else {
        $unicodeState = Get-MenuItemState ([IntPtr]$pop.Hwnd) 'Unicode dựng sẵn'
        [void](Invoke-MenuItem ([IntPtr]$pop.Hwnd) 'VISCII')
        Start-Sleep -Milliseconds 1200
        $cfg = Get-CfgStr 'input' 'charset'
        $tooltip = (Get-TrayIconElement).Current.Name
        $ok = ($unicodeState -eq '16') -and ($cfg -eq 'viscii') -and ($tooltip -match 'VISCII')
        Save-Evidence 'UI-26' @("unicodeState=$unicodeState", "config=$cfg", "tooltip=$tooltip") | Out-Null
        Add-Result 'UI-26' 'tray-charset' $(if ($ok) { 'PASS' } else { 'FAIL' }) "Unicode checked=$($unicodeState -eq '16'), config=$cfg, tooltip='$tooltip'"
      }
    }
  }

  Test-Case 'UI-27' 'tray-open' {
    $hk = Get-AppWindow 'Phím tắt — VietKB'
    if ($hk) { Invoke-El (Find-El $hk 'Đóng' 'btn pri' ([System.Windows.Automation.ControlType]::Button)); Start-Sleep -Milliseconds 600 }
    $mh = Open-TrayMenu
    [void](Invoke-MenuItem $mh 'Phím tắt…')
    $hk2 = Wait-AppWindow 'Phím tắt — VietKB' 5000
    $main = Get-AppWindow 'Cài đặt VietKB'
    if ($main) { Invoke-Button $main 'Đóng, VietKB vẫn chạy' 'wbtn x' | Out-Null; Start-Sleep -Milliseconds 800 }
    $mh2 = Open-TrayMenu
    [void](Invoke-MenuItem $mh2 'Mở cài đặt…')
    $main2 = Wait-AppWindow 'Cài đặt VietKB' 5000
    $ok = $hk2 -and $main2
    Save-Evidence 'UI-27' @("hotkeysOpened=$($null -ne $hk2)", "settingsOpened=$($null -ne $main2)") | Out-Null
    Add-Result 'UI-27' 'tray-open' $(if ($ok) { 'PASS' } else { 'FAIL' }) "Phím tắt… open=$($null -ne $hk2), Mở cài đặt… open=$($null -ne $main2)"
  }

  Test-Case 'UI-28' 'tray-quit' {
    $mh = Open-TrayMenu
    $okInvoke = [QaMenu]::DoDefault($mh, 'Thoát VietKB')
    $deadline = (Get-Date).AddSeconds(4)
    do { $alive = $null -ne (Get-Process vietkb -ErrorAction SilentlyContinue); if (-not $alive) { break }; Start-Sleep -Milliseconds 200 } while ((Get-Date) -lt $deadline)
    $ok = $okInvoke -and (-not $alive)
    Save-Evidence 'UI-28' @("invoked=$okInvoke", "processAliveAfter=$alive") | Out-Null
    Add-Result 'UI-28' 'tray-quit' $(if ($ok) { 'PASS' } else { 'FAIL' }) "quit invoked=$okInvoke, process exited=$(-not $alive)"
  }
}
