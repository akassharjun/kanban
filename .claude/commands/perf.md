---
name: perf
description: Run the cold-start performance smoke (release build, takes ~30s).
---

Run:

```sh
~/.cargo/bin/cargo test --workspace --release -- --ignored
```

This runs `crates/kanban-cli/tests/perf.rs` which is `#[ignore]`'d by default because timing is noisy on shared CI. Locally on a quiet machine, expect cold-start p95 well under the 50ms target.

If it fails, check what else is competing for CPU/disk before assuming a regression.
