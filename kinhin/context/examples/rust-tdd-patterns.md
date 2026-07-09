# Rust TDD Patterns

Consolidated test templates for TDD in Rust: the cycle walkthrough,
unit batches, error-path tests, property tests, and trait-seam
mocking. Methodology: `@~/.claude/kinhin/spec/rust-tdd/rust-tdd-spec.md`.

## Full Cycle Walkthrough: Newtype with Validation

### 🔴 RED — write the test against the API you wish existed

```rust
// email.rs — file starts with ONLY this module
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_plain_valid_address() {
        let email = Email::new("dev@example.com").unwrap();
        assert_eq!(email.as_str(), "dev@example.com");
    }

    #[test]
    fn rejects_address_without_at_sign() {
        assert!(matches!(
            Email::new("not-an-email"),
            Err(ValidationError::InvalidEmail { .. })
        ));
    }
}
```

Compile error: `Email` does not exist. **That IS the failing test.**

### 🟢 GREEN — satisfy the compiler, then the assertions

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Email(String);

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("invalid email: {value}")]
    InvalidEmail { value: String },
}

impl Email {
    pub fn new(raw: &str) -> Result<Self, ValidationError> {
        if !raw.contains('@') {
            return Err(ValidationError::InvalidEmail { value: raw.into() });
        }
        Ok(Self(raw.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

### 🔵 REFACTOR — with both guarantees active

Extract, rename, restructure freely: the compiler holds the types,
the tests hold the behavior.

**Payoff**: every function accepting `Email` from now on needs ZERO
invalid-email tests — the scenario is unrepresentable.

## Unit Test Batch: Scenario Matrix → rstest Cases

Matrix defined before implementation (universal spec), materialized
as one case per cell:

```
calculate_discount:
  Valid:   premium/5y → 15%, premium/1y → 5%, standard → 0%
  Edge:    premium/0y → 0%, exactly at tier boundary (2y) → 10%
  Invalid: (none — tiers are an enum, years are u32: type-eliminated)
```

```rust
#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::premium_top_tier(Tier::Premium, 5, 15)]
    #[case::premium_low_tier(Tier::Premium, 1, 5)]
    #[case::premium_boundary(Tier::Premium, 2, 10)]
    #[case::premium_zero_years(Tier::Premium, 0, 0)]
    #[case::standard_ignores_years(Tier::Standard, 10, 0)]
    fn discount_matches_loyalty_matrix(
        #[case] tier: Tier,
        #[case] years: u32,
        #[case] expected_pct: u8,
    ) {
        let user = User { tier, loyalty_years: years };
        assert_eq!(calculate_discount(&user), Percent(expected_pct));
    }
}
```

Note the matrix's Invalid row: EMPTY, with the reason documented.
`Tier` is exhaustive and `u32` cannot be negative — the compiler
covers those cells.

## Error-Path Tests: One Assertion per Variant

Every variant of an owned error enum appears in at least one test:

```rust
#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("repo not found at {path}")]
    RepoNotFound { path: PathBuf },

    #[error("working tree is dirty")]
    DirtyWorkingTree,

    #[error("push rejected: {reason}")]
    PushRejected { reason: String },
}
```

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_when_repo_path_missing() {
        let result = sync(&missing_repo());
        assert!(matches!(result, Err(SyncError::RepoNotFound { .. })));
    }

    #[test]
    fn refuses_to_sync_dirty_tree() {
        let result = sync(&dirty_repo());
        assert!(matches!(result, Err(SyncError::DirtyWorkingTree)));
    }

    #[test]
    fn surfaces_push_rejection_reason() {
        let result = sync(&diverged_repo());
        let Err(SyncError::PushRejected { reason }) = result else {
            panic!("expected PushRejected, got {result:?}");
        };
        assert!(reason.contains("non-fast-forward"));
    }
}
```

Inspect error FIELDS with `let-else` + `panic!` (as above) when the
variant's data matters; plain `matches!` when only the variant does.

## Property Tests: Constructor Invariant + Roundtrip

```rust
#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    proptest! {
        // Constructor invariant: whatever Email::new accepts
        // satisfies the domain rule — for ALL inputs.
        #[test]
        fn accepted_emails_always_contain_at(raw in "\\PC*") {
            if let Ok(email) = Email::new(&raw) {
                prop_assert!(email.as_str().contains('@'));
            }
        }

        // Roundtrip: serde types survive serialize → deserialize.
        #[test]
        fn config_roundtrips_through_yaml(
            retries in 0u32..100,
            secs in 1u64..3600,
        ) {
            let original = Config::new(retries, secs);
            let yaml = serde_yaml::to_string(&original).unwrap();
            let parsed: Config = serde_yaml::from_str(&yaml).unwrap();
            prop_assert_eq!(original, parsed);
        }
    }
}
```

## Trait Seam + Mock: RED Drives the Boundary

Test needs to substitute git — so the trait comes into existence
BEFORE the service is implemented:

```rust
// 🔴 RED: this test forces the GitClient trait to exist
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_sync_when_already_on_main() {
        let mut git = MockGitClient::new();
        git.expect_current_branch()
            .return_once(|| Ok("main".to_string()));

        let result = sync(&git).unwrap();

        assert_eq!(result, SyncResult::Skipped);
    }
}
```

```rust
// 🟢 GREEN: the seam, then the service against it
#[cfg_attr(test, mockall::automock)]
pub trait GitClient {
    fn current_branch(&self) -> Result<String, GitError>;
    fn pull_rebase(&self) -> Result<(), GitError>;
}

pub fn sync(git: &impl GitClient) -> Result<SyncResult, SyncError> {
    if git.current_branch()? == "main" {
        return Ok(SyncResult::Skipped);
    }
    git.pull_rebase()?;
    Ok(SyncResult::Synced)
}
```

Restraint: `GitClient` earned its trait because tests substitute it.
Pure logic (`calculate_discount`) takes real values — no trait, no
mock.

## Integration Test: Public API Only

```rust
// tests/scan.rs — sees the crate as a consumer
use mycrate::{scan, ScanOptions};

#[test]
fn scan_finds_nested_dirty_repo() {
    let workspace = tempfile::tempdir().unwrap();
    init_repo_with_uncommitted_file(workspace.path().join("nested/repo"));

    let report = scan(workspace.path(), &ScanOptions::default()).unwrap();

    assert_eq!(report.dirty.len(), 1);
    assert!(report.dirty[0].path.ends_with("nested/repo"));
}
```

If this file cannot express the test through the public API, the API
is missing something — fix the API, don't `pub` internals.

## Tests You Do NOT Write in Rust

Anti-pattern: porting Python's defensive test suite verbatim.

| Ported test (❌ redundant) | Why it's free in Rust |
|---------------------------|----------------------|
| `rejects_none_input` | No `Option` in the signature — cannot be None |
| `rejects_negative_amount` | `Cents(u64)` — negativity unrepresentable |
| `rejects_wrong_type` | Compile error |
| `handles_unknown_enum_value` | Exhaustive enum — no unknown values |
| `returns_consistent_type` | Signature IS the guarantee |

Write the TYPE, document the eliminated scenario in the matrix
("type-eliminated"), and spend the test budget on behavior.
