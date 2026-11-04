# TDD Bugfix Guide

Fix bugs by reproducing them in a failing test first. Workflows: `tdd-spec.md` → Workflows (Test Suite First; Scenario Matrix for complex bugs).

## Bugfix Process

### Phase 1: Bug Analysis

**Objective**: Understand the bug completely before writing any code.

**Analysis Steps:**
1. Reproduce the bug consistently
2. Identify affected component(s)
3. Determine root cause
4. Identify related code that might be affected
5. Check if bug affects multiple scenarios

### Phase 2: Bug Reproduction Test

**Objective**: Create test that fails due to the bug.

**Lifecycle Tag**: The reproduction test is `@incident(ref=<ticket>)` at birth. It reproduces a real production failure and is permanent by definition — it is the regression guard that ensures this specific bug never returns.

**Test Requirements:**
- Test must fail with current buggy code
- Test must clearly demonstrate the bug
- Test should be specific to the bug scenario
- Test should include assertion showing expected vs actual behavior

**Example:**
```python
def test_discount_calculation_handles_zero_amount() -> None:
    """Test discount calculation with zero amount (Bug #123).

    Bug: Function raises ZeroDivisionError when amount is zero.
    Expected: Return zero discount gracefully.
    """
    # Arrange
    user = User(subscription_type="premium")
    amount = Decimal("0")

    # Act & Assert
    result = calculate_discount(user=user, amount=amount)

    # This assertion will pass when bug is fixed
    assert result == Decimal("0")
    # Currently raises: ZeroDivisionError
```

### Phase 3: Additional Scenario Tests

**Objective**: Identify related scenarios that might have same bug.

**Lifecycle Tag**: Tests born in this phase are `@scaffold`. They are exploratory — probing whether the bug's root cause affects other paths. Most will be deleted or collapsed into the reproduction test during the Prune phase.

**Considerations:**
- Similar input patterns
- Related functions
- Edge cases near the bug
- Opposite conditions

### Phase 4: Fix Implementation

**Objective**: Fix the bug so all tests pass, including new reproduction test.

**Fix Requirements:**
- Fix must make reproduction test pass
- Fix must not break existing tests
- Fix should handle all related scenarios
- Fix should include appropriate error handling

### Phase 5: Graduate and Prune

**Objective**: Keep the incident test, graduate any scaffold that encodes a real decision, delete the rest.

**The reproduction test (`@incident`) survives unconditionally** — it is the permanent regression guard.

**Phase 3 scaffold tests are evaluated:**
- Does the scenario encode a non-obvious decision about error handling? → Graduate to `@decision(ref=)` with a stated reason.
- Does it cover the same equivalence class as the reproduction test? → Collapse into the reproduction test as an additional parametrized row, or delete.
- Is it a different bug entirely? → Separate it into its own `@incident(ref=)` with its own ticket.

**Mutation parity**: Run mutation testing on changed files. The surviving suite must kill every mutant the full suite killed. If a mutant escapes, restore or write the decision test it implies.

**PR census line**: `written N / pruned M / survivors: X decision, Y contract, Z incident`.

## Bug Complexity Handling

### Simple Bugs (Single function, clear cause)

**Approach**: Single reproduction test + fix

**Steps:**
1. Create reproduction test
2. Verify test fails
3. Implement fix
4. Verify test passes
5. Run full test suite

### Medium Bugs (Multiple scenarios, unclear scope)

**Approach**: Scenario matrix for related cases

**Steps:**
1. Create bug analysis
2. Generate scenario matrix for affected area
3. Create reproduction tests for all scenarios
4. Implement comprehensive fix
5. Validate all scenarios pass

### Complex Bugs (Multiple components, system-wide)

**Approach**: Feature-complete regression testing

**Steps:**
1. Analyze all affected components
2. Create comprehensive test suite for affected area
3. Generate reproduction tests for all scenarios
4. Implement fix across all components
5. Run full integration test suite
6. Graduate and prune (Phase 5): one `@incident` per real bug, scaffolds collapsed or deleted

## Common Pitfalls

### Fixing Symptom Not Cause
**Problem**: Fixing what's visible without addressing root cause.
**Solution**: Complete Phase 1 (Bug Analysis) thoroughly, identify actual root cause.

### Incomplete Test Coverage
**Problem**: Only testing the specific failing case.
**Solution**: Use Phase 3 (Additional Scenario Tests) to find related scenarios.

### Breaking Existing Functionality
**Problem**: Fix introduces new bugs.
**Solution**: Run full test suite after fix, ensure all existing tests still pass.

### No Regression Tests
**Problem**: Bug can reoccur in future.
**Solution**: The Phase 2 reproduction test (`@incident`) is the permanent regression guard; Phase 5 keeps it and prunes the rest.

### Hasty Fixes
**Problem**: Quick patch without understanding.
**Solution**: Invest time in proper analysis and test-driven fix.

### Keeping Every Exploratory Test
**Problem**: Phase 3 exploratory tests survive as permanent regression tests. They probe related scenarios during investigation but most cover the same root cause as the reproduction test. Keeping them all creates maintenance drag on every future change near the bug.
**Solution**: Phase 5 (Graduate and Prune). The reproduction test (`@incident`) is the permanent guard. Exploratory tests collapse into it or die.

## Bug Categories and Approaches

### Logic Bugs
**Characteristics**: Wrong business logic or algorithm.
**Approach**:
- Reproduce with unit test
- Fix logic
- Add property tests for invariants

### Edge Case Bugs
**Characteristics**: Fails on boundary conditions.
**Approach**:
- Create scenario matrix for boundaries
- Test all boundary conditions
- Implement robust boundary handling

### Integration Bugs
**Characteristics**: Fails when components interact.
**Approach**:
- Create integration tests
- Test component interactions
- Fix integration layer

### Performance Bugs
**Characteristics**: Slow or resource-intensive.
**Approach**:
- Create performance test with requirement
- Profile to find bottleneck
- Optimize and validate improvement

### Concurrency Bugs
**Characteristics**: Race conditions, deadlocks.
**Approach**:
- Create test that reproduces race condition
- Add synchronization
- Test under concurrent load

## Example: Complete Bugfix Flow

```
BUG: Calculate discount raises ZeroDivisionError for zero amount

PHASE 1: Analysis
- Component: pricing/discount_calculator.py
- Root cause: Division by amount without zero check
- Related: All discount calculation functions

PHASE 2: Reproduction Test
def test_discount_with_zero_amount() -> None:
    """Bug #123: ZeroDivisionError with zero amount."""
    result = calculate_discount(user, Decimal("0"))
    assert result == Decimal("0")  # Should return 0, not crash

PHASE 3: Related Scenarios
- test_discount_with_negative_amount()
- test_discount_with_very_small_amount()
- test_bulk_discount_with_zero_amount()

PHASE 4: Fix
def calculate_discount(user: User, amount: Decimal) -> Decimal:
    """Calculate discount for user purchase."""
    if amount <= 0:
        return Decimal("0")  # FIX: Handle zero and negative
    # ... rest of logic

PHASE 5: Graduate and Prune
- test_discount_with_zero_amount → @incident(ref="Bug #123"), kept
- negative / very-small amount → collapsed as rows of the incident test
  (same root cause: missing amount <= 0 guard)
- bulk discount with zero amount → kept only if it kills a mutant the
  incident test does not (mutation parity), else deleted
```
