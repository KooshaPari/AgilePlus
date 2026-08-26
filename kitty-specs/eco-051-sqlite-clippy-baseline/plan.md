# Plan: SQLite Clippy Baseline

1. Capture the scoped all-targets lint failure from the #1032 stack.
2. Apply only the three compiler-suggested guard collapses.
3. Validate scoped lint, tests, formatter, whitespace, and generated index.
4. Open a draft stacked PR and merge only after all parent and hosted gates pass.
