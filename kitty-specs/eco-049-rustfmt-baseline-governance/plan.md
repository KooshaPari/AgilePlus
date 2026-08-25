# Plan: Rustfmt Baseline Governance

1. Create an isolated branch from the current `origin/main` and preserve the
   existing formatter candidate unchanged.
2. Apply the reviewed one-commit formatter delta and register this spec.
3. Run focused local formatting, whitespace, and governance-index checks.
4. Open a draft PR with the required governance metadata and await hosted CI.
5. Reconcile independent baseline failures before requesting approval or merge.
