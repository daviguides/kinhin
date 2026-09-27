# Kinhin CLI — Claude Code Project Instructions

## Overview

Rust CLI companion to the Kinhin TDD plugin. The plugin specs (`../kinhin/spec/`) define the lifecycle; this crate executes it. A behaviour change starts in the spec, then lands here. User-facing behaviour, flags and exit codes: `README.md`.

## Commands

```bash
cargo test                               # must pass, zero warnings
cargo build --release
cp target/release/kinhin ~/.cargo/bin/   # install without touching crates.io
```

## Definition of done for any change here

"It compiles" is not evidence. A command is done when it has been run against a real project and its full output read.

- Keep a private copy of a real Python project as fixture; never test in a worktree someone is using.
- After a change to `tag`, `prune` or `gate`: run the command, then `pytest --collect-only`, `kinhin census`, and the project's linter.
- Parsers of tool output are written from captured output, saved under `tests/fixtures/`, never from memory.
- A tool that fails must make the command fail. No result may be reported from an empty or errored run.
- A flag changed here is changed in `README.md` in the same commit.

## Architecture

```
src/
├── main.rs       clap surface and dispatch; each command returns its exit code
├── env.rs        how to launch project tooling (uv run / .venv / PATH), ts package runner
├── detect.rs     language detection by project markers
├── tags.rs       scan test files, read lifecycle tags (Python via pytests.rs; Rust, Java, TS)
├── pytests.rs    Python test-file surgery: locate tests, read/insert markers, delete tests
├── pyproject.rs  additive pyproject.toml edits (pytest markers, [tool.mutmut]) via toml_edit
├── pycheck.rs    pytest --collect-only gate, best-effort ruff fix
├── census.rs     counts by tag
├── audit.rs      per-test report, ref classification
├── run.rs        Runner Contract command per language; pytest args are a pure function
├── setup.rs      dependency/config check, uv install
├── gate.rs       mutmut run, result parsing, killed-set parity
├── agent.rs      TaggerSession over claude-agent-toolkit, NDJSON event log
├── tag.rs        tag flow: cache, session, apply, verify, rollback
├── prune.rs      plan, delete, verify with gate, restore load-bearing, rollback
└── display.rs    tables; warnings/errors/progress go to stderr
```

## Invariants

- **Fail closed.** `gate` exits 2 when mutmut fails, generates no mutants or checks none; `tag` and `prune` restore every file when the suite stops collecting or parity is lost.
- **K is a set.** Parity compares killed mutant ids, never counts. `segfault`/`timeout`/`suspicious` are never killed.
- **Clean mutation runs.** `gate::run_mutmut` deletes `mutants/` first: old results describe another suite.
- **macOS:** mutmut is launched with `NO_PROXY=*`. Without it, urllib's proxy lookup segfaults mutmut's forked workers (crash dialogs, flaky statuses).
- **Destructive commands default to dry run** (`prune`). `tag` applies by default and is safe to repeat.
- **stdout carries results, stderr carries progress**, so `--output json` is always parseable.
- **One agent session per run**, one turn per file; never one session per file. The session has no tools and loads no user or project settings.
- **Python tests are identified as `Class::test`**; bare names collide.
- **A lifecycle tag is read only from the decorator block directly above the test** (or the contiguous annotation/comment block in Rust, Java, TS). A fixed "N lines above" window leaks tags between tests.
- **pyproject edits are additive and idempotent**; existing keys are never replaced.
- **No pip.** Python environments are uv: `uv add --dev`, `uv run`.
- Language detection must match the plugin's table (`../kinhin/prompts/load.md`).

## Scope limits (say so, don't fake it)

`tag` apply, `prune --apply` and `gate` are Python-only. Other languages get an explicit error, not a silent no-op. `run` for Rust/Java/TypeScript builds a command that has not been verified on a real project.

## Dependencies

`claude-agent-toolkit` (git, agent session), `clap`, `comfy-table`, `owo-colors`, `serde`/`serde_json`, `toml_edit`, `walkdir`, `regex`, `chrono`, `tokio`/`futures`, `tempfile`, `anyhow`.

## Release

The CLI is versioned in `Cargo.toml`, separately from the plugin (`../.claude-plugin/plugin.json`). Plugin release workflow: `../CLAUDE.md`.
