# Feature Specification: NanoVMS — Nano Virtual Machine Services

**Feature Branch**: `008-nanovms-completion`
**Created**: 2026-04-02
**Status**: Draft
**Mission**: infrastructure
**Repository**: nanovms

## Overview

NanoVMS (Nano Virtual Machine Services) provides a lightweight, headless VM abstraction for agent-driven development workflows. It implements a **two-level abstraction**:

1. **Infrastructure Layer** (CURRENT): VM runtimes (Native, Lima, WSL, MicroVM, WASM)
2. **Platform Layer** (PLANNED): Target platforms (iOS, Android, tvOS, etc.)

This spec covers the **Infrastructure Layer** completion — implementing the VM adapters and sandbox isolation system.

### Architecture

```
nanovms/
├── cmd/nanovms/              # CLI entry point
├── internal/
│   ├── adapters/             # Infrastructure implementations
│   │   ├── mac/             # Lima/VZ adapter (macOS)
│   │   ├── windows/         # WSL adapter (Windows)
│   │   ├── linux/           # Native/KVM adapter (Linux)
│   │   ├── microvm/         # Firecracker adapter (all platforms)
│   │   ├── wasm/            # WASM runtime adapter
│   │   └── sandbox/         # Isolation adapters (bwrap, gVisor, etc.)
│   ├── domain/               # Core models (Sandbox, VMFlavor, VMConfig)
│   └── ports/                # Interface definitions (VMAdapter)
└── pkg/                      # Public API library
```

## Core Types (IMPLEMENTED)

### VMFlavor — Infrastructure Layer

```go
// internal/domain/sandbox.go

type VMFlavor string

const (
    VMFlavorNative  VMFlavor = "native"   // HyperKit, Hyper-V, KVM
    VMFlavorLima    VMFlavor = "lima"     // Lima with vz driver (macOS)
    VMFlavorWSL     VMFlavor = "wsl"      // Windows Subsystem for Linux
    VMFlavorMicroVM VMFlavor = "microvm"  // Firecracker microVM
    VMFlavorWasm    VMFlavor = "wasm"     // WebAssembly runtime
)
```

### SandboxType — Isolation Levels

```go
type SandboxType string

const (
    SandboxTypeVM        SandboxType = "vm"        // Full virtual machine
    SandboxTypeContainer SandboxType = "container" // Container isolation
    SandboxTypeWasm      SandboxType = "wasm"      // WebAssembly isolation
    SandboxTypeProcess   SandboxType = "process"   // gVisor, landlock
    SandboxTypeNative    SandboxType = "native"    // bwrap, firejail, namespaces
)
```

## VM Tiers

| Tier | Technology | Use Case | Overhead |
|------|------------|----------|----------|
| 1 - Native | HyperKit, Hyper-V, KVM | Production parity | Highest |
| 2 - Lima/WSL | Lima/VZ, WSL2 | Daily development | Medium |
| 3 - MicroVM | Firecracker, Cloud Hypervisor | CI/CD, isolation | Lowest |
| 4 - WASM | Wasmtime, Wasmer | Lightweight execution | Minimal |

## Security Model

| Tier | Isolation | Performance | Use Case |
|------|-----------|-------------|----------|
| Native VM | VT-x/AMD-V | ★★★★☆ | Production testing |
| MicroVM | Firecracker | ★★★★★ | Agent sandboxing |
| Container | namespaces/cgroups | ★★★★★ | Local dev |
| WASM | Bytecode isolation | ★★★★★ | Lightweight workloads |

Sandbox layers (gVisor, landlock, seccomp, WASM) can be stacked for additional security.

## User Scenarios & Testing

### User Story 1 — Create Lima VM on macOS (Priority: P1)

A developer on macOS wants to create a headless Ubuntu VM for development.

**Acceptance Scenarios**:

1. **Given** macOS with Lima installed, **When** the user runs `nanovms create dev --vm-flavor lima --image ubuntu:22.04`, **Then** a Lima VM is created with the specified image.

2. **Given** a running Lima VM, **When** the user runs `nanovms exec dev -- ls /`, **Then** the command executes inside the VM and returns output.

3. **Given** a running Lima VM, **When** the user runs `nanovms delete dev`, **Then** the VM is stopped and deleted.

---

### User Story 2 — Sandbox Isolation with bwrap (Priority: P1)

An agent needs to run untrusted code in an isolated sandbox.

**Acceptance Scenarios**:

1. **Given** a Linux system with bubblewrap installed, **When** the user runs `nanovms sandbox create --sandbox-type native --native-type bwrap`, **Then** a bwrap sandbox is created with namespace isolation.

2. **Given** a running bwrap sandbox, **When** untrusted code attempts to access `/etc/shadow`, **Then** the access is denied and logged.

3. **Given** a running bwrap sandbox, **When** the user runs `nanovms sandbox delete`, **Then** the sandbox process is terminated.

---

### User Story 3 — Firecracker MicroVM (Priority: P2)

A developer needs a lightweight, fast-booting VM for CI/CD.

**Acceptance Scenarios**:

1. **Given** a Linux system with KVM access, **When** the user runs `nanovms create ci --vm-flavor microvm --image alpine:latest`, **Then** a Firecracker microVM is created.

2. **Given** a Firecracker microVM, **When** the user measures boot time, **Then** it boots in under 500ms.

3. **Given** a Firecracker microVM, **When** the user deletes it, **Then** all resources are freed immediately.

---

### User Story 4 — WASM Runtime (Priority: P2)

An agent needs to run lightweight, sandboxed code execution.

**Acceptance Scenarios**:

1. **Given** WASM support installed, **When** the user runs `nanovms wasm run hello.wasm`, **Then** the WASM module executes in a sandboxed environment.

2. **Given** a WASM module with imports, **When** the user provides host functions, **Then** the WASM module can call them securely.

---

### User Story 5 — Multi-Platform Support (Priority: P3)

The system must work across macOS, Windows, and Linux.

**Acceptance Scenarios**:

1. **Given** macOS, **When** the system probes capabilities, **Then** it detects Lima/VZ and enables Lima adapter.

2. **Given** Windows with WSL2, **When** the system probes capabilities, **Then** it detects WSL2 and enables WSL adapter.

3. **Given** Linux with KVM, **When** the system probes capabilities, **Then** it detects KVM and enables Native adapter.

## Requirements

### Functional Requirements

**Core Infrastructure:**
- **FR-001**: System MUST provide a `VMAdapter` interface that abstracts VM runtime implementations.
- **FR-002**: System MUST implement the Lima adapter for macOS using the vz driver.
- **FR-003**: System MUST implement the WSL adapter for Windows using WSL2.
- **FR-004**: System MUST implement the Native adapter for Linux using KVM/HyperKit/Hyper-V.
- **FR-005**: System MUST implement the MicroVM adapter using Firecracker.
- **FR-006**: System MUST implement the WASM adapter using Wasmtime.

**Sandbox Isolation:**
- **FR-007**: System MUST provide sandbox isolation using bwrap on Linux.
- **FR-008**: System MUST provide sandbox isolation using firejail on Linux.
- **FR-009**: System MUST provide sandbox isolation using gVisor (runsc) as an alternative.
- **FR-010**: System MUST support stacking sandbox layers (e.g., bwrap + gVisor).

**CLI Interface:**
- **FR-011**: System MUST provide a CLI with commands: `create`, `delete`, `list`, `exec`, `sandbox`, `probe`.
- **FR-012**: System MUST provide a `probe` command that detects system capabilities and available VM adapters.
- **FR-013**: System MUST provide an `exec` command that runs commands inside a VM or sandbox.

**Configuration:**
- **FR-014**: System MUST support YAML configuration files for VM and sandbox defaults.
- **FR-015**: System MUST support environment variable overrides for configuration.

### Non-Functional Requirements

- **NFR-001**: Lima VM creation MUST complete in under 30 seconds.
- **NFR-002**: Firecracker microVM boot time MUST be under 500ms.
- **NFR-003**: WASM module execution MUST have near-zero overhead compared to native.
- **NFR-004**: All operations MUST be idempotent where possible.
- **NFR-005**: Error messages MUST be actionable and include recovery hints.

## Planned: Platform Target Layer (ROADMAP)

After the Infrastructure Layer is complete, the Platform Target Layer will be implemented:

### Apple Platforms

| Target | Simulator | Infrastructure | Priority |
|--------|-----------|----------------|----------|
| iOS | iPhone, iPad | Lima/VZ | P2 |
| iPadOS | iPad | Lima/VZ | P2 |
| tvOS | Apple TV | Lima/VZ | P2 |
| watchOS | Apple Watch | Lima/VZ | P3 |
| visionOS | Vision Pro | Lima/VZ | P2 |

### Android Ecosystem

| Target | Emulator | Infrastructure | Priority |
|--------|----------|----------------|----------|
| Phone | Android Emulator | WSL2/Lima | P2 |
| Tablet | Various | WSL2/Lima | P2 |
| Wear OS | Wear device | Remote stream | P3 |
| Android TV | TV emulator | WSL2/Lima | P2 |
| Automotive | Auto emulator | WSL2/Lima | P3 |

### AR/VR

| Target | Runtime | Infrastructure | Priority |
|--------|---------|----------------|----------|
| visionOS | Xcode | Lima/VZ | P2 |
| Meta Quest | Meta Horizon | Remote stream | P3 |
| SteamVR | SteamVR | Proton/Wine | P3 |
| SteamOS | ChimeraOS | QEMU | P3 |
| HoloLens | HoloLens Emulator | Hyper-V | P3 |

## Success Criteria

- **SC-001**: All 5 VM adapters (Lima, WSL, Native, MicroVM, WASM) implement the VMAdapter interface.
- **SC-002**: The CLI can create, list, exec, and delete VMs on each supported platform.
- **SC-003**: Sandbox isolation blocks unauthorized filesystem and network access.
- **SC-004**: The `probe` command correctly detects available VM runtimes.
- **SC-005**: All quality gates pass: `go fmt`, `go vet`, `go build`, `go test`.

## Status Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Implemented and tested |
| ⚠️ | Partial implementation (stub) |
| 📋 | Planned (not started) |
| ❌ | Not supported |

---

*This spec covers the Infrastructure Layer (v1). Platform Target Layer (v2) is planned for future releases.*
