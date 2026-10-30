# Exploratory probe 3: option/charset variants, spell-check parity, mark-digit retype.
$ErrorActionPreference = 'Continue'
$exe = 'E:\vietkb\target\debug\vkb.exe'
$B = [char]8
$SP = ' '

function Show([string]$m, [string]$keys, [string[]]$flags) {
  $shown = $keys.Replace([string]$B, '<BS>').Replace(' ', '<SP>')
  $out = & $exe simulate --method $m @flags $keys 2>&1
  $flat = ($out | Out-String).TrimEnd("`r", "`n")
  '{0,-6}|{1,-40}|{2,-28}|{3}' -f $m, ($flags -join ','), $shown, $flat
}

$cases = @(
  ## spell-check parity: the same illegal tone on a fresh word vs on a finished word
  @('telex', 'tacf', @()),
  @('telex', 'tacr', @()),
  @('telex', 'tacx', @()),
  @('telex', 'baajtf', @()),
  @('telex', 'baajtf', @('--no-spell-check')),
  @('telex', 'tacf', @('--no-spell-check')),
  ## same tone re-typed after a reopen of a committed word
  @('telex', "baajt${SP}${B}j", @()),
  @('telex', "baajt${SP}${B}s", @()),
  @('telex', "baajt${SP}${B}b", @()),
  ## same tone re-typed, middle-vowel word
  @('telex', 'hoangff', @()),
  @('telex', 'hoangf' + $B + 'f', @()),
  ## VNI mark digit re-typed
  @('vni', 'ba65t6', @()),
  @('vni', 'ba65t65', @()),
  @('vni', 'ba66', @()),
  ## Telex mark letter re-typed on a finished word
  @('telex', 'baajta', @()),
  @('telex', 'baajtd', @()),
  ## delete the tone char then retype it with a new tone
  @('telex', "bas${B}as", @()),
  @('telex', "bas${B}af", @()),
  @('vni', "ba1${B}a2", @()),
  ## old tone style / no free tone on the tone-change rows
  @('telex', 'baajts', @('--old-tone-style')),
  @('telex', 'baajtj', @('--old-tone-style')),
  @('telex', 'baajts', @('--no-free-tone')),
  @('telex', 'baajtj', @('--no-free-tone')),
  @('vni', 'ba65t5', @('--no-free-tone')),
  @('telex', 'hoafs', @('--old-tone-style')),
  @('telex', 'hoafs', @()),
  @('telex', 'thuys', @()),
  @('telex', 'thuys', @('--old-tone-style')),
  ## NFD charset
  @('telex', 'baajtj', @('--charset', 'unicode_nfd')),
  @('telex', 'baajts', @('--charset', 'unicode_nfd')),
  @('telex', "baajt${B}s", @('--charset', 'unicode_nfd')),
  @('telex', 'baajtf', @('--charset', 'unicode_nfd')),
  @('telex', 'asz', @('--charset', 'unicode_nfd')),
  @('telex', "kieems${B}s", @('--charset', 'unicode_nfd')),
  @('telex', 'baajt', @('--charset', 'unicode_nfd')),
  @('telex', "baajt${B}", @('--charset', 'unicode_nfd')),
  @('telex', 'dduowcj', @('--charset', 'unicode_nfd')),
  @('telex', "dduowcj${B}c", @('--charset', 'unicode_nfd')),
  ## uppercase
  @('telex', 'Baatj', @()),
  @('telex', 'BAATJ', @()),
  @('telex', 'BAATJF', @()),
  @('telex', 'Baatjs', @())
)
foreach ($c in $cases) { Show $c[0] $c[1] ([string[]]$c[2]) }
