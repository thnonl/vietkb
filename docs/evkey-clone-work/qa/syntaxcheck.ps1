$files = @('lib.ps1','cases-a.ps1','cases-b.ps1','cases-c.ps1','run.ps1')
foreach ($f in $files) {
  $path = Join-Path 'C:\Users\namth\AppData\Local\Temp\opencode\vkb-qa' $f
  $tokens = $null; $errors = $null
  [void][System.Management.Automation.Language.Parser]::ParseFile($path, [ref]$tokens, [ref]$errors)
  if ($errors.Count -eq 0) { "OK   $f" } else { "ERR  $f"; foreach ($e in $errors) { "  line $($e.Extent.StartLineNumber): $($e.Message)" } }
}
