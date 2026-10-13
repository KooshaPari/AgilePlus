import assert from "node:assert/strict";
import { test } from "node:test";
import { spawn } from "node:child_process";
import { once } from "node:events";
import { createServer } from "node:net";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";
import { assertFeatures, assertWorkPackages } from "../src/validate.js";
import { featureToProject, wpToIssue, stateGroup, uuidFrom } from "../src/map.js";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const feature = { id: 1, slug: "example-feature", name: "Example", state: "Implementing" };
const wp = { id: 101, feature_id: 1, title: "Work package", state: "Review", sequence: 1, acceptance_criteria: "tests pass" };

test("stable Plane-compatible IDs and explicit AgilePlus state mapping", () => {
  const project = featureToProject(feature);
  assert.equal(project.id, featureToProject({ ...feature }).id);
  assert.match(uuidFrom("issue", "one"), /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/);
  assert.equal(stateGroup("planned"), "unstarted");
  assert.equal(stateGroup("Doing"), "started");
  assert.equal(stateGroup("Review"), "started");
  assert.equal(stateGroup("Done"), "completed");
  assert.equal(stateGroup("Validated"), "started");
  assert.equal(stateGroup("Shipped"), "completed");
  assert.throws(() => stateGroup("unrecognized"), /Unrecognized AgilePlus state/);
  const issue = wpToIssue({ ...wp, acceptance_criteria: "<script>x</script>" }, project);
  assert.equal(issue.state__group, "started");
  assert.equal(issue.completed_at, null);
  assert.ok(!issue.description_html.includes("<script>"));
});

test("invalid feature and work-package data fails closed", () => {
  assert.deepEqual(assertFeatures([]), []);
  assert.deepEqual(assertWorkPackages([], 1), []);
  assert.equal(assertFeatures([feature])[0].slug, feature.slug);
  assert.equal(assertWorkPackages([wp], 1)[0].id, wp.id);
  assert.throws(() => assertFeatures({ results: [] }), /expected array/);
  assert.throws(() => assertFeatures([{ ...feature, state: "Unrecognized" }]), /Invalid/);
  assert.throws(() => assertFeatures([feature, { ...feature, id: 2 }]), /duplicate/);
  assert.throws(() => assertFeatures([{ ...feature, id: -1 }]), /Invalid/);
  assert.throws(() => assertWorkPackages([wp], 2), /mismatched/);
  assert.throws(() => assertWorkPackages([{ ...wp, state: "accepted" }], 1), /Invalid/);
  assert.throws(() => assertWorkPackages([wp, wp], 1), /duplicate/);
  assert.throws(() => assertWorkPackages([{ ...wp, sequence: "one" }], 1), /Invalid/);
});

async function freePort() {
  const server = createServer();
  server.listen(0, "127.0.0.1");
  await once(server, "listening");
  const port = server.address().port;
  await new Promise((done) => server.close(done));
  return port;
}

function start(script, port, env) {
  const child = spawn(process.execPath, [script], {
    cwd: root,
    env: { ...process.env, ...env, PORT: String(port) },
    stdio: ["ignore", "pipe", "pipe"],
  });
  child.stdout.resume();
  child.stderr.resume();
  return child;
}

async function ready(url, timeoutMs = 9000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    try {
      const response = await fetch(url, { signal: AbortSignal.timeout(900) });
      if (response.ok) return response;
    } catch { /* service not listening yet */ }
    await new Promise((resolve) => setTimeout(resolve, 80));
  }
  throw new Error(`Fixture not ready: ${url}`);
}

test("HTTP fixture: real read mapping, no writable or phantom-success endpoints", { timeout: 20000 }, async (t) => {
  const apiPort = await freePort();
  const shimPort = await freePort();
  const key = "test-key-local-only";
  const api = start("fixture-server.js", apiPort, { AGILEPLUS_API_KEY: key });
  t.after(() => api.kill());
  const shim = start("src/server.js", shimPort, {
    AGILEPLUS_API_URL: `http://127.0.0.1:${apiPort}`,
    AGILEPLUS_API_KEY: key,
    WEB_ORIGIN: "http://127.0.0.1:4400",
  });
  t.after(() => shim.kill());
  const origin = `http://127.0.0.1:${shimPort}`;
  await ready(`${origin}/shim/health`);
  const health = await (await fetch(`${origin}/shim/health`)).json();
  assert.deepEqual({ ok: health.ok, mode: health.mode, features: health.features }, {
    ok: true, mode: "local-read-only", features: 2,
  });
  assert.equal(Object.hasOwn(health, "agileplus"), false, "do not leak backend URL");

  const projectsRes = await fetch(`${origin}/api/workspaces/agileplus/projects/`);
  assert.equal(projectsRes.status, 200);
  const projects = await projectsRes.json();
  assert.equal(projects.length, 2);
  const p = projects.find((x) => x._agileplus_slug === "harness-duel");
  assert.ok(p);
  assert.equal(p._agileplus_id, 1);
  const issuesRes = await fetch(`${origin}/api/workspaces/agileplus/projects/${p.id}/issues/`);
  assert.equal(issuesRes.status, 200);
  const issues = await issuesRes.json();
  assert.equal(issues.total_count, 2);
  assert.ok(issues.results.some((x) => x.state__group === "completed"));
  assert.ok(issues.results.some((x) => x.state__group === "started"));
  for (const path of [
    "/api/workspaces/unknown/projects/",
    "/api/workspaces/agileplus/projects/no-such-project/states/",
    "/api/workspaces/agileplus/projects/no-such-project/members/",
    "/api/workspaces/agileplus/projects/no-such-project/issues/",
    "/api/does-not-exist/",
  ]) {
    assert.equal((await fetch(origin + path)).status, 404, path);
  }
  for (const method of ["POST", "PATCH", "PUT", "DELETE"]) {
    const result = await fetch(`${origin}/api/workspaces/agileplus/projects/${p.id}/issues/`, { method });
    assert.equal(result.status, 405, method);
    assert.equal((await result.json()).error, "read_only");
  }
  api.kill();
  const downRes = await fetch(`${origin}/api/workspaces/agileplus/projects/`);
  assert.equal(downRes.status, 502, "backend outage must not show a successful empty board");
});

test("shim refuses to start without explicit backend key", { timeout: 5000 }, async () => {
  const port = await freePort();
  const env = { ...process.env, PORT: String(port) };
  delete env.AGILEPLUS_API_KEY;
  const child = spawn(process.execPath, ["src/server.js"], { cwd: root, env, stdio: ["ignore", "pipe", "pipe"] });
  const [code] = await once(child, "exit");
  assert.notEqual(code, 0);
});
