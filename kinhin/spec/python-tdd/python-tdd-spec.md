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

## Runner Configuration

**Requirement**: Configure the test runner for code-assistant batch workflows. Serial execution with bail-on-first-failure fights the batch processing principle.

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

**Required plugins:** `pytest-xdist` (parallel, per-worker isolation), `pytest-randomly` (randomized order, seed printed), `pytest-json-report` (structured output), `pytest-testmon` (changed-set selection).

**Optional:** `pytest-rerunfailures` — ONLY under the three-run flake protocol, NEVER as retry-until-green; `mutmut` — mutation testing for post-prune verification.

### Run Modes

```bash
pytest --testmon -n auto -p randomly --maxfail=0               # loop: changed set only
pytest --lf --ff                                               # loop, cheap fallback
pytest -m "not scaffold" -n auto -p randomly --maxfail=0       # full gate before PR
pytest tests/test_suspect.py::test_name -p no:randomly -x      # diagnostic only
```

Selection NEVER replaces the full gate run.

### Three-Run Flake Protocol

```bash
# Step 1: isolated, 3 times
pytest tests/test_file.py::test_name --count=3 -p no:randomly -p no:xdist
# All 3 fail → deterministic-fail. Fix the code.

# Step 2: original environment
pytest -p randomly --randomly-seed=<SEED> -n auto
# Fails → isolation-dependent (shared state bug). Fix now.
# Passes → flaky. Quarantine with a ticket. Never retry-to-green.
```

### Structured Output and Clustering

```bash
pytest --json-report --json-report-file=results.json
```

Key fields: `.tests[].outcome`, `.tests[].call.crash.path`, `.tests[].call.crash.lineno`, `.tests[].call.crash.message`.

Without JSON:
```bash
pytest --tb=short 2>&1 | grep -E "^(FAILED|E )" | sort | uniq -c | sort -rn
```

## Python-Specific Coverage

### Construction vs Retention

- **At GREEN (construction-time):** ≥90% branch coverage on changed code. Scaffolds exist to satisfy this — they prove the assistant explored the input space.
- **After PRUNE (retention):** mutation parity on changed files (K₁ ⊇ K₀). Branch coverage MAY drop below 90% after pruning scaffolds — that is expected and correct. An 80% floor is a smoke alarm, not a gate; a `--cov-fail-under=90` setting is never the post-prune gate.

Use `mutmut` scoped to the diff for the post-prune pass:
```bash
mutmut run --paths-to-mutate=src/changed_module.py
```
