. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
$dir = 'E:\vietkb\docs\evkey-clone-work\qa\e2e95'
$out = Join-Path $dir 'budget-p3p5.txt'
Set-Content -Encoding UTF8 $out 'P3/P5 idle'
function W($s) { Add-Content -Encoding UTF8 $out $s; Write-Host $s }
$app = Get-VkbProcess
if (-not $app) { W 'no app'; exit 1 }
$t0 = Get-Date
$cpu0 = $app.TotalProcessorTime.TotalSeconds
$m0 = Get-VkbMemory
W "start $($t0.ToString('HH:mm:ss')) pid=$($app.Id) app=$($m0.AppMB)MB total=$($m0.TotalMB)MB webviews=$($m0.WebViewCount) cpu_s=$([math]::Round($cpu0,2))"
# confirm no settings window is open
$win = Get-TopWindow 'Cài đặt VietKB'
W "settings_window_open=$([bool]$win)"
Start-Sleep -Seconds 600
$app = Get-VkbProcess
if (-not $app) { W 'app died during idle'; exit 1 }
$t1 = Get-Date
$cpu1 = $app.TotalProcessorTime.TotalSeconds
$m1 = Get-VkbMemory
$elapsed = ($t1 - $t0).TotalSeconds
$cpuPct = [math]::Round((($cpu1 - $cpu0) / $elapsed) * 100, 3)
W "end   $($t1.ToString('HH:mm:ss')) app=$($m1.AppMB)MB total=$($m1.TotalMB)MB webviews=$($m1.WebViewCount) cpu_s=$([math]::Round($cpu1,2))"
W "elapsed_s=$([math]::Round($elapsed,1)) cpu_avg_pct=$cpuPct (1 core basis)"
W "P3_pass=$($m1.TotalMB -le 20) P5_pass=$($cpuPct -le 0.1)"
W 'P3P5-DONE'
