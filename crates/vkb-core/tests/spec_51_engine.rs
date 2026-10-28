//! Executable form of `docs/evkey-clone-work/50-test-cases.md` §1 (engine corpus,
//! 551 rows) and §2 (charset round-trips).
//!
//! The markdown table is the single source of truth: this test parses §1 at run
//! time, so a correction made to the table is picked up without touching Rust
//! code. Options come from the §0 option table:
//!
//! | category | deviation from the §0 baseline |
//! |---|---|
//! | `spell-check`, `english-passthrough` | `spell_check = true` |
//! | `macro` | `macro_enabled = true` + the 7-row §1.14 macro table |
//! | `restore-word` | `restore_wrong_word = true` |
//! | `tone-old-style` | `modern_tone = false` |
//!
//! Two rows override their category baseline and are handled by id:
//! `E-TT-061` (`jfwz_as_consonant = true`, stated in its own row) and
//! `E-MC-016` (`macro_enabled = false`, idem).
//!
//! The test always checks its own integrity (551 rows, unique ids, per-category
//! counts from the `### … — N rows` headings) and writes
//! `target/qa/spec51-results.tsv`. Behavioural mismatches are reported, and the
//! test only turns red for them when `VKB_SPEC_STRICT=1` is set:
//!
//! ```text
//! cargo test -p vkb-core --test spec_51_engine -- --nocapture
//! $env:VKB_SPEC_STRICT=1; cargo test -p vkb-core --test spec_51_engine
//! ```

use std::collections::BTreeSet;
use std::path::PathBuf;

use vkb_core::{
    apply_action, simulate_keys, Charset, Engine, EngineOptions, InputMethod, KeyInput, MacroTable,
    VietnameseEngine, VK_ESCAPE, VK_TAB,
};

// ---------------------------------------------------------------------------
// Spec location
// ---------------------------------------------------------------------------

fn spec_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/evkey-clone-work/50-test-cases.md")
}

const EXPECTED_ROWS: usize = 551;

// ---------------------------------------------------------------------------
// Row model + parser
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct Row {
    id: String,
    method: String,
    /// Raw keystrokes after `<sp>`/`<bs>`/… expansion, control chars included.
    input: String,
    expected: String,
    category: String,
    why: String,
}

fn parse_section1(text: &str) -> (Vec<Row>, Vec<(String, usize)>) {
    let mut rows = Vec::new();
    let mut headings: Vec<(String, usize)> = Vec::new();
    let mut in_section = false;
    let mut line_no = 0usize;

    for line in text.lines() {
        line_no += 1;
        if line.starts_with("## 1. Engine corpus") {
            in_section = true;
            continue;
        }
        if in_section && line.starts_with("## 2.") {
            break;
        }
        if !in_section {
            continue;
        }
        if let Some(rest) = line.strip_prefix("### ") {
            // `1.1 telex-basic — 60 rows`
            let (left, right) = rest
                .split_once('—')
                .unwrap_or_else(|| panic!("heading without an em dash on line {line_no}: {rest}"));
            let name = left
                .split_once(' ')
                .map(|(_, name)| name)
                .unwrap_or(left)
                .trim()
                .to_string();
            let declared: usize = right
                .trim()
                .split(' ')
                .next()
                .and_then(|n| n.parse().ok())
                .unwrap_or_else(|| panic!("heading without a row count on line {line_no}: {rest}"));
            headings.push((name, declared));
            continue;
        }
        if !line.starts_with("| E-") {
            continue;
        }
        let cells: Vec<&str> = line.split('|').collect();
        // | id | method | input | expected | category | why |
        assert_eq!(
            cells.len(),
            8,
            "line {line_no}: expected 6 cells, got {}: {line}",
            cells.len() - 2
        );
        let cell = |idx: usize| cells[idx].trim().trim_matches('`').to_string();
        rows.push(Row {
            id: cell(1),
            method: cell(2),
            input: expand(&cell(3)),
            expected: expand(&cell(4)),
            category: cell(5),
            why: cell(6),
        });
    }

    (rows, headings)
}

/// Expand the `<sp>`/`<bs>`/`<ent>`/`<tab>`/`<esc>`/`<bt>`/`<empty>` tokens used
/// by every table in the spec.
fn expand(source: &str) -> String {
    const TOKENS: [(&str, Option<char>); 7] = [
        ("<empty>", None),
        ("<space>", Some(' ')),
        ("<sp>", Some(' ')),
        ("<bs>", Some('\u{8}')),
        ("<ent>", Some('\r')),
        ("<tab>", Some('\t')),
        ("<esc>", Some('\u{1b}')),
    ];
    let mut out = String::with_capacity(source.len());
    let mut rest = source;
    'outer: while !rest.is_empty() {
        if let Some(protected) = rest.strip_prefix("<bt>") {
            out.push('`');
            rest = protected;
            continue;
        }
        for (token, replacement) in TOKENS {
            if let Some(after) = rest.strip_prefix(token) {
                if let Some(ch) = replacement {
                    out.push(ch);
                }
                rest = after;
                continue 'outer;
            }
        }
        let ch = rest.chars().next().expect("non-empty");
        out.push(ch);
        rest = &rest[ch.len_utf8()..];
    }
    out
}

fn keys_for(input: &str) -> Vec<KeyInput> {
    input
        .chars()
        .map(|ch| match ch {
            '\u{8}' => KeyInput::backspace(),
            '\r' => KeyInput::enter(),
            '\t' => KeyInput::key(VK_TAB, Some('\t')),
            '\u{1b}' => KeyInput::key(VK_ESCAPE, Some('\u{1b}')),
            other => KeyInput::char(other),
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Options
// ---------------------------------------------------------------------------

fn spec_options(method: &str, category: &str, id: &str) -> EngineOptions {
    let method = InputMethod::from_str(method)
        .unwrap_or_else(|| panic!("{id}: unknown method {method:?}"));
    let mut options = EngineOptions {
        method,
        charset: Charset::Unicode,
        modern_tone: true,
        spell_check: false,
        restore_wrong_word: false,
        jfwz_as_consonant: false,
        macro_enabled: false,
        auto_capitalize: false,
        free_tone: true,
    };
    match category {
        "spell-check" | "english-passthrough" => options.spell_check = true,
        "macro" => options.macro_enabled = true,
        "restore-word" => options.restore_wrong_word = true,
        "tone-old-style" => options.modern_tone = false,
        _ => {}
    }
    // Per-row overrides stated in the row itself.
    match id {
        "E-TT-061" => options.jfwz_as_consonant = true,
        "E-MC-016" => options.macro_enabled = false,
        _ => {}
    }
    options
}

fn spec_macros() -> MacroTable {
    MacroTable::from_pairs([
        ("btw", "by the way"),
        ("vn", "Việt Nam"),
        ("ko", "không"),
        ("dc", "được"),
        ("ts", "tiến sĩ"),
        ("aa", "@@"),
        ("vietnam", "Việt Nam"),
    ])
}

fn run(row: &Row) -> String {
    let options = spec_options(&row.method, &row.category, &row.id);
    if row.category == "macro" {
        let mut engine = Engine::new(options);
        engine.set_macros(spec_macros());
        let mut out = String::new();
        for key in keys_for(&row.input) {
            let action = engine.process_key(key);
            apply_action(&mut out, action, key);
        }
        return out;
    }
    simulate_keys(options, &keys_for(&row.input))
}

fn visible(text: &str) -> String {
    text.chars()
        .map(|ch| match ch {
            '\r' => "<ent>".to_string(),
            '\t' => "<tab>".to_string(),
            '\u{1b}' => "<esc>".to_string(),
            '\u{8}' => "<bs>".to_string(),
            ' ' => "<sp>".to_string(),
            other => other.to_string(),
        })
        .collect()
}

// ---------------------------------------------------------------------------
// §1 corpus check
// ---------------------------------------------------------------------------

#[test]
fn spec_section1_engine_corpus() {
    let path = spec_path();
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("cannot read {}: {err}", path.display()));
    let (rows, headings) = parse_section1(&text);

    // --- harness integrity -------------------------------------------------
    assert_eq!(
        rows.len(),
        EXPECTED_ROWS,
        "§1 declares {EXPECTED_ROWS} rows; parsed {}",
        rows.len()
    );
    let ids: BTreeSet<&str> = rows.iter().map(|row| row.id.as_str()).collect();
    assert_eq!(ids.len(), rows.len(), "duplicate row ids in §1");
    let declared_total: usize = headings.iter().map(|(_, count)| count).sum();
    assert_eq!(declared_total, EXPECTED_ROWS, "§1 headings do not add up");

    for (name, declared) in &headings {
        let parsed = rows.iter().filter(|row| &row.category == name).count();
        assert_eq!(
            parsed, *declared,
            "category {name}: heading says {declared} rows, table has {parsed}"
        );
    }

    // --- run ---------------------------------------------------------------
    struct Outcome {
        id: String,
        category: String,
        method: String,
        input: String,
        expected: String,
        actual: String,
        why: String,
    }
    let mut failures: Vec<Outcome> = Vec::new();
    for row in &rows {
        let actual = run(row);
        if actual != row.expected {
            failures.push(Outcome {
                id: row.id.clone(),
                category: row.category.clone(),
                method: row.method.clone(),
                input: visible(&row.input),
                expected: visible(&row.expected),
                actual: visible(&actual),
                why: row.why.clone(),
            });
        }
    }

    // --- report ------------------------------------------------------------
    let report_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/qa");
    std::fs::create_dir_all(&report_dir).expect("create target/qa");
    let mut report = String::from("id\tcategory\tmethod\tinput\texpected\tactual\tstatus\n");
    for row in &rows {
        let actual = run(row);
        let status = if actual == row.expected { "PASS" } else { "FAIL" };
        report.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            row.id,
            row.category,
            row.method,
            visible(&row.input),
            visible(&row.expected),
            visible(&actual),
            status
        ));
    }
    std::fs::write(report_dir.join("spec51-results.tsv"), report).expect("write report");

    println!("\n§1 engine corpus — {} rows", rows.len());
    println!("{:<24} {:>6} {:>6} {:>6}", "category", "total", "pass", "fail");
    println!("{}", "-".repeat(46));
    for (name, declared) in &headings {
        let total = rows.iter().filter(|row| &row.category == name).count();
        let failed = failures.iter().filter(|row| &row.category == name).count();
        println!(
            "{:<24} {:>6} {:>6} {:>6}",
            name,
            total,
            total - failed,
            failed
        );
        assert_eq!(total, *declared);
    }
    println!("{}", "-".repeat(46));
    println!(
        "{:<24} {:>6} {:>6} {:>6}",
        "TOTAL",
        rows.len(),
        rows.len() - failures.len(),
        failures.len()
    );

    if !failures.is_empty() {
        println!("\nFAILURES\n");
        for failure in &failures {
            println!(
                "{}\t{}\tinput {:?} | expected {:?} | actual {:?}\n\twhy: {}",
                failure.id,
                failure.method,
                failure.input,
                failure.expected,
                failure.actual,
                failure.why
            );
        }
    }

    if std::env::var("VKB_SPEC_STRICT").as_deref() == Ok("1") {
        assert!(
            failures.is_empty(),
            "{} of {} §1 rows mismatch (see target/qa/spec51-results.tsv)",
            failures.len(),
            rows.len()
        );
    }
}

// ---------------------------------------------------------------------------
// §2 charset round-trips
// ---------------------------------------------------------------------------

/// The 14 representative letters, in the order of the §2 tables.
const LETTERS: [char; 14] = [
    'ă', 'â', 'ê', 'ô', 'ơ', 'ư', 'đ', 'Ă', 'Â', 'Ê', 'Ô', 'Ơ', 'Ư', 'Đ',
];

const S1: &str = "Đường về quê hương";

fn assert_encode(charset: Charset, expected: &[&[u8]]) {
    let mut bad = Vec::new();
    for (letter, want) in LETTERS.iter().zip(expected) {
        let got = charset.encode(&letter.to_string());
        if got != *want {
            bad.push(format!(
                "  {letter} → got {} want {}",
                hex(&got),
                hex(want)
            ));
        }
    }
    assert!(
        bad.is_empty(),
        "{} letter table mismatches:\n{}",
        charset.as_str(),
        bad.join("\n")
    );
}

fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn spec_section2_unicode_codepoints() {
    // CS-UN-01..14, plus the S1 code points from §2.1.
    let codepoints = [
        0x0103u32, 0x00E2, 0x00EA, 0x00F4, 0x01A1, 0x01B0, 0x0111, 0x0102, 0x00C2, 0x00CA, 0x00D4,
        0x01A0, 0x01AF, 0x0110,
    ];
    for (letter, codepoint) in LETTERS.iter().zip(codepoints) {
        assert_eq!(*letter as u32, codepoint, "{letter} codepoint");
        assert_eq!(
            Charset::Unicode.encode(&letter.to_string()),
            letter.to_string().as_bytes(),
            "unicode encode of {letter}"
        );
    }
    // S1 must be NFC already and encode 1:1 to its UTF-8 bytes.
    assert_eq!(Charset::Unicode.encode(S1), S1.as_bytes());
    let s1: Vec<u32> = S1.chars().map(|ch| ch as u32).collect();
    assert_eq!(
        s1,
        vec![
            0x0110, 0x01B0, 0x1EDD, 0x006E, 0x0067, 0x0020, 0x0076, 0x1EC1, 0x0020, 0x0071, 0x0075,
            0x00EA, 0x0020, 0x0068, 0x01B0, 0x01A1, 0x006E, 0x0067
        ]
    );
}

#[test]
fn spec_section2_nfd_decomposition() {
    // CS-NF-01..14: base + combining mark, `đ` has no decomposition, and the
    // mark comes before the tone (§2.2 R2 ordering clause).
    let expected: [&str; 14] = [
        "a\u{306}",
        "a\u{302}",
        "e\u{302}",
        "o\u{302}",
        "o\u{31B}",
        "u\u{31B}",
        "\u{111}",
        "A\u{306}",
        "A\u{302}",
        "E\u{302}",
        "O\u{302}",
        "O\u{31B}",
        "U\u{31B}",
        "\u{110}",
    ];
    for (letter, want) in LETTERS.iter().zip(expected) {
        assert_eq!(
            Charset::UnicodeNfd.encode(&letter.to_string()),
            want.as_bytes(),
            "nfd of {letter}"
        );
    }
    assert_eq!(
        Charset::UnicodeNfd.encode("ế"),
        "\u{65}\u{302}\u{301}".as_bytes()
    );
    assert_eq!(
        Charset::UnicodeNfd.encode("ờ"),
        "\u{6F}\u{31B}\u{300}".as_bytes()
    );
    // S1 in NFD: 17 chars become 17 + one combining mark per marked letter.
    let nfd = Charset::UnicodeNfd.encode(S1);
    assert_eq!(nfd, "Đu\u{31B}o\u{31B}\u{300}ng ve\u{302}\u{300} que\u{302} hu\u{31B}o\u{31B}ng".as_bytes());
}

#[test]
fn spec_section2_utf8_bytes() {
    // CS-U8-01..14 — UTF-8 transport of the same scalars.
    let expected: [&[u8]; 14] = [
        &[0xC4, 0x83],
        &[0xC3, 0xA2],
        &[0xC3, 0xAA],
        &[0xC3, 0xB4],
        &[0xC6, 0xA1],
        &[0xC6, 0xB0],
        &[0xC4, 0x91],
        &[0xC4, 0x82],
        &[0xC3, 0x82],
        &[0xC3, 0x8A],
        &[0xC3, 0x94],
        &[0xC6, 0xA0],
        &[0xC6, 0xAF],
        &[0xC4, 0x90],
    ];
    assert_encode(Charset::Utf8, &expected);
    assert_eq!(Charset::Utf8.encode(S1), S1.as_bytes());
}

#[test]
fn spec_section2_viqr() {
    // CS-VQ-01..14. `đ` is `dd`, not `d-`: RFC 1456 Table 2 ("d bar | dd |
    // (repeated d)"). §2.4 of the spec was corrected accordingly.
    let expected: [&[u8]; 14] = [
        b"a(",
        b"a^",
        b"e^",
        b"o^",
        b"o+",
        b"u+",
        b"dd",
        b"A(",
        b"A^",
        b"E^",
        b"O^",
        b"O+",
        b"U+",
        b"DD",
    ];
    assert_encode(Charset::Viqr, &expected);
    assert_eq!(
        Charset::Viqr.encode(S1),
        b"DDu+o+`ng ve^` que^ hu+o+ng".to_vec()
    );
}

/// R3: a charset that cannot represent a letter must not silently drop it.
/// `Charset::encode` reports an unmapped letter as `?`, so `?` in the output of
/// text that contains none is a no-loss failure.
#[test]
fn spec_section2_no_silent_loss_on_s1() {
    for charset in Charset::ALL {
        let bytes = charset.encode(S1);
        assert!(!bytes.is_empty(), "{}: empty output", charset.as_str());
        assert!(
            !bytes.contains(&b'?'),
            "{}: S1 lost a letter to `?`: {}",
            charset.as_str(),
            hex(&bytes)
        );
    }
}

/// The publishable evidence for the five charsets §2 marks UNVERIFIED: what the
/// engine actually emits for the 14 letters and for S1. Values are printed, not
/// asserted — there is no authoritative table in the repo to assert against.
#[test]
fn spec_section2_evidence_for_unverified_charsets() {
    let unverified = [
        Charset::Tcvn3,
        Charset::VniWindows,
        Charset::Viscii,
        Charset::BkHcm1,
        Charset::Vps,
    ];
    println!("\n§2 evidence — 14 letters and S1, lower-case hex\n");
    for charset in unverified {
        let letters: Vec<String> = LETTERS
            .iter()
            .map(|letter| format!("{letter}={}", hex(&charset.encode(&letter.to_string()))))
            .collect();
        println!("[{}] {}", charset.as_str(), charset.status_note());
        println!("  {}", letters.join(" "));
        let s1 = charset.encode(S1);
        println!(
            "  S1: {} ({} bytes for {} chars)",
            hex(&s1),
            s1.len(),
            S1.chars().count()
        );
    }

    // Structural checks that do not depend on the (unverified) values:
    // TCVN3 / VISCII / VPS are single-byte per letter.
    for charset in [Charset::Tcvn3, Charset::Viscii, Charset::Vps] {
        for letter in LETTERS {
            assert_eq!(
                charset.encode(&letter.to_string()).len(),
                1,
                "{}: {letter} must be one byte",
                charset.as_str()
            );
        }
        assert_eq!(
            charset.encode(S1).len(),
            S1.chars().count(),
            "{}: S1 must be one byte per character",
            charset.as_str()
        );
    }
    // VNI Windows and BK HCM1 are base + combining mark (2 bytes for a toned
    // vowel), so S1 is longer than its 17 characters.
    for charset in [Charset::VniWindows, Charset::BkHcm1] {
        assert!(
            charset.encode(S1).len() > S1.chars().count(),
            "{}: combining layout must be longer than the character count",
            charset.as_str()
        );
    }
    // BK HCM1 is currently defined as the VNI Windows layout (documented
    // assumption, `TableStatus::Unverified`).
    assert_eq!(Charset::BkHcm1.encode(S1), Charset::VniWindows.encode(S1));
}
