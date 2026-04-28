# OpenSSF Best Practices Posture

AgilePlus tracks OpenSSF Scorecard findings in GitHub code scanning. The
`CIIBestPracticesID` finding is not satisfied by repository files alone; the
Scorecard check queries the OpenSSF Best Practices badge service for a project
record associated with this repository URL.

## Current State

- Repository URL: `https://github.com/KooshaPari/AgilePlus`
- Scorecard workflow: `.github/workflows/scorecard.yml`
- Fuzzing workflow: `.github/workflows/fuzz.yml`
- Security policy: `SECURITY.md`
- SBOM: `docs/security/sbom.json`

## Badge Application Checklist

1. Register AgilePlus at `https://www.bestpractices.dev/`.
2. Associate the project record with `https://github.com/KooshaPari/AgilePlus`.
3. Complete the passing-level checklist using this repository's existing
   governance, CI, security policy, license, SBOM, and fuzzing evidence.
4. Add the issued badge URL to `README.md` after the badge service returns a
   project identifier.

Do not add a README badge before the OpenSSF service issues an AgilePlus project
identifier; an unissued badge would be misleading and would not satisfy the
Scorecard check.
