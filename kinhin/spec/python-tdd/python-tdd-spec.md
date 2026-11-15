# Python TDD Specification

## Overview

Python-specific Test-Driven Development requirements. For universal TDD methodology, see `@~/.claude/kinhin/spec/tdd/tdd-spec.md`.

## Python Test Structure

**Requirement**: Follow pytest discovery conventions. Tests live in `tests/`, mirroring `src/`; one test file per source module, named `test_<module>.py` (`src/mypackage/services/user_service.py` → `tests/services/test_user_service.py`). Shared fixtures go in `tests/conftest.py`.

## Python-Specific Test Naming

**Requirement**: snake_case functions `test_{function_name}_{scenario}` (e.g. `test_validate_email_rejects_invalid_format`); PascalCase classes `Test{ComponentName}` (e.g. `TestUserService`).

## Type Hints in Tests

**Requirement**: All test functions must have a return type annotation (`-> None`). All test parameters, fixtures and parametrized arguments must have type hints.

```python
@pytest.mark.parametrize("amount,expected", [(Decimal("100"), Decimal("10"))])
def test_calculation(amount: Decimal, expected: Decimal) -> None:
    """Test calculation with various amounts."""
```

## Docstrings in Tests

**Requirement**: All test functions and all test classes must have docstrings: `"""Test {component} {behavior}."""`. Add a longer description only when the scenario is complex.

## Python-Specific Test Patterns

### Arrange-Act-Assert with Comments
**Requirement**: Use the AAA pattern with Python comments.

```python
def test_calculate_discount_for_premium_user() -> None:
    """Test calculate_discount for premium user."""
    # Arrange
    user = User(id=1, email="test@example.com")

    # Act
    result = calculate_discount(user=user, amount=Decimal("100.00"))

    # Assert
    assert result.discount_amount == Decimal("10.00")
```

### Exception Testing
**Requirement**: Use `pytest.raises` for exception testing and validate the exception details.

```python
with pytest.raises(ValueError, match="must be positive"):
    function_under_test(-1)
```

### Assertions
**Requirement**: Use plain `assert` statements, never `unittest` assertion methods.

## Running Tests

Run the project's suite (`make test`, or the command the plan gives) on the whole affected set. Never `-x` / `--exitfirst` / `--maxfail=1`: read every failure in one run and batch-fix. Never `pytest-rerunfailures` or any rerun-until-green. A failure that comes and goes is classified (rerun isolated ×3: `deterministic-fail`, `isolation-dependent` or `flaky`), never rerun to green.
