# TypeScript TDD Patterns

## Overview

Practical test patterns for TypeScript ecosystems. For methodology,
see `@~/.claude/kinhin/spec/typescript-tdd/typescript-tdd-spec.md`.
This file covers PATTERNS for specific frameworks — what to test, how
to test it, and what NOT to test.

## React Testing

### Testing Library: Behavior Over Structure

**Requirement**: Use `@testing-library/react` with `@testing-library/user-event`.
Tests verify what the USER sees and does, not what the component renders internally.

```typescript
// @decision(ref="user can submit order with valid amount")
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';

test('submits order when amount is valid', async () => {
  const user = userEvent.setup();
  const onSubmit = vi.fn();
  render(<OrderForm onSubmit={onSubmit} />);

  await user.type(screen.getByRole('textbox', { name: /amount/i }), '100');
  await user.click(screen.getByRole('button', { name: /submit/i }));

  expect(onSubmit).toHaveBeenCalledWith(expect.objectContaining({ amount: 100 }));
});
```

### Query Priority

**Requirement**: Query by accessibility semantics, never by
implementation detail:

| Priority | Query | Use when |
|----------|-------|----------|
| 1st | `getByRole` | Always the default — buttons, inputs, headings |
| 2nd | `getByLabelText` | Form controls with labels |
| 3rd | `getByText` | Non-interactive text content |
| Last | `getByTestId` | Only when nothing semantic works |

**Never**: `querySelector`, `getByClassName`, `container.firstChild`.

### User Events Over Fire Events

**Requirement**: Use `userEvent` (not `fireEvent`) for realistic
interactions. `userEvent` simulates the full browser event sequence
(focus → keydown → keypress → input → keyup → change):

```typescript
// ❌ SCAFFOLD — fires one synthetic event, skips the real sequence
fireEvent.change(input, { target: { value: '100' } });

// ✅ DECISION — simulates real user typing
const user = userEvent.setup();
await user.type(input, '100');
```

### Network: MSW, Never Fetch Mocks

**Requirement**: Use MSW (Mock Service Worker) for HTTP mocking. It
intercepts at the network level, so the production fetch/axios/ky code
runs unmodified:

```typescript
import { http, HttpResponse } from 'msw';
import { setupServer } from 'msw/node';

const server = setupServer(
  http.get('/api/orders', () => HttpResponse.json([
    { id: '1', amount: 100, status: 'pending' },
  ])),
);

beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

test('displays order list', async () => {
  render(<OrderList />);
  expect(await screen.findByText('$100.00')).toBeInTheDocument();
});
```

**Never**: `jest.mock('axios')`, `vi.mock('./api')`, `global.fetch = vi.fn()`.
These mock the MECHANISM, not the BOUNDARY.

### Banned Patterns

| Pattern | Why banned | Lifecycle tag if found |
|---------|-----------|----------------------|
| Shallow rendering (`shallow()`) | Tests component in isolation from its children — misses integration bugs, couples to implementation | `@scaffold` — delete |
| Asserting hook internals (`result.current.count`) | Tests implementation, not behavior | `@scaffold` — delete |
| Permanent render-tree snapshots | Couples to markup structure; breaks on every refactor; assistant "updates" them reflexively | `@scaffold` — ALWAYS |
| Testing CSS classes | Classes are styling implementation, not behavior | `@scaffold` — delete |
| Prop-type tests under strict TS | `tsc` already verifies prop shapes | Redundant — don't write |
| `container.innerHTML` assertions | Tests HTML string, not user-visible behavior | `@scaffold` — delete |

### Component Test = Behavior Test

**Requirement**: The test describes what the USER observes, not what
the DOM contains:

```typescript
// ❌ WRONG — tests implementation
expect(container.querySelector('.error-message')).toHaveTextContent('Invalid');

// ✅ CORRECT — tests behavior
expect(screen.getByRole('alert')).toHaveTextContent('Invalid amount');
```

## Prisma / Database Testing

### Real Database, Not Mocks

**Requirement**: Test against a real PostgreSQL instance. The database
IS the system under test for data-layer code — mocking it removes the
thing being verified.

```typescript
// @contract(ref="order creation persists with correct fields")
test('creates order with all fields', async () => {
  const order = await prisma.order.create({
    data: { amount: 100, userId: testUser.id, status: 'PENDING' },
  });

  const found = await prisma.order.findUnique({ where: { id: order.id } });
  expect(found).toMatchObject({
    amount: 100,
    userId: testUser.id,
    status: 'PENDING',
  });
});
```

### Database Provisioning

| Method | When to use |
|--------|------------|
| **Testcontainers** (Docker) | CI, clean-room isolation, disposable |
| **Local Postgres** | Dev inner loop, faster startup |
| **In-memory SQLite** | Never for Prisma — dialect differences hide real bugs |

### Parallel Isolation

**Requirement**: Each test worker gets its own isolated namespace:

```typescript
// Schema-per-worker isolation
const schema = `test_worker_${process.env.JEST_WORKER_ID || process.env.VITEST_POOL_ID || '0'}`;

beforeAll(async () => {
  await prisma.$executeRawUnsafe(`CREATE SCHEMA IF NOT EXISTS "${schema}"`);
  await prisma.$executeRawUnsafe(`SET search_path TO "${schema}"`);
  // Run migrations against this schema
});

afterAll(async () => {
  await prisma.$executeRawUnsafe(`DROP SCHEMA IF EXISTS "${schema}" CASCADE`);
});
```

Alternative: transaction-per-test rollback (simpler but prevents
testing transaction behavior):

```typescript
beforeEach(async () => {
  await prisma.$executeRaw`BEGIN`;
});

afterEach(async () => {
  await prisma.$executeRaw`ROLLBACK`;
});
```

### When Mocking PrismaClient Is Acceptable

**Requirement**: Mocking `PrismaClient` (via `jest-mock-extended` or
manual stubs) is over-mocking in MOST cases. It is acceptable ONLY
for:

- **Pure functions** that receive already-fetched data and transform it
  — the function under test never calls Prisma
- **Unit-testing a calculation** that happens to live in a service with
  a Prisma dependency — extract the calculation to a pure function and
  test THAT without Prisma

```typescript
// ✅ CORRECT — test the pure calculation, not the service wrapper
function calculateOrderTotal(items: OrderItem[]): number {
  return items.reduce((sum, item) => sum + item.price * item.quantity, 0);
}

test('calculates total from items', () => {
  const items = [
    { price: 10, quantity: 2 },
    { price: 5, quantity: 3 },
  ];
  expect(calculateOrderTotal(items)).toBe(35);
});
```

### Contract Tests

**Requirement**: Tests that verify database constraints, indexes,
and migration behavior are `@contract`:

```typescript
// @contract(ref="orders.user_id foreign key")
test('rejects order with non-existent user', async () => {
  await expect(
    prisma.order.create({ data: { amount: 100, userId: 'nonexistent' } }),
  ).rejects.toThrow(/Foreign key constraint/);
});

// @contract(ref="unique index on orders.external_id")
test('rejects duplicate external_id', async () => {
  await prisma.order.create({ data: { ...validOrder, externalId: 'dup' } });
  await expect(
    prisma.order.create({ data: { ...validOrder, externalId: 'dup' } }),
  ).rejects.toThrow(/Unique constraint/);
});
```

### Raw SQL Tests

**Requirement**: Any `$queryRaw` / `$queryRawUnsafe` must be tested
against the real database — the query syntax, the parameter binding,
and the result shape are all invisible to `tsc`:

```typescript
// @contract(ref="analytics aggregate query")
test('aggregates orders by status', async () => {
  // Arrange: seed known data
  await seedOrders([
    { status: 'PENDING', amount: 100 },
    { status: 'PENDING', amount: 200 },
    { status: 'COMPLETED', amount: 50 },
  ]);

  // Act: run the raw query
  const result = await prisma.$queryRaw`
    SELECT status, COUNT(*)::int as count, SUM(amount)::int as total
    FROM orders GROUP BY status ORDER BY status
  `;

  // Assert: verify against known data
  expect(result).toEqual([
    { status: 'COMPLETED', count: 1, total: 50 },
    { status: 'PENDING', count: 2, total: 300 },
  ]);
});
```
