# Load Kinhin TDD Context

## STOP - MANDATORY ACTION REQUIRED

**DO NOT RESPOND until you have resolved languages (Step 0) and used
the Read tool on ALL universal files PLUS every file for EACH
resolved language.**

## Step 0: Resolve Languages

An argument may have been passed (e.g. `/kinhin:load rust`):

| Argument | Resolution |
|----------|-----------|
| `python` | Universal + Python |
| `rust` | Universal + Rust |
| `java` | Universal + Java |
| `all` | Universal + all supported languages |
| (none) | Universal + DETECT — procedure below |

### DETECT Procedure

1. Search for markers in the current directory, its ancestors, AND
   shallow subdirectories (max depth 3). Use Glob or:
   ```bash
   find . -maxdepth 3 \( -name pyproject.toml -o -name Cargo.toml \
     -o -name pom.xml -o -name build.gradle -o -name build.gradle.kts \) \
     -not -path "*/node_modules/*" -not -path "*/.venv/*" \
     -not -path "*/target/*" -not -path "*/.git/*" 2>/dev/null
   ```
2. Map markers to languages:

   | Marker | Language |
   |--------|----------|
   | `pyproject.toml` | python |
   | `Cargo.toml` | rust |
   | `pom.xml` / `build.gradle` / `build.gradle.kts` | java |

3. Load the **UNION** of all detected languages (monorepos load
   multiple languages).
4. **Nothing detected → fallback: python.**

## Universal Files (6) — ALWAYS

### Step 1: Read TDD Methodology
```
Read: ~/.claude/kinhin/spec/tdd/tdd-spec.md
```

### Step 2: Read TDD Implementation Guides
```
Read: ~/.claude/kinhin/context/guides/tdd-implementation-guide.md
Read: ~/.claude/kinhin/context/guides/tdd-feature-development.md
Read: ~/.claude/kinhin/context/guides/tdd-bugfix-guide.md
Read: ~/.claude/kinhin/context/guides/tdd-refactoring-guide.md
```

### Step 3: Read Checklist
```
Read: ~/.claude/kinhin/context/checklists/tdd-checklist.md
```

## Python Files (7)

### Step 4a: Read Python TDD Spec
```
Read: ~/.claude/kinhin/spec/python-tdd/python-tdd-spec.md
```

### Step 5a: Read Python Test Templates
```
Read: ~/.claude/kinhin/context/examples/tdd-unit-tests.md
Read: ~/.claude/kinhin/context/examples/tdd-integration-tests.md
Read: ~/.claude/kinhin/context/examples/tdd-property-tests.md
Read: ~/.claude/kinhin/context/examples/tdd-error-handling-tests.md
Read: ~/.claude/kinhin/context/examples/tdd-performance-tests.md
Read: ~/.claude/kinhin/context/examples/tdd-anti-patterns.md
```

## Rust Files (2)

### Step 4b: Read Rust TDD Spec
```
Read: ~/.claude/kinhin/spec/rust-tdd/rust-tdd-spec.md
```

### Step 5b: Read Rust TDD Patterns
```
Read: ~/.claude/kinhin/context/examples/rust-tdd-patterns.md
```

## Java Files (2)

### Step 4c: Read Java TDD Spec
```
Read: ~/.claude/kinhin/spec/java-tdd/java-tdd-spec.md
```

### Step 5c: Read Java TDD Patterns
```
Read: ~/.claude/kinhin/context/examples/java-tdd-patterns.md
```

---

## HALT CONDITIONS

**If you responded without reading the 6 universal files plus all
files for every resolved language: YOU VIOLATED THIS PRINCIPLE.**

---

## After Loading

When Kinhin TDD context is loaded, ALL implementations must be
test-driven:

1. **Test First** - Write tests before implementation
2. **Red-Green-Refactor** - Follow the TDD cycle
3. **Batch Processing** - Generate tests in batches
4. **Property-Based** - Invariants via hypothesis (Python) / proptest (Rust)
5. **Anti-Hallucination** - Tests prevent LLM hallucinations

**Rust sessions additionally**: RED includes compile errors;
type-eliminated scenarios replace defensive tests; trait seams
before mocks.

**Java sessions additionally**: RED includes compile errors;
NullAway/enum/record scenarios are type-eliminated; value
constraints tested once at constructor boundaries; interface seams
before mocks; ArchUnit rules as tests.

## Confirmation

After reading all files, respond with the card below, filling the
Loaded box with the actual composition (e.g.
`universal (6) + python (7) = 13 files` or
`universal (6) + python (7) + rust (2) + java (2) = 17 files`):

```
╭─────────────────────────────────────────────────────╮
│                                                     │
│     ●        ●                                      │
│     ↓        ↓    │  Walking Meditation for TDD     │
│  K  I  N  H  I  N  │  経行 - Red, Green, Refactor   │
│     ┴        ┴    │  (v1.2.0)                       │
│                                                     │
╰─────────────────────────────────────────────────────╯

┌─ TDD Cycle ─────────────────────────────────────────┐
│ 🔴 RED    → Write failing test first                │
│ 🟢 GREEN  → Make it pass (minimal code)             │
│ 🔵 REFACTOR → Clean up, maintain tests              │
└─────────────────────────────────────────────────────┘

┌─ Loaded ────────────────────────────────────────────┐
│ universal (6) + {languages with counts}             │
│ = {total} files loaded                              │
└─────────────────────────────────────────────────────┘
```
