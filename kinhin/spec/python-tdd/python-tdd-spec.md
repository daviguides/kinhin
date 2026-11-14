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

## Lifecycle Markers

**Requirement**: Every test carries a lifecycle marker at birth. Untagged = `scaffold` by default. Each permanent marker requires its kwarg: `decision(reason=)`, `contract(party=)`, `incident(ref=)`.

**Registration** — `pyproject.toml`:
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
def test_parser_returns_dict() -> None: ...

@pytest.mark.decision(reason="deny before soft-delete because DENIED is terminal")
def test_deny_precedes_soft_delete() -> None: ...

@pytest.mark.contract(party="iOS BrandDetails v2")
def test_brand_details_wire_format() -> None: ...

@pytest.mark.incident(ref="TRYPLAT-186")
def test_scan_match_never_walks_keyspace() -> None: ...
```

**Promotion** — at prune, a scaffold that encodes a real decision is re-tagged: `@pytest.mark.scaffold` → `@pytest.mark.decision(reason="discount is non-negative by contract with billing")`.

**Census / delete set:**
```bash
pytest --collect-only -q | wc -l                  # total
pytest -m scaffold --collect-only -q              # the delete set
pytest -m decision --collect-only -q | wc -l      # likewise contract, incident
```

**CI profiles:**
```bash
pytest -m "not scaffold"   # gate: permanent tests only
pytest                     # full, including scaffolds, during development
```

## Running Tests

Run the project's suite (`make test`, or the command the plan gives) on the whole affected set. Never `-x` / `--exitfirst` / `--maxfail=1`: read every failure in one run and batch-fix. Never `pytest-rerunfailures` or any rerun-until-green. Diagnostic and flake commands: `python-tdd-runner-spec.md`.
