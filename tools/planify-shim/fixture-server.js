/**
 * Minimal AgilePlus read API for Planify shim smoke tests.
 * Serves the three endpoints planify-shim calls without the full Rust API.
 */
import http from "node:http";

const PORT = Number(process.env.PORT || 4000);
const API_KEY = process.env.AGILEPLUS_API_KEY || "dev-api-key";

const features = [
  {
    id: 1,
    slug: "harness-duel",
    name: "Harness Duel",
    state: "Implementing",
    target_branch: "main",
    created_at: "2026-07-22T00:00:00Z",
    updated_at: "2026-07-22T00:00:00Z",
  },
  {
    id: 2,
    slug: "forge-gpu",
    name: "Forge GPU Lane",
    state: "Validated",
    target_branch: "main",
    created_at: "2026-07-22T00:00:00Z",
    updated_at: "2026-07-22T00:00:00Z",
  },
];

const workPackages = {
  "harness-duel": [
    {
      id: 101,
      feature_id: 1,
      title: "heliosBench compare smoke",
      state: "Doing",
      sequence: 1,
      acceptance_criteria: "palindrome task runs",
      pr_url: null,
      created_at: "2026-07-22T00:00:00Z",
      updated_at: "2026-07-22T00:00:00Z",
    },
    {
      id: 102,
      feature_id: 1,
      title: "Benchora baseline gate",
      state: "Done",
      sequence: 2,
      acceptance_criteria: "harness-duel-v0 stored",
      pr_url: null,
      created_at: "2026-07-22T00:00:00Z",
      updated_at: "2026-07-22T00:00:00Z",
    },
  ],
  "forge-gpu": [
    {
      id: 201,
      feature_id: 2,
      title: "Reland GpuLane crate",
      state: "Done",
      sequence: 1,
      acceptance_criteria: "forge PR #95 merged",
      pr_url: "https://github.com/KooshaPari/forgecode/pull/95",
      created_at: "2026-07-22T00:00:00Z",
      updated_at: "2026-07-22T00:00:00Z",
    },
  ],
};

function unauthorized(res) {
  res.writeHead(401, { "Content-Type": "application/json" });
  res.end(JSON.stringify({ error: "unauthorized" }));
}

function json(res, status, body) {
  res.writeHead(status, { "Content-Type": "application/json" });
  res.end(JSON.stringify(body));
}

const server = http.createServer((req, res) => {
  const key = req.headers["x-api-key"];
  if (key !== API_KEY) {
    return unauthorized(res);
  }

  const url = new URL(req.url || "/", `http://127.0.0.1:${PORT}`);
  const path = url.pathname;

  if (req.method === "GET" && path === "/health") {
    return json(res, 200, { status: "ok" });
  }

  if (req.method === "GET" && path === "/api/v1/features") {
    return json(res, 200, features);
  }

  const featureMatch = path.match(/^\/api\/v1\/features\/([^/]+)$/);
  if (req.method === "GET" && featureMatch) {
    const slug = decodeURIComponent(featureMatch[1]);
    const feature = features.find((f) => f.slug === slug);
    if (!feature) return json(res, 404, { error: "not found" });
    return json(res, 200, feature);
  }

  const wpMatch = path.match(/^\/api\/v1\/features\/([^/]+)\/work-packages$/);
  if (req.method === "GET" && wpMatch) {
    const slug = decodeURIComponent(wpMatch[1]);
    return json(res, 200, workPackages[slug] || []);
  }

  json(res, 404, { error: "not found", path });
});

server.listen(PORT, "127.0.0.1", () => {
  console.log(`planify fixture API on http://127.0.0.1:${PORT}`);
});
