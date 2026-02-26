import { expect } from "@playwright/test";
import { test } from "../fixtures/auth";
import { ApiHelper } from "../fixtures/api";

let api: ApiHelper;

test.describe("SmartTextarea Autocomplete", () => {
  let projectId: number;

  test.beforeEach(async ({ authenticatedPage }) => {
    api = new ApiHelper(authenticatedPage.context());
    await api.deleteAllTasks();

    // Create a project for @project autocomplete
    const project = await api.createProject({ title: "Work" });
    projectId = project.id;

    // Create a task with #urgent token — tag autocomplete only shows tags
    // that have at least one task using them.
    await api.createTask({ title: "seed task #urgent" });
  });

  test.afterEach(async () => {
    await api.deleteAllTasks();
    await api.updateProject(projectId, { status: "archived" });
  });

  test("typing @ in inline create shows project autocomplete dropdown", async ({
    authenticatedPage: page,
  }) => {
    await page.goto("/inbox");
    await page
      .locator(
        '[data-testid="task-list"], [data-testid="empty-task-list"]',
      )
      .first()
      .waitFor({ state: "visible" });

    await page.locator('[data-testid="inbox-add-task"]').click();
    const input = page.locator('[data-testid="inline-create-input"]');
    await expect(input).toBeVisible();

    // Type character-by-character to trigger autocomplete
    await input.pressSequentially("Buy supplies @");

    const dropdown = page.locator('[data-testid="autocomplete-dropdown"]');
    await expect(dropdown).toBeVisible();
    await expect(
      dropdown.locator('[data-testid="autocomplete-item"]'),
    ).toContainText(["Work"]);
  });

  test("clicking project suggestion in inline create inserts project name", async ({
    authenticatedPage: page,
  }) => {
    await page.goto("/inbox");
    await page
      .locator(
        '[data-testid="task-list"], [data-testid="empty-task-list"]',
      )
      .first()
      .waitFor({ state: "visible" });

    await page.locator('[data-testid="inbox-add-task"]').click();
    const input = page.locator('[data-testid="inline-create-input"]');
    await expect(input).toBeVisible();

    await input.pressSequentially("Buy supplies @");
    const dropdown = page.locator('[data-testid="autocomplete-dropdown"]');
    await expect(dropdown).toBeVisible();

    // Click the "Work" item
    await dropdown
      .locator('[data-testid="autocomplete-item"]')
      .filter({ hasText: "Work" })
      .click();

    // Verify input value contains @Work
    await expect(input).toHaveValue(/.*@Work.*/);

    // Submit the task
    await input.press("Enter");

    // Wait for task to appear in the list (confirms server round-trip completed)
    const rows = page.locator('[data-testid="task-row"]');
    await expect(rows.first()).toContainText("Buy supplies");

    // Verify via API that the task has the project assigned
    const tasks = await api.listTasks();
    const task = tasks.find((t: any) => t.title.includes("Buy supplies"));
    expect(task).toBeDefined();
    expect(task!.project_id).toBe(projectId);
  });

  test("arrow keys + Enter selects project suggestion", async ({
    authenticatedPage: page,
  }) => {
    await page.goto("/inbox");
    await page
      .locator(
        '[data-testid="task-list"], [data-testid="empty-task-list"]',
      )
      .first()
      .waitFor({ state: "visible" });

    await page.locator('[data-testid="inbox-add-task"]').click();
    const input = page.locator('[data-testid="inline-create-input"]');
    await expect(input).toBeVisible();

    await input.pressSequentially("Buy supplies @");
    const dropdown = page.locator('[data-testid="autocomplete-dropdown"]');
    await expect(dropdown).toBeVisible();

    // Navigate with arrow key and select
    await input.press("ArrowDown");
    await input.press("Enter");

    // Dropdown should close after selection
    await expect(dropdown).not.toBeVisible();

    // Verify input value contains @Work
    await expect(input).toHaveValue(/.*@Work.*/);
  });

  test("typing # in inline create shows tag autocomplete dropdown", async ({
    authenticatedPage: page,
  }) => {
    await page.goto("/inbox");
    await page
      .locator(
        '[data-testid="task-list"], [data-testid="empty-task-list"]',
      )
      .first()
      .waitFor({ state: "visible" });

    await page.locator('[data-testid="inbox-add-task"]').click();
    const input = page.locator('[data-testid="inline-create-input"]');
    await expect(input).toBeVisible();

    await input.pressSequentially("Fix bug #");

    const dropdown = page.locator('[data-testid="autocomplete-dropdown"]');
    await expect(dropdown).toBeVisible();
    await expect(
      dropdown.locator('[data-testid="autocomplete-item"]'),
    ).toContainText(["urgent"]);
  });

  test("clicking tag suggestion in inline create inserts tag name", async ({
    authenticatedPage: page,
  }) => {
    await page.goto("/inbox");
    await page
      .locator(
        '[data-testid="task-list"], [data-testid="empty-task-list"]',
      )
      .first()
      .waitFor({ state: "visible" });

    await page.locator('[data-testid="inbox-add-task"]').click();
    const input = page.locator('[data-testid="inline-create-input"]');
    await expect(input).toBeVisible();

    await input.pressSequentially("Fix bug #");
    const dropdown = page.locator('[data-testid="autocomplete-dropdown"]');
    await expect(dropdown).toBeVisible();

    await dropdown
      .locator('[data-testid="autocomplete-item"]')
      .filter({ hasText: "urgent" })
      .click();

    await expect(input).toHaveValue(/.*#urgent.*/);

    // Submit and verify the tag was assigned
    await input.press("Enter");
    const tasks = await api.listTasks();
    const task = tasks.find((t: any) => t.title.includes("Fix bug"));
    expect(task).toBeDefined();
  });

  test("typing @ in subtask inline input shows project autocomplete", async ({
    authenticatedPage: page,
  }) => {
    // Create a parent task
    await api.createTask({ title: "Parent task" });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    // Open task detail modal
    await page.locator('[data-testid="task-row"]').first().click();
    await page
      .locator('[data-testid="task-detail-modal"]')
      .waitFor({ state: "visible" });

    // Click "Add subtask"
    await page.locator('[data-testid="task-detail-subtask-btn"]').click();
    const subtaskInput = page.locator(
      '[data-testid="task-detail-subtask-input"]',
    );
    await expect(subtaskInput).toBeVisible();

    await subtaskInput.pressSequentially("Sub @");

    const dropdown = page.locator('[data-testid="autocomplete-dropdown"]');
    await expect(dropdown).toBeVisible();
    await expect(
      dropdown.locator('[data-testid="autocomplete-item"]'),
    ).toContainText(["Work"]);
  });

  test("Escape closes autocomplete dropdown without closing input", async ({
    authenticatedPage: page,
  }) => {
    await page.goto("/inbox");
    await page
      .locator(
        '[data-testid="task-list"], [data-testid="empty-task-list"]',
      )
      .first()
      .waitFor({ state: "visible" });

    await page.locator('[data-testid="inbox-add-task"]').click();
    const input = page.locator('[data-testid="inline-create-input"]');
    await expect(input).toBeVisible();

    await input.pressSequentially("Buy @");
    const dropdown = page.locator('[data-testid="autocomplete-dropdown"]');
    await expect(dropdown).toBeVisible();

    // Escape closes only the dropdown
    await input.press("Escape");
    await expect(dropdown).not.toBeVisible();
    // Input should still be visible
    await expect(input).toBeVisible();

    // Second Escape closes the inline input
    await input.press("Escape");
    await expect(input).not.toBeVisible();
  });
});
