# Runs the edge-case table through the engine (headless).
#
#   powershell -NoProfile -File check-edge-cases.ps1
param(
    [string]$Vkb = 'E:\vietkb\target\release\vkb.exe'
)
$ErrorActionPreference = 'Stop'
$here = Split-Path -Parent $MyInvocation.MyCommand.Path
$out = Join-Path $here 'edge-results.txt'
Set-Content -Encoding UTF8 $out 'EDGE-CASE-CHECK'
function Log($s) { Add-Content -Encoding UTF8 $out $s; Write-Host $s }

if (-not (Test-Path $Vkb)) { throw "vkb.exe not found at $Vkb" }

function Run-Simulate([string]$keys, [string]$method, [string]$flags) {
    $tmp = Join-Path $env:TEMP 'vkb-edge.txt'
    $escaped = $keys -replace '"', '\"'
    $args = "simulate --method $method $flags `"$escaped`""
    $p = Start-Process -FilePath $Vkb -ArgumentList $args -NoNewWindow -Wait -PassThru `
        -RedirectStandardOutput $tmp -RedirectStandardError (Join-Path $env:TEMP 'vkb-edge.err')
    if ($p.ExitCode -ne 0) { throw "simulate failed ($($p.ExitCode)) for [$keys]" }
    return (Get-Content $tmp -Raw -Encoding UTF8).TrimEnd("`r", "`n")
}

$rows = Get-Content (Join-Path $here 'edge-cases.tsv') -Encoding UTF8 | Select-Object -Skip 1
$pass = 0
$fail = 0
foreach ($row in $rows) {
    if (-not $row.Trim()) { continue }
    $f = $row -split "`t"
    $id, $method, $flags, $keys, $expected, $note = $f
    $actual = Run-Simulate $keys $method $flags
    if ($actual -ceq $expected) {
        $pass++
        Log ("EDGE {0} {1} [{2}] -> [{3}] PASS {4}" -f $id, $method, $keys, $actual, $note)
    } else {
        $fail++
        Log ("EDGE {0} {1} [{2}] expected=[{3}] actual=[{4}] FAIL {5}" -f $id, $method, $keys, $expected, $actual, $note)
    }
}
Log "EDGE_PASS=$pass EDGE_FAIL=$fail"
Log 'EDGE-DONE'
