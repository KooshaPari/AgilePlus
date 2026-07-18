import React from 'react';
import { cn } from '../../lib/utils';

/**
 * Skeleton Component
 *
 * Loading placeholder with subtle pulse animation. Use to communicate
 * "data is on its way" without resorting to a full SplashScreen.
 *
 * Accessibility:
 *   - aria-busy="true" + role="status" so screen readers know the region is loading
 *   - aria-live="polite" announces updates to AT users
 *   - aria-label can be overridden for explicit descriptions
 *
 * Traces to: FR-UX-04 (loading experience), pillar L51 (Splash Screen)
 */

export interface SkeletonProps extends React.HTMLAttributes<HTMLDivElement> {
  /** Visual shape of the placeholder. */
  variant?: 'text' | 'circular' | 'rectangular';
  /** Number of lines to render (text variant only). */
  count?: number;
  /** Disable the pulse animation. */
  animate?: boolean;
  /** Custom label for screen readers. */
  ariaLabel?: string;
}

/**
 * Skeleton
 *
 * @example
 * <Skeleton variant="text" count={3} />
 * <Skeleton variant="circular" className="h-12 w-12" />
 * <Skeleton variant="rectangular" className="h-32 w-full" />
 */
export const Skeleton: React.FC<SkeletonProps> = ({
  variant = 'text',
  count = 1,
  animate = true,
  ariaLabel = 'Loading',
  className,
  ...rest
}) => {
  const items = Array.from({ length: Math.max(1, count) }, (_, i) => i);

  return (
    <div
      role="status"
      aria-busy="true"
      aria-live="polite"
      aria-label={ariaLabel}
      className={cn('flex flex-col gap-2', className)}
      {...rest}
    >
      {items.map((i) => (
        <span
          key={i}
          aria-hidden="true"
          className={cn(
            'block bg-slate-200 dark:bg-slate-800',
            variant === 'text' && 'h-3 w-full rounded',
            variant === 'circular' && 'h-10 w-10 rounded-full',
            variant === 'rectangular' && 'h-24 w-full rounded-md',
            animate && 'animate-pulse'
          )}
        />
      ))}
    </div>
  );
};

export default Skeleton;