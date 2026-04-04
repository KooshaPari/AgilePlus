/**
 * Phenotype Traceability for Vitest
 * 
 * @example
 * ```typescript
 * import { test } from 'vitest';
 * import { tracesTo } from '@phenotype/tstreqt/vitest';
 * 
 * test('feature', tracesTo('FR-EXAMPLE-001'), () => {
 *   expect(true).toBe(true);
 * });
 * ```
 */

import type { TestContext } from 'vitest';

export type VitestTestFn = (ctx: TestContext) => void | Promise<void>;
export type VitestTestDecorator = (fn: VitestTestFn) => VitestTestFn;

function validateFrId(frId: string): boolean {
  const pattern = /^FR-[A-Z][A-Z0-9]*-\d{3,}(-[A-Z0-9]+)?$/;
  return pattern.test(frId);
}

/**
 * Creates a test decorator for Vitest that marks a test as tracing to an FR.
 * 
 * @param frId - The FR ID (e.g., 'FR-EXAMPLE-001')
 * @returns A Vitest test decorator
 * 
 * @example
 * ```typescript
 * import { test } from 'vitest';
 * import { tracesTo } from '@phenotype/tstreqt/vitest';
 * 
 * test('my test', tracesTo('FR-EXAMPLE-001'), ({ expect }) => {
 *   expect(true).toBe(true);
 * });
 * ```
 */
export function tracesTo(frId: string): VitestTestDecorator {
  if (!validateFrId(frId)) {
    throw new Error(`Invalid FR ID format: ${frId}. Expected: FR-XXXX-NNN`);
  }

  return (fn: VitestTestFn): VitestTestFn => {
    (fn as any).__frId = frId;
    (fn as any).__traceability = { frId, framework: 'vitest' };
    
    // Wrap to log trace info
    return async (ctx: TestContext) => {
      ctx.task.meta = { ...ctx.task.meta, frId };
      return fn(ctx);
    };
  };
}

/**
 * Marks a test as tracing to multiple FRs.
 */
export function tracesToMultiple(frIds: string[]): VitestTestDecorator {
  frIds.forEach(id => {
    if (!validateFrId(id)) {
      throw new Error(`Invalid FR ID format: ${id}. Expected: FR-XXXX-NNN`);
    }
  });

  return (fn: VitestTestFn): VitestTestFn => {
    (fn as any).__frIds = frIds;
    (fn as any).__traceability = { frIds, framework: 'vitest' };
    
    return async (ctx: TestContext) => {
      ctx.task.meta = { ...ctx.task.meta, frIds };
      return fn(ctx);
    };
  };
}

/**
 * Creates a describe block for a specific FR.
 */
export function describeFr(frId: string, description: string, fn: () => void): void {
  if (!validateFrId(frId)) {
    throw new Error(`Invalid FR ID format: ${frId}. Expected: FR-XXXX-NNN`);
  }

  const describe = (globalThis as any).describe;
  if (describe) {
    describe(`${frId}: ${description}`, fn);
  }
}
