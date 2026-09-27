# TDD Specification (LLM-Optimized)

## Authority

Code and test are a closed system: neither validates the other without a reference point outside both. Every test either names an authority outside that system — a documented decision, an external contract, a production incident — or it does not. The tag on a test is a pointer to that authority, not a label of intent.

## Philosophy

Test-Driven Development optimized for Large Language Models leverages LLM strengths while mitigating weaknesses:

**LLM Strengths:**
- Parallel thinking and batch processing
- Template consistency and pattern following
- Comprehensive scenario generation
- Property-based validation

**LLM Weaknesses (Mitigated):**
- Sequential limitations → Batch processing
- Hallucinations → Anti-hallucination patterns
- Context fragmentation → Template-driven consistency
- Test accumulation → Lifecycle tagging
- Serial feedback → Runner contract

## Core Principles

### Batch Processing Over Sequential
**Requirement**: Generate multiple related tests simultaneously rather than one at a time.

**Benefits:**
- Faster development cycle
- Consistent patterns across test suite
- Better context utilization
- Reduced cognitive overhead

**Implementation:**
- Generate entire test categories at once
- Implement functions in logical groups
- Refactor modules for consistency

### Template-Driven Consistency
**Requirement**: Use repeatable patterns for all test structures.

**Benefits:**
- Reduces hallucination risk
- Maintains code quality
- Simplifies review process
- Enables rapid test generation

**Implementation:**
- Establish standard test templates
- Maintain consistent naming conventions
- Use uniform assertion patterns
- Follow structured test organization

### Property-Based Validation
**Requirement**: Focus on invariants that must always hold.

**Benefits:**
- Catches edge cases automatically
- Validates business rules comprehensively
- Acts as anti-hallucination barrier
- Ensures constraint compliance

**Implementation:**
- Define invariants explicitly
- Test constraints systematically
- Validate cross-function consistency
- Use property-based testing tools

### Lifecycle Over Accumulation
**Requirement**: Every test is born with a lifecycle tag. The default tag is `@scaffold`. The burden of proof is on **keeping** a test, not on deleting it.

**Benefits:**
- Prevents test bloat from surviving construction
- Forces explicit reasoning about each test's purpose
- Makes pruning mechanical rather than judgmental
- Separates construction-time verification from long-term regression

**Implementation:**
- Tag every test at birth (untagged = scaffold)
- Prune scaffolds before merge
- Promote tests only with a stated authority
- Collapse redundant scaffolds into table-driven decision tests

## Test Organization Requirements

### Test Suite Structure
**Requirement**: Organize tests by purpose, not by file structure.

**Mandatory Categories:**
1. **Unit tests** - Core logic validation
2. **Integration tests** - Component interaction
3. **Edge case tests** - Boundary conditions
4. **Error handling tests** - Failure scenarios
5. **Property tests** - Invariant validation
6. **Performance tests** - Critical path validation (when applicable)

### Test File Organization
**Requirement**: Group related tests in dedicated files.

**Structure:**
```
tests/
├── test_core_logic/      # Business logic tests
├── test_integrations/    # Component interaction tests
├── test_edge_cases/      # Boundary condition tests
└── test_error_handling/  # Failure scenario tests
```

## Naming Conventions

### Test Function Naming
**Requirement**: All test function names must follow pattern.

**Pattern**: `test_{component}_{scenario}`

**Examples:**
- `test_calculator_adds_positive_numbers`
- `test_validator_rejects_invalid_email`
- `test_processor_handles_empty_input`

### Integration Test Naming
**Requirement**: Clearly indicate components being tested.

**Pattern**: `test_{component_a}_integrates_with_{component_b}`

**Examples:**
- `test_user_service_integrates_with_database`
- `test_api_client_integrates_with_auth_service`

### Error Test Naming
**Requirement**: Specify error type being tested.

**Pattern**: `test_{component}_handles_{error_type}`

**Examples:**
- `test_parser_handles_malformed_input`
- `test_api_handles_network_timeout`

## Scenario Matrix Requirements

### Matrix Definition
**Requirement**: Define comprehensive test scenarios before implementation.

**Structure:**
```
Component:
  Valid Cases:    [happy path scenarios]
  Edge Cases:     [boundary conditions]
  Invalid Cases:  [error conditions]
  Performance:    [load requirements] (optional)
```

### Matrix Coverage
**Requirement**: All matrix cells must have corresponding tests during construction.

**Coverage:**
- 100% of scenario matrix must be tested at GREEN
- All combinations of valid/edge/invalid scenarios
- All critical performance paths
- The matrix is the permanent artifact; the tests derived from it may be pruned

## Test Structure Pattern

### Arrange-Act-Assert (AAA)
**Requirement**: All tests must follow AAA pattern.

**Structure:**
1. **Arrange** (Given): Setup test data and preconditions
2. **Act** (When): Execute function or operation under test
3. **Assert** (Then): Validate results and side effects

**Benefit**: Clear test intent and easy debugging.

### Test Independence
**Requirement**: Tests must be independent and isolated.

**Rules:**
- No shared state between tests
- Each test creates its own fixtures
- Tests can run in any order
- No dependencies between tests

## 3-Question Gate at RED

**Requirement**: Before writing any assertion, answer three questions in order. The answer determines the verification rung and the tag.

### Question 1: Can the type system refuse the wrong state?

Discriminated unions, branded types, required parameters with no default, exhaustive switches. If the compiler rejects the invalid state, write no test for it. **Rung 1** — the authority is the compiler.

### Question 2: Is the rule about shape, not value?

"Every handler must be wrapped in Z." "Nothing outside folder F may import G." "All public functions must have type annotations." If the rule is structural, write an arch test or a structural invariant test. **Rung 2** — the authority is the codebase structure.

### Question 3: What is the authority, and where does it live?

- Named and linkable (ADR, doc section, ticket, API spec) → `@decision(ref=)` / `@contract(ref=)` / `@incident(ref=)`. **Rung 3.**
- Named but only in your head → write it down first, or accept **Rung 4** and say so.
- Not named → `@scaffold`. **Rung 5.** It dies at prune.

The output of the gate IS the tag. No extra ceremony.

## Anti-Hallucination Patterns

### Constraint Validation
**Requirement**: All business constraints must have explicit validation tests.

**Purpose**: Ensure constraints are never violated, preventing hallucinated behavior.

**Implementation:**
- Test each constraint independently
- Validate constraints across all scenarios
- Test constraint boundaries explicitly

### Cross-Function Consistency
**Requirement**: Related functions must maintain data consistency.

**Purpose**: Detect hallucinated behavior in function relationships.

**Implementation:**
- Test data flow between functions
- Validate consistency across function chains
- Ensure related functions maintain invariants

### Type Safety Validation
**Requirement**: All functions must maintain type consistency.

**Purpose**: Catch type-related hallucinations.

**Implementation:**
- All public functions must have type annotations
- Return types must be consistent
- Test validates type contracts

### Explicit Validation
**Requirement**: All assumptions must be tested explicitly.

**Purpose**: Prevent implicit hallucinated behavior.

**Implementation:**
- No implicit conversions in tests
- Explicit input validation
- Clear error messages
- Documented assumptions

## Test Lifecycle

### Tags at Birth

Every test receives a lifecycle tag when written. Untagged tests are `@scaffold` by default.

**Temporary tags** (die at prune):

| Tag | Authority | Lifetime |
|-----|-----------|----------|
| `scaffold` | None — the author's momentary belief during construction | Deleted or collapsed at prune |
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

### Prune Criteria

A test is pruned if **all** hold:

1. It asserts **implementation shape** — mock call counts, argument matching, internal call order, private state.
2. It is a **subset** of a surviving test's killed-mutant set.
3. A wrong implementation would **fail loudly** elsewhere — type error, crash, obvious failing test.
4. Nobody can name the **production failure** it prevents in one sentence.

### Explicit Rulings

| Test kind | Default tag | Rationale |
|-----------|-------------|-----------|
| Property test (invariant) | `@decision` | Invariants are decisions — otherwise they get pruned as "re-derivable" |
| Integration test crossing system boundary | `@contract` | The boundary is the authority |
| Bugfix reproduction test | `@incident` | The incident is the authority |
| Characterization test for refactoring | `@scaffold` | Golden-master dies when the refactor completes |
| Scenario matrix exploration rows | `@scaffold` | Construction verification, not regression |

### Collapse Pattern

N scaffold rows over one function collapse into **one table-driven test** tagged `@decision`:

1. Keep one representative input per equivalence class.
2. Keep each boundary value.
3. Keep each error class.
4. The collapsed test inherits the strongest tag among its sources.
5. Row identifiers appear in assertion messages (ties to Anti-Pattern #8).

### Mutation Parity Protocol

Verification that pruning removed nothing load-bearing:

1. Run mutation testing on changed files with the **full pre-prune suite**. Record the killed-mutant set K₀.
2. Execute the prune.
3. Rerun mutation testing with the **post-prune suite**. Record K₁.
4. **Require K₁ ⊇ K₀.** Any mutant killed only by a pruned test means that test was load-bearing — restore it as a promoted `@decision` or a collapsed row.

Scope mutation testing to changed files only. Never run full-repo mutation in the loop.

### Drift Resistance Ladder

Every verification mechanism has a resistance to drift — the tendency for claims about code to become silently false as code changes. Before writing a test, ask: "can I move up a rung?"

| Rung | Mechanism | Drift resistance | Why |
|------|-----------|-----------------|-----|
| 1 | **Type constraint** (compiler, type checker) | Immune | Won't compile. No value to edit. |
| 2 | **Structural invariant / arch test** | Almost immune | Tests shape, not value. Hard to "fix" wrong. |
| 3 | **`@decision` with ref pointing to live source** | Drifts but screams | The test fails; the ref is checkable. |
| 4 | **`@decision` with inline reason** | Drifts, still fails | Weaker than rung 3 but stronger than a comment — a comment never fails. |
| 5 | **`@scaffold`** | N/A | Dies before it can drift. |

**Rungs 1–2 are not "better tests".** They are the authority encoded as a **constraint** instead of an **assertion**. That is why they do not drift — there is no value to edit. A type that prevents an invalid state and an arch test that greps for a forbidden pattern are both constraints; a unit test that checks `result == expected` is an assertion.

Most unit tests sit at rung 3–4 when the verification could be at rung 1–2. The ladder is a thinking tool applied at the RED phase, not metadata on a test.

## Failure Attribution

### The Problem

When a test fails, the failure is in the **code**, in the **test**, or in **both**. An LLM assistant has a structural bias toward correcting the test — changing an assertion to match output is one line; understanding why code is wrong requires reasoning about intent.

### Attribution by Authority

| Failing test | Authority | Default stance |
|---|---|---|
| `@scaffold` | None | **Investigate.** Either side may be wrong. The assistant decides. |
| `@decision` / `@contract` / `@incident` | Named | **Test is the spec. Fix the code.** Changing the test requires confirming the authority changed. |
| Untagged (legacy) | Unknown | **Treat as scaffold** with a warning. Tagging it is part of the fix. |

### Anti-Pattern: Mirror Test

A **mirror test** is a permanent test edited to assert whatever the code now does. It is worse than deletion: it keeps the badge of authority while carrying none. The test stopped representing the spec and became a reflection of the implementation.

### Golden Rule

Permanent tests (`@decision`, `@contract`, `@incident`) are **never corrected to match changed code** without confirming that the authority itself changed.

**Default path (95% of cases, no pause):** Permanent test fails → treat as spec → change the code → continue. This path never stops execution.

**Blocked path (5%):** The assistant believes the test is wrong. It does **not** edit the test. It:
1. Finishes everything else in the task that does not depend on the failing test.
2. Leaves the test red.
3. Ends the turn with a precise statement: "`@decision(ref=X)` asserts P; the change requires ¬P. Either the decision at X was revisited and the test + doc should change, or the change is wrong."

One stop at the **end** of execution, not a pause in the middle.

**Never**: Quietly re-tag a permanent test to `@scaffold` so it can be edited.

**Escape hatch**: When the user's instruction **is** the authority change ("we're dropping the 30-day grace period"), the assistant may edit the test **and** must update the `ref` target in the same change. Instruction-in-hand counts as approval.

## Dependency Implementation Order

### Test-First Approach
**Requirement**: Generate all tests before implementation.

**Process:**
1. Design complete test suite
2. Generate all test files
3. Implement in dependency order:
   - Data models and types
   - Core utility functions
   - Business logic functions
   - Integration layers
   - Error handling
4. **Prune**: Tag census, collapse scaffolds, verify mutation parity

**Benefit**: Clear roadmap and comprehensive coverage.

## Parallel Test Generation

### Functional Grouping
**Requirement**: Group related functionality for batch generation.

**Structure:**
```
Test Groups:
  authentication: [login, logout, register, validate_session]
  user_management: [create_user, update_user, delete_user]
  data_processing: [validate_input, transform_data, save_data]
```

**Implementation:**
- Generate all tests for group simultaneously
- Implement all functions in group together
- Refactor entire group for consistency
- Prune the group: collapse scaffolds, promote decisions

## Coverage Requirements (Construction-Time, Measured at GREEN)

### Scenario Coverage
**Requirement**: 100% of defined scenarios must have tests.

### Branch Coverage
**Requirement**: ≥ 90% branch coverage for business logic.

### Property Coverage
**Requirement**: 100% of business constraints must have property tests.

### Integration Coverage
**Requirement**: 100% of component interactions must be tested.

## Retention Requirements (Measured After PRUNE)

### Permanent Test Survival
**Requirement**: 100% of `@decision`, `@contract`, and `@incident` tests survive the prune.

### Tag Completeness
**Requirement**: 0 untagged tests at merge. Every test is explicitly `@scaffold` (and removed) or carries a permanent tag with a `ref`.

### Mutation Parity
**Requirement**: Mutation score on changed files after prune ≥ mutation score before prune. K₁ ⊇ K₀.

### Branch Coverage (Smoke Alarm)
**Guidance**: Branch coverage on changed code is reported after prune, with 80% as a smoke alarm. A drop below 80% means collapse was too aggressive — restore rows. This is a signal, not a gate.

### Line Coverage
Dropped. Table-driven collapse keeps it trivially high; it carries no signal.

## Runner Contract

**Requirement**: The test runner must satisfy six language-neutral requirements. These are not recommendations — they are the conditions under which TDD feedback loops function for LLM-driven development.

### 1. No Bail
Never stop at first failure. The assistant batch-fixes. This is the corollary of Batch Processing Over Sequential: generating tests in batches is pointless if the runner reports failures one at a time.

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

## Quality Metrics

### Required Coverage (at GREEN)
- **Scenario coverage**: 100%
- **Branch coverage**: ≥ 90%
- **Property coverage**: 100% of constraints
- **Integration coverage**: 100% of interactions

### Required Retention (after PRUNE)
- **Mutation parity**: K₁ ⊇ K₀
- **Tag completeness**: 0 untagged tests
- **Permanent survival**: 100% of decision/contract/incident tests

### Validation Checklist
- [ ] All functions tested
- [ ] All scenarios covered
- [ ] All constraints validated
- [ ] All integrations tested
- [ ] No hallucination risks identified
- [ ] Performance validated (if applicable)
- [ ] Type safety confirmed
- [ ] Every test tagged
- [ ] Prune pass done
- [ ] Runner has no bail

## Integration with Code Standards

### Style Consistency
**Requirement**: Tests follow same style rules as production code.

**Application:**
- Same formatting rules
- Same naming conventions
- Same documentation standards
- Same linting rules

### Documentation Standards
**Requirement**: Tests must be documented.

**Required:**
- Module docstrings for test files
- Function docstrings for complex tests
- Inline comments for non-obvious assertions
- Test data documentation

## Workflows

### Workflow 1: Test Suite First

**When to Use**: Standard feature development with clear requirements.

**Process:**
1. Design complete test suite with scenario matrix
2. Tag each test at birth (scaffold by default, promote with reason)
3. Generate all test files in batches
4. Implement in dependency order
5. **Prune**: Collapse scaffolds, verify mutation parity, write census line

### Workflow 2: Scenario Matrix Approach

**When to Use**: Complex domains with many combinations of inputs and states.

**Process:**
1. Generate scenario matrix — **the matrix is the permanent artifact, not the tests**
2. Generate tests from matrix cells, tagging each
3. Implement to satisfy the matrix
4. **Prune**: Collapse matrix rows into table-driven decision tests, verify mutation parity

### Workflow 3: Feature-Complete TDD

**When to Use**: Large features with multiple interdependent components.

**Process:**
1. Define complete feature specification
2. Generate comprehensive test suite across all categories
3. Implement in dependency-ordered batches
4. **Prune**: Census, collapse, mutation parity, census line for PR

## Success Criteria

### Test Quality
- Can explain each test in one sentence?
- No duplicate test logic?
- All assertions clearly justified?
- No magic values in tests?

### Coverage Quality (at GREEN)
- All business rules tested?
- All error paths tested?
- All integrations tested?
- All constraints validated?

### Retention Quality (after PRUNE)
- Every surviving test has a permanent tag with ref?
- Mutation parity verified?
- Census line in PR?
- No mirror tests?

### Maintainability
- Easy to add new tests?
- Clear where new tests go?
- Consistent patterns used?
- Tests serve as documentation?
