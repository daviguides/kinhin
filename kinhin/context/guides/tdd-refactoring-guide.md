# TDD Refactoring Guide

## Overview

Specialized guide for refactoring code using TDD methodology. Focuses on preserving behavior while improving code structure, with comprehensive test coverage ensuring safety.

Workflows: `tdd-spec.md` → Workflows (Test Suite First: ensure coverage, then refactor; Feature-Complete for large refactorings, treated as re-implementation).

## Core Principle

**"Refactoring under green tests"**: Never refactor without comprehensive passing tests. Tests are your safety net.

## Refactoring Process

### Phase 1: Pre-Refactoring Assessment

**Objective**: Ensure the behavior being refactored is pinned by tests before any change.

**Assessment Steps:**
1. List the behaviors the refactor touches and the tests that pin each one
2. Write characterization tests for behaviors no test pins
3. Run full test suite (all must pass)
4. Identify refactoring targets and goals
5. Document current behavior

### Phase 2: Test Coverage Completion

**Objective**: Fill any gaps in test coverage before refactoring.

These tests establish a safety net for the refactoring, not permanent regression tests. Do not tag them; the separate prune pass judges them.

**Behavior Requirements:**
- Every behavior of the code being refactored is pinned by a test
- All edge cases tested
- All error paths tested
- All integrations tested

### Phase 3: Behavior Preservation Tests

**Objective**: Create tests that explicitly validate current behavior.

These are characterization tests — golden-master snapshots of current behavior that exist solely to detect unintended changes during the refactoring. They are temporary by design; the separate prune pass graduates or deletes them.

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
2. Verify performance not degraded
3. Run static analysis (linting, type checking)
4. Review code quality improvements

### After the Refactor: Prune Is a Separate Pass

The author stops at Phase 5. Removing the characterization and safety-net tests that served their purpose is a separate pass, by someone other than the author (`kinhin tag` + `/kinhin:prune`, or the milestone's `test-prune` task), with its own review of what was deleted and why.

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
**Solution**: The separate prune pass. Every characterization test is graduated with a reason or deleted there; the default is deletion.
