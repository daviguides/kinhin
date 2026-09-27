//! Edits to a Python project's `pyproject.toml` that Kinhin needs:
//! lifecycle markers registered with pytest, and a `[tool.mutmut]` table.
//! Every edit is additive and idempotent; existing keys are never replaced.

use std::path::Path;

use toml_edit::{Array, DocumentMut, Item, Table, Value};

pub const LIFECYCLE_MARKERS: [(&str, &str); 5] = [
    ("scaffold", "construction-time test, deleted at prune"),
    ("characterization", "golden-master for a refactor, deleted at prune"),
    ("decision", "encodes a non-obvious decision (reason required)"),
    ("contract", "pins a boundary another party depends on (party required)"),
    ("incident", "reproduces a production failure (ref required)"),
];

fn parse(text: &str) -> Result<DocumentMut, String> {
    text.parse::<DocumentMut>()
        .map_err(|e| format!("pyproject.toml is not valid TOML: {e}"))
}

fn marker_name(entry: &str) -> &str {
    entry
        .split([':', '('])
        .next()
        .unwrap_or(entry)
        .trim()
}

/// Lifecycle markers not yet registered under `[tool.pytest.ini_options]`.
pub fn missing_markers(text: &str) -> Result<Vec<&'static str>, String> {
    let doc = parse(text)?;
    let registered: Vec<String> = doc
        .get("tool")
        .and_then(|t| t.get("pytest"))
        .and_then(|p| p.get("ini_options"))
        .and_then(|o| o.get("markers"))
        .and_then(Item::as_array)
        .map(|array| {
            array
                .iter()
                .filter_map(Value::as_str)
                .map(|entry| marker_name(entry).to_string())
                .collect()
        })
        .unwrap_or_default();

    Ok(LIFECYCLE_MARKERS
        .iter()
        .map(|(name, _)| *name)
        .filter(|name| !registered.iter().any(|r| r == name))
        .collect())
}

fn implicit_table() -> Item {
    let mut table = Table::new();
    table.set_implicit(true);
    Item::Table(table)
}

fn child_table<'a>(parent: &'a mut Item, key: &str, implicit: bool) -> Result<&'a mut Item, String> {
    let table = parent
        .as_table_like_mut()
        .ok_or_else(|| format!("expected a table above `{key}` in pyproject.toml"))?;
    if !table.contains_key(key) {
        table.insert(key, if implicit { implicit_table() } else { Item::Table(Table::new()) });
    }
    Ok(table.get_mut(key).expect("just inserted"))
}

/// Register the missing lifecycle markers. Returns the new text and the
/// names added (empty when nothing changed).
pub fn add_markers(text: &str) -> Result<(String, Vec<&'static str>), String> {
    let missing = missing_markers(text)?;
    if missing.is_empty() {
        return Ok((text.to_string(), missing));
    }
    let mut doc = parse(text)?;
    let root = doc.as_item_mut();
    let tool = child_table(root, "tool", true)?;
    let pytest = child_table(tool, "pytest", true)?;
    let options = child_table(pytest, "ini_options", false)?;
    let options = options
        .as_table_like_mut()
        .ok_or("`tool.pytest.ini_options` is not a table")?;

    if !options.contains_key("markers") {
        options.insert("markers", Item::Value(Value::Array(Array::new())));
    }
    let markers = options
        .get_mut("markers")
        .and_then(Item::as_array_mut)
        .ok_or("`tool.pytest.ini_options.markers` is not an array")?;

    for (name, description) in LIFECYCLE_MARKERS {
        if missing.contains(&name) {
            markers.push(format!("{name}: {description}"));
        }
    }
    for value in markers.iter_mut() {
        value.decor_mut().set_prefix("\n    ");
        value.decor_mut().set_suffix("");
    }
    markers.set_trailing_comma(true);
    markers.set_trailing("\n");

    Ok((doc.to_string(), missing))
}

pub fn has_mutmut_table(text: &str) -> Result<bool, String> {
    let doc = parse(text)?;
    Ok(doc.get("tool").and_then(|t| t.get("mutmut")).is_some())
}

/// Append a `[tool.mutmut]` table when the project has none.
/// Returns `None` when one already exists.
pub fn add_mutmut(text: &str, source_paths: &[String]) -> Result<Option<String>, String> {
    if has_mutmut_table(text)? {
        return Ok(None);
    }
    let mut doc = parse(text)?;
    let root = doc.as_item_mut();
    let tool = child_table(root, "tool", true)?;
    let mutmut = child_table(tool, "mutmut", false)?;
    let table = mutmut.as_table_like_mut().ok_or("`tool.mutmut` is not a table")?;
    let mut paths = Array::new();
    for path in source_paths {
        paths.push(path.as_str());
    }
    table.insert("source_paths", Item::Value(Value::Array(paths)));
    Ok(Some(doc.to_string()))
}

/// Where the code under test lives, for `[tool.mutmut] source_paths`.
pub fn guess_source_paths(root: &Path, text: &str) -> Vec<String> {
    if root.join("src").is_dir() {
        return vec!["src".to_string()];
    }
    let project_name = parse(text).ok().and_then(|doc| {
        doc.get("project")
            .and_then(|p| p.get("name"))
            .and_then(Item::as_str)
            .map(|name| name.replace('-', "_"))
    });
    if let Some(name) = project_name {
        if root.join(&name).is_dir() {
            return vec![name];
        }
    }
    let mut packages: Vec<String> = std::fs::read_dir(root)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|entry| entry.path().join("__init__.py").exists())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| !matches!(name.as_str(), "tests" | "test" | "mutants"))
        .collect();
    packages.sort();
    packages
}

/// The project's configured line length (`[tool.ruff] line-length`, then
/// `[tool.black] line-length`), if any.
pub fn line_length(text: &str) -> Option<usize> {
    let doc = parse(text).ok()?;
    let tool = doc.get("tool")?;
    ["ruff", "black"]
        .iter()
        .filter_map(|name| tool.get(name)?.get("line-length")?.as_integer())
        .next()
        .and_then(|n| usize::try_from(n).ok())
}

/// mutmut also reads `[mutmut]` from setup.cfg when pyproject has no table.
pub fn mutmut_configured(root: &Path) -> bool {
    let in_pyproject = std::fs::read_to_string(root.join("pyproject.toml"))
        .ok()
        .and_then(|text| has_mutmut_table(&text).ok())
        .unwrap_or(false);
    let in_setup_cfg = std::fs::read_to_string(root.join("setup.cfg"))
        .is_ok_and(|text| text.lines().any(|l| l.trim() == "[mutmut]"));
    in_pyproject || in_setup_cfg
}

/// Markers still unregistered for this project. `Err` explains why Kinhin
/// cannot tell (no pyproject, or pytest configured in another file).
pub fn markers_missing_in(root: &Path) -> Result<Vec<&'static str>, String> {
    if root.join("pytest.ini").exists() {
        return Err("pytest.ini takes precedence over pyproject.toml; register the lifecycle markers there".into());
    }
    let text = std::fs::read_to_string(root.join("pyproject.toml"))
        .map_err(|_| "no pyproject.toml".to_string())?;
    missing_markers(&text)
}

/// Register lifecycle markers in the project's pyproject.toml.
pub fn ensure_markers(root: &Path) -> Result<Vec<&'static str>, String> {
    if root.join("pytest.ini").exists() {
        return Err("pytest.ini takes precedence over pyproject.toml; register the lifecycle markers there".into());
    }
    let path = root.join("pyproject.toml");
    let text = std::fs::read_to_string(&path).map_err(|_| "no pyproject.toml".to_string())?;
    let (updated, added) = add_markers(&text)?;
    if !added.is_empty() {
        std::fs::write(&path, updated).map_err(|e| format!("cannot write pyproject.toml: {e}"))?;
    }
    Ok(added)
}

/// Write `[tool.mutmut]` when missing. `Ok(None)` when already configured.
pub fn ensure_mutmut(root: &Path) -> Result<Option<Vec<String>>, String> {
    if mutmut_configured(root) {
        return Ok(None);
    }
    let path = root.join("pyproject.toml");
    let text = std::fs::read_to_string(&path).map_err(|_| "no pyproject.toml".to_string())?;
    let source_paths = guess_source_paths(root, &text);
    if source_paths.is_empty() {
        return Err("could not find the source package; add `[tool.mutmut] source_paths = [...]` by hand".into());
    }
    match add_mutmut(&text, &source_paths)? {
        Some(updated) => {
            std::fs::write(&path, updated).map_err(|e| format!("cannot write pyproject.toml: {e}"))?;
            Ok(Some(source_paths))
        }
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: &str = "\
[project]
name = \"marks\"

[tool.pytest.ini_options]
testpaths = [\"tests\"]
asyncio_mode = \"auto\"

[dependency-groups]
dev = [\"pytest>=8.0.0\"]
";

    #[test]
    fn adds_all_markers_and_keeps_existing_keys() {
        let (updated, added) = add_markers(BASE).unwrap();
        assert_eq!(added.len(), 5);
        assert!(updated.contains("asyncio_mode = \"auto\""));
        assert!(updated.contains("\"scaffold: construction-time test, deleted at prune\""));
        assert!(missing_markers(&updated).unwrap().is_empty());
    }

    #[test]
    fn adding_markers_twice_changes_nothing() {
        let (once, _) = add_markers(BASE).unwrap();
        let (twice, added) = add_markers(&once).unwrap();
        assert!(added.is_empty());
        assert_eq!(once, twice);
    }

    #[test]
    fn keeps_user_markers_and_only_adds_the_missing_ones() {
        let text = "[tool.pytest.ini_options]\nmarkers = [\n    \"slow: slow test\",\n    \"decision(reason): mine\",\n]\n";
        let (updated, added) = add_markers(text).unwrap();
        assert_eq!(added, ["scaffold", "characterization", "contract", "incident"]);
        assert!(updated.contains("\"slow: slow test\""));
        assert!(updated.contains("\"decision(reason): mine\""));
    }

    #[test]
    fn creates_pytest_table_when_absent() {
        let (updated, _) = add_markers("[project]\nname = \"x\"\n").unwrap();
        assert!(updated.contains("[tool.pytest.ini_options]"), "{updated}");
        assert!(missing_markers(&updated).unwrap().is_empty());
    }

    #[test]
    fn mutmut_table_is_added_once_and_never_overwritten() {
        let added = add_mutmut(BASE, &["marks".to_string()]).unwrap().unwrap();
        assert!(added.contains("[tool.mutmut]"));
        assert!(added.contains("source_paths = [\"marks\"]"));
        assert!(add_mutmut(&added, &["other".to_string()]).unwrap().is_none());

        let custom = "[tool.mutmut]\nsource_paths = [\"lib\"]\ndebug = true\n";
        assert!(add_mutmut(custom, &["marks".to_string()]).unwrap().is_none());
    }

    #[test]
    fn reads_the_configured_line_length() {
        assert_eq!(line_length("[tool.ruff]\nline-length = 80\n"), Some(80));
        assert_eq!(line_length(BASE), None);
    }

    #[test]
    fn invalid_toml_is_an_error_not_a_rewrite() {
        assert!(add_markers("[tool\nbroken").is_err());
    }
}
