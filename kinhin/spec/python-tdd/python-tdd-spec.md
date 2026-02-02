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
**Requirement**: Use markers to categorize tests.

**Pattern:**
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

## Python-Specific Coverage

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
