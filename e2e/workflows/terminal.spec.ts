import { test, expect, appReady } from "../fixtures/test-base";

test.describe("Workflow: Terminal Panel", () => {
  test.beforeEach(async ({ page }) => {
    await appReady(page);
  });

  test("toggle terminal open via sidebar button", async ({ page }) => {
    // Terminal should not be visible initially
    await expect(page.getByPlaceholder("Type a command...")).not.toBeVisible();

    // Click Terminal in sidebar
    await page.locator("button", { hasText: "Terminal" }).click();

    // Terminal input should appear
    await page.getByPlaceholder("Type a command...").waitFor({ state: "visible" });
    await expect(page.getByPlaceholder("Type a command...")).toBeVisible();
  });

  test("type help and see available commands listed", async ({ page }) => {
    await page.locator("button", { hasText: "Terminal" }).click();
    await page.getByPlaceholder("Type a command...").waitFor({ state: "visible" });

    // Type 'help' and press Enter
    await page.getByPlaceholder("Type a command...").fill("help");
    await page.keyboard.press("Enter");

    // Should show available commands
    await expect(page.getByText("Available commands: help, clear, pwd, ls, echo, whoami")).toBeVisible();
  });

  test("type ls and see file listing", async ({ page }) => {
    await page.locator("button", { hasText: "Terminal" }).click();
    await page.getByPlaceholder("Type a command...").waitFor({ state: "visible" });

    await page.getByPlaceholder("Type a command...").fill("ls");
    await page.keyboard.press("Enter");

    // Should show file listing
    await expect(page.getByText("src/  e2e/  docs/  package.json  tsconfig.json")).toBeVisible();
  });

  test("type clear and output is cleared", async ({ page }) => {
    await page.locator("button", { hasText: "Terminal" }).click();
    await page.getByPlaceholder("Type a command...").waitFor({ state: "visible" });

    // First run help to create some output
    await page.getByPlaceholder("Type a command...").fill("help");
    await page.keyboard.press("Enter");
    await expect(page.getByText("Available commands: help, clear, pwd, ls, echo, whoami")).toBeVisible();

    // Now run clear
    await page.getByPlaceholder("Type a command...").fill("clear");
    await page.keyboard.press("Enter");

    // The help output should be gone
    await expect(page.getByText("Available commands: help, clear, pwd, ls, echo, whoami")).not.toBeVisible();
  });

  test("close terminal with X button", async ({ page }) => {
    await page.locator("button", { hasText: "Terminal" }).click();
    await page.getByPlaceholder("Type a command...").waitFor({ state: "visible" });

    // Find the X (close) button inside the terminal header
    // The terminal header has Minus, Maximize2, and X buttons
    // X is the last button in the header
    const terminalHeader = page.locator(".bg-\\[\\#16162a\\]");
    const closeBtn = terminalHeader.locator("button").last();
    await closeBtn.click();

    // Terminal should be gone
    await expect(page.getByPlaceholder("Type a command...")).not.toBeVisible();
  });

  test("reopen terminal after closing", async ({ page }) => {
    // Open
    await page.locator("button", { hasText: "Terminal" }).click();
    await page.getByPlaceholder("Type a command...").waitFor({ state: "visible" });

    // Run a command to create output
    await page.getByPlaceholder("Type a command...").fill("whoami");
    await page.keyboard.press("Enter");
    await expect(page.getByText("kanban-user")).toBeVisible();

    // Close via X button
    const terminalHeader = page.locator(".bg-\\[\\#16162a\\]");
    const closeBtn = terminalHeader.locator("button").last();
    await closeBtn.click();
    await expect(page.getByPlaceholder("Type a command...")).not.toBeVisible();

    // Reopen — should show initial messages (fresh state)
    await page.locator("button", { hasText: "Terminal" }).click();
    await page.getByPlaceholder("Type a command...").waitFor({ state: "visible" });
    await expect(page.getByText("Terminal ready. Type commands below.")).toBeVisible();
  });
});
