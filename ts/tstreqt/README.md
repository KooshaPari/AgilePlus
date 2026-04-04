# @phenotype/tstreqt

Phenotype Traceability for TypeScript/JavaScript tests.

## Installation

```bash
npm install @phenotype/tstreqt
```

## Usage

### Jest

```typescript
import { tracesTo } from '@phenotype/tstreqt';

test('feature', tracesTo('FR-EXAMPLE-001'), () => {
  expect(true).toBe(true);
});
```

### Vitest

```typescript
import { test } from 'vitest';
import { tracesTo } from '@phenotype/tstreqt/vitest';

test('feature', tracesTo('FR-EXAMPLE-001'), () => {
  expect(true).toBe(true);
});
```

### Playwright

```typescript
import { test } from '@playwright/test';
import { tracesTo } from '@phenotype/tstreqt/playwright';

test('feature', tracesTo('FR-EXAMPLE-001'), async ({ page }) => {
  await page.goto('/');
});
```

### Mocha

```typescript
import { tracesTo, describeFr } from '@phenotype/tstreqt/mocha';

describeFr('FR-EXAMPLE-001', 'Feature', () => {
  it('works', tracesTo('FR-EXAMPLE-002'), (done) => {
    done();
  });
});
```

### AVA

```typescript
import test from 'ava';
import { withFr } from '@phenotype/tstreqt/ava';

test('feature', withFr('FR-EXAMPLE-001'), (t) => {
  t.pass();
});
```

### TAP

```typescript
import { tracesTo } from '@phenotype/tstreqt/tap';
import tap from 'tap';

tap.test('feature', tracesTo('FR-EXAMPLE-001'), (t) => {
  t.end();
});
```

### Node.js Test Runner

```typescript
import { tracesTo, describeFr } from '@phenotype/tstreqt/node';

describeFr('FR-EXAMPLE-001', 'Feature', () => {
  it('works', tracesTo('FR-EXAMPLE-002'), async () => {
    // test code
  });
});
```

## License

Apache-2.0
