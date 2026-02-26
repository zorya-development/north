import { expect } from "@playwright/test";
import { test } from "../fixtures/auth";
import { ApiHelper } from "../fixtures/api";

let api: ApiHelper;

test.describe("Keyboard Navigation", () => {
  test.beforeEach(async ({ authenticatedPage }) => {
    api = new ApiHelper(authenticatedPage.context());
    await api.deleteAllTasks();
  });

  test.afterEach(async () => {
    await api.deleteAllTasks();
  });

  test("cursor moves with arrow keys", async ({ authenticatedPage: page }) => {
    await api.createTask({ title: "Task A" });
    await api.createTask({ title: "Task B" });
    await api.createTask({ title: "Task C" });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    // No selection initially — press Down to select first task
    await page.keyboard.press("ArrowDown");
    await expect(
      page.locator('[data-testid="task-row"][data-focused="true"]'),
    ).toContainText("Task A");

    // Down again — Task B selected
    await page.keyboard.press("ArrowDown");
    await expect(
      page.locator('[data-testid="task-row"][data-focused="true"]'),
    ).toContainText("Task B");

    // Up — back to Task A
    await page.keyboard.press("ArrowUp");
    await expect(
      page.locator('[data-testid="task-row"][data-focused="true"]'),
    ).toContainText("Task A");
  });

  test("arrow right enters first child, arrow left returns to parent", async ({
    authenticatedPage: page,
  }) => {
    const parent = await api.createTask({ title: "Parent" });
    await api.createTask({ title: "Child", parent_id: parent.id });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    // Select parent
    await page.keyboard.press("ArrowDown");
    await expect(
      page.locator('[data-testid="task-row"][data-focused="true"]'),
    ).toContainText("Parent");

    // Right — move to first child
    await page.keyboard.press("ArrowRight");
    await expect(
      page.locator('[data-testid="task-row"][data-focused="true"]'),
    ).toContainText("Child");

    // Left — back to parent
    await page.keyboard.press("ArrowLeft");
    await expect(
      page.locator('[data-testid="task-row"][data-focused="true"]'),
    ).toContainText("Parent");
  });

  test("Enter starts inline edit, Enter saves", async ({
    authenticatedPage: page,
  }) => {
    await api.createTask({ title: "Original Title" });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    // Select task and open inline editor
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("Enter");

    const editInput = page.locator('[data-testid="inline-edit-input"]');
    await expect(editInput).toBeVisible();

    // Clear and type a new title
    await editInput.fill("Updated Title");
    await editInput.press("Enter");

    // Inline editor should close and updated title should be visible
    await expect(editInput).not.toBeVisible();
    await expect(
      page.locator('[data-testid="task-row"]').first(),
    ).toContainText("Updated Title");
  });

  test("Enter starts inline edit, Escape cancels", async ({
    authenticatedPage: page,
  }) => {
    await api.createTask({ title: "Original Title" });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    // Select task and open inline editor
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("Enter");

    const editInput = page.locator('[data-testid="inline-edit-input"]');
    await expect(editInput).toBeVisible();

    // Type something then cancel
    await editInput.fill("Discarded Edit");
    await editInput.press("Escape");

    // Original title should be preserved
    await expect(editInput).not.toBeVisible();
    await expect(
      page.locator('[data-testid="task-row"]').first(),
    ).toContainText("Original Title");
  });

  test("Space toggles completion — task disappears from active list", async ({
    authenticatedPage: page,
  }) => {
    await api.createTask({ title: "Task to complete" });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    await expect(page.locator('[data-testid="task-row"]')).toHaveCount(1);

    // Select task and toggle completion with Space
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("Space");

    // Task is completed (verified via API)
    const tasks = await api.listTasks();
    expect(tasks).toHaveLength(1);
    expect(tasks[0].completed_at).not.toBeNull();
  });

  test("E opens task detail modal, Escape closes it", async ({
    authenticatedPage: page,
  }) => {
    await api.createTask({ title: "Detail Task" });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    // Select task and open detail modal with E
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("e");

    const modal = page.locator('[data-testid="task-detail-modal"]');
    await expect(modal).toBeVisible();

    // Escape closes the modal
    await page.keyboard.press("Escape");
    await expect(modal).not.toBeVisible();
  });

  test("? opens keybindings help modal, Escape dismisses it", async ({
    authenticatedPage: page,
  }) => {
    await page.goto("/inbox");
    await page
      .locator(
        '[data-testid="empty-task-list"], [data-testid="task-list"]',
      )
      .first()
      .waitFor({ state: "visible" });

    // Press ? to open keybindings modal
    await page.keyboard.press("?");

    const modal = page.locator('[data-testid="keybindings-modal"]');
    await expect(modal).toBeVisible();
    await expect(modal).toContainText("Keyboard shortcuts");

    // Escape dismisses the modal
    await page.keyboard.press("Escape");
    await expect(modal).not.toBeVisible();
  });

  test("Shift+Down / Shift+Up reorders siblings", async ({
    authenticatedPage: page,
  }) => {
    await api.createTask({ title: "Task A" });
    await api.createTask({ title: "Task B" });
    await api.createTask({ title: "Task C" });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    const rows = page.locator('[data-testid="task-row"]');

    // Verify initial order
    await expect(rows.nth(0)).toContainText("Task A");
    await expect(rows.nth(1)).toContainText("Task B");
    await expect(rows.nth(2)).toContainText("Task C");

    // Select Task A and move it down
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("Shift+ArrowDown");

    // Order should now be B, A, C
    await expect(rows.nth(0)).toContainText("Task B");
    await expect(rows.nth(1)).toContainText("Task A");
    await expect(rows.nth(2)).toContainText("Task C");

    // Move Task A back up
    await page.keyboard.press("Shift+ArrowUp");

    // Order restored: A, B, C
    await expect(rows.nth(0)).toContainText("Task A");
    await expect(rows.nth(1)).toContainText("Task B");
    await expect(rows.nth(2)).toContainText("Task C");
  });

  test("Shift+Right indents task, Shift+Left unindents it", async ({
    authenticatedPage: page,
  }) => {
    await api.createTask({ title: "Task A" });
    await api.createTask({ title: "Task B" });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    // Select Task A then move to Task B
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("ArrowDown");
    await expect(
      page.locator('[data-testid="task-row"][data-focused="true"]'),
    ).toContainText("Task B");

    // Shift+Right: indent Task B under Task A
    await page.keyboard.press("Shift+ArrowRight");

    // Navigating Left from Task B should move to its parent (Task A)
    await page.keyboard.press("ArrowLeft");
    await expect(
      page.locator('[data-testid="task-row"][data-focused="true"]'),
    ).toContainText("Task A");

    // Navigate back to Task B (Right = first child)
    await page.keyboard.press("ArrowRight");
    await expect(
      page.locator('[data-testid="task-row"][data-focused="true"]'),
    ).toContainText("Task B");

    // Shift+Left: unindent Task B back to root
    await page.keyboard.press("Shift+ArrowLeft");

    // Up from Task B should now reach Task A (they are siblings; B lands below A after unindent)
    await page.keyboard.press("ArrowUp");
    await expect(
      page.locator('[data-testid="task-row"][data-focused="true"]'),
    ).toContainText("Task A");
  });

  test("Delete shows confirmation, Enter confirms deletion", async ({
    authenticatedPage: page,
  }) => {
    await api.createTask({ title: "Task to delete" });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    await expect(page.locator('[data-testid="task-row"]')).toHaveCount(1);

    // Select task and press Delete
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("Delete");

    // Status bar should show confirmation prompt
    const statusBar = page.locator('[data-testid="status-bar"]');
    await expect(statusBar).toBeVisible();

    // Confirm deletion with Enter
    await page.keyboard.press("Enter");

    // Task should be gone
    await expect(page.locator('[data-testid="task-row"]')).toHaveCount(0);
  });

  test("keyboard shortcuts are suppressed when modal is open", async ({
    authenticatedPage: page,
  }) => {
    await api.createTask({ title: "Task A" });
    await api.createTask({ title: "Task B" });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    // Select first task
    await page.keyboard.press("ArrowDown");
    await expect(
      page.locator('[data-testid="task-row"][data-focused="true"]'),
    ).toContainText("Task A");

    // Open detail modal
    await page.keyboard.press("e");
    const modal = page.locator('[data-testid="task-detail-modal"]');
    await expect(modal).toBeVisible();

    // Press Space — should NOT toggle completion while modal is open
    await page.keyboard.press("Space");

    // Close modal
    await page.keyboard.press("Escape");
    await expect(modal).not.toBeVisible();

    // Both tasks should still be present (Space didn't complete the task)
    await expect(page.locator('[data-testid="task-row"]')).toHaveCount(2);
  });
});
