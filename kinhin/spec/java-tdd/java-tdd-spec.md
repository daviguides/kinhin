# Java TDD Specification

## Overview

Java-specific Test-Driven Development requirements. For universal TDD
methodology, see `@~/.claude/kinhin/spec/tdd/tdd-spec.md`. For Java
testing TOOLS (JUnit 6, AssertJ, Mockito, Testcontainers, ArchUnit,
JaCoCo), see `@~/.claude/shodo/spec/java/java-testing-tools-spec.md`
— this spec covers METHODOLOGY only and never re-specifies tooling.

TDD in modern Java (25 LTS, shodo standards) sits between Python and
Rust: the type system eliminates part of the defensive test suite
(sealed types, enums, NullAway-checked nullness), but value
constraints still live in runtime validation that demands tests.

## Principle 1: Types Eliminate Tests — Where They Can

### Do Not Test What the Compiler + NullAway Prove
**Requirement**: Before writing a test, ask "can a type or a checked
annotation make this scenario unrepresentable?" If yes, write the
type — the scenario is covered by construction.

| Python test | Java equivalent |
|-------------|-----------------|
| `test_rejects_none_user` | `@NullMarked` package — NullAway makes unchecked null a build error |
| `test_rejects_unknown_status` | `enum Status` + exhaustive switch — no unknown values |
| `test_wrong_argument_order` | Record newtypes: `UserId` vs `RepoId` cannot swap |
| `test_returns_consistent_type` | Signature IS the guarantee |

### What Remains Testable
Java has no unsigned types and no compile-time value constraints —
these still demand TDD:
- **Compact constructor validation** (`Cents` rejects negatives,
  `Email` rejects malformed input) — the boundary test suite
- Value constraints WITHIN a valid type (discount ≤ 25%)
- Behavior, state transitions, integration, I/O

**Pattern**: validate at the boundary, then trust the type:

```java
// ONE test suite for the compact constructor...
@ParameterizedTest
@ValueSource(strings = {"no-at-sign", "", "   "})
void rejectsInvalidEmail(String raw) {
    assertThatThrownBy(() -> new Email(raw))
        .isInstanceOf(IllegalArgumentException.class);
}

// ...then every method taking Email needs NO invalid-email tests.
void sendWelcome(Email email) { ... }
```

## Principle 2: RED Includes "Does Not Compile"

### The Cycle, Adapted
**Requirement**: Treat compile errors as a valid RED state. Write the
test against the API you WISH existed — the call site dictates the
signature.

```
🔴 RED      → Write the test first. Compile error = the test
              failing for the right reason.
🟢 GREEN    → Define records/signatures until it compiles
              (stub with throw new UnsupportedOperationException()),
              then implement until the test passes.
🔵 REFACTOR → Clean up. Types hold the structure; tests hold
              the behavior.
```

```java
// RED: does not compile — Scanner does not exist yet.
@Test
void reportsDirtyReposOnly() {
    var repos = List.of(repo("clean", false), repo("dirty", true));

    var report = new Scanner(repos).scan();

    assertThat(report.dirtyCount()).isEqualTo(1);
}
```

## Principle 3: Test Placement — Mirrored Tree, Package-Private Seam

### Tests Mirror the Source Package
**Requirement**: `src/test/java` mirrors the source package (the Java
convention — not Rust's inline modules). One test class per source
class: `ScannerService` → `ScannerServiceTest`.

**The package-private seam**: a test class in the SAME package sees
package-private members — that is the sanctioned access level for
testing internals. Never widen `private` to `public` for a test; if
a test needs `public` access to internals, that is a design signal.

### Integration Tests Exercise the Public Seam
**Requirement**: Tagged `@Tag("integration")` tests exercise the
module through its public API, with real dependencies via
Testcontainers. If an integration test reaches into internals, fix
the API, don't open it.

## Principle 4: Error-Path TDD Tests Exception Types, Not Messages

### Exceptions Are API — Test Them as API
**Requirement**: The scenario matrix's "Invalid Cases" column maps
1:1 to domain exception TYPES. Assert the type; assert message
content only when the message carries contract data (an id, a path):

```java
// ✅ CORRECT — type is the contract
assertThatThrownBy(() -> loadConfig(missing))
    .isInstanceOf(ConfigNotFoundException.class);

// ✅ Message fragment ONLY for contract data
assertThatThrownBy(() -> loadConfig(missing))
    .isInstanceOf(ConfigNotFoundException.class)
    .hasMessageContaining(missing.toString());

// ❌ WRONG — prose is presentation, not contract
.hasMessage("Could not find the configuration file, sorry!")
```

### Sealed Results Get Exhaustive Tests
When the outcome is a sealed interface instead of an exception, every
permitted variant appears in at least one test assertion — the Java
form of "all error paths tested". An unmatched variant is an untested
failure scenario.

### Cause Chain Is Contract
**Requirement**: When a domain exception wraps a lower-level failure,
one test asserts the chain (`hasCauseInstanceOf`) — a swallowed cause
is a defect TDD must catch at RED time.

## Principle 5: Interface Seams Come First

### Mocking Requires Design — TDD Forces It Early
Python monkeypatches anything after the fact. Mockito mocks
interfaces and classes, but shodo restricts mocking to **interfaces
at seams you own** — so the seam must exist BEFORE the test can be
written. RED phase surfaces the design decision:

```java
// 🔴 RED: this test forces the GitClient interface to exist
@Test
void skipsSyncWhenAlreadyOnMain() {
    GitClient git = mock();
    when(git.currentBranch()).thenReturn("main");

    var result = new SyncService(git).sync();

    assertThat(result).isInstanceOf(SyncResult.Skipped.class);
}
```

**Rule of restraint**: introduce an interface ONLY where substitution
is needed (I/O: git, network, clock, filesystem). An interface with
one implementation and no test double is the `IFoo`/`FooImpl` noise
shodo forbids — YAGNI applies.

### Prefer Real Values Over Mocks
Records make real inputs cheap. Pure logic takes constructed values;
mocks are reserved for genuine I/O boundaries. Inject `Clock` (the
JDK seam) instead of mocking time.

## Principle 6: Batch Generation Maps to @ParameterizedTest

**Requirement**: The universal scenario matrix materializes as
`@ParameterizedTest` cases — one row per matrix cell, generated as a
batch. `@CsvSource` for literal tables, `@MethodSource` for object
rows:

```java
@ParameterizedTest(name = "{0} with {1}y → {2}%")
@CsvSource({
    "PREMIUM,  5, 15",   // valid: upper tier
    "PREMIUM,  1,  5",   // valid: lower tier
    "PREMIUM,  2, 10",   // edge: tier boundary
    "PREMIUM,  0,  0",   // edge: zero years
    "STANDARD, 10, 0",   // valid: no program
})
void discountMatchesLoyaltyMatrix(Tier tier, int years, int expected) {
    var user = new User(tier, years);
    assertThat(calculateDiscount(user)).isEqualTo(new Percent(expected));
}
```

The Invalid row of the matrix documents type-eliminated cells
("enum — type-eliminated") and constructor-validated cells ("tested
at `Cents` boundary").

## Principle 7: Invariants Without jqwik

**Requirement**: Business constraints get invariant coverage (100%,
per the universal spec). jqwik is in maintenance mode with a
restrictive license (shodo java-testing-tools-spec) — cover
invariants with:

1. **Constructor invariant**: exhaustive `@ParameterizedTest` over
   boundary values (min, max, zero, just-inside, just-outside)
2. **Roundtrip**: `deserialize(serialize(x))` equality for Jackson
   records — records give `equals` for free

```java
@ParameterizedTest
@ValueSource(longs = {0, 1, Long.MAX_VALUE})
void acceptsNonNegativeAmounts(long value) {
    assertThat(new Cents(value).value()).isEqualTo(value);
}

@ParameterizedTest
@ValueSource(longs = {-1, Long.MIN_VALUE})
void rejectsNegativeAmounts(long value) {
    assertThatThrownBy(() -> new Cents(value))
        .isInstanceOf(IllegalArgumentException.class);
}
```

## Naming

**Requirement**: Behavior-first names, no `test` prefix — the
universal `test_{component}_{scenario}` pattern adapts: the test
CLASS carries the component (`ScannerServiceTest`), the method
carries the scenario in camelCase:

```java
class ScannerServiceTest {
    @Test
    void reportsDirtyReposOnly() { ... }

    @Test
    void throwsWhenRootMissing() { ... }
}
```

`@DisplayName`/`@ParameterizedTest(name = ...)` for prose only when
the method name cannot carry it.

## Coverage

Same universal targets (scenario 100%, branch ≥ 90%, constraints
100%), measured with JaCoCo (shodo java-testing-tools-spec).
Type-eliminated scenarios (Principle 1) count as covered BY
CONSTRUCTION — document the type in the matrix, not a redundant test.

## Anti-Hallucination (Java Form)

The universal anti-hallucination patterns get compiler + tooling
enforcement:
- **Type safety validation** → free: signatures + NullAway ARE
  checked contracts
- **Constraint validation** → compact constructors + boundary
  parameterized tests
- **Cross-function consistency** → shared records make drift a
  compile error; sealed switches without `default` make new variants
  loud
- **Architecture drift** → ArchUnit rules ARE tests — layer
  violations fail the suite, not the review
