# Kinhin CLI — Claude Code Project Instructions

## Overview

Rust CLI companion to the Kinhin TDD plugin. Automates lifecycle management: auditing tags, running tests with the Runner Contract, mutation gating, auto-tagging via AI, and pruning construction tests.

## Commands

```bash
cargo build --release              # Build
cargo install --path .             # Install to ~/.cargo/bin/
cargo test                         # Run tests (when they exist)
cp target/release/kinhin ~/.cargo/bin/  # Manual install (if crates.io DNS fails)
```

## Architecture

```
src/
├── main.rs       # clap entry point, subcommand dispatch
├── audit.rs      # kinhin audit — tag census + ref validation
├── census.rs     # kinhin census — tag count display
├── run.rs        # kinhin run — wraps runner with Runner Contract flags
├── setup.rs      # kinhin setup — check/install runner deps
├── gate.rs       # kinhin gate — mutation parity (K₁ ⊇ K₀)
├── tag.rs        # kinhin tag — auto-classify via agent session
├── prune.rs      # kinhin prune — full prune pipeline
├── agent.rs      # TaggerSession — claude-agent-toolkit wrapper
├── detect.rs     # language detection by project markers
├── tags.rs       # tag parser per language (pytest marks, mod scaffold, @Tag, file suffix)
├── display.rs    # comfy-table styled output, icons, colors
└── Cargo.toml
```

## Two modes

**Deterministic** (no LLM): `audit`, `census`, `run`, `setup`, `gate`
**Agent session** (claude-agent-toolkit): `tag`, `prune`

Agent commands open ONE `ClaudeClient` session, process all files sequentially, then disconnect. Never one session per file.

## Key decisions

- **uv-aware**: Python runner detects `uv.lock` and uses `uv run pytest ...` instead of bare `pytest`
- **Package manager detection**: TypeScript detects `bun.lockb` (bunx), `pnpm-lock.yaml` (pnpx), fallback `npx`
- **mutmut v3.8**: config-only API, no `--paths-to-mutate` CLI flag. Config in `[tool.mutmut]` section of `pyproject.toml`
- **Graceful degradation**: `kinhin run` omits flags for missing pytest plugins and prints what to install
- **Event logging**: agent sessions write NDJSON to `.kinhin/session-{timestamp}.jsonl`
- **Tag cache**: `kinhin tag --dry-run` saves to `.kinhin/tag-suggestions.json`, `kinhin tag` reads from cache, `--force` refreshes
- **Default model**: `haiku` for tagging (cost efficiency). Override with `--model sonnet|opus`
- **Default apply**: `kinhin tag` applies by default, `--dry-run` is the preview flag

## Dependencies

| Crate | Purpose |
|---|---|
| `claude-agent-toolkit` (git) | Agent sessions for tag/prune |
| `clap` | CLI parsing (derive) |
| `comfy-table` | Rich terminal tables |
| `owo-colors` | Terminal colors |
| `serde` + `serde_json` | Structured I/O |
| `walkdir` | File tree scanning |
| `regex` | Tag pattern matching |
| `chrono` | Timestamps for event log |
| `tokio` + `futures` | Async runtime for agent sessions |
| `tempfile` | Sandbox for agent sessions |
| `anyhow` | Error handling |

## Scanner skip list

`tags.rs` skips: `node_modules`, `.venv`, `target`, `.git`, `__pycache__`, `dist`, `build`, `.claude`, `.kinhin`, `mutants`, `.mutmut-cache`, `.stryker-tmp`

## Detection table (must match plugin)

| Marker | Language |
|---|---|
| `pyproject.toml` | Python |
| `Cargo.toml` | Rust |
| `pom.xml` / `build.gradle` / `build.gradle.kts` | Java |
| `tsconfig.json` | TypeScript |
