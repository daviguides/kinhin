# Python TDD Specification

## Overview

Python-specific Test-Driven Development requirements. For universal TDD methodology, see `@~/.claude/kinhin/spec/tdd/tdd-spec.md`.

## Python Test Structure

### Directory Structure
**Requirement**: Follow Python test directory conventions.

**Structure:**
```
project/
├── src/
│   └── mypackage/
│       ├── __init__.py
│       ├── module_a.py
│       └── module_b.py
└── tests/
    ├── __init__.py
    ├── conftest.py
    ├── test_module_a.py
    └── test_module_b.py
```

### Test File Naming
**Requirement**: Follow pytest discovery conventions.

**Rules:**
- Test files: `test_*.py` (preferred) or `*_test.py`
- One test file per source module
- Mirror source directory structure in tests/

**Example:**
```
src/mypackage/services/user_service.py
→ tests/services/test_user_service.py
```

## Python-Specific Test Naming

### Test Function Naming
**Requirement**: Use Python snake_case with descriptive names.

**Pattern:**
```python
def test_{function_name}_{scenario}() -> None:
    """Test {function_name} with {scenario}."""
```

**Examples:**
```python
def test_calculate_discount_for_premium_user() -> None:
    """Test calculate_discount for premium user."""

def test_validate_email_rejects_invalid_format() -> None:
    """Test validate_email rejects invalid format."""

def test_process_payment_handles_network_error() -> None:
    """Test process_payment handles network error."""
```

### Test Class Naming
**Requirement**: Use PascalCase for test classes.

**Pattern:**
```python
class Test{ComponentName}:
    """Tests for {ComponentName}."""

    def test_{scenario}_1(self) -> None:
        """Test first scenario."""

    def test_{scenario}_2(self) -> None:
        """Test second scenario."""
```

**Example:**
```python
class TestUserService:
    """Tests for UserService class."""

    def test_create_user_with_valid_data(self) -> None:
        """Test user creation with valid data."""

    def test_create_user_rejects_duplicate_email(self) -> None:
        """Test user creation rejects duplicate email."""
```

## Type Hints in Tests

### Test Function Signatures
**Requirement**: All test functions must have return type annotation.

**Pattern:**
```python
def test_function_name() -> None:
    """Test description."""
```

### Test Parameters
**Requirement**: All test parameters must have type hints.

**Pattern:**
```python
def test_with_fixture(
    sample_user: User,
    sample_amount: Decimal,
) -> None:
    """Test with typed fixtures."""
    result = calculate_discount(user=sample_user, amount=sample_amount)
    assert result.discount_amount >= Decimal("0")
```

### Type Hints in Parametrized Tests
**Requirement**: Type hint parametrized test parameters.

**Pattern:**
```python
import pytest
from decimal import Decimal

@pytest.mark.parametrize(
    "amount,expected",
    [
        (Decimal("100"), Decimal("10")),
        (Decimal("200"), Decimal("20")),
    ],
)
def test_calculation(amount: Decimal, expected: Decimal) -> None:
    """Test calculation with various amounts."""
    result = calculate(amount)
    assert result == expected
```

## Docstrings in Tests

### Test Function Docstrings
**Requirement**: All test functions must have docstrings.

**Format:**
```python
def test_function_name() -> None:
    """Test {component} {behavior}.

    Optional detailed description if test is complex.
    Explain test scenario, setup, or expectations.
    """
```

### Simple Test Docstrings
**Pattern:**
```python
def test_add_positive_numbers() -> None:
    """Test add with positive numbers."""

def test_validate_email_format() -> None:
    """Test validate_email checks format correctly."""

def test_handle_empty_input() -> None:
    """Test function handles empty input gracefully."""
```

### Complex Test Docstrings
**Pattern:**
```python
def test_user_authentication_flow() -> None:
    """Test complete user authentication workflow.

    Validates that users can successfully register, login,
    access protected resources, and logout with proper
    session management throughout the process.

    This test covers:
    - User registration with email verification
    - Login with valid credentials
    - Session token validation
    - Protected resource access
    - Proper logout and session cleanup
    """
```

### Test Class Docstrings
**Requirement**: All test classes must have docstrings.

**Pattern:**
```python
class TestUserService:
    """Tests for UserService class.

    Tests cover user CRUD operations, authentication,
    and session management.
    """
```

## Python-Specific Test Patterns

### Arrange-Act-Assert with Comments
**Requirement**: Use AAA pattern with Python comment convention.

**Pattern:**
```python
def test_function_name() -> None:
    """Test description."""
    # Arrange: Setup test data
    user = User(id=1, email="test@example.com")
    amount = Decimal("100.00")

    # Act: Execute function
    result = calculate_discount(user=user, amount=amount)

    # Assert: Validate results
    assert result.discount_amount == Decimal("10.00")
    assert result.final_price == Decimal("90.00")
```

### Context Managers in Tests
**Requirement**: Use context managers for resource management.

**Pattern:**
```python
def test_file_processing() -> None:
    """Test file processing with context manager."""
    # Arrange
    test_data = "sample data"

    # Act & Assert
    with tempfile.NamedTemporaryFile(mode="w") as f:
        f.write(test_data)
        f.flush()

        result = process_file(f.name)
        assert result.success
```

### Exception Testing
**Requirement**: Use pytest.raises for exception testing.

**Pattern:**
```python
import pytest

def test_function_raises_value_error() -> None:
    """Test function raises ValueError for invalid input."""
    # Arrange
    invalid_input = -1

    # Act & Assert
    with pytest.raises(ValueError) as exc_info:
        function_under_test(invalid_input)

    # Validate exception details
    assert "must be positive" in str(exc_info.value)
```

## Integration with Python Standards

### PEP 8 Compliance
**Requirement**: Tests must follow PEP 8.

**Rules:**
- 80-character line limit (or project standard)
- Proper spacing (2 blank lines between functions)
- Import organization (stdlib, third-party, local)
- Snake_case for functions and variables

### Import Organization
**Requirement**: Organize imports in tests.

**Pattern:**
```python
"""Test module for user service."""

# Standard library
from decimal import Decimal
from typing import Any
from unittest.mock import Mock, patch

# Third-party
import pytest
from hypothesis import given
from hypothesis import strategies as st

# Local
from mypackage.models import User
from mypackage.services import UserService
```

### Constants in Tests
**Requirement**: Define test constants at module level.

**Pattern:**
```python
"""Test module with constants."""

# Test constants
SAMPLE_EMAIL = "test@example.com"
SAMPLE_AMOUNT = Decimal("100.00")
MAX_RETRIES = 3

def test_with_constants() -> None:
    """Test using module-level constants."""
    user = User(email=SAMPLE_EMAIL)
    result = process_user(user)
    assert result.email == SAMPLE_EMAIL
```

## Python Test Organization

### Test Fixtures in conftest.py
**Requirement**: Share fixtures via conftest.py.

**Pattern:**
```python
# tests/conftest.py
"""Shared test fixtures."""

import pytest
from decimal import Decimal
from mypackage.models import User

@pytest.fixture
def sample_user() -> User:
    """Provide sample user for testing."""
    return User(
        id=1,
        email="test@example.com",
        name="Test User",
        is_active=True,
    )

@pytest.fixture
def sample_amount() -> Decimal:
    """Provide sample monetary amount."""
    return Decimal("100.00")
```

### Test Markers
**Requirement**: Use markers to categorize tests by purpose AND lifecycle.

#### Category Markers

```python
import pytest

@pytest.mark.unit
def test_unit_function() -> None:
    """Unit test example."""

@pytest.mark.integration
def test_integration_function() -> None:
    """Integration test example."""

@pytest.mark.slow
@pytest.mark.performance
def test_performance() -> None:
    """Performance test example."""
```

#### Lifecycle Markers

**Requirement**: Every test carries a lifecycle marker at birth. Untagged = `scaffold` by default.

**Registration** — add to `pyproject.toml`:
```toml
[tool.pytest.ini_options]
markers = [
    "scaffold: construction-time verification, deleted at prune",
    "decision: encodes a non-obvious decision (reason required)",
    "contract: pins a boundary another party depends on (party required)",
    "incident: reproduces a production failure (ref required)",
]
addopts = "--strict-markers"
```

`--strict-markers` fails collection on an unregistered marker — no typos, no silent drift.

**Usage:**
```python
@pytest.mark.scaffold
def test_parser_returns_dict() -> None:
    """Scaffold: verifies basic return shape during construction."""

@pytest.mark.decision(reason="deny before soft-delete because DENIED is terminal")
def test_deny_precedes_soft_delete() -> None:
    """Decision: ordering invariant in the cleanup pipeline."""

@pytest.mark.contract(party="iOS BrandDetails v2")
def test_brand_details_wire_format() -> None:
    """Contract: serialization shape consumed by the iOS client."""

@pytest.mark.incident(ref="TRYPLAT-186")
def test_scan_match_never_walks_keyspace() -> None:
    """Incident: Redis SCAN walked 250k keys on every write."""
```

**The delete set** — list every scaffold in the project:
```bash
pytest -m scaffold --collect-only -q
```

**CI exclusion** — production gate runs only permanent tests:
```toml
# pyproject.toml or pytest.ini
[tool.pytest.ini_options]
# CI profile: exclude scaffold
# Run as: pytest -m "not scaffold"
```

**Promotion** — during prune, a scaffold that turns out to encode a real decision gets re-tagged with a reason:
```python
# Before (scaffold during construction):
@pytest.mark.scaffold
def test_discount_never_negative() -> None: ...

# After prune (promoted — encodes a business invariant):
@pytest.mark.decision(reason="discount is non-negative by contract with billing")
def test_discount_never_negative() -> None: ...
```

## Runner Configuration

**Requirement**: Configure the test runner for code-assistant batch workflows. Serial execution with bail-on-first-failure fights the batch processing principle.

### Recommended addopts

```toml
[tool.pytest.ini_options]
addopts = "-n auto -p randomly --maxfail=0 -ra -q --strict-markers"
```

| Flag | Purpose |
|---|---|
| `-n auto` | pytest-xdist: parallel execution, one worker per core |
| `-p randomly` | pytest-randomly: randomized order, seed printed at start |
| `--maxfail=0` | No limit — never bail on first failure |
| `-ra` | Show summary of all non-passing tests at end |
| `-q` | Quiet — reduce noise, keep signal |
| `--strict-markers` | Fail on unregistered markers |

**Explicitly forbidden:** `-x` / `--exitfirst` in batch mode. Use `-x` ONLY in diagnostic mode (isolating a single flake manually).

### Required Plugins

| Plugin | Purpose | Install |
|---|---|---|
| `pytest-xdist` | Parallel execution with per-worker isolation | `pip install pytest-xdist` |
| `pytest-randomly` | Randomized order; seed printed for reproduction | `pip install pytest-randomly` |
| `pytest-json-report` | Structured JSON output for assistant consumption | `pip install pytest-json-report` |
| `pytest-testmon` | Changed-set selection (only tests affected by edits) | `pip install pytest-testmon` |

**Optional (flake protocol only):**

| Plugin | Purpose |
|---|---|
| `pytest-rerunfailures` | Rerun failures for flake classification — ONLY under the three-run protocol, NEVER as retry-until-green |
| `mutmut` | Mutation testing for post-prune verification |

### Three-Run Flake Protocol (Concrete Commands)

When a test fails in the full run:

```bash
# Step 1: Rerun the failure isolated, 3 times
pytest tests/test_failing.py::test_name --count=3 -p no:randomly

# If all 3 fail → deterministic-fail. Fix the code.

# Step 2: If any pass, rerun with the original seed
pytest -p randomly --randomly-seed=<SEED_FROM_ORIGINAL_RUN>

# If fails → isolation-dependent (shared state bug). Fix now.
# If passes → flaky. Quarantine with a ticket. Never retry-to-green.
```

### Changed-Set Selection

Inner loop (during development):
```bash
pytest --testmon          # only tests affected by changed files
# or the cheap fallback:
pytest --lf --ff          # last-failed first, then the rest
```

Gate (before PR):
```bash
pytest -m "not scaffold"  # full suite, permanent tests only
```

Selection NEVER replaces the full gate run.

## Python-Specific Coverage

### Construction vs Retention

Coverage metrics serve two different purposes at two different times:

- **At GREEN (construction-time):** ≥90% branch coverage on changed code. Scaffolds exist to satisfy this — they prove the assistant explored the input space.
- **After PRUNE (retention):** mutation parity on changed files (K₁ ⊇ K₀). Branch coverage MAY drop below 90% after pruning scaffolds — that is expected and correct. An 80% floor is a smoke alarm, not a gate.

Use `mutmut` scoped to the diff for the post-prune pass:
```bash
mutmut run --paths-to-mutate=src/changed_module.py
```

### Coverage Exclusions
**Requirement**: Exclude Python-specific patterns from coverage.

**Patterns:**
```python
def debug_only_function():  # pragma: no cover
    """Function only used for debugging."""

if TYPE_CHECKING:  # Excluded by default
    from typing import TypeAlias

def __repr__(self):  # Excluded by default
    """String representation."""

if __name__ == "__main__":  # Excluded by default
    main()
```

### Abstract Methods
**Requirement**: Exclude abstract methods from coverage.

**Pattern:**
```python
from abc import ABC, abstractmethod

class BaseProcessor(ABC):
    @abstractmethod
    def process(self, data: Any) -> Any:  # Excluded
        """Process data."""
```

## Python Testing Best Practices

### Use Built-in Features
**Requirement**: Leverage Python's built-in testing features.

**Features:**
- `with` statements for context managers
- `assert` statements (not unittest methods)
- `@property` for test helpers
- Type hints for clarity
- F-strings for messages

### Test Readability
**Requirement**: Write readable Python tests.

**Practices:**
- Use descriptive variable names
- Break complex assertions into multiple lines
- Add comments for non-obvious logic
- Use f-strings for assertion messages
- Keep test functions focused

**Example:**
```python
def test_complex_calculation() -> None:
    """Test complex calculation with clear assertions."""
    # Arrange
    input_data = {"value": 100, "multiplier": 2, "offset": 10}

    # Act
    result = complex_calculation(**input_data)

    # Assert - Break down for clarity
    expected_base = input_data["value"] * input_data["multiplier"]
    expected_final = expected_base + input_data["offset"]

    assert result.base == expected_base, (
        f"Base calculation incorrect: {result.base} != {expected_base}"
    )
    assert result.final == expected_final, (
        f"Final calculation incorrect: {result.final} != {expected_final}"
    )
```

## Python Version Compatibility

### Minimum Version
**Requirement**: Support Python >= 3.13.

**Features Used:**
- Type hints (all annotations)
- F-strings for formatting
- `@dataclass` for test data
- `match` statements (Python 3.10+)
- Union types with `|` (Python 3.10+)
