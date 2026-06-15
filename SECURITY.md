# Security Policy — AgilePlus

AgilePlus orchestrates coding agents that operate against your
repositories and CI. The blast radius of a compromised AgilePlus
install is the entire fleet it drives. Security is therefore a
primary feature, not an afterthought. This document explains how to
report a vulnerability, what to expect from us, and how we handle
disclosure.

---

## Supported Versions

AgilePlus follows [semantic versioning](https://semver.org/). The
following table lists the release lines that currently receive
security updates:

| Version line | Status            | Security fixes | End-of-life  |
|--------------|-------------------|----------------|--------------|
| `0.14.x`     | **Active**        | ✅ Backported  | TBA          |
| `0.13.x`     | Maintenance       | ✅ Until 2026-08-31 | 2026-08-31 |
| `0.12.x`     | End of life       | ❌ No longer receiving updates | 2025-12-31 |
| `0.11.x`     | End of life       | ❌ No longer receiving updates | 2025-06-30 |
| `< 0.11`     | End of life       | ❌             | n/a          |
| `< 0.10`     | Not supported     | ❌             | n/a          |

> **Recommendation:** always run the latest patch release of the
> latest two minor lines. We will publish a CVE and a GHSA for every
> security fix landed on `main`.

We take the security of our projects seriously. This document explains how to
report a vulnerability, what to expect from us, and how we coordinate
disclosure.

## Supported Versions

The following versions of this project receive security updates:

| Version | Supported          |
| ------- | ------------------ |
| Latest  | :white_check_mark: |
| < Latest | :x:                |

Only the most recent release line (and `main` / the default branch) is
actively maintained. Older releases will not receive backported patches
unless a critical CVE is involved; please upgrade.

## Reporting a Vulnerability

**Please do not file a public GitHub issue for security vulnerabilities.**

Report privately via one of the following channels (in order of preference):

1. **GitHub Security Advisories** — open a
   [private security advisory](../../security/advisories/new) on this
   repository. This routes directly to the maintainers and keeps the
   discussion out of the public bug tracker.
2. **Email** — send a report to the maintainer listed in
   [`CODEOWNERS`](../CODEOWNERS) / repository metadata. Use this channel for
   vulnerabilities you believe are time-sensitive or that touch multiple
   repositories in the organization.
3. **Phenotype Org security.txt** — for cross-repository or organization-level
   security concerns, refer to the security.txt published at
   `/.well-known/security.txt` on the Phenotype domain and contact
   `private-vuln-reporting@phenotype.local`.

A good report includes:

- A clear description of the vulnerability and its impact
- A reproducer (proof-of-concept, curl transcript, test case, or steps to
  reproduce)
- Affected versions, commit SHAs, and components
- Any known mitigations or workarounds
- Whether you intend to disclose publicly and on what timeline

Please **do not** include real user data, production secrets, or exploit
payloads in the initial report. A redacted, minimal reproducer is
sufficient.

## Response Timeline

We aim to handle every report in a predictable, time-bounded way:

| Stage                                | Target SLA       |
| ------------------------------------ | ---------------- |
| Initial acknowledgement              | within 2 business days |
| Triage & impact assessment           | within 7 business days |
| Status update to reporter            | at least every 7 business days until resolution |
| Patch released for critical issues   | within 30 days of triage confirmation |
| Patch released for high / medium     | within 60–90 days, depending on complexity |
| CVE / GHSA assignment                | coordinated with the reporter |

A "critical" issue is one that is remotely exploitable with low
attacker effort and significant impact (auth bypass, RCE, secret
exposure, supply-chain compromise). Severity is determined at our
discretion using the CVSS v3.1 base score as a guide.

If we are unable to meet a timeline, we will tell you — silence does
not mean your report is being ignored.

## Disclosure Policy

We follow **coordinated disclosure** (a.k.a. responsible disclosure):

- The reporter agrees to keep the vulnerability confidential until a
  fix is published or a mutually agreed embargo expires.
- We agree to credit the reporter in the public advisory (unless they
  prefer to remain anonymous) and to set a release date that is no
  later than **90 days** after the initial acknowledgement, except
  when an earlier release is warranted (e.g. active exploitation) or
  when the reporter asks for additional time.
- We publish a GitHub Security Advisory plus a CVE (via GHSA / MITRE
  CNA) describing the issue, affected versions, severity, and fix.
- We do not pursue legal action against researchers who make a good
  faith effort to follow this policy.

Out-of-band disclosures (blog posts, tweets, conference talks before a
fix is ready) are not authorized and may put users at risk. If you are
unsure whether something is in scope, ask first.

## Acknowledgements

We are grateful to the security researchers and community members who
help keep this project safe. Reporters who follow this policy will be
credited in the published advisory (with their consent) and may be
listed below in chronological order.

- _No reports have been acknowledged yet._

A full historical list of advisories for this project is available
under the repository's **Security** tab → "Advisories".

---

If you believe you have found a security issue in a different
Phenotype repository, please follow that repository's `SECURITY.md` or
contact the organization security team at the address listed in
`/SECURITY.md` of the relevant project.
