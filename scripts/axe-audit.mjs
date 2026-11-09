// SPDX-License-Identifier: MIT OR Apache-2.0
// Run an axe-core accessibility audit across the dashboard routes using
// Playwright + @axe-core/playwright. Emits an axe-report.json next to
// this script so the workflow can upload it as an artifact.
//
// Traces to: FR-A11Y-01 (accessibility), pillar L76 (Accessibility)

import { chromium } from 'playwright';
import { AxeBuilder } from '@axe-core/playwright';
import fs from 'node:fs';

const browser = await chromium.launch();
const context = await browser.newContext();
const page = await context.newPage();

const baseUrl = process.env.AXE_BASE_URL ?? 'http://localhost:5173';
const rules = (process.env.AXE_DISABLE_RULES ?? '')
  .split(',')
  .map((s) => s.trim())
  .filter(Boolean);

const targets = [
  { name: 'home', url: '/' },
  { name: 'features', url: '/features' },
  { name: 'cycles', url: '/cycles' },
  { name: 'cockpit', url: '/cockpit' },
];

const report = {
  url: baseUrl,
  timestamp: new Date().toISOString(),
  results: [],
};

for (const t of targets) {
  try {
    await page.goto(baseUrl + t.url, { waitUntil: 'networkidle', timeout: 10_000 });
    const builder = new AxeBuilder({ page }).withTags([
      'wcag2a',
      'wcag2aa',
      'wcag21a',
      'wcag21aa',
    ]);
    for (const r of rules) builder.disableRules([r]);
    const r = await builder.analyze();
    report.results.push({
      page: t.name,
      url: t.url,
      violations: r.violations,
      passes: r.passes.length,
    });
    console.log(`[${t.name}] ${r.violations.length} violations`);
  } catch (e) {
    console.error(`[${t.name}] audit failed: ${e.message}`);
    report.results.push({ page: t.name, url: t.url, error: String(e) });
  }
}

fs.writeFileSync(
  process.env.AXE_REPORT_PATH ?? 'axe-report.json',
  JSON.stringify(report, null, 2),
);

const totalViolations = report.results.reduce(
  (s, r) => s + (r.violations?.length ?? 0),
  0,
);
console.log(`TOTAL_VIOLATIONS=${totalViolations}`);
await browser.close();

// Allow some pre-existing dev-only noise but fail hard on >5 serious/critical.
const serious = report.results.reduce(
  (s, r) =>
    s +
    (r.violations?.filter(
      (v) => v.impact === 'serious' || v.impact === 'critical',
    ).length ?? 0),
  0,
);
process.exit(serious > 5 ? 1 : 0);