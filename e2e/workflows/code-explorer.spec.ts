import { test, expect, appReady } from "../fixtures/test-base";

test.describe("Workflow: Code Explorer", () => {
  test.beforeEach(async ({ page }) => {
    await appReady(page);
    // Navigate to Code Heat Map page
    await page.locator("button", { hasText: "Code Heat Map" }).click();
    // Wait for the Code Heat Map heading to appear
    await page.getByText("Code Heat Map").first().waitFor({ state: "visible" });
  });

  test("default tab shows Files with file heat data or empty state", async ({ page }) => {
    // The Files tab should be active by default (tab button visible)
    await expect(page.locator("button", { hasText: "Files" }).first()).toBeVisible();
    // Files tab content: either file entries or the empty state message
    // The column headers appear when there are file entries
    const hasEntries = await page.getByText("Issues").first().isVisible().catch(() => false);
    if (!hasEntries) {
      // Empty state
      await expect(page.getByText("No file links found")).toBeVisible();
    } else {
      await expect(page.getByText("Issues").first()).toBeVisible();
    }
  });

  test("click Explorer tab shows directory tree with src, src-tauri, e2e folders", async ({ page }) => {
    await page.locator("button", { hasText: "Explorer" }).click();

    // Tree should show top-level directories
    await expect(page.getByText("src").first()).toBeVisible();
    await expect(page.getByText("src-tauri").first()).toBeVisible();
    await expect(page.getByText("e2e").first()).toBeVisible();
  });

  test("Explorer tab shows PROJECT CONFIG card with CLAUDE.md and AGENTS.md", async ({ page }) => {
    await page.locator("button", { hasText: "Explorer" }).click();

    // Wait for the config card section
    await expect(page.getByText("Project Config", { exact: false })).toBeVisible();

    // CLAUDE.md and AGENTS.md should appear as buttons in the config card
    await expect(page.locator("button", { hasText: "CLAUDE.md" })).toBeVisible();
    await expect(page.locator("button", { hasText: "AGENTS.md" })).toBeVisible();
  });

  test("clicking CLAUDE.md shows content preview with heading", async ({ page }) => {
    await page.locator("button", { hasText: "Explorer" }).click();

    // Wait for config section
    await page.locator("button", { hasText: "CLAUDE.md" }).waitFor({ state: "visible" });

    // Click CLAUDE.md
    await page.locator("button", { hasText: "CLAUDE.md" }).click();

    // Should show file content in the right panel — CLAUDE.md starts with "# Kanban — Project Rules"
    await expect(page.getByText("Kanban", { exact: false }).first()).toBeVisible({ timeout: 8_000 });
    // The h1 from markdown render
    await expect(page.locator("h1").filter({ hasText: /Project Rules|Kanban/ }).first()).toBeVisible({ timeout: 5_000 });
  });

  test("click Directories tab shows directory aggregates", async ({ page }) => {
    await page.locator("button", { hasText: "Directories" }).click();

    // Should show directories tab content: either entries or empty state
    const hasDepthControl = await page.locator("label", { hasText: "Depth:" }).isVisible().catch(() => false);
    expect(hasDepthControl).toBe(true);
  });
});
