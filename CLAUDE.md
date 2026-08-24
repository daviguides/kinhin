# Kinhin - Claude Code Project Instructions

## Project Overview

**Kinhin** (経行 / 禅歩) is the practice of walking meditation in Zen Buddhism, serving as active transition between periods of zazen (seated meditation).

**Philosophy**: Like walking meditation with deliberate steps, Kinhin guides developers through TDD with mindful progression - red, green, refactor.


## Structure

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
│   │   ├── examples/         # Test templates (Python) + rust/java-tdd-patterns.md
│   │   └── checklists/       # TDD checklists (universal)
│   └── prompts/
├── commands/
└── skills/
```


## Commands

| Command | Purpose |
|---------|---------|
| `/kinhin:load [python\|rust\|java\|all]` | Load TDD principles — universal core + language specifics |

**Language resolution**: explicit argument wins; without argument,
detect markers (`pyproject.toml` → python, `Cargo.toml` → rust,
`pom.xml`/`build.gradle`/`build.gradle.kts` → java) in cwd,
ancestors, and shallow subdirectories, loading the UNION
(monorepos load multiple). Nothing detected → python fallback.
Universal files (methodology, guides, checklist) load ALWAYS.

**Rust TDD** is not translated Python TDD — the spec covers the
methodological deltas: the compiler eliminates a class of tests
(type-eliminated scenarios), RED includes compile errors, unit tests
live inline, error tests assert enum variants, and mocking requires
trait seams decided in the RED phase. Tooling (nextest, rstest,
proptest, mockall) is NOT re-specified — it lives in shodo's
`rust-testing-tools-spec.md`.

**Java TDD** follows the same delta approach: NullAway/enum/record
scenarios are type-eliminated, RED includes compile errors, value
constraints are tested once at record constructor boundaries,
mocking requires interface seams decided in the RED phase, and
ArchUnit turns layer rules into tests. Tooling (JUnit 6, AssertJ,
Mockito, Testcontainers, ArchUnit, JaCoCo) lives in shodo's
`java-testing-tools-spec.md`.


## Related Plugins

- **Zazen**: Code quality (naming, structure, zen)
- **Shodo**: Language standards (Python, Rust & Java) — owns testing TOOLS specs
- **Arche**: Behavioral principles for Claude Code
- **Gradient**: Plugin architecture


## Origin

Kinhin was extracted from Zazen to separate concerns:
- **Zazen**: Code quality principles (151 test cases)
- **Kinhin**: TDD practices (45 test cases)
