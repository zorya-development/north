import { expect } from "@playwright/test";
import { test } from "../fixtures/auth";
import { ApiHelper } from "../fixtures/api";

let api: ApiHelper;

test.describe("Review Page", () => {
  test.beforeEach(async ({ authenticatedPage }) => {
    api = new ApiHelper(authenticatedPage.context());
    await api.deleteAllTasks();
  });

  test.afterEach(async () => {
    await api.deleteAllTasks();
  });

  test("overdue-for-review tasks appear", async ({
    authenticatedPage: page,
  }) => {
    // Create a task — it will have no reviewed_at, so it should be due for review
    await api.createTask({ title: "Needs Review" });

    await page.goto("/review");
    await page
      .locator(
        '[data-testid="task-list"], [data-testid="empty-task-list"]',
      )
      .first()
      .waitFor({ state: "visible" });

    const rows = page.locator('[data-testid="task-row"]');
    await expect(rows).toHaveCount(1);
    await expect(rows.first()).toContainText("Needs Review");
  });

  test("R key marks selected task as reviewed", async ({
    authenticatedPage: page,
  }) => {
    await api.createTask({ title: "Review Me" });

    await page.goto("/review");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    await expect(page.locator('[data-testid="task-row"]')).toHaveCount(1);

    // Select task and press R
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("r");

    // Task should disappear from review list
    await expect(page.locator('[data-testid="task-row"]')).toHaveCount(0);
  });

  test("Mark All Reviewed clears the list", async ({
    authenticatedPage: page,
  }) => {
    await api.createTask({ title: "Review A" });
    await api.createTask({ title: "Review B" });

    await page.goto("/review");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    await expect(page.locator('[data-testid="task-row"]')).toHaveCount(2);

    // Click Mark All as Reviewed
    await page.locator('[data-testid="review-mark-all"]').click();

    // All tasks should disappear
    await expect(page.locator('[data-testid="task-row"]')).toHaveCount(0);
  });

  test("show recently reviewed toggle", async ({
    authenticatedPage: page,
  }) => {
    // Create and review a task via API
    const task = await api.createTask({ title: "Already Reviewed" });
    await api.reviewTask(task.id);

    await page.goto("/review");
    await page
      .locator(
        '[data-testid="task-list"], [data-testid="empty-task-list"]',
      )
      .first()
      .waitFor({ state: "visible" });

    // The reviewed task should not appear in the main list
    await expect(page.locator('[data-testid="task-row"]')).toHaveCount(0);

    // Toggle show recently reviewed
    await page.locator('[data-testid="review-toggle-recent"]').click();

    // The reviewed task should now be visible in the recently reviewed section
    await expect(page.locator('[data-testid="task-row"]')).toHaveCount(1);
    await expect(
      page.locator('[data-testid="task-row"]').first(),
    ).toContainText("Already Reviewed");

    // Toggle off
    await page.locator('[data-testid="review-toggle-recent"]').click();
    await expect(page.locator('[data-testid="task-row"]')).toHaveCount(0);
  });
});
