/** Same-origin gateway contract. No browser-held backend credential. */
export interface Feature { id: number; slug: string; name: string; state: string; target_branch: string }
export interface WorkPackage { id: number; title: string; state: string; sequence: number; acceptance_criteria: string }
export interface Governance { version: number }
export interface Receipt {
  request_id: string; feature_id: number; actor: string; governance_version: number;
  committed_at: string; event_id: number;
  accepted_candidates: { wp_id: number; candidate_ref: string; assignment_id: string; attempt_id: string; evaluation_id: string }[];
}
export interface AcceptanceRequest { request_id: string; expected_governance_version: number }
export interface Outcome { receipt: Receipt; replayed: boolean }

export class ApiError extends Error {
  constructor(public readonly status: number, message: string) { super(message); }
}

export async function api<T>(path: string, init: RequestInit = {}): Promise<T> {
  const response = await fetch(`/api/v1${path}`, {
    ...init, credentials: 'same-origin', cache: 'no-store',
    signal: init.signal ?? AbortSignal.timeout(15000),
    headers: { Accept: 'application/json', ...(init.body ? { 'Content-Type': 'application/json' } : {}) },
  });
  if (!response.ok) {
    const payload = await response.json().catch(() => ({})) as { message?: string; error?: string };
    const reason = typeof payload.message === 'string' ? payload.message :
      typeof payload.error === 'string' ? payload.error : response.statusText;
    throw new ApiError(response.status, `HTTP ${response.status}: ${reason}`);
  }
  // A static frontend fallback must never masquerade as a successful API call.
  if (!response.headers.get('content-type')?.includes('application/json')) {
    throw new ApiError(502, 'API returned non-JSON content. Check gateway routing.');
  }
  return await response.json() as T;
}

export const featurePath = (slug: string) => `/features/${encodeURIComponent(slug)}`;
export const pendingKey = (featureId: number) => `agileplus.acceptance.v1.${featureId}`;

export function readPending(featureId: number): AcceptanceRequest | null {
  const raw = localStorage.getItem(pendingKey(featureId));
  if (!raw) return null;
  const pending = JSON.parse(raw) as AcceptanceRequest;
  if (typeof pending.request_id !== 'string' || !pending.request_id.startsWith('browser:') ||
      pending.request_id.length > 128 || !Number.isInteger(pending.expected_governance_version)) {
    throw new Error('The saved acceptance request is invalid. Clear it before starting a new request.');
  }
  return pending;
}

export function prepareRequest(featureId: number, version: number): AcceptanceRequest {
  const existing = readPending(featureId);
  if (existing) return existing;
  const request = { request_id: `browser:${crypto.randomUUID()}`, expected_governance_version: version };
  // Persist BEFORE sending. Storage failure must not produce an unrepeatable mutation.
  localStorage.setItem(pendingKey(featureId), JSON.stringify(request));
  return request;
}

export function describeError(error: unknown): string {
  if (error instanceof ApiError && error.status === 401) return 'Authentication required. Sign in through the private operator gateway, then refresh.';
  if (error instanceof Error) return error.message;
  return 'The backend is unavailable. Refresh to retry.';
}
