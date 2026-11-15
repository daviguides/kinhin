# TDD Prune Specification

The prune pass: what survives construction and what dies. This file is for whoever classifies and prunes — a human running `/kinhin:prune` as a separate step, or the runner's `test-prune` task, once per milestone. The implementing author never tags, prunes or collapses tests; failure attribution lives in `tdd-spec.md`.

## Lifecycle Tags

Tags are assigned by the classification pass (`kinhin tag`, or the pruner), never by the author while writing. A test with no tag is a construction test: `scaffold` until the classification says otherwise. A source the author cited in a docstring is evidence for the classification, not a tag.

**Temporary tags** (die at prune):

| Tag | Authority | Lifetime |
|-----|-----------|----------|
| `scaffold` | None — construction verification | Deleted or collapsed at prune |
| `characterization` | None — golden-master snapshot for a refactoring | Deleted after refactor completes |

**Permanent tags** (survive prune, require a `ref`):

| Tag | Authority | Ref points to |
|-----|-----------|---------------|
| `decision(ref=)` | A documented choice | ADR, doc section, ticket, code comment with rationale |
| `contract(ref=)` | An external consumer | API doc, wire format, schema, iOS client contract |
| `incident(ref=)` | Production behavior that failed | Incident ticket, postmortem, date of failure |

**Corollary**: A test with no authority has no right to outlive construction.

### Graduation Criteria

A test earns a permanent tag if **any one** holds:

1. It encodes a **decision** not visible from the mechanism — the "why", not the "how".
2. It guards a failure that would be **silent in production** — wrong value returned, no exception raised, data quietly corrupted.
3. It pins a **boundary** another party depends on — wire format, public API, serialized payload order.
4. The scenario was **expensive to discover** — a production incident, hours of debugging, a measured false positive.

### Explicit Rulings

| Test kind | Tag | Rationale |
|-----------|-----|-----------|
| Property test (invariant) | `decision` | Invariants are decisions — otherwise they get pruned as "re-derivable" |
| Integration test crossing system boundary | `contract` | The boundary is the authority |
| Bugfix reproduction test | `incident` | The incident is the authority |
| Characterization test for refactoring | `scaffold` | Golden-master dies when the refactor completes |
| Scenario matrix exploration rows | `scaffold` | Construction verification, not regression |

The marker syntax per language (pytest marker, Rust `mod scaffold` and `// kinhin:` comment, JUnit `@Tag`, TypeScript file suffix) is in `tdd-lifecycle-tags.md`, with the commands that read it.

## Prune Criteria

A test is pruned if **all** hold:

1. It asserts **implementation shape** — mock call counts, argument matching, internal call order, private state.
2. Its assertions are a **subset** of a surviving test's.
3. A wrong implementation would **fail loudly** elsewhere — type error, crash, obvious failing test.
4. Nobody can name the **production failure** it prevents in one sentence.

## Independent Prune

A tag on an existing test is a claim, often written under construction pressure (legacy suites tag nearly everything permanent). The pruner treats it as a hint, not a verdict:

1. A permanent tag (`decision`, `contract`, `incident`) whose `ref` resolves to a live authority (an ADR or doc section that exists, a schema or consumer that exists, an incident ticket) keeps the test.
2. A permanent tag with no `ref`, or a `ref` that does not resolve, is re-judged against the Graduation Criteria (above). Meeting none of them, it is scaffold.
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
**Requirement**: 0 untagged tests after the prune pass. The classification tags every test; each one is `scaffold` (and removed or collapsed) or carries a permanent tag with a `ref`.

### Retention Quality
- Every surviving test has a permanent tag with ref?
- Census line in the PR or prune report?
- No mirror tests?

### Anti-Pattern: Coverage as Retention Metric

**Problem**: Using branch or line coverage to decide which tests to keep after pruning.

**Why It's Bad**:
- Coverage rewards keeping shape-tests that touch trivial branches
- A test asserting `mock.call_count == 1` covers the call site — 100% branch, 0% verification
- Coverage measures what was EXECUTED, not what was VERIFIED
- Actively opposes pruning: removing scaffold drops coverage, triggering "add tests back"

**Bad**:
```
# Post-prune review
"Coverage dropped from 94% to 87% after pruning scaffolds.
 Restoring 12 mock-assertion tests to meet the 90% gate."
```

**Good**:
```
# Post-prune review
"Suite green; every @decision/@contract/@incident with a live ref survived;
 no test cited as completeness evidence removed.
 Branch coverage dropped from 94% to 87% (expected, scaffold removal)."
```

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
