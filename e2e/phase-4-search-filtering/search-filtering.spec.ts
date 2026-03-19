import { test, expect, appReady } from "../fixtures/test-base";
import { openSearch, searchFor, createIssue } from "../helpers/actions";

test.describe("Phase 4: Search & Filtering", () => {
  test.beforeEach(async ({ page }) => {
    await appReady(page);
  });

  test("Cmd+K opens search dialog", async ({ page }) => {
    await openSearch(page);
    await expect(page.getByPlaceholder(/Search issues/)).toBeVisible();
  });

  test("search by title finds matching issues", async ({ page }) => {
    // Create an issue with a searchable title
    await createIssue(page, { title: "Searchable drag issue" });

    await searchFor(page, "drag");
    const dialog = page.locator(".w-\\[560px\\]");
    await expect(
      dialog.getByText("Searchable drag issue", { exact: true })
    ).toBeVisible({ timeout: 5_000 });
  });

  test("search by identifier finds issue", async ({ page }) => {
    // Create an issue and find its identifier
    await createIssue(page, { title: "Identifier Search Issue" });

    // Get the identifier text from the board
    const identifierEl = page.getByText("TST-", { exact: false }).first();
    await identifierEl.waitFor({ state: "visible", timeout: 5_000 });
    const identifier = await identifierEl.textContent();

    // Search by identifier
    await searchFor(page, identifier ?? "TST-1");

    await expect(
      page.getByText("Identifier Search Issue", { exact: true })
    ).toBeVisible({ timeout: 5_000 });
  });

  test("select search result opens detail panel", async ({ page }) => {
    await createIssue(page, { title: "Panel Opener Issue" });

    await searchFor(page, "Panel Opener");
    const dialog = page.locator(".w-\\[560px\\]");
    const result = dialog.getByText("Panel Opener Issue", { exact: true });
    await result.waitFor({ state: "visible", timeout: 5_000 });
    await result.click();

    await page.locator('[title="Close (Esc)"]').waitFor({ state: "visible", timeout: 5_000 });

    const panel = page.locator(".rounded-xl.border").first();
    await expect(panel.locator("h2")).toContainText("Panel Opener Issue");
  });

  test("search with no matches shows empty state", async ({ page }) => {
    await searchFor(page, "zzzznonexistent");
    await expect(
      page.getByText("No results found", { exact: true })
    ).toBeVisible({ timeout: 5_000 });
  });

  test("Escape closes search dialog", async ({ page }) => {
    await openSearch(page);
    await expect(page.getByPlaceholder(/Search issues/)).toBeVisible();
    await page.keyboard.press("Escape");
    await expect(page.getByPlaceholder(/Search issues/)).not.toBeVisible({ timeout: 3_000 });
  });
});
