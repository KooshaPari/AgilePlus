/** Real Chromium → checked-in Caddy gateway → Axum → file SQLite → restart. */
import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { spawn, execFileSync, type ChildProcess } from 'node:child_process';
import { mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { resolve, join } from 'node:path';
import { once } from 'node:events';
const root = resolve(import.meta.dirname, '../../../../..');
const temp = mkdtempSync(join(tmpdir(), 'agileplus-operator-'));
const key = 'disposable-browser-fixture-key';
const processes: ChildProcess[] = [];
let backend: ChildProcess;
function startBackend() {
  backend = spawn(join(root, 'target/debug/examples/operator_fixture'), [], {
    env: { ...process.env, AGILEPLUS_TEST_DATABASE: join(temp, 'state.sqlite'), AGILEPLUS_TEST_API_KEY: key }, stdio: 'inherit',
  }); processes.push(backend);
}
async function ready(url: string) {
  await expect.poll(async () => { try { return (await fetch(url)).status; } catch { return 0; } }, { timeout: 20000 }).toBe(200);
}
async function restartBackend() {
  const exited = once(backend, 'exit'); backend.kill('SIGTERM'); await exited;
  startBackend(); await ready('http://127.0.0.1:39001/health');
}
test.beforeAll(async () => {
  startBackend(); await ready('http://127.0.0.1:39001/health');
  const staticServer = spawn('python3', ['-m', 'http.server', '39003', '--bind', '127.0.0.1', '--directory', join(root, 'crates/agileplus-dashboard/web/dist')], { stdio: 'inherit' });
  processes.push(staticServer); await ready('http://127.0.0.1:39003');
  const caddy = process.env.CADDY_BIN || 'caddy';
  const hash = execFileSync(caddy, ['hash-password', '--plaintext', 'fixture-password'], { encoding: 'utf8' }).trim();
  const source = readFileSync(join(root, 'deploy/selfhost/Caddy.operator-alpha.caddy'), 'utf8')
    .replace('agileplus.pheno.studio {', 'http://127.0.0.1:39002 {')
    .replace('https://agileplus.pheno.studio', 'http://127.0.0.1:39002')
    .replace('reverse_proxy 127.0.0.1:3000', 'reverse_proxy 127.0.0.1:39001')
    .replace('reverse_proxy https://{$AGILEPLUS_FRONTEND_UPSTREAM_HOST}', 'reverse_proxy http://127.0.0.1:39003');
  writeFileSync(join(temp, 'Caddyfile'), '{\n admin off\n auto_https off\n}\n' + source);
  const gateway = spawn(caddy, ['run', '--config', join(temp, 'Caddyfile'), '--adapter', 'caddyfile'], {
    env: { ...process.env, AGILEPLUS_OPERATOR_USERNAME: 'smoke', AGILEPLUS_OPERATOR_PASSWORD_HASH: hash,
      AGILEPLUS_API_KEY: key, AGILEPLUS_FRONTEND_UPSTREAM_HOST: '127.0.0.1:39003' }, stdio: 'inherit',
  }); processes.push(gateway);
  await expect.poll(async () => { try { return (await fetch('http://127.0.0.1:39002')).status; } catch { return 0; } }, { timeout: 20000 }).toBe(401);
});
test.afterAll(async () => {
  for (const process of processes.reverse()) if (process.exitCode === null && process.signalCode === null) {
    const exited = once(process, 'exit'); process.kill('SIGTERM'); await exited;
  }
});
test('lost response, backend restart and historical receipt recovery', async ({ page }) => {
  const consoleErrors: string[] = [];
  page.on('pageerror', error => consoleErrors.push(error.message));
  page.on('request', request => {
    if (request.url().includes('/api/')) expect(request.headers()['x-api-key']).toBeUndefined();
  });
  await page.goto('/');
  await expect(page.getByRole('heading', { name: 'Atomic acceptance', exact: true })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Accept feature', exact: true })).toBeEnabled();
  expect((await new AxeBuilder({ page }).withTags(['wcag2a','wcag2aa','wcag21a','wcag21aa']).analyze()).violations).toEqual([]);
  let identity = '';
  await page.route('**/api/v1/features/atomic/accept', async route => {
    identity = JSON.parse(route.request().postData()!).request_id;
    const response = await route.fetch(); expect(response.status()).toBe(200);
    // Drop only the caller response after the real backend has committed.
    await route.abort('failed');
  });
  await page.getByRole('button', { name: 'Accept feature', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Retry acceptance', exact: true })).toBeEnabled();
  await page.unroute('**/api/v1/features/atomic/accept');
  await restartBackend(); await page.reload();
  await expect(page.getByText(identity, { exact: true })).toBeVisible();
  await expect(page.getByText('No committed acceptance receipt.')).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'Accept feature', exact: true })).toBeDisabled();
  const counts = JSON.parse(execFileSync('python3', ['-c',
    'import sqlite3,json,sys; c=sqlite3.connect(sys.argv[1]); print(json.dumps([c.execute("SELECT COUNT(*) FROM feature_acceptance_receipts").fetchone()[0],c.execute("SELECT COUNT(*) FROM events").fetchone()[0]]))',join(temp,'state.sqlite')], { encoding: 'utf8' }));
  expect(counts).toEqual([1,1]); expect(consoleErrors).toEqual([]);
  await page.screenshot({ path: 'operator-browser-receipt.png', fullPage: true });
});
