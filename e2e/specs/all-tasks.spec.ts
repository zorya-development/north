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

  test("shows tasks from all projects with inline project prefix", async ({
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

    // Project task should have @Project Alpha: prefix
    const projectRow = rows.filter({ hasText: "Project Task" });
    await expect(projectRow).toContainText("@Project Alpha");

    // Inbox task should have @Inbox prefix
    const inboxRow = rows.filter({ hasText: "Inbox Task" });
    await expect(inboxRow).toContainText("@Inbox");

    // Clean up project
    await api.updateProject(project.id, { status: "archived" });
  });

  test("project prefix links to project page", async ({
    authenticatedPage: page,
  }) => {
    const project = await api.createProject({ title: "Link Target" });
    await api.createTask({ title: "Linked Task", project_id: project.id });

    await page.goto("/tasks");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    // Click the project prefix link
    const projectLink = page.locator(
      `[data-testid="task-row"] a[href="/projects/${project.id}"]`,
    );
    await expect(projectLink).toBeVisible();
    await projectLink.click();

    // Should navigate to the project page
    await expect(page).toHaveURL(new RegExp(`/projects/${project.id}`));

    // Clean up project
    await api.updateProject(project.id, { status: "archived" });
  });

  test("inline tags appear after task title and link to filter", async ({
    authenticatedPage: page,
  }) => {
    // Create a task with a tag via inline input (triggers token parsing)
    await page.goto("/tasks");
    await page
      .locator(
        '[data-testid="task-list"], [data-testid="empty-task-list"]',
      )
      .first()
      .waitFor({ state: "visible" });

    await page.locator('[data-testid="all-tasks-add-task"]').click();
    const input = page.locator('[data-testid="inline-create-input"]');
    await expect(input).toBeVisible();
    await input.fill("Tagged Task #urgent");
    await input.press("Enter");

    // Wait for the task row to appear
    const row = page.locator('[data-testid="task-row"]').first();
    await expect(row).toBeVisible();

    // The tag should appear after the title — # is outside the link
    const tagLink = row.locator("a", { hasText: "urgent" });
    await expect(tagLink).toBeVisible();
    await tagLink.click();

    // Should navigate to filter page with query param
    await expect(page).toHaveURL(/\/filters\/new\?q=/);
  });

  test("tags with special separators are parsed correctly", async ({
    authenticatedPage: page,
  }) => {
    // Create tasks with tags using different separators via API (triggers token parsing)
    await api.createTask({ title: "Colon task #colon:separated:tag" });
    await api.createTask({ title: "Dot task #dot.separated.tag" });
    await api.createTask({ title: "Dash task #dash-separated-tag" });
    await api.createTask({ title: "Underscore task #under_score_tag" });

    await page.goto("/tasks");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    const rows = page.locator('[data-testid="task-row"]');
    await expect(rows).toHaveCount(4);

    // Each tag should appear as an inline link with correct full name
    const colonRow = rows.filter({ hasText: "Colon task" });
    await expect(colonRow.locator("a", { hasText: "colon:separated:tag" })).toBeVisible();

    const dotRow = rows.filter({ hasText: "Dot task" });
    await expect(dotRow.locator("a", { hasText: "dot.separated.tag" })).toBeVisible();

    const dashRow = rows.filter({ hasText: "Dash task" });
    await expect(dashRow.locator("a", { hasText: "dash-separated-tag" })).toBeVisible();

    const underscoreRow = rows.filter({ hasText: "Underscore task" });
    await expect(underscoreRow.locator("a", { hasText: "under_score_tag" })).toBeVisible();

    // Open modal for colon task to verify tag in detail view too
    await colonRow.click();
    await page.keyboard.press("e");
    const modal = page.locator('[data-testid="task-detail-modal"]');
    await expect(modal).toBeVisible();
    await expect(modal).toContainText("colon:separated:tag");
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
