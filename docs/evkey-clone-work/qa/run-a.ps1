$ErrorActionPreference = 'Stop'
$root = 'C:\Users\namth\AppData\Local\Temp\opencode\vkb-qa'
. "$root\lib.ps1"
. "$root\cases-a.ps1"
. "$root\cases-b.ps1"
. "$root\cases-c.ps1"
Reset-Config
Run-GroupA
$script:Results | Export-Csv -Path "$root\results-a.csv" -NoTypeInformation -Encoding UTF8
"TOTAL=$($script:Results.Count) PASS=$(@($script:Results | Where-Object Status -eq 'PASS').Count) FAIL=$(@($script:Results | Where-Object Status -eq 'FAIL').Count)"
Stop-TestApp
Copy-Item "$root\config-before.toml" $CfgPath -Force
