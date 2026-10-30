. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
$ch = Get-TopWindow 'VietKB E2E probe'
[void][QAInput]::Focus($ch.Hwnd); Start-Sleep -Milliseconds 300
for ($i = 0; $i -lt 4; $i++) {
  [QAInput]::Combo(0x70, $false, $false, $false)   # F1 = enable
  Start-Sleep -Milliseconds 500
  $el = Get-EditElement $ch.Hwnd; if ($el) { [void](Click-Element $el) }
  Clear-TextArea $ch.Hwnd
  [QAInput]::TypeAscii('ba6t5', 120)
  Start-Sleep -Milliseconds 600
  $t = ([string](Get-UiaText $ch.Hwnd).Text).TrimEnd("`r", "`n")
  if ($t -match 'ậ') { Write-Host "state=ON after $($i+1) F1"; break }
  Write-Host "still [${t}]"
}
