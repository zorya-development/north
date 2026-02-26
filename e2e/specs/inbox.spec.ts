import { expect } from "@playwright/test";
import { test } from "../fixtures/auth";
import { ApiHelper } from "../fixtures/api";

let api: ApiHelper;

test.describe("Inbox", () => {
  test.beforeEach(async ({ authenticatedPage }) => {
    api = new ApiHelper(authenticatedPage.context());
    await api.deleteAllTasks();
  });

  test.afterEach(async () => {
    await api.deleteAllTasks();
  });

  test("page loads with task list", async ({ authenticatedPage: page }) => {
    // Create 3 tasks via API
    await api.createTask({ title: "Task A" });
    await api.createTask({ title: "Task B" });
    await api.createTask({ title: "Task C" });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    const rows = page.locator('[data-testid="task-row"]');
    await expect(rows).toHaveCount(3);
  });

  test("creates task via inline input", async ({
    authenticatedPage: page,
  }) => {
    await page.goto("/inbox");
    await page
      .locator('[data-testid="empty-task-list"]')
      .waitFor({ state: "visible" });

    // Click "+ Add task" to open inline input
    await page.locator('[data-testid="inbox-add-task"]').click();

    const input = page.locator('[data-testid="inline-create-input"]');
    await expect(input).toBeVisible();
    await input.fill("My new task");
    await input.press("Enter");

    // Task should appear in the list
    const rows = page.locator('[data-testid="task-row"]');
    await expect(rows).toHaveCount(1);
    await expect(rows.first()).toContainText("My new task");
  });

  test("Ctrl+Enter inserts below, Shift+Enter inserts above", async ({
    authenticatedPage: page,
  }) => {
    await page.goto("/inbox");
    await page
      .locator('[data-testid="empty-task-list"]')
      .waitFor({ state: "visible" });

    // Create anchor task via UI so it gets a valid FractionalIndex sort key
    await page.locator('[data-testid="inbox-add-task"]').click();
    const input = page.locator('[data-testid="inline-create-input"]');
    await expect(input).toBeVisible();
    await input.fill("Anchor Task");
    await input.press("Enter");
    await expect(page.locator('[data-testid="task-row"]')).toHaveCount(1);
    await page.keyboard.press("Escape");

    // Select the anchor task with keyboard
    await page.keyboard.press("ArrowDown");

    // Ctrl+Enter to insert below
    await page.keyboard.press("Control+Enter");
    const createInput = page.locator('[data-testid="inline-create-input"]');
    await expect(createInput).toBeVisible();
    await createInput.fill("Below Task");
    await createInput.press("Enter");
    await expect(page.locator('[data-testid="task-row"]')).toHaveCount(2);
    await page.keyboard.press("Escape");

    // Move cursor back to the anchor task
    await page.keyboard.press("ArrowUp");

    // Shift+Enter to insert above
    await page.keyboard.press("Shift+Enter");
    const createInput2 = page.locator('[data-testid="inline-create-input"]');
    await expect(createInput2).toBeVisible();
    await createInput2.fill("Above Task");
    await createInput2.press("Enter");

    // Verify order: Above Task, Anchor Task, Below Task
    await expect(page.locator('[data-testid="task-row"]')).toHaveCount(3);
    const rows = page.locator('[data-testid="task-row"]');
    await expect(rows.nth(0)).toContainText("Above Task");
    await expect(rows.nth(1)).toContainText("Anchor Task");
    await expect(rows.nth(2)).toContainText("Below Task");
  });

  test("completes task via checkbox", async ({ authenticatedPage: page }) => {
    await api.createTask({ title: "Task to complete" });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    await expect(page.locator('[data-testid="task-row"]')).toHaveCount(1);

    // Click the checkbox to complete
    await page.locator('[data-testid="task-checkbox"]').click();

    // Completed task stays visible until page refresh
    await expect(page.locator('[data-testid="task-row"]')).toHaveCount(1);
  });

  test("uncompletes task", async ({ authenticatedPage: page }) => {
    // Create and complete a task via API
    const task = await api.createTask({ title: "Completed task" });
    await api.updateTask(task.id, {
      completed_at: new Date().toISOString(),
    });

    await page.goto("/inbox");

    // Toggle "Show completed"
    await page.locator('[data-testid="inbox-toggle-completed"]').click();

    // Task should be visible and have line-through styling
    const rows = page.locator('[data-testid="task-row"]');
    await expect(rows).toHaveCount(1);
    await expect(rows.first()).toContainText("Completed task");

    // Click checkbox to uncomplete
    await page.locator('[data-testid="task-checkbox"]').click();

    // Task should still be visible but no longer struck through
    await expect(rows).toHaveCount(1);
    // Verify it's no longer completed — the checkbox should now show as unchecked
    // (the circle SVG, not the filled accent div)
    const checkbox = page.locator('[data-testid="task-checkbox"]');
    await expect(checkbox.locator("svg")).toBeVisible();
  });

  test("deletes task with confirmation", async ({
    authenticatedPage: page,
  }) => {
    await api.createTask({ title: "Task to delete" });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    // Select the task
    await page.keyboard.press("ArrowDown");

    // Press Delete to trigger delete prompt
    await page.keyboard.press("Delete");

    // Status bar should show confirmation prompt
    const statusBar = page.locator('[data-testid="status-bar"]');
    await expect(statusBar).toBeVisible();

    // Press Enter to confirm
    await page.keyboard.press("Enter");

    // Task should be gone
    await expect(page.locator('[data-testid="task-row"]')).toHaveCount(0);
  });

  test("toggle hide non-actionable persists on reload", async ({
    authenticatedPage: page,
  }) => {
    // Create a parent task with sequential_limit and 3 subtasks
    const parent = await api.createTask({ title: "Sequential Parent" });
    await api.updateTask(parent.id, { sequential_limit: 1 });

    await api.createTask({ title: "Sub 1", parent_id: parent.id });
    await api.createTask({ title: "Sub 2", parent_id: parent.id });
    await api.createTask({ title: "Sub 3", parent_id: parent.id });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    // Initially all 4 tasks visible (parent + 3 children)
    await expect(page.locator('[data-testid="task-row"]')).toHaveCount(4);

    // Toggle hide non-actionable
    await page.locator('[data-testid="inbox-toggle-actionable"]').click();

    // Only parent + 1 actionable child should be visible
    await expect(page.locator('[data-testid="task-row"]')).toHaveCount(2);

    // Reload and verify persisted
    await page.reload();
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    await expect(page.locator('[data-testid="task-row"]')).toHaveCount(2);

    // Toggle back to show all
    await page.locator('[data-testid="inbox-toggle-actionable"]').click();
    await expect(page.locator('[data-testid="task-row"]')).toHaveCount(4);
  });

  test("Ctrl+Shift+Enter creates subtask", async ({
    authenticatedPage: page,
  }) => {
    await api.createTask({ title: "Parent Task" });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    // Select the parent task
    await page.keyboard.press("ArrowDown");

    // Ctrl+Shift+Enter creates a subtask
    await page.keyboard.press("Control+Shift+Enter");

    const createInput = page.locator('[data-testid="inline-create-input"]');
    await expect(createInput).toBeVisible();
    await createInput.fill("Subtask");
    await createInput.press("Enter");

    // Parent + child = 2 rows
    await expect(page.locator('[data-testid="task-row"]')).toHaveCount(2);
  });

  test("sequential limit shows only first N subtasks as actionable", async ({
    authenticatedPage: page,
  }) => {
    // Create a parent with sequential_limit=2 and 3 subtasks
    const parent = await api.createTask({ title: "Sequential Parent" });
    await api.updateTask(parent.id, { sequential_limit: 2 });

    await api.createTask({ title: "Sub A", parent_id: parent.id });
    await api.createTask({ title: "Sub B", parent_id: parent.id });
    await api.createTask({ title: "Sub C", parent_id: parent.id });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    // Toggle hide non-actionable
    await page.locator('[data-testid="inbox-toggle-actionable"]').click();

    // Parent + 2 actionable children (Sub A and Sub B) should be visible
    await expect(page.locator('[data-testid="task-row"]')).toHaveCount(3);

    const rows = page.locator('[data-testid="task-row"]');
    await expect(rows.nth(0)).toContainText("Sequential Parent");
    await expect(rows.nth(1)).toContainText("Sub A");
    await expect(rows.nth(2)).toContainText("Sub B");
  });
});
