/**
 * Phenotype Traceability for TAP
 * 
 * @example
 * ```typescript
 * import { tracesTo } from '@phenotype/tstreqt/tap';
 * import tap from 'tap';
 * 
 * tap.test('feature', tracesTo('FR-EXAMPLE-001'), (t) => { 
 *   t.end();
 * });
 * ```
 */

import type { Test } from 'tap';

export type TAPTestFn = (t: Test) => void | Promise<void>;
export type TAPTestDecorator = (fn: TAPTestFn) => TAPTestFn;

function validateFrId(frId: string): boolean {
  const pattern = /^FR-[A-Z][A-Z0-9]*-\d{3,}(-[A-Z0-9]+)?$/;
  return pattern.test(frId);
}

/**
 * Creates a test decorator for TAP that marks a test as tracing to an FR.
 * 
 * @param frId - The FR ID
 * @returns A TAP test decorator
 */
export function tracesTo(frId: string): TAPTestDecorator {
  if (!validateFrId(frId)) {
    throw new Error(`Invalid FR ID format: ${frId}. Expected: FR-XXXX-NNN`);
  }

  return (fn: TAPTestFn): TAPTestFn => {
    return (t: Test) => {
      // Add FR as a diagnostic comment
      t.comment(`Traces to: ${frId}`);
      
      // Store for collection
      (fn as any).__frId = frId;
      (fn as any).__traceability = { frId, framework: 'tap' };
      
      return fn(t);
    };
  };
}

/**
 * Marks a test as tracing to multiple FRs.
 */
export function tracesToMultiple(frIds: string[]): TAPTestDecorator {
  frIds.forEach(id => {
    if (!validateFrId(id)) {
      throw new Error(`Invalid FR ID format: ${id}. Expected: FR-XXXX-NNN`);
    }
  });

  return (fn: TAPTestFn): TAPTestFn => {
    return (t: Test) => {
      t.comment(`Traces to: ${frIds.join(', ')}`);
      (fn as any).__frIds = frIds;
      (fn as any).__traceability = { frIds, framework: 'tap' };
      return fn(t);
    };
  };
}
