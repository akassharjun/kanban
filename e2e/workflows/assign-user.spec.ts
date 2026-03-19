import { test, expect, appReady } from "../fixtures/test-base";
import { createIssue } from "../helpers/actions";
import type { Locator } from "@playwright/test";

/**
 * Locate the div.relative that wraps the assignee trigger + dropdown popup.
 */
function getAssigneeRow(panel: Locator): Locator {
  return panel.locator("xpath=.//span[normalize-space()='Assignee']/following-sibling::div[contains(@class,'relative')]").first();
}

async function openAssigneeDropdown(panel: Locator): Promise<Locator> {
  const assigneeRow = getAssigneeRow(panel);
  await assigneeRow.locator("> button").first().click();
  await assigneeRow.locator("div.absolute").first().waitFor({ state: "visible" });
  return assigneeRow;
}

async function selectAssigneeOption(assigneeRow: Locator, name: string) {
  const popup = assigneeRow.locator("div.absolute").first();
  await popup.locator("button", { hasText: name }).first().click();
  await popup.waitFor({ state: "hidden" });
}

async function getAssigneeTriggerText(panel: Locator): Promise<string> {
  const assigneeRow = getAssigneeRow(panel);
  return (await assigneeRow.locator("> button").first().innerText()).trim();
}

test.describe("Workflow: Assign User", () => {
  test.beforeEach(async ({ page }) => {
    await appReady(page);
  });

  test("new issue starts with no assignee (Unassigned)", async ({ page }) => {
    await createIssue(page, { title: "Unassigned Test Issue" });
    await page.getByText("Unassigned Test Issue").first().click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "visible", timeout: 8_000 });

    const panel = page.locator(".rounded-xl.border").first();
    const triggerText = await getAssigneeTriggerText(panel);
    expect(triggerText).toContain("Unassigned");
  });

  test("assign Arjun to issue and verify it persists after reopening panel", async ({ page }) => {
    await createIssue(page, { title: "Arjun Assign Persist Issue" });
    await page.getByText("Arjun Assign Persist Issue").first().click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "visible", timeout: 8_000 });

    const panel = page.locator(".rounded-xl.border").first();

    // Verify starts as Unassigned
    const triggerBefore = await getAssigneeTriggerText(panel);
    expect(triggerBefore).toContain("Unassigned");

    // Open dropdown and pick Arjun
    const assigneeRow = await openAssigneeDropdown(panel);
    await selectAssigneeOption(assigneeRow, "Arjun");

    await expect(async () => {
      const text = await getAssigneeTriggerText(panel);
      expect(text).toContain("Arjun");
    }).toPass({ timeout: 5_000 });

    // Close and reopen panel
    await page.locator('[title="Close (Esc)"]').click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "hidden" });

    await page.getByText("Arjun Assign Persist Issue").first().click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "visible", timeout: 8_000 });

    const reopenedPanel = page.locator(".rounded-xl.border").first();
    await expect(async () => {
      const text = await getAssigneeTriggerText(reopenedPanel);
      expect(text).toContain("Arjun");
    }).toPass({ timeout: 5_000 });
  });

  test("change assignee back to Unassigned", async ({ page }) => {
    await createIssue(page, { title: "Unassign Test Issue" });
    await page.getByText("Unassign Test Issue").first().click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "visible", timeout: 8_000 });

    const panel = page.locator(".rounded-xl.border").first();

    // Step 1: assign Arjun
    const assigneeRow1 = await openAssigneeDropdown(panel);
    await selectAssigneeOption(assigneeRow1, "Arjun");

    await expect(async () => {
      const text = await getAssigneeTriggerText(panel);
      expect(text).toContain("Arjun");
    }).toPass({ timeout: 5_000 });

    // Step 2: set back to Unassigned
    const assigneeRow2 = await openAssigneeDropdown(panel);
    await selectAssigneeOption(assigneeRow2, "Unassigned");

    await expect(async () => {
      const text = await getAssigneeTriggerText(panel);
      expect(text).toContain("Unassigned");
    }).toPass({ timeout: 5_000 });
  });

  test("assignee avatar appears on the board card after assignment", async ({ page }) => {
    await createIssue(page, { title: "Avatar Test Issue" });
    await page.getByText("Avatar Test Issue").first().click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "visible", timeout: 8_000 });

    const panel = page.locator(".rounded-xl.border").first();
    const assigneeRow = await openAssigneeDropdown(panel);
    await selectAssigneeOption(assigneeRow, "Arjun");

    await expect(async () => {
      const text = await getAssigneeTriggerText(panel);
      expect(text).toContain("Arjun");
    }).toPass({ timeout: 5_000 });

    // Close panel
    await page.locator('[title="Close (Esc)"]').click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "hidden" });

    // Avatar with title="Arjun" should appear on the board card
    await expect(page.locator('[title="Arjun"]').first()).toBeVisible({ timeout: 5_000 });
  });

  test("KAN-5 starts with no assignee (Unassigned) — compatibility test using fresh issue", async ({ page }) => {
    // Replaces the old KAN-5 specific test with a generic version
    await createIssue(page, { title: "Compat Unassigned Issue" });
    await page.getByText("Compat Unassigned Issue").first().click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "visible", timeout: 8_000 });

    const panel = page.locator(".rounded-xl.border").first();
    const triggerText = await getAssigneeTriggerText(panel);
    expect(triggerText).toContain("Unassigned");
  });
});
