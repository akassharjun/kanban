import { test, expect, appReady } from "../fixtures/test-base";
import { createIssue } from "../helpers/actions";

test.describe("Workflow: Issue Comments", () => {
  test.beforeEach(async ({ page }) => {
    await appReady(page);
  });

  test("add and verify comments on a created issue", async ({ page }) => {
    await createIssue(page, { title: "Comment Test Issue" });
    await page.getByText("Comment Test Issue").first().click();
    await page.locator('[title="Close (Esc)"]').waitFor({ state: "visible", timeout: 8_000 });

    const panel = page.locator(".rounded-xl.border").first();

    // Click the Comments tab
    const commentsTab = panel.locator("button", { hasText: /Comments/ });
    await commentsTab.waitFor({ state: "visible" });
    await commentsTab.click();

    const textarea = panel.locator('textarea[placeholder*="Leave a comment"]');
    await textarea.waitFor({ state: "visible" });

    // Type a first comment
    const firstComment = "E2E test comment number one";
    await textarea.fill(firstComment);

    const submitBtn = panel.locator("button", { hasText: "Comment" }).last();
    await submitBtn.waitFor({ state: "visible" });
    await submitBtn.click();

    // Wait for comment to appear
    await expect(panel.getByText(firstComment, { exact: true })).toBeVisible({ timeout: 5_000 });

    // Wait for textarea to reset
    await expect(textarea).toHaveValue("", { timeout: 3_000 });

    // Add a second comment
    const secondComment = "E2E test comment number two";
    await textarea.fill(secondComment);
    await submitBtn.click();

    // Wait for second comment
    await expect(panel.getByText(secondComment, { exact: true })).toBeVisible({ timeout: 5_000 });

    // Verify both comments are visible
    await expect(panel.getByText(firstComment, { exact: true })).toBeVisible();
    await expect(panel.getByText(secondComment, { exact: true })).toBeVisible();
  });
});
