# Rust TDD Specification

## Overview

Rust-specific Test-Driven Development requirements. For universal TDD
methodology, see `@~/.claude/kinhin/spec/tdd/tdd-spec.md`. For Rust
testing TOOLS (nextest, rstest, proptest, mockall, coverage), see
`@~/.claude/shodo/spec/rust/rust-testing-tools-spec.md` — this spec
covers METHODOLOGY only and never re-specifies tooling.

TDD in Rust is not Python TDD with different syntax. The type system,
the compiler, and the ownership model change WHEN tests are written,
WHICH tests exist, and WHAT the RED phase means.

## Principle 1: The Compiler Writes Tests Too

### Do Not Test What the Type System Proves
**Requirement**: Before writing a test, ask "can a type make this
scenario unrepresentable?" If yes, write the type — the test is free
and runs at compile time, forever.

| Python test | Rust equivalent |
|-------------|-----------------|
| `test_rejects_negative_amount` | `struct Cents(u64)` — negativity is unrepresentable |
| `test_rejects_none_user` | Parameter is `&User`, not `Option<&User>` |
| `test_rejects_unknown_status` | Exhaustive `enum Status` — no unknown values exist |
| `test_wrong_argument_order` | Newtypes: `UserId` vs `RepoId` cannot swap |

### What Remains Testable
The type system does NOT prove business rules. These still demand
TDD:
- Value constraints WITHIN a valid type (discount ≤ 25%)
- Newtype constructor validation (`Email::new` rejects malformed input)
- Behavior, state transitions, integration, I/O

**Pattern**: validate at the boundary, then trust the type:
```rust
// ONE test suite for the constructor...
#[rstest]
#[case("no-at-sign")]
#[case("")]
fn email_new_rejects_invalid(#[case] raw: &str) {
    assert!(matches!(Email::new(raw), Err(ValidationError::InvalidEmail { .. })));
}

// ...then every function taking `Email` needs NO invalid-email tests.
fn send_welcome(email: &Email) { ... }
```

## Principle 2: RED Includes "Does Not Compile"

### The Cycle, Adapted
**Requirement**: Treat compile errors as a valid RED state, not as a
blocker before the cycle starts.

```
🔴 RED      → Write the test against the API you WISH existed.
              Compile error = the test failing for the right reason.
🟢 GREEN    → Define types/signatures until it compiles,
              then implement until the test passes.
🔵 REFACTOR → Clean up. The compiler guarantees the refactor
              preserves types; the tests guarantee behavior.
```

### Type-Driven RED
**Requirement**: Let the failing test DICTATE the signature — write
the call site first, derive the function from it.

```rust
// RED: this does not compile — Scanner::scan does not exist yet.
#[test]
fn scan_reports_dirty_repos_only() {
    let repos = vec![repo("clean", false), repo("dirty", true)];
    let report = Scanner::new(repos).scan();
    assert_eq!(report.dirty_count(), 1);
}
```

The compiler now enumerates exactly what GREEN requires: `Scanner::new`,
`scan`, `dirty_count`. Stub with `todo!()` to reach "compiles but
fails", then implement.

## Principle 3: Test Placement Shapes the Loop

### Unit Tests Live in the Source File
**Requirement**: `#[cfg(test)] mod tests` at the bottom of the module
under test — NOT a mirrored `tests/` tree (that is the Python
convention; it does not translate).

**Consequences for TDD:**
- Test and implementation share one file — the red-green loop never
  leaves the buffer
- Tests access private items — test the real internals, no
  `pub`-for-testability leaks
- A NEW module starts as a test module: write `mod tests` first,
  grow the implementation above it

### Integration Tests Exercise the Public API
**Requirement**: `tests/` files see the crate as a consumer would.
If an integration test needs a private item, that is a design signal
— either the item belongs in the public API or the test belongs
inline.

## Principle 4: Error-Path TDD Tests Variants, Not Messages

### Errors Are API — Test Them as API
**Requirement**: The scenario matrix's "Invalid Cases" column maps
1:1 to error enum VARIANTS. Assert the variant, never the message:

```rust
// ✅ CORRECT — variant is the contract
assert!(matches!(
    load_config(&missing),
    Err(ConfigError::NotFound { .. })
));

// ❌ WRONG — Display strings are presentation, not contract
assert!(load_config(&missing).unwrap_err().to_string().contains("not found"));
```

Exception: user-facing output (CLI error text) MAY be
snapshot-tested — as a display concern, separately from the
error-handling logic.

### Exhaustiveness as Coverage
**Requirement**: Every variant of an error enum you own must appear
in at least one test assertion. An unmatched variant is an untested
failure scenario — the Rust form of the universal "all error paths
tested" rule.

## Principle 5: Trait Boundaries Come First

### Mocking Requires Design — TDD Forces It Early
Python monkeypatches anything after the fact. Rust mocks TRAITS
(mockall), so the substitution seam must exist BEFORE the test can
be written. This is a feature: RED phase surfaces the design
decision.

**Requirement**: When a unit test needs to substitute a collaborator
(network, clock, subprocess, filesystem), define the trait FIRST —
the test drives the boundary into existence:

```rust
#[cfg_attr(test, mockall::automock)]
trait GitClient {
    fn current_branch(&self) -> Result<String, GitError>;
}

// Service depends on the trait, not the implementation
fn sync(git: &impl GitClient) -> Result<SyncResult, SyncError> { ... }
```

**Rule of restraint**: introduce a trait ONLY where substitution is
needed. A trait with one implementation and no test double is
speculative abstraction — YAGNI applies.

### Prefer Real Values Over Mocks
Ownership makes pure functions cheap to test with real inputs.
Reserve mocks for genuine I/O boundaries; everything else takes
constructed values.

## Principle 6: Batch Generation Maps to rstest Cases

**Requirement**: The universal scenario matrix materializes as
`#[rstest]` `#[case]` blocks — one attribute line per matrix cell,
generated as a batch:

```rust
#[rstest]
#[case::loyal_5y("premium", 5, 15)]     // valid: upper tier
#[case::loyal_1y("premium", 1, 5)]      // valid: lower tier
#[case::boundary_0y("premium", 0, 0)]   // edge: boundary
#[case::standard("standard", 10, 0)]    // valid: no program
fn discount_by_loyalty(
    #[case] tier: &str,
    #[case] years: u32,
    #[case] expected_pct: u8,
) { ... }
```

Case names (`::name`) document the matrix cell — the test list IS
the scenario matrix.

## Principle 7: Property Tests Guard Invariants

**Requirement**: Business constraints get proptest properties, same
coverage rule as the universal spec (100% of constraints). The two
highest-value Rust patterns:

1. **Constructor invariant**: anything a newtype's `new` accepts
   satisfies the domain rule — for ALL inputs
2. **Roundtrip**: `deserialize(serialize(x)) == x` for serde types

## Naming

**Requirement**: Behavior-first names. The universal
`test_{component}_{scenario}` pattern adapts — `#[test]` marks tests
(no `test_` prefix needed) and the inline `mod tests` already scopes
the component:

```rust
// In scanner.rs → mod tests: component implied, scenario explicit
#[test]
fn reports_dirty_repos_only() { ... }

// In tests/cli.rs (integration): component explicit
#[test]
fn scan_command_exits_nonzero_on_missing_root() { ... }
```

## Coverage

Same universal targets (scenario 100%, branch ≥ 90%, constraints
100%), measured with `cargo llvm-cov nextest`. Type-eliminated
scenarios (Principle 1) count as covered BY CONSTRUCTION — document
the type, not a redundant test.

## Anti-Hallucination (Rust Form)

The universal anti-hallucination patterns get compiler enforcement:
- **Type safety validation** → free: signatures ARE checked contracts
- **Constraint validation** → newtype constructors + property tests
- **Cross-function consistency** → shared types make drift a compile
  error; test only the behavioral consistency that types cannot see
- **Explicit validation** → `#[must_use]`, no `let _ =` on Results
  in production paths (tests assert on every Result)
