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
| `typescript` | Universal + TypeScript |
| `all` | Universal + all supported languages |
| (none) | Universal + DETECT — procedure below |

### DETECT Procedure

1. Search for markers in the current directory, its ancestors, AND
   shallow subdirectories (max depth 3). Use Glob or:
   ```bash
   find . -maxdepth 3 \( -name pyproject.toml -o -name Cargo.toml \
     -o -name pom.xml -o -name build.gradle -o -name build.gradle.kts \
     -o -name tsconfig.json \) \
     -not -path "*/node_modules/*" -not -path "*/.venv/*" \
     -not -path "*/target/*" -not -path "*/.git/*" 2>/dev/null
   ```
2. Map markers to languages:

   | Marker | Language |
   |--------|----------|
   | `pyproject.toml` | python |
   | `Cargo.toml` | rust |
   | `pom.xml` / `build.gradle` / `build.gradle.kts` | java |
   | `tsconfig.json` | typescript |

3. Load the **UNION** of all detected languages (monorepos load
   multiple languages).
4. **Nothing detected → fallback: python.**

## Universal Files (11) — ALWAYS

### Step 1: Read TDD Methodology
```
Read: ~/.claude/kinhin/spec/tdd/tdd-spec.md
Read: ~/.claude/kinhin/spec/tdd/tdd-prune-spec.md
Read: ~/.claude/kinhin/spec/tdd/tdd-runner-spec.md
```

### Step 2: Read TDD Implementation Guides
```
Read: ~/.claude/kinhin/context/guides/tdd-implementation-guide.md
Read: ~/.claude/kinhin/context/guides/tdd-feature-development.md
Read: ~/.claude/kinhin/context/guides/tdd-bugfix-guide.md
Read: ~/.claude/kinhin/context/guides/tdd-refactoring-guide.md
```

### Step 3: Read Prune and Runner Guides
```
Read: ~/.claude/kinhin/context/guides/tdd-prune-guide.md
Read: ~/.claude/kinhin/context/guides/tdd-runner-guide.md
```

### Step 4: Read Checklist and Lifecycle Tags
```
Read: ~/.claude/kinhin/context/checklists/tdd-checklist.md
Read: ~/.claude/kinhin/context/examples/tdd-lifecycle-tags.md
```

## Python Files (8)

### Step 5a: Read Python TDD Spec
```
Read: ~/.claude/kinhin/spec/python-tdd/python-tdd-spec.md
Read: ~/.claude/kinhin/spec/python-tdd/python-tdd-runner-spec.md
```

### Step 6a: Read Python Test Templates
```
Read: ~/.claude/kinhin/context/examples/tdd-unit-tests.md
Read: ~/.claude/kinhin/context/examples/tdd-integration-tests.md
Read: ~/.claude/kinhin/context/examples/tdd-property-tests.md
Read: ~/.claude/kinhin/context/examples/tdd-error-handling-tests.md
Read: ~/.claude/kinhin/context/examples/tdd-performance-tests.md
Read: ~/.claude/kinhin/context/examples/tdd-anti-patterns.md
```

## Rust Files (2)

### Step 5b: Read Rust TDD Spec
```
Read: ~/.claude/kinhin/spec/rust-tdd/rust-tdd-spec.md
```

### Step 6b: Read Rust TDD Patterns
```
Read: ~/.claude/kinhin/context/examples/rust-tdd-patterns.md
```

## Java Files (2)

### Step 5c: Read Java TDD Spec
```
Read: ~/.claude/kinhin/spec/java-tdd/java-tdd-spec.md
```

### Step 6c: Read Java TDD Patterns
```
Read: ~/.claude/kinhin/context/examples/java-tdd-patterns.md
```

## TypeScript Files (2)

### Step 5d: Read TypeScript TDD Spec
```
Read: ~/.claude/kinhin/spec/typescript-tdd/typescript-tdd-spec.md
```

### Step 6d: Read TypeScript TDD Patterns
```
Read: ~/.claude/kinhin/context/examples/typescript-tdd-patterns.md
```

---

## HALT CONDITIONS

**If you responded without reading the 11 universal files plus all
files for every resolved language: YOU VIOLATED THIS PRINCIPLE.**

---

## After Loading

When Kinhin TDD context is loaded, ALL implementations must be
test-driven:

1. **Test First** - Write tests before implementation
2. **Red-Green-Refactor** - The author's cycle; prune is a separate pass
3. **Batch Processing** - Generate tests in batches
4. **Property-Based** - Invariants via hypothesis (Python) / proptest (Rust) / fast-check (TypeScript)
5. **Anti-Hallucination** - Tests prevent LLM hallucinations
6. **Prune separately** - The session that wrote the tests never tags, collapses or deletes them: classify and prune in a separate pass (`kinhin tag` + `/kinhin:prune`)
7. **Suite strength** - Proven by the scenario matrix, RED per block and review (B5); no mutation testing
8. **Runner contract** - No bail, parallel, randomized, structured output
9. **CLI** (optional) - If `kinhin` is on PATH, use it to automate:
   `kinhin audit` (tag report), `kinhin census` (tag counts),
   `kinhin run` (runner contract), `kinhin setup` (install deps),
   `kinhin tag` (auto-classify), `kinhin prune` (full pipeline)

**Rust sessions additionally**: RED includes compile errors;
type-eliminated scenarios replace defensive tests; trait seams
before mocks.

**Java sessions additionally**: RED includes compile errors;
NullAway/enum/record scenarios are type-eliminated; value
constraints tested once at constructor boundaries; interface seams
before mocks; ArchUnit rules as tests.

**TypeScript sessions additionally**: RED includes type errors
(`tsc --noEmit` is step 1); types are erased and unsound — do not
test what strict TS proves; module mocks and snapshot tests carry
no authority.

## Confirmation

After reading all files, respond with the card below, filling the
Loaded box with the actual composition (e.g.
`universal (11) + python (8) = 19 files` or
`universal (11) + python (8) + rust (2) + java (2) + typescript (2) = 25 files`):

```
╭─────────────────────────────────────────────────────╮
│                                                     │
│     ●        ●                                      │
│     ↓        ↓    │  Walking Meditation for TDD     │
│  K  I  N  H  I  N  │  経行 - Red, Green, Refactor  │
│     ┴        ┴    │  (v3.1.0)                       │
│                                                     │
╰─────────────────────────────────────────────────────╯

┌─ TDD Cycle ─────────────────────────────────────────┐
│ 🔴 RED      → Write failing test first              │
│ 🟢 GREEN    → Make it pass (minimal code)           │
│ 🔵 REFACTOR → Clean up, maintain tests              │
└─────────────────────────────────────────────────────┘

┌─ Separate pass ─────────────────────────────────────┐
│ ⚪ PRUNE    → /kinhin:prune, never by the author     │
└─────────────────────────────────────────────────────┘

┌─ Loaded ────────────────────────────────────────────┐
│ universal (11) + {languages with counts}            │
│ = {total} files loaded                              │
└─────────────────────────────────────────────────────┘
```
