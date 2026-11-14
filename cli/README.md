# Kinhin CLI

> TDD lifecycle manager for code-assistant workflows.

Rust companion to the [Kinhin](https://github.com/daviguides/kinhin) Claude Code plugin. It reads the lifecycle tag of every test (`scaffold`, `decision`, `contract`, `incident`), runs the suite under the Runner Contract, tags untagged tests through one Claude session, and deletes scaffolds, restoring every file when the suite stops collecting.

The plugin specs are the source of truth; the CLI is one way to execute them and the plugin works without it.

## What is supported

| | Python (uv + pytest) | Rust, Java, TypeScript |
|---|---|---|
| `census`, `audit` | yes | yes (tag parsers are unit-tested; not exercised on a real project) |
| `run` | yes | builds the runner command; **not verified against a real project** |
| `setup` | checks, and installs with `--install` | checks only |
| `tag` | suggests and writes markers | `--dry-run` only |
| `prune` | plan, delete | plan only |

Python projects not managed by uv are driven through `.venv/bin` or `PATH`, and `setup --install` will not install for them. That path exists in the code but has not been exercised on a real project.

## Install

```bash
cargo install --path cli
```

`kinhin tag` needs the Claude Code CLI, installed and logged in:

```bash
npm install -g @anthropic-ai/claude-code && claude login
```

## The flow

```bash
kinhin setup --install          # dev deps, pytest markers
kinhin census                   # how many tests, how many untagged
kinhin tag --dry-run            # one Claude session; suggestions saved, nothing written
kinhin tag                      # write the cached suggestions (no new session)
kinhin run --mode full          # whole suite minus scaffolds
kinhin prune                    # what would be deleted
kinhin prune --apply           # delete scaffolds; then run the suite
```

## Commands

### `kinhin setup [PATH] [--install]`

Checks, per detected language, what the other commands need. For Python: `pytest`, `pytest-xdist`, `pytest-randomly`, `pytest-testmon` and the five lifecycle markers registered under `[tool.pytest.ini_options] markers`. Also checks that `claude` is on `PATH`.

`--install` (uv-managed Python projects):

- runs `uv add --dev` for the missing packages;
- registers the missing lifecycle markers in `pyproject.toml`, keeping any marker you already have;
The marker edit is idempotent and never replace an existing key. If the project has a `pytest.ini`, markers are not written (pytest would ignore them there); setup says so.

Exit code: `0` ready, `1` something is missing, `2` error.

### `kinhin census [PATH] [--output rich|json]`

Counts tests by lifecycle tag.

### `kinhin audit [PATH] [--output rich|json]`

Lists every test with its tag and classifies what each permanent tag points at:

| Mark | Meaning |
|---|---|
| `✓` linked | a ticket id (`ABC-123`), a URL, or a path that exists (an `#anchor` is ignored) |
| `·` inline | free-text reason: valid, nothing to check it against |
| `⚠` broken | looks like a path and the path does not exist |
| `✗` missing | permanent tag with no reason or ref |

Exit code: `1` when any ref is broken or missing, else `0`. Untagged tests are reported, not an error.

### `kinhin run [--mode loop|full|diagnostic] [--lang L] [--ts-runner jest|vitest] [--path P] [-- ARGS]`

Runs the test runner with Runner Contract flags and exits with the runner's exit code. For a uv project the command is `uv run pytest ...`, so no virtualenv needs activating. Flags for a plugin that is not installed are left out and a warning names what is missing.

| Mode | pytest arguments |
|---|---|
| `loop` (default) | `-ra -q --maxfail=0 -n auto --randomly-seed=N --testmon` |
| `full` | `-ra -q --maxfail=0 -n auto --randomly-seed=N -m 'not scaffold'` |
| `diagnostic` | `-ra -q -x -p no:randomly -p no:xdist` |

The seed is generated per run and printed in the command line, so a failing order can be replayed with `-- --randomly-seed=N`. Everything after `--` goes to the runner. `loop` relies on pytest-testmon, which writes `.testmondata` in the project root.

Commands built for the other languages (unverified): `cargo nextest run --no-fail-fast`, `mvn test` / `gradle test --continue`, `npx|pnpx|bunx jest|vitest`.

### `kinhin tag [--dry-run] [--force] [--model M] [--path P] [--output rich|json]`

Classifies every untagged test in **one** Claude session (one turn per test file), then writes the markers.

- `--dry-run`: run the session, save the suggestions to `.kinhin/tag-suggestions.json`, write nothing.
- no flag: apply. Files whose untagged tests are all covered by the cache are applied from it without a session; only the rest go to a session. The cache is removed after a successful apply.
- `--force`: ignore the cache.
- `--model`: Claude model, default `haiku`.

The session has no tools and loads no user or project settings. Tests are identified as `Class::test_name`, so the same test name in two classes is not confused. An id the model skips, or a tag it invents, becomes `scaffold`. An `incident` with no ticket is written as a `decision`.

What apply writes (Python):

```python
@pytest.mark.scaffold
@pytest.mark.decision(reason="...")
@pytest.mark.contract(party="...")
@pytest.mark.incident(ref="...")
```

- long text is wrapped to the project's `[tool.ruff] line-length` (79 when unset), in the shape `ruff format` keeps;
- `import pytest` is added when the file does not bind `pytest`, and the project's ruff (if installed) sorts the import block of those files;
- the lifecycle markers are registered in `pyproject.toml`;
- tests that already carry a lifecycle marker are left alone, so running `tag` twice changes nothing.

Then it verifies: `pytest --collect-only` must succeed with unknown marks as errors, and no test may be left untagged. If collection fails, every file is restored and nothing is written.

If the CLI process behind the session dies mid-run, kinhin reconnects once and retries that file. If files still fail, the classified ones stay in the cache and nothing is applied.

Every session is logged to `.kinhin/session-<timestamp>.jsonl` (`session_start`, `file_start`, `file_complete` with turns/cost/duration, `file_tagged`, `file_error`, `session_end`).

Exit code: `0` done, `1` not completed (files restored), `2` usage or session error.

### `kinhin prune [--apply] [--path P] [--output rich|json]`

Deterministic; no agent session. Tests tagged `scaffold` or `characterization` are planned for deletion, permanent tags are kept.

- no flag: print the plan.
- `--apply`: refuses while any test is untagged (run `kinhin tag` first). Deletes each planned test with its decorators, removes classes left empty and files left without tests, then checks that the suite still collects. If the project has ruff, it removes imports orphaned by the deletion (`F401`, `I`) in the touched files.
- If any step fails, every file is restored and the exit code is `1`.

After `--apply`, run the suite: a test that goes red means a deleted scaffold was load-bearing. Restore it and promote it to a `decision` with a reason.

The last line is the PR census line: `written N / pruned M / survivors: X decision, Y contract, Z incident, S scaffold`.

Exit code: `0` done, `1` refused or rolled back, `2` tool error.

## Files kinhin leaves in the project

| Path | Written by | Keep out of git |
|---|---|---|
| `.kinhin/` | `tag` (session logs, suggestion cache) | yes |
| `.testmondata` | pytest-testmon, through `run --mode loop` | yes |

## Language detection

Markers looked up to depth 3: `pyproject.toml`, `setup.py`, `setup.cfg` → Python; `Cargo.toml` → Rust; `pom.xml`, `build.gradle(.kts)` → Java; `tsconfig.json` or a `package.json` with jest/vitest/typescript → TypeScript. Same table as the plugin's `/kinhin:load`.

## Development

```bash
cargo test            # parsers, marker insertion/deletion, pyproject edits
cargo build --release
```

## License

MIT OR Apache-2.0
