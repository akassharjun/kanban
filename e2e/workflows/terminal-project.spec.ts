import { test, expect, appReady } from "../fixtures/test-base";

test.describe("Workflow: Terminal with Project Path", () => {
  test.beforeEach(async ({ page }) => {
    await appReady(page);
  });

  test("terminal opens via sidebar button", async ({ page }) => {
    await page.locator("button", { hasText: /^Backlog/ }).waitFor({ state: "visible" });

    // Terminal should not be visible initially
    await expect(page.getByPlaceholder("Type a command...")).not.toBeVisible();

    await page.locator("button", { hasText: "Terminal" }).click();
    await page.getByPlaceholder("Type a command...").waitFor({ state: "visible", timeout: 8_000 });
    await expect(page.getByPlaceholder("Type a command...")).toBeVisible();
  });

  test("terminal shows ready message on open", async ({ page }) => {
    await page.locator("button", { hasText: /^Backlog/ }).waitFor({ state: "visible" });
    await page.locator("button", { hasText: "Terminal" }).click();
    await page.getByPlaceholder("Type a command...").waitFor({ state: "visible", timeout: 8_000 });

    // Terminal should show initial ready message
    await expect(page.getByText("Terminal ready. Type commands below.")).toBeVisible({ timeout: 5_000 });
  });

  test("pwd shows project path", async ({ page }) => {
    await page.locator("button", { hasText: /^Backlog/ }).waitFor({ state: "visible" });
    await page.locator("button", { hasText: "Terminal" }).click();
    await page.getByPlaceholder("Type a command...").waitFor({ state: "visible", timeout: 8_000 });

    await page.getByPlaceholder("Type a command...").fill("pwd");
    await page.keyboard.press("Enter");

    // Mock returns /home/user/kanban for pwd
    await expect(page.getByText("/home/user/kanban").first()).toBeVisible({ timeout: 5_000 });
  });

  test("ls shows file listing", async ({ page }) => {
    await page.locator("button", { hasText: /^Backlog/ }).waitFor({ state: "visible" });
    await page.locator("button", { hasText: "Terminal" }).click();
    await page.getByPlaceholder("Type a command...").waitFor({ state: "visible", timeout: 8_000 });

    await page.getByPlaceholder("Type a command...").fill("ls");
    await page.keyboard.press("Enter");

    await expect(page.getByText("src/  e2e/  docs/  package.json  tsconfig.json")).toBeVisible({ timeout: 5_000 });
  });

  test("help shows available commands", async ({ page }) => {
    await page.locator("button", { hasText: /^Backlog/ }).waitFor({ state: "visible" });
    await page.locator("button", { hasText: "Terminal" }).click();
    await page.getByPlaceholder("Type a command...").waitFor({ state: "visible", timeout: 8_000 });

    await page.getByPlaceholder("Type a command...").fill("help");
    await page.keyboard.press("Enter");

    await expect(page.getByText("Available commands: help, clear, pwd, ls, echo, whoami")).toBeVisible({ timeout: 5_000 });
  });

  test("clear removes previous output", async ({ page }) => {
    await page.locator("button", { hasText: /^Backlog/ }).waitFor({ state: "visible" });
    await page.locator("button", { hasText: "Terminal" }).click();
    await page.getByPlaceholder("Type a command...").waitFor({ state: "visible", timeout: 8_000 });

    // Run help first
    await page.getByPlaceholder("Type a command...").fill("help");
    await page.keyboard.press("Enter");
    await expect(page.getByText("Available commands: help, clear, pwd, ls, echo, whoami")).toBeVisible({ timeout: 5_000 });

    // Now clear
    await page.getByPlaceholder("Type a command...").fill("clear");
    await page.keyboard.press("Enter");

    // Previous output should be gone
    await expect(page.getByText("Available commands: help, clear, pwd, ls, echo, whoami")).not.toBeVisible({ timeout: 3_000 });
  });

  test("close terminal and reopen — fresh state", async ({ page }) => {
    await page.locator("button", { hasText: /^Backlog/ }).waitFor({ state: "visible" });
    await page.locator("button", { hasText: "Terminal" }).click();
    await page.getByPlaceholder("Type a command...").waitFor({ state: "visible", timeout: 8_000 });

    // Run a command
    await page.getByPlaceholder("Type a command...").fill("whoami");
    await page.keyboard.press("Enter");
    await expect(page.getByText("kanban-user")).toBeVisible({ timeout: 5_000 });

    // Close via X button (last button in terminal header)
    const terminalHeader = page.locator(".bg-\\[\\#16162a\\]");
    const closeBtn = terminalHeader.locator("button").last();
    await closeBtn.click();
    await expect(page.getByPlaceholder("Type a command...")).not.toBeVisible({ timeout: 5_000 });

    // Reopen
    await page.locator("button", { hasText: "Terminal" }).click();
    await page.getByPlaceholder("Type a command...").waitFor({ state: "visible", timeout: 8_000 });

    // Should show fresh ready message
    await expect(page.getByText("Terminal ready. Type commands below.")).toBeVisible({ timeout: 5_000 });
  });

  test("project path /home/user/kanban is the project path used in pwd", async ({ page }) => {
    // The project was created with path /home/user/kanban in appReady
    // pwd always returns /home/user/kanban in the mock
    await page.locator("button", { hasText: /^Backlog/ }).waitFor({ state: "visible" });
    await page.locator("button", { hasText: "Terminal" }).click();
    await page.getByPlaceholder("Type a command...").waitFor({ state: "visible", timeout: 8_000 });

    await page.getByPlaceholder("Type a command...").fill("pwd");
    await page.keyboard.press("Enter");

    await expect(page.getByText("/home/user/kanban").first()).toBeVisible({ timeout: 5_000 });
  });
});
