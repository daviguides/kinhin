# Lifecycle Tags — Collapse

Tag syntax and the CI exclusion profile (gate runs without scaffolds) are per language: see the Lifecycle Tagging / Test Markers section of each `<lang>-tdd-spec` (python, rust, java, typescript). Tag semantics: `tdd-spec.md` → Test Lifecycle.

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

