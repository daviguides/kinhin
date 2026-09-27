# Kinhin CLI

> TDD lifecycle manager for code-assistant workflows.

Rust companion to the [Kinhin](https://github.com/daviguides/kinhin) Claude Code plugin. It reads the lifecycle tag of every test (`scaffold`, `decision`, `contract`, `incident`), runs the suite under the Runner Contract, tags untagged tests through one Claude session, deletes scaffolds, and proves with mutation testing that nothing load-bearing was deleted.

The plugin specs are the source of truth; the CLI is one way to execute them and the plugin works without it.

## What is supported

| | Python (uv + pytest) | Rust, Java, TypeScript |
|---|---|---|
| `census`, `audit` | yes | yes (tag parsers are unit-tested; not exercised on a real project) |
| `run` | yes | builds the runner command; **not verified against a real project** |
| `setup` | checks, and installs with `--install` | checks only |
| `tag` | suggests and writes markers | `--dry-run` only |
| `gate` | yes (mutmut ≥ 3) | no |
| `prune` | plan, delete, verify | plan only |

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
kinhin setup --install          # dev deps, pytest markers, [tool.mutmut]
kinhin census                   # how many tests, how many untagged
kinhin tag --dry-run            # one Claude session; suggestions saved, nothing written
kinhin tag                      # write the cached suggestions (no new session)
kinhin run --mode full          # whole suite minus scaffolds
kinhin prune                    # what would be deleted
kinhin prune --apply --verify   # delete scaffolds, keep the load-bearing ones
```

## Commands

### `kinhin setup [PATH] [--install]`

Checks, per detected language, what the other commands need. For Python: `pytest`, `pytest-xdist`, `pytest-randomly`, `pytest-testmon`, `mutmut`, the five lifecycle markers registered under `[tool.pytest.ini_options] markers`, and a `[tool.mutmut]` table. Also checks that `claude` is on `PATH`.

`--install` (uv-managed Python projects):

- runs `uv add --dev` for the missing packages;
- registers the missing lifecycle markers in `pyproject.toml`, keeping any marker you already have;
- writes `[tool.mutmut] source_paths = [...]` when the project has no mutmut config (`src/`, else the package named after `[project] name`, else top-level packages).

Both edits are idempotent and never replace an existing key. If the project has a `pytest.ini`, markers are not written (pytest would ignore them there); setup says so.

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

### `kinhin gate [--save-baseline FILE] [--baseline FILE] [--scope GLOB]... [--path P] [--output rich|json]`

Mutation parity on mutmut. K is the **set** of killed mutant ids, not a count.

- Every run starts clean (`mutants/` is deleted first): results from an earlier run describe a different suite.
- `--save-baseline FILE` writes the run (every mutant id and its status).
- `--baseline FILE` compares: the run passes only if every mutant killed in the baseline is still killed. Mutants that are no longer killed are listed with their new status (`survived`, `segfault`, `timeout`, `absent`, ...).
- `--scope` restricts the run to mutant-name globs (`'pkg.services.billing*'`). A baseline and a run with different scopes are refused.
- `segfault`, `timeout` and `suspicious` mutants are reported under their own status and never count as killed.

Exit code: `0` pass or no baseline given, `1` parity lost, `2` the tool could not produce a result (mutmut missing or unconfigured, suite not green, no mutant checked, unreadable baseline). A failed mutmut run is never a pass; its last output lines are shown.

mutmut needs a green suite. On macOS kinhin runs mutmut with `NO_PROXY=*`: mutmut forks its workers, and urllib's system proxy lookup is not fork-safe and segfaults them (on the marks project, 80 of 634 mutants ended as `segfault` without it, 0 with it).

### `kinhin prune [--apply [--verify]] [--path P] [--output rich|json]`

Deterministic; no agent session. Tests tagged `scaffold` or `characterization` are planned for deletion, permanent tags are kept.

- no flag: print the plan.
- `--apply`: refuses while any test is untagged (run `kinhin tag` first). Deletes each planned test with its decorators, removes classes left empty and files left without tests, then checks that the suite still collects. If the project has ruff, it removes imports orphaned by the deletion (`F401`, `I`) in the touched files.
- `--apply --verify`:
  1. K₀: mutmut with the full suite.
  2. Delete all scaffolds; K₁.
  3. If baseline-killed mutants escaped, the deleted scaffolds that execute the functions those mutants live in are restored (from mutmut's per-function test coverage), and mutmut runs again.
  4. If parity still does not hold, or any step fails, every file is restored and the exit code is `1`.

The restored scaffolds are listed as load-bearing. They keep their `scaffold` tag: promoting one to a `decision` needs a reason only you can give. The selection is conservative (coverage, not proof of which test kills which mutant), so some of them may be deletable.

The last line is the PR census line: `written N / pruned M / survivors: X decision, Y contract, Z incident, S scaffold`.

Exit code: `0` done, `1` refused or rolled back, `2` tool error.

## Files kinhin leaves in the project

| Path | Written by | Keep out of git |
|---|---|---|
| `.kinhin/` | `tag` (session logs, suggestion cache) | yes |
| `mutants/` | mutmut, through `gate` and `prune --verify` | yes |
| `.testmondata` | pytest-testmon, through `run --mode loop` | yes |

## Language detection

Markers looked up to depth 3: `pyproject.toml`, `setup.py`, `setup.cfg` → Python; `Cargo.toml` → Rust; `pom.xml`, `build.gradle(.kts)` → Java; `tsconfig.json` or a `package.json` with jest/vitest/typescript → TypeScript. Same table as the plugin's `/kinhin:load`.

## Development

```bash
cargo test            # parsers, marker insertion/deletion, parity, pyproject edits
cargo build --release
```

`tests/fixtures/mutmut-3.8-results-all.txt` is real `mutmut results --all true` output; the gate parser is tested against it.

## License

MIT OR Apache-2.0
