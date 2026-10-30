# Narrowest repro per distinct failure root (see 57-tone-change-cases.md).
# Raw `vkb-cli simulate --actions` traces: every EngineAction the engine returned.
$ErrorActionPreference = 'Continue'
$exe = 'E:\vietkb\target\debug\vkb.exe'
$B = [char]8
$out = 'E:\vietkb\docs\evkey-clone-work\qa\tone\repro.txt'
Remove-Item $out -ErrorAction SilentlyContinue
function Add-Text([string]$s) { Add-Content -Path $out -Value $s -Encoding utf8 }

Add-Text "vkb-cli $(& $exe --version 2>&1)  |  exe $((Get-Item $exe).Length) B  sha256 $((Get-FileHash $exe -Algorithm SHA256).Hash)"
Add-Text "head $(git -C E:\vietkb rev-parse --short HEAD) + uncommitted working tree (engine.rs A9 amendment + host fixes)"
Add-Text "char path (vkb_core::simulate): one KeyInput per char, <BS> = KeyInput::backspace() (vk 0x08)"
Add-Text ""

function Trace([string]$title, [string]$m, [string[]]$flags, [string]$keys) {
  $arg = $keys.Replace('<SP>', ' ').Replace('<BS>', [string]$B)
  $shown = $keys
  Add-Text "=== $title"
  Add-Text "--- $m $(($flags) -join ' ') $shown"
  $argv = @('simulate', '--method', $m, '--actions') + $flags + @($arg)
  (& $exe @argv 2>&1) | ForEach-Object { Add-Text $_ }
  Add-Text ""
}

Trace 'D-TN-01 same tone key re-typed, coda letter between the two presses' 'telex' @() 'baajtj'
Trace 'D-TN-01 same tone digit re-typed, coda letter in between (A2 says "immediately after")' 'vni' @() 'ba65t5'
Trace 'D-TN-01 same tone key after a Backspace' 'telex' @() 'tieengs<BS>s'
Trace 'D-TN-01 same tone key after a reopen (space, BS)' 'telex' @() 'baajt<SP><BS>j'
Add-Text "control: the documented adjacent escape the rule above was copied from"
Trace 'control A-10' 'telex' @() 'bass'
Trace 'D-TN-02 z must remove the tone (A1: asz -> a)' 'telex' @() 'basz'
Trace 'D-TN-02 z with a circumflex word (A1+A5: baajz -> ba-circumflex)' 'telex' @() 'baajz'
Trace 'D-TN-03 VNI 0 must remove the tone only (A2)' 'vni' @() 'a60'
Trace 'D-TN-03 VNI 0 on a toned circumflex word' 'vni' @() 'ba65t0'
Trace 'D-TN-04 tone change to a tone that is illegal before the coda (A1 rule 5 / A9)' 'telex' @() 'baajtf'
Trace 'D-TN-04 VNI digit 2 before coda t' 'vni' @() 'ba65t2'
Add-Text "control: the same illegal tone on a fresh word - the pre-existing sibling of D-TN-04"
Trace 'control tacf' 'telex' @() 'tacf'
Trace 'D-TN-05 trigger with no target must be a literal digit (A2 rule 1)' 'vni' @() 'ba65t6'
Trace 'D-TN-06 after BS the next key must re-place the tone (A7 step 4)' 'telex' @() 'toans<BS>f'
Trace 'D-TN-06 same, tone already in force' 'telex' @() 'toans<BS>s'
Trace 'D-TN-06 oai shape' 'telex' @() 'hoais<BS>f'
Trace 'D-TN-07 a plain letter is not a trigger: it appends (A9)' 'telex' @() 'baajt<BS>b'
Trace 'D-TN-07 same without any Backspace - the shape is not deletion-specific' 'telex' @() 'baajtb'
Trace 'D-TN-08 a letter whose base equals the last COMPOSED letter must append, not escape the mark' 'telex' @() 'Nguyeexn<BS>e'
Trace 'D-TN-08 same on i-e' 'telex' @() 'kieems<BS>e'
Trace 'control: same shape where the previous key really is the doubled letter (spec escape)' 'telex' @() 'Nguyeexne'

Add-Text "=== coverage check: --method legacy"
(& $exe simulate --method legacy 'baajt' 2>&1) | ForEach-Object { Add-Text $_ }
Add-Text ""
Add-Text "ENGINE POINTERS (measured tree):"
Add-Text "  apply_tone same-tone branch        crates/vkb-core/src/method.rs:402-407"
Add-Text "  step_telex has no 'z' arm         crates/vkb-core/src/method.rs:250-259"
Add-Text "  step_digit '0' clears every mark  crates/vkb-core/src/method.rs:147-155"
Add-Text "  apply_tone never consults spell   crates/vkb-core/src/method.rs:392-412 (no spell:: call)"
Add-Text "  mark_last no-target escape        crates/vkb-core/src/method.rs:452-455"
Add-Text "  compose literalize on invalid     crates/vkb-core/src/method.rs:102-110"
Add-Text "  double_letter reads the composed last letter, not the previous key  crates/vkb-core/src/method.rs:263-283"
Add-Text "  backspace freeze fallback         crates/vkb-core/src/engine.rs:317-333"

