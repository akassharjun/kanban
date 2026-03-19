#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
KANBAN="$PROJECT_ROOT/src-tauri/target/release/kanban"
TEMP_DB=$(mktemp /tmp/kanban-smoke-XXXXXX.db)

trap 'rm -f "$TEMP_DB" "${TEMP_DB}-wal" "${TEMP_DB}-shm"' EXIT

PASS=0
FAIL=0

pass() { PASS=$((PASS + 1)); echo "  ✓ $1"; }
fail() { FAIL=$((FAIL + 1)); echo "  ✗ $1: $2"; }

run_cli() {
    "$KANBAN" --database-url "sqlite://$TEMP_DB?mode=rwc" cli --json "$@" 2>&1
}

# ─── Build ────────────────────────────────────────────────────────────────────

if [[ "${1:-}" != "--skip-build" ]]; then
    echo "Building kanban binary..."
    (cd "$PROJECT_ROOT/src-tauri" && cargo build --release --bin kanban 2>&1) \
        && echo "  Build succeeded." \
        || { echo "  Build FAILED. Aborting."; exit 1; }
fi

if [[ ! -x "$KANBAN" ]]; then
    echo "Binary not found or not executable: $KANBAN"
    exit 1
fi

echo ""
echo "Running smoke tests against: $TEMP_DB"
echo ""

# ─── Project CRUD ─────────────────────────────────────────────────────────────

echo "── Project CRUD ──"

PROJECT_OUTPUT=$(run_cli project create "Smoke Test" --prefix SMK)
if echo "$PROJECT_OUTPUT" | grep -qi "smoke test\|smk\|created\|success\|\"id\""; then
    pass "create project 'Smoke Test' (prefix SMK)"
else
    fail "create project 'Smoke Test' (prefix SMK)" "$PROJECT_OUTPUT"
fi

LIST_OUTPUT=$(run_cli project list)
if echo "$LIST_OUTPUT" | grep -qi "smoke test"; then
    pass "list projects (name 'Smoke Test' appears)"
else
    fail "list projects (name 'Smoke Test' appears)" "$LIST_OUTPUT"
fi

echo ""

# ─── Issue CRUD ───────────────────────────────────────────────────────────────

echo "── Issue CRUD ──"

ISSUE1_OUTPUT=$(run_cli issue create --project 1 --title "First smoke issue" --status 1 --priority high)
if echo "$ISSUE1_OUTPUT" | grep -qi "SMK-1\|first smoke"; then
    pass "create issue SMK-1"
else
    fail "create issue SMK-1" "$ISSUE1_OUTPUT"
fi

ISSUE2_OUTPUT=$(run_cli issue create --project 1 --title "Second smoke issue" --status 1 --priority low)
if echo "$ISSUE2_OUTPUT" | grep -qi "SMK-2\|second smoke"; then
    pass "create issue SMK-2"
else
    fail "create issue SMK-2" "$ISSUE2_OUTPUT"
fi

LIST_ISSUES=$(run_cli issue list --project 1)
if echo "$LIST_ISSUES" | grep -qi "first smoke"; then
    pass "list issues (SMK-1 appears)"
else
    fail "list issues (SMK-1 appears)" "$LIST_ISSUES"
fi
if echo "$LIST_ISSUES" | grep -qi "second smoke"; then
    pass "list issues (SMK-2 appears)"
else
    fail "list issues (SMK-2 appears)" "$LIST_ISSUES"
fi

UPDATE_OUTPUT=$(run_cli issue update SMK-1 --priority urgent)
if echo "$UPDATE_OUTPUT" | grep -qi "urgent\|updated\|success\|SMK-1"; then
    pass "update SMK-1 priority to urgent"
else
    fail "update SMK-1 priority to urgent" "$UPDATE_OUTPUT"
fi

SEARCH_OUTPUT=$(run_cli issue search --project 1 "smoke")
if echo "$SEARCH_OUTPUT" | grep -qi "smoke"; then
    pass "search issues for 'smoke'"
else
    fail "search issues for 'smoke'" "$SEARCH_OUTPUT"
fi

DELETE_OUTPUT=$(run_cli issue delete SMK-2)
if echo "$DELETE_OUTPUT" | grep -qiv "error\|fail"; then
    pass "delete SMK-2"
else
    fail "delete SMK-2" "$DELETE_OUTPUT"
fi

LIST_AFTER_DELETE=$(run_cli issue list --project 1)
if echo "$LIST_AFTER_DELETE" | grep -qi "second smoke"; then
    fail "verify SMK-2 deleted (still appears in list)" "$LIST_AFTER_DELETE"
else
    pass "verify SMK-2 no longer in issue list"
fi

echo ""

# ─── Members ──────────────────────────────────────────────────────────────────

echo "── Members ──"

MEMBER_OUTPUT=$(run_cli member add "alice" --email "alice@example.com")
if echo "$MEMBER_OUTPUT" | grep -qi "alice\|added\|success\|\"id\""; then
    pass "add member 'alice'"
else
    fail "add member 'alice'" "$MEMBER_OUTPUT"
fi

MEMBER_LIST=$(run_cli member list)
if echo "$MEMBER_LIST" | grep -qi "alice"; then
    pass "list members (alice appears)"
else
    fail "list members (alice appears)" "$MEMBER_LIST"
fi

echo ""

# ─── Results ──────────────────────────────────────────────────────────────────

TOTAL=$((PASS + FAIL))
echo "Results: $PASS/$TOTAL passed"

if [[ $FAIL -gt 0 ]]; then
    echo "FAILED ($FAIL test(s) failed)"
    exit 1
else
    echo "All tests passed."
    exit 0
fi
