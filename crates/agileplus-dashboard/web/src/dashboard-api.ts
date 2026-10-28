/**
 * Canonical dashboard data contract.
 *
 * Every route used here is mounted by `agileplus-api` under `/api/v1`:
 *   GET /api/v1/projects
 *   GET /api/v1/projects/{slug}/epics
 *   GET /api/v1/epics/{id}/stories
 *   GET /api/v1/features
 *   GET /api/v1/features/{slug}/work-packages
 *
 * The browser holds no backend credential: the private gateway injects the API
 * key. A failed or non-JSON response is surfaced as an error and is never
 * replaced by substituted data.
 */

const API_ROOT = '/api/v1';
const REQUEST_TIMEOUT_MS = 15000;

export class DashboardApiError extends Error {
  constructor(
    readonly status: number,
    message: string,
  ) {
    super(message);
    this.name = 'DashboardApiError';
  }
}

/** `GET /api/v1/projects` */
export interface Project {
  id: number;
  slug: string;
  name: string;
  description: string | null;
}

/** `GET /api/v1/projects/{slug}/epics` */
export interface Epic {
  id: number;
  project_id: number;
  title: string;
  description: string | null;
  status: string;
  owner_id: number | null;
  requirement_id: string | null;
}

/** `GET /api/v1/epics/{id}/stories` */
export interface Story {
  id: number;
  epic_id: number;
  project_id: number;
  title: string;
  description: string | null;
  status: string;
  points: number | null;
  assignee_id: number | null;
  requirement_id: string | null;
}

/** `GET /api/v1/features` */
export interface Feature {
  id: number;
  slug: string;
  name: string;
  state: string;
}

/** `GET /api/v1/features/{slug}/work-packages` */
export interface WorkPackage {
  id: number;
  feature_id: number;
  title: string;
  state: string;
  sequence: number;
  acceptance_criteria: string;
  pr_url: string | null;
}

export interface DashboardSnapshot {
  epics: Epic[];
  stories: Story[];
  workPackages: WorkPackage[];
}

/** Read a canonical route. Fails closed: non-JSON bodies are errors, not data. */
export async function apiGet<T>(path: string, signal?: AbortSignal): Promise<T> {
  const response = await fetch(`${API_ROOT}${path}`, {
    credentials: 'same-origin',
    cache: 'no-store',
    signal: signal ?? AbortSignal.timeout(REQUEST_TIMEOUT_MS),
    headers: { Accept: 'application/json' },
  });
  if (!response.ok) {
    const payload = (await response.json().catch(() => ({}))) as {
      message?: string;
      error?: string;
    };
    // The backend may report the failure under either key; pick the first
    // present one. Stated as statements rather than a nested ternary
    // (Sonar typescript:S3358).
    let reason = response.statusText;
    if (typeof payload.message === 'string') {
      reason = payload.message;
    } else if (typeof payload.error === 'string') {
      reason = payload.error;
    }
    throw new DashboardApiError(response.status, `HTTP ${response.status}: ${reason}`);
  }
  if (!response.headers.get('content-type')?.includes('application/json')) {
    throw new DashboardApiError(502, 'API returned non-JSON content. Check gateway routing.');
  }
  return (await response.json()) as T;
}

/** Every project epic, and every story of every epic, from the mounted routes. */
export async function loadEpicsAndStories(
  signal?: AbortSignal,
): Promise<{ epics: Epic[]; stories: Story[] }> {
  const projects = await apiGet<Project[]>('/projects', signal);
  const epics: Epic[] = [];
  const stories: Story[] = [];
  for (const project of projects) {
    // The awaits here are deliberately sequential, not parallel: a story is
    // only reachable through its epic, so the epic list has to be resolved
    // before its stories can be requested. Each project is independent, so
    // only the inner story fetch is serial (Sonar typescript:S9382).
    const projectEpics = await apiGet<Epic[]>(
      `/projects/${encodeURIComponent(project.slug)}/epics`,
      signal,
    );
    epics.push(...projectEpics);
    for (const epic of projectEpics) {
      const epicStories = await apiGet<Story[]>(`/epics/${epic.id}/stories`, signal);
      stories.push(...epicStories);
    }
  }
  return { epics, stories };
}

/** Every work package of every feature, from the mounted routes. */
export async function loadWorkPackages(signal?: AbortSignal): Promise<WorkPackage[]> {
  const features = await apiGet<Feature[]>('/features', signal);
  const workPackages: WorkPackage[] = [];
  for (const feature of features) {
    // Sequential for the same reason as loadEpicsAndStories: a work package
    // is only reachable through its feature (Sonar typescript:S9382).
    const featurePackages = await apiGet<WorkPackage[]>(
      `/features/${encodeURIComponent(feature.slug)}/work-packages`,
      signal,
    );
    workPackages.push(...featurePackages);
  }
  return workPackages;
}

/** One consistent snapshot of the canonical dashboard data. */
export async function loadDashboard(signal?: AbortSignal): Promise<DashboardSnapshot> {
  const [epicData, workPackages] = await Promise.all([
    loadEpicsAndStories(signal),
    loadWorkPackages(signal),
  ]);
  return { ...epicData, workPackages };
}

// ---------------------------------------------------------------------------
// Status vocabulary
//
// The canonical backend owns the status strings. The dashboard never invents
// one: it only groups the canonical value for display and counting.
// ---------------------------------------------------------------------------

export type StatusCategory =
  | 'planned'
  | 'in_progress'
  | 'review'
  | 'completed'
  | 'blocked'
  | 'cancelled'
  | 'unknown';

/** Canonical status -> display category, per the `agileplus-domain` enums. */
const STATUS_CATEGORY: Record<string, StatusCategory> = {
  // EpicStatus
  backlog: 'planned',
  active: 'in_progress',
  review: 'review',
  done: 'completed',
  cancelled: 'cancelled',
  // StoryStatus
  todo: 'planned',
  in_progress: 'in_progress',
  blocked: 'blocked',
  // WpState
  planned: 'planned',
  doing: 'in_progress',
};

export function statusCategory(status: string): StatusCategory {
  return STATUS_CATEGORY[status.trim().toLowerCase()] ?? 'unknown';
}

export type StatusVariant = 'success' | 'warning' | 'error' | 'info' | 'default';

const CATEGORY_VARIANT: Record<StatusCategory, StatusVariant> = {
  completed: 'success',
  in_progress: 'info',
  review: 'info',
  planned: 'warning',
  blocked: 'error',
  cancelled: 'default',
  unknown: 'default',
};

export function statusVariant(status: string): StatusVariant {
  return CATEGORY_VARIANT[statusCategory(status)];
}

/** Human-readable reason for an unavailable backend. */
export function describeBackendError(error: unknown): string {
  if (error instanceof DashboardApiError && error.status === 401) {
    return 'Authentication required. Sign in through the private operator gateway, then refresh.';
  }
  if (error instanceof Error && error.message) return error.message;
  return 'The AgilePlus API is unavailable.';
}
