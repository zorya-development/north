import { expect } from "@playwright/test";
import { test } from "../fixtures/auth";
import { ApiHelper } from "../fixtures/api";

let api: ApiHelper;

test.describe("Toolbar", () => {
  test.beforeEach(async ({ authenticatedPage }) => {
    api = new ApiHelper(authenticatedPage.context());
    await api.deleteAllTasks();
  });

  test.afterEach(async () => {
    await api.deleteAllTasks();
  });

  test.describe("Search", () => {
    test("typing in search filters tasks on page", async ({
      authenticatedPage: page,
    }) => {
      await api.createTask({ title: "Buy groceries" });
      await api.createTask({ title: "Fix login bug" });
      await api.createTask({ title: "Buy new keyboard" });

      await page.goto("/inbox");
      await page
        .locator('[data-testid="task-list"]')
        .waitFor({ state: "visible" });

      const rows = page.locator('[data-testid="task-row"]');
      await expect(rows).toHaveCount(3);

      // Type in search input
      const searchInput = page.locator('[data-testid="task-search-input"]');
      await searchInput.fill("Buy");

      // Only matching tasks should be visible
      await expect(rows).toHaveCount(2);
      await expect(rows.nth(0)).toContainText("Buy");
      await expect(rows.nth(1)).toContainText("Buy");

      // Clear search restores all tasks
      await searchInput.fill("");
      await expect(rows).toHaveCount(3);
    });

    test("search shows full ancestor path for deeply nested match", async ({
      authenticatedPage: page,
    }) => {
      // Create a 3-level hierarchy: Grandparent -> Parent -> Child
      const grandparent = await api.createTask({ title: "Grandparent task" });
      const parent = await api.createTask({
        title: "Parent task",
        parent_id: grandparent.id,
      });
      await api.createTask({
        title: "Deeply nested child",
        parent_id: parent.id,
      });
      // Create a sibling tree that should NOT appear
      await api.createTask({ title: "Unrelated task" });

      await page.goto("/inbox");
      await page
        .locator('[data-testid="task-list"]')
        .waitFor({ state: "visible" });

      const rows = page.locator('[data-testid="task-row"]');
      await expect(rows).toHaveCount(4);

      // Search for the deeply nested child
      const searchInput = page.locator('[data-testid="task-search-input"]');
      await searchInput.fill("Deeply nested");

      // Should show 3 rows: grandparent, parent, and the matched child
      await expect(rows).toHaveCount(3);
      await expect(rows.nth(0)).toContainText("Grandparent task");
      await expect(rows.nth(1)).toContainText("Parent task");
      await expect(rows.nth(2)).toContainText("Deeply nested child");

      // Clear search restores all tasks
      await searchInput.fill("");
      await expect(rows).toHaveCount(4);
    });

    test("search filters subtasks in task detail modal", async ({
      authenticatedPage: page,
    }) => {
      const parent = await api.createTask({ title: "Parent Task" });
      await api.createTask({ title: "Write tests", parent_id: parent.id });
      await api.createTask({ title: "Fix bug", parent_id: parent.id });
      await api.createTask({ title: "Write docs", parent_id: parent.id });

      await page.goto("/inbox");
      await page
        .locator('[data-testid="task-list"]')
        .waitFor({ state: "visible" });

      // Open task detail modal
      await page.keyboard.press("ArrowDown");
      await page.keyboard.press("e");

      const modal = page.locator('[data-testid="task-detail-modal"]');
      await expect(modal).toBeVisible();

      // All 3 subtasks should be visible
      const subtaskRows = modal.locator('[data-testid="task-row"]');
      await expect(subtaskRows).toHaveCount(3);

      // Type in search input inside the modal
      const searchInput = modal.locator('[data-testid="task-search-input"]');
      await searchInput.fill("Write");

      // Only matching subtasks should be visible
      await expect(subtaskRows).toHaveCount(2);
      await expect(subtaskRows.nth(0)).toContainText("Write");
      await expect(subtaskRows.nth(1)).toContainText("Write");

      // Clear search restores all subtasks
      await searchInput.fill("");
      await expect(subtaskRows).toHaveCount(3);
    });

    test("search input in modal stays focused when typing", async ({
      authenticatedPage: page,
    }) => {
      const parent = await api.createTask({ title: "Parent Task" });
      await api.createTask({ title: "Sub one", parent_id: parent.id });

      await page.goto("/inbox");
      await page
        .locator('[data-testid="task-list"]')
        .waitFor({ state: "visible" });

      // Open task detail modal
      await page.keyboard.press("ArrowDown");
      await page.keyboard.press("e");

      const modal = page.locator('[data-testid="task-detail-modal"]');
      await expect(modal).toBeVisible();

      // Click search input and type character by character
      const searchInput = modal.locator('[data-testid="task-search-input"]');
      await searchInput.click();
      await expect(searchInput).toBeFocused();

      await page.keyboard.type("test", { delay: 50 });

      // Input should still be focused and contain the typed text
      await expect(searchInput).toBeFocused();
      await expect(searchInput).toHaveValue("test");
    });
  });

  test.describe("Completed toggle", () => {
    test("toggling completed on page reveals completed tasks", async ({
      authenticatedPage: page,
    }) => {
      await api.createTask({ title: "Active task" });
      const completed = await api.createTask({ title: "Done task" });
      await api.updateTask(completed.id, {
        completed_at: new Date().toISOString(),
      });

      await page.goto("/inbox");
      await page
        .locator('[data-testid="task-list"]')
        .waitFor({ state: "visible" });

      const rows = page.locator('[data-testid="task-row"]');

      // Only active task visible initially
      await expect(rows).toHaveCount(1);
      await expect(rows.first()).toContainText("Active task");

      // Toggle completed on
      await page.locator('[data-testid="ttl-toggle-completed"]').click();

      // Both tasks visible now
      await expect(rows).toHaveCount(2);

      // Toggle completed off — only active task remains
      await page.locator('[data-testid="ttl-toggle-completed"]').click();
      await expect(rows).toHaveCount(1);
      await expect(rows.first()).toContainText("Active task");
    });

    test("toggling completed in modal reveals completed subtasks", async ({
      authenticatedPage: page,
    }) => {
      const parent = await api.createTask({ title: "Parent Task" });
      await api.createTask({ title: "Active sub", parent_id: parent.id });
      const completedSub = await api.createTask({
        title: "Done sub",
        parent_id: parent.id,
      });
      await api.updateTask(completedSub.id, {
        completed_at: new Date().toISOString(),
      });

      await page.goto("/inbox");
      await page
        .locator('[data-testid="task-list"]')
        .waitFor({ state: "visible" });

      // Open task detail modal
      await page.keyboard.press("ArrowDown");
      await page.keyboard.press("e");

      const modal = page.locator('[data-testid="task-detail-modal"]');
      await expect(modal).toBeVisible();

      const subtaskRows = modal.locator('[data-testid="task-row"]');

      // Only active subtask visible initially
      await expect(subtaskRows).toHaveCount(1);
      await expect(subtaskRows.first()).toContainText("Active sub");

      // Toggle completed on in the modal's toolbar
      await modal.locator('[data-testid="ttl-toggle-completed"]').click();

      // Both subtasks visible now
      await expect(subtaskRows).toHaveCount(2);

      // Toggle completed off — only active subtask remains
      await modal.locator('[data-testid="ttl-toggle-completed"]').click();
      await expect(subtaskRows).toHaveCount(1);
      await expect(subtaskRows.first()).toContainText("Active sub");
    });

    test("cursor clears when focused task is filtered out", async ({
      authenticatedPage: page,
    }) => {
      // Create a parent with sequential_limit=1 and 2 subtasks
      const parent = await api.createTask({ title: "Parent" });
      await api.updateTask(parent.id, { sequential_limit: 1 });
      await api.createTask({ title: "Sub A", parent_id: parent.id });
      await api.createTask({ title: "Sub B", parent_id: parent.id });

      await page.goto("/inbox");
      await page
        .locator('[data-testid="task-list"]')
        .waitFor({ state: "visible" });

      // All tasks visible: Parent, Sub A, Sub B
      const rows = page.locator('[data-testid="task-row"]');
      await expect(rows).toHaveCount(3);

      // Select Sub B with keyboard
      await page.keyboard.press("ArrowDown"); // Parent
      await page.keyboard.press("ArrowDown"); // Sub A
      await page.keyboard.press("ArrowDown"); // Sub B
      await expect(
        page.locator('[data-testid="task-row"][data-focused="true"]'),
      ).toContainText("Sub B");

      // Toggle actionable filter — Sub B disappears (not actionable)
      await page.locator('[data-testid="ttl-toggle-actionable"]').click();
      await expect(rows).toHaveCount(2);

      // Cursor should be cleared since Sub B is no longer visible
      await expect(
        page.locator('[data-testid="task-row"][data-focused="true"]'),
      ).toHaveCount(0);
    });
  });
});
