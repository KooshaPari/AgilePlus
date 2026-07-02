/**
 * AgilePlus Desktop Shell — Electrobun Configuration
 *
 * RENDERER_URL environment variable:
 *   - Dev: RENDERER_URL=http://localhost:5173 (vite dev server with HMR)
 *   - Prod: RENDERER_URL=http://localhost:8770 (dashboard daemon, set by launcher)
 */
import type { ElectrobunConfig } from "electrobun";

// ── App identity ──────────────────────────────────────────────────────────────
const APP_NAME = "AgilePlus";
const APP_ID = "com.phenotype.agileplus";
const APP_VERSION = "0.1.0";

// ── Renderer configuration ────────────────────────────────────────────────────
// Dev server (HMR): localhost:5173 (vite dev server)
// Daemon (prod): localhost:8770 (agileplus-dashboard binary)
// The launcher sets RENDERER_URL=http://localhost:$PORT when starting Electrobun.
const DEFAULT_RENDERER_URL = "http://localhost:8770";

// ── Bundled fallback page (used when renderer is unreachable) ────────────────
const VIEWS_ENTRYPOINT = "src/views/index.html";

export default {
  app: {
    name: APP_NAME,
    identifier: APP_ID,
    version: APP_VERSION,
  },
  runtime: {
    exitOnLastWindowClosed: true,
    // Passed to main.ts via process.env.RENDERER_URL at runtime
    devRendererUrl: process.env.RENDERER_URL ?? DEFAULT_RENDERER_URL,
  },
  build: {
    bun: {
      entrypoint: "src/main.ts",
    },
    views: [
      {
        name: "app",
        entrypoint: VIEWS_ENTRYPOINT,
      },
    ],
  },
} satisfies ElectrobunConfig;
