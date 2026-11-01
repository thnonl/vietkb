# Third-party data in `vkb-core/data` (D10, 0.2.8)

VietKB is MIT. Everything here is either written for VietKB or derived from
sources under permissive terms. No data is fetched at run time: the files are
generated once by `scripts/gen_wordlists.py` and embedded in the binary.

| file | what | derived from | licence |
|---|---|---|---|
| `english.txt` | English words (levels 1-7) that Telex composes into something else, about 1,350 words | SCOWL 2020.12.07 `english-words.10 … .60` + `scripts/wordlists/tech-words.txt` + `loanwords.txt` | SCOWL licence (below); the two small lists are written for VietKB (MIT) |
| `vi.txt` | Vietnamese syllables and how often they occur (about 2,600 syllables seen at least 4 times) | syllable counts over Tatoeba Vietnamese sentences and the Common Voice Vietnamese sentence list | counts only, see below |
| `pairs.bin` | hashed pairs `previous syllable, syllable` with a count, for the ambiguous syllables only (about 4,300 pairs) | the same two corpora | counts only, see below |
| `consts.rs` | the generated prefix table of Vietnamese rimes, totals | `vi.txt` corpus rimes, curated by `rimes-extra.txt` / `rimes-drop.txt` | counts only |

No sentence, and no word sequence longer than a hashed syllable pair, is
stored. The bigram table is a set of 32-bit hashes with counts.

The full notices (SCOWL copyright verbatim, Tatoeba credit and licence, Common Voice) are in
[`THIRD_PARTY_NOTICES.md`](../../../THIRD_PARTY_NOTICES.md) at the repository root; that file is bundled with the app.
Regenerate the data with `python3 scripts/gen_wordlists.py` (see the script's docstring).
