# TDD Unit Test Templates (Python)

Structure, naming, docstrings, type hints and AAA: `python-tdd-spec.md`. These templates show only what the spec does not.

## Scenario Matrix → Parametrize

One parametrized test per matrix row group; `ids` name each case.

```python
@pytest.mark.parametrize(
    "input_value,expected_output",
    [
        (10, 20),            # happy path
        (0, 0),              # edge: zero
        (sys.maxsize, sys.maxsize * 2),  # edge: maximum
    ],
    ids=["normal-value", "zero-value", "maximum-value"],
)
def test_double_value(input_value: int, expected_output: int) -> None:
    """Test double_value with various inputs."""
    # Act
    result = double_value(input_value)

    # Assert
    assert result == expected_output
```

## Side Effects: Assert the Outcome, Not the Call

Assert observable state, never how an internal collaborator was called (see Anti-Pattern 17).

```python
def test_save_user_persists_record(repository: InMemoryUserRepository) -> None:
    """Test save_user persists the user."""
    # Arrange
    user = User(email="test@example.com", name="Test User")

    # Act
    result = save_user(user, repository=repository)

    # Assert
    assert result.success is True
    assert repository.get_by_email(user.email) == user
```

## State Transitions

Test the allowed transition and the forbidden one.

```python
class TestOrderStateMachine:
    """Tests for order state transitions."""

    def test_order_transitions_from_pending_to_confirmed(self) -> None:
        """Test order can transition from pending to confirmed."""
        order = Order(state=OrderState.PENDING)

        order.confirm()

        assert order.state == OrderState.CONFIRMED
        assert order.confirmed_at is not None

    def test_order_cannot_transition_from_cancelled_to_confirmed(self) -> None:
        """Test order cannot transition from cancelled to confirmed."""
        order = Order(state=OrderState.CANCELLED)

        with pytest.raises(InvalidStateTransitionError, match="Cannot confirm cancelled order"):
            order.confirm()
```

## Tests You Do Not Write

Return-type checks (`assert isinstance(result, int)`) when the code is type-checked: the type checker proves them (3-Question Gate, Q1).
