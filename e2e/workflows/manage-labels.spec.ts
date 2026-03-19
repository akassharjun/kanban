import { test, expect, appReady } from "../fixtures/test-base";
import { createIssue } from "../helpers/actions";

test.describe("Workflow: Manage Issue Labels", () => {
  test.beforeEach(async ({ page }) => {
    await appReady(page);
  });

  async function createLabel(page: import("@playwright/test").Page, name: string, color: string = "#6366f1") {
    // Create a label via Settings > Labels tab
    await page.locator("button", { hasText: "Settings" }).click();
    await page.getByText("Project Settings").waitFor({ state: "visible", timeout: 5_000 });

    // Click the Labels tab in settings
    const labelsTab = page.locator("button", { hasText: "Labels" });
    if (await labelsTab.isVisible()) {
      await labelsTab.click();
    }

    // Look for "Add label" or "New label" button
    const addLabelBtn = page.locator("button", { hasText: /Add label|New label|Create label/i }).first();
    if (await addLabelBtn.isVisible().catch(() => false)) {
      await addLabelBtn.click();
      // Fill in label name
      const nameInput = page.locator("input[placeholder*='label' i], input[placeholder*='name' i]").first();
      if (await nameInput.isVisible().catch(() => false)) {
        await nameInput.fill(name);
        await page.locator("button", { hasText: /Save|Create|Add/i }).last().click();
      }
    }

    // Navigate back to the board
    await page.locator("button", { hasText: "Test Project" }).first().click();
    await page.locator("button", { hasText: /^Backlog/ }).waitFor({ state: "visible", timeout: 8_000 });
  }

  test("labels section is visible in issue detail panel", async ({ page }) => {
    await createIssue(page, { title: "Label Section Issue" });
    await page.getByText("Label Section Issue").first().click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "visible", timeout: 8_000 });

    const panel = page.locator(".rounded-xl.border").first();

    // The Labels row should be visible in the panel properties section
    await expect(panel.getByText("Labels", { exact: true }).first()).toBeVisible({ timeout: 5_000 });
  });

  test("clicking + in labels section opens label picker", async ({ page }) => {
    await createIssue(page, { title: "Label Picker Issue" });
    await page.getByText("Label Picker Issue").first().click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "visible", timeout: 8_000 });

    const panel = page.locator(".rounded-xl.border").first();

    // The Labels row — the Plus (+) button to open the label picker
    const labelsSection = panel.locator("div.relative").filter({ hasText: "Labels" }).first();
    const plusBtn = labelsSection.locator("button").first();
    await plusBtn.waitFor({ state: "visible" });
    await plusBtn.click();

    // The picker dropdown should appear (absolute positioned div)
    const picker = labelsSection.locator("div.absolute");
    await expect(picker).toBeVisible({ timeout: 5_000 });

    // Close picker
    await plusBtn.click();
    await picker.waitFor({ state: "hidden", timeout: 3_000 });
  });

  test("label section handles empty labels gracefully", async ({ page }) => {
    await createIssue(page, { title: "Empty Labels Issue" });
    await page.getByText("Empty Labels Issue").first().click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "visible", timeout: 8_000 });

    const panel = page.locator(".rounded-xl.border").first();

    // Labels section should be present even with no labels
    await expect(panel.getByText("Labels", { exact: true }).first()).toBeVisible({ timeout: 5_000 });

    // Close panel
    await page.locator('[title="Close (Esc)"]').click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "hidden" });
  });
});
