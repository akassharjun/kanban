import { test, expect, appReady } from "../fixtures/test-base";

test.describe("Workflow: Project Lifecycle", () => {
  test.beforeEach(async ({ page }) => {
    await appReady(page);
  });

  test("create project via onboarding and board shows default statuses", async ({ page }) => {
    // appReady already created the project and waits for board
    await expect(page.locator("button", { hasText: /^Backlog/ })).toBeVisible();
    await expect(page.locator("button", { hasText: /^Todo/ })).toBeVisible();
    await expect(page.locator("button", { hasText: /^In Progress/ })).toBeVisible();
    await expect(page.locator("button", { hasText: /^In Review/ })).toBeVisible();
    await expect(page.locator("button", { hasText: /^Done/ })).toBeVisible();
  });

  test("create an issue and verify it appears on board", async ({ page }) => {
    await page.locator("button", { hasText: /^Backlog/ }).waitFor({ state: "visible" });

    // Open create dialog
    await page.keyboard.press("c");
    await page.getByPlaceholder("Issue title").waitFor({ state: "visible" });
    await page.getByPlaceholder("Issue title").fill("My First Issue");
    await page.getByRole("button", { name: "Create", exact: true }).click();
    await page.getByPlaceholder("Issue title").waitFor({ state: "hidden" });

    // Verify issue appears on board
    await expect(page.getByText("My First Issue").first()).toBeVisible({ timeout: 10_000 });
  });

  test("navigate to Settings and verify project name is visible", async ({ page }) => {
    await page.locator("button", { hasText: /^Backlog/ }).waitFor({ state: "visible" });

    await page.locator("button", { hasText: "Settings" }).click();
    await expect(page.getByText("Project Settings")).toBeVisible({ timeout: 5_000 });
    // Project name "Test Project" should appear in settings
    await expect(page.getByText("Test Project").first()).toBeVisible();
  });

  test("navigate back to board — board still shows", async ({ page }) => {
    await page.locator("button", { hasText: /^Backlog/ }).waitFor({ state: "visible" });

    // Go to Settings
    await page.locator("button", { hasText: "Settings" }).click();
    await page.getByText("Project Settings").waitFor({ state: "visible" });

    // Go back to project (board) via sidebar
    await page.locator("button", { hasText: /Test Project/ }).first().click();
    await expect(page.locator("button", { hasText: /^Backlog/ })).toBeVisible({ timeout: 8_000 });
  });

  test("create a second project via sidebar and switch between projects", async ({ page }) => {
    await page.locator("button", { hasText: /^Backlog/ }).waitFor({ state: "visible" });

    // Create an issue in the first project to distinguish it
    await page.keyboard.press("c");
    await page.getByPlaceholder("Issue title").waitFor({ state: "visible" });
    await page.getByPlaceholder("Issue title").fill("First Project Issue");
    await page.getByRole("button", { name: "Create", exact: true }).click();
    await page.getByPlaceholder("Issue title").waitFor({ state: "hidden" });
    await expect(page.getByText("First Project Issue").first()).toBeVisible({ timeout: 8_000 });

    // Create second project via sidebar "New project" button
    await page.locator("button", { hasText: "New project" }).click();
    await page.getByPlaceholder("My Project").waitFor({ state: "visible" });
    await page.getByPlaceholder("My Project").fill("Second Project");
    await page.getByPlaceholder("PRJ").clear();
    await page.getByPlaceholder("PRJ").fill("SEC");
    await page.getByPlaceholder("/path/to/your/project").fill("/home/user/kanban");
    await page.getByRole("button", { name: "Create", exact: true }).click();

    // Board for second project should load (empty)
    await page.locator("button", { hasText: /^Backlog/ }).waitFor({ state: "visible", timeout: 15_000 });

    // Second project should appear in sidebar
    await expect(page.locator("button", { hasText: "Second Project" }).first()).toBeVisible();

    // Switch back to first project
    await page.locator("button", { hasText: /Test Project/ }).first().click();
    await page.locator("button", { hasText: /^Backlog/ }).waitFor({ state: "visible" });

    // The first project issue should be visible again
    await expect(page.getByText("First Project Issue").first()).toBeVisible({ timeout: 8_000 });
  });
});
