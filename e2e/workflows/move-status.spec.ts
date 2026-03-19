import { test, expect, appReady } from "../fixtures/test-base";
import { createIssue } from "../helpers/actions";

test.describe("Workflow: Move Issue Status", () => {
  test.beforeEach(async ({ page }) => {
    await appReady(page);
  });

  test("move issue through status workflow: Backlog → In Progress → In Review → Done", async ({ page }) => {
    await createIssue(page, { title: "Status Workflow Issue" });

    const panel = page.locator(".rounded-xl.border").first();

    // --- Step 1: Open issue and move to In Progress ---
    await page.getByText("Status Workflow Issue").first().click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "visible", timeout: 8_000 });

    const statusTrigger = panel.locator("button", { hasText: "Backlog" }).first();
    await statusTrigger.waitFor({ state: "visible" });
    await statusTrigger.click();

    const inProgressOption = panel.locator("button", { hasText: "In Progress" });
    await inProgressOption.first().waitFor({ state: "visible" });
    await inProgressOption.first().click();

    await expect(panel.locator("button", { hasText: "In Progress" }).first()).toBeVisible();

    // Close panel
    await page.locator('[title="Close (Esc)"]').click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "hidden" });

    // Verify issue is still visible on board
    await expect(page.getByText("Status Workflow Issue").first()).toBeVisible({ timeout: 5_000 });

    // --- Step 2: Reopen and move to In Review ---
    await page.getByText("Status Workflow Issue").first().click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "visible", timeout: 8_000 });

    const statusTrigger2 = panel.locator("button", { hasText: "In Progress" }).first();
    await statusTrigger2.waitFor({ state: "visible" });
    await statusTrigger2.click();

    const inReviewOption = panel.locator("button", { hasText: "In Review" });
    await inReviewOption.first().waitFor({ state: "visible" });
    await inReviewOption.first().click();

    await expect(panel.locator("button", { hasText: "In Review" }).first()).toBeVisible();

    await page.locator('[title="Close (Esc)"]').click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "hidden" });

    await expect(page.getByText("Status Workflow Issue").first()).toBeVisible({ timeout: 5_000 });

    // --- Step 3: Reopen and move to Done ---
    await page.getByText("Status Workflow Issue").first().click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "visible", timeout: 8_000 });

    const statusTrigger3 = panel.locator("button", { hasText: "In Review" }).first();
    await statusTrigger3.waitFor({ state: "visible" });
    await statusTrigger3.click();

    const doneOption = panel.locator("button", { hasText: "Done" });
    await doneOption.first().waitFor({ state: "visible" });
    await doneOption.first().click();

    await expect(panel.locator("button", { hasText: "Done" }).first()).toBeVisible();

    await page.locator('[title="Close (Esc)"]').click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "hidden" });

    // Verify issue is still visible on board (in Done column)
    await expect(page.getByText("Status Workflow Issue").first()).toBeVisible({ timeout: 5_000 });
  });
});
