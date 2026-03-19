import { test, expect, appReady } from "../fixtures/test-base";
import { openIssue } from "../helpers/actions";

test.describe("Workflow: Move Issue Status", () => {
  test.beforeEach(async ({ page }) => {
    await appReady(page);
  });

  test("move KAN-3 through status workflow: Todo → In Progress → In Review → Done", async ({ page }) => {
    const panel = page.locator(".rounded-xl.border").first();

    // --- Step 1: Open KAN-3 (Todo) and move to In Progress ---
    await openIssue(page, "KAN-3");

    const statusTrigger = panel.locator("button", { hasText: "Todo" }).first();
    await statusTrigger.waitFor({ state: "visible" });
    await statusTrigger.click();

    const inProgressOption = panel.locator("button", { hasText: "In Progress" });
    await inProgressOption.first().waitFor({ state: "visible" });
    await inProgressOption.first().click();

    // Verify status updated in panel
    await expect(panel.locator("button", { hasText: "In Progress" }).first()).toBeVisible();

    // Close panel
    await page.locator('[title="Close (Esc)"]').click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "hidden" });

    // Verify KAN-3 is now visible on the board (it has moved)
    await expect(page.getByText("KAN-3", { exact: true }).first()).toBeVisible({ timeout: 5_000 });

    // --- Step 2: Reopen KAN-3 and move to In Review ---
    await openIssue(page, "KAN-3");

    const statusTrigger2 = panel.locator("button", { hasText: "In Progress" }).first();
    await statusTrigger2.waitFor({ state: "visible" });
    await statusTrigger2.click();

    const inReviewOption = panel.locator("button", { hasText: "In Review" });
    await inReviewOption.first().waitFor({ state: "visible" });
    await inReviewOption.first().click();

    await expect(panel.locator("button", { hasText: "In Review" }).first()).toBeVisible();

    // Close panel
    await page.locator('[title="Close (Esc)"]').click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "hidden" });

    // Verify KAN-3 is on the board
    await expect(page.getByText("KAN-3", { exact: true }).first()).toBeVisible({ timeout: 5_000 });

    // --- Step 3: Reopen KAN-3 and move to Done ---
    await openIssue(page, "KAN-3");

    const statusTrigger3 = panel.locator("button", { hasText: "In Review" }).first();
    await statusTrigger3.waitFor({ state: "visible" });
    await statusTrigger3.click();

    const doneOption = panel.locator("button", { hasText: "Done" });
    await doneOption.first().waitFor({ state: "visible" });
    await doneOption.first().click();

    await expect(panel.locator("button", { hasText: "Done" }).first()).toBeVisible();

    // Close panel
    await page.locator('[title="Close (Esc)"]').click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "hidden" });

    // Verify KAN-3 is still visible on the board (in Done column)
    await expect(page.getByText("KAN-3", { exact: true }).first()).toBeVisible({ timeout: 5_000 });
  });
});
