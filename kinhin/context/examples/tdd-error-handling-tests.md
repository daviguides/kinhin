# TDD Error Handling Test Templates

## Overview

Templates for testing error handling, exceptions, and failure scenarios. Ensures code fails gracefully and provides clear error messages.

## Base Error Test Template

```python
def test_{function_name}_handles_{error_type}() -> None:
    """Test {function_name} properly handles {error_type}.

    Error condition: {description}
    Expected behavior: {what should happen}
    """
    # Arrange: Setup error condition
    {error_setup}

    # Act & Assert: Expect specific exception
    with pytest.raises({ExceptionType}) as exc_info:
        {function_name}({parameters})

    # Validate error details
    assert {error_message_validation}
    assert {error_state_validation}
```

## Input Validation Error Templates

### Template 1: Invalid Input Type

```python
def test_calculate_discount_rejects_invalid_amount_type() -> None:
    """Test calculate_discount raises TypeError for invalid amount type."""
    # Arrange
    user = User(subscription_type="premium")
    invalid_amount = "not_a_number"  # Wrong type

    # Act & Assert
    with pytest.raises(TypeError) as exc_info:
        calculate_discount(user=user, amount=invalid_amount)

    assert "amount must be Decimal" in str(exc_info.value)
```

### Template 2: Invalid Input Value

```python
def test_create_user_rejects_invalid_email() -> None:
    """Test create_user raises ValidationError for invalid email."""
    # Arrange
    invalid_data = {"email": "not-an-email", "name": "Test"}

    # Act & Assert
    with pytest.raises(ValidationError) as exc_info:
        create_user(invalid_data)

    assert "email" in str(exc_info.value).lower()
    assert "invalid format" in str(exc_info.value).lower()
```

### Template 3: Missing Required Field

```python
def test_create_order_requires_user_id() -> None:
    """Test create_order raises ValueError when user_id missing."""
    # Arrange
    order_data = {"items": [{"product_id": 1}]}  # Missing user_id

    # Act & Assert
    with pytest.raises(ValueError) as exc_info:
        create_order(order_data)

    assert "user_id" in str(exc_info.value)
    assert "required" in str(exc_info.value).lower()
```

### Template 4: Out of Range Value

```python
def test_set_age_rejects_negative_value() -> None:
    """Test set_age raises ValueError for negative age."""
    # Arrange
    user = User(email="test@example.com")
    invalid_age = -5

    # Act & Assert
    with pytest.raises(ValueError) as exc_info:
        user.set_age(invalid_age)

    assert "age must be positive" in str(exc_info.value).lower()
```

## Business Rule Violation Templates

### Template 5: Constraint Violation

```python
def test_apply_discount_enforces_maximum_discount() -> None:
    """Test apply_discount raises error when discount exceeds maximum."""
    # Arrange
    MAX_DISCOUNT = Decimal("0.50")  # 50% maximum
    amount = Decimal("100.00")
    excessive_discount = Decimal("60.00")  # 60% - exceeds maximum

    # Act & Assert
    with pytest.raises(BusinessRuleViolation) as exc_info:
        apply_discount(amount=amount, discount=excessive_discount)

    assert "maximum discount" in str(exc_info.value).lower()
    assert str(MAX_DISCOUNT) in str(exc_info.value)
```

### Template 6: State Violation

```python
def test_ship_order_requires_confirmed_state() -> None:
    """Test ship_order raises StateError if order not confirmed."""
    # Arrange
    order = Order(state=OrderState.PENDING)

    # Act & Assert
    with pytest.raises(InvalidStateError) as exc_info:
        order.ship()

    assert "confirmed" in str(exc_info.value).lower()
    assert order.state.value in str(exc_info.value)
```

## Resource Error Templates

### Template 7: Not Found Error

```python
def test_get_user_raises_not_found_for_nonexistent_id() -> None:
    """Test get_user raises NotFoundError for nonexistent user."""
    # Arrange
    nonexistent_id = 99999

    # Act & Assert
    with pytest.raises(NotFoundError) as exc_info:
        get_user(nonexistent_id)

    assert "user" in str(exc_info.value).lower()
    assert str(nonexistent_id) in str(exc_info.value)
```

### Template 8: Insufficient Resources

```python
def test_place_order_raises_error_for_insufficient_stock() -> None:
    """Test place_order raises InsufficientStockError."""
    # Arrange
    product = Product(id=1, stock=5)
    order_quantity = 10

    # Act & Assert
    with pytest.raises(InsufficientStockError) as exc_info:
        place_order(product=product, quantity=order_quantity)

    assert f"requested {order_quantity}" in str(exc_info.value)
    assert f"available {product.stock}" in str(exc_info.value)
```

## External Service Error Templates

### Template 9: Network Error Handling

```python
def test_fetch_data_handles_network_timeout(mocker) -> None:
    """Test fetch_data handles network timeout gracefully."""
    # Arrange
    mock_request = mocker.patch("requests.get")
    mock_request.side_effect = requests.Timeout("Connection timeout")

    # Act & Assert
    with pytest.raises(ServiceUnavailableError) as exc_info:
        fetch_data("https://api.example.com/data")

    assert "timeout" in str(exc_info.value).lower()
    # Verify original exception is chained
    assert isinstance(exc_info.value.__cause__, requests.Timeout)
```

### Template 10: API Error Response

```python
def test_process_payment_handles_api_error() -> None:
    """Test process_payment handles payment API errors."""
    # Arrange
    payment_service = MockPaymentService()
    payment_service.configure_to_fail(
        error_code="insufficient_funds",
        message="Card declined"
    )

    # Act & Assert
    with pytest.raises(PaymentError) as exc_info:
        process_payment(service=payment_service, amount=Decimal("100.00"))

    assert "declined" in str(exc_info.value).lower()
    assert exc_info.value.error_code == "insufficient_funds"
```

## Multiple Error Scenario Templates

### Template 11: Multiple Validation Errors

```python
def test_validate_user_returns_multiple_errors() -> None:
    """Test validate_user collects all validation errors."""
    # Arrange - Invalid on multiple fields
    invalid_data = {
        "email": "not-email",  # Invalid format
        "age": -5,             # Negative
        "name": "",            # Empty
    }

    # Act
    result = validate_user(invalid_data)

    # Assert - All errors collected
    assert result.is_valid is False
    assert len(result.errors) == 3
    assert any("email" in err.field for err in result.errors)
    assert any("age" in err.field for err in result.errors)
    assert any("name" in err.field for err in result.errors)
```

## Error Recovery Templates

### Template 12: Retry Logic

```python
def test_fetch_data_retries_on_transient_error(mocker) -> None:
    """Test fetch_data retries on transient errors."""
    # Arrange
    mock_request = mocker.patch("requests.get")
    # Fail twice, then succeed
    mock_request.side_effect = [
        requests.ConnectionError("Connection failed"),
        requests.ConnectionError("Connection failed"),
        MockResponse(data={"result": "success"})
    ]

    # Act
    result = fetch_data_with_retry(
        "https://api.example.com/data",
        max_retries=3
    )

    # Assert - Eventually succeeded
    assert result.data["result"] == "success"
    assert mock_request.call_count == 3
```

### Template 13: Fallback Behavior

```python
def test_get_config_falls_back_to_default() -> None:
    """Test get_config uses default when primary source fails."""
    # Arrange - Primary config source unavailable
    config_service = ConfigService(primary_source="unreachable-url")

    # Act - Should not raise, should use fallback
    result = config_service.get_config("setting_name")

    # Assert - Default value returned
    assert result == DEFAULT_CONFIG_VALUE
    assert config_service.used_fallback is True
```

## Exception Chaining Templates

### Template 14: Exception Context Preservation

```python
def test_save_user_chains_database_exception() -> None:
    """Test save_user preserves database exception context."""
    # Arrange
    database = MockDatabase()
    database.configure_to_raise(
        DatabaseError("Connection lost")
    )
    service = UserService(database=database)

    # Act & Assert
    with pytest.raises(ServiceError) as exc_info:
        service.save_user(User(email="test@example.com"))

    # Assert - Exception chained properly
    assert exc_info.value.__cause__ is not None
    assert isinstance(exc_info.value.__cause__, DatabaseError)
    assert "connection lost" in str(exc_info.value.__cause__).lower()
```

## Error Message Quality Templates

### Template 15: Actionable Error Messages

```python
def test_invalid_email_provides_actionable_message() -> None:
    """Test validation provides actionable error message."""
    # Arrange
    invalid_email = "user@invalid"

    # Act & Assert
    with pytest.raises(ValidationError) as exc_info:
        validate_email(invalid_email)

    error_message = str(exc_info.value)

    # Assert - Message is actionable
    assert invalid_email in error_message  # Shows problematic input
    assert "format" in error_message.lower()  # Explains issue
    assert "@" in error_message or "domain" in error_message  # Hints at fix
```

## Best Practices

### Comprehensive Error Testing
```python
class TestUserServiceErrors:
    """Comprehensive error testing for UserService."""

    def test_create_user_with_invalid_email(self):
        """Test invalid email error."""

    def test_create_user_with_missing_name(self):
        """Test missing name error."""

    def test_create_user_with_database_error(self):
        """Test database error handling."""

    def test_create_user_with_duplicate_email(self):
        """Test duplicate email error."""
```

### Error Test Checklist
- [ ] Specific exception type tested
- [ ] Error message validated
- [ ] Error context preserved (exception chaining)
- [ ] Actionable error message
- [ ] State remains consistent after error
- [ ] Resources cleaned up on error
- [ ] Logging verified (if applicable)

## References

- TDD Spec: `@~/.claude/kinhin/spec/tdd/tdd-spec.md`
- Unit Test Templates: `@~/.claude/kinhin/context/examples/tdd-unit-tests.md`
