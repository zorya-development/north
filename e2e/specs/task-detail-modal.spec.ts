import { expect } from "@playwright/test";
import { test } from "../fixtures/auth";
import { ApiHelper } from "../fixtures/api";

let api: ApiHelper;

test.describe("Task Detail Modal", () => {
  test.beforeEach(async ({ authenticatedPage }) => {
    api = new ApiHelper(authenticatedPage.context());
    await api.deleteAllTasks();
    await api.deleteAllProjects();
  });

  test.afterEach(async () => {
    await api.deleteAllTasks();
    await api.deleteAllProjects();
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

    // Click on the body placeholder to enter edit mode
    const bodyArea = modal.locator('[data-testid="task-detail-body"]');
    const placeholder = bodyArea.getByText("Add description...");
    await expect(placeholder).toBeVisible();
    await placeholder.click();

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

  test("change project in modal", async ({ authenticatedPage: page }) => {
    await api.createProject({ title: "Work" });
    await api.createTask({ title: "Project task" });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    // Open modal
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("e");

    const modal = page.locator('[data-testid="task-detail-modal"]');
    await expect(modal).toBeVisible();

    // Click project picker trigger — a background refetch may re-render
    // modal content (detaching the open popover), so retry if needed
    const workOption = modal
      .locator('[data-testid="project-picker-option"]')
      .filter({ hasText: "Work" });
    await modal.locator('[data-testid="project-picker-trigger"]').click();
    for (let attempt = 0; attempt < 3; attempt++) {
      try {
        await workOption.click({ timeout: 3000 });
        break;
      } catch {
        await modal
          .locator('[data-testid="project-picker-trigger"]')
          .click();
      }
    }

    // Verify project shows in header
    await expect(modal).toContainText("Work");

    // Close and reopen to verify persistence
    await page.locator('[data-testid="task-detail-close"]').click();
    await expect(modal).not.toBeVisible();

    await page.keyboard.press("e");
    await expect(modal).toBeVisible();
    await expect(modal).toContainText("Work");
  });

  test("unset project in modal", async ({ authenticatedPage: page }) => {
    const project = await api.createProject({ title: "Work" });
    await api.createTask({ title: "Project task", project_id: project.id });

    await page.goto("/tasks");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    // Open modal — task is under "Work" project, navigate to it
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("e");

    const modal = page.locator('[data-testid="task-detail-modal"]');
    await expect(modal).toBeVisible();

    // Verify "Work" is shown in sidebar project picker
    await expect(
      modal.locator('[data-testid="project-picker-trigger"]'),
    ).toContainText("Work");

    // Click project picker trigger — retry if background refetch
    // re-renders modal content and closes the popover
    const inboxOption = modal.locator('[data-testid="project-picker-inbox"]');
    await modal.locator('[data-testid="project-picker-trigger"]').click();
    for (let attempt = 0; attempt < 3; attempt++) {
      try {
        await inboxOption.click({ timeout: 3000 });
        break;
      } catch {
        await modal
          .locator('[data-testid="project-picker-trigger"]')
          .click();
      }
    }

    // Verify project picker now shows "Inbox"
    await expect(
      modal.locator('[data-testid="project-picker-trigger"]'),
    ).toContainText("Inbox");

    // Close and reopen to verify persistence
    await page.locator('[data-testid="task-detail-close"]').click();
    await expect(modal).not.toBeVisible();

    await page.keyboard.press("e");
    await expect(modal).toBeVisible();
    await expect(
      modal.locator('[data-testid="project-picker-trigger"]'),
    ).toContainText("Inbox");
  });

  test("change due date in modal", async ({ authenticatedPage: page }) => {
    await api.createTask({ title: "Due date task" });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    // Open modal
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("e");

    const modal = page.locator('[data-testid="task-detail-modal"]');
    await expect(modal).toBeVisible();

    // Fill the due date input
    const dueDateInput = modal.locator('[data-testid="due-date-input"]');
    await dueDateInput.fill("2026-12-25");

    // Close and reopen to verify persistence
    await page.locator('[data-testid="task-detail-close"]').click();
    await expect(modal).not.toBeVisible();

    await page.keyboard.press("e");
    await expect(modal).toBeVisible();
    await expect(
      modal.locator('[data-testid="due-date-input"]'),
    ).toHaveValue("2026-12-25");
  });

  test("change start date in modal", async ({ authenticatedPage: page }) => {
    await api.createTask({ title: "Start date task" });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    // Open modal
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("e");

    const modal = page.locator('[data-testid="task-detail-modal"]');
    await expect(modal).toBeVisible();

    // Click start date trigger — retry if background refetch
    // re-renders modal content and closes the popover
    const dateInput = modal.locator('[data-testid="start-date-input"]');
    await modal.locator('[data-testid="start-date-trigger"]').click();
    for (let attempt = 0; attempt < 3; attempt++) {
      try {
        await dateInput.waitFor({ state: "visible", timeout: 3000 });
        break;
      } catch {
        await modal.locator('[data-testid="start-date-trigger"]').click();
      }
    }
    await dateInput.fill("2026-12-20");

    // Click Save
    await modal.locator('[data-testid="start-date-save"]').click();

    // Verify start date is displayed (trigger should now show the date)
    await expect(
      modal.locator('[data-testid="start-date-trigger"]'),
    ).toBeVisible();
    await expect(
      modal.locator('[data-testid="start-date-trigger"]'),
    ).not.toContainText("Not set");

    // Close and reopen to verify persistence
    await page.locator('[data-testid="task-detail-close"]').click();
    await expect(modal).not.toBeVisible();

    await page.keyboard.press("e");
    await expect(modal).toBeVisible();
    await expect(
      modal.locator('[data-testid="start-date-trigger"]'),
    ).not.toContainText("Not set");
  });

  test("add tag in modal", async ({ authenticatedPage: page }) => {
    await api.createTask({ title: "Tag task" });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    // Open modal
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("e");

    const modal = page.locator('[data-testid="task-detail-modal"]');
    await expect(modal).toBeVisible();

    // Click tag picker trigger — retry if background refetch
    // re-renders modal content and closes the popover
    const tagInput = modal.locator('[data-testid="tag-picker-input"]');
    await modal.locator('[data-testid="tag-picker-trigger"]').click();
    for (let attempt = 0; attempt < 3; attempt++) {
      try {
        await tagInput.waitFor({ state: "visible", timeout: 3000 });
        break;
      } catch {
        await modal.locator('[data-testid="tag-picker-trigger"]').click();
      }
    }
    await tagInput.fill("urgent");
    await tagInput.press("Enter");

    // Close popover by clicking elsewhere
    await modal.locator('[data-testid="task-detail-title"]').click();

    // Verify tag displays in sidebar
    await expect(modal).toContainText("urgent");

    // Close and reopen to verify persistence
    await page.locator('[data-testid="task-detail-close"]').click();
    await expect(modal).not.toBeVisible();

    await page.keyboard.press("e");
    await expect(modal).toBeVisible();
    await expect(modal).toContainText("urgent");
  });

  test("remove tag in modal", async ({ authenticatedPage: page }) => {
    // Create task with tag via token parsing
    await api.createTask({ title: "Tagged task #urgent" });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    // Open modal
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("e");

    const modal = page.locator('[data-testid="task-detail-modal"]');
    await expect(modal).toBeVisible();

    // Verify tag "urgent" is displayed, then click remove.
    // Use retry loop — background refetch may re-render modal content,
    // detaching the element or reverting the optimistic removal.
    await expect(modal).toContainText("urgent");
    const tagRemove = modal.locator('[data-testid="tag-remove"]');
    for (let attempt = 0; attempt < 3; attempt++) {
      await tagRemove.click({ timeout: 3000 });
      const gone = await tagRemove
        .waitFor({ state: "detached", timeout: 3000 })
        .then(() => true)
        .catch(() => false);
      if (gone) break;
    }

    // Wait for store round-trip to complete — the tag remove button should disappear
    await expect(modal.locator('[data-testid="tag-remove"]')).toHaveCount(0, {
      timeout: 10_000,
    });

    // Close and reopen to verify persistence
    await page.locator('[data-testid="task-detail-close"]').click();
    await expect(modal).not.toBeVisible();

    await page.keyboard.press("e");
    await expect(modal).toBeVisible();
    await expect(modal.locator('[data-testid="tag-remove"]')).toHaveCount(0);
  });
});
