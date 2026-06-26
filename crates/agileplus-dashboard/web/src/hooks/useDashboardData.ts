import { useEffect, useState } from 'react';
import { fetchDashboardEpicsStories } from '../lib/api/client';
import type { ApiEpic, ApiStory } from '../types/api';

interface UseDashboardDataResult {
  epics: ApiEpic[];
  stories: ApiStory[];
  loading: boolean;
  error: string | null;
}

/**
 * Fetches main dashboard epics/stories from agileplus-api.
 */
export function useDashboardData(): UseDashboardDataResult {
  const [epics, setEpics] = useState<ApiEpic[]>([]);
  const [stories, setStories] = useState<ApiStory[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    const load = async () => {
      setLoading(true);
      setError(null);
      try {
        const data = await fetchDashboardEpicsStories();
        if (cancelled) return;
        if (data.error) setError(data.error);
        setEpics(data.epics ?? []);
        setStories(data.stories ?? []);
      } catch (err) {
        if (cancelled) return;
        const message =
          err instanceof Error ? err.message : 'Failed to load dashboard data';
        setError(
          `API unavailable: ${message}. Start agileplus-api (default :3000).`,
        );
        setEpics([]);
        setStories([]);
      } finally {
        if (!cancelled) setLoading(false);
      }
    };

    void load();
    return () => {
      cancelled = true;
    };
  }, []);

  return { epics, stories, loading, error };
}

export default useDashboardData;
