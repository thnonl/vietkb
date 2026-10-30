param([switch]$NoLaunch)
. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
$dir = 'E:\vietkb\docs\evkey-clone-work\qa\e2e95'
$out = Join-Path $dir 'probe1.txt'
Set-Content -Encoding UTF8 $out 'PROBE1'
$scratch = Join-Path $dir 'scratch-np.txt'

function W($s) { Add-Content -Encoding UTF8 $out $s; Write-Host $s }

Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Milliseconds 600
Remove-Item $scratch -ErrorAction SilentlyContinue
New-Item -ItemType File -Path $scratch | Out-Null
Start-Process notepad.exe -ArgumentList $scratch | Out-Null
$np = Wait-Window 'scratch-np.txt' 30
if (-not $np) { W 'notepad=NO_WINDOW'; exit 1 }
W "notepad hwnd=$($np.Hwnd) title=$($np.Title)"
[void][QAInput]::Focus($np.Hwnd)
Start-Sleep -Milliseconds 500
$u = Get-UiaText $np.Hwnd
W "uia_initial via=$(if($u){$u.Via}else{'null'}) text=[$(if($u){$u.Text}else{'<null>'})]"

[QAInput]::TypeAscii('abc ba6t5', 12)
Start-Sleep -Milliseconds 400
$u = Get-UiaText $np.Hwnd
W "no_hook via=$($u.Via) text=[$($u.Text)] pass=$($u.Text -ceq 'abc ba6t5')"

if (-not $NoLaunch) {
  Get-Process vietkb -ErrorAction SilentlyContinue | Stop-Process -Force
  Start-Sleep -Milliseconds 300
  $sh = New-Object -ComObject Shell.Application
  [void]$sh.ShellExecute((Join-Path $dir 'launch-vietkb.cmd'), '', 'E:\vietkb', 'open', 0)
  $t0 = Get-Date
  $app = $null
  while ((Get-Date) - $t0 -lt [timespan]::FromSeconds(15)) { $app = Get-VkbProcess; if ($app) { break }; Start-Sleep -Milliseconds 100 }
  if (-not $app) { W 'app=NO_PROCESS'; exit 1 }
  W "app pid=$($app.Id) start=$($app.StartTime.ToString('HH:mm:ss.fff'))"
  Start-Sleep -Milliseconds 1500
  Clear-TextArea $np.Hwnd
  [QAInput]::TypeAscii('ba6t5', 20)
  Start-Sleep -Milliseconds 500
  $u = Get-UiaText $np.Hwnd
  W "hook_on via=$($u.Via) text=[$($u.Text)] pass=$($u.Text -ceq 'bật')"
  $u2 = Get-ClipboardText $np.Hwnd
  W "hook_on_clipboard text=[$($u2.Text)]"
}
W 'PROBE1-DONE'
