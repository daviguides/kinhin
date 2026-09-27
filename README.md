# Kinhin

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> Walking meditation for TDD, designed as a Claude Code plugin.

## What is Kinhin?

**Kinhin** (経行 / 禅歩) is the practice of walking meditation in Zen Buddhism — the active transition between periods of *zazen* (seated meditation). Like walking meditation with deliberate steps, Kinhin guides developers through TDD with mindful progression: **red, green, refactor, prune**.

Kinhin provides **universal TDD methodology** plus **language-specific deltas** for Python, Rust, Java, and TypeScript — workflow guides, test templates, anti-patterns, lifecycle management, runner contracts, and checklists.

## The 4-Phase Cycle

```
🔴 RED      → Write failing test first
🟢 GREEN    → Make it pass (minimal code)
🔵 REFACTOR → Clean up, maintain tests
⚪ PRUNE    → Delete scaffold, keep decisions
```

Every test is born with a **lifecycle tag**. The burden of proof is on *keeping* a test, not deleting it.

- `@scaffold` — construction verification, dies at prune (the default)
- `@decision(ref=)` — encodes a non-obvious decision, permanent
- `@contract(ref=)` — pins a cross-party boundary, permanent
- `@incident(ref=)` — reproduces a production failure, permanent

A Kinhin PR ends with a census line: `written 41 / pruned 33 / survivors: 5 decision, 2 contract, 1 incident`.

## Key Concepts

### Authority Model

Code and test are a closed system — neither validates the other. Every test either names an authority outside that system or it does not. The tag is a pointer to that authority.

### Drift Resistance Ladder

Before writing any test, ask "can I move up a rung?":

1. **Type constraint** — immune to drift (won't compile)
2. **Structural invariant / arch test** — almost immune (no value to adjust)
3. **`@decision` with ref to live source** — drifts but fails
4. **`@decision` with inline reason** — drifts, still fails
5. **`@scaffold`** — dies before it can drift

### Runner Contract

Six language-neutral requirements: no bail, parallel by default, randomized order, three-way flaky classification, changed-set selection, structured output.

### Failure Attribution

When a permanent test fails, the tag's authority is the third referential. Fix the code, not the test. When a scaffold fails, investigate — no authority exists.

## Installation

```bash
bash -c "$(curl -fsSL https://raw.githubusercontent.com/daviguides/kinhin/main/install.sh)"
```

## Philosophy

### The Walk Between Sittings (経行)

TDD is not a single leap from nothing to correct — it's a walk taken in deliberate steps, each one testable before the next.

> *"Red. Green. Refactor. Prune. Repeat."*

### Rust TDD is not translated Python TDD

The Rust spec covers the *methodological deltas*, not a re-skin of the Python spec: the compiler eliminates a class of tests (type-eliminated scenarios), RED includes compile errors, unit tests live inline, error tests assert enum variants, and mocking requires trait seams decided in the RED phase. Tooling (nextest, rstest, proptest, mockall) is **not** re-specified here — it lives in Shodō's `rust-testing-tools-spec.md`.

### Java TDD follows the same delta approach

NullAway/enum/record scenarios are type-eliminated, RED includes compile errors, value constraints are tested once at record constructor boundaries, mocking requires interface seams decided in the RED phase, and ArchUnit turns layer rules into tests. Tooling (JUnit 6, AssertJ, Mockito, Testcontainers, ArchUnit, JaCoCo) lives in Shodō's `java-testing-tools-spec.md`.

### TypeScript TDD: erasure-aware

Types are erased and unsound — the central difference from Rust and Java. RED includes type errors (`tsc --noEmit` is step 1). Do not test what strict TS proves. Module mocks are scaffold smell. Snapshot tests default to `@scaffold`.

## Relationship with Other Plugins

Kinhin was extracted from Zazen to separate concerns: Zazen keeps code quality principles, Kinhin owns TDD practices.

| Plugin | Philosophy | Focus |
|--------|------------|-------|
| **zazen** | Zen (座禅) | Universal principles (any language) |
| **shodo** | Calligraphy (書道) | Language standards (Python, Rust & Java) |
| **kinhin** | Walking meditation (経行) | TDD practices |
| **arche** | Greek (ἀρχή) | LLM behavioral principles |

```
        ┌────────┐
        │ zazen  │  ← Universal principles
        └───┬────┘
            │
    ┌───────┼───────┐
    ▼       ▼       ▼
┌───────┐ ┌───────┐ ┌────────┐
│ shodo │ │kinhin │ │ kyudo  │
└───────┘ └───────┘ └────────┘
 Python,    TDD      Actions
Rust, Java    ▲
 TypeScript   │
          YOU ARE HERE
```

## Project Structure

```
kinhin/
├── kinhin/                   # Bundle (Gradient pattern)
│   ├── spec/
│   │   ├── tdd/              # TDD methodology (universal)
│   │   ├── python-tdd/       # Python-specific TDD
│   │   ├── rust-tdd/         # Rust-specific TDD (methodological deltas)
│   │   ├── java-tdd/         # Java-specific TDD (methodological deltas)
│   │   └── typescript-tdd/   # TypeScript-specific TDD (methodological deltas)
│   ├── context/
│   │   ├── guides/           # TDD workflow guides + prune guide + runner guide
│   │   ├── examples/         # Test templates, anti-patterns, lifecycle tags, TS patterns
│   │   └── checklists/       # TDD checklists (universal)
│   └── prompts/
├── commands/
└── skills/
```

## Usage

```bash
/kinhin:load                 # Detect project languages and load TDD principles
/kinhin:load python          # Load Python TDD only
/kinhin:load rust            # Load Rust TDD only
/kinhin:load java            # Load Java TDD only
/kinhin:load typescript      # Load TypeScript TDD only
/kinhin:load all             # Load every supported language
/kinhin:prune                # Run the prune pass on the current branch
```

Without an argument, Kinhin detects languages by project markers (`pyproject.toml` → Python, `Cargo.toml` → Rust, `pom.xml`/`build.gradle`/`build.gradle.kts` → Java, `tsconfig.json` → TypeScript) — in the current directory, ancestors, and shallow subdirectories — and loads the union (monorepos load multiple languages). Nothing detected → Python fallback. Universal files (methodology, guides, checklists, prune guide, runner guide, lifecycle tags) load always.

## License

MIT License

---

> *"Each step is complete in itself. Each test is complete in itself."*
