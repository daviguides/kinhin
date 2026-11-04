# TDD Anti-Patterns (LLM-Optimized)

## Test Generation Anti-Patterns

### Anti-Pattern 1: Sequential Test Generation

**Problem**: Generating tests one at a time instead of in batches.

**Why It's Bad**:
- Slower development
- Inconsistent patterns
- Poor context utilization
- Higher hallucination risk

**Bad Example**:
```
Generate test for function_a
[wait for result]
Generate test for function_b
[wait for result]
Generate test for function_c
```

**Good Example**:
```
Generate all tests for [function_a, function_b, function_c] simultaneously:
- Use consistent AAA pattern
- Same naming conventions
- Unified fixture usage
```

### Anti-Pattern 2: Implementation Before Tests

**Problem**: Writing implementation before test suite.

**Why It's Bad**:
- Loses TDD benefits
- Tests may be biased toward implementation
- Missing edge cases
- No specification before code

**Bad Example**:
```python
# Implementation first
def calculate_discount(user, amount):
    if user.subscription_type == "premium":
        return amount * 0.1
    return Decimal("0")

# Tests second (biased toward implementation)
def test_premium_gets_discount():
    # Only tests what's implemented
```

**Good Example**:
```python
# Tests first (specification)
def test_discount_scenarios():
    """Test discount for all subscription types."""
    # premium, standard, enterprise, trial...
    # Forces complete implementation

# Implementation second (satisfies tests)
```

### Anti-Pattern 3: Incomplete Scenario Coverage

**Problem**: Only testing happy path scenarios.

**Why It's Bad**:
- Bugs in edge cases
- No error handling validation
- False confidence
- Production failures

**Bad Example**:
```python
def test_calculate_discount():
    """Only test normal case."""
    user = User(subscription_type="premium")
    amount = Decimal("100")
    result = calculate_discount(user, amount)
    assert result > Decimal("0")
```

**Good Example**:
```python
# Use scenario matrix
@pytest.mark.parametrize("user_type,amount,expected", [
    ("premium", Decimal("100"), Decimal("10")),  # Happy path
    ("standard", Decimal("100"), Decimal("0")),  # Different type
    ("premium", Decimal("0"), Decimal("0")),     # Edge case
    ("premium", Decimal("-100"), ValueError),    # Error case
])
def test_calculate_discount_all_scenarios(...):
    # Comprehensive coverage
```

## Test Structure Anti-Patterns

### Anti-Pattern 4: Magic Numbers Without Explanation
Literals that carry a business rule get a name (constant or test name). Owned by zazen (naming spec → Magic Values Prohibition).

### Anti-Pattern 5: Multiple Concerns Per Test

**Problem**: Testing multiple unrelated things in one test.

**Why It's Bad**:
- Hard to understand what failed
- Difficult to debug
- Breaks single responsibility
- Poor test names

**Bad Example**:
```python
def test_user_operations():
    """Tests multiple things."""
    # Create user
    user = create_user(data)
    assert user.id is not None

    # Update user
    update_user(user, new_data)
    assert user.name == "New Name"

    # Delete user
    delete_user(user)
    assert get_user(user.id) is None
```

**Good Example**:
```python
def test_create_user_assigns_id():
    """Test user creation assigns ID."""
    user = create_user(data)
    assert user.id is not None

def test_update_user_changes_name():
    """Test user update changes name."""
    user = create_user(data)
    update_user(user, {"name": "New Name"})
    assert user.name == "New Name"

def test_delete_user_removes_from_database():
    """Test delete removes user."""
    user = create_user(data)
    delete_user(user)
    assert get_user(user.id) is None
```

### Anti-Pattern 6: Hidden Test Setup

**Problem**: Complex setup hidden in fixtures or helpers.

**Why It's Bad**:
- Hard to understand test context
- Difficult to debug
- Obscures test data
- Makes tests fragile

**Bad Example**:
```python
@pytest.fixture
def complex_setup():
    # 50 lines of hidden setup
    user = create_user(...)
    order = create_order(...)
    items = create_items(...)
    # ... more setup
    return user, order, items

def test_something(complex_setup):
    # What's in complex_setup?
    user, order, items = complex_setup
    result = function(user)  # Unclear what's being tested
```

**Good Example**:
```python
def test_calculate_order_total():
    """Test order total calculation."""
    # Setup visible in test
    items = [
        OrderItem(price=Decimal("10"), quantity=2),
        OrderItem(price=Decimal("5"), quantity=3)
    ]
    # Clear what's being tested
    result = calculate_total(items)
    assert result == Decimal("35")  # 10*2 + 5*3
```

## Assertion Anti-Patterns

### Anti-Pattern 7: Weak Assertions

**Problem**: Assertions that don't validate enough.

**Why It's Bad**:
- False positives
- Bugs slip through
- Low confidence
- Hallucinated behavior undetected

**Bad Example**:
```python
def test_create_user():
    result = create_user(data)
    assert result is not None  # Too weak!
```

**Good Example**:
```python
def test_create_user():
    """Test user creation returns complete user object."""
    result = create_user(data)

    # Strong, specific assertions
    assert isinstance(result, User)
    assert result.id is not None
    assert result.email == data["email"]
    assert result.name == data["name"]
    assert result.created_at is not None
```

### Anti-Pattern 8: No Assertion Messages

**Problem**: Assertions without helpful messages.

**Why It's Bad**:
- Hard to debug failures
- Unclear what went wrong
- Wastes time investigating
- Poor developer experience

**Bad Example**:
```python
def test_calculation():
    result = calculate(10, 20)
    assert result == 30  # Fails with "assert 25 == 30"
```

**Good Example**:
```python
def test_addition_calculation():
    """Test calculate performs addition correctly."""
    a, b = 10, 20
    result = calculate(a, b)

    assert result == 30, (
        f"Expected {a} + {b} = 30, but got {result}"
    )
```

## Hallucination-Prone Anti-Patterns

### Anti-Pattern 9: Implicit Conversions

**Problem**: Relying on implicit type conversions.

**Why It's Bad**:
- Hides bugs
- LLMs may hallucinate conversions
- Unclear test intent
- Fragile tests

**Bad Example**:
```python
def test_amount_calculation():
    result = calculate_total(10.5)  # Implicit float
    assert result == 10.5  # May fail with Decimal
```

**Good Example**:
```python
def test_amount_calculation():
    """Test calculation with explicit Decimal types."""
    amount = Decimal("10.50")  # Explicit
    result = calculate_total(amount)

    # The return type is the type checker's job (rung 1), not an assertion
    assert result == Decimal("10.50")
```

### Anti-Pattern 10: Untested Constraints

**Problem**: Not testing business constraints.

**Why It's Bad**:
- LLM may hallucinate constraint violations
- No safety net
- Production bugs
- Invalid data processed

**Bad Example**:
```python
def test_discount():
    # No constraint testing
    result = calculate_discount(user, amount)
    assert result > 0
```

**Good Example**:
```python
def test_discount_never_exceeds_50_percent():
    """Test discount constraint: max 50%."""
    MAX_DISCOUNT_RATE = Decimal("0.50")
    amount = Decimal("1000")

    result = calculate_discount(premium_user, amount)

    max_possible = amount * MAX_DISCOUNT_RATE
    assert result <= max_possible, (
        f"Discount {result} exceeds maximum {max_possible}"
    )
```

### Anti-Pattern 11: No Cross-Function Consistency Checks

**Problem**: Not validating data consistency across functions.

**Why It's Bad**:
- Hallucinated relationships
- Data integrity issues
- Integration bugs
- Inconsistent state

**Bad Example**:
```python
def test_create_and_get_user():
    user = create_user(data)
    retrieved = get_user(user.id)
    # No consistency validation!
```

**Good Example**:
```python
def test_create_and_get_user_maintains_consistency():
    """Test data consistency across create and get."""
    original_data = {"email": "test@example.com", "name": "Test"}

    # Create
    created_user = create_user(original_data)

    # Retrieve
    retrieved_user = get_user(created_user.id)

    # Validate consistency
    assert retrieved_user.id == created_user.id
    assert retrieved_user.email == original_data["email"]
    assert retrieved_user.name == original_data["name"]
    assert retrieved_user.created_at == created_user.created_at
```

## Maintenance Anti-Patterns

### Anti-Pattern 12: Brittle Tests

**Problem**: Tests break with minor code changes.

**Why It's Bad**:
- High maintenance cost
- Discourages refactoring
- False negatives
- Developer frustration

**Bad Example**:
```python
def test_format_user():
    result = format_user(user)
    # Brittle: depends on exact format
    assert result == "User: Test (test@example.com) [ID: 1]"
```

**Good Example**:
```python
def test_format_user_includes_all_fields():
    """Test format includes all user fields."""
    user = User(id=1, name="Test", email="test@example.com")
    result = format_user(user)

    # Strict on values, free on layout: each value appears exactly once,
    # and the ID is a whole token (not any "1" in the string)
    assert result.count("Test") == 1
    assert result.count("test@example.com") == 1
    assert re.search(r"\b1\b", result)
    # When the exact layout IS a contract with a consumer, assert the full
    # string and tag the test @contract instead.
```

### Anti-Pattern 13: Over-Mocking

**Problem**: Mocking too much, testing nothing real.

**Why It's Bad**:
- Tests pass but code broken
- False confidence
- Doesn't catch integration issues
- Wastes time

**Bad Example**:
```python
def test_process_order(mocker):
    # Mock everything!
    mocker.patch("module.validate_order", return_value=True)
    mocker.patch("module.calculate_total", return_value=100)
    mocker.patch("module.save_order", return_value=True)
    mocker.patch("module.send_email", return_value=True)

    result = process_order(order)
    assert result == True  # Testing nothing!
```

**Good Example**:
```python
def test_process_order():
    """Test order processing logic (integration test)."""
    # Only mock external dependencies
    with patch("module.send_email") as mock_email:
        result = process_order(valid_order)

        # Test real logic
        assert result.total == calculate_expected_total(valid_order)
        assert result.status == OrderStatus.CONFIRMED

        # Verify external call
        mock_email.assert_called_once()
```

## LLM-Specific Anti-Patterns

### Anti-Pattern 14: Context Fragmentation
Same anti-pattern as #1 (Sequential Test Generation) seen from the prompt side: related tests split across prompts. Rule: tdd-spec → Batch Processing Over Sequential.

### Anti-Pattern 15: Insufficient Property Testing

**Problem**: Not using property-based tests for invariants.

**Why It's Bad**:
- LLM may hallucinate constraint violations
- Edge cases missed
- No systematic validation
- False confidence

**Bad Example**:
```python
def test_discount_calculation():
    # Only one example
    result = calculate_discount(user, Decimal("100"))
    assert result == Decimal("10")
```

**Good Example**:
```python
@given(st.decimals(min_value=0, max_value=10000))
def test_discount_never_negative(amount: Decimal):
    """Property: discount is never negative."""
    result = calculate_discount(premium_user, amount)
    assert result >= Decimal("0")

@given(st.decimals(min_value=0, max_value=10000))
def test_discount_never_exceeds_amount(amount: Decimal):
    """Property: discount never exceeds amount."""
    result = calculate_discount(premium_user, amount)
    assert result <= amount
```

## Lifecycle Anti-Patterns

### Anti-Pattern 16: Immortal Scaffolding

**Problem**: Characterization or construction tests outliving the work they guided.

**Why It's Bad**:
- Suite grows monotonically
- Scaffold tests assert implementation shape, not behavior
- Refactoring breaks them, assistant burns time fixing tests instead of the feature
- False confidence from high test count

**Bad Example**:
```python
# Written during refactoring as characterization — never removed
def test_legacy_format_output():
    """Characterization: captures current output shape."""
    result = format_report(data)
    assert result == "Header: Test\nBody: content\nFooter: v2"
```

**Good Example**:
```python
# Characterization test tagged and pruned after refactoring
@pytest.mark.scaffold  # Dies at prune
def test_legacy_format_output():
    """Characterization: captures current output during refactor."""
    result = format_report(data)
    assert result == "Header: Test\nBody: content\nFooter: v2"

# After refactoring, scaffold deleted. Surviving decision test:
@pytest.mark.decision(reason="Report must include version in footer")
def test_report_footer_contains_version():
    result = format_report(data)
    assert "v2" in result.split("\n")[-1]
```

### Anti-Pattern 17: Mechanism Assertions

**Problem**: Asserting call counts, call order, or argument matching instead of observable behavior.

**Why It's Bad**:
- Tests the wiring, not the outcome
- Every refactoring breaks them even when behavior is unchanged
- Distinct from Over-Mocking (#13): this is about what you ASSERT, not what you SET UP
- Prime candidates for pruning — they encode no decision

**Bad Example**:
```python
def test_process_order(mocker):
    mock_db = mocker.patch("module.save_order")
    mock_email = mocker.patch("module.send_email")

    process_order(order)

    # Mechanism assertions — testing HOW, not WHAT
    assert mock_db.call_count == 1
    assert mock_db.call_args[0][0].status == "confirmed"
    mock_email.assert_called_once_with(
        to="user@example.com",
        subject="Order Confirmed"
    )
```

**Good Example**:
```python
def test_process_order_confirms_and_notifies():
    """Test observable outcome: order is confirmed and user is notified."""
    result = process_order(valid_order)

    # Behavior assertions — testing WHAT happened
    assert result.status == OrderStatus.CONFIRMED
    assert result.total == calculate_expected_total(valid_order)

    saved = get_order(result.id)
    assert saved.status == OrderStatus.CONFIRMED

    notifications = get_pending_notifications(result.user_id)
    assert any(n.type == "order_confirmed" for n in notifications)
```

### Anti-Pattern 18: Coverage as Retention Metric

**Problem**: Using branch or line coverage to decide which tests to keep after pruning.

**Why It's Bad**:
- Coverage rewards keeping shape-tests that touch trivial branches
- A test asserting `mock.call_count == 1` covers the call site — 100% branch, 0% verification
- Coverage measures what was EXECUTED, not what was VERIFIED
- Actively opposes pruning: removing scaffold drops coverage, triggering "add tests back"

**Bad Example**:
```
# Post-prune review
"Coverage dropped from 94% to 87% after pruning scaffolds.
 Restoring 12 mock-assertion tests to meet the 90% gate."
```

**Good Example**:
```
# Post-prune review
"Mutation parity: K₁ ⊇ K₀ — all mutants killed pre-prune are still killed.
 Branch coverage dropped from 94% to 87% (expected, scaffold removal).
 No load-bearing verification lost."
```

### Anti-Pattern 19: Mirror Test

**Problem**: Editing a permanent test's assertion to match changed code instead of fixing the code.

**Why It's Bad**:
- The permanent test (`@decision`, `@contract`, `@incident`) carries authority
- Changing its assertion to match new output strips that authority silently
- Worse than deleting: the test keeps its badge while carrying no verification
- The original decision may still be correct — the code may be the bug

**Bad Example**:
```python
# Original @decision test
@pytest.mark.decision(reason="Discount never exceeds 50%")
def test_discount_cap():
    result = calculate_discount(premium_user, Decimal("1000"))
    assert result <= Decimal("500")  # 50% cap

# After code change, developer "fixes" the test:
@pytest.mark.decision(reason="Discount never exceeds 50%")
def test_discount_cap():
    result = calculate_discount(premium_user, Decimal("1000"))
    assert result <= Decimal("600")  # "Fixed" to match new behavior
    # ⚠️ The reason still says 50% but the assertion says 60%
```

**Good Example**:
```python
# Test fails → test is the spec → fix the CODE
def calculate_discount(user, amount):
    MAX_DISCOUNT_RATE = Decimal("0.50")  # Fix: restore the 50% cap
    # ...

# OR: the decision genuinely changed (confirmed by authority)
@pytest.mark.decision(reason="Discount cap raised to 60% per PROJ-456")
def test_discount_cap():
    result = calculate_discount(premium_user, Decimal("1000"))
    assert result <= Decimal("600")  # Updated with new ref
```

## Runner Anti-Patterns

### Anti-Pattern 20: Bail on First Failure
`-x` / `--bail` / `--maxfail=1` in the fix loop: one visible failure hides the rest, the fix→run cycle goes serial, cause clustering is impossible. Rule: tdd-spec → Runner Contract #1 (No Bail).

### Anti-Pattern 21: Order-Dependent Tests
A test passes only after another ran (shared global, DB row, file): it verifies order, not behavior. Rule: tdd-spec → Test Independence and Runner Contract #2–#3 (an order-only failure is `isolation-dependent`, a bug to fix).

### Anti-Pattern 22: Retry Until Green
Auto-retry (`reruns`, `--retryTimes`) as a gate reports isolation bugs and flakes as "passed". Rule: tdd-spec → Runner Contract #4 (three-run protocol, three verdicts; retry only inside diagnostic mode).

### Anti-Pattern 23: Diff-String Debugging
Reading pretty-printed ANSI diffs instead of structured operands wastes tokens and hides the delta. Rule: tdd-spec → Runner Contract #6 (machine-readable output, cause clustering).
