import { test, expect, appReady } from "../fixtures/test-base";
import { openIssue } from "../helpers/actions";

test.describe("Workflow: Create Branch from Issue", () => {
  test.beforeEach(async ({ page }) => {
    await appReady(page);
    // Ensure board is visible
    await page.locator("button", { hasText: /^Backlog/ }).waitFor({ state: "visible" });
  });

  test("click Create Branch on KAN-3 adds a git link to the panel", async ({ page }) => {
    await openIssue(page, "KAN-3");

    const panel = page.locator(".rounded-xl.border").first();

    // Read the initial Git section header count
    // Git section shows "Git (N)" where N is the number of git links
    const gitHeadingBefore = panel.locator("h3").filter({ hasText: /^Git \(/ });
    await gitHeadingBefore.waitFor({ state: "visible" });
    const beforeText = await gitHeadingBefore.textContent();
    // e.g. "Git (0)"
    const beforeCount = parseInt((beforeText?.match(/\((\d+)\)/) ?? ["", "0"])[1]);

    // Click "Create Branch"
    await panel.locator("button", { hasText: "Create Branch" }).click();

    // Wait for the branch link to appear — Git count should increment
    await panel.locator("h3").filter({ hasText: /^Git \(/ }).waitFor({ state: "visible" });

    // Poll until the count increments (mock backend adds branch link after createBranchForIssue)
    await expect(async () => {
      const afterText = await panel.locator("h3").filter({ hasText: /^Git \(/ }).textContent();
      const afterCount = parseInt((afterText?.match(/\((\d+)\)/) ?? ["", "0"])[1]);
      expect(afterCount).toBeGreaterThan(beforeCount);
    }).toPass({ timeout: 8_000 });
  });

  test("created branch name follows KAN-3/* pattern", async ({ page }) => {
    await openIssue(page, "KAN-3");

    const panel = page.locator(".rounded-xl.border").first();
    await panel.locator("button", { hasText: "Create Branch" }).waitFor({ state: "visible" });

    // Click Create Branch
    await panel.locator("button", { hasText: "Create Branch" }).click();

    // Wait for the new branch link — should contain "kan-3/"
    await expect(panel.getByText(/kan-3\//)).toBeVisible({ timeout: 8_000 });
  });
});
