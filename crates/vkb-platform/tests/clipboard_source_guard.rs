//! Source-level guards for the Windows clipboard executor.
//!
//! `crates/vkb-windows/src/clipboard.rs` only compiles on Windows, so the rules
//! that keep it safe but live in its CALL ORDER cannot be exercised by the
//! host's tests. These tests read the file as text (no OS code) and pin the
//! rules from `70-impl-notes-windows.md` section 9. They pin USAGE, not mere
//! presence: a pin names the exact statement (`if !read_allowed() {` returning
//! `None`, ...), not just a substring somewhere.
//!
//! 1. `GetClipboardData(` is called only from `capture`, `read_dword`,
//!    `clipboard_text_fp` (and the `#[cfg(test)]` `read_text`), and every call
//!    site of the production code lies inside a collected function body (no
//!    hidden call site in a const / static / closure outside any function).
//! 2. `capture` calls `if !read_allowed() {` -> `return None;` before its first
//!    read; `read_dword(` is only called from `capture`.
//! 3. `clipboard_text_fp(` is only called from `clear_if_ours`, in the exact
//!    expression `owner_is_ours && ...` (an `||` is rejected).
//! 4. `try_paste` records `shared.text =` before it calls `write_temp_text(`,
//!    restores the previous text when nothing was written, probes the current
//!    owner with `if !clipboard_owner_responsive(20) {` -> `return false;` after
//!    opening the clipboard and before the first write, and plans a failed write
//!    with `failed_write_plan(`.
//! 5. `GetClipboardOwner(` is called only from `current_owner` (which tells "no
//!    owner" from a real error).
//!
//! Line endings: a Windows checkout with `core.autocrlf=true` has CRLF in the
//! file. Every guard normalises the source ONCE (`normalise`) and works on the
//! normalised text only; `guards_hold_on_a_crlf_checkout` runs all of them over
//! CRLF and mixed copies, and mutated CRLF copies must still fail.

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

/// A function item (free function, impl method, `pub(crate)`, `const`,
/// `unsafe extern "system"`): its name, the attribute lines right above it and
/// its whole text (nested items included).
struct Func {
    name: String,
    attrs: Vec<String>,
    body: String,
}

/// `fn name` of an item line, after stripping qualifiers.
fn fn_name(line: &str) -> Option<String> {
    let mut rest = line;
    loop {
        let before = rest;
        for q in [
            "pub(crate) ",
            "pub(super) ",
            "pub ",
            "const ",
            "unsafe ",
            "async ",
            "extern \"system\" ",
            "extern \"C\" ",
        ] {
            if let Some(r) = rest.strip_prefix(q) {
                rest = r;
            }
        }
        if rest == before {
            break;
        }
    }
    let rest = rest.strip_prefix("fn ")?;
    let name: String = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
    (!name.is_empty()).then_some(name)
}

/// Net `{` minus `}` of a line, ignoring string and char literals.
fn brace_delta(line: &str) -> i32 {
    let chars: Vec<char> = line.chars().collect();
    let (mut delta, mut i, mut in_str) = (0i32, 0usize, false);
    while i < chars.len() {
        let c = chars[i];
        if in_str {
            if c == '\\' {
                i += 1;
            } else if c == '"' {
                in_str = false;
            }
        } else if c == '"' {
            in_str = true;
        } else if c == '\'' {
            // char literal 'x' or '\x'; otherwise a lifetime
            if chars.get(i + 2) == Some(&'\'') {
                i += 2;
            } else if chars.get(i + 1) == Some(&'\\') && chars.get(i + 3) == Some(&'\'') {
                i += 3;
            }
        } else if c == '{' {
            delta += 1;
        } else if c == '}' {
            delta -= 1;
        }
        i += 1;
    }
    delta
}

/// Every function item, at any indentation (impl methods included), each with
/// its brace-balanced text. Items nested in a function stay inside its text.
fn functions(code: &str) -> Vec<Func> {
    let lines: Vec<&str> = code.lines().collect();
    let mut out = Vec::new();
    let mut attrs: Vec<String> = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim_start();
        if trimmed.starts_with("#[") {
            attrs.push(trimmed.to_string());
            i += 1;
            continue;
        }
        if let Some(name) = fn_name(trimmed) {
            let (mut depth, mut started) = (0i32, false);
            let mut body = String::new();
            while i < lines.len() {
                let l = lines[i];
                body.push_str(l);
                body.push('\n');
                depth += brace_delta(l);
                started |= depth > 0 || l.contains('{');
                i += 1;
                if (started && depth <= 0) || (!started && l.trim_end().ends_with(';')) {
                    break;
                }
            }
            out.push(Func { name, attrs: std::mem::take(&mut attrs), body });
            continue;
        }
        if !trimmed.is_empty() {
            attrs.clear();
        }
        i += 1;
    }
    out
}

fn find<'a>(funcs: &'a [Func], name: &str) -> &'a Func {
    funcs.iter().find(|f| f.name == name).unwrap_or_else(|| panic!("fn {name} not found"))
}

fn squash(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The statements inside the brace block that `header` (ending in `{`) opens,
/// searched from `from`; returns (start index of the header, squashed inner).
fn block(body: &str, header: &str, from: usize) -> Option<(usize, String)> {
    let start = from + body[from..].find(header)?;
    let inner_start = start + header.len();
    let mut depth = 1i32;
    for (off, c) in body[inner_start..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some((start, squash(&body[inner_start..inner_start + off])));
                }
            }
            _ => {}
        }
    }
    None
}

fn prepare(raw: &str) -> (String, Vec<Func>) {
    let normalised = normalise(raw);
    let code = production_code(&normalised);
    let funcs = functions(&code);
    (code, funcs)
}

// ---- the guards: each takes the RAW source and normalises it itself ----------

fn guard_get_clipboard_data_callers(raw: &str) {
    let (code, funcs) = prepare(raw);
    let mut callers = Vec::new();
    for f in &funcs {
        // `#[cfg(test)]` helpers are not production code.
        let test_only = f.attrs.iter().any(|a| a.contains("cfg(test)"));
        if f.body.contains("GetClipboardData(") && !test_only {
            callers.push(f.name.clone());
        }
    }
    callers.sort();
    assert_eq!(
        callers,
        vec!["capture", "clipboard_text_fp", "read_dword"],
        "GetClipboardData( may only be called from capture, read_dword and clipboard_text_fp"
    );
    // No hidden call site: every occurrence lies inside a collected function.
    let total = code.matches("GetClipboardData(").count();
    let inside: usize = funcs.iter().map(|f| f.body.matches("GetClipboardData(").count()).sum();
    assert_eq!(total, inside, "a GetClipboardData( call sits outside every function body");
    // The diagnostic reader is test-only (parsed attributes, not substrings).
    let read_text = find(&funcs, "read_text");
    assert!(
        read_text.attrs.iter().any(|a| a.contains("cfg(test)")),
        "read_text must stay #[cfg(test)] (it reads a clipboard without a probe)"
    );
}

fn guard_capture_probes_first(raw: &str) {
    let (_, funcs) = prepare(raw);
    let capture = find(&funcs, "capture");
    let (gate, inner) = block(&capture.body, "if !read_allowed() {", 0)
        .expect("capture must contain `if !read_allowed() {`");
    assert_eq!(inner, "return None;", "a refused read_allowed() must end the capture");
    let first_read = [capture.body.find("read_dword("), capture.body.find("GetClipboardData(")]
        .into_iter()
        .flatten()
        .min()
        .expect("capture reads data");
    assert!(gate < first_read, "capture must call read_allowed() before its first read");
    // read_dword( is only called from capture.
    let mut callers = Vec::new();
    for f in &funcs {
        let calls = f.body.matches("read_dword(").count();
        let declares = usize::from(f.name == "read_dword");
        if calls > declares {
            callers.push(f.name.clone());
        }
    }
    assert_eq!(callers, vec!["capture"], "read_dword( may only be called by capture");
}

fn guard_our_text_read_only_from_our_window(raw: &str) {
    let (_, funcs) = prepare(raw);
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
    let start = clear.body.find("let text_equal =").expect("clear_if_ours computes text_equal");
    let end = start + clear.body[start..].find(';').expect("statement end");
    let statement = squash(&clear.body[start..end]);
    assert!(
        statement.starts_with("let text_equal = owner_is_ours && "),
        "the read must start with `owner_is_ours && `: {statement}"
    );
    assert!(!statement.contains("||"), "no `||` may weaken the own-window condition: {statement}");
    assert!(statement.contains("clipboard_text_fp("));
    // owner_is_ours itself comes from the owner check before the statement.
    assert!(clear.body[..start].contains("let owner_is_ours = owner_is("));
}

fn guard_try_paste(raw: &str) {
    let (_, funcs) = prepare(raw);
    let paste = find(&funcs, "try_paste");
    let record = paste.body.find("shared.text =").expect("try_paste records our text");
    let write = paste.body.find("write_temp_text(").expect("try_paste writes the text");
    assert!(record < write, "shared.text must be assigned BEFORE write_temp_text(");
    assert_eq!(paste.body.matches("shared.text =").count(), 3, "recorded once, restored twice");
    assert_eq!(
        paste.body.matches("shared.text = previous_text;").count(),
        2,
        "when nothing was written the previous text comes back (both no-change arms)"
    );
    // The probe: after the open, before the write, exactly this statement.
    let open = paste.body.find("ClipboardGuard::open(").expect("try_paste opens the clipboard");
    let (probe, inner) = block(&paste.body, "if !clipboard_owner_responsive(20) {", open)
        .expect("try_paste must probe the current owner after opening the clipboard");
    assert!(probe < write, "the probe must come before the first write");
    assert_eq!(inner, "return false;", "an unanswered probe must end the paste (the guard closes)");
    assert!(
        paste.body.contains("failed_write_plan("),
        "try_paste must decide a failed write with the pure failed_write_plan"
    );
    let write_fn = find(&funcs, "write_temp_text");
    assert!(
        write_fn.body.contains("WriteFailure::NotEmptied") && write_fn.body.contains("WriteFailure::Emptied")
    );
}

fn guard_owner_lookup(raw: &str) {
    let (_, funcs) = prepare(raw);
    let mut callers = Vec::new();
    for f in &funcs {
        if f.body.contains("GetClipboardOwner(") {
            callers.push(f.name.clone());
        }
    }
    assert_eq!(
        callers,
        vec!["current_owner"],
        "GetClipboardOwner( may only be called from current_owner (no owner vs real error)"
    );
}

fn guard_parser_sees_expected_functions(raw: &str) {
    // Sanity: a refactor that renames these would silently disable the guards,
    // and impl methods / qualified fns must be collected too.
    let (_, funcs) = prepare(raw);
    for name in [
        "capture",
        "read_dword",
        "clipboard_text_fp",
        "clear_if_ours",
        "try_paste",
        "read_allowed",
        "write_temp_text",
        "read_text",
        "current_owner",
        "owner_is",
        "open",
        "drop",
        "worker_proc",
    ] {
        find(&funcs, name);
    }
}

fn all_guards(raw: &str) {
    guard_get_clipboard_data_callers(raw);
    guard_capture_probes_first(raw);
    guard_our_text_read_only_from_our_window(raw);
    guard_try_paste(raw);
    guard_owner_lookup(raw);
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
fn try_paste_records_probes_and_plans_in_the_right_order() {
    guard_try_paste(SOURCE);
}

#[test]
fn the_owner_is_only_looked_up_by_current_owner() {
    guard_owner_lookup(SOURCE);
}

#[test]
fn the_guard_parser_sees_the_expected_functions() {
    guard_parser_sees_expected_functions(SOURCE);
}

/// `core.autocrlf=true` checkouts: the same file with CRLF (or mixed) line
/// endings must give the same results.
#[test]
fn guards_hold_on_a_crlf_checkout() {
    let lf = normalise(SOURCE);
    assert!(!lf.contains('\r'), "normalise must leave no CR behind");
    let crlf = lf.replace('\n', "\r\n");
    // The conversion is complete: every LF is part of a CRLF.
    assert_eq!(crlf.matches("\r\n").count(), lf.matches('\n').count());
    assert!(!crlf.replace("\r\n", "").contains('\n'));
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
    let fails = |f: fn(&str), text: &str| std::panic::catch_unwind(|| f(text)).is_err();

    let m = mutate("#[cfg(test)]\nfn read_text", "fn read_text");
    assert!(fails(guard_get_clipboard_data_callers, &m), "read_text guard passed vacuously");
    let m = mutate(
        "    shared.text = OurText::of(text);\n",
        "",
    );
    assert!(fails(guard_try_paste, &m), "record-before-write passed vacuously");
    let m = mutate(
        "    if !clipboard_owner_responsive(20) {\n        return false;\n    }\n    // The snapshot moves into the plan",
        "    // The snapshot moves into the plan",
    );
    assert!(fails(guard_try_paste, &m), "probe guard passed vacuously");
    let m = mutate(
        "    if !clipboard_owner_responsive(20) {\n        return false;\n    }\n    // The snapshot moves into the plan",
        "    if !clipboard_owner_responsive(20) {\n        return true;\n    }\n    // The snapshot moves into the plan",
    );
    assert!(fails(guard_try_paste, &m), "probe usage (return value) passed vacuously");
    let m = mutate(
        "owner_is_ours && !marker && text.fp != 0 && clipboard_text_fp(text)",
        "owner_is_ours || (!marker && text.fp != 0 && clipboard_text_fp(text))",
    );
    assert!(fails(guard_our_text_read_only_from_our_window, &m), "own-window guard passed vacuously");
    let m = mutate("    if !read_allowed() {\n        return None;\n    }\n", "    if !read_allowed() {\n    }\n");
    assert!(fails(guard_capture_probes_first, &m), "capture gate usage passed vacuously");
}
