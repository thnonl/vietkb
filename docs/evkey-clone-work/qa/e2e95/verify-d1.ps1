# verify-d1.ps1 — read the Notepad D1 row from the saved file bytes (UIA normalises
# newlines, which is why the harness marks it FAIL while the text is identical).
. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
$dir = 'E:\vietkb\docs\evkey-clone-work\qa\e2e95'
$f = Join-Path $dir 'scratch-d1.txt'
Remove-Item $f -ErrorAction SilentlyContinue; New-Item -ItemType File -Path $f | Out-Null
Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Milliseconds 600
Start-Process notepad.exe -ArgumentList $f | Out-Null
$w = Wait-Window 'scratch-d1.txt' 30
if (-not $w) { 'SETUP-FAIL'; exit 1 }
$hwnd = $w.Hwnd
Start-Sleep -Milliseconds 1500
[void][QAInput]::Focus($hwnd)
Start-Sleep -Milliseconds 200
$el = Get-EditElement $hwnd
if ($el) { [void](Click-Element $el) } else { Click-Center $hwnd }
Start-Sleep -Milliseconds 200
Clear-TextArea $hwnd
Start-Sleep -Milliseconds 200
[QAInput]::TypeAscii('ba6t5', 55)
[QAInput]::Combo(0x0D, $false, $false, $false)   # Enter
Start-Sleep -Milliseconds 250
[QAInput]::TypeAscii('ba6t5', 55)
Start-Sleep -Milliseconds 400
[QAInput]::Combo(0x53, $false, $true, $false)    # Ctrl+S
Start-Sleep -Milliseconds 700
$bytes = [System.IO.File]::ReadAllBytes($f)
$text = [System.Text.Encoding]::UTF8.GetString($bytes)
"file-bytes-len=$($bytes.Length)"
"text=[$($text.Replace("`r",'\r').Replace("`n",'\n'))]"
$exp = "bật`r`nbật"
"matches-CRLF=$($text -ceq $exp)"
$codepoints = ($text.ToCharArray() | ForEach-Object { 'U+{0:X4}' -f [int]$_ }) -join ' '
"codepoints=$codepoints"
