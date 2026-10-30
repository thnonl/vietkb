# Exploratory probe 4: remaining family-B shapes and the retype-the-same-letter cases.
$ErrorActionPreference = 'Continue'
$exe = 'E:\vietkb\target\debug\vkb.exe'
$B = [char]8
$SP = ' '

function Show([string]$m, [string]$keys, [string[]]$flags) {
  $shown = $keys.Replace([string]$B, '<BS>').Replace(' ', '<SP>')
  $out = & $exe simulate --method $m @flags $keys 2>&1
  $flat = ($out | Out-String).TrimEnd("`r", "`n")
  '{0,-6}|{1,-40}|{2,-30}|{3}' -f $m, ($flags -join ','), $shown, $flat
}

$cases = @(
  @('telex', "baajt${B}t", @()),
  @('telex', "baajt${B}s", @()),
  @('telex', "baajt${B}st", @()),
  @('telex', "baajt${B}bt", @()),
  @('telex', "toans${B}n", @()),
  @('telex', "toans${B}s", @()),
  @('telex', "dduowcj${B}c", @()),
  @('telex', "dduowcj${B}j", @()),
  @('telex', "dduowcj${B}z", @()),
  @('telex', "dduowcj${B}s", @()),
  @('telex', "tieengs${B}g", @()),
  @('telex', "tieengs${B}s", @()),
  @('telex', "tieengs${B}n", @()),
  @('telex', "Thuongwf${B}g", @()),
  @('telex', "Thuongwf${B}s", @()),
  ## committed word, delete the tail letter, then a tone key / space
  @('telex', "hoangf${SP}${B}${B}s", @()),
  @('telex', "hoangf${SP}${B}s", @()),
  @('telex', "hoangf${SP}${B}${B}${SP}", @()),
  ## no-free-tone and old-tone-style on family B
  @('telex', "baajt${B}s", @('--no-free-tone')),
  @('telex', "baajt${B}s", @('--old-tone-style')),
  @('telex', "hoangf${B}s", @('--old-tone-style')),
  ## spell check off on family B
  @('telex', "baajt${B}s", @('--no-spell-check')),
  @('telex', "baajt${B}f", @('--no-spell-check')),
  ## VNI family B
  @('vni', "ba65t${B}1", @()),
  @('vni', "ba65t${B}2", @()),
  @('vni', "ba65t${B}t", @()),
  @('vni', "ba65t${B}${B}t", @()),
  @('vni', "ba65t${B}0", @()),
  @('vni', "ba65t${B}5", @()),
  @('vni', "ba65t${B}9", @()),
  ## NFD family B
  @('telex', "kieems${B}m", @('--charset', 'unicode_nfd')),
  @('telex', "Nguyeexn${B}n", @('--charset', 'unicode_nfd')),
  ## z / 0 with the option flags
  @('telex', 'baajz', @('--no-spell-check')),
  @('telex', 'baajz', @('--no-free-tone')),
  @('vni', 'ba650', @('--no-spell-check'))
)
foreach ($c in $cases) { Show $c[0] $c[1] ([string[]]$c[2]) }
