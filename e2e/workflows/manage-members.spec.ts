import { test, expect, appReady } from "../fixtures/test-base";
import { navigateTo, openIssue } from "../helpers/actions";

test.describe("Workflow: Manage Members", () => {
  test.beforeEach(async ({ page }) => {
    await appReady(page);
  });

  test("verify all existing members are visible on the Members page", async ({ page }) => {
    await navigateTo(page, "members");

    // The heading confirms we're on the correct page
    await expect(page.getByText("Team Members", { exact: true })).toBeVisible({ timeout: 5_000 });

    // All 3 mock members should be visible
    await expect(page.getByText("Arjun").first()).toBeVisible();
    await expect(page.getByText("Claude").first()).toBeVisible();
    await expect(page.getByText("Review Bot").first()).toBeVisible();
  });

  test("add a new member and verify they appear in list and assignee dropdown", async ({ page }) => {
    await navigateTo(page, "members");
    await expect(page.getByText("Team Members", { exact: true })).toBeVisible({ timeout: 5_000 });

    // Click "Add Member" button
    await page.locator("button", { hasText: "Add Member" }).click();

    // The inline form should appear with "New Member" heading
    await expect(page.getByText("New Member", { exact: true })).toBeVisible({ timeout: 3_000 });

    // Fill in the Name field (required) — use exact match to avoid matching "Display Name (optional)"
    await page.getByPlaceholder("Name", { exact: true }).fill("e2euser");

    // Fill in Display Name (optional but we want to verify it)
    await page.getByPlaceholder("Display Name (optional)").fill("E2E User");

    // Fill in Email (optional)
    await page.getByPlaceholder("Email (optional)").fill("e2euser@example.com");

    // Submit by clicking the "Add" button inside the form (last "Add" to avoid "Add Member" trigger)
    await page.locator("button", { hasText: "Add" }).last().click();

    // The form should close (New Member heading gone)
    await expect(page.getByText("New Member", { exact: true })).toBeHidden({ timeout: 5_000 });

    // The new member should appear in the list by display name
    await expect(page.getByText("E2E User").first()).toBeVisible({ timeout: 10_000 });

    // Also verify the email is shown under the display name
    await expect(page.getByText("e2euser@example.com").first()).toBeVisible();

    // Navigate back to the project board
    await navigateTo(page, "project");
    await expect(page.locator("button", { hasText: /^Backlog/ }).first()).toBeVisible({ timeout: 5_000 });

    // Open any issue (KAN-1 has no assignee) to check the assignee dropdown
    await openIssue(page, "KAN-1");

    const panel = page.locator(".rounded-xl.border").first();

    // Open the assignee dropdown — KAN-1 has no assignee so trigger shows "Unassigned"
    const assigneeTrigger = panel.locator("button", { hasText: "Unassigned" }).first();
    await assigneeTrigger.waitFor({ state: "visible" });
    await assigneeTrigger.click();

    // The dropdown popup (absolute positioned) should contain the new member "E2E User"
    const dropdown = panel.locator("div.absolute").first();
    await dropdown.waitFor({ state: "visible" });
    await expect(dropdown.locator("button", { hasText: "E2E User" }).first()).toBeVisible({ timeout: 5_000 });

    // Close the dropdown by pressing Escape
    await page.keyboard.press("Escape");
  });
});
