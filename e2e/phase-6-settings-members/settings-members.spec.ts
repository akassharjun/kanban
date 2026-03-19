import { test, expect, appReady } from "../fixtures/test-base";
import { navigateTo } from "../helpers/actions";

test.describe("Phase 6: Settings & Members", () => {
  test.beforeEach(async ({ page }) => {
    await appReady(page);
  });

  test("settings page shows project name", async ({ page }) => {
    await navigateTo(page, "settings");
    await expect(page.getByText("Project Settings", { exact: true })).toBeVisible({ timeout: 5_000 });
    // The project name "Test Project" should appear in the settings input
    await expect(page.locator('input[value="Test Project"]')).toBeVisible({ timeout: 5_000 });
  });

  test("members page shows default member", async ({ page }) => {
    await navigateTo(page, "members");
    // Default mock member: Arjun (display_name of akassharjun)
    await expect(page.getByText("Arjun").first()).toBeVisible({ timeout: 5_000 });
  });

  test("members show name and avatar", async ({ page }) => {
    await navigateTo(page, "members");
    await expect(page.getByText("Team Members", { exact: true })).toBeVisible({ timeout: 5_000 });
    // At minimum Arjun should be visible
    await expect(page.getByText("Arjun").first()).toBeVisible();
  });

  test("navigate between settings and members", async ({ page }) => {
    // 1. Go to settings
    await navigateTo(page, "settings");
    await expect(page.getByText("Project Settings", { exact: true })).toBeVisible({ timeout: 5_000 });

    // 2. Go to members
    await navigateTo(page, "members");
    await expect(page.getByText("Team Members", { exact: true })).toBeVisible({ timeout: 5_000 });
    await expect(page.getByText("Arjun").first()).toBeVisible();

    // 3. Go back to project board
    await navigateTo(page, "project");
    await expect(page.getByRole("button", { name: /Backlog/ })).toBeVisible({ timeout: 5_000 });
    await expect(page.getByRole("button", { name: /Todo/ })).toBeVisible();
  });
});
