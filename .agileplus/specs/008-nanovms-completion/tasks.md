# Tasks: NanoVMS — Three-Tier Isolation Architecture

**Spec**: 008-nanovms-completion
**Status**: Draft
**Repository**: nanovms
**Last Updated**: 2026-04-02

## Architecture Overview

**Three-Tier Isolation Model** (based on ADR-002):

| Tier | Technology | Startup | Memory | Trust Level |
|------|------------|---------|--------|-------------|
| 1 | Wasmtime | ~1ms | ~1MB | Trusted |
| 2 | gVisor | ~90ms | ~20MB | Semi-trusted |
| 3 | Firecracker | ~125ms | <5MB | Untrusted |

## Work Packages

### Tier 1: WASM Sandboxes

#### WP-001: Wasmtime Integration
**Status**: ⚠️ Partial (stub exists)
**Priority**: P1
**Trust**: Trusted

- [x] Define WASM adapter interface in `internal/ports/ports.go`
- [ ] Implement Wasmtime runtime in `internal/adapters/wasm/wasm.go`
- [ ] Add WASI host function bindings (filesystem, network)
- [ ] Add WASM module compilation and caching
- [ ] Add unit tests for WASM adapter
- [ ] Benchmark: Target <1ms startup

#### WP-002: WASM Tool Host
**Status**: 📋 Planned
**Priority**: P1
**Trust**: Trusted

- [ ] Implement host function exports (stdio, filesystem, network)
- [ ] Add WASM module verification (WASM spec compliance)
- [ ] Add resource limits (memory, CPU, time)
- [ ] Add WASM <-> Go interop layer

### Tier 2: gVisor Containers

#### WP-010: gVisor Adapter
**Status**: 📋 Planned
**Priority**: P1
**Trust**: Semi-trusted

- [ ] Implement gVisor adapter in `internal/adapters/gvisor/gvisor.go`
- [ ] Add runsc runtime detection and installation
- [ ] Add OCI image pulling and caching
- [ ] Add seccomp profile management
- [ ] Add network namespace isolation
- [ ] Add unit tests for gVisor adapter
- [ ] Benchmark: Target <90ms startup

#### WP-011: gVisor Network Isolation
**Status**: 📋 Planned
**Priority**: P2
**Trust**: Semi-trusted

- [ ] Implement network namespace for gVisor containers
- [ ] Add iptables/nftables rules for isolation
- [ ] Add DNS configuration
- [ ] Add port forwarding

### Tier 3: Firecracker MicroVMs

#### WP-020: Firecracker Adapter
**Status**: ⚠️ Partial (stub exists)
**Priority**: P1
**Trust**: Untrusted

- [ ] Implement Firecracker adapter in `internal/adapters/firecracker/firecracker.go`
- [ ] Add Firecracker binary management (download, verify)
- [ ] Add kernel and initrd management
- [ ] Add VM lifecycle (create, start, stop, delete, pause)
- [ ] Add VM networking (tap device, vsock)
- [ ] Add VM storage (virtio-blk)
- [ ] Add unit tests for Firecracker adapter
- [ ] Benchmark: Target <125ms startup, <5MB memory

#### WP-021: Firecracker OCI Integration
**Status**: 📋 Planned
**Priority**: P2
**Trust**: Untrusted

- [ ] Add container image pulling (OCI distribution)
- [ ] Add rootfs extraction to microVM
- [ ] Add firecracker-containerd integration
- [ ] Add snapshot and resume support

### Infrastructure Adapters

#### WP-030: Lima Adapter (macOS)
**Status**: ✅ Partial
**Priority**: P1

- [x] Define VMAdapter interface in `internal/ports/ports.go`
- [x] Implement Lima adapter stub in `internal/adapters/mac/mac.go`
- [ ] Add Lima VM lifecycle (create, start, stop, delete)
- [ ] Add Lima networking (port forwarding)
- [ ] Add Lima filesystem mounts
- [ ] Add unit tests for Lima adapter

#### WP-031: WSL Adapter (Windows)
**Status**: ⚠️ Stub
**Priority**: P1

- [ ] Implement WSL adapter in `internal/adapters/windows/windows.go`
- [ ] Add WSL VM lifecycle (create, start, stop, delete)
- [ ] Add WSL networking (port forwarding)
- [ ] Add unit tests for WSL adapter

#### WP-032: Native Adapter (Linux)
**Status**: ⚠️ Stub
**Priority**: P1

- [ ] Implement Native adapter in `internal/adapters/linux/linux.go`
- [ ] Add KVM/HyperKit/Hyper-V detection
- [ ] Add VM lifecycle (create, start, stop, delete)
- [ ] Add unit tests for Native adapter

### CLI Interface

#### WP-040: Core CLI Commands
**Status**: ⚠️ Partial
**Priority**: P1

- [x] Define CLI structure in `cmd/nanovms/main.go`
- [ ] Implement `create` command
- [ ] Implement `delete` command
- [ ] Implement `list` command
- [ ] Implement `exec` command
- [ ] Implement `probe` command
- [ ] Add shell completion

#### WP-041: Tier-Specific CLI Commands
**Status**: 📋 Planned
**Priority**: P1

- [ ] Implement `sandbox create --tier wasm|gvisor|microvm` command
- [ ] Implement `sandbox exec --tier` with trust level flag
- [ ] Implement `sandbox list --tier` filtering
- [ ] Add `--trust-level` flag to all sandbox commands

### Configuration

#### WP-050: Configuration System
**Status**: ⚠️ Partial
**Priority**: P2

- [ ] Define configuration schema in `internal/domain/config.go`
- [ ] Implement YAML configuration loading
- [ ] Implement environment variable overrides
- [ ] Add default configuration generation
- [ ] Add configuration validation

### Quality Gates

#### WP-060: Code Quality
**Status**: ⚠️ Partial
**Priority**: P1

- [x] Add Go module configuration in `go.mod`
- [ ] Add `go fmt` to pre-commit
- [ ] Add `go vet` to CI
- [ ] Add `golangci-lint` configuration
- [ ] Add `go build` verification
- [ ] Add `go test` coverage

### Documentation

#### WP-070: Documentation
**Status**: ⚠️ Partial
**Priority**: P1

- [x] Update `README.md` to reflect actual architecture
- [x] Update `SPEC.md` to match code implementation
- [x] Create AgilePlus spec in `AgilePlus/.agileplus/specs/008-nanovms-completion/`
- [x] Create ADR-001: Language Selection
- [x] Create ADR-002: Three-Tier Isolation Architecture
- [ ] Add architecture diagrams
- [ ] Add API documentation
- [ ] Add benchmark documentation

---

## Task Status Summary

| Category | Total | ✅ Done | ⚠️ Partial | 📋 Planned |
|----------|-------|---------|-------------|------------|
| Tier 1 (WASM) | 2 | 0 | 1 | 1 |
| Tier 2 (gVisor) | 2 | 0 | 0 | 2 |
| Tier 3 (Firecracker) | 2 | 0 | 1 | 1 |
| Infrastructure | 3 | 0 | 1 | 2 |
| CLI | 2 | 0 | 1 | 1 |
| Config | 1 | 0 | 0 | 1 |
| Quality | 1 | 0 | 1 | 0 |
| Docs | 1 | 0 | 1 | 1 |
| **Total** | **14** | **0** | **5** | **9** |

## Dependencies

```
WP-001 (WASM) ──► WP-002 (Tool Host)
WP-020 (Firecracker) ──► WP-021 (OCI Integration)
WP-030 (Lima) ──► WP-040 (CLI)
WP-031 (WSL) ──► WP-040 (CLI)
WP-032 (Native) ──► WP-040 (CLI)
```

## Research Notes (2026-04-02)

### Performance Benchmarks
- youki (Rust): 111ms container ops
- runc (Go): 225ms container ops
- crun (C): 47ms container ops
- Firecracker: <125ms startup, <5MB RAM

### Language Strategy
- Rust: VMM core, Firecracker, Cloud Hypervisor
- Zig: Hypervisor adapters, sandbox shims, low-level tooling
- Go: CLI, orchestration, agent integration
- C: Performance-critical paths (crun replacement)

### Exotic Platform Support
- **Apple**: iOS, iPadOS, tvOS, watchOS, visionOS (simulators via Lima/VZ)
- **Android**: Phone, Tablet, Wear OS, Android TV, Automotive (headless via Termux/PRoot)
- **Smart TV**: tvOS, Android TV, Tizen, webOS, Roku, Fire TV
- **Gaming**: PlayStation, Nintendo Switch, Xbox
- **IoT/Embedded**: Raspberry Pi, Pine64, ESP32, FreeRTOS
- **AR/VR**: visionOS, Meta Quest, HoloLens, Magic Leap, SteamVR, SteamOS
- **Desktop**: macOS, Windows, Linux

### Mobile Limitations
- iOS: No local containers (App Store sandbox) → Cloud relay only
- Android: PRoot/FreeBSD jail possible but reduced isolation
- Best mobile strategy: Remote VM delegation with CLI interface
