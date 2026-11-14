# TDD Quality Checklist

## Test Generation Checklist

**For each test generated:**

### Structure
- [ ] Follows AAA pattern (Arrange-Act-Assert)
- [ ] Has descriptive function name following convention
- [ ] Has clear docstring explaining what is tested
- [ ] Has type hints on all parameters
- [ ] Returns None (test functions)
- [ ] Tagged at birth (`@scaffold`, `@decision`, `@contract`, or `@incident`)

### Content
- [ ] Arranges test data explicitly (no hidden setup)
- [ ] Acts on function/component under test
- [ ] Asserts expected behavior clearly
- [ ] Uses specific assertions (not just `assert result`)
- [ ] Includes assertion messages for complex checks

### Coverage
- [ ] Tests one logical concern
- [ ] Covers happy path scenario OR
- [ ] Covers edge case scenario OR
- [ ] Covers error scenario OR
- [ ] Covers performance requirement OR
- [ ] Validates property/invariant

## Test Suite Completeness Checklist

**For complete test suite:**

### Scenario Coverage
- [ ] All scenario matrix cells have tests
- [ ] Happy path scenarios tested (3-5 minimum)
- [ ] Edge cases tested (boundaries, limits)
- [ ] Error cases tested (invalid inputs, failures)
- [ ] Performance requirements tested (if applicable)

### Test Categories
- [ ] Unit tests for all core functions
- [ ] Integration tests for all component interactions
- [ ] Property tests for all business constraints
- [ ] Error handling tests for all failure modes
- [ ] Performance tests for critical paths (if applicable)

### Anti-Hallucination Validation
- [ ] All business constraints have property tests
- [ ] Cross-function consistency validated
- [ ] Type safety validated
- [ ] No implicit conversions in tests
- [ ] All assumptions explicitly tested

## Scenario Checklist (at GREEN)

- [ ] Every defined scenario has a test
- [ ] All integration points tested
- [ ] All error paths tested
- [ ] No untested edge cases

## Implementation Checklist

**For implementation:**

### Test-First Validation
- [ ] All tests written before implementation
- [ ] Tests initially fail (red)
- [ ] Implementation makes tests pass (green)
- [ ] No additional untested functionality added

### Dependency Order (wiring first, tdd-spec → Dependency Implementation Order)
- [ ] Wiring skeleton (thinnest end-to-end path, fixtures copied from the codebase) implemented first
- [ ] Data models, core utilities and business logic filled in after it
- [ ] Stubs replaced by real modules with the skeleton tests still passing
- [ ] Error handling implemented last

### Code Quality
- [ ] Follows language style standards
- [ ] Type hints on all functions
- [ ] Docstrings on all public functions
- [ ] Error handling implemented
- [ ] No code smells

## Post-Implementation Checklist

**After implementation:**

### Test Execution
- [ ] All tests passing
- [ ] No skipped tests
- [ ] No warnings in test output
- [ ] Tests run in reasonable time

### Quality Validation
- [ ] Static analysis passing
- [ ] Performance requirements met
- [ ] No hallucination patterns detected

## Integration Checklist

**For integration tests:**

- [ ] All component interactions tested
- [ ] Database operations tested
- [ ] External service integrations mocked properly
- [ ] Transaction handling tested
- [ ] Error propagation tested
- [ ] Cache integration tested (if applicable)

## Property Test Checklist

**For property-based tests:**

- [ ] Invariants identified
- [ ] Appropriate strategies selected
- [ ] Sufficient examples generated (100+ default)
- [ ] Properties validate anti-hallucination
- [ ] Edge cases handled by hypothesis

## Error Handling Checklist

**For error tests:**

- [ ] All error scenarios identified
- [ ] Specific exception types tested
- [ ] Error messages validated
- [ ] Error context preserved (exception chaining)
- [ ] State remains consistent after error
- [ ] Resources cleaned up on error

## Performance Checklist

**For performance tests:**

- [ ] Performance requirements documented
- [ ] Baseline established
- [ ] Load tests implemented (if needed)
- [ ] Memory usage validated (if needed)
- [ ] Scaling characteristics validated
- [ ] Performance regression detection in place

## Documentation Checklist

**Documentation requirements:**

- [ ] Test files have module docstrings
- [ ] Complex test setups documented
- [ ] Test data factories documented
- [ ] Fixture purposes documented
- [ ] Special test configurations documented

## Quick Reference

### Red Flags (Stop if Found)
- ❌ Tests written after implementation
- ❌ Scenario matrix incomplete
- ❌ No property tests for constraints
- ❌ Magic numbers without explanation
- ❌ Weak assertions (just `assert result`)
- ❌ Multiple concerns per test
- ❌ Implicit type conversions
- ❌ Over-mocking (mocking everything)
- ❌ Brittle tests (break with minor changes)
- ❌ Untagged tests at merge
- ❌ Runner bails on first failure
- ❌ Tests pass only in file order
- ❌ Retry-until-green in CI
- ❌ Survivors assert mock call counts
