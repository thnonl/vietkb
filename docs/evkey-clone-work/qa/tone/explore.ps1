# Exploratory probe: measure current engine behaviour on tone-change / partial-delete shapes.
# Read-only: one `vkb-cli simulate` run per row, no product code touched.
$ErrorActionPreference = 'Continue'
$exe = 'E:\vietkb\target\debug\vkb.exe'
$B = [char]8
$SP = ' '

function Show([string]$m, [string]$keys, [string[]]$flags) {
  $shown = $keys.Replace([string]$B, '<BS>').Replace(' ', '<SP>')
  $out = & $exe simulate --method $m @flags $keys 2>&1
  $flat = ($out | Out-String).TrimEnd()
  '{0,-6}|{1,-22}|{2,-34}|{3}' -f $m, ($flags -join ','), $shown, $flat
}

$cases = @(
  @('telex', 'basf', @()),
  @('telex', 'basr', @()),
  @('telex', 'basx', @()),
  @('telex', 'basj', @()),
  @('telex', 'bass', @()),
  @('telex', 'basz', @()),
  @('telex', 'bansf', @()),
  @('telex', 'bansr', @()),
  @('telex', 'banss', @()),
  @('telex', 'baajts', @()),
  @('telex', 'baajtf', @()),
  @('telex', 'baajtr', @()),
  @('telex', 'baajtx', @()),
  @('telex', 'baajtj', @()),
  @('telex', 'baajtz', @()),
  @('vni', 'ba1f', @()),
  @('vni', 'ba1s', @()),
  @('vni', 'ba11', @()),
  @('vni', 'ba10', @()),
  @('vni', 'ba65t1', @()),
  @('vni', 'ba65t2', @()),
  @('vni', 'ba65t5', @()),
  @('vni', 'ba65t0', @()),
  @('telex', "baajt${SP}f", @()),
  @('telex', "baajt${SP}s", @()),
  @('telex', "baajt${SP}${B}s", @()),
  @('telex', 'Baajts', @()),
  @('telex', 'Baajtf', @()),
  @('telex', 'baajt.f', @()),
  @('telex', "baajt${B}s", @()),
  @('telex', "baajt${B}f", @()),
  @('telex', "baajt${B}n", @()),
  @('telex', "baajt${B}z", @()),
  @('telex', "baajt${B}${B}t", @()),
  @('telex', "baajt${B}${B}1", @()),
  @('telex', "bans${B}f", @()),
  @('telex', "bans${B}s", @()),
  @('vni', "ba65t${B}1", @()),
  @('vni', "ba65t${B}${B}t", @()),
  @('telex', "hoangf${B}g", @()),
  @('telex', "hoangf${B}s", @()),
  @('telex', "Nguyeexn${B}n", @()),
  @('telex', "Nguyeexn${B}e", @()),
  @('telex', "ddi${B}i", @()),
  @('telex', "dduowcj${B}c", @()),
  @('telex', "kieems${B}m", @()),
  @('telex', "kieems${B}s", @()),
  @('telex', "tieengs${B}g", @())
)
foreach ($c in $cases) { Show $c[0] $c[1] ([string[]]$c[2]) }
