/**
 * Phenotype Traceability for AVA
 * 
 * @example
 * ```typescript
 * import test from 'ava';
 * import { withFr } from '@phenotype/tstreqt/ava';
 * 
 * test('feature', withFr('FR-EXAMPLE-001'), (t) => { 
 *   t.pass();
 * });
 * ```
 */

import type { ExecutionContext } from 'ava';

export type AVATestFn = (t: ExecutionContext) => void | Promise<void>;
export type AVATestDecorator = (fn: AVATestFn) => AVATestFn;

function validateFrId(frId: string): boolean {
  const pattern = /^FR-[A-Z][A-Z0-9]*-\d{3,}(-[A-Z0-9]+)?$/;
  return pattern.test(frId);
}

/**
 * Creates a test decorator for AVA that marks a test as tracing to an FR.
 * 
 * @param frId - The FR ID
 * @returns An AVA test decorator
 */
export function withFr(frId: string): AVATestDecorator {
  if (!validateFrId(frId)) {
    throw new Error(`Invalid FR ID format: ${frId}. Expected: FR-XXXX-NNN`);
  }

  return (fn: AVATestFn): AVATestFn => {
    return async (t: ExecutionContext) => {
      // Add FR context
      t.context = { ...t.context, frId };
      
      // Log if verbose
      if (process.env.VERBOSE) {
        console.log(`[TRACE] ${t.title} -> ${frId}`);
      }
      
      return fn(t);
    };
  };
}

/**
 * Marks a test as tracing to multiple FRs.
 */
export function withFrs(frIds: string[]): AVATestDecorator {
  frIds.forEach(id => {
    if (!validateFrId(id)) {
      throw new Error(`Invalid FR ID format: ${id}. Expected: FR-XXXX-NNN`);
    }
  });

  return (fn: AVATestFn): AVATestFn => {
    return async (t: ExecutionContext) => {
      t.context = { ...t.context, frIds };
      return fn(t);
    };
  };
}
