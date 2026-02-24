import { expect } from "@playwright/test";
import { test } from "../fixtures/auth";
import { ApiHelper } from "../fixtures/api";

let api: ApiHelper;

test.describe("Task Inline Autocomplete", () => {
  let projectId: number;

  test.beforeEach(async ({ authenticatedPage }) => {
    api = new ApiHelper(authenticatedPage.context());
    await api.deleteAllTasks();
    // Create a project for @project token
    const project = await api.createProject({ title: "Work" });
    projectId = project.id;
  });

  test.afterEach(async () => {
    await api.deleteAllTasks();
    await api.updateProject(projectId, { status: "archived" });
  });

  test("@project token keeps task visible on inbox after creation", async ({
    authenticatedPage: page,
  }) => {
    await page.goto("/inbox");
    await page
      .locator(
        '[data-testid="task-list"], [data-testid="empty-task-list"]',
      )
      .first()
      .waitFor({ state: "visible" });

    // Create task with @project token
    await page.locator('[data-testid="inbox-add-task"]').click();
    const input = page.locator('[data-testid="inline-create-input"]');
    await expect(input).toBeVisible();
    await input.fill("Buy supplies @Work");
    await input.press("Enter");

    // Task should remain visible on inbox (extra_show_ids keeps it)
    const rows = page.locator('[data-testid="task-row"]');
    await expect(rows).toHaveCount(1);
    await expect(rows.first()).toContainText("Buy supplies");
  });

  test("@project token assigns project visible from project page", async ({
    authenticatedPage: page,
  }) => {
    await page.goto("/inbox");
    await page
      .locator(
        '[data-testid="task-list"], [data-testid="empty-task-list"]',
      )
      .first()
      .waitFor({ state: "visible" });

    // Create task with @project token
    await page.locator('[data-testid="inbox-add-task"]').click();
    const input = page.locator('[data-testid="inline-create-input"]');
    await expect(input).toBeVisible();
    await input.fill("Buy supplies @Work");
    await input.press("Enter");

    // Wait for the task to appear (confirms server round-trip completed)
    const rows = page.locator('[data-testid="task-row"]');
    await expect(rows).toHaveCount(1);

    // Verify project assigned via API
    const tasks = await api.listTasks();
    const task = tasks.find((t: any) => t.title.includes("Buy supplies"));
    expect(task).toBeDefined();
    expect(task!.project_id).toBe(projectId);

    // Navigate to project page and verify task is there
    await page.locator('[data-testid="sidebar-project-item"]').filter({ hasText: "Work" }).click();
    await page
      .locator(
        '[data-testid="task-list"], [data-testid="empty-task-list"]',
      )
      .first()
      .waitFor({ state: "visible" });
    const projectRows = page.locator('[data-testid="task-row"]');
    await expect(projectRows).toHaveCount(1);
    await expect(projectRows.first()).toContainText("Buy supplies");
  });
});
