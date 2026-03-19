import { test, expect, appReady } from "../fixtures/test-base";
import { createIssue } from "../helpers/actions";

test.describe("Workflow: Issue Code Context", () => {
  test.beforeEach(async ({ page }) => {
    await appReady(page);
    await page.locator("button", { hasText: /^Backlog/ }).waitFor({ state: "visible" });
  });

  test("new issue detail shows Git section", async ({ page }) => {
    await createIssue(page, { title: "Git Section Test Issue" });
    await page.getByText("Git Section Test Issue").first().click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "visible", timeout: 8_000 });

    const panel = page.locator(".rounded-xl.border").first();

    // Git section should be visible (shows "Git (N)" heading)
    await expect(panel.locator("h3").filter({ hasText: /^Git \(/ })).toBeVisible({ timeout: 8_000 });
  });

  test("Create Branch button adds a git link to the panel", async ({ page }) => {
    await createIssue(page, { title: "Create Branch Test Issue" });
    await page.getByText("Create Branch Test Issue").first().click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "visible", timeout: 8_000 });

    const panel = page.locator(".rounded-xl.border").first();

    // Read the initial Git section count
    const gitHeading = panel.locator("h3").filter({ hasText: /^Git \(/ });
    await gitHeading.waitFor({ state: "visible" });
    const beforeText = await gitHeading.textContent();
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

  test("created branch name follows issue-identifier/* pattern", async ({ page }) => {
    await createIssue(page, { title: "Branch Pattern Issue" });
    await page.getByText("Branch Pattern Issue").first().click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "visible", timeout: 8_000 });

    const panel = page.locator(".rounded-xl.border").first();
    await panel.locator("button", { hasText: "Create Branch" }).waitFor({ state: "visible" });
    await panel.locator("button", { hasText: "Create Branch" }).click();

    // Branch name should follow tst-N/* pattern (prefix is TST → tst-)
    await expect(panel.getByText(/tst-\d+\//i).first()).toBeVisible({ timeout: 8_000 });
  });

  test("issue with no commits does NOT show Code Context section", async ({ page }) => {
    await createIssue(page, { title: "No Code Context Issue" });
    await page.getByText("No Code Context Issue").first().click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "visible", timeout: 8_000 });

    const panel = page.locator(".rounded-xl.border").first();

    // Wait a moment for async data to load
    await page.waitForTimeout(1500);

    // Code Context section should NOT be present (no commits/branches for new TST issues)
    // Use exact heading match to avoid matching the issue title which contains "Code Context"
    await expect(panel.getByRole("heading", { name: "Code Context", exact: true })).not.toBeVisible();
  });
});
