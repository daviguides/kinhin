# TDD Refactoring Guide

## Overview

Specialized guide for refactoring code using TDD methodology. Focuses on preserving behavior while improving code structure, with comprehensive test coverage ensuring safety.

Workflows: `tdd-spec.md` → Workflows (Test Suite First: ensure coverage, then refactor; Feature-Complete for large refactorings, treated as re-implementation).

## Core Principle

**"Refactoring under green tests"**: Never refactor without comprehensive passing tests. Tests are your safety net.

## Refactoring Process

### Phase 1: Pre-Refactoring Assessment

**Objective**: Ensure adequate test coverage before any refactoring.

**Assessment Steps:**
1. Check current test coverage (must be ≥ 90%)
2. Identify gaps in test coverage
3. Run full test suite (all must pass)
4. Identify refactoring targets and goals
5. Document current behavior

### Phase 2: Test Coverage Completion

**Objective**: Fill any gaps in test coverage before refactoring.

**Lifecycle Tag**: Tests born in this phase are `@scaffold` by default. They exist to establish a safety net for the refactoring, not as permanent regression tests. Tag at birth.

**Coverage Requirements:**
- 100% of code being refactored must be tested
- All branches covered
- All edge cases tested
- All error paths tested
- All integrations tested

### Phase 3: Behavior Preservation Tests

**Objective**: Create tests that explicitly validate current behavior.

**Lifecycle Tag**: Tests born in this phase are `@characterization` — golden-master snapshots of current behavior that exist solely to detect unintended changes during the refactoring. They are temporary by design and must be graduated or deleted in the Prune phase.

**Types of Behavior Tests:**
- **Input/Output Tests**: Validate outputs for all input scenarios
- **State Tests**: Validate state changes
- **Side Effect Tests**: Validate all side effects
- **Integration Tests**: Validate interactions with other components

### Phase 4: Refactoring Implementation

**Objective**: Improve code structure while keeping all tests passing.

**Rules:** every test stays green after each step; no new behavior in the same change. The refactoring moves themselves (extract function/class, rename, guard clauses, remove duplication) are code-quality rules owned by zazen.

### Phase 5: Post-Refactoring Validation

**Objective**: Ensure refactoring didn't break anything and improved quality.

**Validation Steps:**
1. Run full test suite (all must pass)
2. Check code coverage (must be ≥ 90%)
3. Verify performance not degraded
4. Run static analysis (linting, type checking)
5. Review code quality improvements

### Phase 6: Prune

**Objective**: Remove construction-time tests that served their purpose, keep only what protects against future regression.

**Steps:**
1. **Tag census**: List all tests by tag. Every `@scaffold` and `@characterization` test is a candidate for deletion.
2. **Graduate or delete**: For each characterization test, decide:
   - Does it encode a non-obvious decision about behavior? → Graduate to `@decision(ref=)` with a stated reason.
   - Is it a golden-master that duplicates what surviving tests already cover? → Delete.
   - Does it pin a boundary another system depends on? → Graduate to `@contract(ref=)`.
3. **Collapse**: Where multiple scaffold tests cover the same function, collapse into one table-driven `@decision` test with representative inputs per equivalence class.
4. **Mutation pass**: Run mutation testing on changed files. Compare killed mutants before prune (K₀) vs after (K₁). Require K₁ ⊇ K₀. Any mutant that escapes means something load-bearing was pruned — restore or write the decision test it implies.
5. **PR census line**: Report `written N / pruned M / survivors: X decision, Y contract, Z incident`.

**Prune is a separate change** from the refactoring itself (in the runner, the agent never commits: the runner commits after the mode). The reviewer sees what was deleted and why.

---

## Refactoring Strategies

### Red-Green-Refactor Cycle

**Standard TDD Cycle Applied to Refactoring:**
1. **Green**: Ensure all tests pass (green)
2. **Refactor**: Make improvement
3. **Green**: Ensure tests still pass
4. **Repeat**: Continue incremental improvements

### Incremental Refactoring

**Strategy**: Make small, safe changes frequently.

**Benefits:**
- Easier to debug if something breaks
- Lower risk per change
- Continuous code improvement
- Can be done during feature development

**Approach:** an ordered list of small changes; after each one, run the tests and keep them green.

### Large-Scale Refactoring

**Strategy**: Treat as feature re-implementation.

**When**: Architectural changes, major restructuring.

**Approach:**
1. Create comprehensive test suite for current behavior
2. Design new architecture
3. Implement new architecture alongside old
4. Migrate tests to new architecture
5. Switch to new architecture
6. Remove old architecture

## Common Pitfalls

### Refactoring Without Tests
**Problem**: No safety net, high risk of breaking code.
**Solution**: Complete Phase 2 (Test Coverage) before any refactoring.

### Combining Refactoring with New Features
**Problem**: Hard to debug issues, mixing concerns.
**Solution**: Keep refactoring and feature work in separate changes (separate blocks or tasks).

### Large Batch Refactorings
**Problem**: Hard to review, high risk, difficult to debug.
**Solution**: Use incremental refactoring strategy.

### Ignoring Test Failures
**Problem**: Breaking code unknowingly.
**Solution**: Never proceed with refactoring if tests fail.

### Over-Engineering
**Problem**: Making code more complex than needed.
**Solution**: Refactor for clarity, not cleverness. YAGNI principle.

### Keeping Every Characterization Test
**Problem**: Characterization tests written to snapshot current behavior before a refactoring outlive the refactoring and become permanent bloat. They test the implementation shape, not decisions, and break on every future refactor — generating maintenance cost with no regression value.
**Solution**: Phase 6 (Prune). Every characterization test must be explicitly graduated with a reason or deleted. The default is deletion.
