# A4 no-spawn test runner

Run from a profile-free PowerShell session:

```powershell
pwsh -NoProfile -File .a4-shim/run.ps1
```

The runner compiles `shim.rs` into native Windows `claude.exe` and
`codex.exe` programs that immediately exit 1. The adjacent `.cmd` and
extensionless shell scripts also exit 1. Generated executables and debug
symbols stay local. Native executables are necessary for Rust subprocess
lookup on Windows; shell-only interception is insufficient.

The shim directory comes first on PATH, followed by the pinned Protoc bin
at `.a4-receipts/protoc/bin`. The runner checks executable resolution and
both version-command exits before running locked workspace tests with two
build jobs and test debug symbols disabled. Full test output and its exit
code are saved in `.a4-receipts/`.

Use `-LogName test-shimmed-final.log` for the confirmation run. PowerShell
profiles must remain disabled: profile functions can cache installed CLI
paths and override PATH lookup. These shims guard PATH-based subprocess
calls; they do not intercept absolute executable paths or tests that replace
PATH entirely. Review those separately before running new tests.

Some integration tests prepend purpose-built Unix shell fixtures. Windows
subprocess lookup can bypass those fixtures and reach these exit-1 shims.
Such failures establish a fixture portability problem, not a need to run a
real model. Keep those failures enabled and report them separately from any
operator-approved real-binary skip list.
