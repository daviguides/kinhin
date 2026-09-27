//! Python test-file surgery: locate pytest tests, read lifecycle markers,
//! insert markers, delete tests. Line-based on purpose — the file is
//! rewritten with every untouched line byte-identical.

use std::collections::{HashMap, HashSet};

use regex::Regex;

use crate::tags::LifecycleTag;

/// Line width used when the project does not configure one (PEP 8).
pub const DEFAULT_LINE_WIDTH: usize = 79;

#[derive(Debug, Clone, PartialEq)]
pub struct PyTest {
    pub name: String,
    /// `TestClass::test_name`, or `test_name` at module level.
    pub qualified: String,
    pub indent: usize,
    /// First line of the decorator block (equals `def_line` without decorators).
    pub block_start: usize,
    pub def_line: usize,
    /// One past the last line of the function body.
    pub end: usize,
    pub tag: LifecycleTag,
    pub ref_value: Option<String>,
}

/// What to write above one test.
#[derive(Debug, Clone)]
pub struct Marker {
    pub tag: LifecycleTag,
    /// Reason, party or ref, depending on the tag. Ignored for temporary tags.
    pub text: Option<String>,
}

fn indent_of(line: &str) -> usize {
    line.chars().take_while(|c| *c == ' ' || *c == '\t').count()
}

fn is_blank_or_comment(line: &str) -> bool {
    let t = line.trim();
    t.is_empty() || t.starts_with('#')
}

/// For each line: does it START in code (as opposed to inside a
/// triple-quoted string opened on an earlier line)?
fn code_mask(lines: &[&str]) -> Vec<bool> {
    let mut mask = Vec::with_capacity(lines.len());
    let mut open: Option<&'static str> = None;

    for line in lines {
        mask.push(open.is_none());
        let bytes = line.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            match open {
                Some(delim) => {
                    if bytes[i] == b'\\' {
                        i += 2;
                    } else if line[i..].starts_with(delim) {
                        open = None;
                        i += 3;
                    } else {
                        i += 1;
                    }
                }
                None => match bytes[i] {
                    b'#' => break,
                    b'"' | b'\'' => {
                        let quote = bytes[i];
                        if line[i..].starts_with("\"\"\"") {
                            open = Some("\"\"\"");
                            i += 3;
                        } else if line[i..].starts_with("'''") {
                            open = Some("'''");
                            i += 3;
                        } else {
                            i += 1;
                            while i < bytes.len() && bytes[i] != quote {
                                if bytes[i] == b'\\' {
                                    i += 1;
                                }
                                i += 1;
                            }
                            i += 1;
                        }
                    }
                    _ => i += 1,
                },
            }
        }
    }
    mask
}

/// Net bracket depth change of a line, ignoring string contents and comments.
fn bracket_delta(line: &str) -> i32 {
    let mut delta = 0;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '#' => break,
            '"' | '\'' => {
                while let Some(n) = chars.next() {
                    if n == '\\' {
                        chars.next();
                    } else if n == c {
                        break;
                    }
                }
            }
            '(' | '[' | '{' => delta += 1,
            ')' | ']' | '}' => delta -= 1,
            _ => {}
        }
    }
    delta
}

/// Locate every test pytest would collect with default naming rules:
/// `test*` functions at module level or directly inside `Test*` classes.
pub fn parse(content: &str) -> Vec<PyTest> {
    let lines: Vec<&str> = content.lines().collect();
    let mask = code_mask(&lines);
    let re_class = Regex::new(r"^\s*class\s+(\w+)").unwrap();
    let re_def = Regex::new(r"^\s*(?:async\s+)?def\s+(\w+)\s*\(").unwrap();

    // (indent, collects_tests, name)
    let mut scopes: Vec<(usize, bool, String)> = Vec::new();
    let mut tests = Vec::new();
    let mut depth = 0i32;

    for (i, line) in lines.iter().enumerate() {
        if !mask[i] || is_blank_or_comment(line) {
            continue;
        }
        let continuation = depth > 0;
        depth = (depth + bracket_delta(line)).max(0);
        if continuation {
            continue;
        }

        let indent = indent_of(line);
        while scopes.last().is_some_and(|(scope_indent, _, _)| *scope_indent >= indent) {
            scopes.pop();
        }

        if let Some(cap) = re_class.captures(line) {
            let name = cap[1].to_string();
            let collects = name.starts_with("Test");
            scopes.push((indent, collects, name));
        } else if let Some(cap) = re_def.captures(line) {
            let name = cap[1].to_string();
            let collected = name.starts_with("test") && scopes.iter().all(|(_, collects, _)| *collects);
            if collected {
                let mut parts: Vec<&str> = scopes.iter().map(|(_, _, n)| n.as_str()).collect();
                parts.push(&name);
                let block_start = decorator_block_start(&lines, i, indent);
                let (tag, ref_value) = read_lifecycle_marker(&lines[block_start..i]);
                tests.push(PyTest {
                    qualified: parts.join("::"),
                    name: name.clone(),
                    indent,
                    block_start,
                    def_line: i,
                    end: function_end(&lines, &mask, i, indent),
                    tag,
                    ref_value,
                });
            }
            scopes.push((indent, false, name));
        }
    }
    tests
}

fn decorator_block_start(lines: &[&str], def_line: usize, indent: usize) -> usize {
    let mut start = def_line;
    let mut pending = 0i32; // closers seen (walking upward) not yet matched by openers

    while start > 0 {
        let line = lines[start - 1];
        let trimmed = line.trim();
        if pending > 0 {
            pending -= bracket_delta(line);
            start -= 1;
            continue;
        }
        if indent_of(line) != indent {
            break;
        }
        if trimmed.starts_with('@') {
            start -= 1;
        } else if trimmed.starts_with([')', ']', '}']) {
            pending = -bracket_delta(line);
            if pending <= 0 {
                break;
            }
            start -= 1;
        } else {
            break;
        }
    }
    start
}

fn function_end(lines: &[&str], mask: &[bool], def_line: usize, indent: usize) -> usize {
    let mut depth = 0;
    let mut signature_end = def_line;
    for (k, line) in lines.iter().enumerate().skip(def_line) {
        depth += bracket_delta(line);
        signature_end = k;
        if depth <= 0 {
            break;
        }
    }

    let mut end = signature_end + 1;
    let mut k = signature_end + 1;
    while k < lines.len() {
        if !mask[k] {
            end = k + 1;
        } else if lines[k].trim().is_empty() {
            // undecided: belongs to the body only if more body follows
        } else if indent_of(lines[k]) > indent {
            end = k + 1;
        } else {
            break;
        }
        k += 1;
    }
    end
}

fn read_lifecycle_marker(block: &[&str]) -> (LifecycleTag, Option<String>) {
    let text = block.join("\n");
    let re = Regex::new(r"@pytest\.mark\.(scaffold|characterization|decision|contract|incident)\b").unwrap();
    let Some(m) = re.captures(&text) else {
        return (LifecycleTag::Untagged, None);
    };
    let tag = match &m[1] {
        "scaffold" => LifecycleTag::Scaffold,
        "characterization" => LifecycleTag::Characterization,
        "decision" => LifecycleTag::Decision,
        "contract" => LifecycleTag::Contract,
        _ => LifecycleTag::Incident,
    };
    if tag.is_temporary() {
        return (tag, None);
    }
    let rest = &text[m.get(0).unwrap().end()..];
    let literal = string_literals_in_call(rest);
    (tag, (!literal.is_empty()).then_some(literal))
}

/// Concatenate the string literals inside the call that starts `rest`
/// (`(reason="a" "b")` → `ab`). Empty when `rest` is not a call.
fn string_literals_in_call(rest: &str) -> String {
    let mut out = String::new();
    let mut chars = rest.chars().peekable();
    if chars.peek() != Some(&'(') {
        return out;
    }
    let mut depth = 0;
    while let Some(c) = chars.next() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            '"' | '\'' => {
                while let Some(n) = chars.next() {
                    if n == '\\' {
                        if let Some(escaped) = chars.next() {
                            out.push(escaped);
                        }
                    } else if n == c {
                        break;
                    } else {
                        out.push(n);
                    }
                }
            }
            _ => {}
        }
    }
    out
}

fn escape_python_string(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            c if c.is_control() => out.push(' '),
            c => out.push(c),
        }
    }
    out.trim().to_string()
}

/// Source lines for one lifecycle marker at `indent`, wrapped to `width`
/// columns the way ruff/black lay out a long keyword argument.
pub fn format_marker(marker: &Marker, indent: usize, width: usize) -> Vec<String> {
    let pad = " ".repeat(indent);
    let keyword = match marker.tag {
        LifecycleTag::Scaffold => return vec![format!("{pad}@pytest.mark.scaffold")],
        LifecycleTag::Characterization => return vec![format!("{pad}@pytest.mark.characterization")],
        LifecycleTag::Untagged => return Vec::new(),
        LifecycleTag::Decision => "reason",
        LifecycleTag::Contract => "party",
        LifecycleTag::Incident => "ref",
    };
    let name = marker.tag.to_string();
    let escaped = escape_python_string(marker.text.as_deref().unwrap_or("unspecified"));

    let single = format!("{pad}@pytest.mark.{name}({keyword}=\"{escaped}\")");
    if single.chars().count() <= width {
        return vec![single];
    }

    let two_level = format!("{pad}    {keyword}=\"{escaped}\",");
    if two_level.chars().count() <= width {
        return vec![format!("{pad}@pytest.mark.{name}("), two_level, format!("{pad})")];
    }

    // Each chunk sits at indent + 8 between two quotes. A formatter joins
    // chunks that fit on one line, so never split what fits.
    let budget = width.saturating_sub(indent + 8 + 2).max(20);
    let mut chunks: Vec<String> = Vec::new();
    let mut rest = escaped.as_str();
    while rest.chars().count() > budget {
        let window: String = rest.chars().take(budget).collect();
        let Some(cut) = window.rfind(' ') else {
            break; // one unbreakable word longer than the line
        };
        chunks.push(rest[..=cut].to_string());
        rest = &rest[cut + 1..];
    }
    chunks.push(rest.to_string());

    let mut out = vec![format!("{pad}@pytest.mark.{name}("), format!("{pad}    {keyword}=(")];
    out.extend(chunks.iter().map(|chunk| format!("{pad}        \"{chunk}\"")));
    out.push(format!("{pad}    ),"));
    out.push(format!("{pad})"));
    out
}

fn binds_pytest_name(content: &str) -> bool {
    Regex::new(r"(?m)^import\s+(?:[\w.]+\s*,\s*)*pytest\s*(?:,|#|$)")
        .unwrap()
        .is_match(content)
}

/// Line index where `import pytest` belongs: right after the last statement
/// of the leading import block, else after the module docstring, else 0.
fn import_insertion_line(lines: &[&str], mask: &[bool]) -> usize {
    let mut after_imports = None;
    let mut depth = 0i32;
    let mut in_import = false;
    let mut after_docstring = 0;
    let mut seen_code = false;

    for (i, line) in lines.iter().enumerate() {
        if !mask[i] {
            if !seen_code {
                after_docstring = i + 1;
            }
            continue;
        }
        let continuing = depth > 0 || in_import && i > 0 && lines[i - 1].trim_end().ends_with('\\');
        if continuing {
            depth = (depth + bracket_delta(line)).max(0);
            if in_import && depth == 0 && !line.trim_end().ends_with('\\') {
                after_imports = Some(i + 1);
                in_import = false;
            }
            continue;
        }
        if is_blank_or_comment(line) {
            continue;
        }
        let trimmed = line.trim_start();
        let top_level = indent_of(line) == 0;
        if !seen_code && top_level && (trimmed.starts_with("\"\"\"") || trimmed.starts_with("'''")) {
            after_docstring = i + 1;
            continue;
        }
        seen_code = true;
        if top_level && (trimmed.starts_with("import ") || trimmed.starts_with("from ")) {
            depth = bracket_delta(line).max(0);
            if depth == 0 && !line.trim_end().ends_with('\\') {
                after_imports = Some(i + 1);
            } else {
                in_import = true;
            }
        } else if top_level {
            break; // first non-import statement ends the leading import block
        }
    }
    after_imports.unwrap_or(after_docstring)
}

pub struct InsertOutcome {
    pub content: String,
    pub inserted: usize,
    pub added_import: bool,
}

/// Write a lifecycle marker above every currently-untagged test that has an
/// entry in `markers` (keyed by qualified name). Already-tagged tests are
/// left alone, which makes the operation idempotent.
pub fn insert_markers(content: &str, markers: &HashMap<String, Marker>, width: usize) -> InsertOutcome {
    let mut lines: Vec<String> = content.lines().map(String::from).collect();
    let tests = parse(content);

    let mut inserted = 0;
    for test in tests.iter().rev() {
        if test.tag != LifecycleTag::Untagged {
            continue;
        }
        let Some(marker) = markers.get(&test.qualified) else {
            continue;
        };
        let marker_lines = format_marker(marker, test.indent, width);
        if marker_lines.is_empty() {
            continue;
        }
        lines.splice(test.block_start..test.block_start, marker_lines);
        inserted += 1;
    }

    let mut added_import = false;
    if inserted > 0 && !binds_pytest_name(content) {
        let borrowed: Vec<&str> = lines.iter().map(String::as_str).collect();
        let mask = code_mask(&borrowed);
        let at = import_insertion_line(&borrowed, &mask);
        let mut block = vec!["import pytest".to_string()];
        let previous_is_import = at > 0
            && {
                let prev = lines[at - 1].trim_start();
                prev.starts_with("import ") || prev.starts_with("from ") || prev.starts_with(')')
            };
        if at > 0 && !previous_is_import && !lines[at - 1].trim().is_empty() {
            block.insert(0, String::new());
        }
        if lines.get(at).is_some_and(|next| !next.trim().is_empty()) && !previous_is_import {
            block.push(String::new());
        }
        lines.splice(at..at, block);
        added_import = true;
    }

    InsertOutcome {
        content: join_lines(&lines, content),
        inserted,
        added_import,
    }
}

pub struct DeleteOutcome {
    pub content: String,
    pub removed: Vec<String>,
    /// No collectable test is left in the file.
    pub empty: bool,
}

/// Remove the named tests (decorators and body included), then any class
/// left without a body.
pub fn delete_tests(content: &str, qualified: &HashSet<String>) -> DeleteOutcome {
    let mut lines: Vec<String> = content.lines().map(String::from).collect();
    let tests = parse(content);
    let mut removed = Vec::new();

    for test in tests.iter().rev() {
        if !qualified.contains(&test.qualified) {
            continue;
        }
        remove_block(&mut lines, test.block_start, test.end, test.indent);
        removed.push(test.qualified.clone());
    }
    removed.reverse();

    while remove_one_empty_class(&mut lines) {}
    collapse_blank_runs(&mut lines);

    let new_content = join_lines(&lines, content);
    let empty = parse(&new_content).is_empty();
    DeleteOutcome {
        content: new_content,
        removed,
        empty,
    }
}

fn remove_block(lines: &mut Vec<String>, block_start: usize, end: usize, indent: usize) {
    let mut start = block_start;
    // comments glued to the top of the block describe the test: they go too
    while start > 0 && lines[start - 1].trim_start().starts_with('#') && indent_of(&lines[start - 1]) == indent {
        start -= 1;
    }
    let mut gap = start;
    while gap > 0 && lines[gap - 1].trim().is_empty() {
        gap -= 1;
    }
    let first_in_scope = gap == 0 || lines[gap - 1].trim_end().ends_with(':');

    let mut stop = end.min(lines.len());
    if first_in_scope {
        // nothing precedes it in its scope: drop the blank lines that follow
        while stop < lines.len() && lines[stop].trim().is_empty() {
            stop += 1;
        }
    } else {
        start = gap; // drop the separator that preceded it
    }
    lines.drain(start..stop);
}

fn remove_one_empty_class(lines: &mut Vec<String>) -> bool {
    let re_class = Regex::new(r"^\s*class\s+\w+").unwrap();
    let borrowed: Vec<&str> = lines.iter().map(String::as_str).collect();
    let mask = code_mask(&borrowed);

    for (i, line) in borrowed.iter().enumerate() {
        if !mask[i] || !re_class.is_match(line) {
            continue;
        }
        let indent = indent_of(line);
        let mut header_end = i;
        let mut depth = 0;
        for (k, l) in borrowed.iter().enumerate().skip(i) {
            depth += bracket_delta(l);
            header_end = k;
            if depth <= 0 {
                break;
            }
        }
        if !borrowed[header_end].trim_end().ends_with(':') {
            continue; // one-line class body
        }

        let mut end = header_end + 1;
        let mut has_statement = false;
        let mut k = header_end + 1;
        let mut first_statement = true;
        while k < borrowed.len() {
            let l = borrowed[k];
            if !mask[k] {
                end = k + 1; // inside the docstring
            } else if l.trim().is_empty() {
            } else if indent_of(l) > indent {
                end = k + 1;
                let t = l.trim();
                let docstring = first_statement && (t.starts_with("\"\"\"") || t.starts_with("'''"));
                if !(docstring || t == "pass" || t == "..." || t.starts_with('#')) {
                    has_statement = true;
                    break;
                }
                if !t.starts_with('#') {
                    first_statement = false;
                }
            } else {
                break;
            }
            k += 1;
        }
        if has_statement {
            continue;
        }
        let block_start = decorator_block_start(&borrowed, i, indent);
        remove_block(lines, block_start, end, indent);
        return true;
    }
    false
}

fn collapse_blank_runs(lines: &mut Vec<String>) {
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut blanks = 0;
    for line in lines.drain(..) {
        if line.trim().is_empty() {
            blanks += 1;
            if blanks > 2 {
                continue;
            }
        } else {
            blanks = 0;
        }
        out.push(line);
    }
    while out.last().is_some_and(|l| l.trim().is_empty()) {
        out.pop();
    }
    *lines = out;
}

fn join_lines(lines: &[String], original: &str) -> String {
    let mut out = lines.join("\n");
    if original.ends_with('\n') || !out.is_empty() {
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn marker(tag: LifecycleTag, text: &str) -> Marker {
        Marker {
            tag,
            text: Some(text.to_string()),
        }
    }

    fn markers(entries: &[(&str, Marker)]) -> HashMap<String, Marker> {
        entries.iter().map(|(k, m)| ((*k).to_string(), m.clone())).collect()
    }

    const TWO_CLASSES: &str = "\
import pytest


class TestA:
    def test_same(self):
        assert 1

    async def test_async(self):
        assert 2


class TestB:
    @pytest.mark.parametrize(
        \"x\",
        [1, 2],
    )
    def test_same(self, x):
        assert x


def test_module_level():
    def test_nested_helper():
        pass
    assert True
";

    #[test]
    fn same_name_in_two_classes_gets_distinct_qualified_names() {
        let names: Vec<String> = parse(TWO_CLASSES).into_iter().map(|t| t.qualified).collect();
        assert_eq!(
            names,
            ["TestA::test_same", "TestA::test_async", "TestB::test_same", "test_module_level"]
        );
    }

    #[test]
    fn multi_line_decorator_belongs_to_the_block() {
        let tests = parse(TWO_CLASSES);
        let b = tests.iter().find(|t| t.qualified == "TestB::test_same").unwrap();
        assert_eq!(b.def_line - b.block_start, 4);
    }

    #[test]
    fn marker_of_previous_test_does_not_bleed_into_the_next() {
        let src = "import pytest\n\n@pytest.mark.scaffold\ndef test_a():\n    pass\n\ndef test_b():\n    pass\n";
        let tests = parse(src);
        assert_eq!(tests[0].tag, LifecycleTag::Scaffold);
        assert_eq!(tests[1].tag, LifecycleTag::Untagged);
    }

    #[test]
    fn test_inside_triple_quoted_string_is_not_a_test() {
        let src = "DOC = \"\"\"\ndef test_fake():\n    pass\n\"\"\"\n\ndef test_real():\n    pass\n";
        let names: Vec<String> = parse(src).into_iter().map(|t| t.qualified).collect();
        assert_eq!(names, ["test_real"]);
    }

    #[test]
    fn inserts_distinct_markers_for_same_named_tests() {
        let out = insert_markers(
            TWO_CLASSES,
            &markers(&[
                ("TestA::test_same", marker(LifecycleTag::Scaffold, "")),
                ("TestB::test_same", marker(LifecycleTag::Decision, "x is positive")),
            ]),
            DEFAULT_LINE_WIDTH,
        );
        assert_eq!(out.inserted, 2);
        let tests = parse(&out.content);
        assert_eq!(tests[0].tag, LifecycleTag::Scaffold);
        assert_eq!(tests[2].tag, LifecycleTag::Decision);
        assert_eq!(tests[2].ref_value.as_deref(), Some("x is positive"));
        assert_eq!(tests[1].tag, LifecycleTag::Untagged);
    }

    #[test]
    fn insertion_is_idempotent() {
        let all = markers(&[
            ("TestA::test_same", marker(LifecycleTag::Scaffold, "")),
            ("TestA::test_async", marker(LifecycleTag::Contract, "wire format")),
            ("TestB::test_same", marker(LifecycleTag::Decision, "x is positive")),
            ("test_module_level", marker(LifecycleTag::Incident, "ISSUE-1")),
        ]);
        let once = insert_markers(TWO_CLASSES, &all, DEFAULT_LINE_WIDTH);
        let twice = insert_markers(&once.content, &all, DEFAULT_LINE_WIDTH);
        assert_eq!(once.inserted, 4);
        assert_eq!(twice.inserted, 0);
        assert_eq!(once.content, twice.content);
    }

    #[test]
    fn reason_with_quotes_and_backslashes_round_trips() {
        let reason = r#"path "C:\tmp" must not be 'normalised'"#;
        let src = "import pytest\n\ndef test_x():\n    pass\n";
        let out = insert_markers(src, &markers(&[("test_x", marker(LifecycleTag::Decision, reason))]), DEFAULT_LINE_WIDTH);
        assert_eq!(parse(&out.content)[0].ref_value.as_deref(), Some(reason));
    }

    #[test]
    fn long_reason_wraps_within_width_and_round_trips() {
        let reason = "encodes the unconditional backup invariant: a backup is written even when zero records were imported from the source file";
        let src = "import pytest\n\nclass TestX:\n    def test_x(self):\n        pass\n";
        let out = insert_markers(src, &markers(&[("TestX::test_x", marker(LifecycleTag::Decision, reason))]), DEFAULT_LINE_WIDTH);
        assert!(out.content.lines().all(|l| l.chars().count() <= DEFAULT_LINE_WIDTH), "{}", out.content);
        assert_eq!(parse(&out.content)[0].ref_value.as_deref(), Some(reason));
    }

    #[test]
    fn text_that_fits_between_the_parentheses_is_not_split() {
        // 66 characters: too long beside `reason=`, fits alone at indent 12 in 80 columns.
        let reason = "Pins rule that URL fragments are stripped during canonicalization.";
        let lines = format_marker(&marker(LifecycleTag::Decision, reason), 4, 80);
        assert_eq!(lines.len(), 5, "{lines:#?}");
        assert_eq!(lines[2], format!("            \"{reason}\""));
    }

    #[test]
    fn import_pytest_goes_after_multi_line_import_not_inside_it() {
        let src = "\"\"\"Doc.\"\"\"\n\nfrom pathlib import Path\n\nfrom marks import (\n    a,\n    b,\n)\n\n\ndef test_x():\n    pass\n";
        let out = insert_markers(src, &markers(&[("test_x", marker(LifecycleTag::Scaffold, ""))]), DEFAULT_LINE_WIDTH);
        assert!(out.added_import);
        let lines: Vec<&str> = out.content.lines().collect();
        let close = lines.iter().position(|l| *l == ")").unwrap();
        assert_eq!(lines[close + 1], "import pytest");
    }

    #[test]
    fn import_pytest_after_docstring_when_file_has_no_imports() {
        let src = "\"\"\"Doc\nspanning lines.\n\"\"\"\n\ndef test_x():\n    pass\n";
        let out = insert_markers(src, &markers(&[("test_x", marker(LifecycleTag::Scaffold, ""))]), DEFAULT_LINE_WIDTH);
        let lines: Vec<&str> = out.content.lines().collect();
        assert_eq!(&lines[..5], ["\"\"\"Doc", "spanning lines.", "\"\"\"", "", "import pytest"]);
    }

    #[test]
    fn existing_import_pytest_is_not_duplicated() {
        let out = insert_markers(TWO_CLASSES, &markers(&[("test_module_level", marker(LifecycleTag::Scaffold, ""))]), DEFAULT_LINE_WIDTH);
        assert!(!out.added_import);
        assert_eq!(out.content.matches("import pytest").count(), 1);
    }

    #[test]
    fn from_pytest_import_does_not_count_as_binding_pytest() {
        assert!(!binds_pytest_name("from pytest import mark\n"));
        assert!(binds_pytest_name("import os, pytest\n"));
    }

    #[test]
    fn deleting_a_test_removes_decorators_and_body_only() {
        let set: HashSet<String> = ["TestB::test_same".to_string()].into();
        let out = delete_tests(TWO_CLASSES, &set);
        assert_eq!(out.removed, ["TestB::test_same"]);
        assert!(!out.content.contains("class TestB"), "empty class must go:\n{}", out.content);
        assert!(!out.content.contains("parametrize"));
        let names: Vec<String> = parse(&out.content).into_iter().map(|t| t.qualified).collect();
        assert_eq!(names, ["TestA::test_same", "TestA::test_async", "test_module_level"]);
        assert!(!out.empty);
    }

    #[test]
    fn deleting_every_test_reports_empty_file() {
        let src = "import pytest\n\n\n@pytest.mark.scaffold\ndef test_a():\n    x = \"\"\"\nzero-indent text\n\"\"\"\n    assert x\n";
        let set: HashSet<String> = ["test_a".to_string()].into();
        let out = delete_tests(src, &set);
        assert!(out.empty);
        assert!(!out.content.contains("zero-indent text"));
    }

    #[test]
    fn deleting_a_middle_method_keeps_one_separator() {
        let src = "class TestA:\n    def test_a(self):\n        pass\n\n    def test_b(self):\n        pass\n\n    def test_c(self):\n        pass\n";
        let set: HashSet<String> = ["TestA::test_b".to_string()].into();
        let out = delete_tests(src, &set);
        assert_eq!(
            out.content,
            "class TestA:\n    def test_a(self):\n        pass\n\n    def test_c(self):\n        pass\n"
        );
    }
}
