---
name: lint
description: Run clippy with -D warnings + check rustfmt. Equivalent to the CI gate (minus tests).
---

Run, in order:

```sh
~/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings
~/.cargo/bin/cargo fmt --all -- --check
```

Both must succeed. If `cargo fmt --all -- --check` fails with a diff, run `cargo fmt --all` to apply, then re-check. If clippy fails, fix the underlying issue rather than `#[allow]`-ing — only allow when the lint is genuinely wrong for the call site, with a one-line justification comment.
