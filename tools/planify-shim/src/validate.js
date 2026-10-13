/** Strict boundary checks: malformed upstream data must never render as an empty/successful board. */
const FEATURE_STATES = new Set(["created", "specified", "researched", "planned", "implementing", "validated", "shipped", "retrospected"]);
const WP_STATES = new Set(["planned", "doing", "review", "done", "blocked"]);

function validId(value) {
  return Number.isSafeInteger(value) && value > 0;
}
function requiredString(value) {
  return typeof value === "string" && value.trim().length > 0;
}
export function assertFeatures(features) {
  if (!Array.isArray(features)) throw new Error("Invalid AgilePlus features response: expected array");
  const seenIds = new Set(), seenSlugs = new Set();
  for (const feature of features) {
    if (!feature || !validId(feature.id) || !requiredString(feature.slug) ||
        !requiredString(feature.name) || !FEATURE_STATES.has(String(feature.state).toLowerCase()) ||
        seenIds.has(feature.id) || seenSlugs.has(feature.slug)) {
      throw new Error("Invalid or duplicate AgilePlus feature");
    }
    seenIds.add(feature.id);
    seenSlugs.add(feature.slug);
  }
  return features;
}
export function assertWorkPackages(wps, featureId) {
  if (!Array.isArray(wps) || !validId(featureId)) throw new Error("Invalid AgilePlus work-package response");
  const ids = new Set();
  for (const wp of wps) {
    if (!wp || !validId(wp.id) || wp.feature_id !== featureId ||
        !requiredString(wp.title) || !WP_STATES.has(String(wp.state).toLowerCase()) ||
        !Number.isSafeInteger(wp.sequence) || typeof wp.acceptance_criteria !== "string" ||
        ids.has(wp.id)) {
      throw new Error("Invalid, mismatched, or duplicate AgilePlus work package");
    }
    ids.add(wp.id);
  }
  return wps;
}
