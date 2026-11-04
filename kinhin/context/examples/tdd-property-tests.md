# TDD Property-Based Test Templates (Python)

Hypothesis properties are anti-hallucination barriers: invariants that must always hold. Strategies and profile configuration: shodo `python-testing-tools-spec.md`.

## Anti-Hallucination Properties

### Hard Constraint

```python
@given(
    st.decimals(min_value=0, max_value=1000),
    st.sampled_from(["standard", "premium", "enterprise"]),
)
def test_discount_respects_maximum_constraint(amount: Decimal, subscription_type: str) -> None:
    """Test discount never exceeds 50%."""
    MAX_DISCOUNT_PERCENTAGE = Decimal("0.50")
    discount = calculate_discount(user=User(subscription_type=subscription_type), amount=amount)
    assert discount <= amount * MAX_DISCOUNT_PERCENTAGE
```

### Round-Trip (Cross-Function Consistency)

```python
@given(st.integers(min_value=1, max_value=1000))
def test_encode_decode_round_trip(value: int) -> None:
    """Test encode/decode maintains consistency."""
    encoded = encode_value(value)
    assert decode_value(encoded) == value, f"{value} -> {encoded} -> round-trip failed"
```

### Idempotence

```python
@given(st.text())
def test_normalize_text_is_idempotent(text: str) -> None:
    """Test normalize_text is idempotent."""
    once = normalize_text(text)
    assert normalize_text(once) == once
```

## Stateful Testing

Invariants across sequences of operations.

```python
class ShoppingCartStateMachine(RuleBasedStateMachine):
    """Test shopping cart maintains invariants across operations."""

    def __init__(self) -> None:
        super().__init__()
        self.cart = ShoppingCart()
        self.expected: dict[int, int] = {}

    @rule(item_id=st.integers(min_value=1, max_value=10))
    def add_item(self, item_id: int) -> None:
        self.cart.add_item(item_id)
        self.expected[item_id] = self.expected.get(item_id, 0) + 1

    @invariant()
    def counts_match(self) -> None:
        assert len(self.cart.items) == len(self.expected)


TestShoppingCart = ShoppingCartStateMachine.TestCase
```

## Property Test Best Practices

- Always cover: domain constraints, round-trip, idempotence where applicable.
- State the property in the docstring; assertion messages explain the violation.
- Avoid over-constraining strategies; do not ignore Hypothesis health-check warnings.
