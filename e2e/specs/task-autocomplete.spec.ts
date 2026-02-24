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

  test("@project token assigns project and removes token from title", async ({
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

    // Task should appear without the @Work token in title
    const rows = page.locator('[data-testid="task-row"]');
    await expect(rows).toHaveCount(1);
    await expect(rows.first()).toContainText("Buy supplies");

    // Verify project assigned via API
    const tasks = await api.listTasks();
    const task = tasks.find((t) => t.title.includes("Buy supplies"));
    expect(task).toBeDefined();
    expect(task!.project_id).toBe(projectId);
  });
});
