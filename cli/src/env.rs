//! Project environment resolution shared by `run`, `setup`, `gate`, `tag`
//! and `prune`: how to invoke Python tooling inside the project's own
//! environment instead of whatever happens to be on PATH.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// How Python tools are launched for a project.
#[derive(Debug, Clone)]
pub struct PyEnv {
    pub root: PathBuf,
    pub uv: bool,
}

impl PyEnv {
    pub fn detect(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            uv: uses_uv(root),
        }
    }

    /// Command for a tool installed in the project environment
    /// (`uv run <tool>`, `.venv/bin/<tool>`, or the bare name).
    pub fn tool(&self, program: &str) -> Command {
        let mut command = if self.uv {
            let mut c = Command::new("uv");
            c.args(["run", program]);
            c
        } else {
            let venv_bin = self.root.join(".venv/bin").join(program);
            if venv_bin.exists() {
                Command::new(venv_bin)
            } else {
                Command::new(program)
            }
        };
        command.current_dir(&self.root);
        command
    }

    pub fn python(&self) -> Command {
        if self.uv {
            return self.tool("python");
        }
        let venv_python = self.root.join(".venv/bin/python");
        let mut command = if venv_python.exists() {
            Command::new(venv_python)
        } else {
            Command::new("python3")
        };
        command.current_dir(&self.root);
        command
    }

    /// The words shown to the user for a tool invocation.
    pub fn display_words(&self, program: &str) -> Vec<String> {
        if self.uv {
            vec!["uv".into(), "run".into(), program.into()]
        } else {
            vec![program.into()]
        }
    }

    /// Which of `modules` are importable in the project environment.
    /// One interpreter launch for the whole list.
    pub fn modules_available(&self, modules: &[&str]) -> HashMap<String, bool> {
        let script = "import importlib.util, sys\n\
             for m in sys.argv[1:]:\n\
             \x20   print(m, 1 if importlib.util.find_spec(m) else 0)";
        let output = self
            .python()
            .arg("-c")
            .arg(script)
            .args(modules)
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output();

        let mut found: HashMap<String, bool> =
            modules.iter().map(|m| ((*m).to_string(), false)).collect();
        if let Ok(out) = output {
            for line in String::from_utf8_lossy(&out.stdout).lines() {
                if let Some((module, flag)) = line.split_once(' ') {
                    found.insert(module.to_string(), flag.trim() == "1");
                }
            }
        }
        found
    }
}

/// A project is driven through uv when it has a `uv.lock` or a `[tool.uv]`
/// table, and the `uv` binary is installed.
pub fn uses_uv(root: &Path) -> bool {
    let declared = root.join("uv.lock").exists()
        || std::fs::read_to_string(root.join("pyproject.toml"))
            .is_ok_and(|content| content.contains("[tool.uv"));
    declared && binary_on_path("uv")
}

pub fn binary_on_path(name: &str) -> bool {
    Command::new("which")
        .arg(name)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

/// Package runner for a TypeScript project, chosen by lockfile.
pub fn ts_package_runner(root: &Path) -> &'static str {
    if root.join("bun.lockb").exists() || root.join("bun.lock").exists() {
        "bunx"
    } else if root.join("pnpm-lock.yaml").exists() {
        "pnpx"
    } else {
        "npx"
    }
}

pub fn canonical_root(path: &str) -> PathBuf {
    Path::new(path)
        .canonicalize()
        .unwrap_or_else(|_| Path::new(path).to_path_buf())
}
