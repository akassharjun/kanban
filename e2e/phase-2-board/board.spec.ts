import { test, expect, appReady } from "../fixtures/test-base";
import { createIssue, openIssue } from "../helpers/actions";

test.describe("Phase 2: Board Interaction", () => {
  test.beforeEach(async ({ page }) => {
    await appReady(page);
  });

  test("board shows all status columns", async ({ page }) => {
    await expect(page.getByRole("button", { name: /Backlog/ })).toBeVisible();
    await expect(page.getByRole("button", { name: /Todo/ })).toBeVisible();
    await expect(page.getByRole("button", { name: /In Progress/ })).toBeVisible();
    await expect(page.getByRole("button", { name: /In Review/ })).toBeVisible();
    await expect(page.getByRole("button", { name: /Done/ })).toBeVisible();
  });

  test("issue cards appear on board after creation", async ({ page }) => {
    // Create two issues
    await createIssue(page, { title: "Board Issue Alpha" });
    await createIssue(page, { title: "Board Issue Beta" });

    // Both should appear on the board
    await expect(page.getByText("Board Issue Alpha").first()).toBeVisible();
    await expect(page.getByText("Board Issue Beta").first()).toBeVisible();
  });

  test("issue card shows identifier and title", async ({ page }) => {
    await createIssue(page, { title: "Identifier Test Issue" });

    // Card should show the TST-x identifier and the title
    await expect(page.getByText("TST-", { exact: false }).first()).toBeVisible();
    await expect(page.getByText("Identifier Test Issue").first()).toBeVisible();
  });

  test("click card opens detail panel", async ({ page }) => {
    await createIssue(page, { title: "Click Panel Issue" });

    // Click the issue card
    await page.getByText("Click Panel Issue").first().click();

    // The detail panel has a close button with title="Close (Esc)"
    await expect(page.locator('[title="Close (Esc)"]')).toBeVisible({ timeout: 8_000 });

    // Panel shows the issue title
    await expect(page.getByText("Click Panel Issue").first()).toBeVisible();

    // Panel shows Status and Priority labels
    const panel = page.locator(".rounded-xl.border");
    await expect(panel.getByText("Status", { exact: true })).toBeVisible();
    await expect(panel.getByText("Priority", { exact: true })).toBeVisible();
  });

  test("close detail panel with Escape", async ({ page }) => {
    await createIssue(page, { title: "Escape Close Issue" });
    await page.getByText("Escape Close Issue").first().click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "visible", timeout: 8_000 });

    // Press Escape to close
    await page.keyboard.press("Escape");
    await expect(page.locator('[title="Close (Esc)"]')).toHaveCount(0);
  });

  test("close detail panel by clicking the close button", async ({ page }) => {
    await createIssue(page, { title: "Close Button Issue" });
    await page.getByText("Close Button Issue").first().click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "visible", timeout: 8_000 });

    // Click the X close button
    await page.locator('[title="Close (Esc)"]').click();
    await expect(page.locator('[title="Close (Esc)"]')).toHaveCount(0);
  });
});
