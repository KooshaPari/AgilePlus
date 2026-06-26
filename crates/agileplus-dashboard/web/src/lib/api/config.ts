const DEFAULT_API_BASE = 'http://localhost:3000';

/** agileplus-api base URL (no trailing slash). */
export function getApiBase(): string {
  const raw = import.meta.env.VITE_API_BASE?.trim();
  const base = raw && raw.length > 0 ? raw : DEFAULT_API_BASE;
  return base.replace(/\/$/, '');
}

export const API_TIMEOUT_MS = Number(import.meta.env.VITE_API_TIMEOUT) || 30_000;
