import { test, expect, appReady } from "../fixtures/test-base";
import { createIssue } from "../helpers/actions";

test.describe("Workflow: Create Branch from Issue", () => {
  test.beforeEach(async ({ page }) => {
    await appReady(page);
    await page.locator("button", { hasText: /^Backlog/ }).waitFor({ state: "visible" });
  });

  test("click Create Branch on an issue adds a git link to the panel", async ({ page }) => {
    await createIssue(page, { title: "Branch Create Test Issue" });
    await page.getByText("Branch Create Test Issue").first().click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "visible", timeout: 8_000 });

    const panel = page.locator(".rounded-xl.border").first();

    // Read the initial Git section header count
    const gitHeadingBefore = panel.locator("h3").filter({ hasText: /^Git \(/ });
    await gitHeadingBefore.waitFor({ state: "visible" });
    const beforeText = await gitHeadingBefore.textContent();
    const beforeCount = parseInt((beforeText?.match(/\((\d+)\)/) ?? ["", "0"])[1]);

    // Click "Create Branch"
    await panel.locator("button", { hasText: "Create Branch" }).click();

    // Git count should increment
    await expect(async () => {
      const afterText = await panel.locator("h3").filter({ hasText: /^Git \(/ }).textContent();
      const afterCount = parseInt((afterText?.match(/\((\d+)\)/) ?? ["", "0"])[1]);
      expect(afterCount).toBeGreaterThan(beforeCount);
    }).toPass({ timeout: 8_000 });
  });

  test("created branch name follows TST-N/* pattern", async ({ page }) => {
    await createIssue(page, { title: "Branch Name Pattern Issue" });
    await page.getByText("Branch Name Pattern Issue").first().click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "visible", timeout: 8_000 });

    const panel = page.locator(".rounded-xl.border").first();
    await panel.locator("button", { hasText: "Create Branch" }).waitFor({ state: "visible" });
    await panel.locator("button", { hasText: "Create Branch" }).click();

    // Branch name should contain "tst-" (prefix TST lowercased)
    await expect(panel.getByText(/tst-\d+\//i).first()).toBeVisible({ timeout: 8_000 });
  });
});
