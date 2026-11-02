//! Human typing scenarios for the real-app QA (owner, 2026-11-02: "gõ sai gõ lại, gõ tốc độ cao
//! và có từ sai, sai giữa câu nhưng viết hết câu rồi mới sửa những từ sai").
//!
//! Each scenario types one natural Vietnamese sentence the way a person does: typos fixed with
//! Backspace, a wrong tone then the right one, a word reopened after its space, two or three words
//! deleted and retyped, a whole word deleted and retyped, and a wrong word left in the sentence and
//! fixed after the sentence is finished (Ctrl+Left back to it, Ctrl+Backspace, retype, End).
//!
//! The test replays every scenario through the engine in a small editor model (caret, word jumps,
//! word delete) and checks the line ends as the intended sentence. With `REAL_APP_EXPORT=<dir>` it
//! also writes `human-telex.tsv` / `human-vni.tsv` (id, keys in the QN2 spec syntax, expected) for
//! `docs/evkey-clone-work/qa/host-e2e/host-e2e8.ps1 -Set human`.

use vkb_core::{Charset, Engine, EngineAction, EngineOptions, InputMethod, KeyInput, VietnameseEngine, VK_BACK};

const VK_END: u16 = 0x23;
const VK_LEFT: u16 = 0x25;

const SENTENCES: &[&str] = &[
    "Hôm nay trời đẹp nên tôi đi bộ ra công viên",
    "Chúng tôi sẽ họp vào chiều thứ năm tuần sau",
    "Mẹ tôi nấu món canh chua rất ngon",
    "Thuở nhỏ tôi thường theo bà ra chợ quê",
    "Bạn đã gửi bản báo cáo cho khách hàng chưa",
    "Năm nay công ty tuyển thêm mười người",
    "Cảm ơn anh đã giúp đỡ em trong dự án này",
    "Trường học của chúng tôi nằm cạnh bờ sông",
    "Buổi tối gia đình tôi thường xem phim cùng nhau",
    "Tôi muốn học thêm tiếng Nhật vào năm tới",
    "Đường phố Hà Nội rất đông vào giờ tan tầm",
    "Cô giáo khen cả lớp đã làm bài rất tốt",
    "Chiếc xe đạp cũ của ông vẫn còn chạy được",
    "Người dân ở đây rất hiếu khách và thân thiện",
    "Mùa thu năm ấy chúng tôi lần đầu gặp nhau",
    "Hãy nhớ khóa cửa trước khi ra khỏi nhà",
    "Em gái tôi vừa thi đỗ vào trường đại học",
    "Những cơn mưa rào làm không khí mát mẻ hơn",
    "Anh ấy luôn đến sớm và về muộn nhất văn phòng",
    "Bữa sáng của tôi thường là bánh mì và cà phê",
    "Ngày mai chúng ta xuất phát lúc sáu giờ sáng",
    "Rượu nếp cẩm là đặc sản của vùng này",
    "Tôi đã đọc xong cuốn sách mà bạn tặng",
    "Hươu cao cổ là loài vật cao nhất thế giới",
];

#[derive(Clone, Copy, PartialEq)]
enum Mark {
    None,
    Hat,
    Breve,
    Horn,
    Bar,
}

struct Word {
    letters: Vec<(char, Mark)>,
    tone: usize,
}

fn parse_word(word: &str) -> Word {
    let nfd = String::from_utf8(Charset::UnicodeNfd.encode(word)).unwrap();
    let mut letters: Vec<(char, Mark)> = Vec::new();
    let mut tone = 0;
    for ch in nfd.chars() {
        match ch {
            '\u{302}' => letters.last_mut().unwrap().1 = Mark::Hat,
            '\u{306}' => letters.last_mut().unwrap().1 = Mark::Breve,
            '\u{31b}' => letters.last_mut().unwrap().1 = Mark::Horn,
            '\u{301}' => tone = 1,
            '\u{300}' => tone = 2,
            '\u{309}' => tone = 3,
            '\u{303}' => tone = 4,
            '\u{323}' => tone = 5,
            'đ' => letters.push(('d', Mark::Bar)),
            'Đ' => letters.push(('D', Mark::Bar)),
            _ => letters.push((ch, Mark::None)),
        }
    }
    Word { letters, tone }
}

fn tone_key(vni: bool, tone: usize) -> char {
    if vni {
        ['0', '1', '2', '3', '4', '5'][tone]
    } else {
        ['z', 's', 'f', 'r', 'x', 'j'][tone]
    }
}

fn is_vowel(ch: char) -> bool {
    matches!(ch.to_ascii_lowercase(), 'a' | 'e' | 'i' | 'o' | 'u' | 'y')
}

fn stop_coda(word: &Word) -> bool {
    let plain: String = word.letters.iter().map(|&(c, _)| c.to_ascii_lowercase()).collect();
    plain.ends_with('c') || plain.ends_with("ch") || plain.ends_with('p') || plain.ends_with('t')
}

/// Keys for `word`, marks in place; the tone at the end, or (`early`) right after the last vowel.
fn word_keys(word: &Word, vni: bool, tone: Option<usize>, early: bool) -> String {
    let mut keys = String::new();
    let last_vowel = word.letters.iter().rposition(|&(c, _)| is_vowel(c));
    for (i, &(c, mark)) in word.letters.iter().enumerate() {
        keys.push(c);
        let m = match (vni, mark) {
            (_, Mark::None) => None,
            (false, Mark::Hat) => Some(c.to_ascii_lowercase()),
            (false, Mark::Breve | Mark::Horn) => Some('w'),
            (false, Mark::Bar) => Some('d'),
            (true, Mark::Hat) => Some('6'),
            (true, Mark::Breve) => Some('8'),
            (true, Mark::Horn) => Some('7'),
            (true, Mark::Bar) => Some('9'),
        };
        keys.extend(m);
        if early && Some(i) == last_vowel {
            if let Some(t) = tone.filter(|&t| t != 0) {
                keys.push(tone_key(vni, t));
            }
        }
    }
    if !early {
        if let Some(t) = tone.filter(|&t| t != 0) {
            keys.push(tone_key(vni, t));
        }
    }
    keys
}

/// One key step: plain text, or a named key for the QN2 spec.
#[derive(Clone, Copy)]
enum Step {
    Ch(char),
    Back,
    Enter,
    Left,
    Up,
    End,
    CtrlEnd,
    CtrlLeft,
    CtrlBack,
}

struct Doc {
    engine: Engine,
    text: Vec<char>,
    caret: usize,
    steps: Vec<Step>,
}

impl Doc {
    fn new(vni: bool) -> Self {
        let method = if vni { InputMethod::Vni } else { InputMethod::Telex };
        Self { engine: Engine::new(EngineOptions { method, ..EngineOptions::default() }), text: Vec::new(), caret: 0, steps: Vec::new() }
    }

    fn apply(&mut self, action: EngineAction, typed: Option<char>, back: bool) {
        match action {
            EngineAction::Noop => {
                if let Some(c) = typed {
                    self.text.insert(self.caret, c);
                    self.caret += 1;
                } else if back && self.caret > 0 {
                    self.caret -= 1;
                    self.text.remove(self.caret);
                }
            }
            EngineAction::Commit(s) => self.insert(&s),
            EngineAction::Replace { delete_units, insert } => {
                self.delete_back(delete_units as usize);
                self.insert(&insert);
            }
            EngineAction::ReplaceThenPass { delete_units, insert } => {
                self.delete_back(delete_units as usize);
                self.insert(&insert);
                if let Some(c) = typed {
                    self.insert(&c.to_string());
                }
            }
            EngineAction::Swallow | EngineAction::Reject | EngineAction::ToggleEnabled => {}
        }
    }

    fn insert(&mut self, s: &str) {
        for c in s.chars() {
            self.text.insert(self.caret, c);
            self.caret += 1;
        }
    }

    fn delete_back(&mut self, n: usize) {
        for _ in 0..n.min(self.caret) {
            self.caret -= 1;
            self.text.remove(self.caret);
        }
    }

    fn line_start(&self, at: usize) -> usize {
        self.text[..at].iter().rposition(|&c| c == '\n').map_or(0, |p| p + 1)
    }

    fn line_end(&self, at: usize) -> usize {
        self.text[at..].iter().position(|&c| c == '\n').map_or(self.text.len(), |p| at + p)
    }

    fn step(&mut self, step: Step) {
        self.steps.push(step);
        // The platform layer (vkb-windows/vkb-macos `route`) gives plain Left/End to the engine
        // (it follows the caret in the remembered line, "gõ dấu đè") and resets it on every other
        // caret mover and every Ctrl chord.
        match step {
            Step::Left => {
                let _ = self.engine.process_key(KeyInput::key(VK_LEFT, None));
            }
            Step::End => {
                let _ = self.engine.process_key(KeyInput::key(VK_END, None));
            }
            Step::Ch(_) | Step::Back | Step::Enter => {}
            _ => self.engine.reset(),
        }
        match step {
            Step::Ch(c) => {
                let action = self.engine.process_key(KeyInput::char(c));
                self.apply(action, Some(c), false);
            }
            Step::Back => {
                let action = self.engine.process_key(KeyInput::key(VK_BACK, None));
                self.apply(action, None, true);
            }
            Step::Enter => {
                let action = self.engine.process_key(KeyInput::enter());
                match action {
                    EngineAction::Noop | EngineAction::ReplaceThenPass { .. } => {
                        if let EngineAction::ReplaceThenPass { delete_units, insert } = action {
                            self.delete_back(delete_units as usize);
                            self.insert(&insert);
                        }
                        self.insert("\n");
                    }
                    other => self.apply(other, None, false),
                }
            }
            Step::Left => self.caret = self.caret.saturating_sub(1),
            Step::End => self.caret = self.line_end(self.caret),
            Step::CtrlEnd => self.caret = self.text.len(),
            Step::Up => {
                let start = self.line_start(self.caret);
                if start > 0 {
                    let col = self.caret - start;
                    let prev = self.line_start(start - 1);
                    self.caret = prev + col.min(start - 1 - prev);
                }
            }
            Step::CtrlLeft => {
                while self.caret > 0 && self.text[self.caret - 1] == ' ' {
                    self.caret -= 1;
                }
                while self.caret > 0 && !matches!(self.text[self.caret - 1], ' ' | '\n') {
                    self.caret -= 1;
                }
            }
            Step::CtrlBack => {
                let mut start = self.caret;
                while start > 0 && !matches!(self.text[start - 1], ' ' | '\n') {
                    start -= 1;
                }
                let n = self.caret - start;
                self.delete_back(n);
            }
        }
    }

    fn keys(&mut self, keys: &str) {
        for c in keys.chars() {
            self.step(Step::Ch(c));
        }
    }

    /// Characters on screen from the caret back to the start of the current word.
    fn word_len_before_caret(&self) -> usize {
        self.text[..self.caret].iter().rev().take_while(|&&c| !matches!(c, ' ' | '\n')).count()
    }

    /// The text, each line without trailing spaces (the harness compares lines that way).
    fn lines(&self) -> String {
        let text: String = self.text.iter().collect();
        text.split('\n').map(str::trim_end).collect::<Vec<_>>().join("\n").trim_end().to_string()
    }

    fn spec(&self) -> String {
        let mut s = String::new();
        for step in &self.steps {
            match *step {
                Step::Ch(c) => s.push(c),
                Step::Back => s.push_str("{BACKSPACE}"),
                Step::Enter => s.push_str("{ENTER}"),
                Step::Left => s.push_str("{LEFT}"),
                Step::Up => s.push_str("{UP}"),
                Step::End => s.push_str("{END}"),
                Step::CtrlEnd => s.push_str("{CTRL+END}"),
                Step::CtrlLeft => s.push_str("{CTRL+LEFT}"),
                Step::CtrlBack => s.push_str("{CTRL+BACKSPACE}"),
            }
        }
        s
    }
}

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
    fn chance(&mut self, percent: u64) -> bool {
        self.next() % 100 < percent
    }
}

/// A tone this word can take that is not its own (for "wrong tone, then the right one").
fn other_tone(word: &Word, rng: &mut Rng) -> usize {
    let choices: Vec<usize> = if stop_coda(word) { vec![1, 5] } else { vec![1, 2, 3, 4, 5] };
    let choices: Vec<usize> = choices.into_iter().filter(|&t| t != word.tone).collect();
    choices[rng.below(choices.len())]
}

const TYPO_LETTERS: &[char] = &['k', 'l', 'm', 'n', 'h', 'g', 'b', 'v', 't', 'p'];


/// Type one word with an in-word mistake (or none), ending with `end` (a space, or nothing).
fn type_word(doc: &mut Doc, parsed: &[Word], i: usize, vni: bool, rng: &mut Rng, used: &mut Vec<&'static str>) {
    let word = &parsed[i];
    let early = !vni && stop_coda(word) && rng.chance(30);
    let keys = word_keys(word, vni, Some(word.tone), early);
    match rng.below(12) {
        // Owner: the tone changed several times (fast typing): two or three wrong tones, then the right one.
        // (Not on a stop coda: only sắc and nặng fit there, so there is a single wrong tone.)
        10 | 11 if word.tone != 0 && !stop_coda(word) => {
            doc.keys(&word_keys(word, vni, None, false));
            let reopen = rng.chance(40);
            if reopen {
                // ... also after the space, back in the word.
                doc.keys(" ");
                doc.step(Step::Back);
            }
            let mut last = 0;
            for _ in 0..2 + rng.below(2) {
                // A different tone each time: the same key twice in a row is the Telex/VNI undo.
                let mut t = other_tone(word, rng);
                while t == last {
                    t = other_tone(word, rng);
                }
                doc.step(Step::Ch(tone_key(vni, t)));
                last = t;
            }
            if last == word.tone {
                doc.step(Step::Ch(tone_key(vni, other_tone(word, rng))));
            }
            doc.step(Step::Ch(tone_key(vni, word.tone)));
            used.push(if reopen { "multi-tone-reopen" } else { "multi-tone" });
        }
        // A typo letter inside the word, fixed with Backspace.
        0 | 1 if keys.chars().count() >= 2 => {
            let at = 1 + rng.below(keys.chars().count() - 1);
            let (a, b): (String, String) = (keys.chars().take(at).collect(), keys.chars().skip(at).collect());
            doc.keys(&a);
            doc.step(Step::Ch(TYPO_LETTERS[rng.below(TYPO_LETTERS.len())]));
            doc.step(Step::Back);
            doc.keys(&b);
            used.push("typo+bs");
        }
        // A wrong tone, then the right one.
        2 if word.tone != 0 => {
            doc.keys(&word_keys(word, vni, None, false));
            doc.step(Step::Ch(tone_key(vni, other_tone(word, rng))));
            doc.step(Step::Ch(tone_key(vni, word.tone)));
            used.push("tone-change");
        }
        // The tone forgotten: space, Backspace back into the word, the tone.
        3 if word.tone != 0 => {
            doc.keys(&word_keys(word, vni, None, false));
            doc.keys(" ");
            doc.step(Step::Back);
            doc.step(Step::Ch(tone_key(vni, word.tone)));
            used.push("reopen");
        }
        // The whole word typed wrong, deleted with Backspace and typed again.
        4 => {
            doc.keys(&word_keys(word, vni, Some(other_tone(word, rng)), false));
            let n = doc.word_len_before_caret();
            for _ in 0..n {
                doc.step(Step::Back);
            }
            doc.keys(&keys);
            used.push("retype-word");
        }
        _ => doc.keys(&keys),
    }
}

/// Index of the first letter after the onset (`qu` and `gi` count as onsets when a vowel follows).
fn rime_start(word: &Word) -> usize {
    let plain: Vec<char> = word.letters.iter().map(|&(c, _)| c.to_ascii_lowercase()).collect();
    let first_vowel = plain.iter().position(|&c| is_vowel(c)).unwrap_or(plain.len());
    let two = |a: char, b: char| plain.len() > 2 && plain[0] == a && plain[1] == b && is_vowel(plain[2]);
    if first_vowel == 1 && (two('q', 'u') || two('g', 'i')) {
        return 2;
    }
    first_vowel
}

/// The wrong form a person types and only notices later: a wrong tone, a missing tone, or a missing
/// mark. The flag: only the tone is wrong (a tone key typed after the word fixes it, "gõ dấu đè").
fn wrong_keys(word: &Word, vni: bool, rng: &mut Rng) -> (String, bool) {
    let has_mark = word.letters.iter().any(|&(_, m)| m != Mark::None);
    match rng.below(3) {
        0 if word.tone != 0 => (word_keys(word, vni, None, false), true),
        1 if has_mark => {
            let plain = Word { letters: word.letters.iter().map(|&(c, _)| (c, Mark::None)).collect(), tone: word.tone };
            (word_keys(&plain, vni, Some(word.tone), false), false)
        }
        _ => (word_keys(word, vni, Some(other_tone(word, rng)), false), true),
    }
}

/// Type `lines` (one sentence each) like a person: in-word mistakes fixed at once, a few words left wrong and
/// fixed after everything is typed (Up to their line, End, Ctrl+Left to the word, delete it, retype it), words
/// deleted back and retyped. Returns the doc and the mistake kinds used.
fn scenario(lines: &[&str], vni: bool, seed: u64) -> (Doc, Vec<&'static str>) {
    let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
    let mut doc = Doc::new(vni);
    let mut used: Vec<&'static str> = Vec::new();
    let parsed: Vec<Vec<Word>> = lines.iter().map(|l| l.split(' ').map(parse_word).collect()).collect();
    // Words left wrong until the end: (line, word), one to three in all.
    let mut late: Vec<(usize, usize)> = Vec::new();
    for _ in 0..1 + rng.below(3) {
        let l = rng.below(lines.len());
        let w = 1 + rng.below(parsed[l].len() - 1);
        if !late.contains(&(l, w)) {
            late.push((l, w));
        }
    }
    // The words left wrong whose tone only is wrong (a tone key after them fixes them).
    let mut tone_only: Vec<(usize, usize)> = Vec::new();
    // The rightmost wrong word on the last line is fixed in memory: Left back to its end and the
    // right tone key ("gõ dấu đè"), when only its tone is wrong.
    let mut overwrite_first = false;
    for (l, words) in parsed.iter().enumerate() {
        let mut i = 0;
        while i < words.len() {
            let last = i + 1 == words.len();
            if late.contains(&(l, i)) {
                let (keys, only_tone) = wrong_keys(&words[i], vni, &mut rng);
                doc.keys(&keys);
                if only_tone {
                    tone_only.push((l, i));
                }
            } else if !last && i + 3 < words.len() && !late.iter().any(|&(ll, w)| ll == l && w > i && w <= i + 3) && rng.chance(12) {
                // Two or three words typed, then deleted back with Backspace and typed again.
                let count = 2 + rng.below(2);
                let start = doc.caret;
                for w in &words[i..i + count] {
                    doc.keys(&word_keys(w, vni, Some(w.tone), false));
                    doc.keys(" ");
                }
                for _ in 0..doc.caret - start {
                    doc.step(Step::Back);
                }
                for w in &words[i..i + count] {
                    doc.keys(&word_keys(w, vni, Some(w.tone), false));
                    doc.keys(" ");
                }
                used.push("bs-words-retype");
                i += count;
                continue;
            } else {
                type_word(&mut doc, words, i, vni, &mut rng, &mut used);
            }
            if !last {
                doc.keys(" ");
            } else if l + 1 < lines.len() {
                doc.step(Step::Enter);
            } else {
                // The sentence ends with a space; it goes again before going back (from after a space VS Code's
                // Ctrl+Left stops elsewhere than Chrome's and the terminal's; from a word's end all land on its start).
                doc.keys(" ");
                let first = late.iter().max().copied();
                overwrite_first =
                    first.is_some_and(|f| f.0 == l && tone_only.contains(&f)) && rng.chance(60);
                if !late.is_empty() && !overwrite_first {
                    doc.step(Step::Back);
                }
            }
            i += 1;
        }
    }
    // Fix the wrong words, last line first, right to left. The caret is at the end of the last line.
    late.sort();
    let mut at_line = lines.len() - 1;
    // The word whose end the caret is at, on `at_line`.
    let mut at_word = parsed[at_line].len() - 1;
    for (k, &(l, w)) in late.iter().rev().enumerate() {
        if k == 0 && overwrite_first {
            // Left back to the end of the word (the line is still in memory), then the right tone.
            let line_start = doc.text.iter().rposition(|&c| c == '\n').map_or(0, |p| p + 1);
            let end = line_start + lines[l].split(' ').take(w + 1).map(|s| s.chars().count()).sum::<usize>() + w;
            for _ in 0..doc.caret - end {
                doc.step(Step::Left);
            }
            let word = &parsed[l][w];
            doc.step(Step::Ch(tone_key(vni, word.tone)));
            used.push("fix-overwrite");
            used.push(if lines.len() > 1 { "fix-other-line" } else { "fix-mid-sentence" });
            at_word = w;
            continue;
        }
        if l != at_line {
            for _ in 0..at_line - l {
                doc.step(Step::Up);
            }
            doc.step(Step::End);
            at_line = l;
            at_word = parsed[l].len() - 1;
        }
        if w < at_word {
            for _ in 0..at_word - w {
                doc.step(Step::CtrlLeft);
            }
            doc.step(Step::Left);
        }
        let word = &parsed[l][w];
        let tail_from = rime_start(word);
        match rng.below(10) {
            // Owner: only the letters from the wrong-toned vowel on are deleted and typed again
            // (`tiểng`: Backspace over `ểng`, type `eengs`); the onset stays on screen.
            // (Not when the onset carries a mark: a wrong `d` for `đ` is outside the retyped part.)
            0..=3 if tail_from > 0 && word.letters[..tail_from].iter().all(|&(_, m)| m == Mark::None) => {
                for _ in 0..word.letters.len() - tail_from {
                    doc.step(Step::Back);
                }
                let tail = Word { letters: word.letters[tail_from..].to_vec(), tone: word.tone };
                doc.keys(&word_keys(&tail, vni, Some(word.tone), false));
                used.push("fix-rime-only");
            }
            4..=7 => {
                doc.step(Step::CtrlBack);
                doc.keys(&word_keys(word, vni, Some(word.tone), false));
                used.push("fix-ctrl-backspace");
            }
            _ => {
                let n = doc.word_len_before_caret();
                for _ in 0..n {
                    doc.step(Step::Back);
                }
                doc.keys(&word_keys(word, vni, Some(word.tone), false));
                used.push("fix-backspace");
            }
        }
        used.push(if lines.len() > 1 { "fix-other-line" } else { "fix-mid-sentence" });
        at_word = w;
    }
    if !late.is_empty() {
        // Back to the end. One line: End (in PowerShell Ctrl+End deletes to the end of the line).
        doc.step(if lines.len() > 1 { Step::CtrlEnd } else { Step::End });
    }
    (doc, used)
}

#[test]
fn human_scenarios_end_as_the_intended_sentence() {
    let export = std::env::var("REAL_APP_EXPORT").ok();
    let mut failures = Vec::new();
    let mut total = 0;
    for (vni, name) in [(false, "telex"), (true, "vni")] {
        let mut rows = String::from("id\tspec\texpected\tkinds\n");
        for seed in 0..48u64 {
            // Every third scenario is two or three sentences on separate lines.
            let n = if seed % 3 == 2 { 2 + (seed as usize / 3) % 2 } else { 1 };
            let lines: Vec<&str> = (0..n).map(|k| SENTENCES[(seed as usize * 3 + k * 7) % SENTENCES.len()]).collect();
            let (doc, used) = scenario(&lines, vni, seed + if vni { 1000 } else { 0 });
            total += 1;
            let want = lines.join("\n");
            let got = doc.lines();
            if got != want {
                failures.push(format!("{name} seed {seed}: {want:?}\n  keys: {}\n  got:  {got:?}", doc.spec()));
            }
            // Lines are joined with a literal `\n` in the TSV; the harness splits them back.
            rows.push_str(&format!("H{seed:02}\t{}\t{}\t{}\n", doc.spec(), want.replace('\n', "\\n"), used.join(",")));
        }
        if let Some(dir) = &export {
            std::fs::write(std::path::Path::new(dir).join(format!("human-{name}.tsv")), rows).expect("write export");
        }
    }
    assert!(failures.is_empty(), "{} of {total} scenarios wrong:\n{}", failures.len(), failures.join("\n"));
}