# Finish path and acceptance dependencies

This is a proposed execution plan, not authorization to implement or deploy.
Reuse existing PRs before creating competing work. Freeze source, artifact,
configuration, subject repository and evidence identity for each acceptance run.

| ID | Work package | Depends on | Required acceptance |
|---|---|---|---|
| A | Reconcile candidate, owners and supported profiles | None | One exact source/PR matrix; dirty work preserved; supported CLI/MCP/web/desktop profiles declared |
| B | Truthful install, health and MCP route | A | Clean install and canonical health/dashboard roundtrip; dependency failure reported honestly; advertised stubs implemented or explicitly removed from supported profile |
| C | Durable core lifecycle | A | Unrelated repo: ask/spec/docs/task trace, execution/evidence, canonical acceptance and receipt; restart/replay/wrong-root/concurrency/stale evidence controls |
| D | Operator workflow and page completion | B,C | Every supported page has real data and usable empty/error/success states; receipt history and request identity survive restart |
| E | Assurance and source-bound packaging | A | Required tests fail closed; declared coverage enforced with negative controls; artifacts proven built from candidate and verified after install |
| F | CVP real-project pilot | B,C,E | One operator completes agreed valuable lifecycle; defects and recovery recorded; no fabricated success; receipt source-bound |
| G | MVP daily-work completeness | D,F | Supported everyday workflows, help/onboarding, errors, repository isolation and repeat pilot meet frozen UX criteria |
| H | Deployment, telemetry and recovery | E,F | Authorized target: auth/proxy/API origin, health, metrics/traces, backup/restore/reboot and desktop restart receipts |
| I | Finite mature contract | G,H | All 12 journeys/92 cases executed against exact candidate with adversarial controls; assurance matrix complete; release decision explicit |

```text
A -> B --+-> F -> G --+
  -> C --+   |       +-> I
  -> E ------+-> H --+
B + C -> D -> G
```

CVP means the smallest product that proves valuable work on a real subject with
correct durable state. MVP here means a supported minimum daily-use product,
including useful operator workflows; it is broader than the first pilot. Neither
label waives canonical acceptance, provenance or error truthfulness.

## Stop rules

Any false-success, wrong repository mutation, forged evidence, acceptance bypass or
unrecoverable state loss blocks that candidate. Repair forward and rerun affected
controls. A source fix, a test pass, a hosted job, an installed artifact and a deployed
journey are distinct receipts. Do not substitute one for another.

## Finish criteria

Full finite-goal completion requires implemented supported interfaces, executed
journeys, a real project witness, a reproducible install/recovery path and an explicit
release receipt. Jira/Linear/Plane superiority needs a separate frozen comparative
study and remains outside this finite finish definition. Access/public ingress and
private tailnet acceptance are separate gates; deployment requires exact target authority.
