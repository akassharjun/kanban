import { test, expect, appReady } from "../fixtures/test-base";
import { navigateTo, switchView } from "../helpers/actions";

test.describe("Phase 1: Navigation", () => {
  test.beforeEach(async ({ page }) => {
    await appReady(page);
  });

  test("app renders with sidebar and project name", async ({ page }) => {
    // "Kanban" appears exactly in the sidebar header; use exact to avoid matching "Kanban Core"
    await expect(page.getByText("Kanban", { exact: true })).toBeVisible();
    // "Kanban Core" appears in both the sidebar button and the project heading; .first() is fine
    await expect(page.getByText("Kanban Core").first()).toBeVisible();
  });

  test("navigate to Members page", async ({ page }) => {
    await navigateTo(page, "members");
    // Use exact to avoid matching "arjun@kanban.dev"
    await expect(page.getByText("Arjun", { exact: true })).toBeVisible();
  });

  test("navigate to Settings page", async ({ page }) => {
    await navigateTo(page, "settings");
    await expect(page.getByText("Project Settings")).toBeVisible();
  });

  test("navigate to Agents page", async ({ page }) => {
    await navigateTo(page, "agents");
    // Match the "Agent Ops" heading / section title — use exact to avoid ambiguity
    await expect(page.getByRole("heading", { name: "Agents" })).toBeVisible();
  });

  test("switch to list view", async ({ page }) => {
    await switchView(page, "list");
    // Use exact to avoid matching KAN-10, KAN-11, KAN-12
    await expect(page.getByText("KAN-1", { exact: true })).toBeVisible();
  });

  test("switch to tree view", async ({ page }) => {
    await switchView(page, "tree");
    // Use exact to avoid matching KAN-10, KAN-11, KAN-12
    await expect(page.getByText("KAN-1", { exact: true })).toBeVisible();
  });

  test("switch back to board view", async ({ page }) => {
    await switchView(page, "list");
    await switchView(page, "board");
    // Match the "In Progress" column header button (not a <select> option)
    await expect(page.getByRole("button", { name: /In Progress/ })).toBeVisible();
    await expect(page.getByText("KAN-6", { exact: true })).toBeVisible();
  });

  test("Cmd+B toggles sidebar", async ({ page }) => {
    // Use "Kanban" exact to match only the sidebar app title, not "Kanban Core"
    await expect(page.getByText("Kanban", { exact: true })).toBeVisible();
    // On Linux, Meta maps to the Super/Windows key; use Control+b which the app also accepts
    await page.keyboard.press("Control+b");
    await expect(page.getByText("Projects")).toBeHidden();
    await page.keyboard.press("Control+b");
    await expect(page.getByText("Projects")).toBeVisible();
  });
});
