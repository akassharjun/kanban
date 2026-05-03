---
name: test
description: Run the full kanban test suite (unit + integration + property + CLI snapshot).
argument-hint: "[optional: --test NAME or -p kanban-core etc.]"
---

Run `~/.cargo/bin/cargo test --workspace $ARGUMENTS` and surface failures clearly.

If a test fails, show the failure (the agent will see it in stdout). Don't proactively re-run — the user wants the failure signal first.

If `$ARGUMENTS` is empty, run the full workspace suite. The current target is 123 tests passing, 1 ignored (perf — runs only with `--ignored`).
