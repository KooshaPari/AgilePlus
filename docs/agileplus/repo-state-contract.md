# AgilePlus repository-state contract

AgilePlus is tooling. It does not own cross-project state. Each Git repository
owns its AgilePlus records and resolves them from that repository's Git root.

## Layout

```text
<repository>/
├── .agileplus/
│   ├── agileplus.db       # mutable local SQLite state
│   ├── locks/             # runtime coordination, ignored
│   ├── logs/              # runtime diagnostics, ignored
│   └── cache/             # rebuildable cache, ignored
└── docs/
    └── agileplus/
        └── <feature-slug>/
            ├── meta.json
            ├── status.md
            ├── audit.jsonl
            └── work-packages/
                └── <work-package-id>.json
```

Files under `docs/agileplus/` are Git-tracked human and review artifacts.
They are materialized from the repository-local source of truth; direct edits
are overwritten on the next materialization. Files under `.agileplus/` are
machine state and must not be committed, except for an intentionally tracked
configuration file explicitly documented by the repository.

## Migration and rollback

Do not infer a destination repository from a feature name. A central record may
be imported only after a reviewed owner mapping identifies its repository. The
export is retained with its checksum and verified audit-chain evidence until the
destination import is verified. If verification fails, stop the migration,
preserve both copies, and restore the repository from its pre-import Git commit;
do not delete or overwrite the central evidence.
