//! Human-style typing QA: seeded, generated scenarios in Telex and VNI.
//!
//! A person does not type a sentence once and perfectly. This test types
//! Vietnamese text the way a person does, with the mistakes and corrections a
//! person makes, and checks that the final document equals the text the person
//! meant to write.
//!
//! Simulator: a `Human` owns the document (`Vec<char>`) and a caret. Every key
//! goes through `Engine::process_key` and its action is applied at the caret the
//! way a backend would: `Noop` types the key (or deletes one character before
//! the caret for Backspace, or inserts a newline for Enter), `Replace` deletes
//! `delete_units` before the caret and types the insert. Caret moves (arrows,
//! mouse click, double-click selection) call `boundary()`, as the platform layer
//! does when the caret moves.
//!
//! Mistake patterns (a scenario mixes them; the name is the report group):
//! - `typo+bs`: one or two wrong letters inside a word, Backspaced, then the word goes on.
//! - `tone-change`: a wrong tone key, then the right one (the second replaces the first).
//! - `tone-remove`: a wrong tone, the remove key (Telex `z`, VNI `0`), then the right tone.
//! - `space-bs-reopen`: the word is finished with a space, Backspace takes the space
//!   back, and the tone is fixed right after the reopened word.
//! - `select-retype`: a word is typed with a wrong tone and left. After its sentence
//!   the person double-clicks it, or clicks at its end and Backspaces it, or arrows
//!   back to it and Backspaces it, then retypes it.
//! - `bs-words-retype`: Backspace over the last 1 to 3 words, then they are typed again.
//! - `enter-bs`: an extra Enter removed with Backspace, or two lines joined with
//!   Backspace and a space.
//!
//! Oracle: the final document equals the intended text. The intended text is the
//! sentences with the separators the person settled on (a joined line is a space).
//! Each word is typed in one of up to three key forms (tone at the end, tone right
//! after the last vowel, or the horn of `uo` typed once at the end). A key form is
//! used only if fresh typing of that form (with a trailing space) gives the word.
//! English loanwords are kept even when that check fails, because D10 is meant to
//! restore them; the rejected forms are listed in the summary.
//!
//! Env `HUMAN_REPORT=<path>` writes every failure to that file.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::Write as _;

use unicode_normalization::UnicodeNormalization;
use vkb_core::{
    simulate, Engine, EngineAction, EngineOptions, InputMethod, KeyInput, VietnameseEngine,
    VK_BACK, VK_RETURN,
};

const VK_LEFT: u16 = 0x25;
const VK_END: u16 = 0x23;

/// Scenarios per input method.
const SCENARIOS: usize = 2000;
const SEED: u64 = 0x5EED_0B0A_2026_1102;
/// Failures printed to stderr in full; the rest go to `HUMAN_REPORT` only.
const SHOW_FIRST: usize = 20;
/// Shortest failing examples printed per pattern.
const EXAMPLES: usize = 3;

/// Intended text, written in NFC. Each sentence is typed as one unit of a scenario.
const SENTENCES: &[&str] = &[
    "Hôm nay tôi đi làm sớm.",
    "Bạn có vui không?",
    "Cuộc meeting lúc chín giờ sáng mai.",
    "Tôi đã gửi email cho anh Nam rồi.",
    "Bạn nhớ mang theo file báo cáo nhé.",
    "Việt Nam có nhiều món ăn ngon.",
    "Chiều nay mình họp lại để chốt kế hoạch.",
    "Trời hôm nay mưa to quá.",
    "Em đang học tiếng Anh mỗi ngày.",
    "Mọi người cố gắng làm việc thật tốt.",
    "Cảm ơn bạn đã giúp tôi nhiều.",
    "Tuần sau chúng ta sẽ đi Đà Nẵng.",
    "Anh ấy vừa mua một chiếc xe mới.",
    "Đừng quên gửi báo cáo trước năm giờ.",
    "Hôm qua trời đẹp nên chúng tôi đi dạo.",
    "Người bạn cũ đã về thăm nhà.",
    "Tôi thích uống cà phê buổi sáng.",
    "Trưa nay mình ăn cơm ở văn phòng.",
];

/// English loanwords typed as plain letters; D10 should put them back on a break.
const ENGLISH: &[&str] = &["email", "meeting", "file"];

/// Letters a person mistypes and then Backspaces. None of them is a Telex mark
/// or tone key, so a stray one never changes the word before the Backspace.
const GARBAGE: &[u8] = b"bcghklmnpqtv";

// ---------------------------------------------------------------------------
// Random source (splitmix64, no dependency)
// ---------------------------------------------------------------------------

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }

    fn chance(&mut self, percent: usize) -> bool {
        self.below(100) < percent
    }
}

// ---------------------------------------------------------------------------
// Keys for a Vietnamese word
// ---------------------------------------------------------------------------

fn options(vni: bool) -> EngineOptions {
    EngineOptions {
        method: if vni { InputMethod::Vni } else { InputMethod::Telex },
        ..EngineOptions::default()
    }
}

fn method_name(vni: bool) -> &'static str {
    if vni {
        "VNI"
    } else {
        "Telex"
    }
}

fn tone_char(vni: bool, tone: u8) -> char {
    if vni {
        (b'0' + tone) as char
    } else {
        ['s', 'f', 'r', 'x', 'j'][tone as usize - 1]
    }
}

fn remove_char(vni: bool) -> char {
    if vni {
        '0'
    } else {
        'z'
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Mk {
    None,
    Hat,
    Breve,
    Horn,
    Stroke,
}

#[derive(Clone)]
struct Unit {
    base: char,
    mark: Mk,
    vowel: bool,
}

/// Split a precomposed word into base letters with marks and a tone (1..=5).
fn decompose(display: &str) -> Option<(Vec<Unit>, Option<u8>)> {
    let mut units: Vec<Unit> = Vec::new();
    let mut tone = None;
    for c in display.nfd() {
        match c {
            '\u{301}' => tone = Some(1),
            '\u{300}' => tone = Some(2),
            '\u{309}' => tone = Some(3),
            '\u{303}' => tone = Some(4),
            '\u{323}' => tone = Some(5),
            '\u{302}' | '\u{306}' | '\u{31B}' => {
                let u = units.last_mut()?;
                if u.mark != Mk::None {
                    return None;
                }
                u.mark = match c {
                    '\u{302}' => Mk::Hat,
                    '\u{306}' => Mk::Breve,
                    _ => Mk::Horn,
                };
            }
            'đ' => units.push(Unit { base: 'd', mark: Mk::Stroke, vowel: false }),
            'Đ' => units.push(Unit { base: 'D', mark: Mk::Stroke, vowel: false }),
            c if c.is_ascii_alphabetic() => units.push(Unit {
                base: c,
                mark: Mk::None,
                vowel: "aeiouyAEIOUY".contains(c),
            }),
            _ => return None,
        }
    }
    if units.is_empty() {
        return None;
    }
    Some((units, tone))
}

fn unit_keys(u: &Unit, vni: bool) -> String {
    let mut s = String::new();
    s.push(u.base);
    match (u.mark, vni) {
        (Mk::None, _) => {}
        (Mk::Stroke, false) | (Mk::Hat, false) => s.push(u.base),
        (Mk::Stroke, true) => s.push('9'),
        (Mk::Hat, true) => s.push('6'),
        (Mk::Breve, false) | (Mk::Horn, false) => s.push('w'),
        (Mk::Breve, true) => s.push('8'),
        (Mk::Horn, true) => s.push('7'),
    }
    s
}

/// One key form of a word. `tone` is the byte index of the tone key in `keys`
/// and the tone number (1..=5).
#[derive(Clone, Debug)]
struct Variant {
    keys: String,
    tone: Option<(usize, u8)>,
    /// Ends in a stop coda (c, ch, p, t): only sắc (1) and nặng (5) are legal tones.
    stop: bool,
}

fn add_variant(out: &mut Vec<Variant>, v: Variant) {
    if !out.iter().any(|o| o.keys == v.keys) {
        out.push(v);
    }
}

/// Key forms of a word: tone at the end (standard), tone right after the last
/// vowel, and (for a bare `ươ` ending) the horn typed once at the end.
fn variants(display: &str, vni: bool) -> Vec<Variant> {
    let Some((units, tone)) = decompose(display) else {
        return Vec::new();
    };
    let plain: Vec<String> = units.iter().map(|u| unit_keys(u, vni)).collect();
    let mut out = Vec::new();
    let n = units.len();
    let stop = !units[n - 1].vowel
        && (matches!(units[n - 1].base.to_ascii_lowercase(), 'c' | 'p' | 't')
            || (units[n - 1].base.eq_ignore_ascii_case(&'h')
                && n >= 2
                && units[n - 2].base.eq_ignore_ascii_case(&'c')));

    let mut keys = plain.concat();
    let mut tp = None;
    if let Some(t) = tone {
        tp = Some((keys.len(), t));
        keys.push(tone_char(vni, t));
    }
    add_variant(&mut out, Variant { keys, tone: tp, stop });

    if let Some(t) = tone {
        if let Some(vi) = units.iter().rposition(|u| u.vowel) {
            let head = plain[..=vi].concat();
            let pos = head.len();
            let keys = format!("{}{}{}", head, tone_char(vni, t), plain[vi + 1..].concat());
            add_variant(&mut out, Variant { keys, tone: Some((pos, t)), stop });
        }
    }

    let horns: Vec<usize> = units
        .iter()
        .enumerate()
        .filter(|(_, u)| u.mark == Mk::Horn)
        .map(|(i, _)| i)
        .collect();
    let others = units.iter().any(|u| matches!(u.mark, Mk::Hat | Mk::Breve | Mk::Stroke));
    if horns.len() == 2
        && horns[1] == horns[0] + 1
        && !others
        && units[horns[0]].base.eq_ignore_ascii_case(&'u')
        && units[horns[1]].base.eq_ignore_ascii_case(&'o')
        && units.last().is_some_and(|u| u.vowel)
    {
        let mut plain2 = units.clone();
        for &i in &horns {
            plain2[i].mark = Mk::None;
        }
        let mut keys: String = plain2.iter().map(|u| unit_keys(u, vni)).collect();
        keys.push(if vni { '7' } else { 'w' });
        let mut tp = None;
        if let Some(t) = tone {
            tp = Some((keys.len(), t));
            keys.push(tone_char(vni, t));
        }
        add_variant(&mut out, Variant { keys, tone: tp, stop });
    }
    out
}

fn fresh_ok(opts: EngineOptions, keys: &str, display: &str) -> bool {
    simulate(opts, &format!("{keys} ")) == format!("{display} ")
}

// ---------------------------------------------------------------------------
// Script: tokens and the intended text
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq)]
enum Brk {
    /// A space between sentences of one line.
    Sp,
    Nl1,
    Nl2,
    /// Typed as Enter, Backspace, space: the person joins two lines.
    Join,
    /// Typed as Enter, Enter, Backspace: an extra Enter taken back.
    Extra,
}

impl Brk {
    fn intended(self) -> &'static str {
        match self {
            Brk::Sp | Brk::Join => " ",
            Brk::Nl1 | Brk::Extra => "\n",
            Brk::Nl2 => "\n\n",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Tok {
    Word(String),
    Punct(char),
    Space,
    Break(Brk),
}

fn tokenize(s: &str) -> Vec<Tok> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for c in s.chars() {
        if c.is_alphabetic() {
            cur.push(c);
            continue;
        }
        if !cur.is_empty() {
            out.push(Tok::Word(std::mem::take(&mut cur)));
        }
        out.push(if c == ' ' { Tok::Space } else { Tok::Punct(c) });
    }
    if !cur.is_empty() {
        out.push(Tok::Word(cur));
    }
    out
}

fn intended_text(t: &Tok) -> String {
    match t {
        Tok::Word(w) => w.clone(),
        Tok::Punct(c) => c.to_string(),
        Tok::Space => " ".to_string(),
        Tok::Break(b) => b.intended().to_string(),
    }
}

struct Vocab {
    words: HashMap<String, Vec<Variant>>,
    pool: Vec<&'static str>,
    notes: Vec<String>,
}

fn build_vocab(vni: bool) -> Vocab {
    let opts = options(vni);
    let mut words: HashMap<String, Vec<Variant>> = HashMap::new();
    let mut notes = Vec::new();
    let mut pool = Vec::new();
    for &s in SENTENCES {
        let mut usable = true;
        for tok in tokenize(s) {
            let Tok::Word(w) = tok else {
                continue;
            };
            if !words.contains_key(&w) {
                let english = ENGLISH.contains(&w.as_str());
                let mut kept = Vec::new();
                for v in variants(&w, vni) {
                    if fresh_ok(opts, &v.keys, &w) {
                        kept.push(v);
                    } else if english {
                        notes.push(format!(
                            "{}: english loanword {w:?} kept although fresh typing of {:?} differs",
                            method_name(vni),
                            v.keys
                        ));
                        kept.push(v);
                    } else {
                        notes.push(format!(
                            "{}: key form {:?} rejected for {w:?} (fresh typing differs)",
                            method_name(vni),
                            v.keys
                        ));
                    }
                }
                words.insert(w.clone(), kept);
            }
            if words[&w].is_empty() {
                usable = false;
                notes.push(format!("{}: no usable key form for {w:?}", method_name(vni)));
            }
        }
        if usable {
            pool.push(s);
        } else {
            notes.push(format!("{}: sentence not used: {s:?}", method_name(vni)));
        }
    }
    Vocab { words, pool, notes }
}

fn build_script(rng: &mut Rng, pool: &[&'static str]) -> Vec<Tok> {
    let n = 2 + rng.below(3);
    let mut toks = Vec::new();
    for i in 0..n {
        toks.extend(tokenize(pool[rng.below(pool.len())]));
        if i + 1 < n {
            let b = match rng.below(100) {
                0..=44 => Brk::Sp,
                45..=69 => Brk::Nl1,
                70..=81 => Brk::Nl2,
                82..=91 => Brk::Join,
                _ => Brk::Extra,
            };
            toks.push(Tok::Break(b));
        }
    }
    toks
}

// ---------------------------------------------------------------------------
// Human simulator
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Debug)]
enum Mode {
    Clean,
    Typo,
    ToneChange,
    ToneRemove,
    Reopen,
    Defer,
}

fn pick_mode(rng: &mut Rng, toned: bool, space_next: bool) -> Mode {
    let m = match rng.below(100) {
        0..=29 => Mode::Typo,
        30..=49 => Mode::ToneChange,
        50..=59 => Mode::ToneRemove,
        60..=79 => Mode::Reopen,
        _ => Mode::Defer,
    };
    match m {
        Mode::Typo => Mode::Typo,
        _ if !toned => Mode::Typo,
        Mode::Reopen if !space_next => Mode::Typo,
        _ => m,
    }
}

/// A tone key a person could plausibly type by mistake: any other tone, but on a
/// stop coda only sắc or nặng (the other tones are not spelling-legal there).
fn wrong_tone(rng: &mut Rng, t: u8, stop: bool) -> u8 {
    let choices: Vec<u8> = (1..=5u8)
        .filter(|&u| u != t && (!stop || u == 1 || u == 5))
        .collect();
    choices[rng.below(choices.len())]
}

/// A word typed wrong and left, to be fixed after its sentence.
struct Deferred {
    start: usize,
    end: usize,
    keys: String,
    /// `end` is settled once the separator after the word is typed: a space or
    /// a mark can make the engine rewrite the word at that boundary (D10).
    pending: bool,
}

/// After the separator following a deferred word is typed, the word ends one
/// character before the caret.
fn settle(deferred: &mut [Deferred], doc: &[char]) {
    if let Some(d) = deferred.last_mut() {
        if d.pending {
            d.pending = false;
            if doc.len() > d.start {
                d.end = doc.len() - 1;
            }
        }
    }
}

struct Human {
    eng: Engine,
    doc: Vec<char>,
    caret: usize,
    log: Vec<String>,
    pats: BTreeSet<&'static str>,
    harness: bool,
}

impl Human {
    fn new(vni: bool) -> Self {
        Self {
            eng: Engine::new(options(vni)),
            doc: Vec::new(),
            caret: 0,
            log: Vec::new(),
            pats: BTreeSet::new(),
            harness: false,
        }
    }

    fn note(&mut self, s: String) {
        self.log.push(s);
    }

    fn insert_str(&mut self, s: &str) {
        for c in s.chars() {
            self.doc.insert(self.caret, c);
            self.caret += 1;
        }
    }

    fn delete_before(&mut self, n: usize) {
        for _ in 0..n {
            if self.caret > 0 {
                self.doc.remove(self.caret - 1);
                self.caret -= 1;
            }
        }
    }

    /// What the app does with a key the engine passed through.
    fn plain_key(&mut self, key: KeyInput) {
        if key.vk == VK_BACK || matches!(key.ch, Some('\u{8}') | Some('\u{7f}')) {
            self.delete_before(1);
        } else if key.vk == VK_RETURN {
            self.insert_str("\n");
        } else if let Some(c) = key.ch {
            self.insert_str(&c.to_string());
        }
    }

    fn press(&mut self, key: KeyInput) {
        let action = self.eng.process_key(key);
        match action {
            EngineAction::Noop => self.plain_key(key),
            EngineAction::Commit(text) => self.insert_str(&text),
            EngineAction::Replace { delete_units, insert } => {
                self.delete_before(delete_units as usize);
                self.insert_str(&insert);
            }
            EngineAction::ReplaceThenPass { delete_units, insert } => {
                self.delete_before(delete_units as usize);
                self.insert_str(&insert);
                self.plain_key(key);
            }
            _ => {}
        }
    }

    fn typed(&mut self, s: &str) {
        if s.is_empty() {
            return;
        }
        self.note(format!("type {s:?}"));
        for c in s.chars() {
            self.press(KeyInput::char(c));
        }
    }

    fn sp(&mut self) {
        self.note("sp".to_string());
        self.press(KeyInput::space());
    }

    fn enter(&mut self, n: usize) {
        self.note(if n == 1 { "ent".to_string() } else { format!("entx{n}") });
        for _ in 0..n {
            self.press(KeyInput::enter());
        }
    }

    fn bs(&mut self, n: usize) {
        if n == 0 {
            return;
        }
        self.note(if n == 1 { "BS".to_string() } else { format!("BSx{n}") });
        for _ in 0..n {
            self.press(KeyInput::backspace());
        }
    }

    /// Arrow Left `n` times: the engine sees each key, then the caret has moved.
    fn left(&mut self, n: usize) {
        if n == 0 {
            return;
        }
        self.note(format!("Lx{n}"));
        for _ in 0..n {
            self.press(KeyInput::key(VK_LEFT, None));
        }
        self.caret -= n;
        self.eng.boundary();
    }

    fn ctrl_end(&mut self) {
        self.note("ctrl-end".to_string());
        self.press(KeyInput::key(VK_END, None));
        self.caret = self.doc.len();
        self.eng.boundary();
    }

    fn emit_word(&mut self, rng: &mut Rng, vni: bool, v: &Variant, mode: Mode) {
        let keys = v.keys.as_str();
        match mode {
            Mode::Clean => self.typed(keys),
            Mode::Typo => {
                let p = rng.below(keys.len() + 1);
                let g = 1 + rng.below(2);
                let garbage: String = (0..g)
                    .map(|_| GARBAGE[rng.below(GARBAGE.len())] as char)
                    .collect();
                self.typed(&keys[..p]);
                self.typed(&garbage);
                self.bs(g);
                self.typed(&keys[p..]);
                self.pats.insert("typo+bs");
            }
            Mode::ToneChange | Mode::ToneRemove | Mode::Reopen | Mode::Defer => {
                let Some((pos, t)) = v.tone else {
                    self.typed(keys);
                    return;
                };
                let wrong = tone_char(vni, wrong_tone(rng, t, v.stop));
                match mode {
                    Mode::ToneChange => {
                        self.typed(&format!("{}{}{}", &keys[..pos], wrong, &keys[pos..]));
                        self.pats.insert("tone-change");
                    }
                    Mode::ToneRemove => {
                        self.typed(&format!(
                            "{}{}{}{}",
                            &keys[..pos],
                            wrong,
                            remove_char(vni),
                            &keys[pos..]
                        ));
                        self.pats.insert("tone-remove");
                    }
                    Mode::Reopen => {
                        self.typed(&format!("{}{}{}", &keys[..pos], wrong, &keys[pos + 1..]));
                        self.sp();
                        self.bs(1);
                        self.typed(&keys[pos..pos + 1]);
                        self.pats.insert("space-bs-reopen");
                    }
                    _ => {
                        // Wrong tone now; the fix comes after the sentence.
                        self.typed(&format!("{}{}{}", &keys[..pos], wrong, &keys[pos + 1..]));
                    }
                }
            }
        }
    }

    fn emit_break_typed(&mut self, b: Brk) {
        match b {
            Brk::Sp => self.sp(),
            Brk::Nl1 => self.enter(1),
            Brk::Nl2 => self.enter(2),
            Brk::Join => {
                self.enter(1);
                self.bs(1);
                self.sp();
                self.pats.insert("enter-bs");
            }
            Brk::Extra => {
                self.enter(2);
                self.bs(1);
                self.pats.insert("enter-bs");
            }
        }
    }

    /// Retype a token the way a person types it when they start again: clean keys.
    fn emit_clean(&mut self, t: &Tok, v: Option<&Variant>) {
        match t {
            Tok::Word(_) => {
                if let Some(v) = v {
                    self.typed(&v.keys);
                }
            }
            Tok::Punct(c) => self.typed(&c.to_string()),
            Tok::Space => self.sp(),
            Tok::Break(b) => match b {
                Brk::Nl2 => self.enter(2),
                Brk::Nl1 | Brk::Extra => self.enter(1),
                Brk::Sp | Brk::Join => self.sp(),
            },
        }
    }

    /// Backspace over the last 1 to 3 words (and what follows them), then type
    /// them again. `upto` is the number of tokens already typed.
    /// Backspace over the last 1 to 3 words (and everything after them, as the
    /// person sees it on screen), then type those tokens again. `tok_pos[j]` is
    /// the document position where token `j` starts.
    fn cut(
        &mut self,
        rng: &mut Rng,
        toks: &[Tok],
        used: &[Option<Variant>],
        tok_pos: &mut [usize],
        upto: usize,
    ) {
        let words: Vec<usize> = (0..upto)
            .filter(|&j| matches!(toks[j], Tok::Word(_)))
            .collect();
        if words.is_empty() {
            return;
        }
        let k = (1 + rng.below(3)).min(words.len());
        let c = words[words.len() - k];
        let presses = self.doc.len().saturating_sub(tok_pos[c]);
        self.pats.insert("bs-words-retype");
        self.bs(presses);
        for j in c..upto {
            tok_pos[j] = self.doc.len();
            self.emit_clean(&toks[j], used[j].as_ref());
        }
    }

    /// Fix one deferred word: double-click it and type over it, click at its end
    /// and Backspace it, or arrow back to its end and Backspace it. Then retype it.
    /// Returns the change in document length.
    fn fix(&mut self, rng: &mut Rng, d: &Deferred) -> isize {
        if d.end > self.doc.len() || d.start > d.end {
            self.harness = true;
            return 0;
        }
        let before = self.doc.len() as isize;
        let len = d.end - d.start;
        let wrong: String = self.doc[d.start..d.end].iter().collect();
        match rng.below(3) {
            0 => {
                self.note(format!("dbl({wrong:?})"));
                self.eng.boundary();
                self.doc.drain(d.start..d.end);
                self.caret = d.start;
            }
            1 => {
                self.note("click".to_string());
                self.eng.boundary();
                self.caret = d.end;
                self.bs(len);
            }
            _ => {
                let n = self.caret - d.end;
                self.left(n);
                self.bs(len);
            }
        }
        self.typed(&d.keys);
        self.pats.insert("select-retype");
        self.doc.len() as isize - before
    }

    fn flush(&mut self, rng: &mut Rng, deferred: &mut Vec<Deferred>, tok_pos: &mut [usize]) {
        if deferred.is_empty() {
            return;
        }
        deferred.sort_by_key(|d| std::cmp::Reverse(d.start));
        for d in deferred.drain(..) {
            let delta = self.fix(rng, &d);
            // Tokens after the fixed word move by the change in its length.
            for p in tok_pos.iter_mut() {
                if *p >= d.end {
                    *p = (*p as isize + delta) as usize;
                }
            }
        }
        self.ctrl_end();
    }
}

// ---------------------------------------------------------------------------
// Scenarios, oracle, report
// ---------------------------------------------------------------------------

struct Outcome {
    seed: u64,
    intended: String,
    actual: String,
    log: String,
    pats: BTreeSet<&'static str>,
    english: bool,
    harness: bool,
}

impl Outcome {
    fn failed(&self) -> bool {
        self.actual != self.intended
    }
}

fn run_scenario(seed: u64, vni: bool, vocab: &Vocab) -> Outcome {
    let mut rng = Rng(seed);
    let mess = if rng.chance(25) { 0 } else { 1 + rng.below(3) };
    let p_word = [0usize, 8, 15, 25][mess];
    let p_cut = [0usize, 12, 22, 32][mess];

    let toks = build_script(&mut rng, &vocab.pool);
    let english = toks
        .iter()
        .any(|t| matches!(t, Tok::Word(w) if ENGLISH.contains(&w.as_str())));
    let mut used: Vec<Option<Variant>> = vec![None; toks.len()];
    let mut tok_pos: Vec<usize> = vec![0; toks.len()];
    let mut h = Human::new(vni);
    let mut deferred: Vec<Deferred> = Vec::new();

    for i in 0..toks.len() {
        tok_pos[i] = h.doc.len();
        match &toks[i] {
            Tok::Word(w) => {
                let choices = &vocab.words[w];
                let v = choices[rng.below(choices.len())].clone();
                let space_next = matches!(toks.get(i + 1), Some(Tok::Space));
                let mode = if mess > 0 && rng.chance(p_word) {
                    pick_mode(&mut rng, v.tone.is_some(), space_next)
                } else {
                    Mode::Clean
                };
                let start = h.doc.len();
                h.emit_word(&mut rng, vni, &v, mode);
                if mode == Mode::Defer {
                    deferred.push(Deferred {
                        start,
                        end: h.doc.len(),
                        keys: v.keys.clone(),
                        pending: true,
                    });
                }
                used[i] = Some(v);
            }
            Tok::Punct(c) => {
                h.typed(&c.to_string());
                settle(&mut deferred, &h.doc);
            }
            Tok::Space => {
                h.sp();
                settle(&mut deferred, &h.doc);
            }
            Tok::Break(b) => {
                h.flush(&mut rng, &mut deferred, &mut tok_pos);
                if mess > 0 && i > 0 && rng.chance(p_cut) {
                    h.cut(&mut rng, &toks, &used, &mut tok_pos, i);
                }
                tok_pos[i] = h.doc.len();
                h.emit_break_typed(*b);
            }
        }
    }
    h.flush(&mut rng, &mut deferred, &mut tok_pos);
    if mess > 0 && rng.chance(p_cut) {
        h.cut(&mut rng, &toks, &used, &mut tok_pos, toks.len());
    }

    Outcome {
        seed,
        intended: toks.iter().map(intended_text).collect(),
        actual: h.doc.iter().collect(),
        log: h.log.join(" | "),
        pats: h.pats,
        english,
        harness: h.harness,
    }
}

fn diff(intended: &str, actual: &str) -> String {
    let a: Vec<char> = intended.chars().collect();
    let b: Vec<char> = actual.chars().collect();
    let i = a
        .iter()
        .zip(b.iter())
        .position(|(x, y)| x != y)
        .unwrap_or(a.len().min(b.len()));
    let lo = i.saturating_sub(12);
    let window = |v: &[char]| -> String {
        let hi = (i + 12).min(v.len());
        v[lo.min(hi)..hi].iter().collect()
    };
    format!(
        "first diff at char {i}: expected {:?}, got {:?}",
        window(&a),
        window(&b)
    )
}

fn render(vni: bool, o: &Outcome) -> String {
    let mut s = String::new();
    writeln!(
        s,
        "[{}] seed=0x{:016X} mistakes={:?}{}",
        method_name(vni),
        o.seed,
        o.pats,
        if o.harness { " HARNESS-MISMATCH" } else { "" }
    )
    .unwrap();
    writeln!(s, "  human: {}", o.log).unwrap();
    writeln!(s, "  expected: {:?}", o.intended).unwrap();
    writeln!(s, "  got:      {:?}", o.actual).unwrap();
    writeln!(s, "  {}", diff(&o.intended, &o.actual)).unwrap();
    s
}

#[derive(Default)]
struct Row {
    runs: usize,
    fails: usize,
    alone: usize,
}

fn group_names(o: &Outcome) -> Vec<&'static str> {
    if o.pats.is_empty() {
        vec!["plain"]
    } else {
        o.pats.iter().copied().collect()
    }
}

#[test]
fn human_typing_generated_scenarios() {
    let mut report = String::new();
    let mut summary = String::new();
    let mut total_fail = 0usize;
    let mut total_run = 0usize;

    for vni in [false, true] {
        let vocab = build_vocab(vni);
        let mut rows: BTreeMap<&'static str, Row> = BTreeMap::new();
        let mut english = Row::default();
        let mut outcomes: Vec<Outcome> = Vec::new();
        let mut shown = 0usize;

        for n in 0..SCENARIOS {
            let method_salt: u64 = if vni { 0xA5A5_0000 } else { 0 };
            let seed = SEED
                .wrapping_add((n as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15))
                ^ method_salt;
            let o = run_scenario(seed, vni, &vocab);
            let failed = o.failed() || o.harness;
            let groups = group_names(&o);
            for &g in &groups {
                let row = rows.entry(g).or_default();
                row.runs += 1;
                if failed {
                    row.fails += 1;
                    if groups.len() == 1 {
                        row.alone += 1;
                    }
                }
            }
            if o.english {
                english.runs += 1;
                if failed {
                    english.fails += 1;
                }
            }
            if failed {
                let text = render(vni, &o);
                if shown < SHOW_FIRST {
                    eprint!("{text}");
                    shown += 1;
                }
                report.push_str(&text);
                outcomes.push(o);
            }
        }

        let fails = outcomes.len();
        total_fail += fails;
        total_run += SCENARIOS;
        writeln!(
            summary,
            "\n== {}: {} scenarios, {} failing ==",
            method_name(vni),
            SCENARIOS,
            fails
        )
        .unwrap();
        writeln!(
            summary,
            "{:<20} {:>8} {:>8} {:>14}",
            "mistake group", "runs", "failing", "failing alone"
        )
        .unwrap();
        for (name, row) in &rows {
            writeln!(
                summary,
                "{:<20} {:>8} {:>8} {:>14}",
                name, row.runs, row.fails, row.alone
            )
            .unwrap();
        }
        writeln!(
            summary,
            "{:<20} {:>8} {:>8}   (scenarios with an English loanword)",
            "english", english.runs, english.fails
        )
        .unwrap();

        for name in rows.keys() {
            let mut ex: Vec<&Outcome> = outcomes
                .iter()
                .filter(|o| group_names(o).contains(name))
                .collect();
            ex.sort_by_key(|o| o.intended.chars().count());
            if ex.is_empty() {
                continue;
            }
            writeln!(summary, "\n-- shortest failing, group {name}:").unwrap();
            for o in ex.iter().take(EXAMPLES) {
                summary.push_str(&render(vni, o));
            }
        }

        if !vocab.notes.is_empty() {
            writeln!(summary, "\n-- vocabulary notes ({}):", method_name(vni)).unwrap();
            for n in &vocab.notes {
                writeln!(summary, "   {n}").unwrap();
            }
        }
    }

    writeln!(summary, "\nTOTAL: {total_run} scenarios, {total_fail} failing").unwrap();
    eprintln!("{summary}");
    if let Ok(path) = std::env::var("HUMAN_REPORT") {
        let body = format!("{summary}\n==== all failures ====\n{report}");
        std::fs::write(&path, body).expect("write HUMAN_REPORT");
        eprintln!("full report written to {path}");
    }
    assert!(
        total_fail == 0,
        "{total_fail} of {total_run} human-typing scenarios failed (see summary above)"
    );
}
