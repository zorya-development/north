import { expect } from "@playwright/test";
import { test } from "../fixtures/auth";
import { ApiHelper } from "../fixtures/api";

let api: ApiHelper;

test.describe("Project task isolation", () => {
  let projectA: { id: number };
  let projectB: { id: number };

  test.beforeEach(async ({ authenticatedPage }) => {
    api = new ApiHelper(authenticatedPage.context());
    await api.deleteAllTasks();

    projectA = await api.createProject({ title: "Project Alpha" });
    projectB = await api.createProject({ title: "Project Beta" });

    await api.createTask({ title: "Alpha Task 1", project_id: projectA.id });
    await api.createTask({ title: "Alpha Task 2", project_id: projectA.id });
    await api.createTask({ title: "Beta Task 1", project_id: projectB.id });
    await api.createTask({ title: "Beta Task 2", project_id: projectB.id });
  });

  test.afterEach(async () => {
    await api.deleteAllTasks();
    await api.updateProject(projectA.id, { status: "archived" });
    await api.updateProject(projectB.id, { status: "archived" });
  });

  test("each project page shows only its own tasks", async ({
    authenticatedPage: page,
  }) => {
    // Navigate to Project Alpha
    await page.goto(`/projects/${projectA.id}`);
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    const rowsA = page.locator('[data-testid="task-row"]');
    await expect(rowsA).toHaveCount(2);
    await expect(rowsA.nth(0)).toContainText("Alpha Task");
    await expect(rowsA.nth(1)).toContainText("Alpha Task");

    // Confirm no Beta tasks appear
    await expect(page.locator("text=Beta Task")).toHaveCount(0);

    // Navigate to Project Beta
    await page.goto(`/projects/${projectB.id}`);
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    const rowsB = page.locator('[data-testid="task-row"]');
    await expect(rowsB).toHaveCount(2);
    await expect(rowsB.nth(0)).toContainText("Beta Task");
    await expect(rowsB.nth(1)).toContainText("Beta Task");

    // Confirm no Alpha tasks appear
    await expect(page.locator("text=Alpha Task")).toHaveCount(0);
  });

  test("navigating between projects via sidebar shows correct tasks", async ({
    authenticatedPage: page,
  }) => {
    // Reload so sidebar picks up newly created projects
    await page.reload();
    await page
      .locator('[data-testid="sidebar-project-item"]')
      .first()
      .waitFor({ state: "visible" });

    // Navigate to Project Alpha via sidebar
    await page
      .locator('[data-testid="sidebar-project-item"]')
      .filter({ hasText: "Project Alpha" })
      .click();
    await page.waitForURL(`**/projects/${projectA.id}`);
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    // Verify Project Alpha tasks
    await expect(page.locator('[data-testid="task-row"]')).toHaveCount(2);
    await expect(
      page.locator('[data-testid="task-row"]').nth(0),
    ).toContainText("Alpha Task");

    // Click Project Beta in sidebar
    await page
      .locator('[data-testid="sidebar-project-item"]')
      .filter({ hasText: "Project Beta" })
      .click();
    await page.waitForURL(`**/projects/${projectB.id}`);
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    // Verify only Beta tasks
    const rows = page.locator('[data-testid="task-row"]');
    await expect(rows).toHaveCount(2);
    await expect(rows.nth(0)).toContainText("Beta Task");
    await expect(rows.nth(1)).toContainText("Beta Task");
    await expect(page.locator("text=Alpha Task")).toHaveCount(0);
  });

  test("completed tasks from other projects do not leak into project view", async ({
    authenticatedPage: page,
  }) => {
    // Complete both Alpha tasks and one Beta task
    const allTasks = await api.listTasks();
    for (const t of allTasks.filter((t) => t.title.startsWith("Alpha Task"))) {
      await api.updateTask(t.id, {
        completed_at: new Date().toISOString(),
      });
    }
    const betaTask = allTasks.find((t) => t.title === "Beta Task 1")!;
    await api.updateTask(betaTask.id, {
      completed_at: new Date().toISOString(),
    });

    // Navigate to Project Beta
    await page.goto(`/projects/${projectB.id}`);
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    // Only the uncompleted Beta task visible
    const rows = page.locator('[data-testid="task-row"]');
    await expect(rows).toHaveCount(1);
    await expect(rows.first()).toContainText("Beta Task 2");

    // Toggle "Show completed"
    await page.locator('[data-testid="ttl-toggle-completed"]').click();

    // Both Beta tasks visible (1 active + 1 completed)
    await expect(rows).toHaveCount(2);
    await expect(rows.nth(0)).toContainText("Beta Task");
    await expect(rows.nth(1)).toContainText("Beta Task");

    // No Alpha tasks leak through
    await expect(page.locator("text=Alpha Task")).toHaveCount(0);
  });
});
