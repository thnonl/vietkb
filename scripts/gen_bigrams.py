#!/usr/bin/env python3
"""Generate crates/vkb-core/data/bigrams.rs: the word-pair tables of the D10 two-sided rule.

An *ambiguous* word is a listed English word (data/english.txt, minus the function words and
determiners, which keep the 0.2.8 rules) whose Telex reading is a valid Vietnamese syllable that
the corpus knows (`max`/mã, `box`/bõ, `mix`/mĩ) or that is a level-1 word with no corpus count
(`room`/rôm, `soon`/sôn). Two tables, both sorted FNV-1a 32 hashes (no text is stored):

  ENGLISH_PAIRS  "w1 w2" for English sentences, when w1 or w2 is an ambiguous word that is a real
            Vietnamese syllable (a word with no corpus count is English by default), seen >= --en-min times.
  VIETNAMESE_PAIRS  "k1 k2" (syllable keys, see lexicon.rs) for Vietnamese sentences, when k1 or k2 is the
            key of an ambiguous word's Vietnamese reading, seen >= --vi-min times.

Inputs (download yourself, run with `python3 -I`; never commit them):
  --eng  eng_sentences_detailed.tsv  (Tatoeba English, CC BY 2.0 FR)
  --vie  vie_sentences_detailed.tsv  (Tatoeba Vietnamese, CC BY 2.0 FR)
  --cv   sentence-collector.txt      (Common Voice Vietnamese, CC0)
Needs cargo (the compose_words example), like gen_wordlists.py.
Also writes a report (--report) with the sizes, the conflict list and the coverage probes.
"""
import argparse, collections, importlib.util, os, re, sys, unicodedata

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(HERE)
DATA = os.path.join(REPO, "crates", "vkb-core", "data")

spec = importlib.util.spec_from_file_location("gen_wordlists", os.path.join(HERE, "gen_wordlists.py"))
gw = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gw)

def lexicon_list(name):
    """A `static NAME: &[&str] = &[...]` list of lexicon.rs."""
    text = open(os.path.join(REPO, "crates", "vkb-core", "src", "lexicon.rs"), encoding="utf-8").read()
    body = re.search(r"static " + name + r": &\[&str\] = &\[(.*?)\];", text, re.S).group(1)
    return set(re.findall(r'"([a-z]+)"', body))


# Function words, determiners and the other closed-class words keep the 0.2.8 behaviour.
FUNCTION_AND_DETERMINERS = lexicon_list("FUNCTION_WORDS") | lexicon_list("DETERMINERS") | lexicon_list("CLOSED_EXTRA")

PROBES_EN = ["meeting room", "room service", "mail box", "post box", "box office", "max speed", "mix tape",
             "bus stop", "tax return", "gas station", "moon light", "full moon", "tree house", "see you",
             "dining room", "living room", "show room", "chat room", "file max", "bed room", "room mate",
             "black box", "mix up", "six pack", "soon as", "door bell", "more than", "tax free", "bus driver"]
PROBES_VI = ["rôm rả", "rôm sảy", "cá trê", "mã nguồn", "mã số", "mĩ thuật", "bõ công", "sĩ quan", "tã lót",
             "gá chồng", "bú sữa", "môn học", "sê ri", "mã vạch", "phòng rôm"]


def fnv(text):
    return gw.fnv1a32(text)


def tokens_en(text):
    return re.findall(r"[a-z]+", text.lower())


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--eng", required=True)
    ap.add_argument("--vie", required=True)
    ap.add_argument("--cv", required=True)
    ap.add_argument("--en-min", type=int, default=3)
    ap.add_argument("--vi-min", type=int, default=1)
    ap.add_argument("--report", default=None)
    args = ap.parse_args()

    levels = {}
    for line in open(os.path.join(DATA, "english.txt")):
        line = line.strip()
        if line:
            levels[line[:-1]] = line[-1]
    vi_count = {}
    for line in open(os.path.join(DATA, "vi.txt")):
        k, _, c = line.strip().partition(" ")
        vi_count[k] = int(c)

    words = sorted(w for w in levels if w not in FUNCTION_AND_DETERMINERS)
    composed, _ = gw.compose(words, REPO)
    ambiguous = {}  # word -> set of vietnamese keys
    for w in words:
        for c in composed.get(w, ()):
            key = gw.key_of(c)
            if key is None or not gw.valid_key(key):
                continue
            if vi_count.get(key, 0) > 0 or levels[w] == "1":
                ambiguous.setdefault(w, set()).add(key)
    # Words the engine keeps Vietnamese although the corpus lacks them (lexicon.rs REAL_BUT_UNCOUNTED).
    # The words the engine keeps Vietnamese although the corpus lacks the reading (lexicon.rs).
    repo = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    with open(os.path.join(repo, "crates/vkb-core/src/lexicon.rs"), encoding="utf-8") as f:
        lexicon_src = f.read()
    block = re.search(r"static REAL_BUT_UNCOUNTED[^=]*=\s*&\[(.*?)\];", lexicon_src, re.S)
    assert block, "REAL_BUT_UNCOUNTED not found in lexicon.rs"
    keep_vietnamese = set(re.findall(r'b"([a-z]+)"', block.group(1)))
    assert keep_vietnamese, "REAL_BUT_UNCOUNTED is empty"
    amb_keys = {k for ks in ambiguous.values() for k in ks}
    # English is already the default for a word with no corpus count (`room`): its English pairs
    # decide nothing. Only words that are real Vietnamese syllables need them.
    # Rarer English forms (SCOWL level 3+, plurals) keep the old rule: Vietnamese wins.
    # A loanword, name, identifier or tech tag (a letter) always counts; a SCOWL digit counts
    # only at levels 1 and 2. (The earlier two-clause form had the same truth table.)
    real = (keep_vietnamese & set(ambiguous)) | {
        w for w, ks in ambiguous.items()
        if any(vi_count.get(k, 0) > 0 for k in ks) and (not levels[w].isdigit() or levels[w] in "12")}
    print(f"ambiguous words: {len(ambiguous)}, keys: {len(amb_keys)}", file=sys.stderr)

    # English pairs.
    en_pairs = collections.Counter()
    seen = set()
    for line in open(args.eng, encoding="utf-8"):
        parts = line.rstrip("\n").split("\t")
        if len(parts) < 3 or parts[2] in seen:
            continue
        seen.add(parts[2])
        toks = tokens_en(parts[2])
        for a, b in zip(toks, toks[1:]):
            if a in real or b in real:
                en_pairs[(a, b)] += 1
    en_hashes = sorted({fnv(f"{a} {b}") for (a, b), n in en_pairs.items() if n >= args.en_min})

    # Vietnamese pairs.
    _, vi_bigrams = gw.read_vietnamese(args.vie, args.cv)
    vi_hashes = sorted({fnv(f"{a} {b}") for (a, b), n in vi_bigrams.items()
                        if n >= args.vi_min and (a in amb_keys or b in amb_keys)})

    out = os.path.join(DATA, "bigrams.rs")
    with open(out, "w") as f:
        f.write("// @generated by scripts/gen_bigrams.py - do not edit.\n")
        f.write("// Sorted FNV-1a 32 hashes of word pairs (no text). See the script for how they are chosen.\n")
        for name, hs in (("ENGLISH_PAIRS", en_hashes), ("VIETNAMESE_PAIRS", vi_hashes)):
            f.write(f"pub static {name}: [u32; {len(hs)}] = [\n")
            for i in range(0, len(hs), 8):
                f.write("    " + ", ".join(str(h) for h in hs[i:i + 8]) + ",\n")
            f.write("];\n")
    print(f"EN pairs {len(en_hashes)}, VI pairs {len(vi_hashes)} -> {out}", file=sys.stderr)

    if args.report:
        en_set, vi_set = set(en_hashes), set(vi_hashes)
        with open(args.report, "w") as r:
            r.write(f"ambiguous words {len(ambiguous)}; EN pairs {len(en_hashes)}; VI pairs {len(vi_hashes)}\n\n")
            r.write("== probes (EN) ==\n")
            for p in PROBES_EN:
                a, b = p.split()
                r.write(f"{p}: raw count {en_pairs.get((a, b), 0)}, in table {fnv(p) in en_set}\n")
            r.write("\n== probes (VI) ==\n")
            for p in PROBES_VI:
                a, b = (gw.key_of(t) for t in p.split())
                r.write(f"{p}: corpus count {vi_bigrams.get((a, b), 0)}, in table {fnv(f'{a} {b}') in vi_set}\n")
            # Conflicts: an ambiguous word with both kinds of neighbour, and a neighbour that
            # is both an English word and a Vietnamese syllable giving an attested pair of each kind.
            r.write("\n== ambiguous words with both an English and a Vietnamese attested pair ==\n")
            for w in sorted(ambiguous):
                en_n = sorted({(a if b == w else b) for (a, b), n in en_pairs.items()
                               if n >= args.en_min and w in (a, b)})
                vi_n = sorted({(a if b in ambiguous[w] else b) for (a, b), n in vi_bigrams.items()
                               if n >= args.vi_min and (a in ambiguous[w] or b in ambiguous[w])})
                if en_n and vi_n:
                    r.write(f"{w} ({'/'.join(sorted(ambiguous[w]))}): EN {len(en_n)} e.g. {en_n[:6]}; VI {len(vi_n)} e.g. {vi_n[:6]}\n")
            r.write("\n== same-neighbour conflicts (one neighbour spelled the same in both tables) ==\n")
            comp_rev = collections.defaultdict(set)
            for w, cs in composed.items():
                for c in cs:
                    k = gw.key_of(c)
                    if k:
                        comp_rev[k].add(w)
            n_conf = 0
            for w in sorted(ambiguous):
                for k in ambiguous[w]:
                    for (a, b), n in vi_bigrams.items():
                        if n < args.vi_min:
                            continue
                        if a == k and b in vi_count:
                            for x in comp_rev.get(b, ()):  # english word x whose reading is b
                                if en_pairs.get((w, x), 0) >= args.en_min:
                                    r.write(f"{w} {x}  EN x{en_pairs[(w, x)]}  vs  VI {k} {b} x{n}\n"); n_conf += 1
                        if b == k and a in vi_count:
                            for x in comp_rev.get(a, ()):
                                if en_pairs.get((x, w), 0) >= args.en_min:
                                    r.write(f"{x} {w}  EN x{en_pairs[(x, w)]}  vs  VI {a} {k} x{n}\n"); n_conf += 1
            r.write(f"total {n_conf}\n")


if __name__ == "__main__":
    main()
