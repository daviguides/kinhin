# Java TDD Patterns

Consolidated test templates for TDD in Java: the cycle walkthrough,
unit batches, error-path tests, invariant tests, and interface-seam
mocking. Methodology: `@~/.claude/kinhin/spec/java-tdd/java-tdd-spec.md`.

## Full Cycle Walkthrough: Validated Record

### 🔴 RED — write the test against the API you wish existed

```java
// src/test/java/.../EmailTest.java — written FIRST
class EmailTest {

    @Test
    void acceptsPlainValidAddress() {
        var email = new Email("dev@example.com");

        assertThat(email.value()).isEqualTo("dev@example.com");
    }

    @ParameterizedTest
    @ValueSource(strings = {"not-an-email", "", "   "})
    void rejectsMalformedAddress(String raw) {
        assertThatThrownBy(() -> new Email(raw))
            .isInstanceOf(IllegalArgumentException.class)
            .hasMessageContaining(raw.strip());
    }
}
```

Compile error: `Email` does not exist. **That IS the failing test.**

### 🟢 GREEN — satisfy the compiler, then the assertions

```java
public record Email(String value) {
    public Email {
        if (value.isBlank() || !value.contains("@")) {
            throw new IllegalArgumentException("invalid email: " + value);
        }
    }
}
```

### 🔵 REFACTOR — with both guarantees active

Extract, rename, restructure freely: the compiler holds the types,
the tests hold the behavior.

**Payoff**: every method accepting `Email` from now on needs ZERO
invalid-email tests — an existing instance is valid by construction.

## Unit Test Batch: Scenario Matrix → Parameterized Cases

Matrix defined before implementation (universal spec), materialized
as one row per cell:

```
calculateDiscount:
  Valid:   PREMIUM/5y → 15%, PREMIUM/1y → 5%, STANDARD → 0%
  Edge:    PREMIUM/0y → 0%, exactly at tier boundary (2y) → 10%
  Invalid: unknown tier — type-eliminated (enum, exhaustive switch)
           negative years — tested at User constructor boundary
```

```java
class DiscountCalculatorTest {

    @ParameterizedTest(name = "{0} with {1}y → {2}%")
    @CsvSource({
        "PREMIUM,  5, 15",
        "PREMIUM,  1,  5",
        "PREMIUM,  2, 10",
        "PREMIUM,  0,  0",
        "STANDARD, 10, 0",
    })
    void discountMatchesLoyaltyMatrix(Tier tier, int years, int expected) {
        var user = new User(tier, years);

        assertThat(DiscountCalculator.calculate(user))
            .isEqualTo(new Percent(expected));
    }
}
```

Note the matrix's Invalid row: partly EMPTY with the reason
documented. `Tier` is an enum with exhaustive switch — the compiler
covers those cells; negative years fail at the `User` boundary suite.

## Error-Path Tests: One Assertion per Type

Every domain exception (or sealed failure variant) appears in at
least one test:

```java
class SyncServiceTest {

    @Test
    void throwsWhenRepoPathMissing() {
        assertThatThrownBy(() -> service.sync(missingRepo()))
            .isInstanceOf(RepoNotFoundException.class);
    }

    @Test
    void refusesToSyncDirtyTree() {
        assertThatThrownBy(() -> service.sync(dirtyRepo()))
            .isInstanceOf(DirtyWorkingTreeException.class);
    }

    @Test
    void surfacesPushRejectionWithCause() {
        assertThatThrownBy(() -> service.sync(divergedRepo()))
            .isInstanceOf(PushRejectedException.class)
            .hasMessageContaining("non-fast-forward")
            .hasCauseInstanceOf(GitCommandException.class);
    }
}
```

Chain assertion (`hasCauseInstanceOf`) — a swallowed cause is a
defect the RED phase must catch.

### Sealed Result Variant Coverage

```java
public sealed interface SyncResult
    permits Synced, Skipped, Failed {}

// Every permitted variant asserted somewhere:
assertThat(service.sync(cleanRepo()))
    .isInstanceOf(SyncResult.Synced.class);

assertThat(service.sync(mainBranchRepo()))
    .isInstanceOf(SyncResult.Skipped.class);
```

## Invariant Tests: Boundary Batches + Roundtrip

No jqwik (license/maintenance — see shodo). Invariants via exhaustive
boundary batches and Jackson roundtrips:

```java
class CentsTest {

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
}

class ConfigTest {

    @Test
    void roundtripsThroughJson() throws Exception {
        var original = new Config(3, Duration.ofSeconds(30));

        var json = mapper.writeValueAsString(original);
        var parsed = mapper.readValue(json, Config.class);

        assertThat(parsed).isEqualTo(original);   // records: equals free
    }
}
```

## Interface Seam + Mock: RED Drives the Boundary

Test needs to substitute git — so the interface comes into existence
BEFORE the service is implemented:

```java
// 🔴 RED: this test forces the GitClient interface to exist
class SyncServiceTest {

    @Test
    void skipsSyncWhenAlreadyOnMain() {
        GitClient git = mock();
        when(git.currentBranch()).thenReturn("main");

        var result = new SyncService(git).sync();

        assertThat(result).isInstanceOf(SyncResult.Skipped.class);
        verify(git, never()).pullRebase();
    }
}
```

```java
// 🟢 GREEN: the seam, then the service against it
public interface GitClient {
    String currentBranch();
    void pullRebase();
}

public class SyncService {

    private final GitClient git;

    public SyncService(GitClient git) {
        this.git = Objects.requireNonNull(git);
    }

    public SyncResult sync() {
        if (git.currentBranch().equals("main")) {
            return new SyncResult.Skipped("already on main");
        }
        git.pullRebase();
        return new SyncResult.Synced(1);
    }
}
```

Restraint: `GitClient` earned its interface because tests substitute
it. Pure logic (`DiscountCalculator`) takes real records — no
interface, no mock. Time comes from an injected `java.time.Clock` —
the JDK's built-in seam:

```java
var fixed = Clock.fixed(Instant.parse("2026-01-01T00:00:00Z"), UTC);
var service = new ReportService(fixed);
```

## Integration Test: Public Seam + Testcontainers

```java
@Tag("integration")
class UserRepositoryIT {

    @Container
    static final PostgreSQLContainer<?> postgres =
        new PostgreSQLContainer<>("postgres:17");

    @Test
    void persistsAndFindsUserByEmail() {
        var repository = new UserRepository(dataSource(postgres));
        var user = new User(new Email("dev@example.com"), "Dev");

        repository.save(user);

        assertThat(repository.findByEmail(new Email("dev@example.com")))
            .contains(user);
    }
}
```

Real database, public API only. If the test cannot express itself
through the public seam, the API is missing something — fix the API,
don't open internals.

## Architecture as Tests: ArchUnit in the TDD Loop

Layer rules are written as tests BEFORE the layers fill up — drift
fails the suite, not the review:

```java
@AnalyzeClasses(packages = "com.example.tool")
class ArchitectureTest {

    @ArchTest
    static final ArchRule servicesFreeOfCli =
        noClasses().that().resideInAPackage("..services..")
            .should().dependOnClassesThat().resideInAPackage("picocli..");

    @ArchTest
    static final ArchRule modelDependsOnNothing =
        classes().that().resideInAPackage("..model..")
            .should().onlyDependOnClassesThat()
            .resideInAnyPackage("java..", "org.jspecify..", "..model..");
}
```

## Tests You Do NOT Write in Java

Anti-pattern: porting Python's defensive test suite verbatim.

| Ported test (❌ redundant) | Why it's free in Java (shodo standards) |
|---------------------------|----------------------------------------|
| `rejectsNullUser` | `@NullMarked` + NullAway — unchecked null fails the build |
| `handlesUnknownEnumValue` | Exhaustive switch, no `default` — no unknown values |
| `rejectsWrongType` | Compile error |
| `wrongArgumentOrder` | Record newtypes (`UserId` vs `RepoId`) cannot swap |
| `returnsConsistentType` | Signature IS the guarantee |

Write the TYPE, document the eliminated scenario in the matrix
("type-eliminated"), and spend the test budget on behavior. What is
NOT free in Java: value constraints (no unsigned types) — those get
the constructor boundary suite, once, at the type's edge.
