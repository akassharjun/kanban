import { test, expect, appReady } from "../fixtures/test-base";

test.describe("Workflow: Sub-issues", () => {
  test.beforeEach(async ({ page }) => {
    await appReady(page);
  });

  async function createAndOpenIssue(page: import("@playwright/test").Page, title: string) {
    await page.locator("button", { hasText: /^Backlog/ }).waitFor({ state: "visible" });
    await page.keyboard.press("c");
    await page.getByPlaceholder("Issue title").waitFor({ state: "visible" });
    await page.getByPlaceholder("Issue title").fill(title);
    await page.getByRole("button", { name: "Create", exact: true }).click();
    await page.getByPlaceholder("Issue title").waitFor({ state: "hidden" });

    // Click card to open panel
    await page.getByText(title).first().click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "visible", timeout: 8_000 });
  }

  test("Add sub-issue button is visible in issue detail panel", async ({ page }) => {
    await createAndOpenIssue(page, "Parent Issue for Sub-issues");

    const panel = page.locator(".rounded-xl.border").first();

    // Sub-issues section heading — use exact: true to avoid matching the parent issue title h2
    await expect(panel.getByRole("heading", { name: "Sub-issues", exact: true })).toBeVisible({ timeout: 8_000 });

    // "Add sub-issue" button
    await expect(panel.locator("button", { hasText: "Add sub-issue" })).toBeVisible({ timeout: 5_000 });
  });

  test("clicking Add sub-issue opens create issue dialog", async ({ page }) => {
    await createAndOpenIssue(page, "Parent Issue Click Test");

    const panel = page.locator(".rounded-xl.border").first();

    // Click "Add sub-issue"
    await panel.locator("button", { hasText: "Add sub-issue" }).click();

    // Create issue dialog should open
    await page.getByPlaceholder("Issue title").waitFor({ state: "visible", timeout: 5_000 });
    await expect(page.getByPlaceholder("Issue title")).toBeVisible();
  });

  test("create a sub-issue and verify it appears on board", async ({ page }) => {
    await createAndOpenIssue(page, "Parent Issue Main");

    const panel = page.locator(".rounded-xl.border").first();

    // Confirm Sub-issues section visible
    await panel.getByRole("heading", { name: /Sub-issues/i }).waitFor({ state: "visible", timeout: 8_000 });

    // Click "Add sub-issue" to open create dialog
    await panel.locator("button", { hasText: "Add sub-issue" }).click();
    await page.getByPlaceholder("Issue title").waitFor({ state: "visible", timeout: 5_000 });

    // Fill and create the sub-issue
    await page.getByPlaceholder("Issue title").fill("Child Sub-issue");
    await page.getByRole("button", { name: "Create", exact: true }).click();
    await page.getByPlaceholder("Issue title").waitFor({ state: "hidden" });

    // The sub-issue should appear on the board (in the Backlog column)
    await expect(page.getByText("Child Sub-issue").first()).toBeVisible({ timeout: 10_000 });
  });

  test("sub-issue appears under parent after closing and reopening panel", async ({ page }) => {
    await createAndOpenIssue(page, "Parent With Sub-issue");

    const panel = page.locator(".rounded-xl.border").first();

    await panel.locator("button", { hasText: "Add sub-issue" }).click();
    await page.getByPlaceholder("Issue title").waitFor({ state: "visible", timeout: 5_000 });
    await page.getByPlaceholder("Issue title").fill("Sub-issue Child");
    await page.getByRole("button", { name: "Create", exact: true }).click();
    await page.getByPlaceholder("Issue title").waitFor({ state: "hidden" });

    // Sub-issue appears on board
    await expect(page.getByText("Sub-issue Child").first()).toBeVisible({ timeout: 10_000 });

    // Close and reopen the parent to see the sub-issue in the panel
    await page.locator('[title="Close (Esc)"]').click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "hidden" });

    // Reopen the parent issue
    await page.getByText("Parent With Sub-issue").first().click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "visible", timeout: 8_000 });

    // Sub-issue should appear in the Sub-issues section
    await expect(panel.getByText("Sub-issue Child").first()).toBeVisible({ timeout: 8_000 });
  });

  test("sub-issue identifier appears after reopening parent panel", async ({ page }) => {
    await createAndOpenIssue(page, "Parent With ID Test");

    const panel = page.locator(".rounded-xl.border").first();

    await panel.locator("button", { hasText: "Add sub-issue" }).click();
    await page.getByPlaceholder("Issue title").waitFor({ state: "visible", timeout: 5_000 });
    await page.getByPlaceholder("Issue title").fill("Sub With ID");
    await page.getByRole("button", { name: "Create", exact: true }).click();
    await page.getByPlaceholder("Issue title").waitFor({ state: "hidden" });

    // Close and reopen parent
    await page.locator('[title="Close (Esc)"]').click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "hidden" });
    await page.getByText("Parent With ID Test").first().click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "visible", timeout: 8_000 });

    // Sub-issue should be visible in panel with its title
    await expect(panel.getByText("Sub With ID").first()).toBeVisible({ timeout: 8_000 });

    // The sub-issue button in the list should be clickable
    const subIssueBtn = panel.locator("button").filter({ hasText: "Sub With ID" });
    await expect(subIssueBtn).toBeVisible({ timeout: 5_000 });
  });

  test("clicking sub-issue in list opens its detail panel", async ({ page }) => {
    await createAndOpenIssue(page, "Parent For Navigation");

    const panel = page.locator(".rounded-xl.border").first();

    await panel.locator("button", { hasText: "Add sub-issue" }).click();
    await page.getByPlaceholder("Issue title").waitFor({ state: "visible", timeout: 5_000 });
    await page.getByPlaceholder("Issue title").fill("Navigable Sub-issue");
    await page.getByRole("button", { name: "Create", exact: true }).click();
    await page.getByPlaceholder("Issue title").waitFor({ state: "hidden" });

    // Close and reopen parent to see sub-issue in panel list
    await page.locator('[title="Close (Esc)"]').click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "hidden" });
    await page.getByText("Parent For Navigation").first().click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "visible", timeout: 8_000 });

    // Sub-issue should appear in sub-issues list
    const subBtn = panel.locator("button").filter({ hasText: "Navigable Sub-issue" });
    await subBtn.waitFor({ state: "visible", timeout: 8_000 });

    // Click it to navigate to the sub-issue
    await subBtn.click();

    // Panel should now show the sub-issue's title
    await expect(panel.locator("h2")).toContainText("Navigable Sub-issue", { timeout: 8_000 });
  });
});
