import { test, expect, appReady } from "../fixtures/test-base";
import { createIssue } from "../helpers/actions";

test.describe("Workflow: Star, Duplicate, and Delete", () => {
  test.beforeEach(async ({ page }) => {
    await appReady(page);
  });

  test("star and unstar an issue", async ({ page }) => {
    await createIssue(page, { title: "Star Test Issue" });
    await page.getByText("Star Test Issue").first().click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "visible", timeout: 8_000 });

    const panel = page.locator(".rounded-xl.border").first();

    // The Star button has title="Star" when not starred, "Unstar" when starred
    const starBtn = panel.locator('[title="Star"], [title="Unstar"]').first();
    await starBtn.waitFor({ state: "visible" });

    const currentTitle = await starBtn.getAttribute("title");

    if (currentTitle === "Star") {
      await starBtn.click();
      await expect(panel.locator('[title="Unstar"]').first()).toBeVisible({ timeout: 3_000 });
    } else {
      await starBtn.click();
      await expect(panel.locator('[title="Star"]').first()).toBeVisible({ timeout: 3_000 });
      await panel.locator('[title="Star"]').first().click();
      await expect(panel.locator('[title="Unstar"]').first()).toBeVisible({ timeout: 3_000 });
    }

    // Close the panel
    await page.locator('[title="Close (Esc)"]').click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "hidden" });

    // Sidebar should show a "Starred" section
    await expect(page.getByText("Starred", { exact: true }).first()).toBeVisible({ timeout: 3_000 });

    // Reopen the issue and unstar it
    await page.getByText("Star Test Issue").first().click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "visible", timeout: 8_000 });

    const unstarBtn = panel.locator('[title="Unstar"]').first();
    await unstarBtn.waitFor({ state: "visible" });
    await unstarBtn.click();

    await expect(panel.locator('[title="Star"]').first()).toBeVisible({ timeout: 3_000 });

    await page.locator('[title="Close (Esc)"]').click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "hidden" });
  });

  test("duplicate an issue", async ({ page }) => {
    await createIssue(page, { title: "Duplicate Test Issue" });
    await page.getByText("Duplicate Test Issue").first().click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "visible", timeout: 8_000 });

    const panel = page.locator(".rounded-xl.border").first();

    // Click the Duplicate button
    const duplicateBtn = panel.locator('[title="Duplicate"]').first();
    await duplicateBtn.waitFor({ state: "visible" });
    await duplicateBtn.click();

    // A toast "Issue duplicated" should appear
    const toastOrBoard = page.getByText("Issue duplicated", { exact: true });
    await expect(toastOrBoard).toBeVisible({ timeout: 5_000 });
  });

  test("delete an issue and verify removal from board", async ({ page }) => {
    await createIssue(page, { title: "Delete Test Issue" });
    await page.getByText("Delete Test Issue").first().click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "visible", timeout: 8_000 });

    // Get the identifier before deleting
    const identifierText = await page.locator(".rounded-xl.border").first().locator("[class*='mono']").first().textContent();

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

    // The "Delete Test Issue" card should no longer be visible on the board
    await expect(page.getByText("Delete Test Issue")).not.toBeVisible({ timeout: 5_000 });
  });
});
