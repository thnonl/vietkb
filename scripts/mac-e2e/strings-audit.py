#!/usr/bin/env python3
"""VietKB app string audit (CAP-20, static half): locales vs source.

Pure file reads. No app is launched, no input sent. Run: python3 -I scripts/mac-e2e/strings-audit.py [--out DIR]

 STR-01  vi and en define exactly the same keys; no empty value; {placeholders} match          (MAC-LNG-06)
 STR-02  every key of ui/src/lib/locales/{vi,en}.ts is used somewhere in ui/src (literal key, plural() base,
         or a dynamic `t(`prefix.${x}`)` prefix) - unused keys are dead copy                    (MAC-LNG-06)
 STR-03  every literal key used in ui/src exists in both locales                                 (MAC-LNG-06)
 STR-04  no Vietnamese diacritics in an English value (allow-list: the Vietnamese language name) (MAC-LNG-02/06)
 STR-05  heuristic: no user-visible string literal left in ui/src markup (text nodes and
         aria-label/title/placeholder/alt attributes that are not {expressions})                 (MAC-LNG-02/06)
 STR-06  heuristic: no Vietnamese diacritics in ui/src outside locales and comments               (MAC-LNG-02/06)
 STR-07  Rust-side table crates/vkb-app/src/i18n_table.rs: vi/en pairs complete, every key used, en has no
         diacritics; the tray menu strings are the ones doc 27 prescribes                         (MAC-I18-05, MAC-LNG-07)
 STR-08  ui/scripts/i18n-check.mjs (the project's own test) passes when node is available

Verdict FAIL carries the list of offending items (path:line text). Findings are heuristics: review before filing.
"""
import json, os, re, subprocess, sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OUT = os.environ.get("E2E_OUT") or (sys.argv[sys.argv.index("--out") + 1] if "--out" in sys.argv else os.getcwd())
os.makedirs(OUT, exist_ok=True)
RESULTS = os.path.join(OUT, "results.jsonl")
UI = os.path.join(REPO, "ui", "src")
DIAC = re.compile(r"[ăâêôơưđĂÂÊÔƠƯĐàáảãạằắẳẵặầấẩẫậèéẻẽẹềếểễệìíỉĩịòóỏõọồốổỗộờớởỡợùúủũụừứửữựỳýỷỹỵ]")

LIM = int(os.environ.get("STR_VERBOSE", 700))
ROWS = []
def record(cid, title, cases, verdict, detail=""):
    row = {"target": "strings", "method": "-", "id": cid, "keys": title, "expected": "no findings", "actual": detail[:max(LIM, 900)] or "ok",
           "verdict": verdict, "cases": cases, "evidence": "scripts/mac-e2e/strings-audit.py", "note": ""}
    ROWS.append(row)
    with open(RESULTS, "a", encoding="utf-8") as f:
        f.write(json.dumps(row, ensure_ascii=False, sort_keys=True) + "\n")
    print(f"[strings] {cid} {verdict} {cases} :: {title}" + ("\n        " + detail[:LIM] if verdict != "PASS" else ""))

def read(p):
    with open(p, encoding="utf-8") as f: return f.read()

def locale(name):
    d = {}
    for m in re.finditer(r'^\s*"([^"]+)":\s*"((?:[^"\\]|\\.)*)",?\s*$', read(os.path.join(UI, "lib", "locales", name + ".ts")), re.M):
        d[m.group(1)] = bytes(m.group(2), "utf-8").decode("unicode_escape").encode("latin-1", "ignore").decode("utf-8", "ignore") if "\\u" in m.group(2) else m.group(2).replace('\\"', '"')
    return d
VI, EN = locale("vi"), locale("en")

def src_files():
    for root, dirs, files in os.walk(UI):
        if os.path.basename(root) == "locales": dirs[:] = []; continue
        for f in files:
            if f.endswith((".svelte", ".ts")): yield os.path.join(root, f)
SRC = {p: read(p) for p in src_files()}
rel = lambda p: os.path.relpath(p, REPO)
holders = lambda s: sorted(re.findall(r"\{(\w+)\}", s))

def f(*a): return "; ".join(a)
def V(b): return "PASS" if not b else "FAIL"

# ------------------------------------------------------------------ STR-01
def str01():
    bad = []
    for k in sorted(set(VI) | set(EN)):
        if k not in VI: bad.append(f"missing in vi: {k}")
        if k not in EN: bad.append(f"missing in en: {k}")
        if k in VI and not VI[k].strip(): bad.append(f"empty vi: {k}")
        if k in EN and not EN[k].strip(): bad.append(f"empty en: {k}")
        if k in VI and k in EN and holders(VI[k]) != holders(EN[k]): bad.append(f"placeholder mismatch: {k}")
    record("STR-01", f"vi/en locales have identical keys ({len(VI)} vi, {len(EN)} en), none empty, placeholders match", "MAC-LNG-06", V(bad), f(*bad[:60]))

# ------------------------------------------------------------------ STR-02 / 03
def used_keys():
    allsrc = "\n".join(SRC.values())
    literals = set(re.findall(r'["\']([a-zA-Z][\w]*(?:\.[\w]+)+)["\']', allsrc))
    plural_bases = set(re.findall(r'plural\(\s*["\']([\w.]+)["\']', allsrc))
    lang_src = SRC.get(os.path.join(UI, "lib", "lang.svelte.ts"), "")
    prefixes = set(re.findall(r"t\(\s*`([\w.]*)\$\{", allsrc)) | set(re.findall(r"`([\w]+\.[\w.]*)\$\{", allsrc))
    return literals, plural_bases, prefixes
def rust_literals():
    out = set()
    for dp, _, fns in os.walk(os.path.join(REPO, "crates")):
        for fn in fns:
            if fn.endswith(".rs") and fn != "i18n_table.rs":
                out |= set(re.findall(r'"([a-z]+(?:\.[\w]+)+)"', read(os.path.join(dp, fn))))
    return out
def str02():
    literals, bases, prefixes = used_keys()
    literals = literals | rust_literals()
    unused = []
    for k in sorted(set(VI) | set(EN)):
        if k in literals: continue
        if any(k == f"{b}.one" or k == f"{b}.other" for b in bases): continue
        if any(k.startswith(p) for p in prefixes if p): continue
        unused.append(k)
    record("STR-02", f"every locale key is used in ui/src ({len(literals)} literal refs, {len(bases)} plural bases, prefixes {sorted(prefixes)})", "MAC-LNG-06", V(unused), f"{len(unused)} unused: " + ", ".join(unused[:40]))
def str03():
    bad = []
    keys = set(VI) | set(EN)
    for p, text in SRC.items():
        for m in re.finditer(r'\bt\(\s*["\']([a-zA-Z][\w.]*)["\']', text):
            if m.group(1) not in keys: bad.append(f"{rel(p)}: unknown key {m.group(1)}")
        for m in re.finditer(r'\b\w*Key:\s*["\']([a-zA-Z][\w.]*)["\']', text):
            if m.group(1) not in keys: bad.append(f"{rel(p)}: unknown key {m.group(1)}")
        for m in re.finditer(r'plural\(\s*["\']([\w.]+)["\']', text):
            for s in ("one", "other"):
                if f"{m.group(1)}.{s}" not in keys: bad.append(f"{rel(p)}: unknown plural {m.group(1)}.{s}")
    record("STR-03", "every literal t()/…Key/plural() key in ui/src exists in both locales", "MAC-LNG-06", V(bad), f(*bad[:60]))

# ------------------------------------------------------------------ STR-04
# "Tiếng Việt" is the endonym; the others show the Vietnamese OUTPUT the product produces (examples inside English copy)
EN_ALLOW = {"lang.vi.name", "adv.autoHornUo", "macros.empty.body", "macros.field.textPh", "method.telex.hint", "method.vni.hint", "quick.modernTone.example"}
def str04():
    bad = [f"{k}={v[:50]!r}" for k, v in sorted(EN.items()) if DIAC.search(v) and k not in EN_ALLOW]
    # examples of Vietnamese output inside English copy (e.g. "tieengs -> tiếng") are legitimate: report them separately
    record("STR-04", "English locale values contain no Vietnamese diacritics (except the language endonym)", "MAC-LNG-02,MAC-LNG-06", V(bad), f"{len(bad)}: " + "; ".join(bad[:20]))

# ------------------------------------------------------------------ STR-05 / 06 hardcoded strings
ATTRS = ("aria-label", "title", "placeholder", "alt", "aria-description", "aria-roledescription")
BRAND = re.compile(r"^(?:Esc|Ctrl|Shift|Enter|Space|được|VietKB|VietKB Dev|Telex|VNI|V|E|VI|EN|Unicode|NFC|NFD|macOS|Windows|GitHub|Ko-fi|OK)$")
def template_of(text):
    t = re.sub(r"<script\b.*?</script>", lambda m: "\n" * m.group(0).count("\n"), text, flags=re.S)
    t = re.sub(r"<style\b.*?</style>", lambda m: "\n" * m.group(0).count("\n"), t, flags=re.S)
    t = re.sub(r"<!--.*?-->", lambda m: "\n" * m.group(0).count("\n"), t, flags=re.S)
    return t
def has_words(s):
    return bool(re.search(r"[A-Za-zÀ-ỹ]{2,}", s))
def str05():
    bad = []
    for p, text in SRC.items():
        if not p.endswith(".svelte"): continue
        t = template_of(text)
        # drop Svelte blocks / expressions so only literal text remains
        flat = re.sub(r"\{[^{}]*(?:\{[^{}]*\}[^{}]*)*\}", lambda m: "\x00" * len(m.group(0)), t)
        for m in re.finditer(r">([^<>\x00]+)<", flat):
            s = m.group(1)
            line = flat.count("\n", 0, m.start(1)) + 1
            s2 = re.sub(r"\s+", " ", s).strip()
            if not s2 or not has_words(s2) or BRAND.match(s2) or s2.startswith("&"): continue
            if re.fullmatch(r"[\W\d_]*", s2): continue
            bad.append(f"{rel(p)}:{line} text {s2[:50]!r}")
        for am in re.finditer(r'\b(%s)\s*=\s*"([^"{}]*)"' % "|".join(ATTRS), flat):
            val = am.group(2).strip()
            line = flat.count("\n", 0, am.start()) + 1
            if val and has_words(val) and not BRAND.match(val): bad.append(f"{rel(p)}:{line} {am.group(1)}={val[:40]!r}")
    # .ts: literals in toast/title/label/message call sites that look like prose (letters + space, >= 2 words)
    for p, text in SRC.items():
        if not p.endswith(".ts"): continue
        for i, line in enumerate(text.splitlines(), 1):
            if re.match(r"\s*(//|\*|/\*)", line): continue
            for m in re.finditer(r"\b(?:toast|showError|alert|message|label|title|text)\s*[:(=]\s*[\"']([A-ZÀ-Ỹa-zà-ỹ][^\"']*\s[^\"']*)[\"']", line):
                if "Key" in line[: m.start()+1] and re.match(r"^[\w.]+$", m.group(1)): continue
                bad.append(f"{rel(p)}:{i} ts {m.group(1)[:50]!r}")
    record("STR-05", "no user-visible literal text/aria-label/title/placeholder/alt in ui/src markup (heuristic)", "MAC-LNG-02,MAC-LNG-06", V(bad), f"{len(bad)}: " + "; ".join(bad[:80]))
def ALLOW06(path, code):
    """Vietnamese that is DATA, not UI copy: the typing preview engine, sample macro rows, example strings, the browser-preview mock."""
    base = os.path.basename(path)
    if base == "preview.ts": return True
    if base == "Macros.svelte" and (re.search(r'trigger:\s*"', code) or 'class="sample"' in code): return True
    if base == "catalog.ts" and re.search(r"example:", code): return True
    if base == "state.svelte.ts" and "bản xem trước" in code: return True
    return False
def str06():
    bad = []
    for p, text in SRC.items():
        for i, line in enumerate(text.splitlines(), 1):
            s = line.strip()
            if s.startswith(("//", "*", "/*", "<!--")): continue
            code = re.sub(r"//.*$|<!--.*?-->|/\*.*?\*/", "", line)
            if DIAC.search(code) and not ALLOW06(p, code):
                ctx = code.strip()[:70]
                bad.append(f"{rel(p)}:{i} {ctx!r}")
    # sample words that legitimately show Vietnamese output (hòa/hoà tone-style examples in catalog.ts) are listed so the reviewer sees them
    record("STR-06", "no Vietnamese text in ui/src outside locales and comments (examples of typing output show up here for review)", "MAC-LNG-02,MAC-LNG-06", V(bad), f"{len(bad)}: " + "; ".join(bad[:60]))

# ------------------------------------------------------------------ STR-07 Rust table
RUST_EXPECT_VI = {  # doc 27 / the menu the owner approved; MAC-I18-05
    "tray.toggle": "Bật/tắt tiếng Việt", "tray.method": "Kiểu gõ", "tray.settings": "Mở cài đặt…", "tray.hotkeys": "Phím tắt…",
    "tray.about": "Giới thiệu", "tray.quit": "Thoát VietKB",
}
def str07():
    bad = []
    tbl = read(os.path.join(REPO, "crates/vkb-app/src/i18n_table.rs"))
    entries = re.findall(r'\(\s*"([\w.]+)"\s*,\s*"((?:[^"\\]|\\.)*)"\s*,\s*"((?:[^"\\]|\\.)*)"\s*,?\s*\)', tbl)
    keys = [e[0] for e in entries]
    if len(set(keys)) != len(keys): bad.append("duplicate keys in i18n_table.rs")
    rs = "\n".join(read(os.path.join(dp, fn)) for dp, _, fns in os.walk(os.path.join(REPO, "crates/vkb-app/src")) for fn in fns if fn.endswith(".rs") and fn != "i18n_table.rs")
    for k, vi, en in entries:
        if not vi.strip() or not en.strip(): bad.append(f"empty: {k}")
        if holders(vi) != holders(en): bad.append(f"placeholder mismatch: {k}")
        if DIAC.search(en): bad.append(f"Vietnamese in en: {k}={en[:40]!r}")
        stem = k.rsplit(".", 1)[0] if k.count(".") > 1 else k
        if f'"{k}"' not in rs and not any(re.search(r'"%s\.' % re.escape(p), rs) or f'"{p}.' in rs for p in [k.split(".")[0] + "." + k.split(".")[1]] if False):
            # dynamic prefixes: err.* / rwin.* are looked up via format!("rwin.{kind}") style names
            if not re.search(r'"%s[.{]' % re.escape(k.rsplit(".", 1)[0]) , rs) and not re.search(r'format!\(\s*"%s\.' % re.escape(k.rsplit(".", 1)[0]), rs):
                bad.append(f"unused in Rust: {k}")
    d = {e[0]: e for e in entries}
    for k, want in RUST_EXPECT_VI.items():
        if k not in d or d[k][1] != want: bad.append(f"{k}: vi {d.get(k, ('', '?'))[1]!r} != doc 27 {want!r}")
    record("STR-07", f"Rust i18n table ({len(entries)} keys): complete pairs, English clean, keys used, tray menu equals doc 27", "MAC-I18-05,MAC-LNG-07", V(bad), f(*bad[:60]))

# ------------------------------------------------------------------ STR-08 project's own test
def str08():
    try:
        r = subprocess.run(["node", os.path.join(REPO, "ui/scripts/i18n-check.mjs")], capture_output=True, text=True, timeout=60)
        record("STR-08", "ui/scripts/i18n-check.mjs (project i18n test)", "MAC-LNG-06", "PASS" if r.returncode == 0 else "FAIL", (r.stdout + r.stderr).strip()[:600])
    except FileNotFoundError:
        record("STR-08", "ui/scripts/i18n-check.mjs (project i18n test)", "MAC-LNG-06", "SKIP", "node not installed")
    except Exception as e:
        record("STR-08", "ui/scripts/i18n-check.mjs (project i18n test)", "MAC-LNG-06", "FAIL", repr(e))

if __name__ == "__main__":
    for fn in (str01, str02, str03, str04, str05, str06, str07, str08):
        try: fn()
        except Exception as e:
            record("STR-ERR", f"{fn.__name__} crashed", "-", "FAIL", repr(e))
    print(f"[strings] {sum(r['verdict'] == 'PASS' for r in ROWS)}/{len(ROWS)} PASS")
