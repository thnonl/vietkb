$dir = 'E:\vietkb\docs\evkey-clone-work\qa\e2e95'
$sb = New-Object System.Text.StringBuilder
foreach ($app in 'notepad', 'chrome', 'vscode') {
  $p = Join-Path $dir "typing-$app.tsv"
  if (-not (Test-Path $p)) { continue }
  [void]$sb.AppendLine("### $app")
  [void]$sb.AppendLine('')
  [void]$sb.AppendLine('| id | keystrokes | expected | actual | read via | verdict |')
  [void]$sb.AppendLine('|---|---|---|---|---|---|')
  $lines = Get-Content $p | Select-Object -Skip 1
  # rejoin rows broken by an embedded newline in the "actual" column
  $rows = @(); $buf = ''
  foreach ($l in $lines) {
    $buf = if ($buf) { "$buf`n$l" } else { $l }
    if (($buf -split "`t").Count -ge 8) { $rows += $buf; $buf = '' }
  }
  if ($buf) { $rows += $buf }
  foreach ($r in $rows) {
    $f = $r -split "`t"
    if ($f.Count -lt 8) {
      [void]$sb.AppendLine("| $($f[1]) | $($f[2]) | $($f[3]) | $($f[4]) | $($f[5]) | (row split) |")
      continue
    }
    $esc = { param($s) ($s -replace '\|', '\|') }
    [void]$sb.AppendLine(("| {0} | `{1}` | {2} | {3} | {4} | {5} |" -f $f[1], $f[2], (& $esc $f[3]), (& $esc $f[4]), $f[5], $f[7]))
  }
  [void]$sb.AppendLine('')
}
Set-Content -Encoding UTF8 (Join-Path $dir 'typing-tables.md') $sb.ToString()
Write-Host "rows written: $((Get-Content (Join-Path $dir 'typing-tables.md')).Count)"
