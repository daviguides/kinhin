# Kinhin CLI

> TDD lifecycle manager for code-assistant workflows.

Rust CLI companion to the [Kinhin](https://github.com/daviguides/kinhin) Claude Code plugin. Deterministic commands for auditing, running, and gating tests, plus agent-powered commands for auto-tagging and pruning via [claude-agent-toolkit](https://github.com/daviguides/claude-agent-toolkit).

## Install

```bash
cd cli && cargo build --release
# or
cargo install --path cli
```

**Requirements for agent commands** (`tag`, `prune`):
```bash
npm install -g @anthropic-ai/claude-code
claude login
```

## Commands

| Command | Mode | What it does |
|---|---|---|
| `kinhin audit` | Deterministic | Scan test files, report tags, validate refs |
| `kinhin census` | Deterministic | Count tests by tag, display summary |
| `kinhin run` | Deterministic | Wrap test runner with Runner Contract config |
| `kinhin gate` | Deterministic | Mutation parity check (K₁ ⊇ K₀) |
| `kinhin tag` | Agent session | Auto-tag tests via Claude |
| `kinhin prune` | Agent session | Census, classify, collapse, delete, verify |

## Usage

```bash
kinhin audit                          # scan and report
kinhin census                         # tag counts

kinhin run --mode full                # permanent tests, no bail
kinhin run --mode loop                # changed-set only, fast feedback
kinhin run --mode diagnostic          # serial, fixed order, isolate flake

kinhin gate --save-baseline pre.json  # save mutation baseline (K₀)
kinhin gate --baseline pre.json       # compare against baseline (K₁ ⊇ K₀)

kinhin tag                            # dry run: show suggestions
kinhin tag --apply                    # auto-tag and write to files

kinhin prune                          # dry run: show prune plan
kinhin prune --apply                  # execute deletions
kinhin prune --apply --verify         # delete + mutation parity check
```

## Global Flags

| Flag | Effect |
|---|---|
| `--output rich` | Styled terminal output with tables and colors (default) |
| `--output json` | Machine-readable JSON |
| `-v` / `-vv` | Increase verbosity (info / debug) |

## Language Detection

Kinhin auto-detects languages by scanning for project markers (max depth 3):

| Marker | Language |
|---|---|
| `pyproject.toml` | Python |
| `Cargo.toml` | Rust |
| `pom.xml` / `build.gradle` / `build.gradle.kts` | Java |
| `tsconfig.json` | TypeScript |

Override with `--lang python|rust|java|typescript` on any command.

## License

Dual-licensed under [MIT](../LICENSE) or Apache-2.0, at your option.
