# TDD Quality Checklist

## Pre-TDD Checklist

**Before starting test-driven development:**

- [ ] Requirements clearly understood
- [ ] Feature specification documented (for new features)
- [ ] Scenario matrix defined
- [ ] Test categories identified (unit, integration, property, etc.)
- [ ] Dependencies identified
- [ ] Workflow selected (Test Suite First, Scenario Matrix, Feature-Complete)
- [ ] Runner configured (see Runner Configuration Checklist)

## Runner Configuration Checklist

**Before running any tests** (`kinhin setup` checks all of these; `kinhin setup --install` installs missing deps):

- [ ] Parallel execution enabled
- [ ] No-bail enabled (never stop at first failure)
- [ ] Randomized order enabled
- [ ] Structured reporter configured (JSON output)
- [ ] Changed-set selection tool configured
- [ ] Randomization seed persisted for reproduction

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

## Code Coverage Checklist (at GREEN)

**Coverage requirements during construction:**

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
- [ ] Follows language style standards
- [ ] Type hints on all functions
- [ ] Docstrings on all public functions
- [ ] Error handling implemented
- [ ] No code smells

## Prune Checklist

**Before opening the PR** (`kinhin prune --apply --verify` automates all of these):

- [ ] Tag census taken (all tests enumerated by tag)
- [ ] Every untagged test classified with one-line justification
- [ ] All `@scaffold` tests collapsed into table-driven `@decision` tests or deleted
- [ ] All `@characterization` tests deleted or graduated with stated reason
- [ ] Each surviving permanent test has a `ref` or documented reason
- [ ] Mutation pass run on changed files (K₁ ⊇ K₀ verified)
- [ ] PR census line written: `+written N / −pruned M / survivors: X decision, Y contract, Z incident`
- [ ] Prune changes are in a separate commit from implementation

## Post-Implementation Checklist

**After implementation:**

### Test Execution
- [ ] All tests passing
- [ ] No skipped tests
- [ ] No warnings in test output
- [ ] Tests run in reasonable time

### Coverage Validation
- [ ] Coverage report generated
- [ ] Coverage meets construction requirements (≥90% at GREEN)
- [ ] No untested code paths
- [ ] Coverage gaps explained/justified

### Quality Validation
- [ ] Static analysis passing
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

## Final Validation Checklist

**Before considering TDD complete:**

- [ ] All pre-TDD checklist items completed
- [ ] All test generation checklist items completed
- [ ] All test suite completeness items validated
- [ ] All code coverage requirements met (at GREEN)
- [ ] All post-implementation validation passed
- [ ] All category-specific checklists completed
- [ ] Prune pass complete
- [ ] Code review approved
- [ ] CI/CD pipeline passing

## Quick Reference

### Red Flags (Stop if Found)
- ❌ Tests written after implementation
- ❌ Scenario matrix incomplete
- ❌ Coverage < 90% at GREEN
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
- ✅ Scaffold deleted before PR
- ✅ One table-driven test per function
- ✅ Randomized order with printed seed

## Workflow-Specific Checklists

### For Test Suite First Workflow
- [ ] Complete test suite structure designed
- [ ] All test categories identified
- [ ] Batch test generation planned
- [ ] Dependency order mapped
- [ ] Prune pass planned after implementation

### For Scenario Matrix Workflow
- [ ] Matrix completely filled
- [ ] All combinations identified
- [ ] Test generation from matrix automated
- [ ] Matrix coverage validated
- [ ] Prune pass planned after implementation

### For Feature-Complete Workflow
- [ ] Feature specification complete
- [ ] All components identified
- [ ] Integration points mapped
- [ ] End-to-end workflows documented
- [ ] Prune pass planned after implementation

## References

- TDD Spec: `@~/.claude/kinhin/spec/tdd/tdd-spec.md`
- Implementation Guide: `@~/.claude/kinhin/context/guides/tdd-implementation-guide.md`
- Anti-Patterns: `@~/.claude/kinhin/context/examples/tdd-anti-patterns.md`
- Prune Guide: `@~/.claude/kinhin/context/guides/tdd-prune-guide.md`
- Runner Guide: `@~/.claude/kinhin/context/guides/tdd-runner-guide.md`
