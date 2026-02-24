import { expect } from "@playwright/test";
import { test } from "../fixtures/auth";
import { ApiHelper } from "../fixtures/api";

let api: ApiHelper;

test.describe("Project Page", () => {
  let projectId: number;

  test.beforeEach(async ({ authenticatedPage }) => {
    api = new ApiHelper(authenticatedPage.context());
    await api.deleteAllTasks();
    const project = await api.createProject({ title: "Test Project" });
    projectId = project.id;
  });

  test.afterEach(async () => {
    await api.deleteAllTasks();
    await api.updateProject(projectId, { status: "archived" });
  });

  test("shows only project tasks", async ({ authenticatedPage: page }) => {
    // Create a task in the project
    await api.createTask({ title: "Project Task", project_id: projectId });

    // Create a task without project (should NOT appear)
    await api.createTask({ title: "Inbox Task" });

    await page.goto(`/projects/${projectId}`);
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    const rows = page.locator('[data-testid="task-row"]');
    await expect(rows).toHaveCount(1);
    await expect(rows.first()).toContainText("Project Task");
  });

  test("project title shown in page header", async ({
    authenticatedPage: page,
  }) => {
    await page.goto(`/projects/${projectId}`);
    await page
      .locator(
        '[data-testid="task-list"], [data-testid="empty-task-list"]',
      )
      .first()
      .waitFor({ state: "visible" });

    const title = page.locator('[data-testid="project-title"]');
    await expect(title).toContainText("Test Project");
  });

  test("creating a task on project page assigns it to project", async ({
    authenticatedPage: page,
  }) => {
    await page.goto(`/projects/${projectId}`);
    await page
      .locator(
        '[data-testid="task-list"], [data-testid="empty-task-list"]',
      )
      .first()
      .waitFor({ state: "visible" });

    // Click "+ Add task"
    await page.locator('[data-testid="project-add-task"]').click();

    const input = page.locator('[data-testid="inline-create-input"]');
    await expect(input).toBeVisible();
    await input.fill("Auto Assigned Task");
    await input.press("Enter");

    const rows = page.locator('[data-testid="task-row"]');
    await expect(rows).toHaveCount(1);
    await expect(rows.first()).toContainText("Auto Assigned Task");

    // Verify via API that the task is assigned to the project
    const tasks = await api.listTasks();
    const created = tasks.find((t) => t.title === "Auto Assigned Task");
    expect(created).toBeDefined();
    expect(created!.project_id).toBe(projectId);
  });

  test("completing a task works same as inbox", async ({
    authenticatedPage: page,
  }) => {
    await api.createTask({
      title: "Complete in Project",
      project_id: projectId,
    });

    await page.goto(`/projects/${projectId}`);
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    await expect(page.locator('[data-testid="task-row"]')).toHaveCount(1);

    // Complete via checkbox
    await page.locator('[data-testid="task-checkbox"]').click();

    // Task should disappear from active list
    await expect(page.locator('[data-testid="task-row"]')).toHaveCount(0);
  });
});
