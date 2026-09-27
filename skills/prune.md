---
description: Prune construction tests from the current branch — tag census, collapse scaffolds, verify mutation parity
user-invocable: true
---

# Kinhin Prune

## Instructions

1. **Load context**: Read `~/.claude/kinhin/context/guides/tdd-prune-guide.md` and `~/.claude/kinhin/context/checklists/tdd-checklist.md`.

2. **Tag census**: Find all test files changed on the current branch (`git diff --name-only main -- '*.test.*' '*.spec.*' '*test_*' '*_test.*'`). For each, classify tests by tag: @scaffold, @decision, @contract, @incident, untagged.

3. **Classify untagged**: For each untagged test, read the test and determine: does it encode a decision, pin a contract, reproduce an incident, or is it construction scaffolding? Propose a tag.

4. **Collapse scaffolds**: For functions with multiple @scaffold tests, propose collapsing into one table-driven @decision test with representative inputs (one per equivalence class + boundaries).

5. **Propose deletions**: List remaining @scaffold tests that should be deleted. Show the delete set.

6. **Report**: Output the census line: `written N / pruned M / survivors: X decision, Y contract, Z incident`.

7. **Do NOT auto-delete without the user seeing the proposal.** Present the plan, then execute on approval.
