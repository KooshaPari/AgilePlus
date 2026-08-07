# DAG / work breakdown

```text
inspect -> verify callers/warnings -> patch -> diff-check -> narrow validation
```

1. Inspect current files and process/disk state: complete.
2. Apply seven-file source/workflow/test patch: complete.
3. Add session evidence docs: complete.
4. Run focused Cargo/Python checks and actionlint: complete where dependencies are available;
   hosted coverage/governance remains external follow-up.
5. Commit, push, and preserve with Airlock snapshot: parent-owned.
