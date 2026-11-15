# Lifecycle Tags — Syntax and Collapse

Tag semantics (temporary and permanent tags, graduation criteria, rulings) are in `tdd-prune-spec.md` → Lifecycle Tags. Tags are written by the classification pass (`kinhin tag`, or the pruner), never by the author while writing tests.

## Marker Syntax per Language

### Python

The classification pass (`kinhin tag`, or the pruner) writes one marker per test. Untagged = `scaffold`. Each permanent marker requires its kwarg: `decision(reason=)`, `contract(party=)`, `incident(ref=)`.

**Registration** — `pyproject.toml`:
```toml
[tool.pytest.ini_options]
markers = [
    "scaffold: construction-time verification, deleted at prune",
    "decision: encodes a non-obvious decision (reason required)",
    "contract: pins a boundary another party depends on (party required)",
    "incident: reproduces a production failure (ref required)",
]
addopts = "--strict-markers"
```

`--strict-markers` fails collection on an unregistered marker — no typos, no silent drift.

**Usage:**
```python
@pytest.mark.scaffold
def test_parser_returns_dict() -> None: ...

@pytest.mark.decision(reason="deny before soft-delete because DENIED is terminal")
def test_deny_precedes_soft_delete() -> None: ...

@pytest.mark.contract(party="iOS BrandDetails v2")
def test_brand_details_wire_format() -> None: ...

@pytest.mark.incident(ref="TRYPLAT-186")
def test_scan_match_never_walks_keyspace() -> None: ...
```

**Promotion** — at prune, a scaffold that encodes a real decision is re-tagged: `@pytest.mark.scaffold` → `@pytest.mark.decision(reason="discount is non-negative by contract with billing")`.

**Census / delete set:**
```bash
pytest --collect-only -q | wc -l                  # total
pytest -m scaffold --collect-only -q              # the delete set
pytest -m decision --collect-only -q | wc -l      # likewise contract, incident
```

**CI profiles:**
```bash
pytest -m "not scaffold"   # gate: permanent tests only
pytest                     # full, including scaffolds, during development
```

### Rust

Rust has no built-in marker system. Use module naming and doc comments.

#### Scaffold Tests — `mod scaffold`

The classification moves construction-time tests into a `mod scaffold` inside `#[cfg(test)]`.

```rust
#[cfg(test)]
mod tests {
    // Permanent tests live here at the top level

    mod scaffold {
        // Construction-time verification — deleted at prune
        use super::*;

        #[test]
        fn parser_returns_expected_keys() { ... }

        #[test]
        fn validator_rejects_empty_string() { ... }
    }
}
```

**The delete set** — list every scaffold in the crate:
```bash
cargo test scaffold:: -- --list
```

#### Permanent Tests — Doc Comment Tags

A permanent test carries a `// kinhin:` doc comment with its tag and reference, written by the classification.

```rust
// kinhin: decision(ref="docs/payments/fees.md#rate-protection")
#[test]
fn grace_period_uses_higher_of_old_and_new_set() { ... }

// kinhin: contract(ref="API v2 wire format")
#[test]
fn serialize_roundtrip_preserves_all_fields() { ... }

// kinhin: incident(ref="ISSUE-186")
#[test]
fn invalidation_never_scans_keyspace() { ... }
```

Do NOT invent `cfg` features for lifecycle tags — `cfg(scaffold)` would require a feature flag and complicate the build.

#### CI Exclusion

The CI gate runs the full suite excluding scaffolds:
```bash
cargo nextest run --filter-expr 'not test(scaffold::)'
```

### Java

JUnit 5 `@Tag` carries lifecycle markers natively.

#### Tag Definitions

```java
// Scaffold — construction-time, deleted at prune
@Tag("scaffold")
@Test
void parserReturnsExpectedKeys() { ... }

// Decision — encodes a non-obvious choice, permanent
@Tag("decision")
@DisplayName("deny before soft-delete: DENIED is terminal")
@Test
void denyPrecedesSoftDelete() { ... }

// Contract — boundary another party depends on, permanent
@Tag("contract")
@DisplayName("Wire format: iOS BrandDetails v2")
@Test
void brandDetailsSerializationMatchesContract() { ... }

// Incident — reproduces a production failure, permanent
@Tag("incident")
@DisplayName("TICKET-186: SCAN walked 250k keys per write")
@Test
void invalidationNeverScansKeyspace() { ... }
```

`@DisplayName` carries the reason/reference for permanent tags. The tag itself is the lifecycle; the display name is the authority.

#### CI Exclusion

Surefire/Gradle exclude scaffolds from the production gate:

**Maven (pom.xml):**
```xml
<plugin>
    <groupId>org.apache.maven.plugins</groupId>
    <artifactId>maven-surefire-plugin</artifactId>
    <configuration>
        <excludedGroups>scaffold</excludedGroups>
    </configuration>
</plugin>
```

**Gradle (build.gradle.kts):**
```kotlin
tasks.test {
    useJUnitPlatform {
        excludeTags("scaffold")
    }
}
```

#### The Delete Set

List scaffolds before prune:
```bash
# Maven: run only scaffolds, dry-run
mvn test -Dgroups=scaffold -DdryRun=true

# Or grep the source:
grep -rn '@Tag("scaffold")' src/test/
```

### TypeScript

#### Jest: File Suffix Convention

Jest has no built-in tag filter. Use file suffixes to separate scaffold
from permanent tests:

```
tests/
├── order.service.test.ts              # permanent tests
├── order.service.scaffold.test.ts     # scaffold — deleted at prune
└── order.service.contract.test.ts     # contract tests (optional suffix)
```

**Prune = delete `*.scaffold.test.ts` files.**

CI exclusion:
```json
// jest.config.ts
{
  "testPathIgnorePatterns": ["\\.scaffold\\.test\\."]
}
```

#### Vitest: Project or Name Prefix

Vitest supports `--project` for workspace-based filtering or test-name
prefix convention:

```typescript
// vitest.config.ts — workspace approach
export default defineWorkspace([
  { test: { include: ['**/*.test.ts'], exclude: ['**/*.scaffold.test.ts'] } },
  { test: { name: 'scaffold', include: ['**/*.scaffold.test.ts'] } },
]);
```

CI exclusion: `vitest --project default` (excludes scaffold workspace).

## Collapse Example

**Before prune** — 12 scaffold tests for a `calculateDiscount` function:

```python
@pytest.mark.scaffold
def test_discount_premium_100() -> None:
    assert calculate_discount("premium", Decimal("100")) == Decimal("10")

@pytest.mark.scaffold
def test_discount_premium_200() -> None:
    assert calculate_discount("premium", Decimal("200")) == Decimal("20")

@pytest.mark.scaffold
def test_discount_premium_500() -> None:
    assert calculate_discount("premium", Decimal("500")) == Decimal("50")

@pytest.mark.scaffold
def test_discount_premium_1000() -> None:
    assert calculate_discount("premium", Decimal("1000")) == Decimal("100")

@pytest.mark.scaffold
def test_discount_standard_100() -> None:
    assert calculate_discount("standard", Decimal("100")) == Decimal("0")

@pytest.mark.scaffold
def test_discount_standard_500() -> None:
    assert calculate_discount("standard", Decimal("500")) == Decimal("0")

@pytest.mark.scaffold
def test_discount_premium_zero() -> None:
    assert calculate_discount("premium", Decimal("0")) == Decimal("0")

@pytest.mark.scaffold
def test_discount_premium_boundary() -> None:
    assert calculate_discount("premium", Decimal("1")) == Decimal("0.10")

@pytest.mark.scaffold
def test_discount_negative_raises() -> None:
    with pytest.raises(ValueError):
        calculate_discount("premium", Decimal("-1"))

@pytest.mark.scaffold
def test_discount_unknown_tier_raises() -> None:
    with pytest.raises(ValueError):
        calculate_discount("unknown", Decimal("100"))

@pytest.mark.scaffold
def test_discount_returns_decimal() -> None:
    result = calculate_discount("premium", Decimal("100"))
    assert isinstance(result, Decimal)

@pytest.mark.scaffold
def test_discount_premium_large() -> None:
    assert calculate_discount("premium", Decimal("99999")) == Decimal("9999.90")
```

**After prune** — 2 tests:

```python
@pytest.mark.decision(reason="discount is 10% for premium, 0% for standard, non-negative")
@pytest.mark.parametrize("tier,amount,expected", [
    ("premium", Decimal("100"), Decimal("10")),      # equivalence class: premium
    ("standard", Decimal("100"), Decimal("0")),       # equivalence class: standard
    ("premium", Decimal("0"), Decimal("0")),          # boundary: zero amount
    ("premium", Decimal("1"), Decimal("0.10")),       # boundary: minimum
],
    ids=["premium-normal", "standard-normal", "zero-amount", "min-amount"],
)
def test_discount_by_tier(tier: str, amount: Decimal, expected: Decimal) -> None:
    """One row per equivalence class + boundaries."""
    assert calculate_discount(tier, amount) == expected

@pytest.mark.decision(reason="invalid inputs must raise, not return wrong values")
@pytest.mark.parametrize("tier,amount", [
    ("premium", Decimal("-1")),    # error: negative
    ("unknown", Decimal("100")),   # error: invalid tier
],
    ids=["negative-amount", "unknown-tier"],
)
def test_discount_rejects_invalid(tier: str, amount: Decimal) -> None:
    """One row per error class."""
    with pytest.raises(ValueError):
        calculate_discount(tier, amount)
```

**What was removed and why:**
- 4 premium-amount variations (same equivalence class — one representative + boundary is enough)
- 1 standard-amount duplicate (same class as `standard_100`)
- 1 type check (`isinstance`) — redundant under type hints + mypy
- 1 large-amount test — same class as `premium_100`, no boundary

**12 → 2 tests. Suite green after the collapse.**

