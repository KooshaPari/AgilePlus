status: specified

# Spec: heliosCLI rmcp-client SDK alignment

## Problem
`codex-rmcp-client` in `heliosCLI` fails to compile on main because the crate pins
`reqwest 0.12.28` while `rmcp 1.3.0` provides its streamable HTTP client impls for
`reqwest 0.13.2`. The SDK mismatch breaks the `StreamableHttpClient` trait bound and
prevents the `sdks` job from building the workspace.

## Scope
- Update `codex-rs/rmcp-client` to use the `reqwest` version required by `rmcp`.
- Keep the change localized to the SDK crate and its direct validation surface.
- Add or adjust a focused test only if a compile-time guard is needed.

## Acceptance Criteria
- `codex-rs/rmcp-client` builds successfully with the resolved `rmcp` feature set.
- The `sdks` job no longer fails on the `StreamableHttpClient` bound mismatch.
- No unrelated crates or workflows are changed.
