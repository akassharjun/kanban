import { test, expect, appReady } from "../fixtures/test-base";
import { createIssue } from "../helpers/actions";

test.describe("Phase 3: Issue Lifecycle", () => {
  test.beforeEach(async ({ page }) => {
    await appReady(page);
  });

  test("create a new issue", async ({ page }) => {
    await page.locator('button', { hasText: /^Backlog/ }).waitFor({ state: "visible" });

    await createIssue(page, { title: "E2E Test Issue" });
    await expect(page.getByText("E2E Test Issue").first()).toBeVisible({ timeout: 10_000 });
  });

  test("edit issue title in detail panel", async ({ page }) => {
    await createIssue(page, { title: "Original Title for Edit" });
    await page.getByText("Original Title for Edit").first().click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "visible", timeout: 8_000 });

    const panel = page.locator(".rounded-xl.border").first();

    // Click the h2 title to switch to editing mode
    await panel.locator("h2").click();

    const titleInput = panel.locator('input.w-full.bg-transparent');
    await titleInput.waitFor({ state: "visible" });
    await titleInput.clear();
    await titleInput.fill("Updated Title via Edit");
    await titleInput.press("Enter");

    await expect(panel.locator("h2")).toHaveText("Updated Title via Edit");
  });

  test("edit issue description in detail panel", async ({ page }) => {
    await createIssue(page, { title: "Description Edit Issue" });
    await page.getByText("Description Edit Issue").first().click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "visible", timeout: 8_000 });

    const panel = page.locator(".rounded-xl.border").first();

    // Find the prose/description area and click to edit
    const descView = panel.locator(".prose.cursor-pointer");
    await descView.waitFor({ state: "visible" });
    await descView.click();

    const textarea = panel.locator("textarea").first();
    await textarea.waitFor({ state: "visible" });
    await textarea.clear();
    await textarea.fill("Updated description via e2e test");
    await textarea.blur();

    await expect(panel.locator(".prose.cursor-pointer")).toContainText("Updated description via e2e test");
  });

  test("change issue status via detail panel", async ({ page }) => {
    await createIssue(page, { title: "Status Change Issue" });
    await page.getByText("Status Change Issue").first().click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "visible", timeout: 8_000 });

    const panel = page.locator(".rounded-xl.border").first();

    // New issue starts in Backlog — click the status dropdown trigger
    const statusTrigger = panel.locator('button', { hasText: "Backlog" }).first();
    await statusTrigger.waitFor({ state: "visible" });
    await statusTrigger.click();

    // Click "In Progress"
    const inProgressOption = panel.locator('button', { hasText: "In Progress" });
    await inProgressOption.first().waitFor({ state: "visible" });
    await inProgressOption.first().click();

    await expect(panel.locator('button', { hasText: "In Progress" }).first()).toBeVisible();
  });

  test("assign member to issue", async ({ page }) => {
    await createIssue(page, { title: "Assign Member Issue" });
    await page.getByText("Assign Member Issue").first().click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "visible", timeout: 8_000 });

    const panel = page.locator(".rounded-xl.border").first();

    // New issue has no assignee — the trigger shows "Unassigned"
    const assigneeTrigger = panel.locator('button', { hasText: "Unassigned" }).first();
    await assigneeTrigger.waitFor({ state: "visible" });
    await assigneeTrigger.click();

    // Click "Arjun" (only mock member)
    const arjunOption = panel.locator('button', { hasText: "Arjun" });
    await arjunOption.first().waitFor({ state: "visible" });
    await arjunOption.first().click();

    await expect(panel.locator('button', { hasText: "Arjun" }).first()).toBeVisible();
  });

  test("activity log shows changes", async ({ page }) => {
    await createIssue(page, { title: "Activity Log Issue" });
    await page.getByText("Activity Log Issue").first().click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "visible", timeout: 8_000 });

    const panel = page.locator(".rounded-xl.border").first();

    // Click the Activity tab
    const activityTab = panel.locator('button', { hasText: "Activity" });
    await activityTab.waitFor({ state: "visible" });
    await activityTab.click();

    // The mock backend returns activity entries with "Changed" text
    await expect(panel.getByText("Changed", { exact: false }).first()).toBeVisible({ timeout: 5_000 });
  });
});
