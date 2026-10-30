param([int]$Minutes = 30)
. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
$dir = 'E:\vietkb\docs\evkey-clone-work\qa\e2e95'
$out = Join-Path $dir 'soak.txt'
Set-Content -Encoding UTF8 $out 'SOAK 30 min, 5 apps'
function W($s) { Add-Content -Encoding UTF8 $out $s; Write-Host $s }

$targets = @()
$ch = Get-TopWindow 'VietKB E2E probe'
if ($ch) { $targets += [pscustomobject]@{ Name = 'chrome'; Hwnd = $ch.Hwnd } }
$vs = Get-TopWindow 'scratch-vsc.txt'
if ($vs) { $targets += [pscustomobject]@{ Name = 'vscode'; Hwnd = $vs.Hwnd } }
Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Milliseconds 500
$npx = Join-Path $dir 'scratch-np.txt'
Remove-Item $npx -ErrorAction SilentlyContinue; New-Item -ItemType File -Path $npx | Out-Null
Start-Process notepad.exe -ArgumentList $npx | Out-Null
$np = Wait-Window 'scratch-np.txt' 20
if ($np) { $targets += [pscustomobject]@{ Name = 'notepad'; Hwnd = $np.Hwnd } }
# console target
Start-Process cmd.exe -ArgumentList '/k', 'title VKB soak console' | Out-Null
Start-Sleep -Milliseconds 1500
$con = Get-TopWindow 'VKB soak console' 15
if ($con) { $targets += [pscustomobject]@{ Name = 'console'; Hwnd = $con.Hwnd } }
# the settings window as a fifth app surface (WebView input)
[void][QAInput]::Focus($ch.Hwnd); Start-Sleep -Milliseconds 200
[QAInput]::Combo(0x53, $false, $true, $true); Start-Sleep -Milliseconds 1500
$st = Get-TopWindow 'Cài đặt VietKB'
if ($st) { $targets += [pscustomobject]@{ Name = 'settings-webview'; Hwnd = $st.Hwnd } }

W "started=$(Get-Date -Format 'HH:mm:ss') targets=$($targets.Name -join ',')"
$sentence = 'bật tắt tiếng Việt, Nguyễn Thu Hà gõ nhanh 100%/50% a/b 2/3 C:\Users\nam '
$rounds = [math]::Ceiling($Minutes / 3)
$keystrokes = 0
$warn0 = @(Select-String -Path (Join-Path $dir 'app.log') -Pattern 'WARN|ERROR' -AllMatches).Count

for ($r = 1; $r -le $rounds; $r++) {
  $t = $targets[($r - 1) % $targets.Count]
  if (-not $t) { continue }
  $deadline = (Get-Date).AddMinutes(3)
  $iter = 0
  while ((Get-Date) -lt $deadline) {
    [void][QAInput]::Focus($t.Hwnd)
    Start-Sleep -Milliseconds 150
    if ($t.Name -eq 'notepad') { [void](Repair-Notepad 'scratch-np.txt') }
    if ($t.Name -eq 'chrome' -or $t.Name -eq 'vscode') {
      $el = Get-EditElement $t.Hwnd; if ($el) { [void](Click-Element $el) }
    }
    Clear-TextArea $t.Hwnd
    for ($b = 0; $b -lt 4; $b++) {
      [QAInput]::TypeAscii($sentence, 25)
      $keystrokes += $sentence.Length
      [QAInput]::Combo(0x0D, $false, $false, $false)
      Start-Sleep -Milliseconds 400
    }
    $iter++
    if ($r % 3 -eq 0 -and $iter -eq 1) {
      [QAInput]::Combo(0x71, $false, $false, $false); Start-Sleep -Milliseconds 300   # F2
      [QAInput]::TypeAscii('ba6t5', 40)
      [QAInput]::Combo(0x70, $false, $false, $false); Start-Sleep -Milliseconds 300   # F1
    }
    if ($r % 2 -eq 0 -and $iter -eq 2) { [QAInput]::TypeAscii('m01 ', 40); $keystrokes += 4 }
    Start-Sleep -Milliseconds 300
  }
  # canary: hook still transforms, in Chrome
  $state = 'NO_OBSERVER'
  if ($ch) {
    [void][QAInput]::Focus($ch.Hwnd); Start-Sleep -Milliseconds 250
    $el = Get-EditElement $ch.Hwnd; if ($el) { [void](Click-Element $el) }
    Clear-TextArea $ch.Hwnd
    [QAInput]::TypeAscii('ba6t5', 90)
    Start-Sleep -Milliseconds 500
    $txt = ([string](Get-UiaText $ch.Hwnd).Text).TrimEnd("`r", "`n")
    $state = if ($txt -eq 'bật') { 'TRANSFORMS' } else { "BROKEN($txt)" }
  }
  $app = Get-VkbProcess
  $mem = Get-VkbMemory
  $warn = @(Select-String -Path (Join-Path $dir 'app.log') -Pattern 'WARN|ERROR' -AllMatches).Count
  W ("round {0}/{1} {2,-16} target={3,-18} app_alive={4} app={5}MB total={6}MB canary={7} warn_err={8} keys~{9} t={10}" -f `
      $r, $rounds, $t.Name, $t.Name, [bool]$app, $mem.AppMB, $mem.TotalMB, $state, ($warn - $warn0), $keystrokes, (Get-Date -Format 'HH:mm:ss'))
  if (-not $app) { W 'APP DIED — soak aborted'; break }
}

# finish: close every app-owned window, return to tray, measure after settling
$st = Get-TopWindow 'Cài đặt VietKB'
if ($st) { [void][QAInput]::Focus($st.Hwnd); Start-Sleep -Milliseconds 300; [QAInput]::Combo(0x1B, $false, $false, $false); Start-Sleep -Milliseconds 500 }
$st = Get-TopWindow 'Cài đặt VietKB'
if ($st) {
  $btn = Find-ElementByName $st.Hwnd 'Đóng, VietKB vẫn chạy' 'Button'
  if ($btn) { try { $btn.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke() } catch { [void](Click-Element $btn) } }
}
Start-Sleep -Seconds 5
$mem = Get-VkbMemory
W "end=$(Get-Date -Format 'HH:mm:ss') keystrokes~$keystrokes final_app=$($mem.AppMB)MB total=$($mem.TotalMB)MB webviews=$($mem.WebViewCount) app_alive=$([bool](Get-VkbProcess))"
W 'SOAK-DONE'
