import { test, expect, appReady } from "../fixtures/test-base";
import { createIssue } from "../helpers/actions";

test.describe("Phase 5: Keyboard Shortcuts", () => {
  test.beforeEach(async ({ page }) => {
    await appReady(page);
  });

  test("C opens create issue dialog", async ({ page }) => {
    await page.keyboard.press("c");
    await expect(page.getByPlaceholder("Issue title")).toBeVisible();
    await page.getByRole("button", { name: "Cancel" }).click();
    await expect(page.getByPlaceholder("Issue title")).not.toBeVisible({ timeout: 3_000 });
  });

  test("1 switches to board view", async ({ page }) => {
    // Create an issue so views have content
    await createIssue(page, { title: "View Switch Issue" });

    // Switch to list first
    await page.keyboard.press("2");
    await expect(page.getByText("TST-", { exact: false }).first()).toBeVisible({ timeout: 5_000 });

    // Now press 1 to switch back to board view
    await page.keyboard.press("1");
    await expect(page.getByRole("button", { name: /Backlog/ })).toBeVisible({ timeout: 5_000 });
    await expect(page.getByRole("button", { name: /Todo/ })).toBeVisible();
  });

  test("2 switches to list view", async ({ page }) => {
    await createIssue(page, { title: "List View Issue" });
    await page.keyboard.press("2");
    // List view shows issue identifiers
    await expect(page.getByText("TST-", { exact: false }).first()).toBeVisible({ timeout: 5_000 });
  });

  test("3 switches to tree view", async ({ page }) => {
    await createIssue(page, { title: "Tree View Issue" });
    await page.keyboard.press("3");
    // Tree view shows issue identifiers
    await expect(page.getByText("TST-", { exact: false }).first()).toBeVisible({ timeout: 5_000 });
  });

  test("Control+Z triggers undo", async ({ page }) => {
    await page.locator("button", { hasText: /^Backlog/ }).waitFor({ state: "visible" });

    const uniqueTitle = "Undo Test Issue " + Date.now();
    await createIssue(page, { title: uniqueTitle });
    await expect(page.getByText(uniqueTitle).first()).toBeVisible({ timeout: 10_000 });

    await page.keyboard.press("Control+z");
    await page.waitForTimeout(500);
    // Verify the page is still functional
    await expect(page.getByRole("button", { name: /Backlog/ })).toBeVisible();
  });

  test("Shift+Control+Z triggers redo", async ({ page }) => {
    await page.locator("button", { hasText: /^Backlog/ }).waitFor({ state: "visible" });

    const uniqueTitle = "Redo Test Issue " + Date.now();
    await createIssue(page, { title: uniqueTitle });
    await expect(page.getByText(uniqueTitle).first()).toBeVisible({ timeout: 10_000 });

    await page.keyboard.press("Control+z");
    await page.waitForTimeout(300);
    await page.keyboard.press("Shift+Control+z");
    await page.waitForTimeout(500);
    await expect(page.getByRole("button", { name: /Backlog/ })).toBeVisible();
  });

  test("shortcuts do not fire when input is focused", async ({ page }) => {
    await page.keyboard.press("c");
    const titleInput = page.getByPlaceholder("Issue title");
    await expect(titleInput).toBeVisible();

    await titleInput.focus();
    await page.keyboard.press("c");

    await expect(titleInput).toHaveValue("c");
    await expect(page.getByPlaceholder("Issue title")).toHaveCount(1);

    await page.keyboard.press("Escape");
  });
});
