import { expect } from "@playwright/test";
import { test } from "../fixtures/auth";
import { ApiHelper } from "../fixtures/api";

let api: ApiHelper;

test.describe("Task URL links", () => {
  test.beforeEach(async ({ authenticatedPage }) => {
    api = new ApiHelper(authenticatedPage.context());
    await api.deleteAllTasks();
  });

  test.afterEach(async () => {
    await api.deleteAllTasks();
  });

  test("markdown link in task title renders as clickable link", async ({
    authenticatedPage: page,
  }) => {
    await api.createTask({
      title: "[Example Domain](https://example.com)",
    });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    const taskRow = page.locator('[data-testid="task-row"]').first();
    const link = taskRow.locator('a[href="https://example.com"]');
    await expect(link).toBeVisible();
    await expect(link).toHaveText("Example Domain");
  });

  test("bare URL in title is auto-resolved to a titled markdown link", async ({
    authenticatedPage: page,
  }) => {
    const youtubeUrl =
      "https://www.youtube.com/watch?v=0doATM2dsO8&list=RD0doATM2dsO8&start_radio=1&pp=oAcB";

    await page.goto("/inbox");
    await page
      .locator('[data-testid="empty-task-list"]')
      .waitFor({ state: "visible" });

    // Create task via inline input — this goes through create_with_tokens
    // which triggers background URL resolution
    await page.locator('[data-testid="inbox-add-task"]').click();
    const input = page.locator('[data-testid="inline-create-input"]');
    await expect(input).toBeVisible();
    await input.fill(youtubeUrl);
    await input.press("Enter");

    const taskRow = page.locator('[data-testid="task-row"]').first();
    await expect(taskRow).toBeVisible();

    // Wait for the URL to be fetched and the title to update with a link.
    // The server fetches the page title in the background, then the client
    // polls every 2s until is_url_fetching clears (up to 60s).
    // YouTube URLs reliably resolve to a page title.
    const link = taskRow.locator(`a[href="${youtubeUrl}"]`);
    await expect(link).toBeVisible({ timeout: 30_000 });
    // Title should no longer be the raw URL — it should be the page title
    await expect(link).not.toHaveText(youtubeUrl);
  });
});
