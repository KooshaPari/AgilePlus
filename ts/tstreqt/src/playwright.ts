/**
 * Phenotype Traceability for Playwright
 * 
 * @example
 * ```typescript
 * import { test } from '@playwright/test';
 * import { tracesTo } from '@phenotype/tstreqt/playwright';
 * 
 * test('feature', tracesTo('FR-EXAMPLE-001'), async ({ page }) => {
 *   await page.goto('/');
 * });
 * ```
 */

import type { Page, TestInfo } from '@playwright/test';

export type PlaywrightTestFn = (args: { page: Page }, testInfo: TestInfo) => Promise<void>;
export type PlaywrightTestDecorator = (fn: PlaywrightTestFn) => PlaywrightTestFn;

function validateFrId(frId: string): boolean {
  const pattern = /^FR-[A-Z][A-Z0-9]*-\d{3,}(-[A-Z0-9]+)?$/;
  return pattern.test(frId);
}

/**
 * Creates a test decorator for Playwright that marks a test as tracing to an FR.
 * 
 * @param frId - The FR ID
 * @returns A Playwright test decorator
 * 
 * @example
 * ```typescript
 * import { test } from '@playwright/test';
 * import { tracesTo } from '@phenotype/tstreqt/playwright';
 * 
 * test('login flow', tracesTo('FR-AUTH-001'), async ({ page }) => {
 *   await page.goto('/login');
 *   // ...
 * });
 * ```
 */
export function tracesTo(frId: string): PlaywrightTestDecorator {
  if (!validateFrId(frId)) {
    throw new Error(`Invalid FR ID format: ${frId}. Expected: FR-XXXX-NNN`);
  }

  return (fn: PlaywrightTestFn): PlaywrightTestFn => {
    return async (args, testInfo) => {
      // Attach FR info to test annotations
      testInfo.annotations.push({ type: 'frId', description: frId });
      
      // Add to test title for visibility
      testInfo.title = `[${frId}] ${testInfo.title}`;
      
      return fn(args, testInfo);
    };
  };
}

/**
 * Marks a test as tracing to multiple FRs.
 */
export function tracesToMultiple(frIds: string[]): PlaywrightTestDecorator {
  frIds.forEach(id => {
    if (!validateFrId(id)) {
      throw new Error(`Invalid FR ID format: ${id}. Expected: FR-XXXX-NNN`);
    }
  });

  return (fn: PlaywrightTestFn): PlaywrightTestFn => {
    return async (args, testInfo) => {
      frIds.forEach(frId => {
        testInfo.annotations.push({ type: 'frId', description: frId });
      });
      testInfo.title = `[${frIds.join(',')}] ${testInfo.title}`;
      return fn(args, testInfo);
    };
  };
}
