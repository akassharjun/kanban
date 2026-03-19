import { test, expect, appReady } from "../fixtures/test-base";
import { openSearch, createIssue } from "../helpers/actions";

test.describe("Workflow: Filter and Search", () => {
  test.beforeEach(async ({ page }) => {
    await appReady(page);
    await page.locator("button", { hasText: /^Backlog/ }).waitFor({ state: "visible" });
  });

  test("filter by status Backlog shows only Backlog issues", async ({ page }) => {
    // Create issues in Backlog (default) and In Progress
    await createIssue(page, { title: "Backlog Issue Filter" });

    // Open create dialog and create an In Progress issue
    await page.keyboard.press("c");
    await page.getByPlaceholder("Issue title").waitFor({ state: "visible" });
    await page.getByPlaceholder("Issue title").fill("In Progress Issue Filter");
    const statusDiv = page.locator("div").filter({ has: page.locator("label", { hasText: /^Status$/ }) }).last();
    await statusDiv.locator("select").selectOption({ label: "In Progress" });
    await page.getByRole("button", { name: "Create", exact: true }).click();
    await page.getByPlaceholder("Issue title").waitFor({ state: "hidden" });

    // Now filter by Backlog
    const statusSelect = page.locator("select").first();
    await statusSelect.waitFor({ state: "visible" });
    await statusSelect.selectOption({ label: "Backlog" });

    const board = page.locator(".overflow-x-auto").first();

    // Backlog issue should be visible, In Progress should not
    await expect(board.getByText("Backlog Issue Filter").first()).toBeVisible({ timeout: 5_000 });
    await expect(board.getByText("In Progress Issue Filter")).not.toBeVisible({ timeout: 3_000 });
  });

  test("filter by status In Progress shows only In Progress issues", async ({ page }) => {
    await createIssue(page, { title: "Backlog Only Issue" });

    // Create an In Progress issue
    await page.keyboard.press("c");
    await page.getByPlaceholder("Issue title").waitFor({ state: "visible" });
    await page.getByPlaceholder("Issue title").fill("Progress Only Issue");
    const statusDiv = page.locator("div").filter({ has: page.locator("label", { hasText: /^Status$/ }) }).last();
    await statusDiv.locator("select").selectOption({ label: "In Progress" });
    await page.getByRole("button", { name: "Create", exact: true }).click();
    await page.getByPlaceholder("Issue title").waitFor({ state: "hidden" });

    const statusSelect = page.locator("select").first();
    await statusSelect.waitFor({ state: "visible" });
    await statusSelect.selectOption({ label: "In Progress" });

    const board = page.locator(".overflow-x-auto").first();

    await expect(board.getByText("Progress Only Issue").first()).toBeVisible({ timeout: 5_000 });
    await expect(board.getByText("Backlog Only Issue")).not.toBeVisible({ timeout: 3_000 });
  });

  test("clear status filter shows all issues", async ({ page }) => {
    await createIssue(page, { title: "Clear Filter Backlog Issue" });

    await page.keyboard.press("c");
    await page.getByPlaceholder("Issue title").waitFor({ state: "visible" });
    await page.getByPlaceholder("Issue title").fill("Clear Filter Progress Issue");
    const statusDiv = page.locator("div").filter({ has: page.locator("label", { hasText: /^Status$/ }) }).last();
    await statusDiv.locator("select").selectOption({ label: "In Progress" });
    await page.getByRole("button", { name: "Create", exact: true }).click();
    await page.getByPlaceholder("Issue title").waitFor({ state: "hidden" });

    const statusSelect = page.locator("select").first();
    await statusSelect.waitFor({ state: "visible" });
    await statusSelect.selectOption({ label: "Backlog" });

    const board = page.locator(".overflow-x-auto").first();
    await expect(board.getByText("Clear Filter Progress Issue")).not.toBeVisible({ timeout: 3_000 });

    // Clear filter
    await statusSelect.selectOption({ label: "All statuses" });

    await expect(board.getByText("Clear Filter Backlog Issue").first()).toBeVisible({ timeout: 5_000 });
    await expect(board.getByText("Clear Filter Progress Issue").first()).toBeVisible({ timeout: 5_000 });
  });

  test("filter by priority High shows only High priority issues", async ({ page }) => {
    // Create high priority issue
    await page.keyboard.press("c");
    await page.getByPlaceholder("Issue title").waitFor({ state: "visible" });
    await page.getByPlaceholder("Issue title").fill("High Priority Issue");
    const priorityDiv = page.locator("div").filter({ has: page.locator("label", { hasText: /^Priority$/ }) }).last();
    await priorityDiv.locator("select").selectOption("high");
    await page.getByRole("button", { name: "Create", exact: true }).click();
    await page.getByPlaceholder("Issue title").waitFor({ state: "hidden" });

    // Create a low priority issue
    await createIssue(page, { title: "Low Priority Issue" });

    const selects = page.locator("select");
    const prioritySelect = selects.nth(1);
    await prioritySelect.waitFor({ state: "visible" });
    await prioritySelect.selectOption({ label: "High" });

    const board = page.locator(".overflow-x-auto").first();

    await expect(board.getByText("High Priority Issue").first()).toBeVisible({ timeout: 5_000 });
    await expect(board.getByText("Low Priority Issue")).not.toBeVisible({ timeout: 3_000 });
  });

  test("filter by assignee Arjun shows only Arjun's issues", async ({ page }) => {
    // Create issue assigned to Arjun
    await page.keyboard.press("c");
    await page.getByPlaceholder("Issue title").waitFor({ state: "visible" });
    await page.getByPlaceholder("Issue title").fill("Arjun Assigned Issue");
    const assigneeDiv = page.locator("div").filter({ has: page.locator("label", { hasText: /^Assignee$/ }) }).last();
    await assigneeDiv.locator("select").selectOption({ label: "Arjun" });
    await page.getByRole("button", { name: "Create", exact: true }).click();
    await page.getByPlaceholder("Issue title").waitFor({ state: "hidden" });

    // Create unassigned issue
    await createIssue(page, { title: "Unassigned Issue" });

    const selects = page.locator("select");
    const assigneeSelect = selects.nth(2);
    await assigneeSelect.waitFor({ state: "visible" });
    await assigneeSelect.selectOption({ label: "Arjun" });

    const board = page.locator(".overflow-x-auto").first();

    await expect(board.getByText("Arjun Assigned Issue").first()).toBeVisible({ timeout: 5_000 });
    await expect(board.getByText("Unassigned Issue")).not.toBeVisible({ timeout: 3_000 });
  });

  test("search for issue by partial title", async ({ page }) => {
    await createIssue(page, { title: "Unique Searchable Title" });

    // Open search dialog
    await openSearch(page);
    const searchInput = page.getByPlaceholder(/Search issues/);
    await searchInput.waitFor({ state: "visible" });
    await searchInput.fill("Unique Searchable");

    const dialog = page.locator(".w-\\[560px\\]");
    const result = dialog.getByText("Unique Searchable Title", { exact: true });
    await result.waitFor({ state: "visible", timeout: 5_000 });

    await result.click();

    await page.locator('[title="Close (Esc)"]').waitFor({ state: "visible", timeout: 5_000 });
    const panel = page.locator(".rounded-xl.border").first();
    await expect(panel.locator("h2")).toContainText("Unique Searchable Title");
  });
});
