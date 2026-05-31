/**
 * planify-shim — exposes a subset of Plane's REST API (the shape the Planify
 * frontend `apps/web` expects) backed by the AgilePlus Axum API.
 *
 * Listens on :8000 (Planify's VITE_API_BASE_URL default). Allows credentialed
 * CORS from the apps/web dev origin (:4400).
 *
 * Auth is a local-review STUB: a fixed CSRF token + a synthetic "me" user are
 * returned so apps/web treats the session as authenticated. The shim itself
 * authenticates to AgilePlus with X-API-Key (AGILEPLUS_API_KEY).
 *
 * READ endpoints implemented (enough to render a populated board):
 *   GET  /auth/get-csrf-token/
 *   GET  /api/instances/
 *   GET  /api/users/me/  (+ /settings/ /profile/ /workspaces/)
 *   GET  /api/workspaces/:slug/
 *   GET  /api/workspaces/:slug/members/
 *   GET  /api/workspaces/:slug/projects/  (+ /details/)
 *   GET  /api/workspaces/:slug/projects/:projectId/
 *   GET  /api/workspaces/:slug/projects/:projectId/project-members/me/
 *   GET  /api/workspaces/:slug/projects/:projectId/members/
 *   GET  /api/workspaces/:slug/projects/:projectId/states/
 *   GET  /api/workspaces/:slug/projects/:projectId/issue-labels/
 *   GET  /api/workspaces/:slug/projects/:projectId/issues/  (grouped TIssuesResponse)
 */
import cors from "cors";
import express from "express";
import { agileplus } from "./agileplus-client.js";
import {
  USER_ID,
  WORKSPACE_ID,
  WORKSPACE_SLUG,
  featureToProject,
  statesForProject,
  wpToIssue,
} from "./map.js";

const PORT = Number(process.env.PORT || 8000);
const WEB_ORIGIN = process.env.WEB_ORIGIN || "http://localhost:4400";
const CSRF = "planify-shim-csrf-token";

const app = express();
app.use(express.json());
app.use(
  cors({
    origin: [WEB_ORIGIN, "http://127.0.0.1:4400"],
    credentials: true,
    exposedHeaders: ["set-cookie"],
  }),
);

const log = (req, _res, next) => {
  console.log(`${req.method} ${req.originalUrl}`);
  next();
};
app.use(log);

const NOW = new Date().toISOString();

// ---- find an AgilePlus feature by the project UUID we synthesised ----
async function featuresIndex() {
  const features = await agileplus.listFeatures();
  const projects = features.map(featureToProject);
  const byId = new Map(projects.map((p) => [p.id, p]));
  return { features, projects, byId };
}

// ───────────────────────────── Auth / instance ─────────────────────────────

app.get("/auth/get-csrf-token/", (_req, res) => {
  res.cookie("csrftoken", CSRF, { sameSite: "lax" });
  res.json({ csrf_token: CSRF });
});

app.get("/api/instances/", (_req, res) => {
  res.json({
    instance: {
      id: "agileplus-instance",
      instance_name: "AgilePlus (via planify-shim)",
      is_setup_done: true,
      is_activated: true,
      is_signup_screen_visible: false,
      is_telemetry_enabled: false,
      product: "plane-ce",
      version: "shim-0.1.0",
      workspaces_exist: true,
    },
    config: { is_smtp_configured: false, github_app_name: "", magic_login: false, email_password_login: true },
  });
});

const ME = {
  id: USER_ID,
  username: "dogfood",
  email: "dogfood@agileplus.dev",
  first_name: "AgilePlus",
  last_name: "Dogfood",
  display_name: "AgilePlus Dogfood",
  avatar: "",
  is_active: true,
  is_onboarded: true,
  is_email_verified: true,
  is_tour_completed: true,
  onboarding_step: { profile_complete: true, workspace_create: true, workspace_invite: true, workspace_join: true },
  last_workspace_id: WORKSPACE_ID,
  theme: {},
  created_at: NOW,
  updated_at: NOW,
};

app.get("/api/users/me/", (_req, res) => res.json(ME));
app.get("/api/users/me/profile/", (_req, res) =>
  res.json({
    id: USER_ID,
    user: USER_ID,
    role: "Engineering",
    last_workspace_id: WORKSPACE_ID,
    theme: {},
    onboarding_step: ME.onboarding_step,
    is_onboarded: true,
    is_tour_completed: true,
    use_case: "Engineering",
  }),
);
app.get("/api/users/me/settings/", (_req, res) =>
  res.json({
    id: USER_ID,
    workspace: { last_workspace_id: WORKSPACE_ID, last_workspace_slug: WORKSPACE_SLUG, fallback_workspace_id: WORKSPACE_ID, fallback_workspace_slug: WORKSPACE_SLUG, invites: 0 },
  }),
);

const workspaceObj = () => ({
  id: WORKSPACE_ID,
  name: "AgilePlus",
  slug: WORKSPACE_SLUG,
  owner: USER_ID,
  logo: "",
  logo_url: null,
  total_members: 1,
  total_issues: 0,
  role: 20,
  organization_size: "1-10",
  created_at: NOW,
  updated_at: NOW,
});

app.get("/api/users/me/workspaces/", (_req, res) => res.json([workspaceObj()]));
app.get("/api/workspaces/:slug/", (_req, res) => res.json(workspaceObj()));

const memberMe = {
  id: USER_ID,
  member: ME,
  role: 20,
  is_active: true,
  workspace: WORKSPACE_ID,
  created_at: NOW,
  updated_at: NOW,
};
app.get("/api/workspaces/:slug/members/", (_req, res) => res.json([memberMe]));
app.get("/api/users/me/workspaces/:slug/", (_req, res) => res.json(workspaceObj()));

// ───────────────────────────── Projects ─────────────────────────────

async function projectsHandler(_req, res) {
  try {
    const { projects } = await featuresIndex();
    res.json(projects);
  } catch (e) {
    res.status(502).json({ error: String(e.message || e) });
  }
}
app.get("/api/workspaces/:slug/projects/", projectsHandler);
app.get("/api/workspaces/:slug/projects/details/", projectsHandler);

app.get("/api/workspaces/:slug/projects/:projectId/", async (req, res) => {
  try {
    const { byId } = await featuresIndex();
    const p = byId.get(req.params.projectId);
    if (!p) return res.status(404).json({ error: "project not found" });
    res.json(p);
  } catch (e) {
    res.status(502).json({ error: String(e.message || e) });
  }
});

const projectMemberMe = (projectId) => ({
  id: USER_ID,
  member: ME,
  role: 20,
  is_active: true,
  project: projectId,
  workspace: WORKSPACE_ID,
  created_at: NOW,
  updated_at: NOW,
});
app.get("/api/workspaces/:slug/projects/:projectId/project-members/me/", (req, res) =>
  res.json(projectMemberMe(req.params.projectId)),
);
app.get("/api/workspaces/:slug/projects/:projectId/members/", (req, res) =>
  res.json([projectMemberMe(req.params.projectId)]),
);

// States (board columns), labels.
app.get("/api/workspaces/:slug/projects/:projectId/states/", (req, res) =>
  res.json(statesForProject(req.params.projectId)),
);
app.get("/api/workspaces/:slug/states/", async (_req, res) => {
  try {
    const { projects } = await featuresIndex();
    res.json(projects.flatMap((p) => statesForProject(p.id)));
  } catch (e) {
    res.status(502).json({ error: String(e.message || e) });
  }
});
app.get("/api/workspaces/:slug/projects/:projectId/issue-labels/", (_req, res) => res.json([]));
app.get("/api/workspaces/:slug/labels/", (_req, res) => res.json([]));

// ───────────────────────────── Issues ─────────────────────────────
//
// Plane's TIssuesResponse (ungrouped) is:
//   { grouped_by, sub_grouped_by, total_count, count, total_pages, extra_stats,
//     results: TIssue[] }

async function issuesHandler(req, res) {
  try {
    const { byId } = await featuresIndex();
    const project = byId.get(req.params.projectId);
    if (!project) return res.status(404).json({ error: "project not found" });
    const wps = await agileplus.listWorkPackages(project._agileplus_slug);
    const results = wps.map((wp) => wpToIssue(wp, project));
    res.json({
      grouped_by: null,
      sub_grouped_by: null,
      total_count: results.length,
      count: results.length,
      total_pages: 1,
      next_page_results: false,
      prev_page_results: false,
      extra_stats: null,
      results,
    });
  } catch (e) {
    res.status(502).json({ error: String(e.message || e) });
  }
}
app.get("/api/workspaces/:slug/projects/:projectId/issues/", issuesHandler);
app.get("/api/workspaces/:slug/projects/:projectId/v2/issues/", issuesHandler);
app.get("/api/workspaces/:slug/projects/:projectId/issues-detail/", issuesHandler);

// Health for quick sanity check.
app.get("/shim/health", async (_req, res) => {
  try {
    const f = await agileplus.listFeatures();
    res.json({ ok: true, agileplus: agileplus.base, features: f.length });
  } catch (e) {
    res.status(502).json({ ok: false, error: String(e.message || e) });
  }
});

// Default: empty-but-valid 200 so unimplemented reads don't 401-redirect-loop
// the frontend. (Writes are out of scope for read-only candidate review.)
app.use((req, res) => {
  console.warn(`UNMAPPED ${req.method} ${req.originalUrl} -> []`);
  res.json([]);
});

app.listen(PORT, () => {
  console.log(`planify-shim listening on http://localhost:${PORT}`);
  console.log(`  -> AgilePlus backend: ${agileplus.base}`);
  console.log(`  -> CORS origin:       ${WEB_ORIGIN}`);
});
