import { test, expect, appReady } from "../fixtures/test-base";
import { openIssue } from "../helpers/actions";

test.describe("Workflow: Manage Issue Labels", () => {
  test.beforeEach(async ({ page }) => {
    await appReady(page);
  });

  test("add and remove labels on KAN-1", async ({ page }) => {
    // KAN-1 starts with "ui" label only
    await openIssue(page, "KAN-1");

    const panel = page.locator(".rounded-xl.border").first();

    // Verify "ui" label badge is visible in the panel
    await expect(panel.getByText("ui", { exact: true }).first()).toBeVisible();

    // The Plus (+) button to open the label picker lives inside the Labels row.
    // The Labels row in the panel is: div.flex.items-start.relative with a "Labels" span child.
    const labelsSection = page.locator("div.relative").filter({ hasText: "Labels" }).first();
    const plusBtn = labelsSection.locator("button").first();
    await plusBtn.waitFor({ state: "visible" });

    // --- Open picker and add "bug" ---
    await plusBtn.click();

    // The picker dropdown renders label buttons inside the panel.
    // Scope to the picker dropdown (absolute positioned div with border bg-popover).
    const picker = labelsSection.locator("div.absolute");
    await picker.waitFor({ state: "visible" });

    // Click "bug" in the picker (it's a button containing the label name)
    await picker.locator("button", { hasText: "bug" }).click();

    // Verify "bug" badge appears in the panel labels area
    await expect(panel.getByText("bug", { exact: true }).first()).toBeVisible({ timeout: 5_000 });

    // --- The picker stays open after clicking a label (React state is preserved).
    // However, loadIssue() causes a re-render — wait for "bug" badge to confirm the reload
    // completed before interacting with the picker again.
    await expect(panel.getByText("bug", { exact: true }).first()).toBeVisible({ timeout: 5_000 });
    // Wait for picker to be visible (it persists through re-render)
    await picker.waitFor({ state: "visible", timeout: 5_000 });
    await picker.locator("button", { hasText: "feature" }).click();

    // Verify "feature" badge appears in the panel
    await expect(panel.getByText("feature", { exact: true }).first()).toBeVisible({ timeout: 5_000 });

    // --- Remove "ui" label (toggle it off) — picker is still open.
    // Wait for "feature" badge to confirm the reload completed.
    await expect(panel.getByText("feature", { exact: true }).first()).toBeVisible({ timeout: 5_000 });
    await picker.waitFor({ state: "visible", timeout: 5_000 });
    await picker.locator("button", { hasText: "ui" }).click();

    // After removing "ui", wait for the update to propagate
    await page.waitForTimeout(500);

    // Close the label picker by clicking the Plus button again (toggle closes it)
    await plusBtn.click();
    await picker.waitFor({ state: "hidden", timeout: 3_000 });

    // Verify the final label state in the panel:
    // - "bug" badge is present
    // - "feature" badge is present
    // - "ui" badge is gone (was removed)
    // The label badges are <span> elements in the labels row (inside the panel, not the picker).
    // Use the labelsSection to scope the check.
    const labelBadges = labelsSection.locator("span");
    await expect(labelBadges.filter({ hasText: "bug" })).toBeVisible({ timeout: 5_000 });
    await expect(labelBadges.filter({ hasText: "feature" })).toBeVisible({ timeout: 5_000 });
    await expect(labelBadges.filter({ hasText: "ui" })).not.toBeVisible({ timeout: 3_000 });

    // Close the panel
    await page.locator('[title="Close (Esc)"]').click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "hidden" });
  });
});
