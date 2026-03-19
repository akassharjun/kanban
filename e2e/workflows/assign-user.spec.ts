import { test, expect, appReady } from "../fixtures/test-base";
import { openIssue } from "../helpers/actions";
import type { Locator } from "@playwright/test";

/**
 * Locate the div.relative that wraps the assignee trigger + dropdown popup.
 *
 * The panel renders each metadata row as:
 *   <div class="flex items-center py-1.5 text-sm">
 *     <span class="w-24 ...">Assignee</span>
 *     <div class="relative">          ← this is what we want
 *       <button ...>trigger</button>
 *       <div class="absolute ...">    ← popup (only when open)
 *         <button>Unassigned</button>
 *         <button>Arjun</button>
 *         ...
 *       </div>
 *     </div>
 *   </div>
 *
 * Strategy: find the span with exact text "Assignee", get its parent row div,
 * then find the div.relative sibling.
 */
function getAssigneeRow(panel: Locator): Locator {
  // The panel has several div.relative elements (Status=0, Priority=1, Assignee=2).
  // We use XPath to find the div.relative that is preceded by a span containing "Assignee".
  // XPath: //span[normalize-space()='Assignee']/following-sibling::div[contains(@class,'relative')]
  return panel.locator("xpath=.//span[normalize-space()='Assignee']/following-sibling::div[contains(@class,'relative')]").first();
}

/** Open the assignee dropdown and return the div.relative wrapper. */
async function openAssigneeDropdown(panel: Locator): Promise<Locator> {
  const assigneeRow = getAssigneeRow(panel);
  // Click the direct child button (the trigger)
  await assigneeRow.locator("> button").first().click();
  // Wait for the popup to appear
  await assigneeRow.locator("div.absolute").first().waitFor({ state: "visible" });
  return assigneeRow;
}

/** Select a named option from an open assignee dropdown, then wait for popup to close. */
async function selectAssigneeOption(assigneeRow: Locator, name: string) {
  const popup = assigneeRow.locator("div.absolute").first();
  await popup.locator("button", { hasText: name }).first().click();
  await popup.waitFor({ state: "hidden" });
}

/** Return the current text on the assignee trigger button (trimmed). */
async function getAssigneeTriggerText(panel: Locator): Promise<string> {
  const assigneeRow = getAssigneeRow(panel);
  return (await assigneeRow.locator("> button").first().innerText()).trim();
}

test.describe("Workflow: Assign User", () => {
  test.beforeEach(async ({ page }) => {
    await appReady(page);
  });

  test("KAN-5 starts with no assignee (Unassigned)", async ({ page }) => {
    await openIssue(page, "KAN-5");

    const panel = page.locator(".rounded-xl.border").first();

    // KAN-5 has no assignee; the trigger should contain "Unassigned"
    const triggerText = await getAssigneeTriggerText(panel);
    expect(triggerText).toContain("Unassigned");
  });

  test("assign Arjun to KAN-5 and verify it persists after reopening panel", async ({ page }) => {
    await openIssue(page, "KAN-5");

    const panel = page.locator(".rounded-xl.border").first();

    // Verify starts as Unassigned
    const triggerBefore = await getAssigneeTriggerText(panel);
    expect(triggerBefore).toContain("Unassigned");

    // Open dropdown and pick Arjun
    const assigneeRow = await openAssigneeDropdown(panel);
    await selectAssigneeOption(assigneeRow, "Arjun");

    // Trigger should now show "Arjun"
    await expect(async () => {
      const text = await getAssigneeTriggerText(panel);
      expect(text).toContain("Arjun");
    }).toPass({ timeout: 5_000 });

    // Close the panel
    await page.locator('[title="Close (Esc)"]').click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "hidden" });

    // Reopen KAN-5 — assignee should still be "Arjun" (persisted in mock state)
    await openIssue(page, "KAN-5");

    const reopenedPanel = page.locator(".rounded-xl.border").first();
    await expect(async () => {
      const text = await getAssigneeTriggerText(reopenedPanel);
      expect(text).toContain("Arjun");
    }).toPass({ timeout: 5_000 });
  });

  test("change assignee from Arjun to Claude on KAN-5", async ({ page }) => {
    await openIssue(page, "KAN-5");

    const panel = page.locator(".rounded-xl.border").first();

    // Step 1: assign Arjun first (KAN-5 starts Unassigned in each test)
    const assigneeRow1 = await openAssigneeDropdown(panel);
    await selectAssigneeOption(assigneeRow1, "Arjun");

    await expect(async () => {
      const text = await getAssigneeTriggerText(panel);
      expect(text).toContain("Arjun");
    }).toPass({ timeout: 5_000 });

    // Step 2: change to Claude
    const assigneeRow2 = await openAssigneeDropdown(panel);
    await selectAssigneeOption(assigneeRow2, "Claude");

    // Verify Claude is now shown
    await expect(async () => {
      const text = await getAssigneeTriggerText(panel);
      expect(text).toContain("Claude");
    }).toPass({ timeout: 5_000 });
  });

  test("change assignee back to Unassigned on KAN-5", async ({ page }) => {
    await openIssue(page, "KAN-5");

    const panel = page.locator(".rounded-xl.border").first();

    // Step 1: assign Claude first
    const assigneeRow1 = await openAssigneeDropdown(panel);
    await selectAssigneeOption(assigneeRow1, "Claude");

    await expect(async () => {
      const text = await getAssigneeTriggerText(panel);
      expect(text).toContain("Claude");
    }).toPass({ timeout: 5_000 });

    // Step 2: set back to Unassigned
    const assigneeRow2 = await openAssigneeDropdown(panel);
    await selectAssigneeOption(assigneeRow2, "Unassigned");

    // Trigger should revert to "Unassigned"
    await expect(async () => {
      const text = await getAssigneeTriggerText(panel);
      expect(text).toContain("Unassigned");
    }).toPass({ timeout: 5_000 });
  });

  test("assignee avatar appears on the board card after assignment", async ({ page }) => {
    // Open KAN-5 and assign to Arjun
    await openIssue(page, "KAN-5");

    const panel = page.locator(".rounded-xl.border").first();
    const assigneeRow = await openAssigneeDropdown(panel);
    await selectAssigneeOption(assigneeRow, "Arjun");

    await expect(async () => {
      const text = await getAssigneeTriggerText(panel);
      expect(text).toContain("Arjun");
    }).toPass({ timeout: 5_000 });

    // Close the panel
    await page.locator('[title="Close (Esc)"]').click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "hidden" });

    // The IssueCard renders the assignee avatar div with title={member.display_name || member.name}
    // for the KAN-5 card. After assignment, a [title="Arjun"] element should appear on the board.
    await expect(page.locator('[title="Arjun"]').first()).toBeVisible({ timeout: 5_000 });
  });
});
