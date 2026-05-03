---
name: check
description: Fast compile check across the workspace (no tests, no clippy).
---

Run `~/.cargo/bin/cargo check --workspace --all-targets`.

Use this for tight iteration on type/borrow errors. Once it's green, run `/lint` and `/test` before committing.
