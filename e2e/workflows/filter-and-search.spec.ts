import { test, expect, appReady } from "../fixtures/test-base";
import { openSearch } from "../helpers/actions";

// The board columns live in a flex container with overflow-x-auto.
// Scope all visibility checks to this container to avoid matching the sidebar
// (which also shows issue titles in the Starred/Recent sections).
function boardColumns(page: import("@playwright/test").Page) {
  return page.locator(".overflow-x-auto").first();
}

test.describe("Workflow: Filter and Search", () => {
  test.beforeEach(async ({ page }) => {
    await appReady(page);
    // Wait for board columns to be fully loaded
    await page.locator("button", { hasText: /^Backlog/ }).waitFor({ state: "visible" });
  });

  test("filter by status Todo shows only Todo issues", async ({ page }) => {
    // The FilterBar has a status <select> with "All statuses" as default option
    const statusSelect = page.locator("select").first();
    await statusSelect.waitFor({ state: "visible" });

    // Select "Todo" from status dropdown
    await statusSelect.selectOption({ label: "Todo" });

    const board = boardColumns(page);

    // KAN-3, KAN-4, KAN-5 are in Todo — their identifiers should appear in board
    await expect(board.getByText("KAN-3", { exact: true }).first()).toBeVisible({ timeout: 5_000 });
    await expect(board.getByText("KAN-4", { exact: true }).first()).toBeVisible({ timeout: 5_000 });
    await expect(board.getByText("KAN-5", { exact: true }).first()).toBeVisible({ timeout: 5_000 });

    // KAN-6 is In Progress — should NOT appear in the board columns area
    await expect(board.getByText("KAN-6", { exact: true })).not.toBeVisible({ timeout: 3_000 });
  });

  test("filter by status In Progress shows only In Progress issues", async ({ page }) => {
    const statusSelect = page.locator("select").first();
    await statusSelect.waitFor({ state: "visible" });

    await statusSelect.selectOption({ label: "In Progress" });

    const board = boardColumns(page);

    // KAN-6 is In Progress — should appear in board
    await expect(board.getByText("KAN-6", { exact: true }).first()).toBeVisible({ timeout: 5_000 });

    // KAN-3 is Todo — should not appear in board columns
    await expect(board.getByText("KAN-3", { exact: true })).not.toBeVisible({ timeout: 3_000 });
  });

  test("clear status filter shows all issues", async ({ page }) => {
    const statusSelect = page.locator("select").first();
    await statusSelect.waitFor({ state: "visible" });

    // Set filter to Todo
    await statusSelect.selectOption({ label: "Todo" });

    const board = boardColumns(page);
    // Verify filter is active
    await expect(board.getByText("KAN-6", { exact: true })).not.toBeVisible({ timeout: 3_000 });

    // Clear filter by selecting "All statuses"
    await statusSelect.selectOption({ label: "All statuses" });

    // Issues from all statuses should now be visible
    await expect(board.getByText("KAN-3", { exact: true }).first()).toBeVisible({ timeout: 5_000 });
    await expect(board.getByText("KAN-6", { exact: true }).first()).toBeVisible({ timeout: 5_000 });
  });

  test("filter by priority Urgent shows only KAN-6", async ({ page }) => {
    // Priority select is the second <select> in the FilterBar
    const selects = page.locator("select");
    const prioritySelect = selects.nth(1);
    await prioritySelect.waitFor({ state: "visible" });

    await prioritySelect.selectOption({ label: "Urgent" });

    const board = boardColumns(page);

    // KAN-6 has urgent priority — should appear
    await expect(board.getByText("KAN-6", { exact: true }).first()).toBeVisible({ timeout: 5_000 });

    // KAN-3 has medium priority — should not appear in board columns
    await expect(board.getByText("KAN-3", { exact: true })).not.toBeVisible({ timeout: 3_000 });

    // KAN-1 has low priority — should not appear in board columns
    await expect(board.getByText("KAN-1", { exact: true })).not.toBeVisible({ timeout: 3_000 });
  });

  test("filter by assignee Arjun shows only Arjun's issues", async ({ page }) => {
    // Assignee select is the third <select> in the FilterBar
    const selects = page.locator("select");
    const assigneeSelect = selects.nth(2);
    await assigneeSelect.waitFor({ state: "visible" });

    // Select Arjun (member display_name "Arjun", id=1)
    await assigneeSelect.selectOption({ label: "Arjun" });

    const board = boardColumns(page);

    // KAN-3 (Arjun), KAN-4 (Arjun) should be visible
    await expect(board.getByText("KAN-3", { exact: true }).first()).toBeVisible({ timeout: 5_000 });
    await expect(board.getByText("KAN-4", { exact: true }).first()).toBeVisible({ timeout: 5_000 });

    // KAN-5 has no assignee — should not appear in board columns
    await expect(board.getByText("KAN-5", { exact: true })).not.toBeVisible({ timeout: 3_000 });

    // KAN-6 is assigned to Claude — should not appear in board columns
    await expect(board.getByText("KAN-6", { exact: true })).not.toBeVisible({ timeout: 3_000 });
  });

  test("search for undo shows KAN-9 and opens panel on click", async ({ page }) => {
    // Open search dialog with Control+k (Linux)
    await openSearch(page);

    // Type "undo" to search
    const searchInput = page.getByPlaceholder(/Search issues/);
    await searchInput.waitFor({ state: "visible" });
    await searchInput.fill("undo");

    // KAN-9 title is "Implement undo/redo for issue edits"
    const dialog = page.locator(".w-\\[560px\\]");
    const result = dialog.getByText("Implement undo/redo for issue edits", { exact: true });
    await result.waitFor({ state: "visible", timeout: 5_000 });

    // Click the result
    await result.click();

    // The detail panel should open for KAN-9
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "visible", timeout: 5_000 });

    const panel = page.locator(".rounded-xl.border").first();
    await expect(panel.locator("h2")).toContainText("Implement undo/redo for issue edits");
  });
});
