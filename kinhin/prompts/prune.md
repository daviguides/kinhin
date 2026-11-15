# Prune Pass

Load the prune guide and spec, then execute the prune procedure on the current branch.

## Steps

1. Read: `~/.claude/kinhin/context/guides/tdd-prune-guide.md`
2. Read: `~/.claude/kinhin/spec/tdd/tdd-prune-spec.md`
3. Read: `~/.claude/kinhin/context/examples/tdd-lifecycle-tags.md` (marker syntax per language)
4. Run the tag census on changed test files
5. Classify untagged tests and permanent tags whose ref does not resolve (tags are hints)
6. Propose collapse and deletion plan
7. Execute on user approval; never delete or rename a test cited as completeness evidence
8. Run the suite: green after the deletions
9. Output census line for PR
