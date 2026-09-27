# TDD Prune Guide

## Overview

Prune is the fourth phase of the TDD cycle: Red → Green → Refactor → **Prune**. It runs once per deliverable, before merge, as its own commit.

Prune answers: "which tests were construction scaffolding and which earned permanent status?"

Reference: `@~/.claude/kinhin/spec/tdd/tdd-spec.md` → Test Lifecycle.

**CLI**: `kinhin prune` automates this entire procedure. Use `kinhin prune` for a dry run (shows the plan), `kinhin prune --apply` to execute, `kinhin prune --apply --verify` to execute and run mutation parity. Without the CLI, follow the manual steps below.


## Procedure

### Step 1: Tag Census

Use `kinhin census` for a quick count, or `kinhin audit` for the full per-test report. Manually:

```bash
# Python
pytest --collect-only -q | wc -l                     # total
pytest -m scaffold --collect-only -q | wc -l          # scaffold count
pytest -m decision --collect-only -q | wc -l          # decision count
pytest -m contract --collect-only -q | wc -l          # contract count
pytest -m incident --collect-only -q | wc -l          # incident count

# Rust
cargo test -- --list 2>/dev/null | wc -l              # total
cargo test scaffold:: -- --list 2>/dev/null | wc -l   # scaffold count

# Java
grep -rn '@Tag("scaffold")' src/test/ | wc -l
grep -rn '@Tag("decision")' src/test/ | wc -l

# TypeScript
find . -name "*.scaffold.test.ts" | wc -l             # scaffold files
find . -name "*.test.ts" -not -name "*.scaffold.*" | wc -l  # permanent files
```

Record: `total`, `scaffold`, `decision`, `contract`, `incident`, `untagged`.

### Step 2: Classify Untagged Tests

Every untagged test must be classified with a one-line justification:

- If it encodes a decision → tag `@decision(reason=...)` and keep
- If it guards a silent failure → tag `@decision` or `@incident` and keep
- If it pins a boundary → tag `@contract(party=...)` and keep
- If none of the above → it was scaffold all along → tag `@scaffold` (then it enters the delete set)

**Rule**: untagged tests in a legacy codebase are "unknown", never auto-delete. Classify first.

### Step 3: Collapse Scaffold

For each function with multiple scaffold tests:

1. Identify equivalence classes (inputs that exercise the same branch)
2. Keep one representative per class + boundary values + error cases
3. Merge into a single table-driven test
4. Tag the collapsed test `@decision` with a reason

**Collapse rules:**
- N tests over one function → 1 parametrized test with M rows (M < N)
- Each row represents one equivalence class OR one boundary OR one error
- Row IDs must appear in the assertion message (for debugging)
- The collapsed test inherits the strongest tag among its sources

### Step 4: Delete Remaining Scaffold

After collapse, delete tests that:
- Are still tagged `@scaffold`
- Were not collapsed into a surviving test
- Have no unique mutation-killing power (verified in Step 5)

### Step 5: Mutation Parity

Run mutation testing on changed files to verify nothing load-bearing was pruned.

**With the CLI:** `kinhin gate --save-baseline pre.json` (before prune), then `kinhin gate --baseline pre.json` (after).

**Manual protocol:**
1. Record the mutation score BEFORE prune: `K₀ = set of killed mutants`
2. Apply the prune (delete + collapse)
3. Re-run mutation: `K₁ = set of killed mutants`
4. Verify: `K₁ ⊇ K₀`

If a mutant was killed only by a pruned test:
- That test was load-bearing
- Either restore it (promoted to `@decision`) or write a new decision test that kills the mutant

```bash
# Python
mutmut run --paths-to-mutate=src/changed_module.py

# Rust
cargo mutants --in-diff HEAD~1

# Java
mvn org.pitest:pitest-maven:mutationCoverage -DtargetClasses="com.example.changed.*"

# TypeScript
npx stryker run --mutate "src/changed/**/*.ts"
```

### Step 6: PR Census Line

End the PR description with:

```
Tests: written 38 / pruned 29 / survivors: 5 decision, 2 contract, 1 incident, 1 table-driven (9 rows)
Mutation parity: K₁ ⊇ K₀ ✓
```


## Worked Example

### Before Prune

A `parseConfig` function with 38 tests generated during construction:

| Tag | Count | Examples |
|---|---|---|
| `@scaffold` | 30 | `test_parse_valid_json`, `test_parse_valid_yaml`, `test_parse_returns_dict`, `test_parse_has_version_key`, 4 type-check tests, 12 duplicate-scenario tests, 10 input-permutation tests |
| `@decision` | 5 | `test_yaml_takes_precedence_over_json` (reason: config resolution order), `test_env_vars_override_file` (reason: 12-factor), 3 more |
| `@contract` | 2 | `test_output_matches_schema_v2` (party: downstream consumer), `test_error_includes_line_number` (party: IDE integration) |
| `@incident` | 1 | `test_handles_utf8_bom` (ref: ISSUE-42, config failed silently on Windows) |

### Prune Process

**Step 1 — Census:** 38 total, 30 scaffold, 5 decision, 2 contract, 1 incident, 0 untagged.

**Step 2 — Classify untagged:** none.

**Step 3 — Collapse scaffold:**
- 12 duplicate-scenario tests → collapsed: same equivalence class as existing decision tests
- 10 input-permutation tests → 1 table-driven test with 5 rows (one per input format: json, yaml, toml, env, empty)
- 4 type-check tests → deleted (redundant under mypy strict)
- 4 remaining scaffolds → 2 deleted (subset of survivors), 2 promoted (one guarded a silent failure: malformed YAML returns empty dict instead of raising)

**Step 4 — Delete:** 28 tests removed.

**Step 5 — Mutation parity:**
```
Before prune: 47 mutants killed, 3 survived
After prune:  47 mutants killed, 3 survived
K₁ ⊇ K₀ ✓
```

**Step 6 — Census line:**
```
Tests: written 38 / pruned 28 / survivors: 6 decision, 2 contract, 1 incident, 1 table-driven (5 rows)
Mutation parity: K₁ ⊇ K₀ ✓
```

### After Prune

| Tag | Count | What survived |
|---|---|---|
| `@decision` | 6 | 5 original + 1 promoted from scaffold (silent failure guard) |
| `@contract` | 2 | unchanged |
| `@incident` | 1 | unchanged |
| table-driven | 1 (5 rows) | collapsed from 10 input-permutation scaffolds |
| **Total** | **10** | **from 38** |


## Decision Tree

```
Is the test tagged @scaffold (or untagged)?
├── NO → it's permanent. Keep it.
│
└── YES → Does it test implementation shape?
    │     (mock call counts, argument order, private state,
    │      render tree, things that change with any refactor)
    │
    ├── YES → DELETE. It tests mechanism, not behavior.
    │
    └── NO → Is it a subset of a surviving test?
        │     (the surviving test covers the same branch
        │      AND kills the same mutants)
        │
        ├── YES → DELETE. The survivor carries this.
        │
        └── NO → Does the scenario guard a silent failure?
            │     (a wrong implementation would pass tsc,
            │      pass other tests, and ship unnoticed)
            │
            ├── YES → PROMOTE to @decision with a reason.
            │
            └── NO → Can it collapse into a table-driven survivor?
                │
                ├── YES → COLLAPSE. Merge as a row in the table.
                │
                └── NO → DELETE. Nobody can name what it prevents.
```


## Common Mistakes

### Mistake 1: Pruning @contract Tests

A contract test "looks redundant" because the implementation obviously matches the spec. But the test's purpose is not to verify today's implementation — it's to detect TOMORROW's accidental change. A contract test is a tripwire, not a proof.

**Rule**: never prune a `@contract` test. Its authority is external.

### Mistake 2: Collapsing Away a Boundary Row

When collapsing N scaffolds into a table, it's tempting to keep only the "normal" case. Boundary rows (zero, max, empty, one-off-by-one) are where bugs live. Keep at least: one normal, one zero/empty, one boundary, one error.

### Mistake 3: Re-Tagging Permanent to Scaffold

An assistant under "execute to completion" pressure may re-tag a `@decision` test as `@scaffold` so it can be edited or deleted without triggering the approval gate. This is the prune equivalent of the "mirror test" anti-pattern.

**Rule**: never downgrade a permanent tag. If the decision changed, update the reason and the ref — don't erase the tag.

### Mistake 4: Pruning Without Mutation Verification

Deleting tests based on judgment alone ("this one is obviously redundant") skips the objective check. Run mutation parity. If a pruned test was the only thing killing a mutant, your judgment was wrong. Restore it.


## References

- Test Lifecycle: `@~/.claude/kinhin/spec/tdd/tdd-spec.md`
- Lifecycle Tags: `@~/.claude/kinhin/context/examples/tdd-lifecycle-tags.md`
- TDD Checklist: `@~/.claude/kinhin/context/checklists/tdd-checklist.md`
