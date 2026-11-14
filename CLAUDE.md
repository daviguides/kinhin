# Kinhin - Claude Code Project Instructions

## Project Overview

**Kinhin** (経行 / 禅歩) is the practice of walking meditation in Zen Buddhism, serving as active transition between periods of zazen (seated meditation).

**Philosophy**: Like walking meditation with deliberate steps, Kinhin guides developers through TDD with mindful progression - red, green, refactor, prune.


## Structure

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
│   │   ├── examples/         # Test templates (Python) + per-language patterns + lifecycle tags + anti-patterns
│   │   └── checklists/       # TDD checklists (universal)
│   └── prompts/
├── commands/
└── skills/
```


## Commands

| Command | Purpose |
|---------|---------|
| `/kinhin:load [python\|rust\|java\|typescript\|all]` | Load TDD principles — universal core + language specifics |
| `/kinhin:prune` | Run the prune pass on the current branch's test files |

**Language resolution**: explicit argument wins; without argument,
detect markers (`pyproject.toml` → python, `Cargo.toml` → rust,
`pom.xml`/`build.gradle`/`build.gradle.kts` → java,
`tsconfig.json` → typescript) in cwd,
ancestors, and shallow subdirectories, loading the UNION
(monorepos load multiple). Nothing detected → python fallback.
Universal files (methodology, prune spec, runner spec, guides,
checklist, prune guide, runner guide, lifecycle tags) load ALWAYS.

**4-phase TDD cycle**: Red → Green → Refactor → **Prune**. Every
test is born with a lifecycle tag (`@scaffold` by default). The
burden of proof is on keeping a test. Scaffolds die at prune;
permanent tests (`@decision`, `@contract`, `@incident`) carry a
ref pointing to their authority source.

**Runner contract**: no bail, parallel by default, randomized order,
three-way flaky classification, changed-set selection in inner loop,
structured output. Language-specific configs in each delta spec.

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

**TypeScript TDD**: types are erased and unsound — the central
difference from Rust and Java. RED includes type errors (`tsc
--noEmit` is step 1). Do not test what strict TS proves. Module
mocks are scaffold smell. Snapshot tests default to `@scaffold`.


## Related Plugins

- **Zazen**: Code quality (naming, structure, zen)
- **Shodo**: Language standards (Python, Rust & Java) — owns testing TOOLS specs
- **Arche**: Behavioral principles for Claude Code
- **Gradient**: Plugin architecture


## Origin

Kinhin was extracted from Zazen to separate concerns:
- **Zazen**: Code quality principles (151 test cases)
- **Kinhin**: TDD practices (45 test cases)

## Runner entries (per mode and per language)

`/kinhin:load` (interactive) keeps loading the union: universal +
detected languages. The runner does not detect: it compiles one thin
entry per mode plus one per task language, so each SDK session pays
only what its mode uses. Entries are files in `kinhin/prompts/` with
direct `Read:` lines only (no entry references another entry; the
compiler caps depth at 2) and carry no card: the runner never shows
it, and every token in the system prompt is paid on every API call.

| Entry | Files | Mode |
|---|---|---|
| `load-planning` | tdd-spec, tdd-feature-development, tdd-bugfix-guide, tdd-refactoring-guide | PLANNING, PLAN_REVIEWING |
| `load-implementing` | tdd-spec, tdd-anti-patterns | IMPLEMENTING |
| `load-validating` | tdd-checklist, tdd-runner-guide, tdd-anti-patterns | VALIDATING |
| `load-python` | python-tdd-spec, tdd-unit/integration/property/error-handling/performance-tests | IMPLEMENTING, VALIDATING |
| `load-rust` / `load-java` / `load-typescript` | `<lang>-tdd-spec`, `<lang>-tdd-patterns` | IMPLEMENTING, VALIDATING |
| `load-pruning` | tdd-spec, tdd-prune-spec, tdd-prune-guide, tdd-lifecycle-tags, tdd-anti-patterns | `test-prune` workflow: IMPLEMENTING (pruner) and VALIDATING (reviewer of the deletions) |

Classification notes:
- `tdd-spec` sits in planning AND implementing: planning needs authority
  per block, scenario matrix, 3-question gate and dependency order;
  implementing needs RED evidence, naming, structure and tags at birth.
  The two never share a session, so nothing is paid twice.
- The implementing author tags tests and never prunes or configures the
  test runner: prune criteria, collapse, retention and the prune CLI live
  in `tdd-prune-spec` (load-pruning, `/kinhin:prune`, `load`); the
  Runner Contract in `tdd-runner-spec` and pytest configuration in
  `python-tdd-runner-spec` (interactive `load` only). The author keeps
  two lines (tdd-spec / python-tdd-spec, Running Tests): never bail on
  the first failure, never rerun until green.
- `tdd-anti-patterns` is language-generic (it lived under Python only by
  placement), so it moves to implementing and validating.
- Text a runner entry compiles never points at a file that entry does
  not load (it states the rule instead); cross-file references live only
  in files the interactive `load` reads.
- `tdd-runner-guide` is in validating (flake protocol, full runs) and
  the interactive `load`; `tdd-lifecycle-tags` (collapse example) rides
  with `load-pruning` and `load`.
- The validator gets the checklist, not `tdd-spec`: the checklist is the
  distilled, checkable form of the spec.
- `tdd-implementation-guide` (prompt templates for a human driving an
  LLM; every rule is in tdd-spec) is interactive-only: in `load.md`.
  `tdd-prune-guide` rides only with `load-pruning`: the
  self-implementation modes never prune. The runner prunes once per
  milestone, in a `test-prune` task (dao-cli workflow) the instrument
  appends to each new milestone; pruner and reviewer both load
  `load-pruning`, in separate sessions. `/kinhin:prune` reads the guide
  from disk.
- Prune judges independently: tags are the author's claim (a hint);
  a permanent tag survives when its `ref` resolves. Tests cited as
  completeness evidence are never deleted. The net is deterministic
  (suite green, evidence preserved) plus the reviewer; no mutation
  testing (removed, owner decision 2026-11-14).
- Coverage is informational, never a target (owner decision
  2026-11-13): no line/branch threshold anywhere in the plugin. An
  owner coverage policy (`owners.yaml coverage`) is enforced by the
  runner, not by the skill (one line in tdd-spec).
- Per-language content (tag syntax, run commands, flake commands, CI
  profiles, structured output) lives ONLY in `<lang>-tdd-spec`, which
  always rides with implementing/validating. Universal files stay
  language-neutral. Python's run modes, flake commands and structured
  output moved to `python-tdd-runner-spec` (interactive only); the
  runner's author uses `make test`. Exception: Java/TypeScript diagnostic commands and
  the TypeScript seed rerun sit in tdd-runner-guide until their specs
  carry them.

Compiled size (`compile_plugin("kinhin", prompt=<entry>)`, ~chars/3.6).
The compiler reads the INSTALLED bundle (`~/.claude/kinhin`): measure
after install, or compile a copy of the source under a scratch HOME.

| Entry | Chars | ~Tokens |
|---|---|---|
| `load` (union, all languages) | 185,750 | 51.6k |
| `load-planning` | 37,378 | 10.4k |
| `load-implementing` | 38,297 | 10.6k |
| `load-validating` | 27,222 | 7.6k |
| `load-pruning` | 57,680 | 16.0k |
| `load-python` | 14,552 | 4.0k |
| `load-rust` | 18,419 | 5.1k |
| `load-java` | 20,733 | 5.8k |
| `load-typescript` | 19,026 | 5.3k |

When adding a spec file: put its `Read:` line in `load.md` (interactive
union) AND in every runner entry whose mode uses it.

---

## Releasing — mandatory workflow

Every plugin modification MUST follow this sequence. No exceptions.

### 1. Bump version

Patch for fixes/tweaks, minor for new skills or behavioral changes:

```bash
# From gradients/kinhin/
# Edit .claude-plugin/plugin.json version field
# Also update install.sh header if it shows a version
```

### 2. Commit and push

```bash
git add -A && git commit -m "bump: vX.Y.Z — <what changed>"
git tag -a vX.Y.Z -m "<what changed>"
git push && git push origin vX.Y.Z
```

### 3. Run install.sh

```bash
~/work/sources/continuum/gradients/kinhin/install.sh
```

Note: install.sh clones from the GitHub remote (not local source), so
the push in step 2 must land before running it.

### 4. Verify cache is not stale

The plugin cache (`~/.claude/plugins/cache/daviguides/kinhin/`) is
unstable — even after a successful install, it can preserve stale
state from previous versions. This is a known unresolved bug in the
Claude Code plugin system.

After install, always verify:

```bash
# Compare installed vs source timestamps
diff <(ls -lR ~/.claude/kinhin/skills/) <(ls -lR skills/)

# Check cache version matches
ls ~/.claude/plugins/cache/daviguides/kinhin/

# If stale, nuke cache and reinstall
rm -rf ~/.claude/plugins/cache/daviguides/kinhin/
rm -rf ~/.claude/kinhin/
./install.sh
```

Do NOT move to the next task with a stale cache — the session will
load outdated skills silently.
