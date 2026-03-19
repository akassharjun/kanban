import { test, expect, appReady } from "../fixtures/test-base";
import { openIssue } from "../helpers/actions";

test.describe("Phase 2: Board Interaction", () => {
  test.beforeEach(async ({ page }) => {
    await appReady(page);
    // Default view after loading is the board view
  });

  test("board shows all status columns", async ({ page }) => {
    await expect(page.getByRole("button", { name: /Backlog/ })).toBeVisible();
    await expect(page.getByRole("button", { name: /Todo/ })).toBeVisible();
    await expect(page.getByRole("button", { name: /In Progress/ })).toBeVisible();
    await expect(page.getByRole("button", { name: /In Review/ })).toBeVisible();
    await expect(page.getByRole("button", { name: /Done/ })).toBeVisible();
  });

  test("issue cards appear in correct columns", async ({ page }) => {
    // KAN-6 is In Progress, KAN-3 is Todo, KAN-9 is In Review, KAN-10 is Done
    // Use .first() to avoid strict mode errors when identifier appears in multiple card areas
    await expect(page.getByText("KAN-6", { exact: true }).first()).toBeVisible();
    await expect(page.getByText("KAN-3", { exact: true }).first()).toBeVisible();
    await expect(page.getByText("KAN-9", { exact: true }).first()).toBeVisible();
    await expect(page.getByText("KAN-10", { exact: true }).first()).toBeVisible();
  });

  test("issue card shows identifier and title", async ({ page }) => {
    await expect(page.getByText("KAN-6", { exact: true }).first()).toBeVisible();
    await expect(
      page.getByText("Fix drag-drop position calculation").first()
    ).toBeVisible();
  });

  test("click card opens detail panel", async ({ page }) => {
    // Click the KAN-6 card
    await page.getByText("KAN-6", { exact: true }).first().click();
    // The detail panel has a close button with title="Close (Esc)" — unique to the panel
    await expect(page.locator('[title="Close (Esc)"]')).toBeVisible();
    // The panel also shows the issue title and property labels
    await expect(
      page.getByText("Fix drag-drop position calculation").first()
    ).toBeVisible();
    // "Status" and "Priority" appear as <span> labels inside the panel's properties section
    // Use a CSS selector scoped to the panel container (border-l flex flex-col)
    const panel = page.locator(".rounded-xl.border");
    await expect(panel.getByText("Status", { exact: true })).toBeVisible();
    await expect(panel.getByText("Priority", { exact: true })).toBeVisible();
  });

  test("close detail panel with Escape", async ({ page }) => {
    // Open panel
    await openIssue(page, "KAN-6");
    // Verify panel is open
    await expect(page.locator('[title="Close (Esc)"]')).toBeVisible();
    // Press Escape to close
    await page.keyboard.press("Escape");
    // After close, the panel's close button should be gone from the DOM
    await expect(page.locator('[title="Close (Esc)"]')).toHaveCount(0);
  });

  test("close detail panel by clicking the close button", async ({ page }) => {
    // Open panel
    await openIssue(page, "KAN-6");
    // Verify panel is open
    await expect(page.locator('[title="Close (Esc)"]')).toBeVisible();
    // Click the X close button
    await page.locator('[title="Close (Esc)"]').click();
    // Panel should be closed — close button gone from DOM
    await expect(page.locator('[title="Close (Esc)"]')).toHaveCount(0);
  });
});
