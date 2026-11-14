# TDD Runner Specification

How the test runner is configured for LLM-driven TDD. Language-specific flags live in each `<lang>-tdd-runner-spec.md` or `<lang>-tdd-spec.md`; run modes and clustering in `tdd-runner-guide.md`.

## Runner Contract

**Requirement**: The test runner must satisfy six language-neutral requirements. These are not recommendations — they are the conditions under which TDD feedback loops function for LLM-driven development.

### 1. No Bail
Never stop at first failure. The assistant batch-fixes. This is the corollary of Batch Processing Over Sequential (`tdd-spec.md`): generating tests in batches is pointless if the runner reports failures one at a time.

### 2. Parallel by Default
Run tests in parallel with per-test isolation. Tests that fail under parallelism but pass serially have a shared-state bug — that bug is the finding, not the parallelism.

### 3. Randomized Order
Randomize test execution order every run. Print and persist the seed. A test that passes only in a fixed order is an **isolation-dependent failure** — a bug to fix, not a flake to retry.

### 4. Three Verdicts
Classify every failure into one of three categories, never two:

| Verdict | Meaning | Action |
|---------|---------|--------|
| `deterministic-fail` | Fails every run, in isolation and in the suite | Fix the code or the test |
| `isolation-dependent` | Passes in isolation, fails in the suite or under a different seed | Shared state bug — fix the test |
| `flaky` | Mixed results across isolated reruns | Quarantine with a ticket — never retry-until-green |

**Three-run protocol**: On a failure, rerun isolated ×3. Still fails = `deterministic-fail`. Passes isolated → rerun in original order with original seed: fails = `isolation-dependent`, passes = `flaky`.

**Retry-until-green is forbidden** outside diagnostic mode. It hides exactly the information the assistant needs.

### 5. Changed-Set Selection
In the inner loop (fix → run → fix), run only tests whose transitive import graph touches the changed files. At the gate (before PR), run the full suite. Never ship on selection alone.

### 6. Machine-Readable Output
Emit failures as structured data: test ID, assertion operands as values (not pre-rendered diffs), project-owned stack frames, exception type.

**Cause clustering**: Group failures by `(innermost project-owned frame, exception type)`. If 40 tests fail because one fixture broke, the assistant sees "1 root cause, 40 dependents" — not 40 tracebacks. Fix one cluster at a time, largest first, then rerun the affected set.

Output token budget: cap per cluster, strip passing tests. The assistant's context is the scarce resource; the runner should treat it like one.
