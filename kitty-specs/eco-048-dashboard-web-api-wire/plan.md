# Plan: Dashboard Web API Wire

1. Add typed axios client (`src/lib/api/`) with `VITE_API_BASE` config
2. Add `useDashboardData` hook for epics/stories fetch
3. Update `useWorkPackages` to use the shared client
4. Fix PostCSS/vitest deps so web quality gates pass locally
5. Register kitty-spec and satisfy PR governance metadata
