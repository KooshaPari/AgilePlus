// Cloudflare secrets bootstrap helper
// Run with:   node scripts/wrangler-secrets-bootstrap.mjs

import { execSync } from "node:child_process";

const REQUIRED = [
  "AGENTSHIELD_TOKEN",
  "PHENOTYPE_REGISTRY_TOKEN",
  "CIVIS_PROVIDER_TOKEN",
  "GITHUB_PAT",
  "OPENTELEMETRY_AUTH",
];

const OPTIONAL = ["SENTRY_DSN", "CF_TURNSTILE_SECRET"];

function list() {
  console.log("\nRequired secrets:");
  for (const s of REQUIRED) console.log("  - " + s);
  console.log("\nOptional secrets:");
  for (const s of OPTIONAL) console.log("  - " + s);
}

function checkWrangler() {
  try {
    execSync("npx wrangler --version", { stdio: "pipe" });
  } catch (e) {
    console.error("wrangler not installed. Run: npm i -g wrangler");
    process.exit(1);
  }
}

function pushSecret(name) {
  console.log("\nSetting: " + name);
  try { execSync("npx wrangler secret put " + name, { stdio: "inherit" }); return true; }
  catch (e) { console.error("fail: " + name); return false; }
}

function main() {
  const args = process.argv.slice(2);
  if (args.includes("--dry-run") || args.includes("-n")) return console.log("Dry-run — see list below."), list();
  if (args.includes("--list") || args.includes("-l")) return list();

  checkWrangler();
  list();
  const ok = [], fail = [];
  for (const n of REQUIRED) (pushSecret(n) ? ok : fail).push(n);
  console.log("\nSet:    " + ok.join(", "));
  if (fail.length) console.log("Failed: " + fail.join(", "));
}

main();
