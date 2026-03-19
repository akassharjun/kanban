import { test, expect, appReady } from "../fixtures/test-base";
import { openIssue } from "../helpers/actions";

test.describe("Workflow: Issue Comments", () => {
  test.beforeEach(async ({ page }) => {
    await appReady(page);
  });

  test("add and verify comments on KAN-6", async ({ page }) => {
    await openIssue(page, "KAN-6");

    const panel = page.locator(".rounded-xl.border").first();

    // The Comments tab is the default active tab — it should already be showing.
    // Confirm by clicking the Comments tab explicitly
    const commentsTab = panel.locator("button", { hasText: /Comments/ });
    await commentsTab.waitFor({ state: "visible" });
    await commentsTab.click();

    // The comment textarea placeholder mentions "Leave a comment"
    // Use the placeholder to locate it — more resilient to re-renders
    const textarea = panel.locator('textarea[placeholder*="Leave a comment"]');
    await textarea.waitFor({ state: "visible" });

    // Type a first comment
    const firstComment = "E2E test comment number one";
    await textarea.fill(firstComment);

    // Click the "Comment" button to submit
    // The button is enabled only when textarea is non-empty
    const submitBtn = panel.locator("button", { hasText: "Comment" }).last();
    await submitBtn.waitFor({ state: "visible" });
    await submitBtn.click();

    // Wait for the comment to appear in the list before proceeding
    await expect(panel.getByText(firstComment, { exact: true })).toBeVisible({ timeout: 5_000 });

    // Wait for textarea to be empty (reset after submit) before typing second comment
    await expect(textarea).toHaveValue("", { timeout: 3_000 });

    // Add a second comment
    const secondComment = "E2E test comment number two";
    await textarea.fill(secondComment);
    await submitBtn.click();

    // Wait for second comment to appear
    await expect(panel.getByText(secondComment, { exact: true })).toBeVisible({ timeout: 5_000 });

    // Verify both comments are visible
    await expect(panel.getByText(firstComment, { exact: true })).toBeVisible();
    await expect(panel.getByText(secondComment, { exact: true })).toBeVisible();
  });
});
