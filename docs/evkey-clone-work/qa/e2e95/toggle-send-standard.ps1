param([switch]$Restore)
. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
$cfg = Join-Path $env:APPDATA 'vietkb\config.toml'
$bak = 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\config.backup.toml'
if ($Restore) {
  Copy-Item $bak $cfg -Force
  Write-Host "restored config hash=$((Get-FileHash $cfg -Algorithm SHA256).Hash)"
} else {
  $t = Get-Content $cfg -Raw
  $t = $t -replace 'send_standard = false', 'send_standard = true'
  Set-Content -Encoding UTF8 $cfg $t
  Write-Host 'config send_standard=true'
}
Get-Process vietkb -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Milliseconds 700
$sh = New-Object -ComObject Shell.Application
[void]$sh.ShellExecute('E:\vietkb\docs\evkey-clone-work\qa\e2e95\launch-vietkb.cmd', '', 'E:\vietkb', 'open', 0)
Start-Sleep -Milliseconds 2500
Write-Host "app pid=$((Get-Process vietkb -ErrorAction SilentlyContinue).Id)"
