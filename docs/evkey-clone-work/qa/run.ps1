# §3 UI run driver — restores the config, runs every group, writes results.
$ErrorActionPreference = 'Stop'
$root = 'C:\Users\namth\AppData\Local\Temp\opencode\vkb-qa'
. "$root\lib.ps1"
. "$root\cases-a.ps1"
. "$root\cases-b.ps1"
. "$root\cases-c.ps1"

$exe = 'E:\vietkb\target\release\vietkb.exe'
$info = Get-Item $exe
$hash = (Get-FileHash $exe -Algorithm SHA256).Hash
Set-Content -Path "$root\build.txt" -Value @(
  "exe=$exe",
  "mtime=$($info.LastWriteTime.ToString('o'))",
  "size=$($info.Length)",
  "sha256=$hash",
  "started=$(Get-Date -Format o)"
) -Encoding UTF8

# Baseline: the config captured before the first mutation of this session.
Copy-Item "$root\config-before.toml" $CfgPath -Force
Stop-TestApp

$groups = @(
  @('A', { Run-GroupA }),
  @('B', { Run-GroupB }),
  @('C', { Run-GroupC }),
  @('D', { Run-GroupD }),
  @('E', { Run-GroupE }),
  @('E2', { Run-GroupE2 }),
  @('F', { Run-GroupF }),
  @('G', { Run-GroupG })
)

foreach ($g in $groups) {
  Write-Output "=== group $($g[0]) $(Get-Date -Format HH:mm:ss) ==="
  $beforeCount = $script:Results.Count
  try { & $g[1] } catch {
    Write-Output "GROUP $($g[0]) ABORTED: $($_.Exception.Message)"
    if ($script:Results.Count -eq $beforeCount) {
      Write-Output "=== retry group $($g[0]) ==="
      Start-Sleep -Milliseconds 800
      try { & $g[1] } catch { Write-Output "GROUP $($g[0]) ABORTED AGAIN: $($_.Exception.Message)" }
    }
  }
}

$script:Results | Export-Csv -Path "$root\results.csv" -NoTypeInformation -Encoding UTF8
$script:Results | ConvertTo-Json -Depth 4 | Set-Content "$root\results.json" -Encoding UTF8
"TOTAL=$($script:Results.Count) PASS=$(@($script:Results | Where-Object Status -eq 'PASS').Count) FAIL=$(@($script:Results | Where-Object Status -eq 'FAIL').Count) BLOCKED=$(@($script:Results | Where-Object Status -eq 'BLOCKED').Count)"

Stop-TestApp
for ($i = 0; $i -lt 10; $i++) {
  try { Copy-Item "$root\config-before.toml" $CfgPath -Force; break } catch { Start-Sleep -Milliseconds 500 }
}
"finished=$(Get-Date -Format o)"
