# QA stream B — CLI reproduction table for D-SC-01..05 / §A9 amendment 2026-10-30.
# Every row is a real `vkb-cli simulate` run (char path through vkb-core::simulate).
$ErrorActionPreference = 'Continue'
Set-Location E:\vietkb
New-Item -ItemType Directory -Force -Path docs\evkey-clone-work\qa\verify | Out-Null
$out = 'docs\evkey-clone-work\qa\verify\cli.txt'
Remove-Item $out -ErrorAction SilentlyContinue

$cases = @(
  # --- 1. the owner report and its siblings (D-SC-01) ---
  @{m='vni';   s='ba65t/ta8t1';        n='owner repro VNI'},
  @{m='telex'; s='baajt/tawts';        n='owner repro Telex (tawts=tắt)'},
  @{m='telex'; s='baajt/taats';        n='audit spelling (taats=tất)'},
  @{m='vni';   s='/ta8t1';             n='leading /'},
  @{m='vni';   s='ba65t//ta8t1';       n='double /'},
  @{m='vni';   s='ba65t /ta8t1';       n='space then /'},
  @{m='vni';   s='ba65t/ta8t1/ba65t';  n='chain of three'},
  @{m='vni';   s='Ba65t/Ta8t1';        n='uppercase'},
  # --- 2. the five VNI triggers after / ---
  @{m='vni';   s='ba65t/a8';           n='trigger a8 after /'},
  @{m='vni';   s='ba65t/a61';          n='trigger a61 after /'},
  @{m='vni';   s='ba65t/d9';           n='trigger d9 after /'},
  @{m='vni';   s='ba65t/o7';           n='trigger o7 after /'},
  @{m='vni';   s='ba65t/u7';           n='trigger u7 after /'},
  # --- 3. the other separators (D-SC-02/03/04) ---
  @{m='vni';   s='ba65t.ta8t1';        n='dot'},
  @{m='vni';   s='ba65t?ta8t1';        n='question'},
  @{m='vni';   s='ba65t_ta8t1';        n='underscore'},
  @{m='vni';   s='ba65t@ta8t1';        n='at'},
  @{m='vni';   s='ba65t:ta8t1';        n='colon'},
  @{m='vni';   s='ba65t=ta8t1';        n='equals'},
  @{m='vni';   s='ba65t#ta8t1';        n='hash'},
  @{m='vni';   s='ba65t\ta8t1';        n='backslash'},
  # --- 4. §2b chained openers (D-SC-05) ---
  @{m='telex'; s='C:\Users\nam';       n='win path Telex'},
  @{m='vni';   s='C:\Users\nam';       n='win path VNI'},
  @{m='telex'; s='C:/Users/nam';       n='win path fwd Telex'},
  @{m='vni';   s='C:/Users/nam';       n='win path fwd VNI'},
  @{m='telex'; s='C:\\server\share';   n='UNC Telex'},
  @{m='vni';   s='C:\\server\share';   n='UNC VNI'},
  @{m='telex'; s='https://x/y/z';      n='https path Telex'},
  @{m='vni';   s='https://x/y/z';      n='https path VNI'},
  @{m='telex'; s='http://a.b/c';       n='http path Telex'},
  @{m='vni';   s='http://a.b/c';       n='http path VNI'},
  # --- 5. §2b negative controls + still-composing pairs ---
  @{m='telex'; s='xyz./tieesng';       n='NEG: . then / must not chain'},
  @{m='telex'; s='baajt:_/tieesng';    n='NEG: _/ must not chain'},
  @{m='telex'; s='baajt/taawts';       n='compose after /'},
  @{m='telex'; s='baajt//taawts';      n='compose after //'},
  @{m='telex'; s='baajt /taawts';      n='compose after space + /'},
  @{m='telex'; s='/taawts';            n='compose after leading /'},
  @{m='vni';   s='a/b';                n='a/b'},
  @{m='vni';   s='2/3';                n='2/3'},
  @{m='telex'; s='baajt.http';         n='dot then word'},
  @{m='telex'; s='baajt./taawts';      n='./ then word'},
  @{m='telex'; s='baajt:_/taawts';     n=':_/ then word'},
  # --- 6. guard list (00-charter §9 / raw_zone.rs) ---
  @{m='telex'; s='github.cof';         n='guard github.cof'},
  @{m='vni';   s='a@b.com';            n='guard a@b.com'},
  @{m='vni';   s='file_name';          n='guard file_name'},
  @{m='vni';   s='x=cof';              n='guard x=cof'},
  @{m='vni';   s='me@cof';             n='guard me@cof'},
  @{m='telex'; s='a. tieengs';         n='guard a. + space'},
  @{m='telex'; s='a.';                 n='guard a.'},
  @{m='telex'; s='tiengs';             n='guard tiengs'},
  @{m='vni';   s='1.5';                n='guard 1.5'},
  @{m='vni';   s='3.5';                n='guard 3.5'},
  @{m='vni';   s='%20';                n='guard %20'},
  @{m='vni';   s='--opt';              n='guard --opt'},
  @{m='telex'; s='...';                n='guard ellipsis'},
  @{m='telex'; s='a...cof';            n='guard ellipsis word'},
  @{m='telex'; s='3.cof';              n='guard 3.cof'},
  # --- 7. new-failure hunt: arm/asymmetry around the fix ---
  @{m='telex'; s='tooi ddang owr C:\Users\nam';      n='NEW: path mid-sentence'},
  @{m='telex'; s='baajt.C:\x';                       n='NEW: bật.C:\x'},
  @{m='telex'; s='a.b/c';                            n='NEW: a.b/c'},
  @{m='telex'; s='e.g./i.e.';                        n='NEW: e.g./i.e.'},
  @{m='vni';   s='v1.2.3/tag';                       n='NEW: v1.2.3/tag'},
  @{m='vni';   s='100%/50%';                         n='NEW: 100%/50%'},
  @{m='telex'; s='#hashtag/tieesng';                 n='NEW/L2: #hashtag/tieesng'},
  @{m='vni';   s='abc/d9uo7ng2';                     n='NEW/L2: abc/d9uo7ng2'},
  @{m='telex'; s='baajt https://x';                  n='NEW: URL after VN word'},
  @{m='telex'; s='baajt:https://x';                  n='NEW: : then https'},
  @{m='telex'; s='cof:/tiengs';                      n='L5: cof:/tiếng?'},
  @{m='vni';   s='a:/ta8t1';                         n='L5: a:/tắt?'},
  @{m='telex'; s='abc://tieesng';                    n='L5: abc://tiếng?'},
  @{m='telex'; s='x=@cof';                           n='L1: x=@cof'},
  @{m='telex'; s='a./cof';                           n='L1: a./cof'},
  @{m='telex'; s='ban.cof';                          n='L3: ban.cof'},
  @{m='telex'; s='baajt.C:\Users\nam';               n='NEW: VN word then path'}
)

"# vkb-cli simulate — QA re-verification 2026-10-30 (stream B, second pass)" | Out-File $out -Encoding utf8
"# repo E:\vietkb · $(git rev-parse --short HEAD) · $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss zzz')" | Out-File $out -Append -Encoding utf8
"# each block: command line as run, then the exact stdout" | Out-File $out -Append -Encoding utf8

foreach ($c in $cases) {
  $cmd = "cargo run -q -p vkb-cli -- simulate --method $($c.m) `"$($c.s)`""
  "" | Out-File $out -Append -Encoding utf8
  "# [$($c.n)] $cmd" | Out-File $out -Append -Encoding utf8
  $res = & cargo run -q -p vkb-cli -- simulate --method $c.m $c.s 2>&1
  $code = $LASTEXITCODE
  ($res | Out-String).TrimEnd() | Out-File $out -Append -Encoding utf8
  "  (exit $code)" | Out-File $out -Append -Encoding utf8
}
Write-Host "done -> $out"
