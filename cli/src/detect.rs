use std::collections::BTreeSet;
use std::path::Path;

use walkdir::WalkDir;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, clap::ValueEnum)]
pub enum Language {
    Python,
    Rust,
    Java,
    TypeScript,
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::Python => write!(f, "Python"),
            Language::Rust => write!(f, "Rust"),
            Language::Java => write!(f, "Java"),
            Language::TypeScript => write!(f, "TypeScript"),
        }
    }
}

const SKIP_DIRS: &[&str] = &[
    "node_modules",
    ".venv",
    "target",
    ".git",
    "__pycache__",
    "dist",
    "build",
];

fn should_skip(entry: &walkdir::DirEntry) -> bool {
    entry.file_type().is_dir()
        && entry
            .file_name()
            .to_str()
            .is_some_and(|name| SKIP_DIRS.contains(&name))
}

pub fn detect_languages(root: &str) -> Vec<Language> {
    let mut found = BTreeSet::new();

    for entry in WalkDir::new(root)
        .max_depth(3)
        .into_iter()
        .filter_entry(|e| !should_skip(e))
        .flatten()
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let Some(name) = entry.file_name().to_str() else {
            continue;
        };

        match name {
            "pyproject.toml" | "setup.py" | "setup.cfg" => {
                found.insert(Language::Python);
            }
            "Cargo.toml" => {
                found.insert(Language::Rust);
            }
            "pom.xml" | "build.gradle" | "build.gradle.kts" => {
                found.insert(Language::Java);
            }
            "tsconfig.json" => {
                found.insert(Language::TypeScript);
            }
            "package.json" => {
                if has_ts_test_deps(entry.path()) {
                    found.insert(Language::TypeScript);
                }
            }
            _ => {}
        }
    }

    found.into_iter().collect()
}

fn has_ts_test_deps(package_json: &Path) -> bool {
    let Ok(content) = std::fs::read_to_string(package_json) else {
        return false;
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) else {
        return false;
    };

    let check_deps = |key: &str| -> bool {
        value
            .get(key)
            .and_then(|v| v.as_object())
            .is_some_and(|deps| {
                deps.keys().any(|k| {
                    matches!(
                        k.as_str(),
                        "jest" | "vitest" | "ts-jest" | "@jest/core" | "typescript"
                    )
                })
            })
    };

    check_deps("devDependencies") || check_deps("dependencies")
}
