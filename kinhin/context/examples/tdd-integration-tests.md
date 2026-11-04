# TDD Integration Test Templates (Python)

Integration tests verify component interactions through real collaborators. Prefer real or in-memory fakes (`InMemoryEventBus`, `InMemoryCache`, `tmp_path`) to mocks; mock only an external boundary (third-party API), and assert on the result, not on the mock's call arguments.

## Transaction Rollback on Failure

```python
def test_order_service_handles_transaction_rollback(isolated_database: Database) -> None:
    """Test OrderService rolls back transaction on failure."""
    # Arrange
    service = OrderService(database=isolated_database)
    isolated_database.insert("users", {"id": 1, "balance": Decimal("50.00")})

    # Act
    with pytest.raises(InsufficientFundsError):
        service.create_order({"user_id": 1, "amount": Decimal("100.00")})

    # Assert - no order created, balance unchanged
    assert isolated_database.query("orders").all() == []
    user = isolated_database.query("users").where(id=1).first()
    assert user.balance == Decimal("50.00")
```

## Isolation Fixtures

```python
@pytest.fixture
def test_database() -> Iterator[Database]:
    """Provide isolated test database."""
    db = create_test_database()
    db.migrate()
    yield db
    db.drop_all()


@pytest.fixture
def isolated_database(test_database: Database) -> Iterator[Database]:
    """Provide database with transaction rollback."""
    transaction = test_database.begin_transaction()
    yield test_database
    transaction.rollback()
```

Function-scoped fixtures by default: a new instance per test (parallel, randomized runs).

## Integration Test Checklist

- [ ] Both/all components properly configured
- [ ] Test data setup complete
- [ ] Integration point clearly tested
- [ ] Both/all component states validated
- [ ] Proper cleanup/teardown
- [ ] Isolation from other tests
- [ ] Error scenarios tested
- [ ] Transaction handling verified
