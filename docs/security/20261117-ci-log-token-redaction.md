# CI log token redaction (2026-11-17)

## What happened

The closure-ledger D38.3 public-surface sweep found three token-shaped values
echoed verbatim into committed CI logs under
`docs/sessions/20261017-state-forward-docket/`. Each is 36 characters
(UUID-shaped, Snyk PR-check identifier shape) and appeared inside a
`https://app.snyk.io/org/<org>/pr-checks/<id>` URL in captured
`gh pr checks` output.

`KooshaPari/AgilePlus` is a PUBLIC repository, so those bytes were public.

## What was changed

Two log files, three lines total, line counts unchanged:

- `docs/sessions/20261017-state-forward-docket/ci_diagnosis_2026-10-18-fox.log` (1 value)
- `docs/sessions/20261017-state-forward-docket/re-entry-2026-10-18/ci-scout-v2-2026-10-18T20-22Z.log` (2 values)

Each value was replaced with the literal placeholder `<REDACTED-ROTATE-ME>`;
surrounding log text is untouched. No other file in the repo matches the
three prefixes as of this commit (verified with `rg --no-ignore --hidden`,
required because these logs are tracked but ignore-rule-shadowed).

## Is this a credential?

Observed shape is a Snyk PR-check UUID inside a public Snyk URL, not
necessarily a Snyk API token (those are 36-char UUIDs too, hence the
classification). Treated as compromised anyway: the classification cannot be
proven from the log line, and the repo is public.

## Not done here (operator actions)

1. **Rotate** any Snyk token that could correspond to these values. Treat them
   as compromised regardless of the UUID-vs-token question.
2. **History purge** if rotation is refused or the values are confirmed
   sensitive. The blobs remain in git history: this is a forward-only
   redaction, no history rewrite was performed or proposed.

## Guard / workflow note

`.gitignore` was intentionally NOT updated: `git ls-files 'docs/sessions/**/*.log'`
reports **13 tracked logs**, and tracked files cannot be ignored. Instead:

- Do not paste raw `gh pr checks` / CI output into committed session logs.
- Before committing under `docs/sessions/**`, run
  `rg --no-ignore -n 'pr-checks/[0-9a-f-]{36}' docs/sessions` and redact hits.
- CI capture scripts should strip URLs with embedded identifiers.
