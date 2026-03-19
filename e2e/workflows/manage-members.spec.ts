import { test, expect, appReady } from "../fixtures/test-base";
import { navigateTo, createIssue } from "../helpers/actions";

test.describe("Workflow: Manage Members", () => {
  test.beforeEach(async ({ page }) => {
    await appReady(page);
  });

  test("verify default member is visible on the Members page", async ({ page }) => {
    await navigateTo(page, "members");

    await expect(page.getByText("Team Members", { exact: true })).toBeVisible({ timeout: 5_000 });

    // Default mock member: Arjun (display_name of akassharjun)
    await expect(page.getByText("Arjun").first()).toBeVisible();
  });

  test("add a new member and verify they appear in list and assignee dropdown", async ({ page }) => {
    await navigateTo(page, "members");
    await expect(page.getByText("Team Members", { exact: true })).toBeVisible({ timeout: 5_000 });

    // Click "Add Member" button
    await page.locator("button", { hasText: "Add Member" }).click();

    // The inline form should appear
    await expect(page.getByText("New Member", { exact: true })).toBeVisible({ timeout: 3_000 });

    // Fill in the Name field
    await page.getByPlaceholder("Name", { exact: true }).fill("e2euser");

    // Fill in Display Name
    await page.getByPlaceholder("Display Name (optional)").fill("E2E User");

    // Fill in Email
    await page.getByPlaceholder("Email (optional)").fill("e2euser@example.com");

    // Submit
    await page.locator("button", { hasText: "Add" }).last().click();

    // Form should close
    await expect(page.getByText("New Member", { exact: true })).toBeHidden({ timeout: 5_000 });

    // New member should appear in the list
    await expect(page.getByText("E2E User").first()).toBeVisible({ timeout: 10_000 });
    await expect(page.getByText("e2euser@example.com").first()).toBeVisible();

    // Navigate back to the project board
    await navigateTo(page, "project");
    await expect(page.locator("button", { hasText: /^Backlog/ }).first()).toBeVisible({ timeout: 5_000 });

    // Create an issue and open the assignee dropdown to verify the new member appears
    await createIssue(page, { title: "Member Dropdown Test" });
    await page.getByText("Member Dropdown Test").first().click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "visible", timeout: 8_000 });

    const panel = page.locator(".rounded-xl.border").first();

    // Open the assignee dropdown
    const assigneeTrigger = panel.locator("button", { hasText: "Unassigned" }).first();
    await assigneeTrigger.waitFor({ state: "visible" });
    await assigneeTrigger.click();

    // The dropdown should contain the new member "E2E User"
    const dropdown = panel.locator("div.absolute").first();
    await dropdown.waitFor({ state: "visible" });
    await expect(dropdown.locator("button", { hasText: "E2E User" }).first()).toBeVisible({ timeout: 5_000 });

    await page.keyboard.press("Escape");
  });
});
