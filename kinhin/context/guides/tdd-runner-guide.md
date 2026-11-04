# TDD Runner Guide

## Overview

How to use test runners for code-assistant batch workflows. The universal spec defines a 6-point runner contract (`tdd-spec.md` → Runner Contract); per-language configuration is in each `<lang>-tdd-spec`. This guide covers the run modes and cause clustering. CLI: `tdd-spec.md` → CLI Tools.


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

### Full Mode (Pre-PR Gate)

**When**: before opening a PR, after all changes are complete.

**Configuration:**
- Run the ENTIRE permanent suite (excluding scaffolds)
- Parallel execution
- Randomized order with seed printed
- No bail

**Purpose**: catch cross-module regressions. Selection NEVER replaces this.

### Diagnostic Mode (Flake Isolation)

**When**: a test fails intermittently and needs classification.

**Configuration:**
- Serial execution (single worker)
- Fixed, reproducible order
- Only the suspect test(s)

**Purpose**: determine if the failure is deterministic, isolation-dependent, or flaky.

Commands for loop, full and diagnostic runs live in each `<lang>-tdd-spec` → Runner Configuration. Two the language specs do not carry:

```bash
# Java — diagnostic (serial, one test)
mvn test -Dtest=SuspectTest#methodName -Djunit.jupiter.execution.parallel.enabled=false

# TypeScript (Jest) — diagnostic, then rerun with the original seed
jest --testNamePattern "suspect test" --runInBand
jest --no-bail --randomize --seed=<SEED>
```

Flake classification (three-run protocol, three verdicts, no retry-until-green): `tdd-spec.md` → Runner Contract #4.


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

