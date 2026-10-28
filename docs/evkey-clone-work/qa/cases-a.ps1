# §3 UI runner — part A (main window + secondary windows)
. 'C:\Users\namth\AppData\Local\Temp\opencode\vkb-qa\lib.ps1'

function Test-Case([string]$Id, [string]$Area, [scriptblock]$Body) {
  try { & $Body }
  catch { Add-Result $Id $Area 'FAIL' ("exception: " + $_.Exception.Message) }
}

function Reset-Config {
  Copy-Item 'C:\Users\namth\AppData\Local\Temp\opencode\vkb-qa\config-before.toml' $script:CfgPath -Force
}

function Run-GroupA {
  Reset-Config
  Restart-TestApp | Out-Null
  $main = Get-AppWindow 'Cài đặt VietKB'
  $scale = Get-Scale $main
  $h = [IntPtr]$main.Current.NativeWindowHandle
  $wr = New-Object QaWin+RECT
  $cr = New-Object QaWin+RECT
  [void][QaWin]::GetWindowRect($h, [ref]$wr)
  [void][QaWin]::GetClientRect($h, [ref]$cr)
  $style = [QaWin]::GetWindowLong($h, -16)
  $logicalW = [math]::Round($main.Current.BoundingRectangle.Width / $scale)
  $logicalH = [math]::Round($main.Current.BoundingRectangle.Height / $scale)

  Test-Case 'UI-1' 'main-window' {
    $frame = (($wr.Right - $wr.Left) - ($cr.Right - $cr.Left)) -or (($wr.Bottom - $wr.Top) - ($cr.Bottom - $cr.Top))
    $ok = ($logicalW -eq 480 -and $logicalH -eq 384 -and -not $frame -and ($style -band 0x00040000) -eq 0 -and ($style -band 0x00010000) -eq 0)
    Save-Evidence 'UI-1' @("uia=$($main.Current.BoundingRectangle)", "scale=$scale", "logical=${logicalW}x${logicalH}", "client=$($cr.Right-$cr.Left)x$($cr.Bottom-$cr.Top)", "style=0x$($style.ToString('X8'))", "thick=$([bool]($style -band 0x00040000)) max=$([bool]($style -band 0x00010000)) framePx=$frame") | Out-Null
    Add-Result 'UI-1' 'main-window' $(if ($ok) { 'PASS' } else { 'FAIL' }) "logical ${logicalW}x${logicalH} @${scale}x, framePx=$frame, thick=$([bool]($style -band 0x00040000)) max=$([bool]($style -band 0x00010000))"
  }

  Test-Case 'UI-2' 'title-bar' {
    $cap = Find-El $main '' 'cap-bar' ([System.Windows.Automation.ControlType]::Group)
    $img = Find-El $main 'VietKB' '' ([System.Windows.Automation.ControlType]::Image)
    $ttl = Find-El $main 'VietKB' '' ([System.Windows.Automation.ControlType]::Text)
    $thm = Find-El $main '' 'tslot' ([System.Windows.Automation.ControlType]::Button)
    $min = Find-El $main 'Thu nhỏ' 'wbtn' ([System.Windows.Automation.ControlType]::Button)
    $cls = Find-El $main 'Đóng, VietKB vẫn chạy' 'wbtn x' ([System.Windows.Automation.ControlType]::Button)
    $names = @($img, $ttl, $thm, $min, $cls) | ForEach-Object { $_.Current.Name }
    $order = @($img, $ttl, $thm, $min, $cls) | ForEach-Object { [int]$_.Current.BoundingRectangle.X }
    $ordered = ($order[0] -lt $order[1] -and $order[1] -lt $order[2] -and $order[2] -lt $order[3] -and $order[3] -lt $order[4])
    $ok = $cap -and $img -and $ttl -and $thm -and $min -and $cls -and $ordered -and $thm.Current.Name -match '^Giao diện:'
    Save-Evidence 'UI-2' @("cap=$($null -ne $cap)", "names=$($names -join ' | ')", "x=$($order -join ',')", "ordered=$ordered") | Out-Null
    Add-Result 'UI-2' 'title-bar' $(if ($ok) { 'PASS' } else { 'FAIL' }) "logo+VietKB+theme+min+close, ordered=$ordered, theme='$($thm.Current.Name)'"
  }

  Test-Case 'UI-3' 'title-bar' {
    # The drag surface is the caption strip minus the controls; Tauri startDragging is
    # wired in TitleBar.svelte. Exercising it would move the window, which the run forbids.
    $cap = Find-El $main '' 'cap-bar' ([System.Windows.Automation.ControlType]::Group)
    $capH = $cap.Current.BoundingRectangle.Height / $scale
    $ok = $cap -and [math]::Abs($capH - 36) -lt 2
    Save-Evidence 'UI-3' @("cap class=cap-bar", "logical height=$capH (expected 36)", "drag handler: TitleBar.svelte onmousedown -> win.startDragging()", "NOT exercised: window must not move") | Out-Null
    Add-Result 'UI-3' 'title-bar' $(if ($ok) { 'PASS' } else { 'FAIL' }) "36 px drag strip present (handler verified in source); actual drag not exercised (no window move allowed)"
  }

  Test-Case 'UI-4' 'theme' {
    $seq = @()
    for ($i = 0; $i -lt 3; $i++) {
      $tbtn = Find-El $main '' 'tslot' ([System.Windows.Automation.ControlType]::Button)
      Invoke-El $tbtn
      Start-Sleep -Milliseconds 500
      $tbtn = Find-El $main '' 'tslot' ([System.Windows.Automation.ControlType]::Button)
      $seq += "$($tbtn.Current.Name)|$(Get-CfgStr 'system' 'theme')"
    }
    $ok = ($seq[0] -match 'Sáng.*\|light') -and ($seq[1] -match 'Tối.*\|dark') -and ($seq[2] -match 'Theo Windows.*\|system')
    Save-Evidence 'UI-4' ($seq + @("config=$(Get-CfgStr 'system' 'theme')")) | Out-Null
    Add-Result 'UI-4' 'theme' $(if ($ok) { 'PASS' } else { 'FAIL' }) ($seq -join ' -> ')
  }

  Test-Case 'UI-5' 'window' {
    $wp = $main.GetCurrentPattern([System.Windows.Automation.WindowPattern]::Pattern)
    Invoke-Button $main 'Thu nhỏ' 'wbtn' | Out-Null
    Start-Sleep -Milliseconds 800
    $min = $wp.Current.WindowVisualState
    $alive = $null -ne (Get-Process vietkb -ErrorAction SilentlyContinue)
    $wp.SetWindowVisualState([System.Windows.Automation.WindowVisualState]::Normal)
    Start-Sleep -Milliseconds 900
    $main = Get-AppWindow 'Cài đặt VietKB'
    $normal = $main.GetCurrentPattern([System.Windows.Automation.WindowPattern]::Pattern).Current.WindowVisualState
    $ok = ($min -eq [System.Windows.Automation.WindowVisualState]::Minimized) -and $alive -and ($normal -eq [System.Windows.Automation.WindowVisualState]::Normal)
    Save-Evidence 'UI-5' @("minimized=$min", "processAlive=$alive", "restored=$normal") | Out-Null
    Add-Result 'UI-5' 'window' $(if ($ok) { 'PASS' } else { 'FAIL' }) "minimize -> $min, process alive=$alive, restore -> $normal"
  }

  Test-Case 'UI-6' 'window' {
    $main = Get-AppWindow 'Cài đặt VietKB'
    Invoke-Button $main 'Đóng, VietKB vẫn chạy' 'wbtn x' | Out-Null
    Start-Sleep -Milliseconds 1000
    $gone = $null -eq (Get-AppWindow 'Cài đặt VietKB')
    $alive = $null -ne (Get-Process vietkb -ErrorAction SilentlyContinue)
    $icon = Get-TrayIconElement
    $reopened = $false
    if ($icon) {
      $mh = Open-TrayMenu
      if ($mh -ne [IntPtr]::Zero) { [void](Invoke-MenuItem $mh 'Mở cài đặt…') }
      Start-Sleep -Milliseconds 1200
      $reopened = $null -ne (Get-AppWindow 'Cài đặt VietKB')
    }
    $ok = $gone -and $alive -and $icon -and $reopened
    Save-Evidence 'UI-6' @("mainAfterClose=$(-not $gone)", "processAlive=$alive", "trayIcon=$($icon.Current.Name)", "reopenedFromTray=$reopened") | Out-Null
    Add-Result 'UI-6' 'window' $(if ($ok) { 'PASS' } else { 'FAIL' }) "X closes window, process alive=$alive, tray='$($icon.Current.Name)', reopen=$reopened"
  }

  Test-Case 'UI-7' 'layout' {
    $main = Get-AppWindow 'Cài đặt VietKB'
    $caption = Find-El $main '' 'cap-bar' ([System.Windows.Automation.ControlType]::Group)
    $capBottom = if ($caption) { $caption.Current.BoundingRectangle.Bottom } else { 0 }
    $controls = @()
    foreach ($b in Get-AllButtons $main) {
      $cls = $b.Current.ClassName
      if ($cls -notmatch '(^|\s)(tg|btn|v4lk)(\s|$)') { continue }
      if ($cls -match '(^|\s)tslot(\s|$)') { continue }
      if ($b.Current.BoundingRectangle.Y -lt $capBottom) { continue }
      $controls += $b.Current.Name
    }
    foreach ($cb in $main.FindAll([System.Windows.Automation.TreeScope]::Descendants, (New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ControlTypeProperty, [System.Windows.Automation.ControlType]::ComboBox)))) { $controls += $cb.Current.Name }
    $ok = ($controls.Count -le 12)
    Save-Evidence 'UI-7' @("count=$($controls.Count)", "limit=12", ($controls -join ' | ')) | Out-Null
    Add-Result 'UI-7' 'layout' $(if ($ok) { 'PASS' } else { 'FAIL' }) "$($controls.Count)/12 content controls: $($controls -join ', ')"
  }

  Test-Case 'UI-8' 'layout' {
    $main = Get-AppWindow 'Cài đặt VietKB'
    $bars = Get-ScrollBars $main
    $doc = Find-El $main '' '' ([System.Windows.Automation.ControlType]::Document)
    $vScroll = $null
    try { $vScroll = $doc.GetCurrentPattern([System.Windows.Automation.ScrollPattern]::Pattern).Current.VerticallyScrollable } catch {}
    $rect = $main.Current.BoundingRectangle
    $outside = @()
    $sheet = Find-El $main '' 'v4sheet' ([System.Windows.Automation.ControlType]::Group)
    foreach ($b in Get-AllButtons $sheet) {
      $r = $b.Current.BoundingRectangle
      if ($r.Bottom -gt $rect.Bottom -or $r.Top -lt $rect.Top -or $r.Right -gt $rect.Right) { $outside += $b.Current.Name }
    }
    $ok = ($bars.Count -eq 0) -and (-not $vScroll) -and ($outside.Count -eq 0)
    Save-Evidence 'UI-8' @("scrollbars=$($bars.Count)", "documentVerticallyScrollable=$vScroll", "controlsOutsideClient=$($outside -join ',')", "scale=$scale", "100%/150%: display scaling not changed") | Out-Null
    Add-Result 'UI-8' 'layout' $(if ($ok) { 'PASS' } else { 'FAIL' }) "no scrollbar=$($bars.Count -eq 0), doc scrollable=$vScroll, clipped=$($outside.Count) @${scale}x; 100/150% not exercised"
  }

  Test-Case 'UI-9' 'enable-row' {
    $main = Get-AppWindow 'Cài đặt VietKB'
    $hk = Find-El $main 'Ctrl + Shift' '' ([System.Windows.Automation.ControlType]::Text)
    $tg = Get-ToggleElement $main 'Bật tiếng Việt'
    $before = $tg.GetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern).Current.ToggleState
    $target = -not ($before -eq [System.Windows.Automation.ToggleState]::On)
    Set-Toggle $main 'Bật tiếng Việt' $target | Out-Null
    $persisted = Wait-Cfg 'input' 'enabled' $(if ($target) { 'true' } else { 'false' }) 2500
    $after = (Get-ToggleElement $main 'Bật tiếng Việt').GetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern).Current.ToggleState
    Set-Toggle $main 'Bật tiếng Việt' ($before -eq [System.Windows.Automation.ToggleState]::On) | Out-Null
    $ok = $hk -and ($after -eq [System.Windows.Automation.ToggleState]::On) -and $persisted
    Save-Evidence 'UI-9' @("hotkeyLabel=$($hk.Current.Name)", "before=$before", "after=$after", "persisted=$persisted") | Out-Null
    Add-Result 'UI-9' 'enable-row' $(if ($ok) { 'PASS' } else { 'FAIL' }) "hotkey label '$($hk.Current.Name)', toggle $before->$after, persisted=$persisted"
  }

  Test-Case 'UI-10' 'method' {
    $main = Get-AppWindow 'Cài đặt VietKB'
    $opts = Get-ComboOptions $main 'Kiểu gõ'
    $before = Get-Combo $main 'Kiểu gõ'
    Set-Combo $main 'Kiểu gõ' 'VNI' | Out-Null
    $persisted = Wait-Cfg 'input' 'method' '"vni"' 2500
    Set-Combo $main 'Kiểu gõ' 'Telex' | Out-Null
    $ok = ($opts.Count -eq 2) -and $persisted
    Save-Evidence 'UI-10' @("options=$($opts -join ' | ')", "before=$before", "persisted=$persisted", "VIQR absent from UI (catalog has Telex, VNI only)") | Out-Null
    Add-Result 'UI-10' 'method' $(if ($ok) { 'PASS' } else { 'FAIL' }) "options=$($opts.Count) [$($opts -join ', ')], change persisted=$persisted"
  }

  Test-Case 'UI-11' 'charset' {
    $main = Get-AppWindow 'Cài đặt VietKB'
    $sel = Find-El $main 'Bảng mã' 'sel' ([System.Windows.Automation.ControlType]::ComboBox)
    $label = Find-El $main 'Bảng mã' '' ([System.Windows.Automation.ControlType]::Text)
    $ok = $null -ne $sel
    Save-Evidence 'UI-11' @("labelPresent=$($null -ne $label)", "selectPresent=$($null -ne $sel)", "tray still has Bảng mã submenu") | Out-Null
    Add-Result 'UI-11' 'charset' $(if ($ok) { 'PASS' } else { 'FAIL' }) "Bảng mã select present=$($null -ne $sel) (label present=$($null -ne $label))"
  }

  Test-Case 'UI-12' 'modern-tone' {
    $main = Get-AppWindow 'Cài đặt VietKB'
    $hint = Find-El $main 'hòa, thúy' '' ([System.Windows.Automation.ControlType]::Text)
    $before = Get-Toggle $main 'Đặt dấu kiểu mới'
    $target = -not ($before -eq [System.Windows.Automation.ToggleState]::On)
    Set-Toggle $main 'Đặt dấu kiểu mới' $target | Out-Null
    $persisted = Wait-Cfg 'typing' 'modern_tone' $(if ($target) { 'true' } else { 'false' }) 2500
    Set-Toggle $main 'Đặt dấu kiểu mới' ($before -eq [System.Windows.Automation.ToggleState]::On) | Out-Null
    $ok = $hint -and $persisted
    Save-Evidence 'UI-12' @("hint=$($hint.Current.Name)", "persisted=$persisted") | Out-Null
    Add-Result 'UI-12' 'modern-tone' $(if ($ok) { 'PASS' } else { 'FAIL' }) "hint '$($hint.Current.Name)', toggle persisted=$persisted"
  }

  Test-Case 'UI-13' 'spell-check' {
    $main = Get-AppWindow 'Cài đặt VietKB'
    $before = Get-Toggle $main 'Kiểm tra chính tả'
    $target = -not ($before -eq [System.Windows.Automation.ToggleState]::On)
    Set-Toggle $main 'Kiểm tra chính tả' $target | Out-Null
    $persisted = Wait-Cfg 'typing' 'spell_check' $(if ($target) { 'true' } else { 'false' }) 2500
    Set-Toggle $main 'Kiểm tra chính tả' ($before -eq [System.Windows.Automation.ToggleState]::On) | Out-Null
    $ok = $persisted
    Save-Evidence 'UI-13' @("persisted=$persisted") | Out-Null
    Add-Result 'UI-13' 'spell-check' $(if ($ok) { 'PASS' } else { 'FAIL' }) "toggle persisted=$persisted"
  }

  Test-Case 'UI-14' 'macros-entry' {
    $main = Get-AppWindow 'Cài đặt VietKB'
    $tg = Get-ToggleElement $main 'Cho phép gõ tắt'
    $before = $tg.GetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern).Current.ToggleState
    Set-Toggle $main 'Cho phép gõ tắt' (-not ($before -eq [System.Windows.Automation.ToggleState]::On)) | Out-Null
    $persisted = Wait-Cfg 'typing' 'macro_enabled' $(if ($before -eq [System.Windows.Automation.ToggleState]::On) { 'false' } else { 'true' }) 2500
    Set-Toggle $main 'Cho phép gõ tắt' ($before -eq [System.Windows.Automation.ToggleState]::On) | Out-Null
    Invoke-Button $main 'Bảng gõ tắt…' 'btn' | Out-Null
    $mac = Wait-AppWindow 'Bảng gõ tắt — VietKB' 5000
    $ok = $persisted -and $mac
    Save-Evidence 'UI-14' @("persisted=$persisted", "macroWindow=$($null -ne $mac)") | Out-Null
    Add-Result 'UI-14' 'macros-entry' $(if ($ok) { 'PASS' } else { 'FAIL' }) "macro toggle persisted=$persisted, Bảng gõ tắt… opens=$($null -ne $mac)"
    if ($mac) {
      $close = Find-El $mac 'Đóng' 'btn ghost sm' ([System.Windows.Automation.ControlType]::Button)
      if ($close) { Invoke-El $close }
      Start-Sleep -Milliseconds 500
    }
  }

  Test-Case 'UI-15' 'app-rules-entry' {
    $main = Get-AppWindow 'Cài đặt VietKB'
    $countEl = $null
    foreach ($t in $main.FindAll([System.Windows.Automation.TreeScope]::Descendants, (New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ControlTypeProperty, [System.Windows.Automation.ControlType]::Text)))) {
      $r = $t.Current.BoundingRectangle
      if ($t.Current.Name -match '^\d+$' -and $r.Y -gt 800 -and $r.Y -lt 870) { $countEl = $t }
    }
    $expected = ([regex]::Matches((Get-ConfigRaw), '(?m)^\s*process\s*=\s*"')).Count
    $shown = $countEl.Current.Name
    Invoke-Button $main 'Quản lý…' 'btn' | Out-Null
    Start-Sleep -Milliseconds 800
    $dlg = Get-Dialog $main 'Loại trừ ứng dụng'
    $ok = $dlg -and ($shown -eq [string]$expected)
    Save-Evidence 'UI-15' @("configRules=$expected", "shownCount=$shown", "dialogOpen=$($null -ne $dlg)") | Out-Null
    Add-Result 'UI-15' 'app-rules-entry' $(if ($ok) { 'PASS' } else { 'FAIL' }) "count shown '$shown' vs config $expected, dialog opens=$($null -ne $dlg)"
    if ($dlg) {
      $close = Find-El $dlg 'Đóng' 'btn pri' ([System.Windows.Automation.ControlType]::Button)
      if ($close) { Invoke-El $close }
      Start-Sleep -Milliseconds 400
    }
  }

  Test-Case 'UI-16' 'footer' {
    $main = Get-AppWindow 'Cài đặt VietKB'
    $foot = Find-El $main '' 'v4foot' ([System.Windows.Automation.ControlType]::Group)
    $order = @()
    foreach ($b in Get-AllButtons $foot) { $order += $b.Current.Name }
    $ok = ($order.Count -eq 4) -and $order[0] -eq 'Phím tắt…' -and $order[1] -eq 'Nâng cao…' -and $order[2] -eq 'Thoát VietKB' -and $order[3] -eq 'Đóng'
    Save-Evidence 'UI-16' @("order=$($order -join ' | ')") | Out-Null
    Add-Result 'UI-16' 'footer' $(if ($ok) { 'PASS' } else { 'FAIL' }) "order: $($order -join ', ')"
  }

  Test-Case 'UI-17' 'language' {
    $main = Get-AppWindow 'Cài đặt VietKB'
    Invoke-Button $main 'Nâng cao…' 'v4lk' | Out-Null
    $adv = Wait-AppWindow 'Nâng cao — VietKB' 5000
    Set-Combo $adv 'Ngôn ngữ hiển thị' 'English' | Out-Null
    Start-Sleep -Milliseconds 1500
    $persisted = Wait-Cfg 'system' 'language' '"en"' 2500
    $labels = @()
    foreach ($n in @('Bật tiếng Việt', 'Kiểm tra chính tả', 'Gõ', 'Đóng')) {
      $e = Find-El $main $n '' ([System.Windows.Automation.ControlType]::Text)
      if (-not $e) { $e = Find-El $main $n '' ([System.Windows.Automation.ControlType]::Button) }
      $labels += "$n=$($null -ne $e)"
    }
    $english = Find-El $main 'Vietnamese input' '' $null
    Set-Combo $adv 'Ngôn ngữ hiển thị' 'Tiếng Việt' | Out-Null
    $ok = $persisted -and ($null -ne $english)
    Save-Evidence 'UI-17' (@("persisted=$persisted", "englishLabel=$($null -ne $english)") + $labels) | Out-Null
    Add-Result 'UI-17' 'language' $(if ($ok) { 'PASS' } else { 'FAIL' }) "language persisted=$persisted, labels changed=$($null -ne $english); $($labels -join ', ')"
    $close = Find-El $adv 'Đóng' 'btn pri' ([System.Windows.Automation.ControlType]::Button)
    if ($close) { Invoke-El $close }
  }
}
