# TDD Performance Test Templates

## Overview

Templates for performance and load testing. Validates performance requirements are met.

## Base Performance Test Template

```python
import pytest
import time

@pytest.mark.performance
def test_{function_name}_meets_performance_requirement() -> None:
    """Test {function_name} meets {requirement} performance requirement."""
    # Arrange
    {setup_test_data}
    PERFORMANCE_THRESHOLD = {threshold}  # e.g., 0.1 seconds

    # Act
    start_time = time.perf_counter()
    result = {function_name}({parameters})
    elapsed_time = time.perf_counter() - start_time

    # Assert
    assert elapsed_time < PERFORMANCE_THRESHOLD, (
        f"Execution took {elapsed_time:.4f}s, "
        f"threshold is {PERFORMANCE_THRESHOLD}s"
    )
    assert {result_validation}
```

## Execution Time Templates

### Template 1: Single Operation Performance

```python
@pytest.mark.performance
def test_calculate_discount_completes_within_100ms() -> None:
    """Test calculate_discount completes within 100ms."""
    # Arrange
    user = User(subscription_type="premium")
    amount = Decimal("1000.00")
    MAX_EXECUTION_TIME = 0.1  # 100ms

    # Act
    start = time.perf_counter()
    result = calculate_discount(user=user, amount=amount)
    duration = time.perf_counter() - start

    # Assert
    assert duration < MAX_EXECUTION_TIME
    assert result >= Decimal("0")
```

### Template 2: Batch Operation Performance

```python
@pytest.mark.performance
def test_process_batch_of_1000_records_within_5_seconds() -> None:
    """Test batch processing of 1000 records completes within 5 seconds."""
    # Arrange
    records = [generate_test_record(i) for i in range(1000)]
    MAX_BATCH_TIME = 5.0  # 5 seconds

    # Act
    start = time.perf_counter()
    results = process_batch(records)
    duration = time.perf_counter() - start

    # Assert
    assert duration < MAX_BATCH_TIME, (
        f"Processed 1000 records in {duration:.2f}s, "
        f"expected < {MAX_BATCH_TIME}s"
    )
    assert len(results) == 1000
```

## Throughput Templates

### Template 3: Operations Per Second

```python
@pytest.mark.performance
def test_api_handles_100_requests_per_second() -> None:
    """Test API can handle minimum 100 requests/second."""
    # Arrange
    MIN_THROUGHPUT = 100  # requests per second
    TEST_DURATION = 5  # seconds
    total_requests = 0

    # Act
    start = time.perf_counter()
    while time.perf_counter() - start < TEST_DURATION:
        response = api_call()
        assert response.success
        total_requests += 1

    duration = time.perf_counter() - start
    throughput = total_requests / duration

    # Assert
    assert throughput >= MIN_THROUGHPUT, (
        f"Throughput {throughput:.2f} req/s < "
        f"minimum {MIN_THROUGHPUT} req/s"
    )
```

## Load Testing Templates

### Template 4: Concurrent Load Test

```python
import concurrent.futures

@pytest.mark.performance
@pytest.mark.slow
def test_service_handles_concurrent_requests() -> None:
    """Test service handles 50 concurrent requests."""
    # Arrange
    CONCURRENT_REQUESTS = 50
    MAX_RESPONSE_TIME = 2.0  # seconds

    def make_request(request_id: int):
        start = time.perf_counter()
        result = service.process_request({"id": request_id})
        duration = time.perf_counter() - start
        return duration, result

    # Act
    with concurrent.futures.ThreadPoolExecutor(
        max_workers=CONCURRENT_REQUESTS
    ) as executor:
        futures = [
            executor.submit(make_request, i)
            for i in range(CONCURRENT_REQUESTS)
        ]
        results = [f.result() for f in futures]

    # Assert - All requests completed
    assert len(results) == CONCURRENT_REQUESTS

    # Assert - All within time limit
    durations = [r[0] for r in results]
    assert all(d < MAX_RESPONSE_TIME for d in durations), (
        f"Some requests exceeded {MAX_RESPONSE_TIME}s: "
        f"max={max(durations):.2f}s"
    )
```

## Memory Usage Templates

### Template 5: Memory Efficiency

```python
import tracemalloc

@pytest.mark.performance
def test_process_large_file_memory_efficient() -> None:
    """Test large file processing uses < 100MB memory."""
    # Arrange
    MAX_MEMORY_MB = 100
    large_file = create_test_file(size_mb=500)

    # Act
    tracemalloc.start()
    process_file(large_file)
    current, peak = tracemalloc.get_traced_memory()
    tracemalloc.stop()

    peak_mb = peak / (1024 * 1024)

    # Assert
    assert peak_mb < MAX_MEMORY_MB, (
        f"Peak memory {peak_mb:.2f}MB exceeds {MAX_MEMORY_MB}MB"
    )
```

## Scaling Templates

### Template 6: Linear Scaling

```python
@pytest.mark.performance
@pytest.mark.parametrize("data_size", [100, 1000, 10000])
def test_processing_scales_linearly(data_size: int) -> None:
    """Test processing time scales linearly with data size."""
    # Arrange
    data = [generate_record() for _ in range(data_size)]
    ACCEPTABLE_FACTOR = 1.5  # Allow up to 50% slower than linear

    # Act
    start = time.perf_counter()
    process_data(data)
    duration = time.perf_counter() - start

    # Calculate expected linear time (baseline: 100 items in 0.1s)
    expected_linear = 0.1 * (data_size / 100)
    max_acceptable = expected_linear * ACCEPTABLE_FACTOR

    # Assert
    assert duration <= max_acceptable, (
        f"Processing {data_size} items took {duration:.3f}s, "
        f"expected ≤ {max_acceptable:.3f}s (linear with margin)"
    )
```

## Database Query Performance Templates

### Template 7: Query Performance

```python
@pytest.mark.performance
@pytest.mark.database
def test_user_search_query_completes_within_100ms() -> None:
    """Test user search query completes within 100ms."""
    # Arrange - Populate test database with 10,000 users
    populate_test_database(user_count=10000)
    MAX_QUERY_TIME = 0.1  # 100ms

    # Act
    start = time.perf_counter()
    results = search_users(query="test")
    duration = time.perf_counter() - start

    # Assert
    assert duration < MAX_QUERY_TIME, (
        f"Query took {duration*1000:.2f}ms, expected < 100ms"
    )
    assert len(results) > 0  # Found matching users
```

## Cache Performance Templates

### Template 8: Cache Hit Performance

```python
@pytest.mark.performance
def test_cache_hit_is_100x_faster_than_database() -> None:
    """Test cache retrieval is significantly faster than database."""
    # Arrange
    user_id = 1
    service = UserService(cache=test_cache, database=test_database)

    # Warm up cache
    service.get_user(user_id)

    # Act - Measure database query
    test_cache.clear()
    start = time.perf_counter()
    user_from_db = service.get_user(user_id)
    db_time = time.perf_counter() - start

    # Act - Measure cache query
    start = time.perf_counter()
    user_from_cache = service.get_user(user_id)
    cache_time = time.perf_counter() - start

    # Assert - Cache is significantly faster
    speedup = db_time / cache_time
    assert speedup >= 100, (
        f"Cache only {speedup:.1f}x faster, expected ≥100x"
    )
```

## pytest-benchmark Integration

### Template 9: Benchmark Test

```python
def test_function_performance_benchmark(benchmark):
    """Benchmark function performance over multiple runs."""
    # Arrange
    test_data = generate_test_data()

    # Act & Assert - pytest-benchmark handles timing
    result = benchmark(function_to_test, test_data)

    # Additional assertions
    assert result.is_valid
```

### Template 10: Benchmark with Setup

```python
def test_with_setup_benchmark(benchmark):
    """Benchmark with setup/teardown."""
    # Setup function (not timed)
    def setup():
        return generate_expensive_test_data()

    # Function to benchmark
    def run(data):
        return process_data(data)

    # Teardown function (not timed)
    def teardown(data):
        cleanup(data)

    # Act & Assert
    result = benchmark.pedantic(
        run,
        setup=setup,
        teardown=teardown,
        rounds=100
    )
```

## Performance Regression Detection

### Template 11: Performance Baseline

```python
@pytest.mark.performance
def test_api_response_time_baseline() -> None:
    """Test API response time hasn't regressed."""
    # Arrange
    BASELINE_TIME = 0.05  # 50ms baseline
    REGRESSION_THRESHOLD = 1.2  # Allow 20% slower
    MAX_ACCEPTABLE = BASELINE_TIME * REGRESSION_THRESHOLD

    # Act
    start = time.perf_counter()
    response = api_endpoint()
    duration = time.perf_counter() - start

    # Assert
    assert duration <= MAX_ACCEPTABLE, (
        f"Response time {duration*1000:.2f}ms exceeds "
        f"baseline {BASELINE_TIME*1000:.2f}ms by "
        f"{((duration/BASELINE_TIME)-1)*100:.1f}%"
    )
```

## Best Practices

### Performance Test Structure
```python
@pytest.mark.performance
@pytest.mark.slow  # Mark long-running tests
def test_name():
    # Always include:
    # 1. Clear performance requirement
    # 2. Adequate warm-up if needed
    # 3. Multiple iterations for stability
    # 4. Helpful assertion messages
    # 5. Result validation (not just timing)
    pass
```

### Performance Test Markers
```toml
# pyproject.toml
[tool.pytest.ini_options]
markers = [
    "performance: Performance tests (deselect with '-m \"not performance\"')",
    "slow: Slow tests that take >1 second",
]
```

## References

- TDD Spec: `@~/.claude/kinhin/spec/tdd/tdd-spec.md`
- Python Testing Tools: `@~/.claude/kinhin/spec/python/python-testing-tools-spec.md`
