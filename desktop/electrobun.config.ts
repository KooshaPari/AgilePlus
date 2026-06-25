/**
 * Electrobun build configuration for AgilePlus desktop (step-1).
 *
 * Defines a single `main` view that ships the main process bundle and
 * the renderer bundle (HTML + TS + CSS) together. The native build
 * target — the END-STATE per the platform ADR — will replace this
 * whole file with platform-specific manifests.
 */

import { defineConfig } from "electrobun/config";

export default defineConfig({
  app: {
    name: "AgilePlus",
    identifier: "dev.agileplus.desktop",
    version: "0.1.0",
  },
  build: {
    views: {
      main: {
        entry: "src/views/main.html",
        bundle: "src/views/main.ts",
      },
    },
    main: {
      entry: "src/index.ts",
    },
  },
});
