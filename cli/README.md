# Kinhin CLI

> TDD lifecycle manager for code-assistant workflows.

Rust CLI companion to the [Kinhin](https://github.com/daviguides/kinhin) Claude Code plugin. Deterministic commands for auditing, running, and gating tests, plus agent-powered commands for auto-tagging and pruning via [claude-agent-toolkit](https://github.com/daviguides/claude-agent-toolkit).

## Install

```bash
cargo install --path cli
# or
cd cli && cargo build --release && cp target/release/kinhin ~/.cargo/bin/
```

**Requirements for agent commands** (`tag`, `prune`):
```bash
npm install -g @anthropic-ai/claude-code
claude login
```

## Quick Start

```bash
kinhin setup              # check runner deps for detected language
kinhin setup --install    # install missing deps
kinhin audit              # scan test files, report tags, validate refs
kinhin census             # tag counts at a glance
kinhin run --mode loop    # fast feedback: changed-set, parallel, no bail
kinhin tag --dry-run      # preview tag suggestions (saves to cache)
kinhin tag                # apply from cache (or run fresh if no cache)
kinhin prune              # show prune plan (dry run)
kinhin prune --apply      # execute deletions
```

## Commands

| Command | Mode | What it does |
|---|---|---|
| `kinhin setup` | Deterministic | Check/install runner dependencies |
| `kinhin audit` | Deterministic | Scan test files, report tags, validate refs |
| `kinhin census` | Deterministic | Count tests by lifecycle tag |
| `kinhin run` | Deterministic | Wrap test runner with Runner Contract config |
| `kinhin gate` | Deterministic | Mutation parity check (K₁ ⊇ K₀) |
| `kinhin tag` | Agent session | Auto-tag tests via Claude |
| `kinhin prune` | Agent session | Census → classify → collapse → delete → verify |

### `kinhin setup`

Checks and installs runner dependencies per detected language.

```bash
kinhin setup              # check only
kinhin setup --install    # install missing
```

### `kinhin run`

Wraps the project's test runner with Runner Contract flags (no bail, parallel, randomized, structured output). Detects the environment:
- **Python with `uv.lock`**: uses `uv run pytest ...`
- **Python without uv**: uses `pytest ...` directly
- **TypeScript with `pnpm-lock.yaml`**: uses `pnpx jest|vitest ...`
- **TypeScript with `bun.lockb`**: uses `bunx jest|vitest ...`
- **TypeScript default**: uses `npx jest|vitest ...`
- **Rust**: uses `cargo nextest run ...`
- **Java**: uses `gradle test` or `mvn test`

```bash
kinhin run --mode loop         # changed-set only, fast feedback (default)
kinhin run --mode full         # full suite, exclude scaffolds, pre-PR gate
kinhin run --mode diagnostic   # serial, fixed order, isolate flake
kinhin run --lang python       # override language detection
kinhin run --ts-runner vitest  # override TypeScript runner
kinhin run -- --timeout 30     # pass extra args to underlying runner
```

### `kinhin gate`

Runs mutation testing and checks parity (K₁ ⊇ K₀).

```bash
kinhin gate --save-baseline pre.json   # save K₀ before prune
kinhin gate --baseline pre.json        # compare K₁ against K₀ after prune
kinhin gate                            # just run and report (no comparison)
```

Exit codes: `0` pass, `1` fail (parity lost), `2` tool not found.

### `kinhin tag`

Auto-classifies tests via Claude agent session. One session for all files (not per-file).

```bash
kinhin tag --dry-run       # run session, save to .kinhin/tag-suggestions.json, don't apply
kinhin tag                 # apply from cache if exists, else run fresh session
kinhin tag --force         # ignore cache, run fresh session, apply
kinhin tag --model sonnet  # use a different model (default: haiku)
```

Inserts per-language markers: `@pytest.mark.scaffold`, `// kinhin: decision(ref="...")`, `@Tag("scaffold")`, file suffix convention for TypeScript. Adds `import pytest` if missing.

### `kinhin prune`

Full prune pipeline: census → agent classify untagged → propose collapse/delete → apply → mutation verify.

```bash
kinhin prune                    # dry run: show plan
kinhin prune --apply            # execute deletions
kinhin prune --apply --verify   # execute + mutation parity check
```

### `kinhin audit`

Scans all test files, reports each test's lifecycle tag, validates refs.

```bash
kinhin audit                    # rich table
kinhin audit --output json      # machine-readable
```

### `kinhin census`

Quick tag count with visual bars.

```bash
kinhin census                   # rich table
kinhin census --output json     # machine-readable
```

## Global Flags

| Flag | Effect |
|---|---|
| `--output rich` | Styled terminal output with tables and colors (default) |
| `--output json` | Machine-readable JSON |
| `-v` / `-vv` | Increase verbosity (info / debug) |

## Language Detection

Auto-detects by scanning for project markers (max depth 3):

| Marker | Language |
|---|---|
| `pyproject.toml` | Python |
| `Cargo.toml` | Rust |
| `pom.xml` / `build.gradle` / `build.gradle.kts` | Java |
| `tsconfig.json` | TypeScript |

Override with `--lang python|rust|java|typescript`.

## Event Logging

Agent sessions (`tag`, `prune`) write NDJSON events to `.kinhin/session-{timestamp}.jsonl`:
- `session_start`, `file_start`, `file_complete` (turns, cost, tool_calls), `file_tagged`, `session_end`
- Session metrics printed at disconnect: files processed, total turns, total cost

## Correspondence to Kinhin Guides

| Guide Step | CLI Command |
|---|---|
| Prune Guide → Step 1: Tag Census | `kinhin census` / `kinhin audit` |
| Prune Guide → Step 2: Classify Untagged | `kinhin tag` |
| Prune Guide → Step 3-4: Collapse + Delete | `kinhin prune --apply` |
| Prune Guide → Step 5: Mutation Parity | `kinhin gate` |
| Prune Guide → Step 6: Census Line | `kinhin census --output json` |
| Runner Guide → Loop Mode | `kinhin run --mode loop` |
| Runner Guide → Full Mode | `kinhin run --mode full` |
| Runner Guide → Diagnostic Mode | `kinhin run --mode diagnostic` |
| Checklist → Runner Config | `kinhin setup --install` |

## License

Dual-licensed under [MIT](../LICENSE) or Apache-2.0, at your option.
