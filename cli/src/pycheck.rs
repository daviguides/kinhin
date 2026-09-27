//! Post-edit checks for Python test files: does the suite still collect,
//! and best-effort lint hygiene through the project's own ruff.

use std::path::PathBuf;
use std::process::Stdio;

use crate::env::PyEnv;

/// `pytest --collect-only` with unknown marks promoted to errors.
/// `Ok(n)` = n tests collected; `Err` carries pytest's own explanation.
pub fn collect(py: &PyEnv) -> Result<usize, String> {
    let output = py
        .tool("pytest")
        .args([
            "--collect-only",
            "-q",
            "-p",
            "no:cacheprovider",
            "-W",
            "error::pytest.PytestUnknownMarkWarning",
        ])
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("cannot launch pytest: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let code = output.status.code().unwrap_or(-1);
    if code == 0 || code == 5 {
        return Ok(stdout.lines().filter(|l| l.contains("::")).count());
    }
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();
    let tail = lines[lines.len().saturating_sub(15)..].join("\n    ");
    Err(format!("pytest could not collect the suite (exit {code}):\n    {tail}"))
}

/// Run `ruff check --fix` limited to `rules` on `files`, when the project
/// has ruff. Returns whether ruff ran. Failures are ignored: this is
/// hygiene, the collect check is the correctness gate.
pub fn ruff_fix(py: &PyEnv, files: &[PathBuf], rules: &str) -> bool {
    if files.is_empty() || !py.modules_available(&["ruff"])["ruff"] {
        return false;
    }
    py.tool("ruff")
        .args(["check", "--quiet", "--fix", "--select", rules])
        .args(files)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}
