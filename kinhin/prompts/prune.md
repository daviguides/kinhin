# Prune Pass

Load the prune guide and checklist, then execute the prune procedure on the current branch.

## Steps

1. Read: `~/.claude/kinhin/context/guides/tdd-prune-guide.md`
2. Read: `~/.claude/kinhin/context/checklists/tdd-checklist.md`
3. Run the tag census on changed test files
4. Classify untagged tests
5. Propose collapse and deletion plan
6. Execute on user approval
7. Run mutation parity check if tooling is available
8. Output census line for PR
