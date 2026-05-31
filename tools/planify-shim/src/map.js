/**
 * Maps AgilePlus domain objects onto Plane REST shapes.
 *
 *   AgilePlus root          -> single Plane workspace  (slug "agileplus")
 *   AgilePlus Feature/Epic  -> Plane project
 *   AgilePlus WorkPackage   -> Plane issue
 *   Feature/WP state        -> Plane state (board column), grouped by Plane group
 *
 * Plane requires UUIDs for ids; we derive stable v5-ish UUIDs from a namespace
 * + a string key so the same Feature/WP always maps to the same id across calls.
 */
import { createHash } from "node:crypto";

export const WORKSPACE_SLUG = "agileplus";
export const WORKSPACE_ID = uuidFrom("workspace", "agileplus");
export const USER_ID = uuidFrom("user", "dogfood@agileplus.dev");

/** Deterministic UUID (name-based, formatted as a v4-shaped string) from any key. */
export function uuidFrom(ns, key) {
  const h = createHash("sha256").update(`${ns}:${key}`).digest("hex");
  return [
    h.slice(0, 8),
    h.slice(8, 12),
    `4${h.slice(13, 16)}`,
    `8${h.slice(17, 20)}`,
    h.slice(20, 32),
  ].join("-");
}

const NOW = new Date().toISOString();

/**
 * AgilePlus state name -> Plane state group.
 * Plane groups: backlog | unstarted | started | completed | cancelled
 */
const STATE_GROUP = {
  created: "backlog",
  specified: "backlog",
  researched: "unstarted",
  planned: "unstarted",
  implementing: "started",
  validated: "started",
  shipped: "completed",
  retrospected: "completed",
  // work-package states (best-effort; unknown -> unstarted)
  todo: "unstarted",
  doing: "started",
  done: "completed",
  blocked: "started",
};

const GROUP_COLOR = {
  backlog: "#a3a3a3",
  unstarted: "#3f76ff",
  started: "#f59e0b",
  completed: "#16a34a",
  cancelled: "#ef4444",
};

export function stateGroup(name) {
  return STATE_GROUP[String(name || "").toLowerCase()] || "unstarted";
}

/** Build the canonical, fixed set of board columns (states) for a project. */
export function statesForProject(projectId) {
  const order = ["created", "planned", "implementing", "validated", "shipped"];
  return order.map((name, i) => {
    const group = stateGroup(name);
    return {
      id: uuidFrom("state", `${projectId}:${name}`),
      name: name.charAt(0).toUpperCase() + name.slice(1),
      color: GROUP_COLOR[group],
      group,
      default: i === 0,
      sequence: (i + 1) * 1000,
      project: projectId,
      workspace: WORKSPACE_ID,
      description: "",
      created_at: NOW,
      updated_at: NOW,
    };
  });
}

/** AgilePlus Feature -> Plane project. */
export function featureToProject(f) {
  const id = uuidFrom("project", f.slug);
  const identifier = (f.slug || `F${f.id}`)
    .replace(/[^a-zA-Z0-9]/g, "")
    .slice(0, 5)
    .toUpperCase() || `F${f.id}`;
  return {
    id,
    name: f.name || f.slug,
    identifier,
    description: `AgilePlus feature '${f.slug}' (state: ${f.state}, branch: ${f.target_branch})`,
    network: 2,
    workspace: WORKSPACE_ID,
    cover_image: null,
    icon_prop: null,
    emoji: null,
    logo_props: {},
    archived_at: null,
    is_member: true,
    member_role: 20,
    total_members: 1,
    total_cycles: 0,
    total_modules: 0,
    project_lead: USER_ID,
    default_assignee: USER_ID,
    cycle_view: true,
    module_view: true,
    issue_views_view: true,
    page_view: true,
    inbox_view: false,
    guest_view_all_features: false,
    created_at: f.created_at || NOW,
    updated_at: f.updated_at || NOW,
    created_by: USER_ID,
    updated_by: USER_ID,
    sort_order: f.id ?? 0,
    estimate: null,
    _agileplus_slug: f.slug,
  };
}

/** AgilePlus WorkPackage -> Plane issue. */
export function wpToIssue(wp, project) {
  const id = uuidFrom("issue", `${project.id}:${wp.id}`);
  const group = stateGroup(wp.state);
  const stateId = uuidFrom("state", `${project.id}:${nearestColumn(wp.state)}`);
  return {
    id,
    name: wp.title || `WP-${wp.id}`,
    description_html: `<p>${escapeHtml(wp.acceptance_criteria || "")}</p>`,
    description_stripped: wp.acceptance_criteria || "",
    priority: "none",
    start_date: null,
    target_date: null,
    sequence_id: wp.sequence ?? wp.id ?? 0,
    sort_order: (wp.sequence ?? wp.id ?? 0) * 1000,
    state_id: stateId,
    state__group: group,
    project_id: project.id,
    workspace_id: WORKSPACE_ID,
    parent_id: null,
    cycle_id: null,
    module_ids: [],
    label_ids: [],
    assignee_ids: [],
    estimate_point: null,
    sub_issues_count: 0,
    attachment_count: 0,
    link_count: wp.pr_url ? 1 : 0,
    is_draft: false,
    archived_at: null,
    completed_at: group === "completed" ? wp.updated_at || NOW : null,
    created_at: wp.created_at || NOW,
    updated_at: wp.updated_at || NOW,
    created_by: USER_ID,
    updated_by: USER_ID,
  };
}

// Snap any AgilePlus state onto one of the 5 fixed board columns.
function nearestColumn(name) {
  const g = stateGroup(name);
  return (
    { backlog: "created", unstarted: "planned", started: "implementing", completed: "shipped" }[g] ||
    "planned"
  );
}

function escapeHtml(s) {
  return String(s)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}
