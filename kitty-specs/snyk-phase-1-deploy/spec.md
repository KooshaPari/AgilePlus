# Deploy Snyk Security Scanning (Pilot: phenotype-infrakit + AgilePlus)

## Overview
Configure Snyk vulnerability detection, GitHub Actions workflows, .snyk policy files, and GitHub Secrets integration. Pilot phase tests cost impact on 2 repos before rolling to all 30.

## Objectives
- Enable automated dependency and code vulnerability scanning via Snyk
- Integrate Snyk checks into GitHub Actions CI/CD pipeline
- Configure policies (.snyk files) for both pilot repositories
- Validate cost impact and alert sensitivity before organization-wide rollout
- Establish baseline security posture and remediation workflow

## Scope
**Pilot Phase**: 2 repositories
- phenotype-infrakit (Rust monorepo with 24 crates)
- AgilePlus (Rust + multi-language crate collection)

**Out of Scope (Phase 2+)**:
- Rolling out to remaining 28 repositories
- Snyk container scanning for Docker images
- Advanced SAST (Static Application Security Testing)
- Snyk Code plan features (code vulnerabilities)

## Success Criteria
- [ ] Snyk API token configured in GitHub organization secrets
- [ ] GitHub Actions workflow deployed to both pilot repos
- [ ] .snyk policy files created for phenotype-infrakit and AgilePlus
- [ ] First full scan completed without errors
- [ ] At least 1 vulnerability detected and triaged (or confirmed 0 vulns)
- [ ] Cost per scan documented (estimated: $0.50-$2.00/scan for OSS)
- [ ] Baseline alert rules configured (fail on critical/high severity)
- [ ] Team notified of findings; remediation plan drafted

## Deliverables
1. Snyk organization account setup (with GitHub app integration)
2. GitHub Actions workflow file: `.github/workflows/snyk-security-scan.yml`
3. `.snyk` policy files (phenotype-infrakit and AgilePlus roots)
4. GitHub Secrets configuration guide (SNYK_TOKEN, SNYK_ORG_ID)
5. Vulnerability triage report (initial scan results)
6. Phase 2 rollout plan (deployment to all 30 repos)

## Estimated Effort
- Snyk account setup + GitHub app integration: 30 min
- Workflow implementation + testing: 2 hours
- Policy tuning per repo: 1 hour each (2 repos) = 2 hours
- Initial scan + triage: 1 hour
- Documentation + rollout plan: 1 hour
- **Total: ~6.5 hours**

## Risk & Mitigation
| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| High false positive rate | Medium | Blocks CI | Tune `.snyk` policies aggressively; use ignore rules |
| GitHub Actions billing overrun | Low | Cost increase | Monitor monthly cost, set max-run limits in workflow |
| Snyk API rate limiting | Low | Workflow failure | Implement exponential backoff; cache results |
| Integration conflict with existing CI | Medium | Build failure | Test in separate workflow first; keep non-blocking initially |

## Dependencies
- GitHub organization settings (ability to add secrets + apps)
- Snyk account (create at snyk.io, free tier eligible)
- Each pilot repo has maintained Cargo.lock and package.json (for SCA)

## Related Specs
- Phase 1 (this): Pilot on 2 repos
- Phase 2 (future): Rollout to all 30 repos in AgilePlus ecosystem
- Phase 3 (future): Advanced SAST + container scanning

## Notes
- Use Snyk free tier for initial pilot (includes dependency scanning + 2GB analysis storage)
- GitHub integration is automatic via OAuth; no manual token setup required for read-only
- Pilot repos are representative: 1 pure Rust (infrakit), 1 multi-language (AgilePlus)
