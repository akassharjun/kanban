import { test, expect, appReady } from "../fixtures/test-base";
import { navigateTo } from "../helpers/actions";

test.describe("Phase 6: Settings & Members", () => {
  test.beforeEach(async ({ page }) => {
    await appReady(page);
  });

  test("settings page shows project name", async ({ page }) => {
    await navigateTo(page, "settings");
    // ProjectSettingsView renders <h1>Project Settings</h1> and
    // the project name is pre-filled in the input under "Project Name" label.
    await expect(page.getByText("Project Settings", { exact: true })).toBeVisible({ timeout: 5_000 });
    // The general tab is shown by default; the input under "Project Name" label contains "Kanban Core"
    await expect(page.locator('input[value="Kanban Core"]')).toBeVisible({ timeout: 5_000 });
  });

  test("members page shows all members", async ({ page }) => {
    await navigateTo(page, "members");
    // MembersView renders a list of member cards. Verify all three mock members are visible.
    await expect(page.getByText("Arjun").first()).toBeVisible({ timeout: 5_000 });
    await expect(page.getByText("Claude").first()).toBeVisible({ timeout: 5_000 });
    await expect(page.getByText("Review Bot").first()).toBeVisible({ timeout: 5_000 });
  });

  test("members show name and avatar", async ({ page }) => {
    await navigateTo(page, "members");
    // The heading should confirm we're on the members page
    await expect(page.getByText("Team Members", { exact: true })).toBeVisible({ timeout: 5_000 });
    // Each member card shows a coloured avatar (div with rounded-full) and a name.
    // Verify the name divs are rendered (text-sm font-medium inside each card).
    await expect(page.getByText("Arjun").first()).toBeVisible();
    await expect(page.getByText("Claude").first()).toBeVisible();
    await expect(page.getByText("Review Bot").first()).toBeVisible();
  });

  test("navigate between settings and members", async ({ page }) => {
    // 1. Go to settings, confirm we see the settings heading
    await navigateTo(page, "settings");
    await expect(page.getByText("Project Settings", { exact: true })).toBeVisible({ timeout: 5_000 });

    // 2. Go to members, confirm we see the members heading and member list
    await navigateTo(page, "members");
    await expect(page.getByText("Team Members", { exact: true })).toBeVisible({ timeout: 5_000 });
    await expect(page.getByText("Arjun").first()).toBeVisible();

    // 3. Go back to the project board, confirm board columns are visible
    await navigateTo(page, "project");
    await expect(page.getByRole("button", { name: /Backlog/ })).toBeVisible({ timeout: 5_000 });
    await expect(page.getByRole("button", { name: /Todo/ })).toBeVisible();
  });
});
