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

fn is_test_file(name: &str, _path: &Path, lang: Language) -> bool {
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
    let mut results = Vec::new();
    let re_func = Regex::new(r"(?m)^\s*def (test_\w+)").unwrap();
    let re_scaffold = Regex::new(r"@pytest\.mark\.scaffold").unwrap();
    let re_characterization = Regex::new(r"@pytest\.mark\.characterization").unwrap();
    let re_decision = Regex::new(r#"@pytest\.mark\.decision\((?:reason=)?"([^"]+)""#).unwrap();
    let re_contract = Regex::new(r#"@pytest\.mark\.contract\((?:party=)?"([^"]+)""#).unwrap();
    let re_incident = Regex::new(r#"@pytest\.mark\.incident\((?:ref=)?"([^"]+)""#).unwrap();

    let lines: Vec<&str> = content.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        if let Some(cap) = re_func.captures(line) {
            let func_name = cap[1].to_string();
            let context = if i >= 5 {
                lines[i - 5..i].join("\n")
            } else {
                lines[..i].join("\n")
            };

            let (tag, ref_value) = if re_scaffold.is_match(&context) {
                (LifecycleTag::Scaffold, None)
            } else if re_characterization.is_match(&context) {
                (LifecycleTag::Characterization, None)
            } else if let Some(cap) = re_decision.captures(&context) {
                (LifecycleTag::Decision, Some(cap[1].to_string()))
            } else if let Some(cap) = re_contract.captures(&context) {
                (LifecycleTag::Contract, Some(cap[1].to_string()))
            } else if let Some(cap) = re_incident.captures(&context) {
                (LifecycleTag::Incident, Some(cap[1].to_string()))
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
    }

    results
}

fn parse_rust_tags(path: &Path, content: &str) -> Vec<TaggedTest> {
    let mut results = Vec::new();

    if !content.contains("#[cfg(test)]") && !content.contains("#[test]") {
        return results;
    }

    let _in_scaffold = Regex::new(r"(?m)mod scaffold\s*\{").unwrap();
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

        let context = if i >= 3 {
            lines[i - 3..i].join("\n")
        } else {
            lines[..i].join("\n")
        };

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

        let context_start = i.saturating_sub(5);
        let context = lines[context_start..=i].join("\n");

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

            let context = if i >= 3 {
                lines[i - 3..i].join("\n")
            } else {
                lines[..i].join("\n")
            };

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
