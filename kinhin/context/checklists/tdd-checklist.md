# TDD Quality Checklist

## Pre-TDD Checklist

**Before starting test-driven development:**

- [ ] Requirements clearly understood
- [ ] Feature specification documented (for new features)
- [ ] Scenario matrix defined
- [ ] Test categories identified (unit, integration, property, etc.)
- [ ] Dependencies identified
- [ ] Workflow selected (Test Suite First, Scenario Matrix, Feature-Complete)

## Test Generation Checklist

**For each test generated:**

### Structure
- [ ] Follows AAA pattern (Arrange-Act-Assert)
- [ ] Has descriptive function name following convention
- [ ] Has clear docstring explaining what is tested
- [ ] Has type hints on all parameters
- [ ] Returns None (test functions)

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

## Code Coverage Checklist

**Coverage requirements:**

- [ ] Overall coverage ≥ 90%
- [ ] Branch coverage ≥ 90%
- [ ] All business logic functions 100% covered
- [ ] All integration points 100% covered
- [ ] All error paths tested
- [ ] No untested edge cases

## Implementation Checklist

**For implementation:**

### Test-First Validation
- [ ] All tests written before implementation
- [ ] Tests initially fail (red)
- [ ] Implementation makes tests pass (green)
- [ ] No additional untested functionality added

### Dependency Order
- [ ] Data models implemented first
- [ ] Core utilities implemented second
- [ ] Business logic implemented third
- [ ] Integration layers implemented fourth
- [ ] Error handling implemented last

### Code Quality
- [ ] Follows Python style standards
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

### Coverage Validation
- [ ] Coverage report generated
- [ ] Coverage meets requirements (≥90%)
- [ ] No untested code paths
- [ ] Coverage gaps explained/justified

### Quality Validation
- [ ] Static analysis passing (ruff, pyright)
- [ ] Performance requirements met
- [ ] No hallucination patterns detected
- [ ] Code review completed

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

## Maintenance Checklist

**For long-term maintenance:**

- [ ] Tests are not brittle
- [ ] Tests use minimal mocking
- [ ] Test data is clear and explicit
- [ ] Tests serve as documentation
- [ ] Tests are easy to understand
- [ ] Tests are easy to modify

## Final Validation Checklist

**Before considering TDD complete:**

- [ ] All pre-TDD checklist items completed
- [ ] All test generation checklist items completed
- [ ] All test suite completeness items validated
- [ ] All code coverage requirements met
- [ ] All post-implementation validation passed
- [ ] All category-specific checklists completed
- [ ] Code review approved
- [ ] CI/CD pipeline passing

## Quick Reference

### Red Flags (Stop if Found)
- ❌ Tests written after implementation
- ❌ Scenario matrix incomplete
- ❌ Coverage < 90%
- ❌ No property tests for constraints
- ❌ Magic numbers without explanation
- ❌ Weak assertions (just `assert result`)
- ❌ Multiple concerns per test
- ❌ Implicit type conversions
- ❌ Over-mocking (mocking everything)
- ❌ Brittle tests (break with minor changes)

### Green Flags (Good Signs)
- ✅ Tests generated in batches
- ✅ Comprehensive scenario matrix
- ✅ Strong, specific assertions
- ✅ Property tests for invariants
- ✅ Clear test names and docstrings
- ✅ Explicit test data
- ✅ Minimal mocking
- ✅ Fast test execution
- ✅ Tests serve as documentation
- ✅ Easy to add new tests

## Workflow-Specific Checklists

### For Test Suite First Workflow
- [ ] Complete test suite structure designed
- [ ] All test categories identified
- [ ] Batch test generation planned
- [ ] Dependency order mapped

### For Scenario Matrix Workflow
- [ ] Matrix completely filled
- [ ] All combinations identified
- [ ] Test generation from matrix automated
- [ ] Matrix coverage validated

### For Feature-Complete Workflow
- [ ] Feature specification complete
- [ ] All components identified
- [ ] Integration points mapped
- [ ] End-to-end workflows documented

## References

- TDD Spec: `@~/.claude/kinhin/spec/tdd/tdd-spec.md`
- Implementation Guide: `@~/.claude/kinhin/context/guides/tdd-implementation-guide.md`
- Anti-Patterns: `@~/.claude/kinhin/context/examples/tdd-anti-patterns.md`
