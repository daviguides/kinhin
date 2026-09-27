# TDD Runner Guide

## Overview

How to configure and use test runners for code-assistant batch workflows. The universal spec defines a 6-point runner contract (`@~/.claude/kinhin/spec/tdd/tdd-spec.md` → Runner Contract). This guide maps the contract to concrete tools and recipes.


## Three Run Modes

Every Kinhin workflow uses three modes at different points:

### Loop Mode (Inner Development Loop)

**When**: during active development, after each code change.

**Configuration:**
- Run only tests affected by changed files (changed-set selection)
- Parallel execution
- Randomized order
- No bail — report ALL failures

**Purpose**: fast feedback. Typically < 60 seconds.

```bash
# Python
pytest --testmon -n auto -p randomly --maxfail=0

# Rust
cargo nextest run -p <changed_crate> --no-fail-fast

# Java (Gradle)
gradle :changed-module:test --continue

# TypeScript (Jest)
jest --findRelatedTests src/changed-file.ts --no-bail --maxWorkers=50%

# TypeScript (Vitest)
vitest run --changed --reporter=json
```

### Full Mode (Pre-PR Gate)

**When**: before opening a PR, after all changes are complete.

**Configuration:**
- Run the ENTIRE permanent suite (excluding scaffolds)
- Parallel execution
- Randomized order with seed printed
- No bail

**Purpose**: catch cross-module regressions. Selection NEVER replaces this.

```bash
# Python
pytest -m "not scaffold" -n auto -p randomly --maxfail=0

# Rust
cargo nextest run --filter-expr 'not test(scaffold::)' --profile ci

# Java
mvn test  # with <excludedGroups>scaffold</excludedGroups>

# TypeScript (Jest)
jest --testPathIgnorePatterns scaffold --no-bail --randomize

# TypeScript (Vitest)
vitest run --exclude "**/**.scaffold.test.ts"
```

### Diagnostic Mode (Flake Isolation)

**When**: a test fails intermittently and needs classification.

**Configuration:**
- Serial execution (single worker)
- Fixed, reproducible order
- Only the suspect test(s)

**Purpose**: determine if the failure is deterministic, isolation-dependent, or flaky.

```bash
# Python
pytest tests/test_suspect.py::test_name -p no:randomly -x

# Rust
cargo nextest run --filter-expr 'test(suspect_test)' -j 1

# Java
mvn test -Dtest=SuspectTest#methodName -Djunit.jupiter.execution.parallel.enabled=false

# TypeScript
jest --testNamePattern "suspect test" --runInBand
```


## Three-Run Flake Protocol

When a test fails in the full run, classify it before acting:

```
STEP 1: Rerun the failure ISOLATED (fresh process, alone)
        Run it 3 times.
        ┌─ All 3 fail    → DETERMINISTIC-FAIL
        │                   Action: fix the code or the test.
        │
        └─ Any pass       → Go to Step 2.

STEP 2: Rerun WITH the original order and seed
        (reproduces the original environment)
        ┌─ Fails          → ISOLATION-DEPENDENT
        │                   Action: shared state bug. Fix NOW.
        │                   This is not a flake — it's a real bug
        │                   that only shows under concurrency.
        │
        └─ Passes         → FLAKY
                            Action: quarantine with a ticket.
                            Never retry-until-green.
                            Never mark as "known flaky" and ignore.
```

### Concrete Commands

**Python:**
```bash
# Step 1: isolated, 3 times
pytest tests/test_file.py::test_name --count=3 -p no:randomly -p no:xdist

# Step 2: original environment
pytest -p randomly --randomly-seed=<SEED> -n auto
```

**Rust:**
```bash
# Step 1: isolated
cargo nextest run --filter-expr 'test(the_test)' -j 1 --retries 2

# Step 2: with original partition
cargo nextest run --no-fail-fast
```

**TypeScript (Jest):**
```bash
# Step 1: isolated
jest --testNamePattern "the test" --runInBand --forceExit

# Step 2: original environment
jest --no-bail --randomize --seed=<SEED>
```

### Why Not Retry-Until-Green

Auto-retry plugins (`pytest-rerunfailures` with `--reruns`, Jest `--retryTimes`) hide exactly the information the assistant needs. A test that passes on retry 2 of 3 is either isolation-dependent (a real bug) or flaky (a test bug). Retrying to green classifies both as "passed" and the bug ships.

Use retry plugins ONLY inside the three-run protocol, never as a CI gate.


## Cause Clustering

When multiple tests fail, don't fix them one at a time. Cluster first.

### The Algorithm

1. For each failure, extract:
   - **Exception type** (e.g., `AssertionError`, `ConnectionRefused`, `TypeError`)
   - **Innermost project-owned stack frame** (first frame in YOUR code, not library code)

2. **Clustering key** = `(exception_type, normalized_frame)`
   - Normalize: strip line numbers, strip argument values
   - Example: `(AssertionError, src/discount.py:calculate_discount)` and `(AssertionError, src/discount.py:calculate_discount)` are the same cluster even if the assertion message differs

3. **Group failures by key**

4. **Fix the largest cluster first** — it likely has one root cause

5. **Rerun only the affected set** — tests in the fixed cluster + tests that import the changed module

### Example

40 test failures after a refactor:

```
Cluster 1 (32 failures): TypeError at src/config.py:load
  → load() signature changed, 32 callers broke
  → Fix: update callers. One fix, 32 tests pass.

Cluster 2 (6 failures): AssertionError at src/discount.py:calculate
  → Business logic regression
  → Fix: restore the discount cap. One fix, 6 tests pass.

Cluster 3 (2 failures): ConnectionRefused at src/db.py:connect
  → Test database not running
  → Fix: start the database. Infrastructure, not code.
```

3 fixes instead of 40. This is why the runner contract requires structured output.

### Manual Clustering (No JSON Available)

When the runner only provides text output:

1. Pipe failures through a simple grouper:
   ```bash
   # Python: extract the last project frame from each traceback
   pytest --tb=short 2>&1 | grep -E "^(FAILED|E )" | sort | uniq -c | sort -rn
   ```

2. Read the output. Same file+function appearing in many failures = one cluster.

3. Fix the top entry. Rerun.


## Structured Output Per Language

### Python — pytest-json-report

```bash
pytest --json-report --json-report-file=results.json
```

Key fields: `.tests[].outcome`, `.tests[].call.crash.path`, `.tests[].call.crash.lineno`, `.tests[].call.crash.message`.

### Rust — nextest JUnit XML

```toml
# .config/nextest.toml
[profile.ci.junit]
path = "target/nextest/ci/junit.xml"
```

Key fields: `<testcase name>`, `<failure message>`, `<failure type>`.

### Java — Surefire XML

Located at `target/surefire-reports/TEST-*.xml`.

Key fields: `<testcase classname="" name="">`, `<failure type="" message="">`, `<error>`.

### TypeScript — Jest JSON

```bash
jest --json --outputFile=results.json
```

Key fields: `.testResults[].testResults[].status`, `.testResults[].testResults[].failureMessages`.

### TypeScript — Vitest JSON

```bash
vitest run --reporter=json --outputFile=results.json
```

Key fields: `.testResults[].assertionResults[].status`, `.testResults[].assertionResults[].failureMessages`.


## References

- Runner Contract: `@~/.claude/kinhin/spec/tdd/tdd-spec.md`
- Anti-Patterns #20-23 (runner): `@~/.claude/kinhin/context/examples/tdd-anti-patterns.md`
- Per-language configs: see each language's TDD spec
