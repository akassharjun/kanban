# Project Path Intelligence — Implementation Plan

**Goal:** Turn the project `path` field into a bridge between the Kanban board and the actual codebase, surfacing git info, file structure, agent configs, and worktree status across the entire UI.

**Prerequisite:** Project must have `path` set in Settings → General.

---

## Phase A: File System + Directory Tree + Config Viewer (Code Page)

- Read the file system from `project.path` via Tauri command (or mock in browser)
- Show expandable directory tree with issue count heat overlay
- Color-code: red = high bug density, blue = active work, gray = untouched
- Detect and render `.claude/`, `.codex/`, `CLAUDE.md`, `AGENTS.md` from project root
- Show config file content as rendered markdown or syntax-highlighted JSON
- Click a file → see linked issues

## Phase B: Git Integration (Status, Branches, Commits, Worktree Scanning)

- Current branch, uncommitted changes count, ahead/behind remote
- Recent commits (last 10) with author, message, linked issue identifiers
- Branch list with KAN-* matching to Kanban issues
- Scan `.git/worktrees/` — list all worktrees with path, branch, status
- Git status badge on sidebar project name (green/yellow/red)

## Phase C: Per-Issue Code Context (Issue Detail Panel)

- Show commits referencing this issue's identifier
- Show branches matching this issue
- File content preview on hover for linked files
- Auto-suggest related files based on issue title/description keywords
- Diff preview across linked commits

## Phase D: Sidebar Health + Agent Worktree Correlation

- Project health indicator badge (clean/dirty/conflicts)
- Tooltip: "3 uncommitted files, 2 commits ahead of main"
- Active worktree map on Agent Dashboard
- Cross-reference agent `worktree_path` with `.git/worktrees/`
- Visual: "Claude Opus working on KAN-14 in `worktrees/kan-14-fix-auth`"
- Worktree status: idle / active / stale

## Terminal Panel (independent)

- Toggle button in sidebar bottom
- Full-width resizable panel at bottom of window (JetBrains-style)
- xterm.js integration for Tauri mode
- Mock/placeholder in browser mode
