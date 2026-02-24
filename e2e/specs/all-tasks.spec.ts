import { expect } from "@playwright/test";
import { test } from "../fixtures/auth";
import { ApiHelper } from "../fixtures/api";

let api: ApiHelper;

test.describe("All Tasks Page", () => {
  test.beforeEach(async ({ authenticatedPage }) => {
    api = new ApiHelper(authenticatedPage.context());
    await api.deleteAllTasks();
  });

  test.afterEach(async () => {
    await api.deleteAllTasks();
  });

  test("shows tasks from all projects", async ({
    authenticatedPage: page,
  }) => {
    // Create a project and tasks in it
    const project = await api.createProject({ title: "Project Alpha" });
    await api.createTask({ title: "Project Task", project_id: project.id });

    // Create a task without a project
    await api.createTask({ title: "Inbox Task" });

    await page.goto("/tasks");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    const rows = page.locator('[data-testid="task-row"]');
    await expect(rows).toHaveCount(2);

    // Clean up project
    await api.updateProject(project.id, { status: "archived" });
  });

  test("creating a task here without project lands in inbox", async ({
    authenticatedPage: page,
  }) => {
    await page.goto("/tasks");
    await page
      .locator(
        '[data-testid="task-list"], [data-testid="empty-task-list"]',
      )
      .first()
      .waitFor({ state: "visible" });

    // Click "+ Add task"
    await page.locator('[data-testid="all-tasks-add-task"]').click();

    const input = page.locator('[data-testid="inline-create-input"]');
    await expect(input).toBeVisible();
    await input.fill("All Tasks Created");
    await input.press("Enter");

    // Task should appear in the list
    const rows = page.locator('[data-testid="task-row"]');
    await expect(rows).toHaveCount(1);
    await expect(rows.first()).toContainText("All Tasks Created");

    // Verify the task has no project via API
    const tasks = await api.listTasks();
    const created = tasks.find((t) => t.title === "All Tasks Created");
    expect(created).toBeDefined();
    expect(created!.project_id).toBeNull();
  });
});
