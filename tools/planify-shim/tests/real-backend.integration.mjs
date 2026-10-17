import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { randomBytes } from "node:crypto";
import { mkdtemp, mkdir, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve, join } from "node:path";
import net from "node:net";

const repo = resolve(import.meta.dirname, "../../..");
const shimDir = resolve(repo, "tools/planify-shim");
const tmp = await mkdtemp(join(tmpdir(), "agileplus-api-smoke-"));
const home = join(tmp, "home");
const processes = [];
async function port() {
  const server = net.createServer();
  await new Promise((done) => server.listen(0, "127.0.0.1", done));
  const n = server.address().port;
  await new Promise((done) => server.close(done));
  return n;
}
function start(command, args, cwd, env, label) {
  const p = spawn(command, args, { cwd, env: { ...process.env, ...env }, stdio: ["ignore", "pipe", "pipe"] });
  let log = "";
  for (const s of [p.stdout, p.stderr]) s.on("data", (b) => { log = (log + b.toString()).slice(-7000); });
  const item = { p, label, tail: () => log };
  processes.push(item);
  return item;
}
async function ready(url, item, timeout = 45000) {
  const deadline = Date.now() + timeout;
  while (Date.now() < deadline) {
    if (item.p.exitCode !== null) throw Error(item.label + " exited:\n" + item.tail());
    try { const r = await fetch(url, { signal: AbortSignal.timeout(1000) }); if (r.ok) return; } catch {}
    await new Promise((done) => setTimeout(done, 160));
  }
  throw Error(item.label + " timed out:\n" + item.tail());
}
async function apiCall(base, key, path, method = "GET", body) {
  const r = await fetch(base + path, { method, headers: {
    "X-API-Key": key, Accept: "application/json", ...(body ? { "Content-Type": "application/json" } : {})
  }, body: body ? JSON.stringify(body) : undefined, signal: AbortSignal.timeout(10000) });
  const data = await r.json().catch(() => ({}));
  return { status: r.status, data };
}
async function stop(item) {
  if (item.p.exitCode !== null) return;
  item.p.kill("SIGTERM");
  await Promise.race([
    new Promise((resolve) => item.p.once("exit", resolve)),
    new Promise((resolve) => setTimeout(resolve, 2000)),
  ]);
  if (item.p.exitCode === null) item.p.kill("SIGKILL");
}
await mkdir(home, { recursive: true });
try {
  const apiPort = await port();
  const shimPort = await port();
  const api = "http://127.0.0.1:" + apiPort;
  const shim = "http://127.0.0.1:" + shimPort;
  const key = randomBytes(24).toString("hex");
  const binary = process.env.AGILEPLUS_API_BIN || resolve(repo, "target/debug/agileplus-api");
  const rust = start(binary, [], repo, {
    HOME: home, API_HOST: "127.0.0.1", AGILEPLUS_API_PORT: String(apiPort),
    AGILEPLUS_API_KEY: key, AGILEPLUS_CREDENTIAL_KEY: randomBytes(32).toString("hex"),
    AGILEPLUS_CORE_DB_PATH: join(tmp, "ephemeral.db"),
    AGILEPLUS_CONFIG_PATH: join(tmp, "missing.toml"),
  }, "Rust API");
  await ready(api + "/health", rust);
  assert.equal((await apiCall(api, "wrong", "/api/v1/features")).status, 401);
  const name = "SSOT Isolated Real Rust " + randomBytes(3).toString("hex");
  const f = await apiCall(api, key, "/api/v1/features", "POST", { title: name });
  assert.equal(f.status, 201, "feature create: " + JSON.stringify(f.data));
  assert.ok(f.data.id > 0);
  const wp = await apiCall(api, key, "/api/v1/features/" + f.data.slug + "/work-packages", "POST", {
    title: "SSOT Real API Work Package",
    acceptance_criteria: "CI integration only; no production state",
    sequence: 1
  });
  assert.equal(wp.status, 201, "WP create: " + JSON.stringify(wp.data));
  for (const next of ["doing", "review"]) {
    const r = await apiCall(api, key, "/api/v1/work-packages/" + wp.data.id + "/transition", "POST", { target_state: next });
    assert.equal(r.status, 200, "transition " + next + ": " + JSON.stringify(r.data));
  }
  const done = await apiCall(api, key, "/api/v1/work-packages/" + wp.data.id + "/transition", "POST", { target_state: "done" });
  assert.equal(done.status, 409, "canonical API must reject direct Done");
  const proxy = start(process.execPath, ["src/server.js"], shimDir, {
    AGILEPLUS_API_URL: api, AGILEPLUS_API_KEY: key, PORT: String(shimPort),
    WEB_ORIGIN: "http://127.0.0.1:3000"
  }, "Planify shim");
  await ready(shim + "/shim/health", proxy);
  const projectResponse = await fetch(shim + "/api/workspaces/agileplus/projects/");
  assert.equal(projectResponse.status, 200);
  const projects = await projectResponse.json();
  const project = projects.find((p) => p._agileplus_slug === f.data.slug);
  assert.ok(project, "feature did not map to Planify project");
  const issuePath = "/api/workspaces/agileplus/projects/" + project.id + "/issues/";
  const issues = await (await fetch(shim + issuePath)).json();
  assert.equal(issues.total_count, 1);
  assert.equal(issues.results[0]._agileplus_source_state, "review");
  assert.equal(issues.results[0]._agileplus_product_acceptance, "not_inferred");
  assert.equal((await fetch(shim + issuePath, { method: "POST" })).status, 405);
  assert.equal((await apiCall(api, key, "/api/v1/work-packages/" + wp.data.id)).data.state, "review");
  if (process.env.PLANIFY_DIR) {
    const { chromium } = await import("playwright");
    const { writeFile } = await import("node:fs/promises");
    const path = process.env.PLANIFY_ARTIFACTS_DIR || join(tmp, "browser-evidence");
    await mkdir(path, { recursive: true });
    const web = start("pnpm", ["--filter", "web", "dev"], process.env.PLANIFY_DIR, {
      VITE_API_BASE_URL: shim,
      VITE_WEB_BASE_URL: "http://127.0.0.1:3000",
    }, "Planify browser frontend");
    await ready("http://127.0.0.1:3000", web, 120000);
    const browser = await chromium.launch({ headless: true });
    const evidence = { browserErrors: [], shimRequests: [], expectedWorkPackage: "SSOT Real API Work Package" };
    try {
      const page = await browser.newPage({ viewport: { width: 1440, height: 960 } });
      page.on("pageerror", (error) => evidence.browserErrors.push(error.message));
      page.on("response", (response) => {
        if (response.url().startsWith(shim)) {
          evidence.shimRequests.push({ path: response.url().slice(shim.length), status: response.status() });
        }
      });
      const url = "http://127.0.0.1:3000/agileplus/projects/" + project.id + "/issues";
      await page.goto(url, { waitUntil: "domcontentloaded", timeout: 90000 });
      await page.waitForTimeout(14000);
      evidence.url = page.url();
      evidence.title = await page.title();
      evidence.body = (await page.locator("body").innerText()).slice(0, 6000);
      await page.screenshot({ path: join(path, "real-planify.png"), fullPage: true });
      await writeFile(join(path, "result.json"), JSON.stringify(evidence, null, 2));
      assert.ok(evidence.body.includes("SSOT Real API Work Package"), "Planify browser did not render canonical WP; inspect uploaded screenshot and response trace");
      assert.ok(evidence.shimRequests.some((r) => r.path.includes("/projects/") && r.status === 200),
        "Planify UI did not load a real project response from the shim");
      console.log("PLANIFY BROWSER PASS: actual owned fork rendered a Rust-backed WP");
    } finally { await browser.close(); }
  }

  console.log("REAL RUST INTEGRATION PASS: source=" + f.data.slug + ", WP=" + wp.data.id + ", project=" + project.id);
} finally {
  for (const item of processes.reverse()) await stop(item);
  await rm(tmp, { recursive: true, force: true });
}
