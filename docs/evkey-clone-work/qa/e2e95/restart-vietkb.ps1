$dir = 'E:\vietkb\docs\evkey-clone-work\qa\e2e95'
Get-Process vietkb -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Milliseconds 800
Remove-Item (Join-Path $env:APPDATA 'vietkb\keys.log') -ErrorAction SilentlyContinue
$sh = New-Object -ComObject Shell.Application
[void]$sh.ShellExecute((Join-Path $dir 'launch-vietkb.cmd'), '', 'E:\vietkb', 'open', 0)
$t0 = Get-Date
while ((Get-Date) - $t0 -lt [timespan]::FromSeconds(20)) {
  $p = Get-Process vietkb -ErrorAction SilentlyContinue
  if ($p) { Write-Host "started pid=$($p.Id) at $($p.StartTime.ToString('HH:mm:ss.fff'))"; break }
  Start-Sleep -Milliseconds 150
}
