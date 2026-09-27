# TypeScript TDD Specification

## Overview

TypeScript-specific Test-Driven Development requirements. For universal
TDD methodology, see `@~/.claude/kinhin/spec/tdd/tdd-spec.md`. For
TypeScript testing PATTERNS (React, Prisma), see
`@~/.claude/kinhin/context/examples/typescript-tdd-patterns.md` — this
spec covers METHODOLOGY only.

TDD in TypeScript sits between Python and Rust. Types are **erased at
runtime and unsound by design** — this is the central difference from
Rust and Java. The type system eliminates a class of tests only within
the trust boundary; outside it (JSON.parse, HTTP, env, DB rows, `any`
casts), every value is unknown at runtime regardless of what the type
annotation says.

## Principle 1: Do Not Test What Strict TS Proves

### The Trust Boundary

**Requirement**: Under `strict` + `noUncheckedIndexedAccess` +
`exactOptionalPropertyTypes`, do NOT test null/undefined handling or
shape validation for internal typed calls. The compiler proves these
within the trust boundary.

| Python/Jest test | TypeScript equivalent |
|------------------|----------------------|
| `expect(result).toBeDefined()` | Redundant — typed return, `strictNullChecks` active |
| `expect(result).toHaveProperty('id')` | Redundant — interface enforces the shape |
| `expect(typeof result.name).toBe('string')` | Redundant — type annotation is the proof |
| `expect(fn(null)).toThrow()` | Redundant if parameter is `param: User`, not `param: User \| null` |

### What Remains Testable

The type system does NOT prove business rules or runtime boundaries.
These still demand TDD:

- **Runtime entry points**: `JSON.parse`, HTTP request/response bodies,
  environment variables, database rows, WebSocket messages, URL params
- **`any` and `as` casts**: each cast reopens the class of tests the
  type system eliminated. Ban `as` and `any` in non-test code or
  require a justification comment — each cast is a hole in the trust
  boundary
- **Value constraints within a valid type**: discount ≤ 25%, email
  format, non-empty string
- **Behavior, state transitions, integration, side effects**

### Boundary Validation Schemas

**Requirement**: Zod, Valibot, or ArkType schemas that validate runtime
input are `@contract` tests — they pin the shape another party depends
on (an API consumer, a client, a database migration).

```typescript
// @contract(ref="POST /api/orders request body")
describe('OrderInput schema', () => {
  it.each([
    [{ amount: -1 }, 'negative amount'],
    [{ amount: 'ten' }, 'string amount'],
    [{}, 'missing amount'],
  ])('rejects %s (%s)', (input) => {
    expect(OrderInput.safeParse(input).success).toBe(false);
  });
});
```

## Principle 2: RED Includes Type Errors

### The Trap: Runners Strip Types

**Requirement**: `tsc --noEmit` is step 1 of every RED loop. This is
non-negotiable.

Vitest (with esbuild/swc) and Jest (with `@swc/jest` or `babel-jest`)
**strip type annotations without checking them**. A test file with a
type error compiles, runs, and passes — the type error is invisible to
the runner. This is the single most dangerous gap in TypeScript TDD.

```
🔴 RED      → tsc --noEmit FIRST. Type error = valid RED.
              Then run the test. Assertion failure = valid RED.
              Both must be checked. A test that "passes" with
              tsc errors is not green — it is unverified.
🟢 GREEN    → tsc clean AND test passes.
🔵 REFACTOR → tsc clean AND tests pass after changes.
⚪ PRUNE     → Scaffold tests deleted. tsc still clean.
```

### Runner Choice

**Requirement**: Use swc or esbuild for speed, but run `tsc --noEmit`
as a separate step — never rely on the runner for type checking.

| Runner | Type-checks? | Speed | Recommendation |
|--------|-------------|-------|----------------|
| Vitest (esbuild) | No | Fast | Use + separate `tsc` |
| Jest + `@swc/jest` | No | Fast | Use + separate `tsc` |
| Jest + `ts-jest` | Yes | Slow | Avoid for inner loop |
| Vitest `--typecheck` | Yes | Medium | Alternative to separate `tsc` |

## Principle 3: Type-Level Tests

### Types Are API When Exported

**Requirement**: For library code or public API types, use type-level
testing to verify the type contracts themselves:

```typescript
// @contract(ref="public API — UserService return type")
import { expectTypeOf } from 'vitest';
import type { UserService } from './user.service';

test('getUser returns User, not User | undefined', () => {
  expectTypeOf<ReturnType<UserService['getUser']>>()
    .toEqualTypeOf<Promise<User>>();
});
```

- `expectTypeOf` (Vitest built-in) or `tsd` for standalone — these
  are `@contract` tests
- Internal type assertions (verifying a helper's generic resolves
  correctly) are `@scaffold`

## Principle 4: Module Mocks Are Scaffold Smell

### The Problem

`vi.mock()` and `jest.mock()` hoist to the top of the file, bypass the
module system, and couple the test to the import graph. ESM makes this
worse: Jest's ESM mock support is fragile, and Vitest's `vi.mock`
hoisting interacts poorly with top-level await and re-exports.

### The Rule

**Requirement**: Module mocks (`vi.mock('module')`, `jest.mock('module')`)
are `@scaffold` by default. If a test cannot run without mocking an
entire module, the production code needs a dependency injection seam:

```typescript
// ❌ SCAFFOLD — module mock couples test to import graph
vi.mock('./email.service', () => ({
  sendEmail: vi.fn(),
}));

// ✅ DECISION — injected dependency, testable without module mock
class OrderService {
  constructor(private readonly email: EmailPort) {}
}

// Test with a simple stub — no mock hoisting, no import coupling
const fakeEmail: EmailPort = { sendEmail: async () => {} };
const service = new OrderService(fakeEmail);
```

**Exception**: Mocking environment or infrastructure boundaries
(`process.env`, `fetch` via MSW, `Date.now`) is acceptable — these are
genuine I/O boundaries.

## Lifecycle Tagging

### Jest: File Suffix Convention

Jest has no built-in tag filter. Use file suffixes to separate scaffold
from permanent tests:

```
tests/
├── order.service.test.ts              # permanent tests
├── order.service.scaffold.test.ts     # scaffold — deleted at prune
└── order.service.contract.test.ts     # contract tests (optional suffix)
```

**Prune = delete `*.scaffold.test.ts` files.**

CI exclusion:
```json
// jest.config.ts
{
  "testPathIgnorePatterns": ["\\.scaffold\\.test\\."]
}
```

### Vitest: Project or Name Prefix

Vitest supports `--project` for workspace-based filtering or test-name
prefix convention:

```typescript
// vitest.config.ts — workspace approach
export default defineWorkspace([
  { test: { include: ['**/*.test.ts'], exclude: ['**/*.scaffold.test.ts'] } },
  { test: { name: 'scaffold', include: ['**/*.scaffold.test.ts'] } },
]);
```

CI exclusion: `vitest --project default` (excludes scaffold workspace).

## Runner Configuration

### Jest

**Requirement**: Never `--bail`. Configure for batch processing:

```bash
# Inner loop — changed files only
jest --no-bail --maxWorkers=50% --randomize \
     --json --outputFile=test-results.json \
     --findRelatedTests <changed-files>

# Full gate — before PR
jest --no-bail --maxWorkers=50% --randomize \
     --json --outputFile=test-results.json

# Changed since base branch
jest --no-bail --onlyChanged --changedSince=main
```

Key flags:
- `--no-bail` — see ALL failures, never stop at first
- `--maxWorkers=50%` — parallel without starving the system
- `--randomize` — detect order-dependent tests (Jest ≥ 29.2)
- `--json --outputFile` — structured output for the assistant
- `--findRelatedTests` — dependency-aware selection

### Vitest

**Requirement**: Never `bail: 1` in config. Configure:

```typescript
// vitest.config.ts
export default defineConfig({
  test: {
    bail: 0,                    // never bail
    sequence: { shuffle: true },// randomize order
    reporters: ['json'],        // structured output
    pool: 'forks',              // per-test process isolation
  },
});
```

```bash
# Inner loop — affected tests only
vitest --changed --reporter=json

# Programmatic — daemon-like inner loop
# Use startVitest() API for sub-second re-runs in the agent harness
```

### Mutation Testing

**Requirement**: Stryker, scoped to the diff:

```bash
npx stryker run --mutate "src/changed-file.ts"
```

Never run full-repo mutation in the inner loop. Scope to changed files.
Run once in the Prune phase to verify mutation parity.

## Coverage

Same construction-time vs retention distinction as the universal spec:

### Required Coverage (at GREEN)
- **Scenario coverage**: 100% of matrix cells
- **Branch coverage**: ≥ 90% on changed code

### Required Retention (after PRUNE)
- **Mutation parity**: K₁ ⊇ K₀
- **Tag completeness**: 0 untagged tests
- **Branch coverage**: reported, 80% floor as smoke alarm (not a gate)

Type-eliminated scenarios (Principle 1) count as covered BY
CONSTRUCTION — document the type, not a redundant test.

## Anti-Hallucination (TypeScript Form)

The universal anti-hallucination patterns adapt to TypeScript's mixed
static/dynamic nature:

- **Type safety validation** → under `strict`, the compiler checks
  contracts for internal code. `tsc` IS the anti-hallucination gate for
  shape and nullability — within the trust boundary
- **Constraint validation** → boundary schemas (zod/valibot) + business
  rule tests. Every `as` cast reopens the hallucination surface
- **Cross-function consistency** → shared types prevent structural
  drift; test behavioral consistency that types cannot see (ordering,
  side effects, timing)
- **Explicit validation** → every `Promise` is awaited or returned
  (no floating promises — `@typescript-eslint/no-floating-promises`).
  Every `Result`/error union is narrowed before access

### Snapshot Tests

**Requirement**: Snapshot tests (`.toMatchSnapshot()`,
`.toMatchInlineSnapshot()`) default to `@scaffold` — they verify the
current shape, not the intended shape.

**Exception**: A snapshot of a serialized format consumed by an external
party (API response, wire protocol, file format) is `@contract`.
Render-tree snapshots are NEVER permanent — component markup is
implementation, not specification.

## Naming

**Requirement**: Behavior-first names. `describe` scopes the component,
`it`/`test` states the scenario:

```typescript
// Component implied by describe, scenario explicit
describe('OrderService', () => {
  it('applies loyalty discount for premium users', () => { ... });
  it('rejects negative amounts', () => { ... });
  it('rounds to nearest cent', () => { ... });
});
```

Avoid implementation-revealing names:
```typescript
// ❌ WRONG — names the mechanism
it('calls calculateDiscount with user tier', () => { ... });

// ✅ CORRECT — names the behavior
it('applies 15% discount for 5-year premium members', () => { ... });
```
