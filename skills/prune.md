---
description: Prune construction tests from the current branch — tag census, independent classification, collapse scaffolds, evidence-safe deletion
user-invocable: true
---

# Kinhin Prune

## Instructions

1. **Load context**: Read `~/.claude/kinhin/context/guides/tdd-prune-guide.md` and `~/.claude/kinhin/spec/tdd/tdd-prune-spec.md`.

2. **Tag census**: Find all test files changed on the current branch (`git diff --name-only main -- '*.test.*' '*.spec.*' '*test_*' '*_test.*'`). For each, classify tests by tag: @scaffold, @decision, @contract, @incident, untagged.

3. **Classify untagged and unproven**: For each untagged test, and each permanent-tagged test whose `ref` is missing or does not resolve (tags are the author's claim, a hint), read the test and determine: does it pin a contract, reproduce a real bug, hold an invariant of a pure function, encode a decision with a live ref, or is it construction scaffolding? Propose a category and tag.

   **Never** delete or rename a test cited as evidence in a delivered task's `completeness.md`.

4. **Collapse scaffolds**: For functions with multiple @scaffold tests, propose collapsing into one table-driven @decision test with representative inputs (one per equivalence class + boundaries).

5. **Propose deletions**: List remaining @scaffold tests that should be deleted. Show the delete set.

6. **Verify**: run the suite (`make test` / `make check`): green after the deletions.

7. **Report**: Output the census line: `written N / pruned M / survivors: X decision, Y contract, Z incident`.

8. **Do NOT auto-delete without the user seeing the proposal.** Present the plan, then execute on approval. (The runner's `test-prune` workflow is the exception: there a reviewer in a separate session judges every deletion — see the prune guide, Independent Prune.)
