import { expect } from "@playwright/test";
import { test } from "../fixtures/auth";
import { ApiHelper } from "../fixtures/api";

let api: ApiHelper;

test.describe("Task Detail Modal", () => {
  test.beforeEach(async ({ authenticatedPage }) => {
    api = new ApiHelper(authenticatedPage.context());
    await api.deleteAllTasks();
  });

  test.afterEach(async () => {
    await api.deleteAllTasks();
  });

  test("opens when clicking a task and shows correct data", async ({
    authenticatedPage: page,
  }) => {
    await api.createTask({ title: "Modal Task" });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    // Select task and open with E
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("e");

    const modal = page.locator('[data-testid="task-detail-modal"]');
    await expect(modal).toBeVisible();

    // Title should contain the task title
    const titleArea = modal.locator('[data-testid="task-detail-title"]');
    await expect(titleArea).toBeVisible();
    await expect(titleArea.locator("input")).toHaveValue("Modal Task");
  });

  test("edit title persists on close and reopen", async ({
    authenticatedPage: page,
  }) => {
    await api.createTask({ title: "Original Title" });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    // Open modal
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("e");

    const modal = page.locator('[data-testid="task-detail-modal"]');
    await expect(modal).toBeVisible();

    // Edit title
    const titleInput = modal.locator(
      '[data-testid="task-detail-title"] input',
    );
    await titleInput.fill("Updated Title");
    await titleInput.press("Enter");

    // Close modal
    await page.locator('[data-testid="task-detail-close"]').click();
    await expect(modal).not.toBeVisible();

    // Verify title updated in the list
    await expect(
      page.locator('[data-testid="task-row"]').first(),
    ).toContainText("Updated Title");

    // Reopen and verify
    await page.keyboard.press("e");
    await expect(modal).toBeVisible();
    await expect(
      modal.locator('[data-testid="task-detail-title"] input'),
    ).toHaveValue("Updated Title");
  });

  test("Escape closes the modal", async ({ authenticatedPage: page }) => {
    await api.createTask({ title: "Close Me" });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("e");

    const modal = page.locator('[data-testid="task-detail-modal"]');
    await expect(modal).toBeVisible();

    await page.keyboard.press("Escape");
    await expect(modal).not.toBeVisible();
  });

  test("close button closes the modal", async ({
    authenticatedPage: page,
  }) => {
    await api.createTask({ title: "Close Via Button" });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("e");

    const modal = page.locator('[data-testid="task-detail-modal"]');
    await expect(modal).toBeVisible();

    await page.locator('[data-testid="task-detail-close"]').click();
    await expect(modal).not.toBeVisible();
  });

  test("navigate between tasks with prev/next buttons", async ({
    authenticatedPage: page,
  }) => {
    await api.createTask({ title: "Task A" });
    await api.createTask({ title: "Task B" });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    // Open first task
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("e");

    const modal = page.locator('[data-testid="task-detail-modal"]');
    await expect(modal).toBeVisible();
    const titleInput = modal.locator(
      '[data-testid="task-detail-title"] input',
    );
    await expect(titleInput).toHaveValue("Task A");

    // Navigate to next task
    await page.locator('[data-testid="task-detail-next"]').click();
    await expect(titleInput).toHaveValue("Task B");

    // Navigate back to previous
    await page.locator('[data-testid="task-detail-prev"]').click();
    await expect(titleInput).toHaveValue("Task A");
  });

  test("delete button removes the task", async ({
    authenticatedPage: page,
  }) => {
    await api.createTask({ title: "Delete Me" });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("e");

    const modal = page.locator('[data-testid="task-detail-modal"]');
    await expect(modal).toBeVisible();

    // Click delete
    await page.locator('[data-testid="task-detail-delete"]').click();

    // Modal should close and task should be gone
    await expect(modal).not.toBeVisible();
    await expect(page.locator('[data-testid="task-row"]')).toHaveCount(0);
  });

  test("create subtask via inline input", async ({
    authenticatedPage: page,
  }) => {
    await api.createTask({ title: "Parent Task" });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    // Open modal
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("e");

    const modal = page.locator('[data-testid="task-detail-modal"]');
    await expect(modal).toBeVisible();

    // Click "+ Add subtask"
    await modal.locator('[data-testid="task-detail-subtask-btn"]').click();

    // Fill in subtask
    const subtaskInput = modal.locator(
      '[data-testid="task-detail-subtask-input"]',
    );
    await expect(subtaskInput).toBeVisible();
    await subtaskInput.fill("My Subtask");
    await subtaskInput.press("Enter");

    // Subtask should appear in the modal's task list
    await expect(modal.locator('[data-testid="task-row"]')).toHaveCount(1);
    await expect(
      modal.locator('[data-testid="task-row"]').first(),
    ).toContainText("My Subtask");
  });

  test("edit body persists", async ({ authenticatedPage: page }) => {
    await api.createTask({ title: "Body Test" });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("e");

    const modal = page.locator('[data-testid="task-detail-modal"]');
    await expect(modal).toBeVisible();

    // Click on the body area to enter edit mode
    const bodyArea = modal.locator('[data-testid="task-detail-body"]');
    await bodyArea.click();

    // Type in the textarea
    const textarea = bodyArea.locator("textarea");
    await expect(textarea).toBeVisible();
    await textarea.fill("This is the task description");

    // Click elsewhere to blur and save
    await modal.locator('[data-testid="task-detail-title"]').click();

    // Close and reopen to verify persistence
    await page.locator('[data-testid="task-detail-close"]').click();
    await expect(modal).not.toBeVisible();

    await page.keyboard.press("e");
    await expect(modal).toBeVisible();

    // Body should show the saved description
    await expect(bodyArea).toContainText("This is the task description");
  });
});
