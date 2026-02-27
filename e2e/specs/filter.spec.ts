import { expect } from "@playwright/test";
import { test } from "../fixtures/auth";
import { ApiHelper } from "../fixtures/api";

let api: ApiHelper;

test.describe("Filter Page", () => {
  test.beforeEach(async ({ authenticatedPage }) => {
    api = new ApiHelper(authenticatedPage.context());
    await api.deleteAllTasks();
    await api.deleteAllFilters();
  });

  test.afterEach(async () => {
    await api.deleteAllTasks();
    await api.deleteAllFilters();
  });

  test("valid DSL query shows matching tasks", async ({
    authenticatedPage: page,
  }) => {
    await api.createTask({ title: "Active Task" });
    const task = await api.createTask({ title: "Completed Task" });
    await api.updateTask(task.id, {
      completed_at: new Date().toISOString(),
    });

    await page.goto("/filters/new");

    const queryInput = page.locator('[data-testid="filter-query-input"]');
    await expect(queryInput).toBeVisible();

    // Type a query for active tasks
    await queryInput.fill("status = active");

    // Click search
    await page.locator('[data-testid="filter-search-btn"]').click();

    // Should show only the active task
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    const rows = page.locator('[data-testid="task-row"]');
    await expect(rows).toHaveCount(1);
    await expect(rows.first()).toContainText("Active Task");
  });

  test("invalid DSL shows error message", async ({
    authenticatedPage: page,
  }) => {
    await page.goto("/filters/new");

    const queryInput = page.locator('[data-testid="filter-query-input"]');
    await queryInput.fill("invalid ??? query");

    // Wait for parse error to appear
    const error = page.locator('[data-testid="filter-error"]');
    await expect(error).toBeVisible();

    // Search button should be disabled
    await expect(
      page.locator('[data-testid="filter-search-btn"]'),
    ).toBeDisabled();
  });

  test("save filter and it appears in sidebar", async ({
    authenticatedPage: page,
  }) => {
    await api.createTask({ title: "Filter Test Task" });

    await page.goto("/filters/new");

    const queryInput = page.locator('[data-testid="filter-query-input"]');
    await queryInput.fill("status = active");

    // Click save
    await page.locator('[data-testid="filter-save-btn"]').click();

    // Save modal should appear
    const saveModal = page.locator('[data-testid="filter-save-modal"]');
    await expect(saveModal).toBeVisible();

    // Enter filter title
    const titleInput = page.locator('[data-testid="filter-save-input"]');
    await titleInput.fill("My Active Filter");
    await titleInput.press("Enter");

    // Modal should close
    await expect(saveModal).not.toBeVisible();

    // Filter should appear in sidebar
    const filterItem = page.locator('[data-testid="sidebar-filter-item"]');
    await expect(filterItem.first()).toContainText("My Active Filter");

    // Navigate away and back via saved filter
    await page.goto("/inbox");
    await page
      .locator(
        '[data-testid="task-list"], [data-testid="empty-task-list"]',
      )
      .first()
      .waitFor({ state: "visible" });

    // Click on the saved filter in sidebar
    await filterItem.first().click();
    await expect(page).toHaveURL(/\/filters\/\d+/);

    // Query should be pre-filled
    await expect(
      page.locator('[data-testid="filter-query-input"]'),
    ).toHaveValue("status = active");
  });
});
