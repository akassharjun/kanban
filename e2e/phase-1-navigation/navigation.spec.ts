import { test, expect, appReady } from "../fixtures/test-base";
import { navigateTo, switchView, createIssue } from "../helpers/actions";

test.describe("Phase 1: Navigation", () => {
  test.beforeEach(async ({ page }) => {
    await appReady(page);
  });

  test("app renders with sidebar and project name", async ({ page }) => {
    // "Kanban" appears in the sidebar header
    await expect(page.getByText("Kanban", { exact: true })).toBeVisible();
    // The created project "Test Project" appears in the sidebar button
    await expect(page.locator("button", { hasText: /Test Project/ }).first()).toBeVisible();
  });

  test("navigate to Members page", async ({ page }) => {
    await navigateTo(page, "members");
    // "Arjun" is the default member in the mock (display_name)
    await expect(page.getByText("Arjun", { exact: true }).first()).toBeVisible();
  });

  test("navigate to Settings page", async ({ page }) => {
    await navigateTo(page, "settings");
    await expect(page.getByText("Project Settings")).toBeVisible();
  });

  test("navigate to Agents page", async ({ page }) => {
    await navigateTo(page, "agents");
    await expect(page.getByRole("heading", { name: "Agents" })).toBeVisible();
  });

  test("switch to list view with created issues", async ({ page }) => {
    // Create an issue so list view has something to show
    await createIssue(page, { title: "Nav List View Issue" });
    await switchView(page, "list");
    // In list view, the issue identifier should be visible (TST-1)
    await expect(page.getByText("TST-", { exact: false }).first()).toBeVisible({ timeout: 5_000 });
  });

  test("switch to tree view with created issues", async ({ page }) => {
    await createIssue(page, { title: "Nav Tree View Issue" });
    await switchView(page, "tree");
    // Tree view shows issue identifiers
    await expect(page.getByText("TST-", { exact: false }).first()).toBeVisible({ timeout: 5_000 });
  });

  test("switch back to board view", async ({ page }) => {
    await switchView(page, "list");
    await switchView(page, "board");
    // Board view shows status column buttons
    await expect(page.getByRole("button", { name: /In Progress/ })).toBeVisible();
  });

  test("Cmd+B toggles sidebar", async ({ page }) => {
    await expect(page.getByText("Kanban", { exact: true })).toBeVisible();
    // On Linux, use Control+b
    await page.keyboard.press("Control+b");
    await expect(page.getByText("Projects")).toBeHidden();
    await page.keyboard.press("Control+b");
    await expect(page.getByText("Projects")).toBeVisible();
  });
});
