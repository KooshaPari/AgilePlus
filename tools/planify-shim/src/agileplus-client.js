/**
 * Thin client for the AgilePlus Axum REST API.
 *
 * AgilePlus exposes (auth: `X-API-Key` or `Authorization: Bearer`):
 *   GET /api/v1/features                       -> [FeatureResponse]
 *   GET /api/v1/features/:slug                  -> FeatureResponse
 *   GET /api/v1/features/:slug/work-packages    -> [WorkPackageResponse]
 *
 * FeatureResponse:     { id, slug, name, state, target_branch, created_at, updated_at }
 * WorkPackageResponse: { id, feature_id, title, state, sequence, acceptance_criteria,
 *                        pr_url, created_at, updated_at }
 */

const BASE = process.env.AGILEPLUS_API_URL || "http://localhost:4000";
const API_KEY = process.env.AGILEPLUS_API_KEY;
if (!API_KEY) throw new Error("AGILEPLUS_API_KEY must be explicitly provided (local review only)");

async function call(path) {
  const res = await fetch(`${BASE}${path}`, {
    headers: { "X-API-Key": API_KEY, Accept: "application/json" },
    signal: AbortSignal.timeout(5000),
  });
  if (!res.ok) {
    const body = await res.text().catch(() => "");
    throw new Error(`AgilePlus ${path} -> ${res.status} ${body}`);
  }
  return res.json();
}

export const agileplus = {
  base: BASE,
  listFeatures: () => call("/api/v1/features"),
  getFeature: (slug) => call(`/api/v1/features/${encodeURIComponent(slug)}`),
  listWorkPackages: (slug) =>
    call(`/api/v1/features/${encodeURIComponent(slug)}/work-packages`),
};
