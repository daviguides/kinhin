# Python TDD Runner Specification

pytest configuration for the Runner Contract (`tdd-runner-spec.md`). The implementing author does not configure the runner: it runs the project's suite (`python-tdd-spec.md`, Running Tests).

## pytest Configuration

**Requirement**: Configure the test runner for code-assistant batch workflows (Runner Contract, `tdd-runner-spec.md`). Serial execution with bail-on-first-failure fights the batch processing principle.

```toml
[tool.pytest.ini_options]
addopts = "-n auto -p randomly --maxfail=0 -ra -q --strict-markers"
```

| Flag | Purpose |
|---|---|
| `-n auto` | pytest-xdist: parallel execution, one worker per core |
| `-p randomly` | pytest-randomly: randomized order, seed printed at start |
| `--maxfail=0` | No limit — never bail on first failure |
| `-ra` | Show summary of all non-passing tests at end |
| `-q` | Quiet — reduce noise, keep signal |
| `--strict-markers` | Fail on unregistered markers |

**Explicitly forbidden:** `-x` / `--exitfirst` in batch mode. Use `-x` ONLY in diagnostic mode (isolating a single flake manually).

**Plugins:** `pytest-xdist` (parallel, per-worker isolation), `pytest-randomly` (randomized order, seed printed), `pytest-json-report` (structured output), `pytest-testmon` (changed-set selection) and `pytest-rerunfailures` (ONLY under the three-run flake protocol, NEVER as retry-until-green) are used when the project installs them. A flag whose plugin is absent is left out of `addopts`: the project's `make test` is the source of truth.

### Run Modes

```bash
pytest --testmon -n auto -p randomly --maxfail=0               # loop: changed set only
pytest --lf --ff                                               # loop, cheap fallback
pytest -m "not scaffold" -n auto -p randomly --maxfail=0       # full gate before PR
pytest tests/test_suspect.py::test_name -p no:randomly -x      # diagnostic only
```

Selection NEVER replaces the full gate run.

### Three-Run Flake Protocol

```bash
# Step 1: isolated, 3 times
pytest tests/test_file.py::test_name --count=3 -p no:randomly -p no:xdist
# All 3 fail → deterministic-fail. Fix the code.

# Step 2: original environment
pytest -p randomly --randomly-seed=<SEED> -n auto
# Fails → isolation-dependent (shared state bug). Fix now.
# Passes → flaky. Quarantine with a ticket. Never retry-to-green.
```

### Structured Output and Clustering

```bash
pytest --json-report --json-report-file=results.json
```

Key fields: `.tests[].outcome`, `.tests[].call.crash.path`, `.tests[].call.crash.lineno`, `.tests[].call.crash.message`.

Without JSON:
```bash
pytest --tb=short 2>&1 | grep -E "^(FAILED|E )" | sort | uniq -c | sort -rn
```
