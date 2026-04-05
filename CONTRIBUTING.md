# Contributing to Phenotype

Standardized Phenotype enterprise contribution guidelines.

## PR Requirements

All pull requests must include the following before review:

### 1. Visual Evidence (REQUIRED)

Every PR must include embedded visual evidence showing the change:

| Type | Format | Example |
|------|--------|---------|
| UI/Frontend | GIF/Screenshot | Drag-and-drop into PR description |
| CLI/Terminal | GIF or asciinema | `vhs`, `asciinema` recordings |
| API/Backend | Screenshot of response | Insomnia/Postman screenshot |

**Why:** Visual evidence lets stakeholders understand changes without running the application. It creates a browsable visual history.

### 2. Completed Specification (REQUIRED)

Feature PRs must reference a completed kitty-spec:

- [ ] Spec exists at `kitty-specs/<feature-id>/`
- [ ] All work packages in `spec.md` marked complete
- [ ] Implementation aligns with spec acceptance criteria

Create a new spec: `agileplus specify --title "<feature>" --description "<desc>"`

### 3. Documentation Page (REQUIRED)

Every feature needs a docs page at `docs/<category>/<feature-id>.md`:

- [ ] Page includes: description, usage, visual examples, API reference
- [ ] Page linked in VitePress sidebar (`docs/.vitepress/site-meta.mjs`)
- [ ] Cross-referenced in spec's documentation section

See [Documentation Guide](docs/contributing/documentation.md) for details.

### 4. Changelog Entry (REQUIRED for user-facing changes)

- [ ] Entry added to `CHANGELOG.md`
- [ ] Entry includes visual preview link: `![feature](docs/assets/gifs/feature.gif)`
- [ ] Follows CalVer: `YEAR.MONTH(WAVE).PATCH`

---

## Development Workflow

1. **Create spec**: `agileplus specify --title "<feature>" --description "<desc>"`
2. **Work in feature worktree**: `repos/worktrees/<project>/<category>/<branch>`
3. **Complete all spec work packages**
4. **Create documentation** page with GIF/screenshot at `docs/<category>/<feature>.md`
5. **Update CHANGELOG.md** with visual entry
6. **Validate PR readiness**:
   ```bash
   ./bin/pvalidate --feature <id>
   ```
7. **Submit PR** with:
   - Screenshot/GIF in PR description
   - All checkboxes checked
   - Link to spec and docs

## PR Requirements Quick Check

Before submitting, run the validator:

```bash
./bin/pvalidate --feature <id>
```

This checks:
- ✅ Spec exists and WPs complete
- ✅ Documentation page exists
- ✅ CHANGELOG entry present
- ✅ Visual assets referenced

## Standards

- Follow the branch-based delivery protocol in CLAUDE.md
- Ensure all CI policy gates are green before requesting review
- Write worklogs for research, decisions, and significant findings

## Related

- [PR Requirements Policy](GOVERNANCE_PR_REQUIREMENTS.md)
- [Documentation Guide](docs/contributing/documentation.md)
- [Recording Visuals](docs/contributing/recording-visuals.md)
- [pvalidate Tool](docs/contributing/pvalidate.md)
