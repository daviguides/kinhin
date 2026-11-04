# TDD Feature Development Guide

New features from scratch: full scope and scenarios before any code. Workflows: `tdd-spec.md` → Workflows (Feature-Complete for most features, Test Suite First for small ones).

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

### Phase 2: Scenario Matrix Creation

**Objective**: Identify all test scenarios before implementation.

**Categories:**
- **Happy Path**: Standard use cases (3-5 scenarios minimum)
- **Edge Cases**: Boundary conditions, limits, special states
- **Error Cases**: Invalid inputs, system failures, constraint violations
- **Performance**: Load handling, response times, resource usage
- **Security**: Authentication, authorization, data protection

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

### Phase 4: Implementation — Wiring First

**Objective**: Implement the wiring skeleton first, then fill in modules.

See `tdd-spec.md` → *Dependency Implementation Order* → *Wiring First*.

**Implementation Order:**
1. **Wiring Skeleton** - Thinnest end-to-end path with stubs and codebase fixtures
2. **Data Models** - Define all domain models and types
3. **Core Utilities** - Implement helper functions
4. **Business Logic** - Implement core feature logic
5. **Replace Stubs** - Swap stubs with real modules (skeleton tests keep passing)
6. **Error Handling** - Add comprehensive error handling

### Phase 5: Validation

Coverage at GREEN: `tdd-spec.md` → Coverage Requirements.

### Phase 6: Prune

`tdd-spec.md` → Test Lifecycle (prune criteria, collapse pattern, mutation parity) and Retention Requirements. In the runner, the agent never commits: the runner commits after the mode.

---

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
