import React from 'react';
import { cn } from '../../lib/utils';

/**
 * EmptyState Component
 *
 * Used when a list/grid has no results to show — communicates the empty
 * state clearly while pointing the user at the next action.
 *
 * Accessibility:
 *   - role="status" announces the empty state to screen readers
 *   - The action button must be a real button/link (not a div)
 *
 * Traces to: FR-UX-05 (empty states), pillar L73 (Empty States)
 */

export interface EmptyStateProps {
  /** Optional illustration (emoji, SVG, or icon). */
  illustration?: React.ReactNode;
  /** Heading text. */
  title: string;
  /** Helper text below the title. */
  description?: string;
  /** Primary action (e.g. "Create your first story"). */
  action?: React.ReactNode;
  /** Secondary action (e.g. "Learn more"). */
  secondaryAction?: React.ReactNode;
  /** Center the content horizontally. Default: true. */
  centered?: boolean;
  /** Additional classes for the outer wrapper. */
  className?: string;
}

/**
 * EmptyState
 *
 * @example
 * <EmptyState
 *   title="No stories yet"
 *   description="Create your first story to start tracking work."
 *   action={<Button onClick={...}>Create story</Button>}
 * />
 */
export const EmptyState: React.FC<EmptyStateProps> = ({
  illustration,
  title,
  description,
  action,
  secondaryAction,
  centered = true,
  className,
}) => (
  <div
    role="status"
    className={cn(
      'flex flex-col gap-3 rounded-lg border border-dashed border-slate-300 bg-slate-50 p-8 dark:border-slate-700 dark:bg-slate-900',
      centered && 'items-center justify-center text-center',
      className
    )}
  >
    {illustration && (
      <div
        aria-hidden="true"
        className="text-4xl text-slate-400 dark:text-slate-600"
      >
        {illustration}
      </div>
    )}
    <h3 className="text-base font-semibold text-slate-900 dark:text-slate-100">
      {title}
    </h3>
    {description && (
      <p className="max-w-sm text-sm text-slate-600 dark:text-slate-400">
        {description}
      </p>
    )}
    {action && <div className="mt-2">{action}</div>}
    {secondaryAction && (
      <div className="mt-1 text-xs text-slate-500">{secondaryAction}</div>
    )}
  </div>
);

export default EmptyState;