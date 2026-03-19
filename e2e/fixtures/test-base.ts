import { test as base, expect, type Page } from "@playwright/test";

/**
 * Create a test project if none exists (onboarding screen visible).
 * Fills Name="Test Project", Prefix="TST", Path="/home/user/kanban".
 * Waits for the board to appear with status columns.
 */
export async function ensureTestProject(page: Page) {
  // Give the app a moment to determine if it has projects
  await page.waitForLoadState("networkidle").catch(() => {});

  const onboardingVisible = await page.getByText("Welcome to Kanban").isVisible().catch(() => false);
  if (onboardingVisible) {
    await page.getByRole("button", { name: "Create Your First Project" }).click();
    // Dialog opens
    await page.getByPlaceholder("My Project").waitFor({ state: "visible" });

    await page.getByPlaceholder("My Project").fill("Test Project");
    // Prefix auto-fills — override to TST
    await page.getByPlaceholder("PRJ").clear();
    await page.getByPlaceholder("PRJ").fill("TST");

    // Path: type /home to trigger autocomplete
    await page.getByPlaceholder("/path/to/your/project").fill("/home/user/kanban");

    // Submit — use exact: true to avoid matching "Create Your First Project" button
    await page.getByRole("button", { name: "Create", exact: true }).click();
  }

  // Wait for board with default statuses to appear
  await page.locator("button", { hasText: /^Backlog/ }).waitFor({ state: "visible", timeout: 15_000 });
}

/**
 * Navigate to the app and wait for it to be ready.
 * If no project exists (empty mock), create one first.
 */
export async function appReady(page: Page) {
  await page.goto("/");
  await ensureTestProject(page);
}

export { base as test, expect };
