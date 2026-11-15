# TDD Prune Guide

## Overview

Prune is a separate pass after the TDD cycle (Red → Green → Refactor), never by the author: `kinhin tag` + `/kinhin:prune` interactively, once per deliverable before merge, as its own commit; the milestone's `test-prune` task in the runner. In the runner it runs once per milestone, as its own task (`workflow: test-prune`), after every other task of the milestone merged: see Independent Prune below.

Prune answers: "which tests were construction scaffolding and which earned permanent status?"

Reference: `@~/.claude/kinhin/spec/tdd/tdd-prune-spec.md` (lifecycle tags, graduation, criteria, collapse, retention) and `tdd-lifecycle-tags.md` (marker syntax per language).

**CLI**: `kinhin prune` automates this entire procedure. Use `kinhin prune` for a dry run (shows the plan), `kinhin prune --apply` to execute. Without the CLI, follow the manual steps below.


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

Every untagged test, and every permanent-tagged test whose `ref` is missing or does not resolve, must be classified with a one-line justification (tags are hints; see Independent Prune):

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
- Are not cited as evidence in a delivered task's `completeness.md`

### Step 5: Run the Suite

Run the suite (`make test` / `make check`): green after the deletions. A red test after the prune means a deleted test was load-bearing — restore it as a promoted `@decision` or a collapsed row.

### Step 6: PR Census Line

End the PR description with:

```
Tests: written 38 / pruned 29 / survivors: 5 decision, 2 contract, 1 incident, 1 table-driven (9 rows)
```


## Independent Prune (runner `test-prune`)

The instrument appends one prune task to each new milestone, depending on every other task; the runner executes it like any task, on the review pin, in a session separate from the authors. Unit tests are construction scaffolding for the code assistant: after delivery, only tests pinned to something stable remain.

**Keep** (and say which, in the report):

| Category | What it pins | Tag it carries |
|---|---|---|
| `contract` | a boundary another party reads: schema, external tool format, CLI output, wire format | `@contract(ref=)` |
| `regression` | a real bug that happened (incident, ticket, a fix commit) | `@incident(ref=)` |
| `invariant` | a property of a pure function that holds for all inputs | `@decision` (property) |
| `decision` | a documented choice whose ref resolves | `@decision(ref=)` |
| `evidence` | cited by a delivered task's `completeness.md` | unchanged |

**Remove or collapse** everything else: shape assertions (mock call counts, private state), subsets of survivors, scenario-matrix rows (collapse into one table-driven test).

Tags are hints. Legacy suites were tagged under construction pressure (one counted 1356 `decision` against 33 `scaffold`): a permanent tag without a resolving `ref` gets judged like an untagged test. New tests arrive untagged: the author does not tag, so the classification happens here.

**Safety net:**
1. The suite stays green: `make test` and `make check` for every component.
2. Evidence preservation: every test a delivered task cited as evidence still exists, same file, same name. The runner checks it before the reviewer.
3. The owner's coverage policy, when the owner has one (`owners.yaml coverage`): a gate on constrains how much can go.
4. A reviewer in a separate session judges every deletion against `prune-report.md`.

**`prune-report.md`** (task docs dir): one row per test the milestone's tasks added or changed:

```markdown
| test | file | category | action | reason |
|---|---|---|---|---|
| test_output_matches_schema_v2 | tests/test_cli.py | contract | keep | review_v1 schema consumed by app |
| test_parse_valid_json | tests/test_parse.py | scaffold | collapse | row `json` of test_parse_formats |
| test_calls_helper_once | tests/test_flow.py | scaffold | remove | mock call count, shape only |
```

`action`: keep | remove | collapse. End with the census line (Step 6).


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

**Step 5 — Run the suite:** green.

**Step 6 — Census line:**
```
Tests: written 38 / pruned 28 / survivors: 6 decision, 2 contract, 1 incident, 1 table-driven (5 rows)
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
Is the test cited as evidence in a delivered task's completeness.md?
├── YES → KEEP. Never delete or rename it.
│
Does it carry a permanent tag whose ref resolves to a live authority?
├── YES → it's permanent. Keep it.
│
└── NO (scaffold, untagged, or a permanent tag without a resolving ref)
    │
    Does it test implementation shape?
    │     (mock call counts, argument order, private state,
    │      render tree, things that change with any refactor)
    │
    ├── YES → DELETE. It tests mechanism, not behavior.
    │
    └── NO → Is it a subset of a surviving test?
        │     (the surviving test covers the same branch
        │      and asserts the same outcome)
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

**Rule**: never prune a contract test whose party exists: a schema, a consumer, a format someone reads. Its authority is external. A `@contract` tag naming no party is a hint, judged like any other.

### Mistake 2: Collapsing Away a Boundary Row

When collapsing N scaffolds into a table, it's tempting to keep only the "normal" case. Boundary rows (zero, max, empty, one-off-by-one) are where bugs live. Keep at least: one normal, one zero/empty, one boundary, one error.

### Mistake 3: Re-Tagging Permanent to Scaffold

An assistant under "execute to completion" pressure may re-tag a `@decision` test as `@scaffold` so it can be edited or deleted without triggering the approval gate. This is the prune equivalent of the "mirror test" anti-pattern.

**Rule**: never downgrade a permanent tag. If the decision changed, update the reason and the ref — don't erase the tag.

### Mistake 4: Pruning Without a Safety Net

Deleting tests on judgment alone ("this one is obviously redundant") skips the objective check. The net is deterministic: the suite stays green, evidence-cited tests survive, the owner's coverage policy holds, and a second reader (the reviewer) judges each deletion.


## References

- Prune Spec (lifecycle tags, graduation, criteria, retention, checklist): `@~/.claude/kinhin/spec/tdd/tdd-prune-spec.md`
- Marker syntax per language: `@~/.claude/kinhin/context/examples/tdd-lifecycle-tags.md`
