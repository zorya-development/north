import { expect } from "@playwright/test";
import { test } from "../fixtures/auth";
import { ApiHelper } from "../fixtures/api";

let api: ApiHelper;

test.describe("Task Extra Visibility", () => {
  test.beforeEach(async ({ authenticatedPage }) => {
    api = new ApiHelper(authenticatedPage.context());
    await api.deleteAllTasks();
  });

  test.afterEach(async () => {
    await api.deleteAllTasks();
  });

  test("task stays visible on inbox after changing project in detail modal", async ({
    authenticatedPage: page,
  }) => {
    const project = await api.createProject({ title: "Work" });
    await api.createTask({ title: "My Task" });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    // Open detail modal
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("e");

    const modal = page.locator('[data-testid="task-detail-modal"]');
    await expect(modal).toBeVisible();

    // Change project to "Work"
    await modal.locator('[data-testid="project-picker-trigger"]').click();
    await modal
      .locator('[data-testid="project-picker-option"]')
      .filter({ hasText: "Work" })
      .click();

    // Verify project shows in modal
    await expect(modal).toContainText("Work");

    // Close modal
    await page.locator('[data-testid="task-detail-close"]').click();
    await expect(modal).not.toBeVisible();

    // Task should still be visible on inbox (extra_show_ids keeps it)
    const rows = page.locator('[data-testid="task-row"]');
    await expect(rows).toHaveCount(1);
    await expect(rows.first()).toContainText("My Task");

    // Cleanup
    await api.updateProject(project.id, { status: "archived" });
  });

  test("completed task stays visible until page refresh", async ({
    authenticatedPage: page,
  }) => {
    await api.createTask({ title: "Task A" });
    await api.createTask({ title: "Task B" });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    await expect(page.locator('[data-testid="task-row"]')).toHaveCount(2);

    // Complete Task A via checkbox
    await page
      .locator('[data-testid="task-row"]')
      .filter({ hasText: "Task A" })
      .locator('[data-testid="task-checkbox"]')
      .click();

    // Both tasks should still be visible (completed task is pinned)
    await expect(page.locator('[data-testid="task-row"]')).toHaveCount(2);
    await expect(
      page.locator('[data-testid="task-row"]').filter({ hasText: "Task A" }),
    ).toBeVisible();

    // After page refresh, the completed task should disappear
    await page.reload();
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    await expect(page.locator('[data-testid="task-row"]')).toHaveCount(1);
    await expect(
      page.locator('[data-testid="task-row"]').first(),
    ).toContainText("Task B");
  });

  test("completed task via keyboard stays visible until page refresh", async ({
    authenticatedPage: page,
  }) => {
    await api.createTask({ title: "Keyboard Task" });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    await expect(page.locator('[data-testid="task-row"]')).toHaveCount(1);

    // Select task and complete via Space key
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("Space");

    // Task should still be visible
    await expect(page.locator('[data-testid="task-row"]')).toHaveCount(1);
    await expect(
      page.locator('[data-testid="task-row"]').first(),
    ).toContainText("Keyboard Task");

    // After refresh it disappears
    await page.reload();
    await page
      .locator(
        '[data-testid="task-list"], [data-testid="empty-task-list"]',
      )
      .first()
      .waitFor({ state: "visible" });

    await expect(page.locator('[data-testid="task-row"]')).toHaveCount(0);
  });
});
