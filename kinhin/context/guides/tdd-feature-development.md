# TDD Feature Development Guide

## Overview

Specialized guide for developing new features from scratch using TDD. Focuses on comprehensive test coverage before implementation.

## Recommended Workflow

**Primary**: Feature-Complete TDD (Workflow 3)
**Alternative**: Test Suite First (Workflow 1) for smaller features

Reference: `@~/.claude/kinhin/context/guides/tdd-implementation-guide.md`

## Feature Development Process

### Phase 1: Feature Specification

**Objective**: Define complete feature scope before writing any code.

**Steps:**
1. List all components required
2. Define all functions and their signatures
3. Specify all data models
4. Document all constraints (security, performance, reliability)
5. Identify all integrations with external systems
6. Define all error scenarios

**Prompt Template:**
```
Create feature specification for: [FEATURE_NAME]

Include:
- Components list with responsibilities
- Function signatures with type hints
- Data model definitions
- Security constraints
- Performance requirements
- External integrations
- Error scenarios

Generate comprehensive specification.
```

### Phase 2: Scenario Matrix Creation

**Objective**: Identify all test scenarios before implementation.

**Categories:**
- **Happy Path**: Standard use cases (3-5 scenarios minimum)
- **Edge Cases**: Boundary conditions, limits, special states
- **Error Cases**: Invalid inputs, system failures, constraint violations
- **Performance**: Load handling, response times, resource usage
- **Security**: Authentication, authorization, data protection

**Prompt Template:**
```
Create scenario matrix for feature: [FEATURE_NAME]

For each component in [COMPONENT_LIST]:
| Component | Happy Path | Edge Case | Error Case | Performance | Security |
|-----------|------------|-----------|------------|-------------|----------|
| [COMP_1]  | [scenarios]| [scenarios]| [scenarios]| [criteria]  |[checks]  |

Generate complete matrix covering all scenarios.
```

### Phase 3: Test Suite Generation

**Objective**: Generate all tests upfront in logical groups.

**Test Organization:**
```
tests/[feature_name]/
├── __init__.py
├── conftest.py              # Feature-specific fixtures
├── test_unit.py             # Unit tests for core logic
├── test_integration.py      # Integration tests
├── test_e2e.py             # End-to-end workflows
├── test_performance.py      # Performance validation
└── test_security.py         # Security validation
```

**Prompt Template:**
```
Generate complete test suite for feature: [FEATURE_NAME]

Scenario matrix: [PASTE_MATRIX]
Specification: [PASTE_SPEC]

Generate test files:
1. test_unit.py - All unit tests for core logic
2. test_integration.py - All integration tests
3. test_e2e.py - Complete user workflows
4. test_performance.py - Performance requirements
5. test_security.py - Security validations

Generate all test files simultaneously with complete implementations.
```

### Phase 4: Implementation in Dependency Order

**Objective**: Implement all components in order that respects dependencies.

**Implementation Order:**
1. **Data Models** - Define all domain models and types
2. **Core Utilities** - Implement helper functions
3. **Business Logic** - Implement core feature logic
4. **Integration Layers** - Connect to external systems
5. **API/Interface** - Expose feature functionality
6. **Error Handling** - Add comprehensive error handling

**Prompt Template:**
```
Implement feature [FEATURE_NAME] in dependency order:

Tests to satisfy: tests/[feature_name]/

Implementation order:
1. Data Models: [list models]
2. Core Utilities: [list utilities]
3. Business Logic: [list core functions]
4. Integration Layers: [list integrations]
5. API/Interface: [list endpoints/interfaces]
6. Error Handling: [list error scenarios]

Generate implementations for phase [N] ensuring all related tests pass.
```

### Phase 5: Validation and Refinement

**Objective**: Ensure all tests pass and coverage is complete.

**Validation Checklist:**
- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] All e2e tests pass
- [ ] Performance requirements met
- [ ] Security validations pass
- [ ] Code coverage ≥ 90%
- [ ] No untested edge cases
- [ ] All constraints validated

## Feature Complexity Handling

### Simple Features (1-3 components)

**Approach**: Test Suite First (Workflow 1)

**Process:**
1. Define test suite structure
2. Generate all tests in one batch
3. Implement all components in one batch
4. Validate and refine

### Medium Features (4-10 components)

**Approach**: Feature-Complete TDD (Workflow 3)

**Process:**
1. Create feature specification
2. Generate scenario matrix
3. Generate test suite by category
4. Implement in dependency batches
5. Validate each batch before proceeding

### Complex Features (10+ components)

**Approach**: Hybrid (Feature-Complete + Scenario Matrix)

**Process:**
1. Create overall feature specification
2. Identify complex components requiring scenario matrix
3. Use scenario matrix for complex components
4. Use standard test suite for straightforward components
5. Implement in dependency order with validation gates

## Prompt Engineering Strategies

### For Comprehensive Coverage

```
Generate test suite with COMPREHENSIVE coverage for [FEATURE]:

Coverage Requirements:
- 100% of scenario matrix cells tested
- All business constraints have property tests
- All integrations have integration tests
- All error paths have error tests
- All performance criteria have performance tests

Use batch generation:
- Generate all tests in category simultaneously
- Use consistent templates
- Apply AAA pattern
- Include type hints and docstrings

Validate anti-hallucination:
- Explicit assertions for all expected behavior
- No implicit conversions
- Clear error messages
- Documented assumptions
```

### For Dependency Management

```
Implement [FEATURE] respecting dependencies:

Dependency Graph:
[MODEL_A] → [UTILITY_B] → [LOGIC_C] → [INTEGRATION_D]

Implementation Order:
1. Implement [MODEL_A] (no dependencies)
2. Implement [UTILITY_B] (depends on MODEL_A)
3. Implement [LOGIC_C] (depends on UTILITY_B)
4. Implement [INTEGRATION_D] (depends on LOGIC_C)

For each step:
- Run related tests
- Verify all pass
- Proceed to next dependency level

Generate implementations respecting dependency order.
```

### For Performance Requirements

```
Implement [FEATURE] with performance validation:

Performance Requirements:
- [Requirement 1]: [criterion]
- [Requirement 2]: [criterion]
- [Requirement 3]: [criterion]

Include performance tests:
- Measure actual performance
- Validate meets requirements
- Test under load
- Test resource usage

Generate implementation optimized for performance requirements.
```

## Common Pitfalls

### Incomplete Specification
**Problem**: Starting implementation before fully understanding requirements.
**Solution**: Complete Phase 1 (Feature Specification) thoroughly before proceeding.

### Missing Edge Cases
**Problem**: Only testing happy path scenarios.
**Solution**: Use scenario matrix to systematically identify all edge cases.

### Sequential Implementation
**Problem**: Implementing one function at a time.
**Solution**: Batch implementation by dependency level.

### Insufficient Integration Tests
**Problem**: Only unit testing, missing integration issues.
**Solution**: Include dedicated integration test file with all component interactions.

### Performance Afterthought
**Problem**: Discovering performance issues late.
**Solution**: Include performance tests from the beginning, validate throughout.

## Success Criteria

Feature development is complete when:
- [ ] Feature specification documented
- [ ] Scenario matrix complete
- [ ] All test files created and passing
- [ ] All components implemented
- [ ] Code coverage ≥ 90%
- [ ] Performance requirements met
- [ ] Security validations pass
- [ ] Integration tests pass
- [ ] E2E workflows work correctly
- [ ] Documentation updated
- [ ] No known bugs or issues

## Next Steps After Feature Complete

1. **Code Review**: Review against TDD checklist
2. **Documentation**: Update user and technical documentation
3. **Integration**: Integrate with existing codebase
4. **Deployment**: Follow deployment procedures
5. **Monitoring**: Set up monitoring for new feature

## References

- Main Workflows: `@~/.claude/kinhin/context/guides/tdd-implementation-guide.md`
- TDD Checklist: `@~/.claude/kinhin/context/checklists/tdd-checklist.md`
- Test Templates: `@~/.claude/kinhin/context/examples/tdd-unit-tests.md`
