#!/usr/bin/env python3
"""Generate the D10 lexicon files embedded by `vkb-core` (crates/vkb-core/data).

Inputs (download them yourself; see crates/vkb-core/data/THIRD_PARTY.md):
  --scowl   directory `scowl-2020.12.07/final` (SCOWL, permissive licence)
  --tatoeba vie_sentences_detailed.tsv (Tatoeba, CC BY 2.0 FR / CC0)
  --cv      common-voice server/data/vi/sentence-collector.txt (CC0)
  tech-words.txt (this directory, written for VietKB, MIT)

Output (written to --out):
  english.txt  `word` + level (digit 1-7 = SCOWL 10..60, tech words = 2; a
               letter a-g instead of the digit marks a loanword), sorted by word. Only words that Telex
               composes into something other than the typed keys (the only ones
               the engine ever has to restore).
  vi.txt       `key count`, sorted by key. Vietnamese syllables seen >= MIN_COUNT
               times; key is the ASCII syllable key described in lexicon.rs.
  consts.rs    generated Rust constants.

Needs `cargo` (builds the vkb-core example `compose_words`).
"""
import argparse, collections, glob, os, re, subprocess, sys, unicodedata

MIN_COUNT = 1
RIME_MIN = 3
LEVELS = ["10", "20", "35", "40", "50", "55", "60"]
LEVEL_DIGIT = {lv: str(i + 1) for i, lv in enumerate(LEVELS)}
TECH_LEVEL = "2"  # same as SCOWL 20

MARKS = {"̂": "^", "̆": "(", "̛": "+"}
TONES = {"́": "1", "̀": "2", "̉": "3", "̃": "4", "̣": "5"}


def key_of(token):
    """ASCII syllable key: letters with marks (^ ( +, d-bar = D) then tone digit."""
    token = unicodedata.normalize("NFC", token.lower())
    out, tone = [], ""
    for ch in unicodedata.normalize("NFD", token.replace("đ", "ð")):
        if ch == "ð":
            out.append("D")
        elif ch in MARKS:
            out.append(MARKS[ch])
        elif ch in TONES:
            tone = TONES[ch]
        elif "a" <= ch <= "z":
            out.append(ch)
        else:
            return None
    return "".join(out) + tone


NUC = r"[aeiouy]{1,3}"
SYL = re.compile(r"^(ngh|ng|nh|ph|th|tr|ch|gh|gi|kh|qu|[bcdghklmnpqrstvx])?(" + NUC + r")(ch|ng|nh|c|m|n|p|t)?$")
SYL_FOREIGN = re.compile(r"^(ngh|ng|nh|ph|th|tr|ch|gh|gi|kh|qu|[bcdghklmnpqrstvxfjwz])?(" + NUC + r")(ch|ng|nh|c|m|n|p|t)?$")


def valid_key(key, foreign=False):
    tone = key[-1] if key[-1].isdigit() else ""
    body = key[:-1] if tone else key
    plain = re.sub(r"[\^(+]", "", body).replace("D", "d")
    m = (SYL_FOREIGN if foreign else SYL).match(plain)
    if not m:
        return False
    if not foreign and m.group(3) in ("c", "ch", "p", "t") and tone not in ("1", "5"):
        return False
    return True


def fnv1a32(text):
    h = 0x811C9DC5
    for b in text.encode():
        h = ((h ^ b) * 0x01000193) & 0xFFFFFFFF
    return h


def read_english(scowl, tech_file):
    levels = {}
    for lv in LEVELS:
        path = os.path.join(scowl, "english-words." + lv)
        for line in open(path, encoding="latin-1"):
            w = line.strip()
            if re.fullmatch(r"[a-z]{2,24}", w) and w not in levels:
                levels[w] = LEVEL_DIGIT[lv]
    for line in open(tech_file):
        w = line.strip()
        if w and not w.startswith("#"):
            levels[w] = min(levels.get(w, TECH_LEVEL), TECH_LEVEL)
    # An inflected form (`cats`, `its`) is rarer than its base: never "common".
    for w in list(levels):
        if w.endswith("s") and w[:-1] in levels and levels[w] < "3":
            levels[w] = "3"
    return levels


def read_loans(path):
    return {l.strip() for l in open(path) if l.strip() and not l.startswith("#")}


def compose(words, repo):
    """raw -> composed text, for default options and for `jfwz_as_consonant = false`.
    Second result: the same with spell check off (`order` -> `ỏde`)."""
    out = {}
    nospell = {}
    for env in ({}, {"COMPOSE_NO_JFWZ": "1"}, {"COMPOSE_NO_SPELL": "1"}):
        res = subprocess.run(
            ["cargo", "run", "-q", "-p", "vkb-core", "--example", "compose_words"],
            input="\n".join(words), text=True, capture_output=True, cwd=repo,
            env={**os.environ, **env}, check=True)
        for line in res.stdout.splitlines():
            raw, _, composed = line.partition("\t")
            if composed and composed != raw:
                (nospell if "COMPOSE_NO_SPELL" in env else out).setdefault(raw, set()).add(composed)
    return out, nospell


def read_vietnamese(tatoeba, cv):
    sentences = []
    for line in open(tatoeba, encoding="utf-8"):
        parts = line.rstrip("\n").split("\t")
        if len(parts) >= 3:
            sentences.append(parts[2])
    sentences += [l.strip() for l in open(cv, encoding="utf-8")]
    unigrams, bigrams = collections.Counter(), collections.Counter()
    seen = set()
    for sentence in sentences:
        if sentence in seen:
            continue
        seen.add(sentence)
        for clause in re.split(r"[^\w\s]|_|\d", unicodedata.normalize("NFC", sentence)):
            prev = None
            for token in clause.split():
                key = key_of(token)
                if key is None or not valid_key(key):
                    prev = None
                    continue
                unigrams[key] += 1
                if prev is not None:
                    bigrams[(prev, key)] += 1
                prev = key
    return unigrams, bigrams


ONSETS = ["ngh", "ng", "nh", "ph", "th", "tr", "ch", "gh", "gi", "kh", "qu"] + list("bcdghklmnpqrstvxfjwz")


def rime_of(key):
    """Mark-free, tone-free rime of a syllable key (onset stripped, longest
    onset that leaves a vowel first)."""
    body = key[:-1] if key[-1].isdigit() else key
    plain = re.sub(r"[\^(+]", "", body).replace("D", "d")
    for onset in [o for o in ONSETS if len(o) == 3] + [o for o in ONSETS if len(o) == 2] + [o for o in ONSETS if len(o) == 1] + [""]:
        if plain.startswith(onset):
            rest = plain[len(onset):]
            if rest and rest[0] in "aeiouy":
                return rest
    return None


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--scowl", required=True)
    ap.add_argument("--tatoeba", required=True)
    ap.add_argument("--cv", required=True)
    ap.add_argument("--out", default="crates/vkb-core/data")
    ap.add_argument("--exclude", default="scripts/wordlists/english-wins.txt")
    args = ap.parse_args()
    here = os.path.dirname(os.path.abspath(__file__))
    repo = os.path.dirname(here)
    os.makedirs(args.out, exist_ok=True)

    english = read_english(args.scowl, os.path.join(here, "wordlists", "tech-words.txt"))
    loans = read_loans(os.path.join(here, "wordlists", "loanwords.txt"))
    techs = read_loans(os.path.join(here, "wordlists", "tech-words.txt"))
    escape_deny = read_loans(os.path.join(here, "wordlists", "escape-deny.txt"))
    escape_allow = read_loans(os.path.join(here, "wordlists", "escape-allow.txt"))
    idents = read_loans(os.path.join(here, "wordlists", "ident-words.txt"))
    names = read_loans(os.path.join(here, "wordlists", "names.txt"))
    for w in loans | idents | names | escape_allow:
        english.setdefault(w, TECH_LEVEL)
    composed, nospell = compose(sorted(english), repo)
    unigrams, bigrams = read_vietnamese(args.tatoeba, args.cv)
    total = sum(unigrams.values())

    vi = {k: c for k, c in unigrams.items() if c >= MIN_COUNT}
    # Syllables the owner wants English for even though a corpus saw them
    # (D10: user, use, were, here ...). One composed form per line.
    wins = set()
    path = os.path.join(repo, args.exclude)
    if os.path.exists(path):
        for line in open(path):
            line = line.strip()
            if line and not line.startswith("#"):
                wins.add(line)
    for k in list(vi):
        if k in wins:
            del vi[k]

    rimes = collections.Counter()
    for k, c in unigrams.items():
        r = rime_of(k)
        if r:
            rimes[r] += c
    if os.environ.get("DUMP_RIMES"):
        for r, c in sorted(rimes.items()):
            print(f"RIME {r} {c}")
    keep = {r for r, c in rimes.items() if c >= RIME_MIN}
    extra = os.path.join(here, "wordlists", "rimes-extra.txt")
    drop = os.path.join(here, "wordlists", "rimes-drop.txt")
    for path, op in ((extra, keep.add), (drop, keep.discard)):
        if os.path.exists(path):
            for line in open(path):
                line = line.strip()
                if line and not line.startswith("#"):
                    op(line)
    # A token whose rime Vietnamese does not have (`yanh`, a name or an English
    # word in a sentence) is not a Vietnamese syllable.
    vi = {k: c for k, c in vi.items() if rime_of(k) in keep}

    cand = {}
    for raw, forms in composed.items():
        keys = set()
        for form in forms:
            k = key_of(form)
            if k and valid_key(k, foreign=True):
                keys.add(k)
        # A doubled tone key typed in an English word (`pass`, `error`, `coffee`)
        # is swallowed as an undo: the screen shows `pas`, `eror`. Restore those
        # when the English word is common (SCOWL size 20 or less) and the
        # collapsed text is not itself a common English word (`ass` -> `as`, the
        # Telex idiom for typing a literal `as`; `lass` -> `las` stays too).
        if not keys and raw not in escape_deny and (english[raw] <= "2" or raw in escape_allow):
            for form in forms:
                if len(form) < len(raw) and form.isascii() and (
                        raw in escape_allow or english.get(form.lower(), "9") > "2"):
                    keys.add("!" + form.lower())
        if keys:
            cand[raw] = keys
    # Spell check off: the structural check no longer keeps `order`, `teacher`
    # raw, so the common English words (size 20 or less, brands, code words) are
    # listed for the word break and the mid-word switch as well.
    # Only forms that cannot be a Vietnamese syllable matter: a plausible one
    # stays Vietnamese (`mootj` -> `một`), list or no list.
    def garbled(forms):
        for form in forms:
            k = key_of(form)
            if k and valid_key(k, foreign=True):
                return False
        return True

    for raw in nospell:
        if raw not in cand and raw not in escape_deny and garbled(nospell[raw]) and (
                english[raw] <= "1" or raw in names or raw in idents or raw in loans):
            cand[raw] = {"!nospell"}
    ambiguous = {k for keys in cand.values() for k in keys if k in vi}

    with open(os.path.join(args.out, "english.txt"), "w") as f:
        for w in sorted(cand):
            level = english[w]
            if w in loans:
                level = chr(ord("a") + int(level) - 1)
            elif w in idents:
                level = chr(ord("h") + int(level) - 1)
            elif w in names:
                level = chr(ord("o") + int(level) - 1)
            elif w in techs:
                level = chr(ord("A") + int(level) - 1)
            f.write(w + level + "\n")
    with open(os.path.join(args.out, "vi.txt"), "w") as f:
        for k in sorted(vi):
            f.write(f"{k} {vi[k]}\n")
    with open(os.path.join(args.out, "rimes.txt"), "w") as f:
        for r in sorted(keep):
            f.write(r + "\n")
    symbols = {c: i + 1 for i, c in enumerate("aeiouychmnptg")}
    prefixes = set()
    for r in keep:
        assert len(r) <= 5 and all(c in symbols for c in r), r
        for n in range(1, len(r) + 1):
            code = 0
            for c in r[:n]:
                code = code * 14 + symbols[c]
            prefixes.add(code)
    with open(os.path.join(args.out, "consts.rs"), "w") as f:
        f.write("// @generated by scripts/gen_wordlists.py - do not edit.\n")
        f.write("/// Every prefix of every Vietnamese rime, base-14 coded (a=1 e i o u y c h m n p t g=13).\n")
        f.write("pub static RIME_PREFIXES: [u32; %d] = [\n" % len(prefixes))
        ordered = sorted(prefixes)
        for i in range(0, len(ordered), 12):
            f.write("    " + ", ".join(str(c) for c in ordered[i:i + 12]) + ",\n")
        f.write("];\n")
    print(f"english candidates {len(cand)} of {len(english)}; vi syllables {len(vi)} "
          f"(tokens {total}); ambiguous vi {len(ambiguous)}", file=sys.stderr)
    if os.environ.get("DUMP_AMBIGUOUS"):
        for raw in sorted(cand):
            for k in sorted(cand[raw]):
                if k in vi:
                    print(f"{raw}\t{k}\t{vi[k]}\tL{english[raw]}")


if __name__ == "__main__":
    main()
