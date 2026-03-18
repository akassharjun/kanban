import { test as base, expect, type Page } from "@playwright/test";

/**
 * Navigate to the app and wait for it to be ready.
 * The mock backend has a 30ms delay per command, so we wait for
 * the sidebar project name to appear as a signal the app is loaded.
 */
export async function appReady(page: Page) {
  await page.goto("/");
  // Wait for the sidebar to render with the first project name
  await page.getByText("Kanban Core").waitFor({ state: "visible", timeout: 10_000 });
}

export { base as test, expect };
