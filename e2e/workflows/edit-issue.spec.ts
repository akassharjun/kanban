import { test, expect, appReady } from "../fixtures/test-base";
import { createIssue } from "../helpers/actions";

test.describe("Workflow: Edit Issue", () => {
  test.beforeEach(async ({ page }) => {
    await appReady(page);
  });

  test("full edit issue workflow", async ({ page }) => {
    // Create a fresh issue to edit
    await createIssue(page, { title: "Issue to Edit" });

    // Open the issue detail panel
    await page.getByText("Issue to Edit").first().click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "visible", timeout: 8_000 });

    const panel = page.locator(".rounded-xl.border").first();

    // --- Edit title ---
    await panel.locator("h2").click();
    const titleInput = panel.locator("input.w-full.bg-transparent");
    await titleInput.waitFor({ state: "visible" });
    await titleInput.clear();
    await titleInput.fill("Updated Title");
    await titleInput.press("Enter");

    // Verify h2 updates
    await expect(panel.locator("h2")).toHaveText("Updated Title");

    // --- Edit description ---
    const descView = panel.locator(".prose.cursor-pointer");
    await descView.waitFor({ state: "visible" });
    await descView.click();

    const textarea = panel.locator("textarea").first();
    await textarea.waitFor({ state: "visible" });
    await textarea.clear();
    await textarea.fill("New description text");
    await textarea.blur();

    // Verify prose updates
    await expect(panel.locator(".prose.cursor-pointer")).toContainText("New description text");

    // --- Change status: Backlog -> In Progress ---
    const statusTrigger = panel.locator("button", { hasText: "Backlog" }).first();
    await statusTrigger.waitFor({ state: "visible" });
    await statusTrigger.click();

    const inProgressOption = panel.locator("button", { hasText: "In Progress" });
    await inProgressOption.first().waitFor({ state: "visible" });
    await inProgressOption.first().click();

    // Verify status shows "In Progress"
    await expect(panel.locator("button", { hasText: "In Progress" }).first()).toBeVisible();

    // --- Change priority: None -> High ---
    const priorityTrigger = panel.locator("button", { hasText: /none/i }).first();
    await priorityTrigger.waitFor({ state: "visible" });
    await priorityTrigger.click();

    const highOption = panel.locator("button", { hasText: /^High$/i });
    await highOption.first().waitFor({ state: "visible" });
    await highOption.first().click();

    // Verify priority shows "High"
    await expect(panel.locator("button", { hasText: /^High$/i }).first()).toBeVisible();

    // --- Assign member: Unassigned -> Arjun ---
    const assigneeTrigger = panel.locator("button", { hasText: "Unassigned" }).first();
    await assigneeTrigger.waitFor({ state: "visible" });
    await assigneeTrigger.click();

    const arjunOption = panel.locator("button", { hasText: "Arjun" });
    await arjunOption.first().waitFor({ state: "visible" });
    await arjunOption.first().click();

    // Verify shows "Arjun"
    await expect(panel.locator("button", { hasText: "Arjun" }).first()).toBeVisible();

    // --- Change due date ---
    const dateInput = panel.locator('input[type="date"]');
    await dateInput.waitFor({ state: "visible" });
    await dateInput.fill("2026-04-15");
    await dateInput.blur();
    await expect(dateInput).toHaveValue("2026-04-15");

    // --- Close panel and verify card moved to "In Progress" column ---
    await page.locator('[title="Close (Esc)"]').click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "hidden" });

    // The card "Updated Title" should now appear in In Progress column area
    await expect(page.getByText("Updated Title").first()).toBeVisible({ timeout: 10_000 });
  });
});
