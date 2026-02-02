# TDD Implementation Guide (LLM-Optimized)

## Overview

Comprehensive guide for implementing Test-Driven Development optimized for Large Language Models. This guide provides three main workflows, prompt engineering techniques, and speed optimization strategies.

## Three Main Workflows

### Workflow 1: Test Suite First

**When to Use**: Standard feature development with clear requirements.

**Philosophy**: Generate comprehensive test suite upfront, then implement in dependency order.

#### Step 1: Comprehensive Test Design

**Prompt Template:**
```
Design complete test suite for [FEATURE_NAME] with these components:
- Unit tests for core logic
- Integration tests for component interaction
- Edge case tests for boundary conditions
- Error handling tests for failure scenarios

Requirements:
- [REQUIREMENT_1]
- [REQUIREMENT_2]
- [CONSTRAINT_1]
- [CONSTRAINT_2]

Generate test function signatures and docstrings only (no implementation yet).
```

**Output**: Complete test suite structure with all test functions defined.

#### Step 2: Batch Test Generation

**Structure:**
```
tests/
├── test_core_logic.py          # All business logic tests
├── test_integrations.py        # All component interaction tests
├── test_edge_cases.py          # All boundary condition tests
└── test_error_handling.py      # All failure scenario tests
```

**Prompt Template:**
```
Implement all tests for [TEST_CATEGORY] simultaneously:

Test file: tests/test_[CATEGORY].py

Functions to test:
- [FUNCTION_1]: [purpose]
- [FUNCTION_2]: [purpose]
- [FUNCTION_3]: [purpose]

Generate complete test implementations for all functions in this category.
Use consistent patterns and templates.
Include all scenarios from scenario matrix.
```

#### Step 3: Dependency-Ordered Implementation

**Implementation Order:**
1. Data models and types
2. Core utility functions
3. Business logic functions
4. Integration layers
5. Error handling and validation

**Prompt Template:**
```
Implement [COMPONENT_CATEGORY] to satisfy tests in tests/test_[CATEGORY].py:

Implementation order:
1. [DEPENDENCY_1]
2. [DEPENDENCY_2]
3. [MAIN_COMPONENT]

Generate all implementations in this category simultaneously.
Ensure all tests pass before moving to next category.
```

---

### Workflow 2: Scenario Matrix Approach

**When to Use**: Complex domains with many combinations of inputs and states.

**Philosophy**: Define comprehensive scenario matrix first, then generate tests systematically.

#### Step 1: Matrix Generation

**Prompt Template:**
```
Generate scenario matrix for [COMPONENT_NAME]:

| Component | Happy Path | Edge Case | Error Case | Performance |
|-----------|------------|-----------|------------|-------------|
| [FUNC_1]  | [scenarios]| [scenarios]| [scenarios]| [criteria]  |
| [FUNC_2]  | [scenarios]| [scenarios]| [scenarios]| [criteria]  |
| [FUNC_3]  | [scenarios]| [scenarios]| [scenarios]| [criteria]  |

For each cell, provide:
- Scenario description
- Input values
- Expected output
- Assertions to validate
```

**Example Matrix:**
```
| Component | Happy Path | Edge Case | Error Case | Performance |
|-----------|------------|-----------|------------|-------------|
| email_validator | valid@email.com | a@b.co | invalid-email | 10k emails/sec |
| discount_calculator | standard_user | premium_large_order | negative_amount | bulk_calculations |
| file_processor | valid_json | empty_file | corrupted_data | large_files_100mb |
```

#### Step 2: Batch Test Generation from Matrix

**Prompt Template:**
```
Generate all tests for scenario matrix:

Component: [COMPONENT_NAME]
Matrix: [PASTE_MATRIX]

Generate test function for each matrix cell:
- Use template: test_[component]_[scenario_type]_[number]
- Include descriptive docstring
- Use AAA pattern (Arrange-Act-Assert)
- Add assertions for expected behavior

Generate all tests simultaneously for entire matrix.
```

**Generated Structure:**
```python
# Happy path tests
def test_email_validator_happy_path_1() -> None:
    """Test email validator with valid email format."""

def test_discount_calculator_happy_path_1() -> None:
    """Test discount calculator with standard user."""

# Edge case tests
def test_email_validator_edge_case_1() -> None:
    """Test email validator with minimal valid email."""

# Error case tests
def test_email_validator_error_case_1() -> None:
    """Test email validator rejects invalid format."""

# Performance tests
def test_email_validator_performance_1() -> None:
    """Test email validator handles 10k emails efficiently."""
```

#### Step 3: Matrix-Driven Implementation

**Prompt Template:**
```
Implement [COMPONENT_NAME] to satisfy all scenarios in matrix:

Matrix: [PASTE_MATRIX]

Implement with these constraints:
- Handle all happy path scenarios
- Handle all edge cases gracefully
- Raise appropriate exceptions for error cases
- Meet performance requirements

Generate complete implementation satisfying all matrix cells.
```

---

### Workflow 3: Feature-Complete TDD

**When to Use**: Large features with multiple interdependent components.

**Philosophy**: Define entire feature specification, generate complete test suite, implement in one coherent batch.

#### Step 1: Feature Specification

**Prompt Template:**
```
Define complete specification for [FEATURE_NAME]:

FEATURE_SPEC:
- Name: [feature_name]
- Components: [list all components]
- Functions: [list all functions]
- Data Models: [list all models]
- Constraints:
  - Security: [requirements]
  - Performance: [requirements]
  - Reliability: [requirements]
- Integrations: [external dependencies]
- Error Handling: [error scenarios]

Generate detailed specification covering all aspects.
```

**Example Specification:**
```python
FEATURE_SPEC = {
    "name": "user_authentication",
    "functions": [
        "login",
        "logout",
        "register",
        "validate_session",
        "reset_password"
    ],
    "data_models": [
        "User",
        "Session",
        "AuthToken"
    ],
    "constraints": {
        "security": "bcrypt hashing, secure session tokens",
        "performance": "<200ms response time",
        "reliability": "graceful failure handling, retry logic"
    },
    "integrations": [
        "database",
        "email_service",
        "audit_log"
    ]
}
```

#### Step 2: Complete Test Suite Generation

**Prompt Template:**
```
Generate complete test suite for feature: [FEATURE_NAME]

Specification: [PASTE_SPEC]

Generate tests for:
1. Unit tests for each function (all scenarios)
2. Integration tests for each integration point
3. End-to-end tests for complete workflows
4. Performance tests for critical paths
5. Security tests for auth/validation
6. Error handling tests for all failure modes

Organize tests by category:
- tests/[feature]/test_unit.py
- tests/[feature]/test_integration.py
- tests/[feature]/test_e2e.py
- tests/[feature]/test_performance.py

Generate all test files simultaneously.
```

#### Step 3: Feature-Complete Implementation

**Prompt Template:**
```
Implement complete feature: [FEATURE_NAME]

Test Suite: tests/[feature]/
Specification: [PASTE_SPEC]

Implement in dependency order:
1. Data models (User, Session, AuthToken)
2. Core functions (password hashing, token generation)
3. Authentication functions (login, logout, register)
4. Validation functions (validate_session, validate_token)
5. Integration layers (database, email, audit)
6. Error handling and recovery

Generate all implementations simultaneously.
Ensure all tests pass before completion.
```

---

## Prompt Engineering for LLM TDD

### Comprehensive Test Generation Prompts

#### Template: Complete Test Suite
```
Generate complete test suite for [COMPONENT] using TDD optimization:

1. SCENARIO MATRIX
   Create comprehensive scenario matrix:
   - Happy path cases (3-5 scenarios minimum)
   - Edge cases (boundary conditions, limits)
   - Error cases (invalid inputs, failures)
   - Performance cases (if applicable)

2. BATCH IMPLEMENTATION
   Generate all tests simultaneously using consistent templates:
   - Use AAA pattern (Arrange-Act-Assert)
   - Type hints on all parameters
   - Descriptive docstrings
   - Clear assertion messages

3. PROPERTY VALIDATION
   Include property-based tests validating:
   - Type consistency (output types always correct)
   - Business rule compliance (constraints never violated)
   - Cross-function consistency (data flows correctly)
   - Constraint adherence (limits always respected)

4. ANTI-HALLUCINATION FOCUS
   Use concrete, verifiable assertions:
   - No implicit conversions
   - Explicit validation of all assumptions
   - Clear error messages
   - Documented edge cases

5. CONSISTENCY WITH CODEBASE
   Match existing patterns:
   - Error handling approaches
   - Naming conventions
   - Type hint patterns
   - Assertion styles

Generate complete test suite following these guidelines.
```

### Context-Aware Test Design

#### Template: Pattern-Matching Tests
```
Analyze existing codebase patterns and generate tests consistent with:

EXISTING PATTERNS TO MATCH:
- Error handling: [describe existing error patterns]
- Naming conventions: [describe naming style]
- Type hints: [describe type hint usage]
- Mock/fixture patterns: [describe test setup patterns]
- Assertion styles: [describe assertion approaches]

NEW COMPONENT TO TEST: [component_name]

Generate tests that feel native to this codebase.
Match existing patterns exactly.
Use established test utilities and fixtures.
Follow naming conventions precisely.
```

### Validation-First Prompting

#### Template: Constraint-Driven Testing
```
Design comprehensive validation tests for [FUNCTION_NAME] before implementation:

1. CONSTRAINT TESTS
   What constraints must NEVER be violated?
   - [Constraint 1]: [description]
   - [Constraint 2]: [description]
   - [Constraint 3]: [description]

   Generate tests that validate each constraint independently.

2. INVARIANT TESTS
   What properties must ALWAYS hold?
   - [Invariant 1]: [description]
   - [Invariant 2]: [description]

   Generate property-based tests for each invariant.

3. INTEGRATION TESTS
   How should this function interact with others?
   - [Integration 1]: [description]
   - [Integration 2]: [description]

   Generate tests validating correct interaction.

4. EDGE CASE TESTS
   What inputs could cause unexpected behavior?
   - [Edge case 1]: [description]
   - [Edge case 2]: [description]

   Generate tests for each edge case.

5. PERFORMANCE TESTS
   What are the performance requirements?
   - [Requirement 1]: [description]

   Generate tests validating performance.

Generate complete validation suite BEFORE implementation.
Then implement function to satisfy all validation tests.
```

---

## Speed Optimization Techniques

### Parallel Test Categories

**Strategy**: Generate entire test categories simultaneously.

**Prompt Template:**
```
Generate the following test categories in parallel:

1. UNIT TESTS (tests/test_unit.py)
   - Test all core functions
   - Focus on business logic
   - Isolate external dependencies

2. INTEGRATION TESTS (tests/test_integration.py)
   - Test component interactions
   - Test database operations
   - Test external service calls

3. PROPERTY TESTS (tests/test_properties.py)
   - Test invariants
   - Test constraints
   - Test type safety

4. PERFORMANCE TESTS (tests/test_performance.py)
   - Test critical paths
   - Test load handling
   - Test resource usage

Generate all four test files simultaneously.
Use consistent structure across all files.
```

### Smart Test Grouping

**Strategy**: Group related functionality for efficient batch processing.

**Grouping Pattern:**
```python
TEST_GROUPS = {
    "authentication": [
        "login",
        "logout",
        "register",
        "validate_session",
        "reset_password",
        "change_password"
    ],
    "user_management": [
        "create_user",
        "update_user",
        "delete_user",
        "get_user",
        "list_users",
        "search_users"
    ],
    "data_processing": [
        "validate_input",
        "transform_data",
        "save_data",
        "load_data",
        "generate_report",
        "export_results"
    ]
}
```

**Prompt Template:**
```
Generate tests for functional group: [GROUP_NAME]

Functions in group: [LIST_FUNCTIONS]

Generate complete test suite for entire group:
- All unit tests for group functions
- Integration tests for function interactions
- Edge cases for each function
- Error handling for each function

Generate all tests for this group simultaneously.
```

### Template Inheritance for Speed

**Strategy**: Use base templates that LLM can inherit and customize.

**Base Template Pattern:**
```python
# Base test utilities
class BaseTestCase:
    """Base test case with common setup and utilities."""

    def setup_test_data(self) -> dict:
        """Standard test data setup."""
        return {
            "valid_user": {"email": "test@example.com", "name": "Test User"},
            "invalid_user": {"email": "invalid", "name": ""},
            "edge_case_user": {"email": "a@b.co", "name": "X"}
        }

    def assert_valid_response(self, response: dict) -> None:
        """Standard response validation."""
        assert "status" in response
        assert response["status"] in ["success", "error"]
        assert "data" in response or "error" in response

    def assert_error_response(
        self,
        response: dict,
        expected_error: str
    ) -> None:
        """Standard error response validation."""
        assert response["status"] == "error"
        assert expected_error in response["error"]
```

**Prompt Template for Inheritance:**
```
Use BaseTestCase for all tests in this module.

Base class provides:
- setup_test_data(): Standard test data
- assert_valid_response(): Response validation
- assert_error_response(): Error validation

Generate test class inheriting BaseTestCase:

class Test[ComponentName](BaseTestCase):
    """Tests for [ComponentName]."""

    def test_[scenario_1](self):
        data = self.setup_test_data()
        # Use inherited utilities

    def test_[scenario_2](self):
        # Use inherited assertions

Generate complete test class using base utilities.
```

---

## Workflow Selection Guide

### Decision Matrix

| Scenario | Recommended Workflow | Reason |
|----------|---------------------|--------|
| New feature with clear requirements | Test Suite First | Comprehensive coverage, clear structure |
| Complex domain with many states | Scenario Matrix | Systematic coverage of all combinations |
| Large multi-component feature | Feature-Complete | Holistic approach, coherent implementation |
| Bugfix with existing tests | Test Suite First | Add regression tests, modify existing |
| Refactoring with full coverage | Feature-Complete | Ensure behavior preservation |
| Performance optimization | Scenario Matrix | Specific performance scenarios |

### Workflow Combinations

**Strategy**: Combine workflows for complex scenarios.

**Example: Large Feature with Complex Domain**
1. Use **Feature-Complete** for overall structure
2. Use **Scenario Matrix** for complex components
3. Use **Test Suite First** for straightforward components

**Prompt Template:**
```
Implement [LARGE_FEATURE] using hybrid workflow:

1. FEATURE-COMPLETE STRUCTURE
   Overall feature specification:
   [PASTE_SPEC]

2. SCENARIO MATRIX for [COMPLEX_COMPONENT]
   Matrix for complex component:
   [PASTE_MATRIX]

3. TEST SUITE FIRST for [SIMPLE_COMPONENTS]
   Standard test suite for:
   - [Component 1]
   - [Component 2]

Generate complete implementation using hybrid approach.
```

---

## Best Practices

### Always Include
- Type hints on all test functions
- Descriptive docstrings
- AAA pattern (Arrange-Act-Assert)
- Clear assertion messages
- Property-based tests for constraints

### Always Avoid
- Sequential test generation (use batch instead)
- Implicit assumptions in tests
- Magic values without explanation
- Incomplete scenario coverage
- Tests without clear purpose

### Quality Checks
Before finalizing, verify:
- [ ] All scenarios from matrix covered
- [ ] All constraints have property tests
- [ ] All integrations tested
- [ ] All error paths tested
- [ ] Performance requirements validated
- [ ] Anti-hallucination patterns applied
- [ ] Consistent with codebase patterns

---

## Integration with Other Standards

### Zen Principles Integration
Apply Zen principles to test code:
- **Explicit > Implicit**: Clear test intent
- **Readability counts**: Readable tests
- **Flat > Nested**: Avoid deep test nesting
- **Simple > Complex**: Simple test logic

### Python Standards Integration
Follow Python standards:
- PEP 8 compliance in tests
- Type hints on all test code
- Proper import organization
- Descriptive naming

### Reference Checklist
Use TDD checklist for validation:
`@~/.claude/kinhin/context/checklists/tdd-checklist.md`
