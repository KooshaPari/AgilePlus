import { defineConfig } from '@playwright/test';
export default defineConfig({
  testDir: './tests/operator', workers: 1, retries: 0, timeout: 60000,
  use: { baseURL: 'http://127.0.0.1:39002', httpCredentials: { username: 'smoke', password: 'fixture-password' } },
  reporter: [['list'], ['html', { outputFolder: 'operator-browser-report', open: 'never' }]],
});
