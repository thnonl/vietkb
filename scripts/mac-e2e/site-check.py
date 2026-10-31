#!/usr/bin/env python3
"""VietKB static site / docs checker (CAP-19; also the grep half of CAP-18 for MAC-DOC cases).

Pure file reads under the repo: no network, no browser, no process is launched, no input is sent, so it needs
no idle gate and touches nothing of the owner's. Run: python3 -I scripts/mac-e2e/site-check.py [--out DIR]

Checks `site/index.html` (vi), `site/en/index.html` (en), `site/404.html`, README.md, release-notes-mac-draft.md and
`.github/workflows/pages.yml`. Each check appends one row to $E2E_OUT/results.jsonl (same schema as the
Swift harness; target = "site") and prints it. Exit code is always 0; the verdict is in the rows.
"""
import json, os, re, sys
from html.parser import HTMLParser

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OUT = os.environ.get("E2E_OUT") or (sys.argv[sys.argv.index("--out") + 1] if "--out" in sys.argv else os.getcwd())
os.makedirs(OUT, exist_ok=True)
RESULTS = os.path.join(OUT, "results.jsonl")

def rd(rel):
    with open(os.path.join(REPO, rel), encoding="utf-8") as f:
        return f.read()

ROWS = []
def record(cid, title, cases, verdict, detail=""):
    row = {"target": "site", "method": "-", "id": cid, "keys": title, "expected": "no findings", "actual": detail[:900] or "ok",
           "verdict": verdict, "cases": cases, "evidence": "scripts/mac-e2e/site-check.py", "note": ""}
    ROWS.append(row)
    with open(RESULTS, "a", encoding="utf-8") as f:
        f.write(json.dumps(row, ensure_ascii=False, sort_keys=True) + "\n")
    print(f"[site] {cid} {verdict} {cases} :: {title}" + (f"\n        {detail[:600]}" if verdict != "PASS" else ""))

def verdict(findings):
    return "PASS" if not findings else "FAIL"

# ---------------------------------------------------------------- parsing helpers
VOID = {"meta", "link", "img", "br", "hr", "input", "source", "path", "circle", "rect", "polygon", "use", "area", "base", "col", "wbr"}
class Page(HTMLParser):
    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.tags = []        # (tag, attrs dict, line)
        self.ids = set()
        self.text = []        # (line, text) of visible text outside script/style
        self.scripts = []     # inline script bodies
        self._stack = []
    def handle_starttag(self, tag, attrs):
        a = dict(attrs)
        self.tags.append((tag, a, self.getpos()[0]))
        if "id" in a: self.ids.add(a["id"])
        if tag not in VOID: self._stack.append(tag)
    def handle_endtag(self, tag):
        if tag in self._stack:
            while self._stack and self._stack.pop() != tag: pass
    def handle_data(self, data):
        cur = self._stack[-1] if self._stack else ""
        if cur == "script": self.scripts.append(data)
        elif cur == "style": pass
        elif data.strip(): self.text.append((self.getpos()[0], data.strip()))

def parse(rel):
    p = Page(); raw = rd(rel); p.feed(raw); p.raw = raw; p.rel = rel
    return p

VI = parse("site/index.html")
EN = parse("site/en/index.html")
NF = parse("site/404.html")
PAGES = {"vi": VI, "en": EN}
DIAC = re.compile(r"[ăâêôơưđĂÂÊÔƠƯĐàáảãạằắẳẵặầấẩẫậèéẻẽẹềếểễệìíỉĩịòóỏõọồốổỗộờớởỡợùúủũụừứửữựỳýỷỹỵ]")

def strip_tags(s):
    s = re.sub(r"<script.*?</script>|<style.*?</style>|<!--.*?-->|<svg.*?</svg>", " ", s, flags=re.S)
    s = re.sub(r"<[^>]+>", " ", s)
    return re.sub(r"\s+", " ", s)

def section_slice(p, h3_id):
    """Raw HTML of one `<h3 class="platform-title" id=...>` block: up to the next h3 or the end of its <section>."""
    m = re.search(r'<h3[^>]*id="%s"[^>]*>' % re.escape(h3_id), p.raw)
    if not m: return None
    rest = p.raw[m.end():]
    e = re.search(r"<h3\b|</section>", rest)
    return rest[: e.start()] if e else rest

def meta(p, name=None, prop=None):
    for tag, a, _ in p.tags:
        if tag == "meta" and ((name and a.get("name") == name) or (prop and a.get("property") == prop)):
            return a.get("content", "")
    return None

# ---------------------------------------------------------------- MAC-LNG-21: every relative link / asset resolves
def resolve(page_rel, ref):
    ref = ref.split("#")[0].split("?")[0]
    if not ref: return None
    if ref.startswith("/vietkb/"): path = os.path.join(REPO, "site", ref[len("/vietkb/"):])   # GitHub Pages project root
    elif ref.startswith("/"): path = os.path.join(REPO, "site", ref.lstrip("/"))
    else: path = os.path.join(os.path.dirname(os.path.join(REPO, page_rel)), ref)
    path = os.path.normpath(path)
    if os.path.isdir(path): path = os.path.join(path, "index.html")
    return path

def check_links():
    bad, n = [], 0
    for lang, p in [("vi", VI), ("en", EN), ("404", NF)]:
        for tag, a, line in p.tags:
            for attr in ("href", "src", "poster"):
                v = a.get(attr)
                if not v or re.match(r"(https?:|mailto:|data:|tel:|javascript:)", v): continue
                if v.startswith("#"):
                    if v[1:] and v[1:] not in p.ids: bad.append(f"{p.rel}:{line} anchor {v} has no id")
                    continue
                n += 1
                path = resolve(p.rel, v)
                if path and not os.path.exists(path): bad.append(f"{p.rel}:{line} {attr}={v} -> missing")
                elif path and "#" in v and path.endswith(".html"):
                    frag = v.split("#", 1)[1]
                    other = parse(os.path.relpath(path, REPO))
                    if frag and frag not in other.ids: bad.append(f"{p.rel}:{line} {v}: id '{frag}' not in target")
    record("SITE-01", f"all {n} relative href/src/poster and #anchors resolve from /, /en/ and 404", "MAC-LNG-21,MAC-LNG-28", verdict(bad), "; ".join(bad))

# ---------------------------------------------------------------- MAC-LNG-20: lang attr + VI|EN switch
def check_lang_switch():
    bad = []
    for lang, p in PAGES.items():
        html = next((a for t, a, _ in p.tags if t == "html"), {})
        if html.get("lang") != lang: bad.append(f"{p.rel}: <html lang={html.get('lang')!r}> != {lang!r}")
        sw = {a.get("hreflang"): a for t, a, _ in p.tags if t == "a" and a.get("hreflang") in ("vi", "en") and a.get("lang") in ("vi", "en")}
        if set(sw) != {"vi", "en"}: bad.append(f"{p.rel}: header switch has {sorted(sw)}, want vi+en"); continue
        if sw[lang].get("aria-current") != "page": bad.append(f"{p.rel}: current-language link lacks aria-current=page")
        other = "en" if lang == "vi" else "vi"
        target = resolve(p.rel, sw[other]["href"])
        want = os.path.join(REPO, "site", "en" if other == "en" else "", "index.html")
        if not target or os.path.normpath(target) != os.path.normpath(want): bad.append(f"{p.rel}: switch to {other} goes to {sw[other]['href']}")
        if not sw[lang].get("aria-label") or not sw[other].get("aria-label"): bad.append(f"{p.rel}: switch links need aria-label")
    record("SITE-02", "<html lang> = vi / en and the VI|EN header switch links to the equivalent page, aria-current set", "MAC-LNG-20", verdict(bad), "; ".join(bad))

# ---------------------------------------------------------------- MAC-LNG-22: head metadata
def check_head():
    bad = []
    titles, descs = {}, {}
    for lang, p in PAGES.items():
        t = next((" ".join(x for _, x in p.text if False) for _ in []), None)
        m = re.search(r"<title>(.*?)</title>", p.raw, re.S); titles[lang] = m.group(1).strip() if m else ""
        descs[lang] = meta(p, name="description") or ""
        if not titles[lang]: bad.append(f"{p.rel}: no <title>")
        if not descs[lang]: bad.append(f"{p.rel}: no meta description")
        alts = {a.get("hreflang"): a.get("href") for t_, a, _ in p.tags if t_ == "link" and a.get("rel") == "alternate" and a.get("hreflang")}
        if set(alts) != {"vi", "en", "x-default"}: bad.append(f"{p.rel}: hreflang set {sorted(alts)} != vi,en,x-default")
        canon = next((a.get("href") for t_, a, _ in p.tags if t_ == "link" and a.get("rel") == "canonical"), None)
        if canon != alts.get(lang): bad.append(f"{p.rel}: canonical {canon} != own hreflang {alts.get(lang)}")
        loc = meta(p, prop="og:locale")
        if loc != ("vi_VN" if lang == "vi" else "en_US"): bad.append(f"{p.rel}: og:locale {loc}")
        if not meta(p, prop="og:locale:alternate"): bad.append(f"{p.rel}: og:locale:alternate missing")
        if not meta(p, name="theme-color"): bad.append(f"{p.rel}: theme-color missing")
        robots = meta(p, name="robots") or ""
        if "noindex" in robots: bad.append(f"{p.rel}: noindex on a content page")
    if PAGES["vi"] and titles["vi"] == titles["en"]: bad.append("vi and en titles identical")
    if descs["vi"] == descs["en"]: bad.append("vi and en descriptions identical")
    if "noindex" not in (meta(NF, name="robots") or ""): bad.append("404.html should be noindex")
    a_vi = {a.get("hreflang"): a.get("href") for t_, a, _ in VI.tags if t_ == "link" and a.get("rel") == "alternate"}
    a_en = {a.get("hreflang"): a.get("href") for t_, a, _ in EN.tags if t_ == "link" and a.get("rel") == "alternate"}
    if a_vi != a_en: bad.append("hreflang alternates differ between the two pages (must cross-link identically)")
    record("SITE-03", "per-language title/description/canonical/hreflang(vi,en,x-default)/og:locale/theme-color; noindex only on 404", "MAC-LNG-22", verdict(bad), "; ".join(bad))

# ---------------------------------------------------------------- MAC-LNG-23: parity
def numbers(p):
    t = strip_tags(re.sub(r"<head>.*?</head>", "", p.raw, flags=re.S))
    t = re.sub(r"\b(?:id|class|width|height|viewBox)=\S+", "", t)
    return sorted(set(n.replace(",", ".") for n in re.findall(r"\d+(?:[.,]\d+)?", t)))
def check_parity():
    bad = []
    def count(p, tag, cls=None):
        return sum(1 for t, a, _ in p.tags if t == tag and (cls is None or cls in a.get("class", "").split()))
    for what, tag, cls in [("sections", "section", None), ("h3 platform titles", "h3", "platform-title"), ("FAQ <details>", "details", None),
                           ("install steps", "li", "step"), ("shortcut table rows", "tr", None), ("screenshots", "img", None),
                           ("kbd keys", "kbd", None), ("notes", "div", "note"), ("headings h2", "h2", None), ("headings h4", "h4", None)]:
        a, b = count(VI, tag, cls), count(EN, tag, cls)
        if a != b: bad.append(f"{what}: vi {a} != en {b}")
    sv = [a.get("aria-labelledby") for t, a, _ in VI.tags if t == "section"]; se = [a.get("aria-labelledby") for t, a, _ in EN.tags if t == "section"]
    if len(sv) == len(se): pass
    nv, ne = set(numbers(VI)), set(numbers(EN))
    if nv ^ ne: bad.append(f"numeric facts only in one language: vi-only {sorted(nv - ne)} en-only {sorted(ne - nv)}")
    # the two pages must reference the same set of external URLs
    ev = {a["href"] for t, a, _ in VI.tags if t == "a" and a.get("href", "").startswith("http")}
    ee = {a["href"] for t, a, _ in EN.tags if t == "a" and a.get("href", "").startswith("http")}
    if ev ^ ee: bad.append(f"external links differ: vi-only {sorted(ev - ee)} en-only {sorted(ee - ev)}")
    record("SITE-04", "vi and en pages carry the same sections, steps, FAQ, tables, screenshots, numeric facts and external links", "MAC-LNG-23,MAC-DOC-01", verdict(bad), "; ".join(bad))

# ---------------------------------------------------------------- download links without JS (MAC-DOC-02 / MAC-LNG-25)
def check_downloads():
    bad = []
    for lang, p in PAGES.items():
        links = [(a, line) for t, a, line in p.tags if t == "a" and "releases" in a.get("href", "")]
        # the anchor text is not in `tags`; read it from the raw HTML around each releases link
        txts = re.findall(r'<a [^>]*href="https://github\.com/thnonl/vietkb/releases/latest"[^>]*>(.*?)</a>', p.raw, re.S)
        flat = [re.sub(r"\s+", " ", strip_tags(t)).strip() for t in txts]
        want_win = r"(Tải cho Windows)" if lang == "vi" else r"(Download for Windows)"
        want_mac = r"(Tải cho macOS)" if lang == "vi" else r"(Download for macOS)"
        if not any(re.search(want_win, t) for t in flat): bad.append(f"{p.rel}: no static Windows download link ({flat[:4]})")
        if not any(re.search(want_mac, t) for t in flat): bad.append(f"{p.rel}: no static macOS download link ({flat[:4]})")
        # neither may be hidden by default
        for t, a, line in p.tags:
            if t == "a" and a.get("id") in ("dl-primary", "dl-alt") and "hidden" in a: bad.append(f"{p.rel}:{line} download anchor hidden without JS")
        js = "\n".join(p.scripts)
        if not re.search(r"\\\.dmg\$", js) or not re.search(r"\\\.exe\$", js): bad.append(f"{p.rel}: apply() does not read both .dmg and .exe assets")
        if "userAgentData" not in js and "navigator.userAgent" not in js: bad.append(f"{p.rel}: no OS detection")
    record("SITE-05", "both Windows and macOS download links are present and visible without JS; script reads .exe and .dmg", "MAC-DOC-02,MAC-LNG-25", verdict(bad), "; ".join(bad))

# ---------------------------------------------------------------- stale static version (MAC-DOC-03 / MAC-LNG-30)
def check_fallback():
    bad = []
    ver = re.search(r'"version":\s*"([^"]+)"', rd("crates/vkb-app/tauri.conf.json")).group(1)
    for lang, p in PAGES.items():
        for m in re.finditer(r"\bv(\d+\.\d+\.\d+)\b", strip_tags(p.raw)):
            if m.group(1) != ver: bad.append(f"{p.rel}: static v{m.group(1)} != product {ver}")
    if "v0.1.1" in VI.raw + EN.raw: bad.append("stale v0.1.1 fallback still present")
    record("SITE-06", f"static fallbacks carry no stale version (product {ver})", "MAC-DOC-03,MAC-LNG-30", verdict(bad), "; ".join(bad))

# ---------------------------------------------------------------- macOS section: no Windows-only claims (MAC-DOC-01/04/05/11/12, MAC-LNG-24)
MAC_FORBID = {  # Windows-only vocabulary that must not appear in the macOS install section
    r"\.exe\b": ".exe", r"SmartScreen": "SmartScreen", r"khay hệ thống|system tray|\btray\b": "tray (Mac: menu bar)",
    r"Khởi động cùng Windows|Start with Windows|with Windows": "Windows autostart wording",
    r"\bF1\b|\bF2\b": "F1/F2 as Mac defaults", r"VK_PACKET": "VK_PACKET", r"Windows 10|Windows 11": "Windows versions",
    r"Nhấp trái|[Ll]eft-click": "left-click toggles (Windows behaviour)", r"nhấp đúp|double-click": "double click",
}
def check_mac_sections():
    bad = []
    for lang, p, hid in [("vi", VI, "cai-mac"), ("en", EN, "install-mac")]:
        sec = section_slice(p, hid)
        if sec is None: bad.append(f"{p.rel}: no macOS install block #{hid}"); continue
        txt = strip_tags(sec)
        for pat, label in MAC_FORBID.items():
            for m in re.finditer(pat, txt): bad.append(f"{p.rel} #{hid}: '{label}' ... {txt[max(0, m.start()-30):m.end()+30].strip()!r}")
        need = {"dmg": r"\.dmg", "Applications": r"Applications", "Gatekeeper/blocked path": r"Open Anyway|Vẫn mở|quarantine|Gatekeeper",
                "Accessibility": r"Accessibility|Trợ năng"}
        for label, pat in need.items():
            if not re.search(pat, txt): bad.append(f"{p.rel} #{hid}: missing {label}")
    record("SITE-07", "macOS install sections (vi + en): no Windows-only claims, and dmg / Applications / Gatekeeper / Accessibility are explained", "MAC-DOC-01,MAC-DOC-04,MAC-DOC-05,MAC-DOC-12,MAC-LNG-24", verdict(bad), "; ".join(bad))

# ---------------------------------------------------------------- stale-claim greps from the DOC catalog
STALE = [  # (case, regex, label, applies to)
    ("MAC-DOC-01", r"Chưa có bản cho macOS|No (?:build|version) for macOS|macOS is not available", "'no macOS build' claim"),
    ("MAC-DOC-04", r"Nhấp trái biểu tượng V/E để (?:bật|tắt)|[Ll]eft-click the V/E icon to (?:turn|switch|toggle)(?! on Windows)", "left click toggles V/E stated without the Windows qualifier"),
    ("MAC-DOC-06", r"Chín hành động|[Nn]ine (?:actions|shortcuts)", "'nine actions' (UI has 4 rows)"),
    ("MAC-DOC-06", r"(?:đặt được trong|can still set them (?:in|under))\s*<em>(?:Phím tắt|Shortcuts)", "F-keys settable in the Shortcuts window (they are config.toml only)"),
    ("MAC-DOC-07", r"Đánh dấu URL hoặc đoạn mã|Mark URLs or code", "raw-zone marking control (no such control)"),
    ("MAC-DOC-08", r"Unicode NFC hoặc NFD(?!.*Windows)", "NFC/NFD choice without a Windows qualifier"),
    ("MAC-DOC-10", r"ba cửa sổ phụ|three more:|Shortcuts, Abbreviations, Advanced\.", "'three auxiliary windows' (there are four incl. About)"),
    ("MAC-DOC-10", r"480\s*[×x]\s*460", "480x460 window size (Mac ~480x453)"),
    ("MAC-DOC-09", r"chọn giao diện Sáng hoặc Tối(?!.*(?:Windows|Mac))|choose (?:Light|Dark) or (?:Light|Dark)(?!.*(?:Windows|Mac))", "theme choice claimed without a platform qualifier"),
]
def check_stale():
    by_case = {}
    for lang, p in PAGES.items():
        text_lines = {}
        for i, line in enumerate(p.raw.splitlines(), 1):
            text_lines[i] = line
        # multi-line friendly: search the tag-light raw per line and the whole flattened text
        flat = re.sub(r"\s+", " ", p.raw)
        for case, pat, label in STALE:
            for m in re.finditer(pat, flat):
                by_case.setdefault((case, label), []).append(f"{p.rel}: ...{re.sub('<[^>]+>', '', flat[max(0, m.start()-25):m.end()+25])}...")
    cases = sorted({c for c, _, _ in STALE})
    for case in cases:
        hits = {k: v for k, v in by_case.items() if k[0] == case}
        labels = [l for c, _, l in STALE if c == case]
        record(f"SITE-08/{case[-2:]}", f"stale-claim grep: {'; '.join(labels)}", case, verdict(hits), "; ".join(f"[{k[1]}] {x}" for k, v in hits.items() for x in v[:2]))

# ---------------------------------------------------------------- Mac wording present in both languages (positive greps)
def check_positive():
    bad = []
    wants = {
        "vi": {"title/meta mention macOS": r"macOS", "Mở cùng macOS or equivalent": r"Mở cùng (?:máy|macOS)", "⌃⇧ default": r"⌃⇧", "Control + Shift": r"Control \+ Shift",
               "Mac screenshot": r"mac-settings-dark\.png", "menu screenshot": r"mac-menu-dark\.png"},
        "en": {"title/meta mention macOS": r"macOS", "Open at login": r"Open at login", "⌃⇧ default": r"⌃⇧", "Control \\+ Shift": r"Control \+ Shift",
               "Mac screenshot": r"mac-settings-dark\.png", "menu screenshot": r"mac-menu-dark\.png"},
    }
    for lang, p in PAGES.items():
        for label, pat in wants[lang].items():
            if not re.search(pat, p.raw): bad.append(f"{p.rel}: missing {label}")
    for what, p, pats in [("404", NF, [r"macOS"])]:
        for pat in pats:
            if not re.search(pat, p.raw): bad.append(f"{p.rel}: missing {pat}")
    record("SITE-09", "Mac content present in both languages and in the 404 (title/meta, Mac screenshots, default shortcuts, autostart wording)", "MAC-DOC-01,MAC-DOC-05,MAC-LNG-24", verdict(bad), "; ".join(bad))

# ---------------------------------------------------------------- English page must not leak Vietnamese (MAC-LNG-29 aria/alt; LNG-06 site half)
ALLOW_EN = re.compile(r"tiếng|Tiếng|Việt|ưo|ươ|tieengs|Telex|VNI")   # examples of the product's own output
def check_en_leaks():
    bad = []
    for tag, a, line in EN.tags:
        for attr in ("alt", "aria-label", "title", "placeholder", "content"):
            v = a.get(attr, "")
            if DIAC.search(v) and not ALLOW_EN.search(v): bad.append(f"{EN.rel}:{line} {tag}[{attr}]={v[:60]!r}")
    for line, t in EN.text:
        if len(t) > 3 and DIAC.search(t) and not ALLOW_EN.search(t): bad.append(f"{EN.rel}:{line} text {t[:60]!r}")
    m = re.search(r"<title>(.*?)</title>", EN.raw, re.S)
    if m and DIAC.search(m.group(1)): bad.append("en <title> has Vietnamese")
    # and the reverse: the vi page should not ship whole English paragraphs (count words from a small stop list)
    vi_en = [f"{VI.rel}:{l} {t[:50]!r}" for l, t in VI.text if len(t.split()) > 6 and re.search(r"\b(the|and|with|your|you)\b", t) and not DIAC.search(t)]
    record("SITE-10", "English page has no untranslated Vietnamese (text, alt, aria-label, title); Vietnamese page has no English paragraphs", "MAC-LNG-29,MAC-LNG-06", verdict(bad + vi_en), "; ".join((bad + vi_en)[:10]))

# ---------------------------------------------------------------- Pages workflow publishes site/ incl. site/en
def check_pages():
    bad = []
    wf = rd(".github/workflows/pages.yml")
    if not re.search(r"upload-pages-artifact@v\d+\s*\n\s*with:\s*\n\s*path:\s*site\s*$", wf, re.M): bad.append("pages.yml does not upload path: site")
    if "site/**" not in wf: bad.append("pages.yml not triggered by site/**")
    if "workflow_dispatch" not in wf: bad.append("no workflow_dispatch")
    if not os.path.exists(os.path.join(REPO, "site/en/index.html")): bad.append("site/en/index.html missing")
    record("SITE-11", "Pages workflow uploads site/ (which contains en/), runs on site/** and manually", "MAC-LNG-21,MAC-DOC-22", verdict(bad), "; ".join(bad))

# ---------------------------------------------------------------- README + release notes (MAC-DOC-01/05/13/23)
def check_readme():
    bad = []
    rm = rd("README.md")
    first = rm.split("\n\n", 3)[:3]
    if "macOS" not in " ".join(first): bad.append("README intro does not mention macOS")
    if "vkb-macos" not in rm: bad.append("README structure lacks crates/vkb-macos")
    if not re.search(r"\.dmg", rm): bad.append("README has no dmg install")
    if not re.search(r"Gatekeeper|quarantine|Open Anyway|Vẫn mở", rm): bad.append("README has no Gatekeeper note")
    if not re.search(r"Trợ năng|Accessibility", rm): bad.append("README has no Accessibility note")
    if not re.search(r"Mở cùng macOS", rm): bad.append("README lacks 'Mở cùng macOS'")
    if not re.search(r"cargo|npm", rm.split("macOS", 1)[-1]): bad.append("README has no Mac build steps")
    record("SITE-12", "README: macOS install (dmg, Gatekeeper, Accessibility), Mac autostart wording, vkb-macos row, Mac build", "MAC-DOC-01,MAC-DOC-05,MAC-DOC-23", verdict(bad), "; ".join(bad))
    bad = []
    rn = rd("docs/evkey-clone-work/release-notes-mac-draft.md")
    if re.search(r"không có biểu tượng trên Dock", rn): bad.append("release notes still say there is no Dock icon")
    if not re.search(r"biểu tượng trên Dock|Dock", rn): bad.append("release notes do not describe the Dock icon")
    if re.search(r"đặt được trong\s*`?Phím tắt", rn): bad.append("release notes claim F-keys are settable in Phím tắt…")
    if not re.search(r"Input Monitoring|Giám sát đầu vào", rn): bad.append("release notes never mention Input Monitoring (MAC-DOC-15)")
    record("SITE-13", "Mac release notes: Dock icon wording (M3), F-key claim, Input Monitoring mentioned", "MAC-DOC-13,MAC-DOC-06,MAC-DOC-15", verdict(bad), "; ".join(bad))

# ---------------------------------------------------------------- 404 handles both languages and root-absolute assets
def check_404():
    bad = []
    t = strip_tags(NF.raw)
    if not (DIAC.search(t) and re.search(r"Page not found|not found", t)): bad.append("404 is not bilingual")
    for tag, a, line in NF.tags:
        for attr in ("href", "src"):
            v = a.get(attr, "")
            if v and not re.match(r"(https?:|#|mailto:)", v) and not v.startswith("/vietkb/"): bad.append(f"404.html:{line} {attr}={v} is not root-absolute (/vietkb/...)")
    record("SITE-14", "404.html is bilingual and every link/asset is root-absolute, so it renders under /en/ too", "MAC-LNG-28,MAC-DOC-21", verdict(bad), "; ".join(bad))

if __name__ == "__main__":
    for fn in (check_links, check_lang_switch, check_head, check_parity, check_downloads, check_fallback, check_mac_sections, check_stale,
               check_positive, check_en_leaks, check_pages, check_readme, check_404):
        try: fn()
        except Exception as e:   # a broken check is a FAIL of the harness, not silence
            record("SITE-ERR", f"{fn.__name__} crashed", "-", "FAIL", repr(e))
    p = sum(r["verdict"] == "PASS" for r in ROWS)
    print(f"[site] {p}/{len(ROWS)} PASS")
