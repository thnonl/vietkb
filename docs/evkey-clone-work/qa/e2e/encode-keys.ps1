# Encodes Vietnamese text back into the keystrokes a user would type.
#
#   . .\encode-keys.ps1
#   ConvertTo-Keys 'tiếng Việt' 'telex'   # -> tieengs Vieetj
#   ConvertTo-Keys 'tiếng Việt' 'vni'     # -> tie6ng1 Vie6t5
#
# Shapes and tones are read from the Unicode decomposition, so there is no
# hand-written letter table: `ế` is `e` + U+0302 + U+0301.
# Telex: ê/ô/â double their own vowel, ă/ơ/ư take `w`, đ takes `dd`.
# VNI:   6/7/8/9 for circumflex/horn/breve/đ, tones 1..5.

$script:TelexTone = @{ 0x0301 = 's'; 0x0300 = 'f'; 0x0309 = 'r'; 0x0303 = 'x'; 0x0323 = 'j' }
$script:VniTone = @{ 0x0301 = '1'; 0x0300 = '2'; 0x0309 = '3'; 0x0303 = '4'; 0x0323 = '5' }
$script:VniShape = @{ 0x0302 = '6'; 0x0306 = '8'; 0x031B = '7' }

function Encode-Word([string]$word, [string]$method) {
    $vni = $method -eq 'vni'
    $toneMap = if ($vni) { $script:VniTone } else { $script:TelexTone }
    $chars = $word.Normalize([Text.NormalizationForm]::FormD).ToCharArray()
    $sb = New-Object System.Text.StringBuilder
    $tone = ''
    $i = 0
    while ($i -lt $chars.Count) {
        $c = $chars[$i]
        # đ / Đ have no canonical decomposition. Compare code points: PowerShell's
        # `-eq` folds case, so `đ` would otherwise match `Đ`.
        if ([int]$c -eq 0x0111 -or [int]$c -eq 0x0110) {
            [void]$sb.Append($(if ([int]$c -eq 0x0110) { 'D' } else { 'd' }))
            [void]$sb.Append($(if ($vni) { '9' } else { 'd' }))
            $i++
            continue
        }
        [void]$sb.Append($c)
        $lower = [char]::ToLowerInvariant($c)
        $i++
        while ($i -lt $chars.Count -and [int]$chars[$i] -ge 0x0300 -and [int]$chars[$i] -le 0x036F) {
            $mark = [int]$chars[$i]
            if ($toneMap.ContainsKey($mark)) {
                $tone = $toneMap[$mark]
            } elseif ($vni) {
                if ($script:VniShape.ContainsKey($mark)) { [void]$sb.Append($script:VniShape[$mark]) }
            } else {
                switch ($mark) {
                    0x0302 { [void]$sb.Append($lower) }        # ê/ô/â double their vowel
                    0x0306 { [void]$sb.Append('w') }           # ă
                    0x031B { [void]$sb.Append('w') }           # ơ, ư
                }
            }
            $i++
        }
    }
    # One tone key per word, at the end: the engine places the mark itself.
    [void]$sb.Append($tone)
    return $sb.ToString()
}

function ConvertTo-Keys([string]$text, [string]$method) {
    $sb = New-Object System.Text.StringBuilder
    foreach ($m in [regex]::Matches($text, '[\p{L}\p{M}]+|[^\p{L}\p{M}]+')) {
        $token = $m.Value
        if ($token -match '^[\p{L}\p{M}]+$') { [void]$sb.Append((Encode-Word $token $method)) }
        else { [void]$sb.Append($token) }
    }
    return $sb.ToString()
}
