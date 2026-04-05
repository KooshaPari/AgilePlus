# Ziggistry Research Report

**Date:** 2026-04-04  
**Status:** Registry not found / Not standardized

## Findings

### Ziggistry Does Not Exist as a Standard Registry

After research, **Ziggistry** does not appear to exist as a standardized package registry for Zig:

1. **No web presence:** ziggistry.dev does not resolve
2. **No GitHub Actions:** No standard publish-zig action exists
3. **Zig's ecosystem status:** Zig is still maturing its package management

### Current State of Zig Package Management

As of early 2024-2025:

1. **build.zig.zon** - Zig's native package manifest format (similar to Cargo.toml)
2. **No centralized registry** - Zig fetches dependencies directly from URLs (Git repos, tarballs)
3. **Package hash verification** - Uses content hashing for integrity

### How Zig Packages Currently Work

```zig
// build.zig.zon example
.{
    .name = "my-package",
    .version = "0.1.0",
    .dependencies = .{
        .some_lib = .{
            .url = "https://github.com/user/repo/archive/refs/tags/v1.0.0.tar.gz",
            .hash = "sha256-1234567890abcdef...",
        },
    },
}
```

### Recommendation

**DO NOT create a Ziggistry workflow at this time.**

Instead:
1. **Tag-based releases:** Use Git tags for Zig package versions
2. **Direct URL fetching:** Consumers fetch directly from GitHub releases
3. **Wait for official registry:** The Zig team may create one in the future
4. **Azig (if needed):** Community has discussed "Azig" as a potential registry name

### Alternative: GitHub Release Publishing

For Zig packages, create a workflow that:
1. Validates `build.zig.zon`
2. Runs `zig build test`
3. Creates GitHub release with source tarball
4. Updates documentation with URL/hash for consumers

---

**Decision:** Defer Ziggistry support until a standardized registry emerges.
