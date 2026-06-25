/**
 * View wrapper. Holds the bootstrap hook (the renderer calls
 * `getRepoState` to learn the selected repo) and resolves the
 * view's URL for Electrobun to load.
 *
 * The bridge and CLI are the only side-effects in step-1; everything
 * else is just rendering local files.
 */

import { electrobun, type BrowserWindow } from "electrobun";
import type { RepoBridge } from "../repo-bridge";
import type { CLI } from "../cli";
import type { AppPaths } from "../paths";

export class MainView {
  constructor(
    private readonly deps: {
      repo: RepoBridge;
      cli: CLI;
      paths: AppPaths;
    },
  ) {
    electrobun.rpc.register("getRepoState", async () => {
      const [specs, adrs, traces] = await Promise.all([
        this.deps.repo.listSpecs(),
        this.deps.repo.listAdrs(),
        this.deps.repo.listTraces(),
      ]);
      return {
        repoRoot: this.deps.paths.repoRoot,
        specs,
        adrs,
        traces,
      };
    });
  }

  /**
   * Resolves the path the renderer is served from. Step-1 returns a
   * `file://` URL pointing at the bundled HTML; end-state native
   * shells will not need this.
   */
  url(): string {
    // Bundled at build time by Electrobun. The `views/main` view id is
    // defined in electrobun.config.ts; for step-1 we point at the html
    // directly and let the bundler resolve the runtime URL.
    return "views://main/";
  }

  /** Push the latest state to the renderer on boot. */
  bootstrap(win: BrowserWindow): void {
    void win; // step-1: nothing to push; the renderer pulls on mount.
  }
}
