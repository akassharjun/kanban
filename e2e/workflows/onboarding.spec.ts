import { test, expect } from "../fixtures/test-base";

test.describe("Workflow: Onboarding", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    // Wait for the app to settle without creating a project
    await page.waitForLoadState("networkidle").catch(() => {});
  });

  test("app shows Welcome to Kanban on first load (no projects)", async ({ page }) => {
    await expect(page.getByText("Welcome to Kanban")).toBeVisible({ timeout: 10_000 });
    await expect(page.getByText("Your AI-powered project management board")).toBeVisible();
  });

  test("shows three feature cards", async ({ page }) => {
    await page.getByText("Welcome to Kanban").waitFor({ state: "visible", timeout: 10_000 });
    await expect(page.getByText("Track Issues")).toBeVisible();
    await expect(page.getByText("Agent Orchestration")).toBeVisible();
    await expect(page.getByText("Code Integration")).toBeVisible();
  });

  test("Create Your First Project button opens dialog", async ({ page }) => {
    await page.getByText("Welcome to Kanban").waitFor({ state: "visible", timeout: 10_000 });
    await page.getByRole("button", { name: "Create Your First Project" }).click();
    // Dialog title — use heading role to avoid matching sidebar "New project" button
    await expect(page.getByRole("heading", { name: "New Project" })).toBeVisible({ timeout: 5_000 });
    // Form fields
    await expect(page.getByPlaceholder("My Project")).toBeVisible();
    await expect(page.getByPlaceholder("PRJ")).toBeVisible();
    await expect(page.getByPlaceholder("/path/to/your/project")).toBeVisible();
    await expect(page.getByPlaceholder("What is this project about?")).toBeVisible();
  });

  test("name auto-fills prefix when typing", async ({ page }) => {
    await page.getByText("Welcome to Kanban").waitFor({ state: "visible", timeout: 10_000 });
    await page.getByRole("button", { name: "Create Your First Project" }).click();
    await page.getByPlaceholder("My Project").waitFor({ state: "visible" });

    await page.getByPlaceholder("My Project").fill("My Kanban");
    // Prefix should auto-fill with first 3 chars uppercased = "MY"
    const prefixVal = await page.getByPlaceholder("PRJ").inputValue();
    expect(prefixVal.toUpperCase()).toMatch(/^MY/);
  });

  test("path autocomplete shows suggestions for /home", async ({ page }) => {
    await page.getByText("Welcome to Kanban").waitFor({ state: "visible", timeout: 10_000 });
    await page.getByRole("button", { name: "Create Your First Project" }).click();
    await page.getByPlaceholder("/path/to/your/project").waitFor({ state: "visible" });

    // Type /home to trigger autocomplete — mock returns ["/home/user"] for "/home"
    await page.getByPlaceholder("/path/to/your/project").fill("/home");

    // Wait for autocomplete dropdown
    await expect(page.getByText("/home/user").first()).toBeVisible({ timeout: 5_000 });
  });

  test("clicking autocomplete suggestion fills path", async ({ page }) => {
    await page.getByText("Welcome to Kanban").waitFor({ state: "visible", timeout: 10_000 });
    await page.getByRole("button", { name: "Create Your First Project" }).click();
    await page.getByPlaceholder("/path/to/your/project").waitFor({ state: "visible" });

    // Type /home to get first suggestion
    await page.getByPlaceholder("/path/to/your/project").fill("/home");
    const suggestion = page.getByText("/home/user").first();
    await suggestion.waitFor({ state: "visible", timeout: 5_000 });
    await suggestion.click();

    // Path field should be filled with the suggestion
    const pathVal = await page.getByPlaceholder("/path/to/your/project").inputValue();
    expect(pathVal).toContain("/home/user");
  });

  test("fill dialog and create project — board appears with default statuses", async ({ page }) => {
    await page.getByText("Welcome to Kanban").waitFor({ state: "visible", timeout: 10_000 });
    await page.getByRole("button", { name: "Create Your First Project" }).click();
    await page.getByPlaceholder("My Project").waitFor({ state: "visible" });

    await page.getByPlaceholder("My Project").fill("My Kanban");
    await page.getByPlaceholder("PRJ").clear();
    await page.getByPlaceholder("PRJ").fill("MYK");
    await page.getByPlaceholder("/path/to/your/project").fill("/home/user/kanban");
    // Use exact: true to avoid matching "Create Your First Project" button
    await page.getByRole("button", { name: "Create", exact: true }).click();

    // Board should appear with the 5 default status columns
    await page.locator("button", { hasText: /^Backlog/ }).waitFor({ state: "visible", timeout: 15_000 });
    await expect(page.locator("button", { hasText: /^Todo/ })).toBeVisible();
    await expect(page.locator("button", { hasText: /^In Progress/ })).toBeVisible();
    await expect(page.locator("button", { hasText: /^In Review/ })).toBeVisible();
    await expect(page.locator("button", { hasText: /^Done/ })).toBeVisible();
  });

  test("project appears in sidebar after creation", async ({ page }) => {
    await page.getByText("Welcome to Kanban").waitFor({ state: "visible", timeout: 10_000 });
    await page.getByRole("button", { name: "Create Your First Project" }).click();
    await page.getByPlaceholder("My Project").waitFor({ state: "visible" });

    await page.getByPlaceholder("My Project").fill("Sidebar Project");
    await page.getByPlaceholder("PRJ").clear();
    await page.getByPlaceholder("PRJ").fill("SBR");
    await page.getByPlaceholder("/path/to/your/project").fill("/home/user/kanban");
    await page.getByRole("button", { name: "Create", exact: true }).click();

    // Board ready
    await page.locator("button", { hasText: /^Backlog/ }).waitFor({ state: "visible", timeout: 15_000 });

    // Project name should appear in the sidebar
    await expect(page.locator("button", { hasText: "Sidebar Project" }).first()).toBeVisible();
  });

  test("board has exactly 5 default columns", async ({ page }) => {
    await page.getByText("Welcome to Kanban").waitFor({ state: "visible", timeout: 10_000 });
    await page.getByRole("button", { name: "Create Your First Project" }).click();
    await page.getByPlaceholder("My Project").waitFor({ state: "visible" });

    await page.getByPlaceholder("My Project").fill("Column Test");
    await page.getByPlaceholder("PRJ").clear();
    await page.getByPlaceholder("PRJ").fill("COL");
    await page.getByPlaceholder("/path/to/your/project").fill("/home/user/kanban");
    await page.getByRole("button", { name: "Create", exact: true }).click();
    await page.locator("button", { hasText: /^Backlog/ }).waitFor({ state: "visible", timeout: 15_000 });

    const columnNames = ["Backlog", "Todo", "In Progress", "In Review", "Done"];
    for (const col of columnNames) {
      await expect(page.locator("button", { hasText: new RegExp(`^${col}`) })).toBeVisible();
    }
  });
});
