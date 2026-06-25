/**
 * AgilePlus Desktop — Electrobun main process (step-1, offline-first).
 *
 * Responsibilities:
 *   - Manage a single BrowserWindow (Electrobun view).
 *   - Read/write local repo state: specs, ADRs, traces, acceptance files.
 *   - Drive the `agileplus` CLI for project management state transitions.
 *   - Stay offline: no network calls. Separate from the web client.
 *
 * End-state: native shells per platform (WinUI-rs on Windows, SwiftUI on
 * macOS, Linux-native via GTK/whatever the platform team selects). See
 * ADR for the platform split. This file is the step-1 Electrobun shell.
 */

import { BrowserWindow } from "electrobun";
import { RepoBridge } from "./repo-bridge";
import { CLI } from "./cli";
import { AppPaths } from "./paths";
import { MainView } from "./views";

const paths = AppPaths.fromCwd(process.cwd());
const repo = new RepoBridge(paths);
const cli = new CLI(paths);

const mainView = new MainView({ repo, cli, paths });

const win = new BrowserWindow({
  title: "AgilePlus",
  width: 1180,
  height: 760,
  minWidth: 880,
  minHeight: 540,
  // The HTML view source. Electrobun compiles `src/views/main.html`
  // (or equivalent) to a runtime-resolvable URL.
  url: mainView.url(),
});

win.on("ready", () => {
  // Step-1: the renderer asks for the repo root on boot. Everything else
  // is file-driven through the same bridge.
  mainView.bootstrap(win);
});

win.on("closed", () => {
  // No background work to tear down — we are fully offline.
});
