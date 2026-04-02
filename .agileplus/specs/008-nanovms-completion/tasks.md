# Tasks: NanoVMS — Infrastructure Layer Completion

**Spec**: 008-nanovms-completion
**Status**: Draft
**Repository**: nanovms

## Work Packages

### Infrastructure Adapters

#### WP-001: Lima Adapter (macOS)
**Status**: ✅ Partial
**Priority**: P1

- [x] Define VMAdapter interface in `internal/ports/ports.go`
- [x] Implement Lima adapter in `internal/adapters/mac/mac.go`
- [ ] Add Lima VM lifecycle (create, start, stop, delete)
- [ ] Add Lima networking (port forwarding)
- [ ] Add Lima filesystem mounts
- [ ] Add unit tests for Lima adapter

#### WP-002: WSL Adapter (Windows)
**Status**: ⚠️ Stub
**Priority**: P1

- [ ] Implement WSL adapter in `internal/adapters/windows/windows.go`
- [ ] Add WSL VM lifecycle (create, start, stop, delete)
- [ ] Add WSL networking (port forwarding)
- [ ] Add unit tests for WSL adapter

#### WP-003: Native Adapter (Linux)
**Status**: ⚠️ Stub
**Priority**: P1

- [ ] Implement Native adapter in `internal/adapters/linux/linux.go`
- [ ] Add KVM/HyperKit/Hyper-V detection
- [ ] Add VM lifecycle (create, start, stop, delete)
- [ ] Add unit tests for Native adapter

#### WP-004: MicroVM Adapter (Firecracker)
**Status**: 📋 Planned
**Priority**: P2

- [ ] Implement Firecracker adapter in `internal/adapters/microvm/microvm.go`
- [ ] Add Firecracker configuration (kernel, initrd)
- [ ] Add VM lifecycle (create, start, stop, delete)
- [ ] Add microVM networking (tap device)
- [ ] Add unit tests for MicroVM adapter

#### WP-005: WASM Adapter
**Status**: ⚠️ Stub
**Priority**: P2

- [ ] Implement WASM adapter in `internal/adapters/wasm/wasm.go`
- [ ] Add Wasmtime integration
- [ ] Add host function bindings
- [ ] Add unit tests for WASM adapter

### Sandbox Isolation

#### WP-010: Native Sandbox Adapters
**Status**: ✅ Implemented
**Priority**: P1

- [x] Define SandboxType and NativeSandboxType in `internal/domain/sandbox.go`
- [x] Implement bwrap adapter in `internal/adapters/sandbox/bwrap.go`
- [x] Implement firejail adapter in `internal/adapters/sandbox/firejail.go`
- [x] Implement unshare adapter in `internal/adapters/sandbox/unshare.go`
- [ ] Add sandbox capability detection
- [ ] Add unit tests for sandbox adapters

#### WP-011: Process-Level Sandboxes
**Status**: 📋 Planned
**Priority**: P2

- [ ] Implement gVisor (runsc) adapter
- [ ] Implement landlock restrictions
- [ ] Add seccomp profiles
- [ ] Add unit tests

### CLI Interface

#### WP-020: Core CLI Commands
**Status**: ⚠️ Partial
**Priority**: P1

- [x] Define CLI structure in `cmd/nanovms/main.go`
- [ ] Implement `create` command
- [ ] Implement `delete` command
- [ ] Implement `list` command
- [ ] Implement `exec` command
- [ ] Implement `probe` command
- [ ] Add shell completion

#### WP-021: Sandbox CLI Commands
**Status**: ⚠️ Partial
**Priority**: P1

- [ ] Implement `sandbox create` command
- [ ] Implement `sandbox delete` command
- [ ] Implement `sandbox list` command
- [ ] Implement `sandbox exec` command

### Configuration

#### WP-030: Configuration System
**Status**: ⚠️ Partial
**Priority**: P2

- [ ] Define configuration schema in `internal/domain/config.go`
- [ ] Implement YAML configuration loading
- [ ] Implement environment variable overrides
- [ ] Add default configuration generation
- [ ] Add configuration validation

### Quality Gates

#### WP-040: Code Quality
**Status**: ⚠️ Partial
**Priority**: P1

- [x] Add Go module configuration in `go.mod`
- [ ] Add `go fmt` to pre-commit
- [ ] Add `go vet` to CI
- [ ] Add `golangci-lint` configuration
- [ ] Add `go build` verification
- [ ] Add `go test` coverage

### Documentation

#### WP-050: Documentation
**Status**: ⚠️ Partial
**Priority**: P1

- [x] Update `README.md` to reflect actual architecture
- [x] Update `SPEC.md` to match code implementation
- [x] Create AgilePlus spec in `AgilePlus/.agileplus/specs/008-nanovms-completion/`
- [ ] Add architecture diagrams
- [ ] Add API documentation

---

## Task Status Summary

| Category | Total | ✅ Done | ⚠️ Partial | 📋 Planned |
|----------|-------|---------|-------------|------------|
| Adapters | 5 | 1 | 2 | 2 |
| Sandbox | 2 | 1 | 0 | 1 |
| CLI | 2 | 0 | 1 | 1 |
| Config | 1 | 0 | 0 | 1 |
| Quality | 1 | 0 | 1 | 0 |
| Docs | 1 | 0 | 2 | 1 |
| **Total** | **12** | **2** | **6** | **6** |

## Dependencies

- WP-001 (Lima) depends on: WP-020 (Core CLI)
- WP-010 (Sandbox) depends on: WP-020 (Core CLI)
- WP-004 (MicroVM) depends on: Firecracker binary available

## Notes

- Stub implementations in `windows/windows.go`, `linux/linux.go`, and `wasm/wasm.go` need full implementation
- Code has pre-existing type-definition issues in `internal/domain/sandbox.go` (line 224: unused variable)
- Windows adapter has unused variable on line 11
