. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
$dir = 'E:\vietkb\docs\evkey-clone-work\qa\e2e95'
$out = Join-Path $dir 'budget-p6p4.txt'
Set-Content -Encoding UTF8 $out 'P6/P4'
function W($s) { Add-Content -Encoding UTF8 $out $s; Write-Host $s }

$ch = Get-TopWindow 'VietKB E2E probe'
if (-not $ch) { W 'chrome=NO_WINDOW'; exit 1 }
$hwnd = $ch.Hwnd

# ---- P6: cold start -> first keystroke transforms ----
$times = @()
for ($run = 1; $run -le 3; $run++) {
  Get-Process vietkb -ErrorAction SilentlyContinue | Stop-Process -Force
  Start-Sleep -Milliseconds 1200
  [void][QAInput]::Focus($hwnd); Start-Sleep -Milliseconds 150
  $el = Get-EditElement $hwnd; if ($el) { [void](Click-Element $el) }
  Clear-TextArea $hwnd
  Start-Sleep -Milliseconds 200
  $t0 = [DateTime]::UtcNow
  $sh = New-Object -ComObject Shell.Application
  [void]$sh.ShellExecute((Join-Path $dir 'launch-vietkb.cmd'), '', 'E:\vietkb', 'open', 0)
  $ready = $false; $ms = -1
  while ((([DateTime]::UtcNow) - $t0).TotalMilliseconds -lt 10000) {
    [QAInput]::TypeAscii('a6', 8)
    Start-Sleep -Milliseconds 35
    $u = Get-UiaText $hwnd
    $t = [string]$u.Text
    if ($t -match 'â') { $ms = [math]::Round((([DateTime]::UtcNow) - $t0).TotalMilliseconds, 0); $ready = $true; break }
    Clear-TextArea $hwnd
    Start-Sleep -Milliseconds 20
  }
  $times += $ms
  W "P6 run$run hook_ready_ms=$ms ready=$ready"
  Start-Sleep -Milliseconds 1500
}

# ---- P4: settings window open / closed memory ----
[void][QAInput]::Focus($hwnd); Start-Sleep -Milliseconds 200
$m0 = Get-VkbMemory
W "P4 tray_only app=$($m0.AppMB)MB total=$($m0.TotalMB)MB webviews=$($m0.WebViewCount)"
[QAInput]::Combo(0x53, $false, $true, $true); Start-Sleep -Milliseconds 2500
$m1 = Get-VkbMemory
W "P4 window_open app=$($m1.AppMB)MB total=$($m1.TotalMB)MB webviews=$($m1.WebViewCount)"
# close via the window's close button (UIA)
$win = Get-TopWindow 'Cài đặt VietKB'
if ($win) {
  $btn = Find-ElementByName $win.Hwnd 'Đóng, VietKB vẫn chạy' 'Button'
  if (-not $btn) { $btn = Find-ElementByName $win.Hwnd 'Đóng' 'Button' }
  if ($btn) { try { $btn.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke() } catch { [void](Click-Element $btn) } }
  else { [void][QAInput]::Focus($win.Hwnd); [QAInput]::Combo(0x1B, $false, $false, $false) }
  Start-Sleep -Milliseconds 3000
  $m2 = Get-VkbMemory
  W "P4 after_close_3s app=$($m2.AppMB)MB total=$($m2.TotalMB)MB webviews=$($m2.WebViewCount)"
  W "P4 drop_mb=$([math]::Round($m1.TotalMB - $m2.TotalMB, 1))"
} else { W 'P4 settings window not found for close' }
W 'P6P4-DONE'
