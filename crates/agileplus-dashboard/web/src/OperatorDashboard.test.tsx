import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { OperatorDashboard } from './OperatorDashboard';
import { pendingKey } from './operator-api';

const feature = { id: 1, slug: 'atomic', name: 'Atomic acceptance', state: 'implementing', target_branch: 'main' };
const receipt = { request_id: 'browser:original', feature_id: 1, actor: 'http:api-key', governance_version: 1,
  committed_at: '2026-10-07T00:00:00Z', event_id: 1,
  accepted_candidates: [{ wp_id: 1, candidate_ref: 'git:exact', evaluation_id: 'evaluation:1', assignment_id: 'assignment:1', attempt_id: 'attempt:1' }] };
const json = (data: unknown, status = 200) => new Response(JSON.stringify(data), { status, headers: { 'Content-Type': 'application/json' } });
let post: (init: RequestInit) => Promise<Response>;
let lookup: () => Response;
let fetchMock: ReturnType<typeof vi.fn>;

beforeEach(() => {
  localStorage.clear();
  lookup = () => json({ error: 'acceptance_receipt_not_found' }, 404);
  post = async (init) => json({ replayed: false, receipt: { ...receipt, request_id: JSON.parse(String(init.body)).request_id } });
  fetchMock = vi.fn(async (url: string, init: RequestInit) => {
    if (url === '/api/v1/features') return json([feature]);
    if (url === '/api/v1/features/atomic') return json(feature);
    if (url.endsWith('/work-packages')) return json([{ id: 1, title: 'Real package', state: 'review', sequence: 1, acceptance_criteria: 'Frozen criterion' }]);
    if (url.endsWith('/governance')) return json({ version: 1 });
    if (url.endsWith('/acceptance-receipt')) return lookup();
    if (url.endsWith('/accept')) return post(init);
    throw new Error(`Unexpected request ${url}`);
  });
  vi.stubGlobal('fetch', fetchMock);
});
afterEach(() => vi.unstubAllGlobals());

describe('operator dashboard', () => {
  it('reads canonical feature/work-package routes without credentials or seed data', async () => {
    render(<OperatorDashboard />);
    expect(await screen.findByText(/WP01 · Real package/)).toBeInTheDocument();
    for (const [url, init] of fetchMock.mock.calls) {
      expect(url).toMatch(/^\/api\/v1\/features/);
      expect(init.credentials).toBe('same-origin');
      expect(JSON.stringify(init.headers)).not.toMatch(/authorization|x-api-key/i);
    }
  });

  it('shows authentication failure instead of synthetic success', async () => {
    fetchMock.mockResolvedValue(json({}, 401));
    render(<OperatorDashboard />);
    expect(await screen.findByRole('alert')).toHaveTextContent('Authentication required');
    expect(screen.queryByRole('button', { name: 'Accept feature' })).not.toBeInTheDocument();
  });

  it('shows an honest empty backend', async () => {
    fetchMock.mockResolvedValue(json([]));
    render(<OperatorDashboard />);
    expect(await screen.findByText('No features in this backend.')).toBeInTheDocument();
  });

  it('preserves the same request across a lost response and remount', async () => {
    post = async () => { throw new TypeError('Network response lost'); };
    const first = render(<OperatorDashboard />);
    const user = userEvent.setup();
    await user.click(await screen.findByRole('button', { name: 'Accept feature' }));
    await screen.findByText('Network response lost');
    const saved = localStorage.getItem(pendingKey(1));
    expect(saved).not.toBeNull();
    first.unmount();
    render(<OperatorDashboard />);
    await user.click(await screen.findByRole('button', { name: 'Retry acceptance' }));
    await waitFor(() => expect(fetchMock.mock.calls.filter(([url]) => url.endsWith('/accept'))).toHaveLength(2));
    const requests = fetchMock.mock.calls.filter(([url]) => url.endsWith('/accept'));
    expect(requests[0][1].body).toBe(requests[1][1].body);
    expect(localStorage.getItem(pendingKey(1))).toBe(saved);
  });

  it('recovers a server receipt after refresh and clears only its matching pending request', async () => {
    localStorage.setItem(pendingKey(1), JSON.stringify({ request_id: receipt.request_id, expected_governance_version: 1 }));
    lookup = () => json(receipt);
    render(<OperatorDashboard />);
    expect(await screen.findByText(receipt.request_id)).toBeInTheDocument();
    expect(screen.getByText(/git:exact/)).toBeInTheDocument();
    expect(localStorage.getItem(pendingKey(1))).toBeNull();
    expect(fetchMock.mock.calls.some(([url]) => url.endsWith('/accept'))).toBe(false);
  });

  it.each([409, 422, 500])('shows HTTP %i without inventing a receipt', async (status) => {
    post = async () => json({ error: 'acceptance_rejected', message: 'Canonical rejection' }, status);
    render(<OperatorDashboard />);
    await userEvent.setup().click(await screen.findByRole('button', { name: 'Accept feature' }));
    expect(await screen.findByRole('alert')).toHaveTextContent(`HTTP ${status}: Canonical rejection`);
    expect(screen.getByText('No committed acceptance receipt.')).toBeInTheDocument();
    expect(localStorage.getItem(pendingKey(1))).not.toBeNull();
    expect(screen.queryByRole('button', { name: 'Clear rejected request' }) !== null).toBe(status !== 500);
  });

  it('fails closed when the response is a frontend HTML fallback', async () => {
    fetchMock.mockResolvedValue(new Response('<html>frontend</html>', { headers: { 'Content-Type': 'text/html' } }));
    render(<OperatorDashboard />);
    expect(await screen.findByRole('alert')).toHaveTextContent('non-JSON content');
  });
});
