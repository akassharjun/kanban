import { test, expect, appReady } from "../fixtures/test-base";
import { openIssue } from "../helpers/actions";

test.describe("Workflow: Star, Duplicate, and Delete", () => {
  test.beforeEach(async ({ page }) => {
    await appReady(page);
  });

  test("star and unstar an issue", async ({ page }) => {
    // KAN-1 is NOT starred by default (mock data shows KAN-3, KAN-6, KAN-7 are starred for member 1)
    // The sidebar "Starred" section is visible only if issues are starred for the current member.
    // We test toggle behavior via the panel star button.
    await openIssue(page, "KAN-1");

    const panel = page.locator(".rounded-xl.border").first();

    // The Star button has title="Star" when not starred, "Unstar" when starred
    const starBtn = panel.locator('[title="Star"], [title="Unstar"]').first();
    await starBtn.waitFor({ state: "visible" });

    // Get current state and click to toggle
    const currentTitle = await starBtn.getAttribute("title");

    if (currentTitle === "Star") {
      // Click to star
      await starBtn.click();
      // After starring, it should become "Unstar"
      await expect(panel.locator('[title="Unstar"]').first()).toBeVisible({ timeout: 3_000 });
    } else {
      // Already starred — click to unstar
      await starBtn.click();
      // After unstarring, it should become "Star"
      await expect(panel.locator('[title="Star"]').first()).toBeVisible({ timeout: 3_000 });
      // Click again to re-star for further assertions
      await panel.locator('[title="Star"]').first().click();
      await expect(panel.locator('[title="Unstar"]').first()).toBeVisible({ timeout: 3_000 });
    }

    // Close the panel
    await page.locator('[title="Close (Esc)"]').click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "hidden" });

    // Sidebar should show a "Starred" section with KAN-1 listed
    await expect(page.getByText("Starred", { exact: true }).first()).toBeVisible({ timeout: 3_000 });
    await expect(page.getByText("KAN-1", { exact: true }).first()).toBeVisible();

    // Reopen KAN-1 and click Unstar
    await openIssue(page, "KAN-1");
    const unstarBtn = panel.locator('[title="Unstar"]').first();
    await unstarBtn.waitFor({ state: "visible" });
    await unstarBtn.click();

    // Should now show "Star"
    await expect(panel.locator('[title="Star"]').first()).toBeVisible({ timeout: 3_000 });

    await page.locator('[title="Close (Esc)"]').click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "hidden" });
  });

  test("duplicate an issue", async ({ page }) => {
    await openIssue(page, "KAN-1");

    const panel = page.locator(".rounded-xl.border").first();

    // Click the Duplicate button (title="Duplicate")
    const duplicateBtn = panel.locator('[title="Duplicate"]').first();
    await duplicateBtn.waitFor({ state: "visible" });
    await duplicateBtn.click();

    // A toast "Issue duplicated" should appear, or a new card should be visible
    // Wait for either the toast or the panel to still be present
    // The toast uses a specific class/role; check for the message text
    const toastOrBoard = page.getByText("Issue duplicated", { exact: true });
    await expect(toastOrBoard).toBeVisible({ timeout: 5_000 });
  });

  test("delete an issue and verify removal from board", async ({ page }) => {
    // Open KAN-5 (Todo, low priority, no assignee)
    await openIssue(page, "KAN-5");

    const panel = page.locator(".rounded-xl.border").first();

    // First click on delete shows "Confirm?" button
    const deleteBtn = panel.locator('[title="Delete"]').first();
    await deleteBtn.waitFor({ state: "visible" });
    await deleteBtn.click();

    // Confirm the delete
    const confirmBtn = panel.locator('[title="Confirm Delete"]').first();
    await confirmBtn.waitFor({ state: "visible" });
    await confirmBtn.click();

    // Panel should close after deletion
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "hidden", timeout: 5_000 });

    // KAN-5 card should no longer be visible on the board
    await expect(page.getByText("KAN-5", { exact: true })).not.toBeVisible({ timeout: 5_000 });
  });
});
