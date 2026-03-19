import { test, expect, appReady } from "../fixtures/test-base";
import { openIssue } from "../helpers/actions";

test.describe("Workflow: Issue Code Context", () => {
  test.beforeEach(async ({ page }) => {
    await appReady(page);
    // Ensure we are on the board view for the default project (Kanban Core)
    await page.locator("button", { hasText: /^Backlog/ }).waitFor({ state: "visible" });
  });

  test("KAN-6 issue detail shows Code Context section with branch", async ({ page }) => {
    await openIssue(page, "KAN-6");

    const panel = page.locator(".rounded-xl.border").first();

    // Scroll down to the Code Context section
    // The section only renders when issueCommits or issueBranches is non-empty
    // Mock returns branch: kan-6/fix-drag-drop and commit: feat: review workflow for KAN-6
    await panel.getByText("Code Context").waitFor({ state: "visible", timeout: 8_000 });
    await expect(panel.getByText("Code Context")).toBeVisible();

    // The branch 'kan-6/fix-drag-drop' should be shown
    await expect(panel.getByText("kan-6/fix-drag-drop")).toBeVisible();
  });

  test("KAN-6 issue detail shows commit referencing KAN-6 in Code Context", async ({ page }) => {
    await openIssue(page, "KAN-6");

    const panel = page.locator(".rounded-xl.border").first();
    await panel.getByText("Code Context").waitFor({ state: "visible", timeout: 8_000 });

    // The commit message: "feat: review workflow for KAN-6"
    await expect(panel.getByText(/review workflow for KAN-6/)).toBeVisible();
  });

  test("KAN-3 issue detail shows branch 'kan-3/add-keyboard-shortcuts' in Code Context", async ({ page }) => {
    await openIssue(page, "KAN-3");

    const panel = page.locator(".rounded-xl.border").first();
    await panel.getByText("Code Context").waitFor({ state: "visible", timeout: 8_000 });

    // Branch for KAN-3
    await expect(panel.getByText("kan-3/add-keyboard-shortcuts")).toBeVisible();
  });

  test("KAN-1 issue detail does NOT show Code Context section", async ({ page }) => {
    await openIssue(page, "KAN-1");

    const panel = page.locator(".rounded-xl.border").first();
    // Wait a moment for async data to load
    await page.waitForTimeout(1500);

    // Code Context section should NOT be present (KAN-1 has no commits/branches in mock data)
    await expect(panel.getByText("Code Context")).not.toBeVisible();
  });
});
