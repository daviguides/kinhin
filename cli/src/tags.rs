use std::path::{Path, PathBuf};

use regex::Regex;
use walkdir::WalkDir;

use crate::detect::Language;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub enum LifecycleTag {
    Scaffold,
    Characterization,
    Decision,
    Contract,
    Incident,
    Untagged,
}

impl LifecycleTag {
    pub fn is_permanent(&self) -> bool {
        matches!(self, Self::Decision | Self::Contract | Self::Incident)
    }

    pub fn is_temporary(&self) -> bool {
        matches!(self, Self::Scaffold | Self::Characterization)
    }
}

impl std::fmt::Display for LifecycleTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Scaffold => write!(f, "scaffold"),
            Self::Characterization => write!(f, "characterization"),
            Self::Decision => write!(f, "decision"),
            Self::Contract => write!(f, "contract"),
            Self::Incident => write!(f, "incident"),
            Self::Untagged => write!(f, "untagged"),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TaggedTest {
    pub file: PathBuf,
    /// Display name. For Python this is the qualified `Class::test` form,
    /// which is unique within the file.
    pub name: Option<String>,
    pub tag: LifecycleTag,
    pub ref_value: Option<String>,
}

const SKIP_DIRS: &[&str] = &[
    "node_modules",
    ".venv",
    "target",
    ".git",
    "__pycache__",
    "dist",
    "build",
    ".claude",
    ".kinhin",
    "mutants",
    ".mutmut-cache",
    ".stryker-tmp",
    ".tox",
    "site-packages",
];

fn should_skip(entry: &walkdir::DirEntry) -> bool {
    entry.file_type().is_dir()
        && entry
            .file_name()
            .to_str()
            .is_some_and(|name| SKIP_DIRS.contains(&name))
}

pub fn scan_test_files(root: &str, languages: &[Language]) -> Vec<TaggedTest> {
    let mut results = Vec::new();

    for entry in WalkDir::new(root)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|e| !should_skip(e))
        .flatten()
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        for lang in languages {
            if is_test_file(name, path, *lang) {
                let Ok(content) = std::fs::read_to_string(path) else {
                    continue;
                };
                let mut tests = parse_tags(path, &content, *lang);
                results.append(&mut tests);
                break;
            }
        }
    }

    results
}

pub fn is_test_file(name: &str, _path: &Path, lang: Language) -> bool {
    match lang {
        Language::Python => {
            name.starts_with("test_") && name.ends_with(".py")
                || name.ends_with("_test.py")
        }
        Language::Rust => name.ends_with(".rs"),
        Language::Java => name.ends_with("Test.java") || name.starts_with("Test"),
        Language::TypeScript => {
            name.ends_with(".test.ts")
                || name.ends_with(".test.tsx")
                || name.ends_with(".spec.ts")
                || name.ends_with(".spec.tsx")
        }
    }
}

fn parse_tags(path: &Path, content: &str, lang: Language) -> Vec<TaggedTest> {
    match lang {
        Language::Python => parse_python_tags(path, content),
        Language::Rust => parse_rust_tags(path, content),
        Language::Java => parse_java_tags(path, content),
        Language::TypeScript => parse_typescript_tags(path, content),
    }
}

fn parse_python_tags(path: &Path, content: &str) -> Vec<TaggedTest> {
    crate::pytests::parse(content)
        .into_iter()
        .map(|test| TaggedTest {
            file: path.to_path_buf(),
            name: Some(test.qualified),
            tag: test.tag,
            ref_value: test.ref_value,
        })
        .collect()
}

/// The contiguous run of lines directly above `index` that satisfy `belongs`
/// (annotations, attributes, comments). A blank line or code ends the run,
/// so a tag never leaks from one test to the next.
fn block_above(lines: &[&str], index: usize, belongs: impl Fn(&str) -> bool) -> String {
    let mut start = index;
    while start > 0 && belongs(lines[start - 1].trim()) {
        start -= 1;
    }
    lines[start..index].join("\n")
}

fn parse_rust_tags(path: &Path, content: &str) -> Vec<TaggedTest> {
    let mut results = Vec::new();

    if !content.contains("#[cfg(test)]") && !content.contains("#[test]") {
        return results;
    }

    let re_test = Regex::new(r"(?m)^\s*(?:#\[test\]|#\[rstest\])").unwrap();
    let re_fn = Regex::new(r"(?m)^\s*fn (\w+)").unwrap();
    let re_kinhin = Regex::new(r#"// kinhin: (\w+)\(ref=["']([^"']+)["']\)"#).unwrap();

    let scaffold_ranges = find_mod_scaffold_ranges(content);
    let lines: Vec<&str> = content.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        if !re_test.is_match(line) {
            continue;
        }
        // find the fn name on the next few lines
        let search_end = (i + 3).min(lines.len());
        let following = lines[i..search_end].join("\n");
        let Some(fn_cap) = re_fn.captures(&following) else {
            continue;
        };
        let func_name = fn_cap[1].to_string();

        let byte_offset = lines[..i].iter().map(|l| l.len() + 1).sum::<usize>();
        let in_scaffold_mod = scaffold_ranges
            .iter()
            .any(|(start, end)| byte_offset >= *start && byte_offset < *end);

        let context = block_above(&lines, i, |l| l.starts_with("//") || l.starts_with("#["));

        let (tag, ref_value) = if in_scaffold_mod {
            (LifecycleTag::Scaffold, None)
        } else if let Some(cap) = re_kinhin.captures(&context) {
            let kind = &cap[1];
            let reference = cap[2].to_string();
            let tag = match kind {
                "decision" => LifecycleTag::Decision,
                "contract" => LifecycleTag::Contract,
                "incident" => LifecycleTag::Incident,
                _ => LifecycleTag::Untagged,
            };
            (tag, Some(reference))
        } else {
            (LifecycleTag::Untagged, None)
        };

        results.push(TaggedTest {
            file: path.to_path_buf(),
            name: Some(func_name),
            tag,
            ref_value,
        });
    }

    results
}

fn find_mod_scaffold_ranges(content: &str) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let re = Regex::new(r"mod scaffold\s*\{").unwrap();

    for m in re.find_iter(content) {
        let start = m.start();
        let mut depth = 0;
        let mut end = start;

        for (i, ch) in content[start..].char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = start + i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }

        ranges.push((start, end));
    }

    ranges
}

fn parse_java_tags(path: &Path, content: &str) -> Vec<TaggedTest> {
    let mut results = Vec::new();
    let re_test = Regex::new(r"(?m)^\s*@Test").unwrap();
    let re_method = Regex::new(r"(?m)^\s*(?:public |private |protected )?void (\w+)\s*\(").unwrap();
    let re_tag = Regex::new(r#"@Tag\("(\w+)"\)"#).unwrap();
    let re_display = Regex::new(r#"@DisplayName\("([^"]+)"\)"#).unwrap();

    let lines: Vec<&str> = content.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        if !re_test.is_match(line) {
            continue;
        }
        let search_end = (i + 3).min(lines.len());
        let following = lines[i..search_end].join("\n");
        let Some(method_cap) = re_method.captures(&following) else {
            continue;
        };
        let method_name = method_cap[1].to_string();

        // Annotations may sit above or below `@Test`: take the whole run that ends at the method.
        let method_line = (i..search_end)
            .find(|k| re_method.is_match(lines[*k]))
            .unwrap_or(i);
        let context = block_above(&lines, method_line, |l| l.starts_with('@') || l.starts_with("//"));

        let (tag, ref_value) = if let Some(tag_cap) = re_tag.captures(&context) {
            let kind = &tag_cap[1];
            let display_name = re_display
                .captures(&context)
                .map(|c| c[1].to_string());

            let tag = match kind {
                "scaffold" => LifecycleTag::Scaffold,
                "characterization" => LifecycleTag::Characterization,
                "decision" => LifecycleTag::Decision,
                "contract" => LifecycleTag::Contract,
                "incident" => LifecycleTag::Incident,
                _ => LifecycleTag::Untagged,
            };
            (tag, display_name)
        } else {
            (LifecycleTag::Untagged, None)
        };

        results.push(TaggedTest {
            file: path.to_path_buf(),
            name: Some(method_name),
            tag,
            ref_value,
        });
    }

    results
}

fn parse_typescript_tags(path: &Path, content: &str) -> Vec<TaggedTest> {
    let mut results = Vec::new();
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    let is_scaffold_file = file_name.contains(".scaffold.");

    let re_it = Regex::new(r#"(?m)^\s*(?:it|test)\s*\(\s*['"](.*?)['"]\s*,"#).unwrap();
    let re_kinhin = Regex::new(r#"// @(decision|contract|incident)\(ref=["']([^"']+)["']\)"#).unwrap();

    let lines: Vec<&str> = content.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        if let Some(cap) = re_it.captures(line) {
            let test_name = cap[1].to_string();

            if is_scaffold_file {
                results.push(TaggedTest {
                    file: path.to_path_buf(),
                    name: Some(test_name),
                    tag: LifecycleTag::Scaffold,
                    ref_value: None,
                });
                continue;
            }

            let context = block_above(&lines, i, |l| l.starts_with("//"));

            let (tag, ref_value) = if let Some(kcap) = re_kinhin.captures(&context) {
                let kind = &kcap[1];
                let reference = kcap[2].to_string();
                let tag = match kind {
                    "decision" => LifecycleTag::Decision,
                    "contract" => LifecycleTag::Contract,
                    "incident" => LifecycleTag::Incident,
                    _ => LifecycleTag::Untagged,
                };
                (tag, Some(reference))
            } else {
                (LifecycleTag::Untagged, None)
            };

            results.push(TaggedTest {
                file: path.to_path_buf(),
                name: Some(test_name),
                tag,
                ref_value,
            });
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tags_of(tests: &[TaggedTest]) -> Vec<(String, LifecycleTag, Option<String>)> {
        tests
            .iter()
            .map(|t| (t.name.clone().unwrap_or_default(), t.tag.clone(), t.ref_value.clone()))
            .collect()
    }

    #[test]
    fn python_reads_every_lifecycle_marker_with_its_text() {
        let src = r#"
import pytest

@pytest.mark.scaffold
def test_a(): pass

@pytest.mark.decision(reason="discount never negative")
def test_b(): pass

@pytest.mark.contract(party="iOS v2")
def test_c(): pass

@pytest.mark.incident(ref="ISSUE-186")
def test_d(): pass

@pytest.mark.characterization
def test_e(): pass

@pytest.mark.slow
def test_f(): pass
"#;
        assert_eq!(
            tags_of(&parse_python_tags(Path::new("test_x.py"), src)),
            [
                ("test_a".into(), LifecycleTag::Scaffold, None),
                ("test_b".into(), LifecycleTag::Decision, Some("discount never negative".into())),
                ("test_c".into(), LifecycleTag::Contract, Some("iOS v2".into())),
                ("test_d".into(), LifecycleTag::Incident, Some("ISSUE-186".into())),
                ("test_e".into(), LifecycleTag::Characterization, None),
                ("test_f".into(), LifecycleTag::Untagged, None),
            ]
        );
    }

    #[test]
    fn rust_scaffold_module_and_kinhin_comments() {
        let src = r#"
#[cfg(test)]
mod tests {
    use super::*;

    // kinhin: decision(ref="docs/fees.md#grace")
    #[test]
    fn grace_uses_higher_set() {}

    #[test]
    fn plain() {}

    mod scaffold {
        use super::*;

        #[test]
        fn parser_returns_keys() {}
    }
}
"#;
        assert_eq!(
            tags_of(&parse_rust_tags(Path::new("lib.rs"), src)),
            [
                ("grace_uses_higher_set".into(), LifecycleTag::Decision, Some("docs/fees.md#grace".into())),
                ("plain".into(), LifecycleTag::Untagged, None),
                ("parser_returns_keys".into(), LifecycleTag::Scaffold, None),
            ]
        );
    }

    #[test]
    fn java_tag_annotations_with_display_name_as_ref() {
        let src = r#"
class FooTest {
    @Tag("scaffold")
    @Test
    void parserReturnsKeys() {}

    @Tag("incident")
    @DisplayName("TICKET-186: scan walked the keyspace")
    @Test
    void neverScans() {}

    @Test
    void untagged() {}
}
"#;
        assert_eq!(
            tags_of(&parse_java_tags(Path::new("FooTest.java"), src)),
            [
                ("parserReturnsKeys".into(), LifecycleTag::Scaffold, None),
                (
                    "neverScans".into(),
                    LifecycleTag::Incident,
                    Some("TICKET-186: scan walked the keyspace".into())
                ),
                ("untagged".into(), LifecycleTag::Untagged, None),
            ]
        );
    }

    #[test]
    fn typescript_scaffold_by_file_suffix_and_comment_tags() {
        let src = "// @contract(ref=\"API v2\")\nit('keeps the wire shape', () => {});\n\ntest('plain', () => {});\n";
        assert_eq!(
            tags_of(&parse_typescript_tags(Path::new("a.test.ts"), src)),
            [
                ("keeps the wire shape".into(), LifecycleTag::Contract, Some("API v2".into())),
                ("plain".into(), LifecycleTag::Untagged, None),
            ]
        );
        let scaffold = parse_typescript_tags(Path::new("a.scaffold.test.ts"), src);
        assert!(scaffold.iter().all(|t| t.tag == LifecycleTag::Scaffold));
    }

    #[test]
    fn test_file_detection_per_language() {
        let p = Path::new("x");
        assert!(is_test_file("test_a.py", p, Language::Python));
        assert!(is_test_file("a_test.py", p, Language::Python));
        assert!(!is_test_file("conftest.py", p, Language::Python));
        assert!(is_test_file("a.spec.tsx", p, Language::TypeScript));
        assert!(!is_test_file("a.ts", p, Language::TypeScript));
        assert!(is_test_file("FooTest.java", p, Language::Java));
    }
}
