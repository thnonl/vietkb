# Exploratory probe 2: follow-ups on the shapes that broke in probe 1.
$ErrorActionPreference = 'Continue'
$exe = 'E:\vietkb\target\debug\vkb.exe'
$B = [char]8
$SP = ' '

function Show([string]$m, [string]$keys, [string[]]$flags) {
  $shown = $keys.Replace([string]$B, '<BS>').Replace(' ', '<SP>')
  $out = & $exe simulate --method $m @flags $keys 2>&1
  $flat = ($out | Out-String).TrimEnd()
  '{0,-6}|{1,-34}|{2,-30}|{3}' -f $m, ($flags -join ','), $shown, $flat
}

$cases = @(
  ## z / 0 tone removal (spec A1 `asz -> a`, A2 `0` = remove tone)
  @('telex', 'asz', @()),
  @('telex', 'az', @()),
  @('telex', 'aszs', @()),
  @('telex', 'toansz', @()),
  @('telex', 'baajz', @()),
  @('vni', 'ba10', @()),
  @('vni', 'a60', @()),
  @('vni', 'ba650', @()),
  @('vni', 'ba6510', @()),
  ## same-tone retype on a finished word (A family), upper + lowercase, all five keys
  @('telex', 'baajtj', @()),
  @('telex', 'Baajtj', @()),
  @('telex', 'baajts', @()),
  @('telex', 'baajtf', @()),
  @('telex', 'baajtr', @()),
  @('telex', 'baajtx', @()),
  @('vni', 'ba65t1', @()),
  @('vni', 'ba65t2', @()),
  @('vni', 'ba65t3', @()),
  @('vni', 'ba65t4', @()),
  @('vni', 'ba65t5', @()),
  ## same-tone retype on a coda-less word
  @('telex', 'basf', @()),
  @('telex', 'basr', @()),
  @('telex', 'bass', @()),
  @('telex', 'basj', @()),
  @('telex', 'basx', @()),
  @('vni', 'ba11', @()),
  @('vni', 'ba12', @()),
  @('vni', 'ba15', @()),
  ## word with the tone on a middle vowel, tone re-typed
  @('telex', 'hoangfj', @()),
  @('telex', 'hoangfs', @()),
  @('telex', 'hoaf', @()),
  @('telex', 'hoafs', @()),
  ## BS then retype tone / letter with a new diacritic
  @('telex', "khoong${B}g", @()),
  @('telex', "khoong${B}s", @()),
  @('telex', "baajt${B}st", @()),
  @('telex', "kieems${B}m", @()),
  @('telex', "kieems${B}es", @()),
  @('telex', "kieems${B}ee", @()),
  @('telex', "Nguyeexn${B}ee", @()),
  @('telex', "Nguyeexn${B}n", @()),
  @('telex', "Nguyeexn${B}nn", @()),
  @('telex', "Nguyeexn${B}e", @()),
  @('telex', "duowcj${B}c", @()),
  @('telex', "duowcj${B}w", @()),
  @('telex', "duowcj${B}o", @()),
  @('telex', "dduowng${B}gf", @()),
  @('telex', "ddi${B}d", @()),
  @('telex', "ddi${B}dd", @()),
  @('telex', "ddi${B}e", @()),
  ## BS over a mark, then a different mark key
  @('telex', "baa${B}w", @()),
  @('telex', "baw${B}a", @()),
  @('telex', "baaw${B}", @()),
  ## BS at the very end of a committed word, then space
  @('telex', "baajt${SP}${B}${B}${SP}", @()),
  @('telex', "baajt${SP}${B}${B}n${SP}", @()),
  @('telex', "baajt${SP}${B}s", @()),
  @('telex', "baajt${SP}${B}f", @()),
  ## tone reselect on the last word of a sentence
  @('telex', "baajt.${B}s", @()),
  @('telex', "baajt,${B}s", @()),
  ## tone change with a comma boundary
  @('telex', 'baajt,s', @()),
  @('telex', 'baajt.s', @())
)
foreach ($c in $cases) { Show $c[0] $c[1] ([string[]]$c[2]) }
