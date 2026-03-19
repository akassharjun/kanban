import { test, expect, appReady } from "../fixtures/test-base";
import { openSearch, searchFor } from "../helpers/actions";

test.describe("Phase 4: Search & Filtering", () => {
  test.beforeEach(async ({ page }) => {
    await appReady(page);
  });

  test("Cmd+K opens search dialog", async ({ page }) => {
    await openSearch(page);
    await expect(page.getByPlaceholder(/Search issues/)).toBeVisible();
  });

  test("search by title finds matching issues", async ({ page }) => {
    await searchFor(page, "drag");
    // The search dialog is scoped to the w-[560px] container
    const dialog = page.locator(".w-\\[560px\\]");
    // Wait for results to appear (200ms debounce in SearchDialog)
    await expect(
      dialog.getByText("Fix drag-drop position calculation", { exact: true })
    ).toBeVisible({ timeout: 5_000 });
  });

  test("search by identifier finds issue", async ({ page }) => {
    await searchFor(page, "KAN-9");
    // KAN-9 title is "Implement undo/redo for issue edits"
    await expect(
      page.getByText("Implement undo/redo for issue edits", { exact: true })
    ).toBeVisible({ timeout: 5_000 });
  });

  test("select search result opens detail panel", async ({ page }) => {
    await searchFor(page, "drag");
    // Scope to the search dialog container to avoid matching board cards
    const dialog = page.locator(".w-\\[560px\\]");
    // Wait for the KAN-6 result to appear
    const result = dialog.getByText("Fix drag-drop position calculation", { exact: true });
    await result.waitFor({ state: "visible", timeout: 5_000 });
    await result.click();

    // After clicking, the search dialog closes and the detail panel opens
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "visible", timeout: 5_000 });

    // Verify the detail panel is showing KAN-6
    const panel = page.locator(".rounded-xl.border").first();
    await expect(panel.locator("h2")).toContainText("Fix drag-drop position calculation");
  });

  test("search with no matches shows empty state", async ({ page }) => {
    await searchFor(page, "zzzznonexistent");
    // Wait for debounce + search to complete, then check for empty state
    await expect(
      page.getByText("No results found", { exact: true })
    ).toBeVisible({ timeout: 5_000 });
  });

  test("Escape closes search dialog", async ({ page }) => {
    await openSearch(page);
    // Confirm dialog is open
    await expect(page.getByPlaceholder(/Search issues/)).toBeVisible();
    // Press Escape to close
    await page.keyboard.press("Escape");
    // Dialog input should no longer be visible
    await expect(page.getByPlaceholder(/Search issues/)).not.toBeVisible({ timeout: 3_000 });
  });
});
