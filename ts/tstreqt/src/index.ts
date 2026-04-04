/**
 * Phenotype Traceability for Jest
 * 
 * @example
 * ```typescript
 * import { tracesTo } from '@phenotype/tstreqt';
 * 
 * test('feature', tracesTo('FR-EXAMPLE-001'), () => {
 *   expect(true).toBe(true);
 * });
 * ```
 */

export type TestFn = () => void | Promise<void>;
export type TestDecorator = (fn: TestFn) => TestFn;

/**
 * Validates an FR ID format: FR-XXXX-NNN or FR-XXXX-NNN-YYY
 */
function validateFrId(frId: string): boolean {
  const pattern = /^FR-[A-Z][A-Z0-9]*-\d{3,}(-[A-Z0-9]+)?$/;
  return pattern.test(frId);
}

/**
 * Creates a test decorator that marks a test as tracing to a Feature Requirement.
 * 
 * @param frId - The FR ID (e.g., 'FR-EXAMPLE-001')
 * @returns A test decorator function
 * 
 * @example
 * ```typescript
 * test('my test', tracesTo('FR-EXAMPLE-001'), () => {
 *   // test code
 * });
 * ```
 */
export function tracesTo(frId: string): TestDecorator {
  if (!validateFrId(frId)) {
    throw new Error(`Invalid FR ID format: ${frId}. Expected: FR-XXXX-NNN`);
  }

  return (fn: TestFn): TestFn => {
    // Attach metadata to the function for collection
    (fn as any).__frIds = [frId];
    (fn as any).__traceability = { frId, framework: 'jest' };
    
    // Log in verbose mode
    if (process.env.VERBOSE) {
      console.log(`[TRACE] Test traces to: ${frId}`);
    }
    
    return fn;
  };
}

/**
 * Marks a test as tracing to multiple Feature Requirements.
 * 
 * @param frIds - Array of FR IDs
 * @returns A test decorator function
 * 
 * @example
 * ```typescript
 * test('my test', tracesToMultiple(['FR-001', 'FR-002']), () => {
 *   // test code
 * });
 * ```
 */
export function tracesToMultiple(frIds: string[]): TestDecorator {
  frIds.forEach(id => {
    if (!validateFrId(id)) {
      throw new Error(`Invalid FR ID format: ${id}. Expected: FR-XXXX-NNN`);
    }
  });

  return (fn: TestFn): TestFn => {
    (fn as any).__frIds = frIds;
    (fn as any).__traceability = { frIds, framework: 'jest' };
    return fn;
  };
}

/**
 * Creates a describe block for a specific FR.
 * 
 * @param frId - The FR ID
 * @param description - Description of the FR
 * @param fn - The describe block function
 * 
 * @example
 * ```typescript
 * describeFr('FR-EXAMPLE-001', 'User Authentication', () => {
 *   test('login works', () => { ... });
 *   test('logout works', () => { ... });
 * });
 * ```
 */
export function describeFr(frId: string, description: string, fn: () => void): void {
  if (!validateFrId(frId)) {
    throw new Error(`Invalid FR ID format: ${frId}. Expected: FR-XXXX-NNN`);
  }

  const describe = (globalThis as any).describe;
  if (describe) {
    describe(`${frId}: ${description}`, fn);
  } else {
    throw new Error('describe not found. Is Jest loaded?');
  }
}

/**
 * Collects all FR traces from the current test run.
 * Used by the ptrace CLI to generate coverage reports.
 */
export function collectTraces(): Record<string, string[]> {
  // This would be populated by a test environment setup
  return (globalThis as any).__traceabilityTraces || {};
}
