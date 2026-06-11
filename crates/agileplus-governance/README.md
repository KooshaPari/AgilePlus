# agileplus-governance

Governance system for release channels, audit logging, policy checks, and rate limiting.

## Public API Index

- `GovernanceClient`, `GovernanceConfig`, `GovernanceError`, `Result`.
- Release APIs: `ReleaseChannel`, `ChannelMetadata`, `PromotionRequest`, `Channel` alias.
- Policy APIs: `PolicyCheck`, `PolicyContext`, `PolicyEngine`, `PolicyResult`.
- Operations: `AuditLogger`, `RateLimiter`, and types from `types`.

## Validation

```bash
cargo test -p agileplus-governance
```

