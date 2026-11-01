#!/usr/bin/env python3
"""Turn crates/vkb-core/tests/data/tech_mixed_vi.src.txt (the text a user means,
one paragraph per line) into the keystroke fixtures the engine tests and the
Mac QA scripts type:

  tech_mixed_vi.txt      Telex keystrokes, one paragraph per line
  tech_mixed_vi_vni.txt  VNI keystrokes, one paragraph per line

Words with a Vietnamese letter are converted; ASCII words (English, code, URLs,
numbers) are typed as they are. Run from the repo root:

  python3 scripts/gen_tech_fixture.py
"""
import os
import unicodedata

TONE_TELEX = {"́": "s", "̀": "f", "̉": "r", "̃": "x", "̣": "j"}
TONE_VNI = {"́": "1", "̀": "2", "̉": "3", "̃": "4", "̣": "5"}
MARK_TELEX = {"̂": "{c}", "̆": "w", "̛": "w"}  # circumflex doubles the letter
MARK_VNI = {"̂": "6", "̆": "8", "̛": "7"}


def convert_word(word, telex):
    keys, tone = [], ""
    for ch in word:
        upper = ch.isupper()
        low = ch.lower()
        if low == "đ":
            letter = "d" + ("d" if telex else "9")
            keys.append(letter.upper()[0] + letter[1:] if upper else letter)
            continue
        parts = unicodedata.normalize("NFD", low)
        base, marks = parts[0], parts[1:]
        out = base
        for mark in marks:
            if mark in (TONE_TELEX if telex else TONE_VNI):
                tone = (TONE_TELEX if telex else TONE_VNI)[mark]
            elif telex:
                out += base if mark == "̂" else "w"
            else:
                out += MARK_VNI[mark]
        keys.append(out.capitalize() if upper else out)
    return "".join(keys) + tone


def convert_line(line, telex):
    out, word = [], []

    def flush():
        if word:
            token = "".join(word)
            out.append(convert_word(token, telex) if not token.isascii() else token)
            word.clear()

    for ch in line:
        if ch.isalpha():
            word.append(ch)
        else:
            flush()
            out.append(ch)
    flush()
    return "".join(out)


def main():
    here = os.path.dirname(os.path.abspath(__file__))
    data = os.path.join(os.path.dirname(here), "crates", "vkb-core", "tests", "data")
    lines = [l.rstrip("\n") for l in open(os.path.join(data, "tech_mixed_vi.src.txt"), encoding="utf-8") if l.strip()]
    for name, telex in (("tech_mixed_vi.txt", True), ("tech_mixed_vi_vni.txt", False)):
        with open(os.path.join(data, name), "w", encoding="utf-8") as f:
            for line in lines:
                f.write(convert_line(line, telex) + "\n")
    words = sum(len(l.split()) for l in lines)
    print(f"{len(lines)} paragraphs, {words} words")


if __name__ == "__main__":
    main()
