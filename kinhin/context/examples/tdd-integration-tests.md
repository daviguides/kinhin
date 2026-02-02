# TDD Integration Test Templates

## Overview

Templates for integration tests that verify component interactions. Integration tests ensure components work together correctly.

## Base Integration Test Template

```python
def test_{component_a}_integrates_with_{component_b}() -> None:
    """Test integration between {component_a} and {component_b}.

    Describes what interaction is being tested.
    """
    # Arrange: Setup both components
    {setup_component_a}
    {setup_component_b}
    {setup_test_data}

    # Act: Trigger integration
    result = {integration_point}({parameters})

    # Assert: Validate integration
    assert {integration_assertion}
    {verify_component_a_state}
    {verify_component_b_state}
```

## Database Integration Templates

### Template 1: Database Write Integration

```python
def test_user_service_saves_to_database() -> None:
    """Test UserService integrates with database for user creation."""
    # Arrange
    service = UserService(database=test_database)
    user_data = {"email": "test@example.com", "name": "Test User"}

    # Act
    result = service.create_user(user_data)

    # Assert - Service returns success
    assert result.success is True
    assert result.user.id is not None

    # Assert - Database contains user
    db_user = test_database.query("users").where(id=result.user.id).first()
    assert db_user is not None
    assert db_user.email == user_data["email"]
    assert db_user.name == user_data["name"]
```

### Template 2: Database Read Integration

```python
def test_user_repository_loads_from_database() -> None:
    """Test UserRepository integrates with database for loading users."""
    # Arrange
    test_database.insert("users", {
        "id": 1,
        "email": "test@example.com",
        "name": "Test User"
    })
    repository = UserRepository(database=test_database)

    # Act
    user = repository.get_by_id(1)

    # Assert
    assert user is not None
    assert user.id == 1
    assert user.email == "test@example.com"
    assert user.name == "Test User"
```

### Template 3: Database Transaction Integration

```python
def test_order_service_handles_transaction_rollback() -> None:
    """Test OrderService rolls back transaction on failure."""
    # Arrange
    service = OrderService(database=test_database)
    order_data = {"user_id": 1, "amount": Decimal("100.00")}

    # Insert user
    test_database.insert("users", {"id": 1, "balance": Decimal("50.00")})

    # Act & Assert - Should raise InsufficientFundsError
    with pytest.raises(InsufficientFundsError):
        service.create_order(order_data)  # Will fail, insufficient balance

    # Assert - No order created (transaction rolled back)
    orders = test_database.query("orders").all()
    assert len(orders) == 0

    # Assert - User balance unchanged (transaction rolled back)
    user = test_database.query("users").where(id=1).first()
    assert user.balance == Decimal("50.00")
```

## API/Service Integration Templates

### Template 4: External API Integration

```python
def test_payment_service_integrates_with_stripe_api(mocker) -> None:
    """Test PaymentService integrates with Stripe API."""
    # Arrange
    mock_stripe = mocker.patch("stripe.Charge.create")
    mock_stripe.return_value = {
        "id": "ch_123",
        "status": "succeeded",
        "amount": 10000
    }

    service = PaymentService(api_key="test_key")
    payment_data = {"amount": Decimal("100.00"), "currency": "USD"}

    # Act
    result = service.process_payment(payment_data)

    # Assert - Service processed correctly
    assert result.success is True
    assert result.transaction_id == "ch_123"

    # Assert - API called correctly
    mock_stripe.assert_called_once_with(
        amount=10000,  # cents
        currency="usd",
        source="tok_visa"
    )
```

### Template 5: Microservice Integration

```python
def test_order_service_integrates_with_inventory_service(mocker) -> None:
    """Test OrderService integrates with InventoryService."""
    # Arrange
    mock_inventory = mocker.patch.object(
        InventoryService,
        "reserve_items",
        return_value=ReservationResult(success=True, reservation_id="res_123")
    )

    order_service = OrderService(inventory_service=InventoryService())
    order_data = {"items": [{"product_id": 1, "quantity": 2}]}

    # Act
    result = order_service.create_order(order_data)

    # Assert - Order created
    assert result.success is True

    # Assert - Inventory service called
    mock_inventory.assert_called_once()
    call_args = mock_inventory.call_args[1]
    assert call_args["items"][0]["product_id"] == 1
    assert call_args["items"][0]["quantity"] == 2
```

## Message Queue Integration Templates

### Template 6: Event Publishing Integration

```python
def test_user_service_publishes_user_created_event() -> None:
    """Test UserService publishes event when user created."""
    # Arrange
    event_bus = InMemoryEventBus()
    service = UserService(event_bus=event_bus)
    user_data = {"email": "test@example.com", "name": "Test User"}

    # Act
    result = service.create_user(user_data)

    # Assert - User created
    assert result.success is True

    # Assert - Event published
    events = event_bus.get_published_events()
    assert len(events) == 1
    assert events[0].type == "user.created"
    assert events[0].data["user_id"] == result.user.id
    assert events[0].data["email"] == user_data["email"]
```

### Template 7: Event Consumer Integration

```python
def test_notification_service_consumes_user_created_event() -> None:
    """Test NotificationService consumes user.created event."""
    # Arrange
    event_bus = InMemoryEventBus()
    email_service = MockEmailService()
    notification_service = NotificationService(
        event_bus=event_bus,
        email_service=email_service
    )

    event = UserCreatedEvent(
        user_id=1,
        email="test@example.com",
        name="Test User"
    )

    # Act
    event_bus.publish(event)
    event_bus.process_events()  # Process async events

    # Assert - Welcome email sent
    sent_emails = email_service.get_sent_emails()
    assert len(sent_emails) == 1
    assert sent_emails[0].to == "test@example.com"
    assert "Welcome" in sent_emails[0].subject
```

## Cache Integration Templates

### Template 8: Cache Write Integration

```python
def test_user_service_writes_to_cache_on_create() -> None:
    """Test UserService writes to cache when user created."""
    # Arrange
    cache = InMemoryCache()
    service = UserService(cache=cache)
    user_data = {"email": "test@example.com", "name": "Test User"}

    # Act
    result = service.create_user(user_data)

    # Assert - User created
    assert result.success is True

    # Assert - User in cache
    cached_user = cache.get(f"user:{result.user.id}")
    assert cached_user is not None
    assert cached_user["email"] == user_data["email"]
```

### Template 9: Cache Read Integration

```python
def test_user_service_reads_from_cache_before_database() -> None:
    """Test UserService checks cache before querying database."""
    # Arrange
    cache = InMemoryCache()
    database = MockDatabase()
    service = UserService(cache=cache, database=database)

    # Populate cache
    cache.set("user:1", {"id": 1, "email": "cached@example.com"})

    # Act
    user = service.get_user(1)

    # Assert - User returned from cache
    assert user.email == "cached@example.com"

    # Assert - Database not queried
    assert database.query_count == 0
```

## File System Integration Templates

### Template 10: File Write Integration

```python
def test_report_service_writes_to_file_system(tmp_path) -> None:
    """Test ReportService writes report to file system."""
    # Arrange
    service = ReportService(output_dir=tmp_path)
    report_data = {"title": "Test Report", "data": [1, 2, 3]}

    # Act
    result = service.generate_report(report_data)

    # Assert - Report generated successfully
    assert result.success is True
    assert result.file_path is not None

    # Assert - File exists and contains correct data
    report_file = tmp_path / result.file_path.name
    assert report_file.exists()

    content = report_file.read_text()
    assert "Test Report" in content
```

### Template 11: File Read Integration

```python
def test_import_service_reads_from_file_system(tmp_path) -> None:
    """Test ImportService reads data from file system."""
    # Arrange
    test_file = tmp_path / "data.csv"
    test_file.write_text("id,name\n1,Test\n2,User")

    service = ImportService()

    # Act
    result = service.import_file(test_file)

    # Assert - Data imported correctly
    assert result.success is True
    assert len(result.records) == 2
    assert result.records[0]["name"] == "Test"
    assert result.records[1]["name"] == "User"
```

## Component Chain Integration Templates

### Template 12: Multi-Component Integration

```python
def test_complete_order_flow_integrates_all_services() -> None:
    """Test complete order flow integrates all required services."""
    # Arrange - Setup all services
    database = TestDatabase()
    inventory_service = InventoryService(database=database)
    payment_service = PaymentService(api_key="test")
    notification_service = NotificationService()
    order_service = OrderService(
        database=database,
        inventory=inventory_service,
        payment=payment_service,
        notifications=notification_service
    )

    # Setup test data
    database.insert("users", {"id": 1, "email": "test@example.com"})
    database.insert("products", {"id": 1, "stock": 10})

    order_data = {
        "user_id": 1,
        "items": [{"product_id": 1, "quantity": 2}],
        "payment_method": "credit_card"
    }

    # Act
    result = order_service.create_order(order_data)

    # Assert - Order created
    assert result.success is True

    # Assert - Inventory updated
    product = database.query("products").where(id=1).first()
    assert product.stock == 8  # 10 - 2

    # Assert - Payment processed
    assert result.payment_status == "succeeded"

    # Assert - Notification sent
    # (Would check notification service mock/spy)
```

## Error Propagation Integration Templates

### Template 13: Error Propagation Through Layers

```python
def test_service_propagates_database_error_correctly() -> None:
    """Test service layer propagates database errors correctly."""
    # Arrange
    database = MockDatabase()
    database.configure_to_raise(DatabaseConnectionError("Connection lost"))
    service = UserService(database=database)

    # Act & Assert - Service propagates error with context
    with pytest.raises(ServiceError) as exc_info:
        service.get_user(1)

    # Assert - Error context preserved
    assert "database connection" in str(exc_info.value).lower()
    assert exc_info.value.__cause__.__class__ == DatabaseConnectionError
```

## Best Practices for Integration Tests

### Setup and Teardown

```python
@pytest.fixture
def test_database():
    """Provide isolated test database."""
    db = create_test_database()
    db.migrate()  # Setup schema
    yield db
    db.drop_all()  # Cleanup

@pytest.fixture
def test_services(test_database):
    """Provide configured service instances."""
    return {
        "user_service": UserService(database=test_database),
        "order_service": OrderService(database=test_database),
        "inventory_service": InventoryService(database=test_database)
    }
```

### Isolation Strategies

```python
# Use transactions for isolation
@pytest.fixture
def isolated_database(test_database):
    """Provide database with transaction rollback."""
    transaction = test_database.begin_transaction()
    yield test_database
    transaction.rollback()  # Undo all changes

# Use separate test instances
@pytest.fixture(scope="function")  # New instance per test
def user_service():
    """Provide fresh UserService instance."""
    return UserService()
```

## Integration Test Checklist

- [ ] Both/all components properly configured
- [ ] Test data setup complete
- [ ] Integration point clearly tested
- [ ] Both/all component states validated
- [ ] Proper cleanup/teardown
- [ ] Isolation from other tests
- [ ] Error scenarios tested
- [ ] Transaction handling verified

## References

- TDD Spec: `@~/.claude/kinhin/spec/tdd/tdd-spec.md`
- Unit Test Templates: `@~/.claude/kinhin/context/examples/tdd-unit-tests.md`
- Error Handling Templates: `@~/.claude/kinhin/context/examples/tdd-error-handling-tests.md`
