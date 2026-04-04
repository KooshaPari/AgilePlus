/**
 * Phenotype Traceability for Mocha
 * 
 * @example
 * ```typescript
 * import { tracesTo, describeFr } from '@phenotype/tstreqt/mocha';
 * 
 * describeFr('FR-EXAMPLE-001', 'Feature', () => {
 *   it('works', tracesTo('FR-EXAMPLE-002'), (done) => { 
 *     done();
 *   });
 * });
 * ```
 */

export type MochaTestFn = (done: (err?: any) => void) => void | Promise<void>;
export type MochaTestDecorator = (fn: MochaTestFn) => MochaTestFn;

function validateFrId(frId: string): boolean {
  const pattern = /^FR-[A-Z][A-Z0-9]*-\d{3,}(-[A-Z0-9]+)?$/;
  return pattern.test(frId);
}

/**
 * Creates a test decorator for Mocha that marks a test as tracing to an FR.
 * 
 * @param frId - The FR ID
 * @returns A Mocha test decorator
 */
export function tracesTo(frId: string): MochaTestDecorator {
  if (!validateFrId(frId)) {
    throw new Error(`Invalid FR ID format: ${frId}. Expected: FR-XXXX-NNN`);
  }

  return (fn: MochaTestFn): MochaTestFn => {
    (fn as any).__frId = frId;
    (fn as any).__traceability = { frId, framework: 'mocha' };
    
    // Wrap to add context
    return function(done: (err?: any) => void) {
      const test = (this as any).test || (this as any).currentTest;
      if (test) {
        test.title = `[${frId}] ${test.title}`;
      }
      return fn.call(this, done);
    };
  };
}

/**
 * Marks a test as tracing to multiple FRs.
 */
export function tracesToMultiple(frIds: string[]): MochaTestDecorator {
  frIds.forEach(id => {
    if (!validateFrId(id)) {
      throw new Error(`Invalid FR ID format: ${id}. Expected: FR-XXXX-NNN`);
    }
  });

  return (fn: MochaTestFn): MochaTestFn => {
    (fn as any).__frIds = frIds;
    (fn as any).__traceability = { frIds, framework: 'mocha' };
    
    return function(done: (err?: any) => void) {
      const test = (this as any).test || (this as any).currentTest;
      if (test) {
        test.title = `[${frIds.join(',')}] ${test.title}`;
      }
      return fn.call(this, done);
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
