# TDD Bugfix Guide

## Overview

Specialized guide for fixing bugs using TDD methodology. Focuses on reproducing bugs through tests before fixing, ensuring regression prevention.

## Recommended Workflow

**Primary**: Test Suite First (Workflow 1) - Add failing test, then fix
**For Complex Bugs**: Scenario Matrix (Workflow 2) - Systematic coverage of bug scenarios

Reference: `@~/.claude/kinhin/context/guides/tdd-implementation-guide.md`

## Bugfix Process

### Phase 1: Bug Analysis

**Objective**: Understand the bug completely before writing any code.

**Analysis Steps:**
1. Reproduce the bug consistently
2. Identify affected component(s)
3. Determine root cause
4. Identify related code that might be affected
5. Check if bug affects multiple scenarios

**Prompt Template:**
```
Analyze bug: [BUG_DESCRIPTION]

Provide:
1. Affected component(s): [list]
2. Root cause analysis: [explain]
3. Related code: [list files/functions]
4. Potential scenarios affected: [list]
5. Reproduction steps: [step by step]

Generate comprehensive bug analysis.
```

### Phase 2: Bug Reproduction Test

**Objective**: Create test that fails due to the bug.

**Test Requirements:**
- Test must fail with current buggy code
- Test must clearly demonstrate the bug
- Test should be specific to the bug scenario
- Test should include assertion showing expected vs actual behavior

**Prompt Template:**
```
Create bug reproduction test for: [BUG_DESCRIPTION]

Bug details:
- Component: [component_name]
- Root cause: [cause]
- Expected behavior: [expected]
- Actual behavior: [actual]
- Reproduction steps: [steps]

Generate test that:
1. Reproduces the bug consistently
2. Fails with current code
3. Will pass when bug is fixed
4. Has clear assertion showing the issue

Test location: tests/test_[component]_bugfix_[bug_id].py
```

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

**Considerations:**
- Similar input patterns
- Related functions
- Edge cases near the bug
- Opposite conditions

**Prompt Template:**
```
Identify related scenarios for bug: [BUG_DESCRIPTION]

Original bug scenario: [original_scenario]

Generate tests for related scenarios:
1. Similar input patterns: [scenarios]
2. Related functions: [scenarios]
3. Edge cases: [scenarios]
4. Boundary conditions: [scenarios]

Create test suite covering all related scenarios that might have same issue.
```

### Phase 4: Fix Implementation

**Objective**: Fix the bug so all tests pass, including new reproduction test.

**Fix Requirements:**
- Fix must make reproduction test pass
- Fix must not break existing tests
- Fix should handle all related scenarios
- Fix should include appropriate error handling

**Prompt Template:**
```
Fix bug: [BUG_DESCRIPTION]

Failing test: tests/test_[component]_bugfix_[bug_id].py
Root cause: [cause]
Affected component: [component_path]

Requirements:
1. Make reproduction test pass
2. Keep all existing tests passing
3. Handle all related scenarios from Phase 3
4. Add appropriate error handling
5. Follow code standards

Generate fix that satisfies all requirements.
```

### Phase 5: Regression Test Suite

**Objective**: Ensure bug won't reoccur and related bugs are prevented.

**Regression Tests Include:**
- Original bug reproduction test
- Related scenario tests
- Boundary condition tests
- Error handling tests

**Prompt Template:**
```
Create regression test suite for bug: [BUG_DESCRIPTION]

Bug fix: [describe fix]

Generate regression tests for:
1. Original bug scenario
2. All related scenarios discovered
3. Boundary conditions around fix
4. Error cases similar to bug
5. Integration tests if applicable

Organize in: tests/regression/test_bug_[bug_id].py
```

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
6. Create extensive regression test suite

## Prompt Engineering Strategies

### For Bug Reproduction

```
Create precise bug reproduction test:

Bug Information:
- ID: [bug_id]
- Component: [component]
- Symptom: [what goes wrong]
- Expected: [what should happen]
- Actual: [what actually happens]
- Input: [input that triggers bug]

Generate test that:
- Fails reliably with buggy code
- Uses exact input that triggers bug
- Has clear assertion showing expected vs actual
- Includes detailed docstring explaining bug

Test must serve as regression test after fix.
```

### For Root Cause Analysis

```
Analyze root cause of bug: [BUG_DESCRIPTION]

Code involved: [paste relevant code]

Analyze:
1. Why does this code fail?
2. What assumption was violated?
3. What input condition was not handled?
4. Are there related scenarios with same issue?
5. What is the minimal fix required?

Provide detailed root cause analysis before suggesting fix.
```

### For Comprehensive Fix

```
Implement comprehensive fix for bug: [BUG_DESCRIPTION]

Root cause: [cause]
Affected code: [component/function]
Reproduction test: tests/test_bugfix_[bug_id].py

Fix requirements:
1. Address root cause (not symptom)
2. Handle all edge cases identified
3. Add input validation if missing
4. Add error handling if appropriate
5. Maintain backward compatibility
6. Follow existing code patterns

Verify fix makes all new tests pass without breaking existing tests.
```

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
**Solution**: Phase 5 (Regression Test Suite) ensures bug prevention.

### Hasty Fixes
**Problem**: Quick patch without understanding.
**Solution**: Invest time in proper analysis and test-driven fix.

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

## Validation Checklist

Bugfix is complete when:
- [ ] Bug reproduction test created
- [ ] Reproduction test fails with buggy code
- [ ] Root cause identified and documented
- [ ] Fix implemented addressing root cause
- [ ] Reproduction test now passes
- [ ] All existing tests still pass
- [ ] Related scenarios tested
- [ ] Regression test suite created
- [ ] Code reviewed
- [ ] Documentation updated

## Post-Fix Actions

1. **Verify Fix**: Confirm bug is resolved in all scenarios
2. **Update Tests**: Ensure regression tests in place
3. **Document**: Update documentation if behavior changed
4. **Review**: Code review focusing on fix correctness
5. **Monitor**: Watch for bug recurrence or related issues

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

PHASE 5: Regression Suite
Created tests/regression/test_bug_123.py with:
- Original reproduction test
- All related scenario tests
- Boundary condition tests
```

## References

- Main Workflows: `@~/.claude/kinhin/context/guides/tdd-implementation-guide.md`
- TDD Checklist: `@~/.claude/kinhin/context/checklists/tdd-checklist.md`
- Error Handling Templates: `@~/.claude/kinhin/context/examples/tdd-error-handling-tests.md`
