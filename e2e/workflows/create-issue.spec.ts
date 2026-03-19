import { test, expect, appReady } from "../fixtures/test-base";

test.describe("Workflow: Create Issue", () => {
  test.beforeEach(async ({ page }) => {
    await appReady(page);
  });

  test("full create issue workflow with all fields", async ({ page }) => {
    await page.locator("button", { hasText: /^Backlog/ }).waitFor({ state: "visible" });

    // Open the create dialog with keyboard shortcut
    await page.keyboard.press("c");
    await page.getByPlaceholder("Issue title").waitFor({ state: "visible" });

    // Fill the title
    await page.getByPlaceholder("Issue title").fill("Workflow Test Issue");

    // Select Status: "In Progress"
    const statusDiv = page.locator("div").filter({
      has: page.locator("label", { hasText: /^Status$/ }),
    }).last();
    await statusDiv.locator("select").selectOption({ label: "In Progress" });

    // Select Priority: "High"
    const priorityDiv = page.locator("div").filter({
      has: page.locator("label", { hasText: /^Priority$/ }),
    }).last();
    await priorityDiv.locator("select").selectOption("high");

    // Select Assignee: "Arjun" (only mock member)
    const assigneeDiv = page.locator("div").filter({
      has: page.locator("label", { hasText: /^Assignee$/ }),
    }).last();
    await assigneeDiv.locator("select").selectOption({ label: "Arjun" });

    // Submit the form — use exact: true to avoid matching "Create Your First Project"
    await page.getByRole("button", { name: "Create", exact: true }).click();

    // Wait for dialog to close
    await page.getByPlaceholder("Issue title").waitFor({ state: "hidden" });

    // Verify the new card appears on the board
    await expect(page.getByText("Workflow Test Issue").first()).toBeVisible({ timeout: 10_000 });

    // Click the new card to open the detail panel
    await page.getByText("Workflow Test Issue").first().click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "visible" });

    const panel = page.locator(".rounded-xl.border").first();

    // Verify title
    await expect(panel.locator("h2")).toContainText("Workflow Test Issue");

    // Verify status shows "In Progress"
    await expect(panel.locator("button", { hasText: "In Progress" }).first()).toBeVisible();

    // Verify priority shows "High"
    await expect(panel.locator("button", { hasText: "High" }).first()).toBeVisible();

    // Verify assignee shows "Arjun"
    await expect(panel.locator("button", { hasText: "Arjun" }).first()).toBeVisible();
  });
});
