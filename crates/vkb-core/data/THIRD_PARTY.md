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

## SCOWL (Spell Checker Oriented Word Lists)

<http://wordlist.aspell.net/>, release 2020.12.07, by Kevin Atkinson. Used:
`final/english-words.{10,20,35,40,50,55,60}`. Licence, verbatim from the
release's `Copyright` file:

> The collective work is Copyright 2000-2018 by Kevin Atkinson as well as any
> of the copyrights mentioned below:
>
> Copyright 2000-2018 by Kevin Atkinson
>
> Permission to use, copy, modify, distribute and sell these word lists, the
> associated scripts, the output created from the scripts, and its documentation
> for any purpose is hereby granted without fee, provided that the above
> copyright notice appears in all copies and that both that copyright notice
> and this permission notice appear in supporting documentation. Kevin Atkinson
> makes no representations about the suitability of this array for any purpose.
> It is provided "as is" without express or implied warranty.

SCOWL's own sources (Moby Words II, public domain; the UK English Wordlist with
Frequency Classification, public domain; 12Dicts; ENABLE; and the others listed
in its `Copyright` file) are public domain or under the same kind of
permissive notice. Only the word spellings and the size level are used here.

## Tatoeba

<https://tatoeba.org>, Vietnamese sentences (`vie_sentences_detailed.tsv`,
downloaded 2026-11-01). Sentences are CC BY 2.0 FR (some CC0), contributed by
the Tatoeba community. Attribution: "Tatoeba contributors, tatoeba.org". Only
syllable and syllable-pair counts derived from the sentences are kept.

## Mozilla Common Voice sentence collector, Vietnamese

`server/data/vi/sentence-collector.txt` in
<https://github.com/common-voice/common-voice>. Common Voice sentences are
contributed under CC0 (public domain dedication). Only counts are kept.

## Written for VietKB (MIT)

`scripts/wordlists/tech-words.txt` (programming and tooling words),
`scripts/wordlists/loanwords.txt` (English words Vietnamese text borrows),
`scripts/wordlists/rimes-extra.txt` and `rimes-drop.txt` (rime curation), the
short function-word list in `src/lexicon.rs`, and the app table in
`src/appkind.rs`.

## Regenerating

```
python3 scripts/gen_wordlists.py --scowl <scowl>/final --tatoeba vie_sentences_detailed.tsv \
    --cv sentence-collector.txt --out crates/vkb-core/data
```

It builds `crates/vkb-core/examples/compose_words.rs` with cargo to find which
English words Telex changes. Review the diff of the output files before
committing.
