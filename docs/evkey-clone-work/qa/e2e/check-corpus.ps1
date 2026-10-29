# Types each corpus paragraph through the engine (headless) and compares it with
# the authored text. Fast pre-check before the desktop run: if the keystrokes are
# valid, a GUI failure is the hook path; if they are not, this run says so first.
#
#   powershell -NoProfile -File check-corpus.ps1
#   powershell -NoProfile -File check-corpus.ps1 -Method vni
param(
    [ValidateSet('telex', 'vni', 'both')][string]$Method = 'both',
    [string]$Vkb = 'E:\vietkb\target\release\vkb.exe'
)
$ErrorActionPreference = 'Stop'
$here = Split-Path -Parent $MyInvocation.MyCommand.Path
. (Join-Path $here 'encode-keys.ps1')

$out = Join-Path $here 'corpus-results.txt'
Set-Content -Encoding UTF8 $out 'CORPUS-PRE-CHECK'
function Log($s) { Add-Content -Encoding UTF8 $out $s; Write-Host $s }

if (-not (Test-Path $Vkb)) { throw "vkb.exe not found at $Vkb" }

$methods = if ($Method -eq 'both') { @('telex', 'vni') } else { @($Method) }
$failures = 0

function Run-Simulate([string]$keys, [string]$method) {
    # Start-Process takes one argument string, so the quotes in a paragraph
    # (`"bên B thanh toán..."`) survive; `&` would let PowerShell split them.
    $tmp = Join-Path $env:TEMP 'vkb-sim.txt'
    $escaped = $keys -replace '"', '\"'
    $args = "simulate --method $method `"$escaped`""
    $p = Start-Process -FilePath $Vkb -ArgumentList $args -NoNewWindow -Wait -PassThru `
        -RedirectStandardOutput $tmp -RedirectStandardError (Join-Path $env:TEMP 'vkb-sim.err')
    if ($p.ExitCode -ne 0) { throw "simulate failed ($($p.ExitCode)) for [$keys]" }
    return (Get-Content $tmp -Raw -Encoding UTF8).TrimEnd("`r", "`n")
}

foreach ($m in $methods) {
    Log "METHOD $m"
    foreach ($file in Get-ChildItem (Join-Path $here 'corpus') -Filter *.txt | Sort-Object Name) {
        $expected = (Get-Content $file.FullName -Raw -Encoding UTF8).Trim()
        $keys = ConvertTo-Keys $expected $m
        $actual = Run-Simulate $keys $m
        if ($actual -ceq $expected) {
            Log ("PARA {0} {1} PASS keys={2}" -f $file.BaseName, $m, $keys.Length)
        } else {
            $failures++
            Log ("PARA {0} {1} FAIL" -f $file.BaseName, $m)
            Log ("  expected=[{0}]" -f $expected)
            Log ("  actual  =[{0}]" -f $actual)
            Log ("  keys    =[{0}]" -f $keys)
            # Point at the first differing word, so the report stays readable.
            $ew = $expected -split '\s+'
            $aw = $actual -split '\s+'
            for ($i = 0; $i -lt [Math]::Max($ew.Count, $aw.Count); $i++) {
                $a = if ($i -lt $ew.Count) { $ew[$i] } else { '<none>' }
                $b = if ($i -lt $aw.Count) { $aw[$i] } else { '<none>' }
                if ($a -cne $b) { Log ("  first_diff word#{0} expected=[{1}] actual=[{2}]" -f ($i + 1), $a, $b); break }
            }
        }
    }
}

Log "PARAGRAPH_FAILURES=$failures"
Log 'CORPUS-DONE'
