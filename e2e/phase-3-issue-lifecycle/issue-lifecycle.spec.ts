import { test, expect, appReady } from "../fixtures/test-base";
import { createIssue, openIssue } from "../helpers/actions";

test.describe("Phase 3: Issue Lifecycle", () => {
  test.beforeEach(async ({ page }) => {
    await appReady(page);
  });

  test("create a new issue", async ({ page }) => {
    // Wait for the board columns to be visible — statuses must be loaded before
    // opening the create dialog, otherwise the dialog initializes with status_id=0
    // (which renders the select as empty in the DOM and submits with no status_id).
    await page.locator('button', { hasText: /^Backlog/ }).waitFor({ state: "visible" });

    await createIssue(page, { title: "E2E Test Issue" });
    // The useIssues hook updates local state immediately after create.
    // The new card should appear in the Backlog column (default status).
    await expect(page.getByText("E2E Test Issue").first()).toBeVisible({ timeout: 10_000 });
  });

  test("edit issue title in detail panel", async ({ page }) => {
    await openIssue(page, "KAN-3");

    // The panel is a border-l flex flex-col container
    const panel = page.locator(".border-l").first();

    // Click the h2 title to switch to editing mode
    await panel.locator("h2").click();

    // An input should appear with autoFocus; clear it and type new title
    const titleInput = panel.locator('input.w-full.bg-transparent');
    await titleInput.waitFor({ state: "visible" });
    await titleInput.clear();
    await titleInput.fill("Updated KAN-3 Title");
    await titleInput.press("Enter");

    // After save, h2 should show the new title
    await expect(panel.locator("h2")).toHaveText("Updated KAN-3 Title");
  });

  test("edit issue description in detail panel", async ({ page }) => {
    // KAN-7 appears both in a group card and as a standalone card on the board.
    // Click the standalone card (last occurrence of "KAN-7" text) to avoid
    // clicking the parent group card which would open KAN-8 instead.
    await page.getByText("KAN-7", { exact: true }).last().click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "visible" });

    // Verify that the panel shows KAN-7 (not KAN-8)
    const panel = page.locator(".border-l").first();
    // The panel title is an h2 with class "cursor-pointer text-lg font-semibold"
    await expect(panel.locator("h2.cursor-pointer")).toContainText("Improve issue detail panel UX");

    // The description section renders markdown; find the prose div and click it
    const descView = panel.locator(".prose.cursor-pointer");
    await descView.waitFor({ state: "visible" });
    // Verify original description text is visible
    await expect(descView).toContainText("Improvements needed");

    // Click to enter edit mode
    await descView.click();

    // MentionInput renders a <textarea>; wait for it
    const textarea = panel.locator("textarea").first();
    await textarea.waitFor({ state: "visible" });

    // Clear and type new description
    await textarea.clear();
    await textarea.fill("Updated description via e2e test");

    // Blur to trigger save (handleDescSave is bound to onBlur)
    await textarea.blur();

    // The prose view should return with updated content (first .prose is the description view)
    await expect(panel.locator(".prose.cursor-pointer")).toContainText("Updated description via e2e test");
  });

  test("change issue status via detail panel", async ({ page }) => {
    await openIssue(page, "KAN-3");

    const panel = page.locator(".border-l").first();

    // KAN-3 starts in Todo — click the status dropdown trigger
    const statusTrigger = panel.locator('button', { hasText: "Todo" }).first();
    await statusTrigger.waitFor({ state: "visible" });
    await statusTrigger.click();

    // The dropdown renders status options as buttons; click "In Progress"
    const inProgressOption = panel.locator('button', { hasText: "In Progress" });
    await inProgressOption.first().waitFor({ state: "visible" });
    await inProgressOption.first().click();

    // The dropdown should close and the status label should update
    await expect(panel.locator('button', { hasText: "In Progress" }).first()).toBeVisible();
  });

  test("assign member to issue", async ({ page }) => {
    await openIssue(page, "KAN-5");

    const panel = page.locator(".border-l").first();

    // KAN-5 has no assignee — the trigger shows "Unassigned"
    const assigneeTrigger = panel.locator('button', { hasText: "Unassigned" }).first();
    await assigneeTrigger.waitFor({ state: "visible" });
    await assigneeTrigger.click();

    // The dropdown lists members; click "Arjun" (display_name)
    const arjunOption = panel.locator('button', { hasText: "Arjun" });
    await arjunOption.first().waitFor({ state: "visible" });
    await arjunOption.first().click();

    // After selection the assignee trigger should now show "Arjun"
    await expect(panel.locator('button', { hasText: "Arjun" }).first()).toBeVisible();
  });

  test("activity log shows changes", async ({ page }) => {
    await openIssue(page, "KAN-6");

    const panel = page.locator(".border-l").first();

    // Click the Activity tab
    const activityTab = panel.locator('button', { hasText: "Activity" });
    await activityTab.waitFor({ state: "visible" });
    await activityTab.click();

    // The mock backend returns 2 activity entries; each shows "Changed <field>"
    // Wait for at least one entry to appear
    await expect(panel.getByText("Changed", { exact: false }).first()).toBeVisible();
  });
});
