# DINO Quality Gate

## Gate: dino_quality_gate

### Scope
All changes to the Dino submodule (`Dino/`) and its worktrees (`Dino-wtrees/`).

### Required Checks
| Check | Severity | Command |
|-------|----------|---------|
| CI Build | FAIL | Dino repo must have green CI on the referenced SHA |
| Fuzz Testing | FAIL | `fuzz.yml` must pass (nightly) |
| Benchmarks | WARN | `benchmarks.yml` must not regress |
| Pack Validation | FAIL | `pack-validation.yml` must pass |
| Secret Scan | FAIL | TruffleHog + CodeQL must be clean |
| Type Coverage | WARN | .NET type coverage must not decrease |
| Game Launch | FAIL | `game-launch-validation.yml` must pass |

### Submodule Rules
- Dino submodule SHA must be updated via PR only (no direct pushes)
- Worktree changes follow standard `Dino-wtrees/<topic>/` convention
- Minimum 1 approval required from `@KooshaPari`

### Scorecard Threshold
- Overall pillar grade must be >= C (60/100) for merge
- Any F-grade pillar blocks merge unless explicitly waived

### Monitoring
- CI status tracked in `ci-status-badges.yml`
- Performance tracked in `benchmarks.yml`
- Security tracked in `scorecard.yml` + `trufflehog.yml`

### Derived from
- Dino 30-pillar audit (2026-08-18): C (61/100)
- AgilePlus integration spec (PR#972)
