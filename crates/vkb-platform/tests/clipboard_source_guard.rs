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

const SOURCE: &str = include_str!("../../vkb-windows/src/clipboard.rs");

/// The production part of the file (the `#[cfg(test)] mod tests` is cut off),
/// without `//` and `///` comment lines.
fn production_code() -> String {
    let end = SOURCE.find("#[cfg(test)]\nmod tests").unwrap_or(SOURCE.len());
    strip_comments(&SOURCE[..end])
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

#[test]
fn get_clipboard_data_is_called_only_from_the_gated_readers() {
    let code = production_code();
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
    // The diagnostic reader is test-only.
    assert!(
        SOURCE.contains("#[cfg(test)]\nfn read_text"),
        "read_text must stay #[cfg(test)] (it reads a clipboard without a probe)"
    );
}

#[test]
fn capture_probes_the_owner_before_its_first_read() {
    let code = production_code();
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

#[test]
fn our_text_is_only_read_from_our_own_window() {
    let code = production_code();
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

#[test]
fn try_paste_records_our_text_before_it_writes() {
    let code = production_code();
    let funcs = functions(&code);
    let paste = find(&funcs, "try_paste");
    let record = paste.body.find("shared.text =").expect("try_paste records our text");
    let write = paste.body.find("write_temp_text(").expect("try_paste writes the text");
    assert!(record < write, "shared.text must be assigned BEFORE write_temp_text(");
    assert_eq!(paste.body.matches("shared.text =").count(), 1, "recorded exactly once");
}

#[test]
fn the_guard_parser_sees_the_expected_functions() {
    // Sanity: a refactor that renames these would silently disable the guards.
    let code = production_code();
    let funcs = functions(&code);
    for name in ["capture", "read_dword", "clipboard_text_fp", "clear_if_ours", "try_paste", "read_allowed"] {
        find(&funcs, name);
    }
    assert!(funcs.iter().all(|f| f.attrs.len() < 8));
}
