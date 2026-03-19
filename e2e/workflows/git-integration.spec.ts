import { test, expect, appReady } from "../fixtures/test-base";

test.describe("Workflow: Git Integration (Code Heat Map — Git Tab)", () => {
  test.beforeEach(async ({ page }) => {
    await appReady(page);
    // Navigate to Code Heat Map
    await page.locator("button", { hasText: "Code Heat Map" }).click();
    await page.getByText("Code Heat Map").first().waitFor({ state: "visible" });
    // Click Git tab
    await page.locator("button", { hasText: "Git" }).click();
  });

  test("shows current branch name 'dev'", async ({ page }) => {
    // Wait for git data to load — branch name in status bar
    await page.getByText("dev").first().waitFor({ state: "visible", timeout: 8_000 });
    await expect(page.getByText("dev").first()).toBeVisible();
  });

  test("shows status badges: ahead and uncommitted", async ({ page }) => {
    // Mock returns: ahead=3, uncommitted=2
    await page.getByText(/3 ahead/).waitFor({ state: "visible", timeout: 8_000 });
    await expect(page.getByText(/3 ahead/)).toBeVisible();
    await expect(page.getByText(/2 uncommitted/)).toBeVisible();
  });

  test("shows untracked badge", async ({ page }) => {
    // Mock returns untracked=1
    await page.getByText(/1 untracked/).waitFor({ state: "visible", timeout: 8_000 });
    await expect(page.getByText(/1 untracked/)).toBeVisible();
  });

  test("shows at least one commit with KAN-* badge", async ({ page }) => {
    // Commits section should load and contain KAN-* badges
    await page.getByText("Recent Commits", { exact: false }).waitFor({ state: "visible", timeout: 8_000 });

    // Mock data has commits with KAN-9, KAN-7, KAN-6 references
    // These are rendered as <span> elements with the KAN-* text
    const kanBadge = page.locator("span").filter({ hasText: /^KAN-\d+$/ }).first();
    await kanBadge.waitFor({ state: "visible", timeout: 5_000 });
    await expect(kanBadge).toBeVisible();
  });

  test("shows branches section with 'main' and at least one kan-* branch", async ({ page }) => {
    await page.getByText("Branches", { exact: false }).waitFor({ state: "visible", timeout: 8_000 });

    // 'main' branch should be listed
    await expect(page.getByText("main").first()).toBeVisible();

    // At least one kan-* branch
    const kanBranch = page.getByText(/kan-\d+\//).first();
    await kanBranch.waitFor({ state: "visible", timeout: 5_000 });
    await expect(kanBranch).toBeVisible();
  });

  test("shows worktrees section", async ({ page }) => {
    // Use exact: true to match the section heading span only
    await page.getByText("Worktrees", { exact: true }).first().waitFor({ state: "visible", timeout: 8_000 });
    await expect(page.getByText("Worktrees", { exact: true }).first()).toBeVisible();

    // Mock data: main worktree + two agent worktrees
    // Worktree paths are shown as font-mono text
    await expect(page.getByText("/home/user/kanban", { exact: true })).toBeVisible();
  });
});
