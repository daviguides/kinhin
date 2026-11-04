# TDD Error Handling Test Templates (Python)

`pytest.raises` with detail validation is in `python-tdd-spec.md`. These templates cover what it does not.

## Exception Chaining

The original cause must survive the wrap.

```python
def test_save_user_chains_database_exception() -> None:
    """Test save_user preserves database exception context."""
    database = FailingDatabase(DatabaseError("Connection lost"))
    service = UserService(database=database)

    with pytest.raises(ServiceError) as exc_info:
        service.save_user(User(email="test@example.com"))

    assert isinstance(exc_info.value.__cause__, DatabaseError)
```

## Multiple Validation Errors

```python
def test_validate_user_returns_multiple_errors() -> None:
    """Test validate_user collects all validation errors."""
    invalid_data = {"email": "not-email", "age": -5, "name": ""}

    result = validate_user(invalid_data)

    assert result.is_valid is False
    assert {err.field for err in result.errors} == {"email", "age", "name"}
```

## Retry on Transient Error

Patching the external boundary (`requests.get`) is allowed; assert the outcome.

```python
def test_fetch_data_retries_on_transient_error(mocker: MockerFixture) -> None:
    """Test fetch_data retries on transient errors."""
    mocker.patch(
        "requests.get",
        side_effect=[
            requests.ConnectionError("Connection failed"),
            requests.ConnectionError("Connection failed"),
            MockResponse(data={"result": "success"}),
        ],
    )

    result = fetch_data_with_retry("https://api.example.com/data", max_retries=3)

    assert result.data["result"] == "success"
```

## Error Test Checklist

- [ ] Specific exception type tested
- [ ] Error message validated and actionable (shows the input, explains the issue)
- [ ] Error context preserved (exception chaining)
- [ ] State remains consistent after error
- [ ] Resources cleaned up on error
- [ ] Logging verified (if applicable)
