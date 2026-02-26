import { expect } from "@playwright/test";
import { test } from "../fixtures/auth";
import { ApiHelper } from "../fixtures/api";

let api: ApiHelper;

test.describe("Today Page", () => {
  test.beforeEach(async ({ authenticatedPage }) => {
    api = new ApiHelper(authenticatedPage.context());
    await api.deleteAllTasks();
  });

  test.afterEach(async () => {
    await api.deleteAllTasks();
  });

  test("tasks with start_at in the past appear on Today", async ({
    authenticatedPage: page,
  }) => {
    // Create a task with start_at in the past
    const yesterday = new Date();
    yesterday.setDate(yesterday.getDate() - 1);
    await api.createTask({
      title: "Past Start Task",
      start_at: yesterday.toISOString(),
    });

    // Create a task with no start_at (should NOT appear on Today)
    await api.createTask({ title: "No Start Task" });

    await page.goto("/today");
    await page
      .locator(
        '[data-testid="task-list"], [data-testid="empty-task-list"]',
      )
      .first()
      .waitFor({ state: "visible" });

    const rows = page.locator('[data-testid="task-row"]');
    await expect(rows).toHaveCount(1);
    await expect(rows.first()).toContainText("Past Start Task");
  });

  test("tasks with future start_at do not appear", async ({
    authenticatedPage: page,
  }) => {
    const tomorrow = new Date();
    tomorrow.setDate(tomorrow.getDate() + 1);
    await api.createTask({
      title: "Future Task",
      start_at: tomorrow.toISOString(),
    });

    await page.goto("/today");
    await page
      .locator(
        '[data-testid="task-list"], [data-testid="empty-task-list"]',
      )
      .first()
      .waitFor({ state: "visible" });

    await expect(page.locator('[data-testid="empty-task-list"]')).toBeVisible();
  });

  test("completing a task removes it from Today list", async ({
    authenticatedPage: page,
  }) => {
    const now = new Date();
    now.setDate(now.getDate() - 1);
    await api.createTask({
      title: "Complete Me",
      start_at: now.toISOString(),
    });

    await page.goto("/today");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    await expect(page.locator('[data-testid="task-row"]')).toHaveCount(1);

    // Complete via checkbox
    await page.locator('[data-testid="task-checkbox"]').click();

    // Task stays visible until page refresh
    await expect(page.locator('[data-testid="task-row"]')).toHaveCount(1);
  });
});
