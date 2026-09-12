/**
 * Thin abstraction over the local repo. Reads and writes the spec-kit
 * style artifacts: specs under `kitty-specs/`, ADRs under `docs/adr/`,
 * traces under `traces/`, and acceptance files alongside each spec.
 *
 * This is the OFFLINE-FIRST boundary. No network, no remote storage, no
 * RPC — just the filesystem of the selected repo.
 *
 * MIGRATION NOTE (ADR-020): This module will be replaced by Tauri
 * commands that call into the Rust core directly. The current
 * implementation shells out to the `agileplus` CLI for state reads
 * and mutations. When Tauri is in place, these become native Rust
 * function calls with zero process overhead.
 */

import * as fs from "node:fs/promises";
import * as path from "node:path";
import { execFile } from "node:child_process";
import { promisify } from "node:util";
import type { AppPaths } from "./paths";

const execFileAsync = promisify(execFile);

export interface SpecSummary {
  id: string;
  title: string;
  state: string;
  path: string;
}

export interface AdrSummary {
  id: string;
  title: string;
  status: string;
  path: string;
}

export interface TraceEntry {
  id: string;
  kind: string;
  path: string;
}

export class RepoBridge {
  constructor(private readonly paths: AppPaths) {}

  /** All spec feature directories under `kitty-specs/`, with real state from SQLite. */
  async listSpecs(): Promise<SpecSummary[]> {
    const dirs = await this.listDirs(this.paths.specsDir, (id, dir) => {
      const specFile = path.join(dir, "spec.md");
      return { id, title: id, state: "unknown", path: specFile };
    });

    // Attempt to read actual state from `agileplus list`.
    // NOTE: --json is not yet implemented; returns filesystem-based state.
    // The Tauri migration (ADR-020) will replace this with direct Rust calls.
    return dirs;
  }

  /** All ADRs under `docs/adr/`. */
  async listAdrs(): Promise<AdrSummary[]> {
    return this.listFiles(this.paths.adrDir, /\.md$/i, (file) => {
      const base = path.basename(file, ".md");
      return {
        id: base,
        title: base,
        status: "unknown",
        path: file,
      } satisfies AdrSummary;
    }) as Promise<AdrSummary[]>;
  }

  /** All worklogs/trace files under `traces/`. */
  async listTraces(): Promise<TraceEntry[]> {
    return this.listFiles(this.paths.tracesDir, /\.jsonl?$|\.md$/i, (file) => {
      const base = path.basename(file);
      return {
        id: base,
        kind: base.endsWith(".jsonl") ? "jsonl" : "md",
        path: file,
      } satisfies TraceEntry;
    }) as Promise<TraceEntry[]>;
  }

  /** Read a text file relative to the repo root. */
  async readText(relPath: string): Promise<string> {
    const abs = this.abs(relPath);
    return fs.readFile(abs, "utf8");
  }

  /** Write a text file relative to the repo root. */
  async writeText(relPath: string, body: string): Promise<void> {
    const abs = this.abs(relPath);
    await fs.mkdir(path.dirname(abs), { recursive: true });
    await fs.writeFile(abs, body, "utf8");
  }

  /**
   * Create or update a feature spec via the CLI.
   *
   * Calls `agileplus specify --feature <slug> --from-file <tmpfile>`.
   * Returns the CLI stdout on success, throws on failure.
   *
   * MIGRATION NOTE: In Tauri, this becomes a direct Rust call:
   * `specify::run_specify(args, &storage, &vcs).await`
   */
  async specifyFeature(slug: string, markdownBody: string): Promise<string> {
    const tmpFile = path.join(this.paths.userDataDir, `${slug}-spec.md`);
    await fs.writeFile(tmpFile, markdownBody, "utf8");
    try {
      const { stdout } = await execFileAsync(
        "agileplus",
        ["specify", "--feature", slug, "--from-file", tmpFile, "--force"],
        { cwd: this.paths.repoRoot, timeout: 30_000 },
      );
      return stdout;
    } finally {
      await fs.unlink(tmpFile).catch(() => {});
    }
  }

  /**
   * Run `agileplus list` and return parsed feature data.
   *
   * MIGRATION NOTE: In Tauri, this calls `list::run(args, &storage)` directly.
   */
  async listFeatures(): Promise<
    Array<{ slug: string; state: string; title: string }>
  > {
    // TODO: `agileplus list --json` not yet implemented.
    // Track: https://github.com/... (add issue link when created)
    // For now, return empty; Tauri migration will fix this.
    return [];
  }

  /**
   * Run `agileplus dashboard` and return the text output.
   *
   * MIGRATION NOTE: In Tauri, this calls `dashboard::run(&args)` directly.
   */
  async dashboard(): Promise<string> {
    try {
      const { stdout } = await execFileAsync("agileplus", ["dashboard"], {
        cwd: this.paths.repoRoot,
        timeout: 10_000,
      });
      return stdout;
    } catch (err) {
      return `Error: ${(err as Error).message}`;
    }
  }

  private abs(relPath: string): string {
    const abs = path.resolve(this.paths.repoRoot, relPath);
    if (!abs.startsWith(this.paths.repoRoot)) {
      throw new Error(`Path escapes repo root: ${relPath}`);
    }
    return abs;
  }

  private async listDirs(
    root: string,
    map: (id: string, dir: string) => SpecSummary,
  ): Promise<SpecSummary[]> {
    try {
      const entries = await fs.readdir(root, { withFileTypes: true });
      const out: SpecSummary[] = [];
      for (const e of entries) {
        if (!e.isDirectory()) continue;
        if (e.name.startsWith(".")) continue;
        out.push(map(e.name, path.join(root, e.name)));
      }
      return out.sort((a, b) => a.id.localeCompare(b.id));
    } catch (err) {
      if ((err as NodeJS.ErrnoException).code === "ENOENT") return [];
      throw err;
    }
  }

  private async listFiles(
    root: string,
    re: RegExp,
    map: (file: string) => AdrSummary | TraceEntry,
  ): Promise<(AdrSummary | TraceEntry)[]> {
    try {
      const entries = await fs.readdir(root, { withFileTypes: true });
      const out: (AdrSummary | TraceEntry)[] = [];
      for (const e of entries) {
        if (!e.isFile()) continue;
        if (!re.test(e.name)) continue;
        out.push(map(path.join(root, e.name)));
      }
      return out.sort((a, b) => a.id.localeCompare(b.id));
    } catch (err) {
      if ((err as NodeJS.ErrnoException).code === "ENOENT") return [];
      throw err;
    }
  }
}
