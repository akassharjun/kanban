import type { Page } from "@playwright/test";

/** Navigate to a sidebar page.
 * Uses locator("button", { hasText }) because sidebar buttons contain
 * Lucide SVG icons alongside text — getByRole({ name }) may not match
 * reliably since accessible name calculation with inline SVGs is inconsistent.
 */
export async function navigateTo(page: Page, target: "project" | "members" | "settings" | "agents") {
  switch (target) {
    case "project":
      // Click the first project in sidebar
      await page.getByText("Kanban Core").click();
      break;
    case "members":
      await page.locator("button", { hasText: "Members" }).click();
      break;
    case "settings":
      await page.locator("button", { hasText: "Settings" }).click();
      break;
    case "agents":
      await page.locator("button", { hasText: "Agent Ops" }).click();
      break;
  }
}

/** Switch the board view mode using keyboard shortcuts (1=board, 2=list, 3=tree).
 * This is more reliable than clicking view switcher buttons since the button
 * labels may be icon-only.
 */
export async function switchView(page: Page, view: "board" | "list" | "tree") {
  const keys: Record<string, string> = { board: "1", list: "2", tree: "3" };
  await page.keyboard.press(keys[view]);
}

/** Open the create issue dialog via keyboard shortcut */
export async function openCreateDialog(page: Page) {
  await page.keyboard.press("c");
  await page.getByPlaceholder("Issue title").waitFor({ state: "visible" });
}

/** Create an issue through the dialog */
export async function createIssue(page: Page, opts: { title: string }) {
  await openCreateDialog(page);
  await page.getByPlaceholder("Issue title").fill(opts.title);
  await page.getByRole("button", { name: "Create" }).click();
  // Wait for dialog to close
  await page.getByPlaceholder("Issue title").waitFor({ state: "hidden" });
}

/** Open an issue detail panel by clicking its card */
export async function openIssue(page: Page, identifier: string) {
  await page.getByText(identifier).first().click();
  // Wait for the panel's close button — its title="Close (Esc)" is unique to the detail panel
  await page.locator('[title="Close (Esc)"]').waitFor({ state: "visible" });
}

/** Open search dialog via Cmd+K */
export async function openSearch(page: Page) {
  await page.keyboard.press("Meta+k");
  await page.getByPlaceholder(/Search issues/).waitFor({ state: "visible" });
}

/** Search for a query in the search dialog */
export async function searchFor(page: Page, query: string) {
  await openSearch(page);
  await page.getByPlaceholder(/Search issues/).fill(query);
}
