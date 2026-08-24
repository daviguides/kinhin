# Kinhin

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> Walking meditation for TDD, designed as a Claude Code plugin.

## What is Kinhin?

**Kinhin** (経行 / 禅歩) is the practice of walking meditation in Zen Buddhism — the active transition between periods of *zazen* (seated meditation). Like walking meditation with deliberate steps, Kinhin guides developers through TDD with mindful progression: **red, green, refactor**.

Kinhin provides **universal TDD methodology** plus **language-specific deltas** for Python, Rust, and Java — workflow guides, test templates, anti-patterns, and checklists.

## Installation

```bash
bash -c "$(curl -fsSL https://raw.githubusercontent.com/daviguides/kinhin/main/install.sh)"
```

## Philosophy

### The Walk Between Sittings (経行)

TDD is not a single leap from nothing to correct — it's a walk taken in deliberate steps, each one testable before the next.

> *"Red. Green. Refactor. Repeat."*

### Rust TDD is not translated Python TDD

The Rust spec covers the *methodological deltas*, not a re-skin of the Python spec: the compiler eliminates a class of tests (type-eliminated scenarios), RED includes compile errors, unit tests live inline, error tests assert enum variants, and mocking requires trait seams decided in the RED phase. Tooling (nextest, rstest, proptest, mockall) is **not** re-specified here — it lives in Shodō's `rust-testing-tools-spec.md`.

### Java TDD follows the same delta approach

NullAway/enum/record scenarios are type-eliminated, RED includes compile errors, value constraints are tested once at record constructor boundaries, mocking requires interface seams decided in the RED phase, and ArchUnit turns layer rules into tests. Tooling (JUnit 6, AssertJ, Mockito, Testcontainers, ArchUnit, JaCoCo) lives in Shodō's `java-testing-tools-spec.md`.

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
              │
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
│   │   └── java-tdd/         # Java-specific TDD (methodological deltas)
│   ├── context/
│   │   ├── guides/           # TDD workflow guides (universal)
│   │   ├── examples/         # Test templates + rust/java-tdd-patterns.md
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
/kinhin:load all             # Load every supported language
```

Without an argument, Kinhin detects languages by project markers (`pyproject.toml` → Python, `Cargo.toml` → Rust, `pom.xml`/`build.gradle`/`build.gradle.kts` → Java) — in the current directory, ancestors, and shallow subdirectories — and loads the union (monorepos load multiple languages). Nothing detected → Python fallback. Universal files (methodology, guides, checklists) load always.

## License

MIT License

---

> *"Each step is complete in itself. Each test is complete in itself."*
