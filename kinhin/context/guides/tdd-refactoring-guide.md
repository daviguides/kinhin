# TDD Refactoring Guide

## Overview

Specialized guide for refactoring code using TDD methodology. Focuses on preserving behavior while improving code structure, with comprehensive test coverage ensuring safety.

## Core Principle

**"Refactoring under green tests"**: Never refactor without comprehensive passing tests. Tests are your safety net.

## Recommended Workflow

**Primary**: Test Suite First (Workflow 1) - Ensure coverage, then refactor
**For Large Refactorings**: Feature-Complete (Workflow 3) - Treat as re-implementation

Reference: `@~/.claude/kinhin/context/guides/tdd-implementation-guide.md`

## Refactoring Process

### Phase 1: Pre-Refactoring Assessment

**Objective**: Ensure adequate test coverage before any refactoring.

**Assessment Steps:**
1. Check current test coverage (must be ≥ 90%)
2. Identify gaps in test coverage
3. Run full test suite (all must pass)
4. Identify refactoring targets and goals
5. Document current behavior

**Prompt Template:**
```
Assess code for refactoring readiness: [COMPONENT_PATH]

Analyze:
1. Current test coverage: [check coverage report]
2. Test gaps: [identify untested code paths]
3. Current behavior: [document what code does]
4. Refactoring goals: [what to improve]
5. Risk assessment: [what could break]

Provide assessment before proceeding with refactoring.
```

### Phase 2: Test Coverage Completion

**Objective**: Fill any gaps in test coverage before refactoring.

**Coverage Requirements:**
- 100% of code being refactored must be tested
- All branches covered
- All edge cases tested
- All error paths tested
- All integrations tested

**Prompt Template:**
```
Complete test coverage for: [COMPONENT_PATH]

Current coverage: [X%]
Target coverage: 100% for refactoring safety

Generate missing tests for:
1. Untested code paths: [list]
2. Untested branches: [list]
3. Untested edge cases: [list]
4. Untested error paths: [list]
5. Untested integrations: [list]

Generate all missing tests simultaneously.
Ensure all tests pass before proceeding with refactoring.
```

### Phase 3: Behavior Preservation Tests

**Objective**: Create tests that explicitly validate current behavior.

**Types of Behavior Tests:**
- **Input/Output Tests**: Validate outputs for all input scenarios
- **State Tests**: Validate state changes
- **Side Effect Tests**: Validate all side effects
- **Integration Tests**: Validate interactions with other components

**Prompt Template:**
```
Create behavior preservation tests for: [COMPONENT]

Current behavior to preserve:
- Inputs: [list all input scenarios]
- Outputs: [expected outputs]
- State changes: [what state changes occur]
- Side effects: [what side effects occur]
- Integrations: [how it interacts with other code]

Generate tests that explicitly validate all current behavior.
These tests must pass both before and after refactoring.
```

### Phase 4: Refactoring Implementation

**Objective**: Improve code structure while keeping all tests passing.

**Refactoring Types:**

#### Extract Function
**When**: Long functions, repeated code, complex logic
```
Extract function from: [COMPONENT]

Original function: [FUNCTION_NAME]
Code to extract: [lines X-Y]

Refactoring:
1. Extract to new function: [NEW_FUNCTION_NAME]
2. Replace original code with call to new function
3. Add tests for extracted function
4. Verify all existing tests still pass

Generate refactored code.
```

#### Rename for Clarity
**When**: Unclear names, inconsistent naming
```
Rename for clarity in: [COMPONENT]

Renamings:
- [old_name_1] → [new_name_1]: [reason]
- [old_name_2] → [new_name_2]: [reason]

Refactoring:
1. Rename all occurrences
2. Update docstrings/comments
3. Update tests if needed
4. Verify all tests pass

Generate refactored code with clear names.
```

#### Simplify Conditional Logic
**When**: Deep nesting, complex conditions
```
Simplify conditional logic in: [COMPONENT]

Current logic: [describe current]
Complexity issues: [deep nesting, unclear conditions, etc.]

Refactoring strategy:
1. Use guard clauses for early returns
2. Extract complex conditions to named functions
3. Flatten nested structures
4. Use match/case if appropriate

Generate simplified code.
Verify all tests still pass.
```

#### Extract Class
**When**: Class doing too much, cohesion issues
```
Extract class from: [ORIGINAL_CLASS]

New class: [NEW_CLASS_NAME]
Responsibilities to extract: [list]
Methods to move: [list]
Data to move: [list]

Refactoring:
1. Create new class with extracted responsibilities
2. Move methods and data
3. Update original class to use new class
4. Create tests for new class
5. Verify all existing tests pass

Generate refactored code with new class structure.
```

#### Reduce Code Duplication (DRY)
**When**: Repeated code patterns
```
Reduce duplication in: [COMPONENT_LIST]

Duplicated code: [describe pattern]
Occurrences: [list all locations]

Refactoring:
1. Extract common code to shared function/utility
2. Replace all occurrences with calls to shared code
3. Add tests for shared code
4. Verify all existing tests pass

Generate DRY refactored code.
```

### Phase 5: Post-Refactoring Validation

**Objective**: Ensure refactoring didn't break anything and improved quality.

**Validation Steps:**
1. Run full test suite (all must pass)
2. Check code coverage (must be ≥ 90%)
3. Verify performance not degraded
4. Run static analysis (linting, type checking)
5. Review code quality improvements

**Prompt Template:**
```
Validate refactoring of: [COMPONENT]

Run validation:
1. Test suite status: [all passing?]
2. Code coverage: [maintained or improved?]
3. Performance: [maintained or improved?]
4. Static analysis: [passing?]
5. Quality metrics: [improved?]

Provide validation report.
If any validation fails, identify and fix issue.
```

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

**Approach:**
```
Incremental refactoring plan for: [COMPONENT]

Changes (in order):
1. [Small change 1]: [description]
   - Make change
   - Run tests
   - Verify pass
2. [Small change 2]: [description]
   - Make change
   - Run tests
   - Verify pass
3. [Small change 3]: [description]
   - Make change
   - Run tests
   - Verify pass

Each change must keep tests green.
```

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

**Prompt Template:**
```
Large-scale refactoring: [COMPONENT/SYSTEM]

Current architecture: [describe]
Target architecture: [describe]
Migration strategy: [describe]

Phase 1: Comprehensive behavior tests
Generate tests covering 100% of current behavior.

Phase 2: New architecture implementation
Implement new architecture parallel to old.

Phase 3: Migration
Migrate functionality incrementally with tests passing throughout.

Phase 4: Cutover
Switch to new architecture, verify all tests pass.

Phase 5: Cleanup
Remove old architecture.

Generate refactoring plan and implementation.
```

## Common Refactoring Patterns

### 1. Extract Method
**Before:**
```python
def process_order(order: Order) -> OrderResult:
    """Process customer order."""
    # Validation (10 lines)
    # Calculation (15 lines)
    # Database save (8 lines)
    # Notification (7 lines)
```

**After:**
```python
def process_order(order: Order) -> OrderResult:
    """Process customer order."""
    validate_order(order)
    total = calculate_order_total(order)
    save_order_to_database(order, total)
    send_order_notification(order)
    return OrderResult(success=True)
```

### 2. Guard Clauses
**Before:**
```python
def calculate_discount(user: User, amount: Decimal) -> Decimal:
    """Calculate discount."""
    if user.is_active:
        if user.subscription_type == "premium":
            if amount > 100:
                return amount * Decimal("0.10")
```

**After:**
```python
def calculate_discount(user: User, amount: Decimal) -> Decimal:
    """Calculate discount."""
    if not user.is_active:
        return Decimal("0")

    if user.subscription_type != "premium":
        return Decimal("0")

    if amount <= 100:
        return Decimal("0")

    return amount * Decimal("0.10")
```

### 3. Magic Numbers to Constants
**Before:**
```python
def calculate_fee(amount: Decimal) -> Decimal:
    if amount > 1000:
        return amount * 0.02
    return amount * 0.05
```

**After:**
```python
LARGE_TRANSACTION_THRESHOLD = Decimal("1000")
LARGE_TRANSACTION_FEE_RATE = Decimal("0.02")
STANDARD_FEE_RATE = Decimal("0.05")

def calculate_fee(amount: Decimal) -> Decimal:
    """Calculate transaction fee based on amount."""
    if amount > LARGE_TRANSACTION_THRESHOLD:
        return amount * LARGE_TRANSACTION_FEE_RATE
    return amount * STANDARD_FEE_RATE
```

## Refactoring Safety Checklist

Before refactoring:
- [ ] Test coverage ≥ 90% for code being refactored
- [ ] All tests passing
- [ ] Behavior preservation tests created
- [ ] Refactoring goal clearly defined

During refactoring:
- [ ] Making incremental changes
- [ ] Running tests after each change
- [ ] All tests remain passing
- [ ] No new functionality added

After refactoring:
- [ ] All tests still passing
- [ ] Code coverage maintained or improved
- [ ] Performance maintained or improved
- [ ] Static analysis passing
- [ ] Code quality improved
- [ ] Documentation updated

## Common Pitfalls

### Refactoring Without Tests
**Problem**: No safety net, high risk of breaking code.
**Solution**: Complete Phase 2 (Test Coverage) before any refactoring.

### Combining Refactoring with New Features
**Problem**: Hard to debug issues, mixing concerns.
**Solution**: Separate refactoring commits from feature commits.

### Large Batch Refactorings
**Problem**: Hard to review, high risk, difficult to debug.
**Solution**: Use incremental refactoring strategy.

### Ignoring Test Failures
**Problem**: Breaking code unknowingly.
**Solution**: Never proceed with refactoring if tests fail.

### Over-Engineering
**Problem**: Making code more complex than needed.
**Solution**: Refactor for clarity, not cleverness. YAGNI principle.

## Refactoring Triggers

### When to Refactor

**Code Smells:**
- Long functions (>50 lines)
- Deep nesting (>3 levels)
- Duplicated code
- Unclear names
- Magic numbers
- God classes (too many responsibilities)
- Long parameter lists (>4 parameters)

**Technical Debt:**
- Hard to understand code
- Hard to test code
- Hard to modify code
- Performance issues
- Inconsistent patterns

**Boy Scout Rule:**
"Leave code better than you found it" - refactor opportunistically.

## Integration with Zen Principles

### Refactoring Goals Aligned with Zen

- **Readability Counts**: Make code more readable
- **Explicit > Implicit**: Make intentions clearer
- **Simple > Complex**: Simplify logic
- **Flat > Nested**: Reduce nesting
- **Beautiful > Ugly**: Improve aesthetics

### Refactoring Prompt with Zen

```
Refactor [COMPONENT] following Zen of Python:

Current issues:
- [Issue 1]: Violates [Zen Principle]
- [Issue 2]: Violates [Zen Principle]

Refactoring goals:
1. Improve readability
2. Make intentions explicit
3. Simplify complex logic
4. Flatten nested structures
5. Follow established patterns

Generate refactored code adhering to Zen principles.
Ensure all tests pass.
```

## Success Metrics

Refactoring is successful when:
- [ ] All tests passing
- [ ] Code coverage maintained/improved
- [ ] Code quality metrics improved
- [ ] Performance maintained/improved
- [ ] Readability improved (team consensus)
- [ ] Easier to understand
- [ ] Easier to modify
- [ ] Easier to test
- [ ] Technical debt reduced

## References

- Main Workflows: `@~/.claude/kinhin/context/guides/tdd-implementation-guide.md`
- TDD Checklist: `@~/.claude/kinhin/context/checklists/tdd-checklist.md`
- Zen Principles: `@~/.claude/kinhin/spec/principles/zen-principles-spec.md`
- Test Templates: `@~/.claude/kinhin/context/examples/tdd-unit-tests.md`
