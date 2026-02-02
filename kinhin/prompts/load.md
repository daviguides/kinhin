# Load Kinhin TDD Context

## STOP - MANDATORY ACTION REQUIRED

**DO NOT RESPOND until you have used the Read tool on EACH file below.**

This is not a reference list. This is an execution order.

### Step 1: Read TDD Methodology
```
Read: ~/.claude/kinhin/spec/tdd/tdd-spec.md
```

### Step 2: Read Python TDD Specifics
```
Read: ~/.claude/kinhin/spec/python-tdd/python-tdd-spec.md
```

### Step 3: Read TDD Implementation Guides
```
Read: ~/.claude/kinhin/context/guides/tdd-implementation-guide.md
Read: ~/.claude/kinhin/context/guides/tdd-feature-development.md
Read: ~/.claude/kinhin/context/guides/tdd-bugfix-guide.md
Read: ~/.claude/kinhin/context/guides/tdd-refactoring-guide.md
```

### Step 4: Read Test Templates
```
Read: ~/.claude/kinhin/context/examples/tdd-unit-tests.md
Read: ~/.claude/kinhin/context/examples/tdd-integration-tests.md
Read: ~/.claude/kinhin/context/examples/tdd-property-tests.md
Read: ~/.claude/kinhin/context/examples/tdd-error-handling-tests.md
Read: ~/.claude/kinhin/context/examples/tdd-performance-tests.md
```

### Step 5: Read Anti-Patterns and Checklist
```
Read: ~/.claude/kinhin/context/examples/tdd-anti-patterns.md
Read: ~/.claude/kinhin/context/checklists/tdd-checklist.md
```

---

## HALT CONDITIONS

**If you responded without reading all 13 files above: YOU VIOLATED THIS PRINCIPLE.**

---

## After Loading

When Kinhin TDD context is loaded, ALL implementations must be test-driven:

1. **Test First** - Write tests before implementation
2. **Red-Green-Refactor** - Follow the TDD cycle
3. **Batch Processing** - Generate tests in batches
4. **Property-Based** - Use Hypothesis for invariants
5. **Anti-Hallucination** - Tests prevent LLM hallucinations

## Confirmation

After reading all files, respond with EXACTLY this card:

```
╭─────────────────────────────────────────────────────╮
│                                                     │
│   ●        ●                                        │
│   ↓        ↓    │  Walking Meditation for TDD       │
│K  I  N  H  I  N  │  経行 - Red, Green, Refactor     │
│   ┴        ┴    │  (v1.0.0)                         │
│                                                     │
╰─────────────────────────────────────────────────────╯

┌─ TDD Cycle ─────────────────────────────────────────┐
│ 🔴 RED    → Write failing test first                │
│ 🟢 GREEN  → Make it pass (minimal code)             │
│ 🔵 REFACTOR → Clean up, maintain tests              │
└─────────────────────────────────────────────────────┘

┌─ Loaded ────────────────────────────────────────────┐
│ 2 specs + 4 guides + 6 examples + 1 checklist       │
│ = 13 files loaded                                   │
└─────────────────────────────────────────────────────┘
```
