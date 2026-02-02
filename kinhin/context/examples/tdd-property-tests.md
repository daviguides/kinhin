# TDD Property-Based Test Templates

## Overview

Property-based test templates using Hypothesis. These tests validate invariants and properties that must always hold, serving as anti-hallucination barriers.

## Base Property Test Template

```python
from hypothesis import given
from hypothesis import strategies as st

@given(st.{strategy}())
def test_{function_name}_property_{property}(input_data):
    """Test {function_name} maintains {property} property.

    Property: {description of invariant}
    """
    # Act
    result = {function_name}(input_data)

    # Assert - Property that must always hold
    assert {property_assertion}
```

## Invariant Validation Templates

### Template 1: Type Consistency Property

```python
@given(st.integers())
def test_double_value_returns_integer(value: int) -> None:
    """Test double_value always returns integer for integer input."""
    # Act
    result = double_value(value)

    # Assert - Type invariant
    assert isinstance(result, int), f"Expected int, got {type(result)}"
```

### Template 2: Domain Constraint Property

```python
@given(st.decimals(min_value=0, max_value=10000))
def test_calculate_discount_never_exceeds_amount(amount: Decimal) -> None:
    """Test discount never exceeds original amount."""
    # Arrange
    user = User(subscription_type="premium")

    # Act
    discount = calculate_discount(user=user, amount=amount)

    # Assert - Domain constraint
    assert discount <= amount, (
        f"Discount {discount} exceeds amount {amount}"
    )
    assert discount >= Decimal("0"), (
        f"Discount {discount} is negative"
    )
```

### Template 3: Business Rule Property

```python
@given(
    st.integers(min_value=0, max_value=100),
    st.integers(min_value=0, max_value=100)
)
def test_order_total_equals_sum_of_items(
    item1_price: int,
    item2_price: int
) -> None:
    """Test order total always equals sum of item prices."""
    # Arrange
    items = [
        OrderItem(price=Decimal(item1_price)),
        OrderItem(price=Decimal(item2_price))
    ]

    # Act
    total = calculate_order_total(items)

    # Assert - Business rule
    expected_total = Decimal(item1_price + item2_price)
    assert total == expected_total, (
        f"Total {total} != sum {expected_total}"
    )
```

## Common Strategy Templates

### Template 4: Text Property

```python
@given(st.text(min_size=1, max_size=100))
def test_sanitize_input_preserves_length_or_reduces(text: str) -> None:
    """Test sanitize_input never increases text length."""
    # Act
    sanitized = sanitize_input(text)

    # Assert - Length property
    assert len(sanitized) <= len(text), (
        f"Sanitized length {len(sanitized)} > original {len(text)}"
    )
    assert isinstance(sanitized, str)
```

### Template 5: List Property

```python
@given(st.lists(st.integers(), min_size=1, max_size=100))
def test_sort_function_returns_sorted_list(numbers: list[int]) -> None:
    """Test sort function returns correctly sorted list."""
    # Act
    result = custom_sort(numbers)

    # Assert - Sorting property
    assert len(result) == len(numbers), "Length changed"
    assert all(result[i] <= result[i+1] for i in range(len(result)-1)), (
        "List not sorted"
    )
    assert sorted(result) == sorted(numbers), "Elements changed"
```

### Template 6: Composite Strategy

```python
user_strategy = st.fixed_dictionaries({
    "id": st.integers(min_value=1),
    "email": st.emails(),
    "age": st.integers(min_value=18, max_value=120),
    "is_active": st.booleans()
})

@given(user_strategy)
def test_validate_user_with_generated_data(user_data: dict) -> None:
    """Test user validation with hypothesis-generated data."""
    # Act
    result = validate_user(user_data)

    # Assert - Validation properties
    if result.is_valid:
        assert user_data["age"] >= 18
        assert "@" in user_data["email"]
    else:
        assert result.errors is not None
        assert len(result.errors) > 0
```

## Anti-Hallucination Property Templates

### Template 7: Constraint Validation

```python
@given(
    st.decimals(min_value=0, max_value=1000),
    st.sampled_from(["standard", "premium", "enterprise"])
)
def test_discount_respects_maximum_constraint(
    amount: Decimal,
    subscription_type: str
) -> None:
    """Test discount never exceeds 50% (anti-hallucination)."""
    # Arrange
    MAX_DISCOUNT_PERCENTAGE = Decimal("0.50")
    user = User(subscription_type=subscription_type)

    # Act
    discount = calculate_discount(user=user, amount=amount)

    # Assert - Hard constraint
    max_possible_discount = amount * MAX_DISCOUNT_PERCENTAGE
    assert discount <= max_possible_discount, (
        f"Discount {discount} exceeds maximum {max_possible_discount}"
    )
```

### Template 8: Cross-Function Consistency

```python
@given(st.integers(min_value=1, max_value=1000))
def test_encode_decode_round_trip(value: int) -> None:
    """Test encode/decode maintains consistency (anti-hallucination)."""
    # Act
    encoded = encode_value(value)
    decoded = decode_value(encoded)

    # Assert - Round-trip property
    assert decoded == value, (
        f"Round-trip failed: {value} -> {encoded} -> {decoded}"
    )
```

### Template 9: Idempotence Property

```python
@given(st.text())
def test_normalize_text_is_idempotent(text: str) -> None:
    """Test normalize_text is idempotent (anti-hallucination)."""
    # Act
    normalized_once = normalize_text(text)
    normalized_twice = normalize_text(normalized_once)

    # Assert - Idempotence property
    assert normalized_once == normalized_twice, (
        f"Not idempotent: {normalized_once} != {normalized_twice}"
    )
```

## Stateful Testing Templates

### Template 10: Stateful Model

```python
from hypothesis.stateful import RuleBasedStateMachine, rule

class ShoppingCartStateMachine(RuleBasedStateMachine):
    """Test shopping cart maintains invariants across operations."""

    def __init__(self):
        super().__init__()
        self.cart = ShoppingCart()
        self.expected_items = {}

    @rule(item_id=st.integers(min_value=1, max_value=10))
    def add_item(self, item_id: int):
        """Add item to cart."""
        self.cart.add_item(item_id)
        self.expected_items[item_id] = (
            self.expected_items.get(item_id, 0) + 1
        )

    @rule(item_id=st.integers(min_value=1, max_value=10))
    def remove_item(self, item_id: int):
        """Remove item from cart."""
        if item_id in self.cart.items:
            self.cart.remove_item(item_id)
            if self.expected_items[item_id] > 1:
                self.expected_items[item_id] -= 1
            else:
                del self.expected_items[item_id]

    @rule()
    def check_invariants(self):
        """Verify cart invariants always hold."""
        # Invariant 1: Item counts match
        assert len(self.cart.items) == len(self.expected_items)

        # Invariant 2: Total is sum of item prices
        expected_total = sum(
            self.cart.get_item_price(id) * qty
            for id, qty in self.expected_items.items()
        )
        assert self.cart.total == expected_total

TestShoppingCart = ShoppingCartStateMachine.TestCase
```

## Numeric Property Templates

### Template 11: Commutativity

```python
@given(st.integers(), st.integers())
def test_add_is_commutative(a: int, b: int) -> None:
    """Test addition is commutative."""
    # Act
    result1 = add(a, b)
    result2 = add(b, a)

    # Assert - Commutativity
    assert result1 == result2
```

### Template 12: Associativity

```python
@given(st.integers(), st.integers(), st.integers())
def test_add_is_associative(a: int, b: int, c: int) -> None:
    """Test addition is associative."""
    # Act
    result1 = add(add(a, b), c)
    result2 = add(a, add(b, c))

    # Assert - Associativity
    assert result1 == result2
```

### Template 13: Identity Property

```python
@given(st.integers())
def test_add_zero_is_identity(value: int) -> None:
    """Test adding zero is identity operation."""
    # Act
    result = add(value, 0)

    # Assert - Identity
    assert result == value
```

## Hypothesis Configuration

```python
from hypothesis import settings, HealthCheck

# Project-wide settings
settings.register_profile(
    "ci",
    max_examples=1000,
    deadline=None,
    suppress_health_check=[HealthCheck.too_slow]
)

settings.register_profile("dev", max_examples=100)

# Use in tests
@settings(max_examples=500)
@given(st.integers())
def test_with_custom_examples(value: int) -> None:
    """Test with custom number of examples."""
    pass
```

## Custom Strategies

```python
# Email strategy with custom domain
email_strategy = st.builds(
    lambda user, domain: f"{user}@{domain}",
    user=st.text(
        alphabet=st.characters(whitelist_categories=("Lu", "Ll", "Nd")),
        min_size=1,
        max_size=20
    ),
    domain=st.sampled_from(["example.com", "test.org", "demo.net"])
)

# Decimal amount strategy
amount_strategy = st.decimals(
    min_value=Decimal("0.01"),
    max_value=Decimal("10000.00"),
    places=2
)

# User model strategy
user_model_strategy = st.builds(
    User,
    id=st.integers(min_value=1),
    email=email_strategy,
    age=st.integers(min_value=18, max_value=120),
    is_active=st.booleans()
)
```

## Property Test Best Practices

### Always Test
- Type consistency
- Domain constraints
- Business rule compliance
- Round-trip properties
- Idempotence where applicable
- Commutativity/associativity for math operations

### Always Include
- Clear property description in docstring
- Assertion messages explaining property violation
- Appropriate strategy for input generation
- Constraint validation

### Avoid
- Testing implementation details
- Over-constraining input strategies
- Ignoring Hypothesis warnings
- Testing without clear property

## References

- TDD Spec: `@~/.claude/kinhin/spec/tdd/tdd-spec.md`
- Python Testing Tools: `@~/.claude/kinhin/spec/python/python-testing-tools-spec.md`
- Unit Test Templates: `@~/.claude/kinhin/context/examples/tdd-unit-tests.md`
