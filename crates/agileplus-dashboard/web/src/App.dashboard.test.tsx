import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { App } from './App';

/**
 * Rows the dashboard used to fabricate whenever the API could not be read.
 * They must never be rendered again, in either the success or the failure
 * branch: a fabricated row that survives only on the error path is still a
 * fabricated row.
 */
const FABRICATED_TITLES = [
  'Core domain entities',
  'SQLite persistence layer',
  'Axum REST API',
  'Native dashboard frontend',
  'GitHub sync adapter',
  'Plane.so integration',
  'Define Epic/Story/WorkPackage value objects',
  'Evidence Gallery view (PHASE2)',
];

const json = (data: unknown, status = 200) =>
  new Response(JSON.stringify(data), {
    status,
    headers: { 'Content-Type': 'application/json' },
  });

const project = { id: 1, slug: 'core', name: 'Core', description: null };
const epic = {
  id: 7,
  project_id: 1,
  title: 'Backend epic',
  description: null,
  status: 'active',
  owner_id: null,
  requirement_id: 'FR-REAL-001',
};
const story = {
  id: 11,
  epic_id: 7,
  project_id: 1,
  title: 'Backend story',
  description: null,
  status: 'todo',
  points: null,
  assignee_id: null,
  requirement_id: null,
};
const feature = { id: 1, slug: 'atomic', name: 'Atomic', state: 'validated' };
const workPackage = {
  id: 5,
  feature_id: 1,
  title: 'Backend package',
  state: 'doing',
  sequence: 1,
  acceptance_criteria: 'Real criterion',
  pr_url: null,
};

let fetchMock: ReturnType<typeof vi.fn>;

beforeEach(() => {
  localStorage.clear();
  localStorage.setItem('onboarding_complete', 'true');
  fetchMock = vi.fn(async (url: string) => {
    if (url === '/api/v1/projects') return json([project]);
    if (url === '/api/v1/projects/core/epics') return json([epic]);
    if (url === '/api/v1/epics/7/stories') return json([story]);
    if (url === '/api/v1/features') return json([feature]);
    if (url === '/api/v1/features/atomic/work-packages') return json([workPackage]);
    throw new Error(`Unexpected request ${url}`);
  });
  vi.stubGlobal('fetch', fetchMock);
});

afterEach(() => vi.unstubAllGlobals());

describe('dashboard data contract', () => {
  it('renders backend rows from the canonical routes and no fabricated row', async () => {
    render(<App />);

    expect(await screen.findByText('Backend epic')).toBeInTheDocument();
    expect(screen.getByText('FR-REAL-001')).toBeInTheDocument();

    await userEvent.setup().click(screen.getByRole('button', { name: 'Stories' }));
    expect(await screen.findByText('Backend story')).toBeInTheDocument();

    for (const title of FABRICATED_TITLES) {
      expect(screen.queryByText(title)).not.toBeInTheDocument();
    }

    // Only mounted canonical routes, same-origin, and no browser-held key.
    expect(fetchMock).toHaveBeenCalled();
    for (const [url, init] of fetchMock.mock.calls) {
      expect(url).toMatch(/^\/api\/v1\//);
      expect(init.credentials).toBe('same-origin');
      expect(JSON.stringify(init.headers)).not.toMatch(/authorization|x-api-key/i);
    }
  });

  it('reports an unavailable backend instead of rendering rows', async () => {
    fetchMock.mockRejectedValue(new TypeError('Failed to fetch'));
    render(<App />);

    const alert = await screen.findByRole('alert');
    expect(alert).toHaveTextContent('AgilePlus backend unavailable');
    expect(screen.queryByText('Backend epic')).not.toBeInTheDocument();
    expect(screen.queryByText('No epics in this backend.')).not.toBeInTheDocument();
    for (const title of FABRICATED_TITLES) {
      expect(screen.queryByText(title)).not.toBeInTheDocument();
    }
  });

  it('distinguishes an empty backend from an unavailable one', async () => {
    fetchMock.mockImplementation(async (url: string) => {
      if (url === '/api/v1/projects') return json([]);
      if (url === '/api/v1/features') return json([]);
      throw new Error(`Unexpected request ${url}`);
    });
    render(<App />);

    expect(await screen.findByText('No epics in this backend.')).toBeInTheDocument();
    expect(screen.queryByRole('alert')).not.toBeInTheDocument();
  });
});
