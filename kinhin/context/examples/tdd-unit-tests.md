# TDD Unit Test Templates

## Overview

Complete templates for unit tests following LLM-optimized TDD methodology. Use these templates for consistent, high-quality unit test generation.

## Base Unit Test Template

### Standard AAA Pattern

```python
def test_{function_name}_{scenario}() -> None:
    """Test {function_name} with {scenario}.

    Optional detailed description if test is complex.
    """
    # Arrange: Setup test data
    {setup_variables}
    {setup_mocks}

    # Act: Execute function
    result = {function_name}({parameters})

    # Assert: Validate results
    assert {primary_assertion}
    assert {secondary_assertions}

    # Additional validations (if needed)
    {property_validations}
    {mock_validations}
```

## Function Test Templates

### Template 1: Simple Function Test

```python
def test_add_positive_numbers() -> None:
    """Test add with positive numbers."""
    # Arrange
    a = 5
    b = 3

    # Act
    result = add(a, b)

    # Assert
    assert result == 8
    assert isinstance(result, int)
```

### Template 2: Function with Complex Input

```python
def test_calculate_discount_for_premium_user() -> None:
    """Test calculate_discount for premium user with large purchase."""
    # Arrange
    user = User(
        id=1,
        email="premium@example.com",
        subscription_type="premium",
        is_active=True
    )
    purchase_amount = Decimal("1000.00")
    expected_discount = Decimal("100.00")  # 10% for premium

    # Act
    result = calculate_discount(user=user, amount=purchase_amount)

    # Assert
    assert result == expected_discount
    assert isinstance(result, Decimal)
    assert result >= Decimal("0")  # Discount never negative
```

### Template 3: Function with Side Effects

```python
def test_save_user_creates_database_record(mocker) -> None:
    """Test save_user creates database record correctly."""
    # Arrange
    user = User(email="test@example.com", name="Test User")
    mock_db = mocker.patch("myapp.database.insert")

    # Act
    result = save_user(user)

    # Assert
    assert result.success is True
    mock_db.assert_called_once_with(
        table="users",
        data={"email": user.email, "name": user.name}
    )
```

### Template 4: Function with Multiple Scenarios (Parametrized)

```python
@pytest.mark.parametrize(
    "input_value,expected_output",
    [
        # Happy path cases
        (10, 20),
        (0, 0),
        (100, 200),
        # Edge cases
        (1, 2),
        (sys.maxsize, sys.maxsize * 2),
    ],
    ids=[
        "normal-value",
        "zero-value",
        "large-value",
        "minimum-value",
        "maximum-value",
    ]
)
def test_double_value(input_value: int, expected_output: int) -> None:
    """Test double_value with various inputs."""
    # Act
    result = double_value(input_value)

    # Assert
    assert result == expected_output
    assert isinstance(result, int)
```

## Class Method Test Templates

### Template 5: Instance Method Test

```python
class TestUserService:
    """Tests for UserService class."""

    def test_create_user_with_valid_data(self) -> None:
        """Test create_user with valid user data."""
        # Arrange
        service = UserService()
        user_data = {
            "email": "test@example.com",
            "name": "Test User"
        }

        # Act
        result = service.create_user(user_data)

        # Assert
        assert result.success is True
        assert result.user.email == user_data["email"]
        assert result.user.name == user_data["name"]
        assert result.user.id is not None
```

### Template 6: Method with State Change

```python
class TestShoppingCart:
    """Tests for ShoppingCart class."""

    def test_add_item_updates_cart_state(self) -> None:
        """Test add_item updates cart state correctly."""
        # Arrange
        cart = ShoppingCart()
        item = Item(id=1, name="Product", price=Decimal("10.00"))
        assert cart.item_count == 0  # Initial state

        # Act
        cart.add_item(item, quantity=2)

        # Assert - State changes
        assert cart.item_count == 2
        assert item.id in cart.items
        assert cart.items[item.id].quantity == 2
        assert cart.total == Decimal("20.00")
```

### Template 7: Class Method Test

```python
class TestUserFactory:
    """Tests for UserFactory class."""

    def test_create_from_dict_class_method(self) -> None:
        """Test User.from_dict class method."""
        # Arrange
        user_dict = {
            "id": 1,
            "email": "test@example.com",
            "name": "Test User"
        }

        # Act
        user = User.from_dict(user_dict)

        # Assert
        assert isinstance(user, User)
        assert user.id == user_dict["id"]
        assert user.email == user_dict["email"]
        assert user.name == user_dict["name"]
```

## Validation Test Templates

### Template 8: Input Validation Test

```python
def test_validate_email_with_valid_format() -> None:
    """Test validate_email accepts valid email format."""
    # Arrange
    valid_emails = [
        "user@example.com",
        "test.email@subdomain.example.org",
        "user+tag@example.com",
        "firstname.lastname@example.co.uk"
    ]

    # Act & Assert
    for email in valid_emails:
        result = validate_email(email)
        assert result.is_valid is True, f"Email {email} should be valid"
        assert result.error is None
```

### Template 9: Constraint Validation Test

```python
def test_create_order_enforces_minimum_amount() -> None:
    """Test create_order enforces minimum order amount constraint."""
    # Arrange
    MINIMUM_ORDER_AMOUNT = Decimal("10.00")
    invalid_amount = Decimal("5.00")

    # Act & Assert
    with pytest.raises(ValueError) as exc_info:
        create_order(amount=invalid_amount)

    # Validate exception details
    assert "minimum order amount" in str(exc_info.value).lower()
    assert str(MINIMUM_ORDER_AMOUNT) in str(exc_info.value)
```

## Data Transformation Test Templates

### Template 10: Data Transformation Test

```python
def test_transform_user_data_to_dto() -> None:
    """Test transform_user_data converts to DTO correctly."""
    # Arrange
    user_data = {
        "id": 1,
        "email": "test@example.com",
        "name": "Test User",
        "created_at": "2024-01-01T00:00:00Z",
        "internal_field": "should_not_appear"
    }

    # Act
    dto = transform_user_data(user_data)

    # Assert - Correct fields transformed
    assert dto.id == user_data["id"]
    assert dto.email == user_data["email"]
    assert dto.name == user_data["name"]

    # Assert - Internal fields excluded
    assert not hasattr(dto, "internal_field")

    # Assert - Type correctness
    assert isinstance(dto, UserDTO)
```

### Template 11: Data Aggregation Test

```python
def test_calculate_order_totals_aggregates_correctly() -> None:
    """Test calculate_order_totals aggregates line items correctly."""
    # Arrange
    line_items = [
        LineItem(product="Item A", quantity=2, price=Decimal("10.00")),
        LineItem(product="Item B", quantity=1, price=Decimal("25.00")),
        LineItem(product="Item C", quantity=3, price=Decimal("5.00")),
    ]
    expected_subtotal = Decimal("60.00")  # (2*10 + 1*25 + 3*5)
    expected_tax = Decimal("6.00")  # 10%
    expected_total = Decimal("66.00")

    # Act
    result = calculate_order_totals(line_items, tax_rate=Decimal("0.10"))

    # Assert
    assert result.subtotal == expected_subtotal
    assert result.tax == expected_tax
    assert result.total == expected_total
    assert result.item_count == 6  # 2 + 1 + 3
```

## State Management Test Templates

### Template 12: State Machine Test

```python
class TestOrderStateMachine:
    """Tests for order state transitions."""

    def test_order_transitions_from_pending_to_confirmed(self) -> None:
        """Test order can transition from pending to confirmed."""
        # Arrange
        order = Order(state=OrderState.PENDING)
        assert order.state == OrderState.PENDING

        # Act
        order.confirm()

        # Assert - State changed
        assert order.state == OrderState.CONFIRMED
        assert order.confirmed_at is not None

    def test_order_cannot_transition_from_cancelled_to_confirmed(self) -> None:
        """Test order cannot transition from cancelled to confirmed."""
        # Arrange
        order = Order(state=OrderState.CANCELLED)

        # Act & Assert
        with pytest.raises(InvalidStateTransitionError) as exc_info:
            order.confirm()

        assert "Cannot confirm cancelled order" in str(exc_info.value)
```

## Fixture-Based Test Templates

### Template 13: Using Fixtures

```python
# conftest.py
@pytest.fixture
def sample_user() -> User:
    """Provide sample user for testing."""
    return User(
        id=1,
        email="test@example.com",
        name="Test User",
        is_active=True
    )

@pytest.fixture
def user_service() -> UserService:
    """Provide UserService instance."""
    return UserService()

# test file
def test_get_user_by_id(user_service: UserService, sample_user: User) -> None:
    """Test get_user_by_id retrieves correct user."""
    # Arrange
    user_service.add_user(sample_user)

    # Act
    result = user_service.get_user_by_id(sample_user.id)

    # Assert
    assert result is not None
    assert result.id == sample_user.id
    assert result.email == sample_user.email
```

## Type Safety Test Templates

### Template 14: Type Validation Test

```python
def test_function_returns_correct_type() -> None:
    """Test function returns correct type in all scenarios."""
    # Arrange
    test_cases = [
        (10, int),
        ("text", str),
        (Decimal("10.5"), Decimal),
        (True, bool),
    ]

    # Act & Assert
    for input_value, expected_type in test_cases:
        result = process_value(input_value)
        assert isinstance(result, expected_type), (
            f"Expected {expected_type}, got {type(result)}"
        )
```

## Collection Test Templates

### Template 15: List Processing Test

```python
def test_filter_active_users() -> None:
    """Test filter_active_users returns only active users."""
    # Arrange
    users = [
        User(id=1, email="user1@example.com", is_active=True),
        User(id=2, email="user2@example.com", is_active=False),
        User(id=3, email="user3@example.com", is_active=True),
    ]

    # Act
    active_users = filter_active_users(users)

    # Assert
    assert len(active_users) == 2
    assert all(user.is_active for user in active_users)
    assert active_users[0].id == 1
    assert active_users[1].id == 3
```

### Template 16: Dictionary Processing Test

```python
def test_group_users_by_subscription() -> None:
    """Test group_users_by_subscription groups correctly."""
    # Arrange
    users = [
        User(id=1, subscription="free"),
        User(id=2, subscription="premium"),
        User(id=3, subscription="free"),
    ]

    # Act
    grouped = group_users_by_subscription(users)

    # Assert
    assert "free" in grouped
    assert "premium" in grouped
    assert len(grouped["free"]) == 2
    assert len(grouped["premium"]) == 1
    assert grouped["free"][0].id == 1
    assert grouped["free"][1].id == 3
    assert grouped["premium"][0].id == 2
```

## Math/Calculation Test Templates

### Template 17: Numerical Calculation Test

```python
def test_calculate_compound_interest() -> None:
    """Test calculate_compound_interest with known values."""
    # Arrange
    principal = Decimal("1000.00")
    rate = Decimal("0.05")  # 5% annual rate
    years = 10
    compounds_per_year = 12  # Monthly compounding

    # Expected: 1000 * (1 + 0.05/12)^(12*10) ≈ 1647.01
    expected = Decimal("1647.01")

    # Act
    result = calculate_compound_interest(
        principal=principal,
        annual_rate=rate,
        years=years,
        compounds_per_year=compounds_per_year
    )

    # Assert
    assert abs(result - expected) < Decimal("0.01")  # Within 1 cent
    assert result > principal  # Interest was earned
```

## Best Practices in Templates

### Always Include
1. **Type hints** on all parameters and return values
2. **Descriptive docstrings** explaining what is tested
3. **AAA pattern** (Arrange-Act-Assert) with comments
4. **Clear variable names** (no single letters except in loops)
5. **Explicit assertions** with failure messages when helpful

### Always Avoid
1. **Magic numbers** - use named constants
2. **Implicit assertions** - be explicit about what you're testing
3. **Multiple concerns** - one logical assertion per test
4. **Hidden setup** - make test data visible in test
5. **Shared mutable state** - each test should be independent

## Template Selection Guide

| Scenario | Template | Notes |
|----------|----------|-------|
| Simple function, simple input | Template 1 | Basic AAA pattern |
| Complex input objects | Template 2 | Show object construction |
| Function with side effects | Template 3 | Use mocks to verify |
| Multiple similar scenarios | Template 4 | Use parametrize |
| Instance methods | Template 5 | Test class behavior |
| State changes | Template 6 | Assert state before/after |
| Class methods | Template 7 | Test factory methods |
| Input validation | Template 8 | Test multiple valid cases |
| Constraint enforcement | Template 9 | Test constraint violation |
| Data transformation | Template 10 | Verify correct mapping |
| Data aggregation | Template 11 | Verify calculations |
| State machines | Template 12 | Test transitions |
| Using fixtures | Template 13 | Reusable test data |
| Type safety | Template 14 | Verify type correctness |
| List processing | Template 15 | Test collection operations |
| Dictionary processing | Template 16 | Test grouping/mapping |
| Calculations | Template 17 | Test numerical accuracy |

## References

- TDD Spec: `@~/.claude/kinhin/spec/tdd/tdd-spec.md`
- Python TDD Spec: `@~/.claude/kinhin/spec/python/python-tdd-spec.md`
- Implementation Guide: `@~/.claude/kinhin/context/guides/tdd-implementation-guide.md`
