/**
 * Ambient type declarations for Electrobun runtime APIs.
 *
 * Electrobun v2 ships as a CLI bootstrap only; the actual type definitions
 * are provided by the native runtime at build time.  These declarations
 * cover the subset used by the AgilePlus desktop step-1 shell so that
 * TypeScript can type-check the renderer and main-process code.
 *
 * TODO: replace with upstream Electrobun types once they ship .d.ts files.
 */

declare module "electrobun/bun" {
  export interface FrameOptions {
    x?: number;
    y?: number;
    width?: number;
    height?: number;
  }

  export interface BrowserWindowOptions {
    title: string;
    url: string;
    rpc?: unknown;
    frame?: FrameOptions;
  }

  export class BrowserWindow {
    constructor(options: BrowserWindowOptions);
  }
}

declare module "electrobun/view" {
  export interface RPCDefinition {
    bun: {
      requests: Record<string, { params: unknown; response: unknown }>;
      messages: Record<string, unknown>;
    };
    webview: {
      requests: Record<string, { params: unknown; response: unknown }>;
      messages: Record<string, unknown>;
    };
  }

  export interface RPCRequestHandler {
    request: Record<string, (params: unknown) => Promise<unknown>>;
  }

  export interface ElectroviewOptions {
    rpc?: RPCRequestHandler;
  }

  export class Electrobun {
    constructor(options: ElectroviewOptions);
    rpc?: RPCRequestHandler;
  }

  export namespace Electroview {
    function defineRPC<T extends RPCDefinition>(options: {
      maxRequestTime?: number;
      handlers: {
        requests: Record<string, (params: unknown) => Promise<unknown>>;
        messages: Record<string, (params: unknown) => void>;
      };
    }): RPCRequestHandler;
  }
}

declare module "electrobun" {
  export type { ElectrobunConfig } from "electrobun/config";
}

declare module "electrobun/config" {
  export interface ElectrobunConfig {
    app: {
      name: string;
      identifier: string;
      version: string;
    };
    build: {
      bun: { entrypoint: string };
      views: Record<string, { entrypoint: string }>;
      copy?: Record<string, string>;
      mac?: { bundleCEF?: boolean };
      linux?: { bundleCEF?: boolean };
      win?: { bundleCEF?: boolean };
    };
  }
}
