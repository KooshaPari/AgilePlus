/**
 * Phenotype Traceability for Node.js Test Runner
 * 
 * @example
 * ```typescript
 * import { describeFr, tracesTo } from '@phenotype/tstreqt/node';
 * 
 * describeFr('FR-EXAMPLE-001', 'Feature', () => {
 *   it('works', tracesTo('FR-EXAMPLE-002'), async () => {
 *     // test code
 *   });
 * });
 * ```
 */

export type NodeTestFn = () => void | Promise<void>;
export type NodeTestDecorator = (fn: NodeTestFn) => NodeTestFn;

function validateFrId(frId: string): boolean {
  const pattern = /^FR-[A-Z][A-Z0-9]*-\d{3,}(-[A-Z0-9]+)?$/;
  return pattern.test(frId);
}

/**
 * Creates a test decorator for Node.js test runner.
 * 
 * @param frId - The FR ID
 * @returns A Node.js test decorator
 */
export function tracesTo(frId: string): NodeTestDecorator {
  if (!validateFrId(frId)) {
    throw new Error(`Invalid FR ID format: ${frId}. Expected: FR-XXXX-NNN`);
  }

  return (fn: NodeTestFn): NodeTestFn => {
    (fn as any).__frId = frId;
    (fn as any).__traceability = { frId, framework: 'node' };
    
    // Log if verbose
    const wrapped = async () => {
      if (process.env.VERBOSE) {
        console.log(`[TRACE] Test traces to: ${frId}`);
      }
      return fn();
    };
    
    // Copy metadata
    (wrapped as any).__frId = frId;
    (wrapped as any).__traceability = { frId, framework: 'node' };
    
    return wrapped;
  };
}

/**
 * Marks a test as tracing to multiple FRs.
 */
export function tracesToMultiple(frIds: string[]): NodeTestDecorator {
  frIds.forEach(id => {
    if (!validateFrId(id)) {
      throw new Error(`Invalid FR ID format: ${id}. Expected: FR-XXXX-NNN`);
    }
  });

  return (fn: NodeTestFn): NodeTestFn => {
    const wrapped = async () => {
      if (process.env.VERBOSE) {
        console.log(`[TRACE] Test traces to: ${frIds.join(', ')}`);
      }
      return fn();
    };
    
    (wrapped as any).__frIds = frIds;
    (wrapped as any).__traceability = { frIds, framework: 'node' };
    
    return wrapped;
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
