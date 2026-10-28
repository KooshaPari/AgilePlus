import { useEffect, useState } from 'react';
import { useAgilePlusStore } from '../stores/agileplus';
import { describeBackendError, loadWorkPackages, statusCategory } from '../dashboard-api';
import type { WorkPackage } from '../types';

// ============================================================================
// useWorkPackages Hook
// Fetch and manage work package data from the canonical API
// ============================================================================

interface UseWorkPackagesOptions {
  skip?: boolean;
}

/**
 * Hook to fetch and manage work packages.
 *
 * The canonical API exposes work packages per feature
 * (`GET /api/v1/features/{slug}/work-packages`); there is no flat list route.
 * Only fields the backend actually returns are mapped - a failed read is
 * reported as an error and never replaced by substituted rows.
 *
 * @example
 * const { workPackages, loading, error } = useWorkPackages();
 */
export function useWorkPackages(options: UseWorkPackagesOptions = {}) {
  const { skip = false } = options;
  const { workPackages, setWorkPackages, setLoading } = useAgilePlusStore();
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (skip) return;
    const controller = new AbortController();

    const fetchWorkPackages = async () => {
      setLoading(true);
      try {
        const canonical = await loadWorkPackages(controller.signal);
        setWorkPackages(
          canonical.map(
            (wp): WorkPackage => ({
              id: String(wp.id),
              title: wp.title,
              status: statusCategory(wp.state),
            }),
          ),
        );
        setError(null);
      } catch (error_) {
        if (controller.signal.aborted) return;
        setWorkPackages([]);
        setError(describeBackendError(error_));
        console.error('Failed to fetch work packages:', error_);
      } finally {
        setLoading(false);
      }
    };

    fetchWorkPackages();
    return () => controller.abort();
  }, [skip, setWorkPackages, setLoading]);

  return {
    workPackages,
    loading: useAgilePlusStore((state) => state.loading),
    error,
  };
}

export default useWorkPackages;
