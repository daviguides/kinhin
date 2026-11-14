# TDD Prune Specification

The prune pass: what survives construction and what dies. Tags at birth, graduation criteria and attribution live in `tdd-spec.md`; this file is for whoever prunes — a human at the end of a branch (`/kinhin:prune`), or the runner's `test-prune` task, once per milestone. The implementing author tags tests and never prunes.

## Prune Criteria

A test is pruned if **all** hold:

1. It asserts **implementation shape** — mock call counts, argument matching, internal call order, private state.
2. Its assertions are a **subset** of a surviving test's.
3. A wrong implementation would **fail loudly** elsewhere — type error, crash, obvious failing test.
4. Nobody can name the **production failure** it prevents in one sentence.

## Independent Prune

A tag is the author's claim, written under construction pressure. When the pruner is not the author (the runner's `test-prune` workflow, one prune per milestone), the tag is a hint, not a verdict:

1. A permanent tag (`decision`, `contract`, `incident`) whose `ref` resolves to a live authority (an ADR or doc section that exists, a schema or consumer that exists, an incident ticket) keeps the test.
2. A permanent tag with no `ref`, or a `ref` that does not resolve, is re-judged against the Graduation Criteria (`tdd-spec.md`). Meeting none of them, it is scaffold.
3. An untagged test is judged the same way.
4. A test cited as evidence in a delivered task's `completeness.md` is never deleted or renamed: the delivery's proof points at it.
5. What survives is what is pinned to something stable: a boundary contract (schema, external tool format, CLI), a regression of a real bug, an invariant of a pure function, a decision with a live ref.

Never re-tag a test to make its deletion pass: record the reclassification and its reason in the prune report instead.

The safety net is deterministic plus a second reader: the suite stays green, evidence-cited tests survive, and a reviewer in a separate session judges every deletion.

## Collapse Pattern

N scaffold rows over one function collapse into **one table-driven test** tagged `@decision`:

1. Keep one representative input per equivalence class.
2. Keep each boundary value.
3. Keep each error class.
4. The collapsed test inherits the strongest tag among its sources.
5. Row identifiers appear in assertion messages (ties to Anti-Pattern #8).

## Retention Requirements (Measured After PRUNE)

### Permanent Test Survival
**Requirement**: 100% of `@decision`, `@contract`, and `@incident` tests whose `ref` resolves survive the prune, and 100% of tests cited as completeness evidence (see Independent Prune).

### Tag Completeness
**Requirement**: 0 untagged tests at merge. Every test is explicitly `@scaffold` (and removed) or carries a permanent tag with a `ref`.

### Retention Quality
- Every surviving test has a permanent tag with ref?
- Census line in the PR or prune report?
- No mirror tests?

## Prune Checklist

**Before opening the PR** (`kinhin prune --apply` automates the census, collapse and deletion):

- [ ] Tag census taken (all tests enumerated by tag)
- [ ] Every untagged test classified with one-line justification
- [ ] All `@scaffold` tests collapsed into table-driven `@decision` tests or deleted
- [ ] All `@characterization` tests deleted or graduated with stated reason
- [ ] Each surviving permanent test has a `ref` or documented reason
- [ ] Suite green after the deletions; no test cited as completeness evidence removed
- [ ] PR census line written: `+written N / −pruned M / survivors: X decision, Y contract, Z incident`
- [ ] Prune changes are separate from implementation (in the runner, the runner commits; the agent never runs git)

## CLI Tools

The Kinhin CLI automates the prune. All commands are optional — every operation can be performed manually. When the CLI is installed, prefer it for consistency.

| Concept | CLI Command | Manual Equivalent |
|---|---|---|
| Tag census and ref validation | `kinhin audit` | Grep for markers, check ref paths |
| Tag count by lifecycle | `kinhin census` | Count markers manually |
| Runner Contract execution | `kinhin run --mode loop\|full\|diagnostic` | Configure runner flags per language |
| Install runner dependencies | `kinhin setup --install` | Install plugins per language manually |
| Auto-tag untagged tests | `kinhin tag` | Classify each test manually |
| Prune pipeline | `kinhin prune` | Census → classify → collapse → delete → run the suite |

Install: `cargo install --path cli` from the Kinhin repo. Agent commands (`tag`, `prune`) require the Claude Code CLI.
