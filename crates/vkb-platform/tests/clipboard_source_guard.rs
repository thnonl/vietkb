//! Source-level guards for the Windows clipboard executor.
//!
//! `crates/vkb-windows/src/clipboard.rs` only compiles on Windows, so the rules
//! that keep it safe but live in its CALL ORDER cannot be exercised by the
//! host's tests. These tests read the file as text (no OS code, no parsing
//! beyond top-level `fn` items) and pin the call-order rules from
//! `70-impl-notes-windows.md` section 9:
//!
//! 1. `GetClipboardData(` is called only from `capture`, `read_dword`,
//!    `clipboard_text_fp` (and the test-only `read_text`).
//! 2. `capture` calls `read_allowed()` (the owner probe, with the clipboard
//!    open) before its first read.
//! 3. `clipboard_text_fp(` is only called from `clear_if_ours`, behind an
//!    `owner_is_ours &&` (a read of OUR window's clipboard, never a foreign one).
//! 4. `try_paste` records `shared.text =` before it calls `write_temp_text(`.
//! 5. `try_paste` probes the current owner after opening the clipboard and
//!    before the first write, and plans a failed write with `failed_write_plan(`.
//!
//! Line endings: a Windows checkout with `core.autocrlf=true` has CRLF in the
//! file. Every guard therefore normalises the source ONCE (`normalise`) and
//! works on the normalised text only; `guards_hold_on_a_crlf_checkout` runs all
//! of them over a CRLF-converted copy and a mutated CRLF copy must still fail,
//! so no guard can pass vacuously on CRLF.

const SOURCE: &str = include_str!("../../vkb-windows/src/clipboard.rs");

/// LF-only text, whatever the checkout's line endings are.
fn normalise(source: &str) -> String {
    source.replace("\r\n", "\n")
}

/// The production part of the file (the `#[cfg(test)] mod tests` is cut off),
/// without `//` and `///` comment lines. Takes a NORMALISED source.
fn production_code(normalised: &str) -> String {
    let end = normalised
        .find("#[cfg(test)]\nmod tests")
        .expect("clipboard.rs must keep its `#[cfg(test)] mod tests`");
    strip_comments(&normalised[..end])
}

fn strip_comments(code: &str) -> String {
    code.lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// A top-level `fn`: its name, the attribute lines right above it, its text.
struct Func {
    name: String,
    attrs: Vec<String>,
    body: String,
}

/// Split into top-level functions (items that start in column 0).
fn functions(code: &str) -> Vec<Func> {
    let mut out: Vec<Func> = Vec::new();
    let mut attrs: Vec<String> = Vec::new();
    let mut current: Option<Func> = None;
    for line in code.lines() {
        let top_level = !line.is_empty()
            && !line.starts_with(char::is_whitespace)
            && !line.starts_with('}')
            && !line.starts_with(')');
        if top_level {
            if let Some(f) = current.take() {
                out.push(f);
            }
            if line.starts_with("#[") {
                attrs.push(line.to_string());
                continue;
            }
            let decl = line
                .trim_start_matches("pub ")
                .trim_start_matches("unsafe extern \"system\" ")
                .trim_start_matches("unsafe ");
            if let Some(rest) = decl.strip_prefix("fn ") {
                let name: String =
                    rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
                current = Some(Func { name, attrs: std::mem::take(&mut attrs), body: String::new() });
            } else {
                attrs.clear();
            }
        }
        if let Some(f) = current.as_mut() {
            f.body.push_str(line);
            f.body.push('\n');
        }
    }
    if let Some(f) = current {
        out.push(f);
    }
    out
}

fn find<'a>(funcs: &'a [Func], name: &str) -> &'a Func {
    funcs.iter().find(|f| f.name == name).unwrap_or_else(|| panic!("fn {name} not found"))
}

// ---- the guards: each takes the RAW source and normalises it itself ----------

fn guard_get_clipboard_data_callers(raw: &str) {
    let normalised = normalise(raw);
    let code = production_code(&normalised);
    let funcs = functions(&code);
    let mut callers = Vec::new();
    for f in &funcs {
        // `#[cfg(test)]` helpers are not production code.
        let test_only = f.attrs.iter().any(|a| a.contains("cfg(test)"));
        if f.body.contains("GetClipboardData(") && !test_only {
            callers.push(f.name.clone());
        }
    }
    callers.sort();
    // A reader that is not `#[cfg(test)]` shows up here and fails the comparison.
    assert_eq!(
        callers,
        vec!["capture", "clipboard_text_fp", "read_dword"],
        "GetClipboardData( may only be called from capture, read_dword and clipboard_text_fp"
    );
    // The diagnostic reader is test-only (checked on the parsed attributes, not
    // on a line-ending-dependent substring).
    let read_text = find(&funcs, "read_text");
    assert!(
        read_text.attrs.iter().any(|a| a.contains("cfg(test)")),
        "read_text must stay #[cfg(test)] (it reads a clipboard without a probe)"
    );
}

fn guard_capture_probes_first(raw: &str) {
    let normalised = normalise(raw);
    let code = production_code(&normalised);
    let funcs = functions(&code);
    let capture = find(&funcs, "capture");
    let gate = capture.body.find("read_allowed()").expect("capture must call read_allowed()");
    let first_read = [capture.body.find("read_dword("), capture.body.find("GetClipboardData(")]
        .into_iter()
        .flatten()
        .min()
        .expect("capture reads data");
    assert!(gate < first_read, "capture must call read_allowed() before its first read");
    // And the gate really stops the read.
    assert!(
        capture.body[gate..first_read].contains("return None"),
        "a refused read_allowed() must end the capture"
    );
}

fn guard_our_text_read_only_from_our_window(raw: &str) {
    let normalised = normalise(raw);
    let code = production_code(&normalised);
    let funcs = functions(&code);
    let mut callers = Vec::new();
    for f in &funcs {
        let calls = f.body.matches("clipboard_text_fp(").count();
        let declares = usize::from(f.name == "clipboard_text_fp");
        if calls > declares {
            callers.push(f.name.clone());
        }
    }
    assert_eq!(callers, vec!["clear_if_ours"], "clipboard_text_fp( may only be called by clear_if_ours");
    let clear = find(&funcs, "clear_if_ours");
    let owner_check = clear.body.find("let owner_is_ours").expect("owner_is_ours is computed");
    let call = clear.body.find("clipboard_text_fp(").unwrap();
    assert!(owner_check < call, "owner_is_ours must be computed before the read");
    let guarded = clear.body[owner_check..call]
        .split(';')
        .next_back()
        .is_some_and(|stmt| stmt.contains("owner_is_ours") && stmt.contains("&&"));
    assert!(guarded, "the read must be behind `owner_is_ours &&`");
}

fn guard_text_recorded_before_write(raw: &str) {
    let normalised = normalise(raw);
    let code = production_code(&normalised);
    let funcs = functions(&code);
    let paste = find(&funcs, "try_paste");
    let record = paste.body.find("shared.text =").expect("try_paste records our text");
    let write = paste.body.find("write_temp_text(").expect("try_paste writes the text");
    assert!(record < write, "shared.text must be assigned BEFORE write_temp_text(");
    assert_eq!(paste.body.matches("shared.text =").count(), 1, "recorded exactly once");
}

fn guard_probe_after_open_before_write(raw: &str) {
    let normalised = normalise(raw);
    let code = production_code(&normalised);
    let funcs = functions(&code);
    let paste = find(&funcs, "try_paste");
    let open = paste.body.find("ClipboardGuard::open(").expect("try_paste opens the clipboard");
    let write = paste.body.find("write_temp_text(").expect("try_paste writes the text");
    // Some probe sits between the open and the first write (the probe before the
    // open does not count: the owner can change in between).
    assert!(
        paste.body[open..write].contains("clipboard_owner_responsive("),
        "try_paste must probe the current owner after OpenClipboard and before write_temp_text("
    );
    assert!(
        paste.body[open..write].contains("return false"),
        "an unanswered probe must end the paste (the guard closes the clipboard)"
    );
}

fn guard_failed_write_planned(raw: &str) {
    let normalised = normalise(raw);
    let code = production_code(&normalised);
    let funcs = functions(&code);
    let paste = find(&funcs, "try_paste");
    assert!(
        paste.body.contains("failed_write_plan("),
        "try_paste must decide a failed write with the pure failed_write_plan"
    );
    // The write reports how far it got.
    let write = find(&funcs, "write_temp_text");
    assert!(write.body.contains("WriteFailure::NotEmptied") && write.body.contains("WriteFailure::Emptied"));
}

fn guard_parser_sees_expected_functions(raw: &str) {
    // Sanity: a refactor that renames these would silently disable the guards.
    let normalised = normalise(raw);
    let code = production_code(&normalised);
    let funcs = functions(&code);
    for name in [
        "capture",
        "read_dword",
        "clipboard_text_fp",
        "clear_if_ours",
        "try_paste",
        "read_allowed",
        "write_temp_text",
        "read_text",
    ] {
        find(&funcs, name);
    }
    assert!(funcs.iter().all(|f| f.attrs.len() < 8));
}

fn all_guards(raw: &str) {
    guard_get_clipboard_data_callers(raw);
    guard_capture_probes_first(raw);
    guard_our_text_read_only_from_our_window(raw);
    guard_text_recorded_before_write(raw);
    guard_probe_after_open_before_write(raw);
    guard_failed_write_planned(raw);
    guard_parser_sees_expected_functions(raw);
}

// ---- the tests -----------------------------------------------------------------

#[test]
fn get_clipboard_data_is_called_only_from_the_gated_readers() {
    guard_get_clipboard_data_callers(SOURCE);
}

#[test]
fn capture_probes_the_owner_before_its_first_read() {
    guard_capture_probes_first(SOURCE);
}

#[test]
fn our_text_is_only_read_from_our_own_window() {
    guard_our_text_read_only_from_our_window(SOURCE);
}

#[test]
fn try_paste_records_our_text_before_it_writes() {
    guard_text_recorded_before_write(SOURCE);
}

#[test]
fn try_paste_probes_the_current_owner_after_opening_and_before_writing() {
    guard_probe_after_open_before_write(SOURCE);
}

#[test]
fn a_failed_write_is_planned_by_what_it_changed() {
    guard_failed_write_planned(SOURCE);
}

#[test]
fn the_guard_parser_sees_the_expected_functions() {
    guard_parser_sees_expected_functions(SOURCE);
}

/// `core.autocrlf=true` checkouts: the same file with CRLF line endings must
/// give the same results, and the CRLF copy must really be CRLF.
#[test]
fn guards_hold_on_a_crlf_checkout() {
    let lf = normalise(SOURCE);
    let crlf = lf.replace('\n', "\r\n");
    assert!(crlf.contains("\r\n") && !crlf.contains("#[cfg(test)]\nfn read_text"));
    // A guard that looked for "\n"-joined substrings in the raw text would miss.
    assert!(!crlf.contains("#[cfg(test)]\nmod tests"));
    all_guards(&crlf);
    // Mixed endings (a partly converted checkout) too.
    let mixed: String = lf
        .lines()
        .enumerate()
        .map(|(i, l)| if i % 2 == 0 { format!("{l}\r\n") } else { format!("{l}\n") })
        .collect();
    all_guards(&mixed);
}

/// A violation must still be caught on CRLF: none of the guards may pass
/// vacuously there. Each mutation breaks exactly one rule in a CRLF copy.
#[test]
fn guards_still_fail_on_a_mutated_crlf_checkout() {
    let lf = normalise(SOURCE);
    let crlf = lf.replace('\n', "\r\n");
    let mutate = |from: &str, to: &str| -> String {
        let from = from.replace('\n', "\r\n");
        let to = to.replace('\n', "\r\n");
        assert_eq!(crlf.matches(&from).count(), 1, "mutation anchor not unique: {from:?}");
        crlf.replacen(&from, &to, 1)
    };
    let fails = |f: fn(&str), text: &str| {
        std::panic::catch_unwind(|| f(text)).is_err()
    };
    // read_text without #[cfg(test)]
    let m = mutate("#[cfg(test)]\nfn read_text", "fn read_text");
    assert!(fails(guard_get_clipboard_data_callers, &m), "read_text guard passed vacuously");
    // our text recorded after the write
    let m = mutate("    shared.text = OurText::of(text);\n", "");
    assert!(fails(guard_text_recorded_before_write, &m), "record-before-write passed vacuously");
    // no probe between open and write
    let m = mutate(
        "    if !clipboard_owner_responsive(20) {\n        return false;\n    }\n    // The snapshot moves into the plan",
        "    // The snapshot moves into the plan",
    );
    assert!(fails(guard_probe_after_open_before_write, &m), "probe guard passed vacuously");
    // the own-window condition dropped
    let m = mutate(
        "owner_is_ours && !marker && text.fp != 0 && clipboard_text_fp(text)",
        "!marker && text.fp != 0 && clipboard_text_fp(text)",
    );
    assert!(fails(guard_our_text_read_only_from_our_window, &m), "own-window guard passed vacuously");
}
