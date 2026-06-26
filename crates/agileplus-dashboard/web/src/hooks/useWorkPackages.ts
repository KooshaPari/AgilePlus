import { useEffect } from 'react';
import { fetchDashboardWorkPackages } from '../lib/api/client';
import { useAgilePlusStore } from '../stores/agileplus';
import type { WorkPackage } from '../types';

// ============================================================================
// useWorkPackages Hook
// Fetch and manage work package data from API
// ============================================================================

interface UseWorkPackagesOptions {
  skip?: boolean;
}

function toWorkPackage(wp: {
  id: string;
  title?: string;
  status?: string;
  priority?: string;
  assignee?: string | null;
}): WorkPackage {
  return {
    id: String(wp.id),
    title: wp.title ?? '(untitled)',
    status: (wp.status ?? 'planned') as WorkPackage['status'],
    priority: (wp.priority ?? 'medium') as WorkPackage['priority'],
    assignee: wp.assignee ?? undefined,
  };
}

/**
 * Hook to fetch and manage work packages
 * Integrates with Zustand store and API
 *
 * @example
 * const { workPackages, loading, error } = useWorkPackages();
 */
export function useWorkPackages(options: UseWorkPackagesOptions = {}) {
  const { skip = false } = options;
  const { workPackages, setWorkPackages, setLoading } = useAgilePlusStore();

  useEffect(() => {
    if (skip) return;

    let cancelled = false;

    const load = async () => {
      setLoading(true);
      try {
        const data = await fetchDashboardWorkPackages();
        if (cancelled) return;
        setWorkPackages((data.work_packages ?? []).map(toWorkPackage));
      } catch (error) {
        if (!cancelled) {
          console.error('Failed to fetch work packages:', error);
        }
      } finally {
        if (!cancelled) setLoading(false);
      }
    };

    void load();
    return () => {
      cancelled = true;
    };
  }, [skip, setWorkPackages, setLoading]);

  return {
    workPackages,
    loading: useAgilePlusStore((state) => state.loading),
  };
}

export default useWorkPackages;
