# TDD Performance Test Templates (Python)

Performance tests never assert wall-clock thresholds inside the parallel, randomized suite (`-n auto -p randomly`): timing there is noise, and the result is a flake. They run as a separate, serial benchmark run, marked `performance` and excluded from the gate.

```bash
pytest -m "not scaffold and not performance"       # gate: permanent tests, no timing tests
pytest -m performance -p no:xdist -p no:randomly   # benchmark run, serial
```

## pytest-benchmark

```python
@pytest.mark.performance
def test_function_performance_benchmark(benchmark: BenchmarkFixture) -> None:
    """Benchmark function performance over multiple runs."""
    test_data = generate_test_data()

    result = benchmark(function_to_test, test_data)

    assert result.is_valid  # validate the result, not only the timing
```

With setup and teardown outside the timed section:

```python
@pytest.mark.performance
def test_with_setup_benchmark(benchmark: BenchmarkFixture) -> None:
    """Benchmark with setup/teardown."""
    benchmark.pedantic(
        process_data,
        setup=lambda: ((generate_expensive_test_data(),), {}),
        rounds=100,
    )
```

## Marker Registration

Add to the lifecycle `markers` list (`--strict-markers`):

```toml
[tool.pytest.ini_options]
markers = [
    "performance: benchmark run only, excluded from the parallel gate",
]
```
