import { expect } from "@playwright/test";
import { test } from "../fixtures/auth";
import { ApiHelper } from "../fixtures/api";

let api: ApiHelper;

test.describe("Timezone handling", () => {
  test.beforeEach(async ({ authenticatedPage }) => {
    api = new ApiHelper(authenticatedPage.context());
    await api.deleteAllTasks();
  });

  test.afterEach(async ({ authenticatedPage: page }) => {
    await api.deleteAllTasks();
    // Reset timezone to UTC
    await page.goto("/settings");
    await page.locator('[data-testid="settings-timezone"]').selectOption("UTC");
    await page.locator('[data-testid="settings-save"]').click();
    await page.waitForTimeout(500);
  });

  test("date picker converts local time to UTC for storage", async ({
    authenticatedPage: page,
  }) => {
    // Set timezone to Asia/Jakarta (UTC+7)
    await page.goto("/settings");
    await page
      .locator('[data-testid="settings-timezone"]')
      .selectOption("Asia/Jakarta");
    await page.locator('[data-testid="settings-save"]').click();
    await page.waitForTimeout(500);

    // Create a task with no start_at
    const task = await api.createTask({ title: "Timezone Test Task" });

    // Navigate to inbox and open task detail modal
    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("e");

    const modal = page.locator('[data-testid="task-detail-modal"]');
    await expect(modal).toBeVisible();

    // Open date picker and set date/time
    await modal.locator('[data-testid="start-date-trigger"]').click();
    await modal.locator('[data-testid="start-date-input"]').fill("2026-01-15");
    await modal.locator('input[type="time"]').fill("14:00");
    await modal.locator('[data-testid="start-date-save"]').click();

    // Wait for the save to propagate
    await page.waitForTimeout(1000);

    // Fetch task via API and verify UTC conversion
    const tasks = await api.listTasks();
    const updated = tasks.find((t) => t.id === task.id);
    expect(updated).toBeDefined();
    expect(updated!.start_at).toBeDefined();

    // 14:00 Jakarta (UTC+7) should be stored as 07:00 UTC
    const storedDate = new Date(updated!.start_at!);
    expect(storedDate.getUTCHours()).toBe(7);
    expect(storedDate.getUTCMinutes()).toBe(0);
    expect(storedDate.getUTCFullYear()).toBe(2026);
    expect(storedDate.getUTCMonth()).toBe(0); // January = 0
    expect(storedDate.getUTCDate()).toBe(15);
  });

  test("start_at is displayed in user's timezone in task meta and modal", async ({
    authenticatedPage: page,
  }) => {
    // Set timezone to Asia/Jakarta (UTC+7)
    await page.goto("/settings");
    await page
      .locator('[data-testid="settings-timezone"]')
      .selectOption("Asia/Jakarta");
    await page.locator('[data-testid="settings-save"]').click();
    await page.waitForTimeout(500);

    // Create task via API with start_at = 07:00 UTC (= 2:00 PM Jakarta)
    await api.createTask({
      title: "Display TZ Task",
      start_at: "2026-01-15T07:00:00Z",
    });

    // Navigate to inbox
    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    // Task meta row should show Jakarta local time (2:00 PM)
    const meta = page.locator('[data-testid="task-meta-start-at"]');
    await expect(meta).toBeVisible();
    await expect(meta).toContainText("2:00 PM");
    await expect(meta).toContainText("Jan 15");

    // Open task detail modal
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("e");

    const modal = page.locator('[data-testid="task-detail-modal"]');
    await expect(modal).toBeVisible();

    // The start-date trigger in the modal sidebar should also show Jakarta local time
    const trigger = modal.locator('[data-testid="start-date-trigger"]');
    await expect(trigger).toBeVisible();
    await expect(trigger).toContainText("2:00 PM");
    await expect(trigger).toContainText("Jan 15");
  });
});
