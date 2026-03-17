# Phase 4: CI/CD Pipeline + Cross-Platform Distribution

## Context

Phase 1 produced a single `kanban` binary with SQLite as the default backend. Phase 4 automates building and distributing this binary for macOS and Linux via GitHub Releases and Homebrew.

## What Gets Built

The full Tauri app (GUI + CLI + MCP) as a single `kanban` binary for:
- macOS arm64 (Apple Silicon)
- macOS x86_64 (Intel)
- Linux x86_64 (Ubuntu/Debian)

## CI Pipeline

**Trigger:** Push a git tag matching `v*` (e.g., `v0.1.0`).

### Jobs

1. **test** — Ubuntu runner. `cargo test` + `npm run test:run`. Gates all builds.
2. **build-macos-arm64** — `macos-latest` (arm64). Needs: test. Builds with `--target aarch64-apple-darwin`.
3. **build-macos-x86_64** — `macos-13` (Intel). Needs: test. Builds with `--target x86_64-apple-darwin`.
4. **build-linux-x86_64** — `ubuntu-22.04`. Needs: test. Installs webkit2gtk deps, builds.
5. **release** — Needs: all builds. Creates GitHub Release, uploads all artifacts.
6. **update-homebrew** — Needs: release. Computes SHA256 of tarballs, updates `HomebrewFormula/kanban.rb`, commits to repo.

### Build Steps (per platform)

```
npm ci
cargo tauri build [--target <triple>]
```

Tauri produces `.dmg` (macOS) and `.deb` (Linux) automatically. We also create `.tar.gz` archives of the raw binary for Homebrew.

## Release Artifacts

| Platform | Files |
|----------|-------|
| macOS arm64 | `kanban-macos-aarch64.dmg`, `kanban-macos-aarch64.tar.gz` |
| macOS x86_64 | `kanban-macos-x86_64.dmg`, `kanban-macos-x86_64.tar.gz` |
| Linux x86_64 | `kanban-linux-x86_64.deb`, `kanban-linux-x86_64.tar.gz` |

The `.tar.gz` contains just the `kanban` binary. The `.dmg` and `.deb` are full installers.

## Homebrew Formula

Lives at `HomebrewFormula/kanban.rb` in this repo. Users install via:

```
brew tap akassharjun/kanban https://github.com/akassharjun/kanban
brew install kanban
```

The formula detects the current architecture and downloads the matching `.tar.gz` from the GitHub Release. CI updates the formula's version and SHA256 on each release.

## Files

| File | Purpose |
|------|---------|
| `.github/workflows/release.yml` | Full CI/CD pipeline |
| `HomebrewFormula/kanban.rb` | Homebrew formula |

## Risks

| Risk | Mitigation |
|------|-----------|
| macOS code signing | Unsigned for now; users run `xattr -cr` or Gatekeeper bypass. Add signing later. |
| webkit2gtk version on Ubuntu | Pin to `ubuntu-22.04` runner with known working deps |
| Homebrew formula commit on tag push | Use `GITHUB_TOKEN` with contents write permission |
