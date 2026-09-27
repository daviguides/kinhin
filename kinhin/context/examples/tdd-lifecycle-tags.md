# Lifecycle Tags — Syntax, Collapse, CI Exclusion

## Per-Language Tag Syntax

### Python — pytest Markers

```python
# pyproject.toml
[tool.pytest.ini_options]
markers = [
    "scaffold: construction-time, deleted at prune",
    "decision: non-obvious decision (reason required)",
    "contract: external boundary (party required)",
    "incident: prod failure reproduced (ref required)",
]
addopts = "--strict-markers"
```

```python
@pytest.mark.scaffold
def test_parser_returns_dict() -> None: ...

@pytest.mark.decision(reason="discount non-negative by billing contract")
def test_discount_never_negative() -> None: ...

@pytest.mark.contract(party="iOS BrandDetails v2")
def test_wire_format_matches_spec() -> None: ...

@pytest.mark.incident(ref="ISSUE-186")
def test_invalidation_avoids_keyspace_scan() -> None: ...
```

### Rust — Module Convention + Doc Comments

```rust
#[cfg(test)]
mod tests {
    // kinhin: decision(ref="docs/fees.md#rate-protection")
    #[test]
    fn grace_uses_higher_of_old_and_new() { ... }

    // kinhin: contract(ref="API v2 wire format")
    #[test]
    fn roundtrip_preserves_all_fields() { ... }

    mod scaffold {
        use super::*;

        #[test]
        fn parser_returns_expected_keys() { ... }

        #[test]
        fn validator_handles_empty() { ... }
    }
}
```

### Java — JUnit 5 @Tag + @DisplayName

```java
@Tag("scaffold")
@Test
void parserReturnsExpectedKeys() { ... }

@Tag("decision")
@DisplayName("deny before soft-delete: DENIED is terminal")
@Test
void denyPrecedesSoftDelete() { ... }

@Tag("incident")
@DisplayName("TICKET-186: SCAN walked 250k keys")
@Test
void invalidationNeverScansKeyspace() { ... }
```

### TypeScript — File Suffix Convention

Jest has no tag filter. Use file suffixes:

```
tests/
├── parser.test.ts              # permanent tests
├── parser.scaffold.test.ts     # construction tests — deleted at prune
├── wire-format.test.ts         # @contract via describe block name
└── regression-186.test.ts      # @incident via describe block name
```

Vitest: use `--project` or test-name prefix convention.


## Collapse Example

**Before prune** — 12 scaffold tests for a `calculateDiscount` function:

```python
@pytest.mark.scaffold
def test_discount_premium_100() -> None:
    assert calculate_discount("premium", Decimal("100")) == Decimal("10")

@pytest.mark.scaffold
def test_discount_premium_200() -> None:
    assert calculate_discount("premium", Decimal("200")) == Decimal("20")

@pytest.mark.scaffold
def test_discount_premium_500() -> None:
    assert calculate_discount("premium", Decimal("500")) == Decimal("50")

@pytest.mark.scaffold
def test_discount_premium_1000() -> None:
    assert calculate_discount("premium", Decimal("1000")) == Decimal("100")

@pytest.mark.scaffold
def test_discount_standard_100() -> None:
    assert calculate_discount("standard", Decimal("100")) == Decimal("0")

@pytest.mark.scaffold
def test_discount_standard_500() -> None:
    assert calculate_discount("standard", Decimal("500")) == Decimal("0")

@pytest.mark.scaffold
def test_discount_premium_zero() -> None:
    assert calculate_discount("premium", Decimal("0")) == Decimal("0")

@pytest.mark.scaffold
def test_discount_premium_boundary() -> None:
    assert calculate_discount("premium", Decimal("1")) == Decimal("0.10")

@pytest.mark.scaffold
def test_discount_negative_raises() -> None:
    with pytest.raises(ValueError):
        calculate_discount("premium", Decimal("-1"))

@pytest.mark.scaffold
def test_discount_unknown_tier_raises() -> None:
    with pytest.raises(ValueError):
        calculate_discount("unknown", Decimal("100"))

@pytest.mark.scaffold
def test_discount_returns_decimal() -> None:
    result = calculate_discount("premium", Decimal("100"))
    assert isinstance(result, Decimal)

@pytest.mark.scaffold
def test_discount_premium_large() -> None:
    assert calculate_discount("premium", Decimal("99999")) == Decimal("9999.90")
```

**After prune** — 2 tests:

```python
@pytest.mark.decision(reason="discount is 10% for premium, 0% for standard, non-negative")
@pytest.mark.parametrize("tier,amount,expected", [
    ("premium", Decimal("100"), Decimal("10")),      # equivalence class: premium
    ("standard", Decimal("100"), Decimal("0")),       # equivalence class: standard
    ("premium", Decimal("0"), Decimal("0")),          # boundary: zero amount
    ("premium", Decimal("1"), Decimal("0.10")),       # boundary: minimum
],
    ids=["premium-normal", "standard-normal", "zero-amount", "min-amount"],
)
def test_discount_by_tier(tier: str, amount: Decimal, expected: Decimal) -> None:
    """One row per equivalence class + boundaries."""
    assert calculate_discount(tier, amount) == expected

@pytest.mark.decision(reason="invalid inputs must raise, not return wrong values")
@pytest.mark.parametrize("tier,amount", [
    ("premium", Decimal("-1")),    # error: negative
    ("unknown", Decimal("100")),   # error: invalid tier
],
    ids=["negative-amount", "unknown-tier"],
)
def test_discount_rejects_invalid(tier: str, amount: Decimal) -> None:
    """One row per error class."""
    with pytest.raises(ValueError):
        calculate_discount(tier, amount)
```

**What was removed and why:**
- 4 premium-amount variations (same equivalence class — one representative + boundary is enough)
- 1 standard-amount duplicate (same class as `standard_100`)
- 1 type check (`isinstance`) — redundant under type hints + mypy
- 1 large-amount test — same class as `premium_100`, no boundary

**12 → 2 tests. Mutation parity: K₁ ⊇ K₀ confirmed.**


## CI Exclusion Profiles

### Python
```bash
# Gate (permanent only):
pytest -m "not scaffold"

# Full (including scaffold, during development):
pytest
```

### Rust
```bash
# Gate:
cargo nextest run --filter-expr 'not test(scaffold::)'

# Full:
cargo nextest run
```

### Java (Maven)
```xml
<excludedGroups>scaffold</excludedGroups>
```

```bash
# Gate:
mvn test  # excludedGroups applied by default

# Full (including scaffold):
mvn test -DexcludedGroups=""
```

### Java (Gradle)
```kotlin
tasks.test {
    useJUnitPlatform {
        excludeTags("scaffold")
    }
}
```

### TypeScript (Jest)
```bash
# Gate (permanent only):
jest --testPathIgnorePatterns "scaffold"

# Full:
jest
```

### TypeScript (Vitest)
```bash
# Gate:
vitest run --exclude "**/**.scaffold.test.ts"

# Full:
vitest run
```
