import { expect } from "@playwright/test";
import { test } from "../fixtures/auth";
import { ApiHelper } from "../fixtures/api";

let api: ApiHelper;

test.describe("Someday Feature", () => {
  test.beforeEach(async ({ authenticatedPage }) => {
    api = new ApiHelper(authenticatedPage.context());
    await api.deleteAllTasks();
  });

  test.afterEach(async () => {
    await api.deleteAllTasks();
  });

  test("S key toggles task to someday", async ({
    authenticatedPage: page,
  }) => {
    await api.createTask({ title: "Normal Task" });
    await page.goto("/tasks");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    // Select task and press S
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("s");

    // Task should show @Someday prefix
    const row = page.locator('[data-testid="task-row"]').first();
    await expect(row).toContainText("@Someday");
  });

  test("someday page shows only someday tasks", async ({
    authenticatedPage: page,
  }) => {
    const task = await api.createTask({ title: "Someday Task" });
    await api.updateTask(task.id, { someday: true });
    await api.createTask({ title: "Normal Task" });

    await page.goto("/someday");
    await page
      .locator(
        '[data-testid="task-list"], [data-testid="empty-task-list"]',
      )
      .first()
      .waitFor({ state: "visible" });

    const rows = page.locator('[data-testid="task-row"]');
    await expect(rows).toHaveCount(1);
    await expect(rows.first()).toContainText("Someday Task");
  });

  test("someday page shows empty state when no someday tasks", async ({
    authenticatedPage: page,
  }) => {
    await api.createTask({ title: "Normal Task" });
    await page.goto("/someday");
    await page
      .locator(
        '[data-testid="task-list"], [data-testid="empty-task-list"]',
      )
      .first()
      .waitFor({ state: "visible" });
    await expect(page.locator('[data-testid="task-row"]')).toHaveCount(0);
  });

  test("sidebar has Someday nav link", async ({
    authenticatedPage: page,
  }) => {
    await page.goto("/inbox");
    const nav = page.locator('a[href="/someday"]');
    await expect(nav).toBeVisible();
    await nav.click();
    await expect(page).toHaveURL(/\/someday/);
  });

  test("someday tasks excluded from review", async ({
    authenticatedPage: page,
  }) => {
    const task = await api.createTask({
      title: "Someday Review",
      reviewed_at: "2020-01-01",
    });
    await api.updateTask(task.id, { someday: true });

    await page.goto("/review");
    await page
      .locator(
        '[data-testid="task-list"], [data-testid="empty-task-list"]',
      )
      .first()
      .waitFor({ state: "visible" });
    await expect(page.locator('[data-testid="task-row"]')).toHaveCount(0);
  });

  test("S key toggles someday off (returns to normal)", async ({
    authenticatedPage: page,
  }) => {
    const task = await api.createTask({ title: "Toggle Back" });
    await api.updateTask(task.id, { someday: true });

    await page.goto("/someday");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });
    await expect(page.locator('[data-testid="task-row"]')).toHaveCount(1);

    // Select and press S to un-someday
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("s");

    // Task should disappear from someday page
    await expect(page.locator('[data-testid="task-row"]')).toHaveCount(0);
  });
});
