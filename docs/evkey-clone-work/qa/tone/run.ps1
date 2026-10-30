# QA — tone change / partial deletion matrix (owner report 2026-10-30).
# One `vkb-cli simulate` run per row (char path through vkb_core::simulate).
# <BS> is a real 0x08 (VK_BACK) keystroke, <SP> a real space.
# Emits matrix.txt (TSV) and matrix.md (the table pasted into 57-tone-change-cases.md).
$ErrorActionPreference = 'Continue'
$exe = 'E:\vietkb\target\debug\vkb.exe'
$B = [char]8
$outDir = 'E:\vietkb\docs\evkey-clone-work\qa\tone'
New-Item -ItemType Directory -Force -Path $outDir | Out-Null

# id, method, charset, extra flags, keys, expected (NFC), source/rationale
$rows = @(
  ## ---------------- Family A: tone re-typed on a word already on screen ----------------
  @('A-01','telex','unicode','',        'baajts',        'bất',   'A4+A9: s replaces nang, sac legal before coda t'),
  @('A-02','telex','unicode','',        'baajtj',        'bật',   'EVKey: the same tone key on a finished word changes nothing; A1 escape table / E-TE-002 pin only the ADJACENT repeat'),
  @('A-03','telex','unicode','',        'baajtf',        'bậtf',  'A1 rule 5 + A9: huyen is illegal before coda t -> trigger literal'),
  @('A-04','telex','unicode','',        'baajtr',        'bậtr',  'A1 rule 5 + A9: hoi illegal before coda t'),
  @('A-05','telex','unicode','',        'baajtx',        'bậtx',  'A1 rule 5 + A9: nga illegal before coda t'),
  @('A-06','telex','unicode','',        'basf',          'bà',    'A4: no coda, huyen legal'),
  @('A-07','telex','unicode','',        'basr',          'bả',    'A4: no coda, hoi legal'),
  @('A-08','telex','unicode','',        'basx',          'bã',    'A4: no coda, nga legal'),
  @('A-09','telex','unicode','',        'basj',          'bạ',    'A4: no coda, nang legal'),
  @('A-10','telex','unicode','',        'bass',          'bas',   'A1 escape table / 50-test E-TE-002: adjacent repeat is the documented escape'),
  @('A-11','telex','unicode','',        'basz',          'ba',    'A1: z removes the tone'),
  @('A-12','telex','unicode','',        'az',            'az',    'A1: z with no tone present is literal'),
  @('A-13','telex','unicode','',        'toansz',        'toan',  'A1: z removes the tone of a coda word'),
  @('A-14','telex','unicode','',        'baajz',         'bâ',    'A1+A5: z removes the tone, the circumflex stays'),
  @('A-15','telex','unicode','',        'bans<BS>s',     'bas',   'A7 rebuilds raw bas; the repeated s is the documented ADJACENT escape (A-10 / E-TE-002 analogue)'),
  @('A-16','telex','unicode','',        'hoangff',       'hoangf','documented adjacent escape: raw hoangf is what E-TE-002 pins for a double press'),
  @('A-17','telex','unicode','',        'hoangfj',       'hoạng', 'A4: different tone replaces, tone on a of ang'),
  @('A-18','telex','unicode','',        'hoangfs',       'hoáng', 'A4: different tone replaces'),
  @('A-19','telex','unicode','',        'baajt<SP>f',    'bật f', 'A6: the space committed the word; f starts a new word with no vowel -> literal'),
  @('A-20','telex','unicode','',        'baajt<SP>s',    'bật s', 'A6: same, s is literal in the next word'),
  @('A-21','telex','unicode','',        'baajt<SP><SP>as','bật  á','A6: the tone key still reaches the new word'),
  @('A-22','telex','unicode','',        'baajt.f',       'bật.f', 'A6: punctuation commits; f is literal in the next word'),
  @('A-23','telex','unicode','',        'baajt.<BS>s',   'bất',   'A7 step 1: BS after a separator re-opens the word, s changes the tone'),
  @('A-24','telex','unicode','',        'Baajtf',        'Bậtf',  'A12 + A1 rule 5: case of the base key, trigger still illegal'),
  @('A-25','telex','unicode','',        'Baajts',        'Bất',   'A12: capital first letter, legal tone change'),
  @('A-26','telex','unicode','',        'BAATJf',        'BẬTf',  'A12 + A1 rule 5'),
  @('A-27','telex','unicode','',        'BAATJs',        'BẤT',   'A12'),
  @('A-28','telex','unicode','',        'bas<BS>as',     'bá',    'A7: delete the toned char, retype it with the same tone'),
  @('A-29','telex','unicode','',        'bas<BS>af',     'bà',    'A7: retype the vowel, then a different tone key'),
  @('A-30','telex','unicode','',        'bans<BS>f',     'bà',    'A7 step 4: the next key re-places the tone on the surviving word'),
  @('A-31','telex','unicode','',        'dduowcjf',      'đượcf', 'A9: huyen illegal before coda c'),
  @('A-32','telex','unicode','',        'dduowcjs',      'đước',  'A9: sac legal before coda c'),
  @('A-33','telex','unicode','',        'kieemsf',       'kiềm',  'A9: m is not in {c ch p t}, huyen legal'),
  @('A-34','telex','unicode','',        'dduowcjj',      'dduowcj','documented adjacent escape (raw minus the first trigger)'),
  @('A-35','vni','unicode','',          'ba65t1',        'bất',   'A2: 1 = sac, legal before t'),
  @('A-36','vni','unicode','',          'ba65t2',        'bật2',  'A2 rule 5 + A9: huyen illegal before t -> digit literal'),
  @('A-37','vni','unicode','',          'ba65t3',        'bật3',  'A2 rule 5 + A9: hoi illegal before t'),
  @('A-38','vni','unicode','',          'ba65t4',        'bật4',  'A2 rule 5 + A9: nga illegal before t'),
  @('A-39','vni','unicode','',          'ba65t5',        'bật',   'EVKey: the tone digit already in force changes nothing'),
  @('A-40','vni','unicode','',          'ba11',          'ba1',   'A2 digit escape: same digit repeated reverts + literal'),
  @('A-41','vni','unicode','',          'ba12',          'bà',    'A2: 2 = huyen'),
  @('A-42','vni','unicode','',          'ba15',          'bạ',    'A2: 5 = nang'),
  @('A-43','vni','unicode','',          'a60',           'â',     'A2: 0 removes the tone only'),
  @('A-44','vni','unicode','',          'ba650',         'bâ',    'A2+A5: 0 removes the tone, the circumflex stays'),
  @('A-45','vni','unicode','',          'ba65t0',        'bât',   'A2+A5: same with a coda'),
  @('A-46','vni','unicode','',          'ba10',          'ba',    'A2: 0 with no mark present leaves the word alone'),
  @('A-47','vni','unicode','',          'ba65t6',        'bật6',  'A2 rule 1: a digit with no target is literal (the 6 cannot mark an already-circumflexed a)'),
  @('A-48','vni','unicode','--no-free-tone','ba65t1',    'bất',   'A2 + free_tone off: different tone still replaces'),
  @('A-49','vni','unicode','--no-free-tone','ba65t5',    'bật',   'free_tone off must not change the same-tone rule'),
  @('A-50','telex','unicode','--old-tone-style','baajts','bất',   'tone style does not affect a coda word'),
  @('A-51','telex','unicode','--no-free-tone','baajts',  'bất',   'free_tone off: tone change still works'),
  @('A-52','telex','unicode','--no-spell-check','baajtf','bầt',   'A9: with spell check off the transform applies (control row)'),
  @('A-53','telex','unicode_nfd','',    'baajts',        'bất',   'same as A-01 in NFD'),
  @('A-54','telex','unicode_nfd','',    'baajtj',        'bật',   'same as A-02 in NFD'),
  @('A-55','telex','unicode_nfd','',    'baajtz',        'bât',   'same as A-14 in NFD'),

  ## ---------------- Family B: delete one composed unit, then retype ----------------
  @('B-01','telex','unicode','',        'baajt<BS>',     'bậ',    'A7 step 3: the press removes t, the tone stays on a'),
  @('B-02','telex','unicode','',        'baajt<BS>s',    'bấ',    'A7 step 4: the next key re-places the tone'),
  @('B-03','telex','unicode','',        'baajt<BS>st',   'bất',   'A7 step 4 + A4'),
  @('B-04','telex','unicode','',        'baajt<BS>f',    'bầ',    'A7 step 4: huyen legal on the surviving open syllable'),
  @('B-05','telex','unicode','',        'baajt<BS>n',    'bận',   'A7 step 4: an unrelated letter continues the word'),
  @('B-06','telex','unicode','',        'baajt<BS>b',    'bậb',   'A9: a plain letter is not a trigger, it appends'),
  @('B-07','telex','unicode','',        'baajt<BS>z',    'bâ',    'A1: z removes the tone of the surviving word'),
  @('B-08','telex','unicode','',        'baajt<BS><BS>', 'b',     'A7 step 2-3: one displayed character per press (b-a-t: a and its tone are one character)'),
  @('B-09','telex','unicode','',        'baajt<BS><BS>t','bt',    'A7 step 2-3: the second press also removes a'),
  @('B-10','telex','unicode','',        'baajt<BS><BS>b','bb',    'A7: a plain letter continues the two-letter onset'),
  @('B-11','telex','unicode','',        'Nguyeexn<BS>',  'Nguyễ', 'A7 step 3: the press removes n, the tone char is one character'),
  @('B-12','telex','unicode','',        'Nguyeexn<BS>n','Nguyễn','A7 step 4: retype the deleted letter'),
  @('B-13','telex','unicode','',        'Nguyeexn<BS>nn','Nguyễnn','A9: a plain letter appends'),
  @('B-14','telex','unicode','',        'Nguyeexn<BS>e','Nguyễe','A1 escape table applies to a doubled LETTER key (ee), not to a letter typed after a tone key'),
  @('B-15','telex','unicode','',        'Nguyeexn<BS>ee','Nguyễee','A9: two more letters append'),
  @('B-16','telex','unicode','',        'hoangf<BS>',    'hoàn',  'A7 step 3: the press removes g, tone stays on a'),
  @('B-17','telex','unicode','',        'hoangf<BS>g',   'hoàng', 'A7 step 4: retype the deleted letter'),
  @('B-18','telex','unicode','',        'hoangf<BS>s',   'hoán',  'A7 step 4 + A4: sac on the surviving syllable'),
  @('B-19','telex','unicode','',        'hoangf<BS>f',   'hoanf','documented adjacent escape: the rebuilt history ends in f, so the second f is the double press'),
  @('B-20','telex','unicode','',        'hoangf<SP><BS>s','hoáng','A7 step 1 + 4: reopen the committed word, then retone'),
  @('B-21','telex','unicode','',        'hoangf<SP><BS><BS>s','hoán','A7 step 1-4: reopen, delete g, retone'),
  @('B-22','telex','unicode','',        'hoangf<SP><BS><BS><SP>','hoàn ','A7 + A6: delete the last char and commit with space'),
  @('B-23','telex','unicode','',        'ddi<BS>',       'đ',     'A7 step 5: the press removes i, d stays barred'),
  @('B-24','telex','unicode','',        'ddi<BS>i',      'đi',    'A7 step 4: retype the deleted letter'),
  @('B-25','telex','unicode','',        'ddi<BS>e',      'đe',    'A9: a plain vowel after d appends'),
  @('B-26','telex','unicode','',        'ddi<BS>d',      'dd',    'A1 doubled-letter escape: ddd -> dd (documented)'),
  @('B-27','telex','unicode','',        'dduowcj<BS>',   'đượ',   'A7 step 5: the press removes c, the horn and tone stay'),
  @('B-28','telex','unicode','',        'dduowcj<BS>c',  'được',  'A7 step 4: retype the deleted letter'),
  @('B-29','telex','unicode','',        'dduowcj<BS>s',  'đướ',   'A7 step 4 + A4: sac on o of uo'),
  @('B-30','telex','unicode','',        'dduowcj<BS>j',  'dduowj','documented adjacent escape of the rebuilt history dduowj'),
  @('B-31','telex','unicode','',        'kieems<BS>',    'kiế',   'A7 step 3'),
  @('B-32','telex','unicode','',        'kieems<BS>m',   'kiếm',  'A7 step 4: retype the deleted letter'),
  @('B-33','telex','unicode','',        'kieems<BS>s',   'kiees', 'documented adjacent escape of the rebuilt history kiees'),
  @('B-34','telex','unicode','',        'kieems<BS>e',   'kiếe',  'A1 doubled-letter escape needs two e KEY presses in a row; here the previous key is the tone s'),
  @('B-35','telex','unicode','',        'tieengs<BS>',   'tiến',  'A7 step 2 (pinned by editing.rs)'),
  @('B-36','telex','unicode','',        'tieengs<BS>g',  'tiếng', 'A7 step 4: retype the deleted letter'),
  @('B-37','telex','unicode','',        'tieengs<BS>s',  'tieens','documented adjacent escape of the rebuilt history tieens (pinned by editing.rs for the reopen route)'),
  @('B-38','telex','unicode','',        'tieengs<BS>n',  'tiếnn', 'A9: a plain letter appends'),
  @('B-39','telex','unicode','',        'tieengs<SP><BS>s','tieengs','documented adjacent escape - pinned today by editing.rs::a_tone_trigger_on_a_reopened_toned_word_keeps_the_escape_rule'),
  @('B-40','telex','unicode','',        'toans<BS>',     'toá',   'A7 step 4: the remaining characters stay as displayed'),
  @('B-41','telex','unicode','',        'toans<BS>n',    'toán',  'A7 step 4: retype the deleted letter'),
  @('B-42','telex','unicode','',        'toans<BS>s',    'tóa',   'A7 step 4: the next key re-runs A4 (oa without coda -> tóa)'),
  @('B-43','telex','unicode','',        'toans<BS>f',    'tòa',   'A7 step 4 + A4'),
  @('B-44','telex','unicode','',        'toans<BS>r',    'tỏa',   'A7 step 4 + A4'),
  @('B-45','telex','unicode','',        'hoais<BS>',     'hoá',   'A7 step 4 (A4 no longer applies to oai without coda)'),
  @('B-46','telex','unicode','',        'hoais<BS>s',    'hóa',   'A7 step 4 + A4'),
  @('B-47','telex','unicode','',        'hoais<BS>f',    'hòa',   'A7 step 4 + A4'),
  @('B-48','telex','unicode','',        'hoais<BS>i',    'hoái',  'A7 step 4: retype the deleted letter'),
  @('B-49','telex','unicode','',        'thuys<BS>s',    'thus',  'documented adjacent escape of the rebuilt history thus'),
  @('B-50','telex','unicode','',        'thuys<BS>f',    'thù',   'A4: different tone replaces'),
  @('B-51','telex','unicode','',        'baajt<SP><BS><BS><SP>','bậ ','A7 step 1-4 + A6'),
  @('B-52','telex','unicode','',        'baajt<SP><BS><BS>n<SP>','bận ','A7 step 1-4 + A6'),
  @('B-53','telex','unicode','',        'baajt<SP><BS>s','bất',   'A7 step 1: reopen + different tone'),
  @('B-54','telex','unicode','',        'baajt<SP><BS>j','bật',   'A7 step 1: reopen + same tone = no change'),
  @('B-55','vni','unicode','',          'ba65t<BS>',     'bậ',    'A7 step 3'),
  @('B-56','vni','unicode','',          'ba65t<BS>1',    'bấ',    'A7 step 4: digit re-places the tone'),
  @('B-57','vni','unicode','',          'ba65t<BS>2',    'bầ',    'A7 step 4: huyen legal on the surviving open syllable'),
  @('B-58','vni','unicode','',          'ba65t<BS>t',    'bật',   'A7 step 4: retype the deleted letter'),
  @('B-59','vni','unicode','',          'ba65t<BS><BS>t','bt',    'A7 step 2-3: one displayed character per press'),
  @('B-60','vni','unicode','',          'ba65t<BS>5',    'ba65',  'documented adjacent digit escape of the rebuilt history ba65'),
  @('B-61','vni','unicode','',          'ba65t<BS>0',    'bâ',    'A2: 0 removes the tone only'),
  @('B-62','telex','unicode','--no-free-tone','baajt<BS>s','bấ',  'free_tone off: tone change after a deletion still works'),
  @('B-63','telex','unicode','--old-tone-style','baajt<BS>s','bấ','old tone style: same'),
  @('B-64','telex','unicode','--no-spell-check','baajt<BS>f','bầ','spell off: transform applies (control row)'),
  @('B-65','telex','unicode_nfd','',    'baajt<BS>',     'bậ',    'same as B-01 in NFD (the press deletes every unit)'),
  @('B-66','telex','unicode_nfd','',    'kieems<BS>m',   'kiếm',  'same as B-32 in NFD'),
  @('B-67','telex','unicode_nfd','',    'baajt<BS>s',    'bấ',    'same as B-02 in NFD'),
  @('B-68','telex','unicode_nfd','',    'Nguyeexn<BS>n','Nguyễn','same as B-12 in NFD')
)

function Norm([string]$s) { $s.Normalize([Text.NormalizationForm]::FormC) }

$tsv = New-Object System.Collections.Generic.List[string]
$md  = New-Object System.Collections.Generic.List[string]
$tsv.Add("id`tmethod`tcharset`toptions`tkeystrokes`texpected`tactual`tverdict`tsource")
$md.Add('| id | method | charset | options | keystrokes | expected | actual | verdict | source |')
$md.Add('|---|---|---|---|---|---|---|---|---|')

$pass = 0; $fail = 0
$fails = New-Object System.Collections.Generic.List[string]

foreach ($r in $rows) {
  $id, $m, $cs, $flags, $keys, $exp, $src = $r
  $shown = $keys                                    # <BS> / <SP> markers, as typed
  $arg = $keys.Replace('<SP>', ' ').Replace('<BS>', [string]$B)   # real space / 0x08 in the argument
  $argv = @('simulate', '--method', $m, '--charset', $cs)
  if ($flags -ne '') { $argv += $flags.Split(' ') }
  $argv += $arg
  $raw = (& $exe @argv 2>&1 | Out-String)
  $act = $raw.TrimEnd("`r", "`n")
  $verdict = if ((Norm $act) -ceq (Norm $exp)) { 'PASS' } else { 'FAIL' }
  if ($verdict -eq 'PASS') { $pass++ } else { $fail++; $fails.Add($id) }
  $tsv.Add(("{0}`t{1}`t{2}`t{3}`t{4}`t{5}`t{6}`t{7}`t{8}" -f $id, $m, $cs, $flags, $shown, $exp, $act, $verdict, $src))
  $tick = [char]96
  $md.Add(("| {0} | {1} | {2} | {3} | {9}{4}{9} | {5} | {6} | **{7}** | {8} |" -f $id, $m, $cs, ($(if($flags -eq ''){'-'}else{$flags})), $shown, $exp, $act, $verdict, $src, $tick))
}

$summary = New-Object System.Collections.Generic.List[string]
$summary.Add("rows: $($rows.Count)   PASS: $pass   FAIL: $fail")
$summary.Add("failing ids: $($fails -join ', ')")
$summary.Add("build: $exe  $((Get-Item $exe).Length) B  sha256 $((Get-FileHash $exe -Algorithm SHA256).Hash)")
$summary.Add("head: $(git -C E:\vietkb rev-parse --short HEAD) (+ uncommitted working tree)")
$summary.Add("harness: docs/evkey-clone-work/qa/tone/run.ps1")

Set-Content -Path "$outDir\matrix.txt" -Value (($summary + '' + $tsv) -join "`r`n") -Encoding utf8
Set-Content -Path "$outDir\matrix.md"  -Value (($md) -join "`r`n") -Encoding utf8
$summary | ForEach-Object { $_ }


