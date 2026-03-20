import { expect } from "@playwright/test";
import { test } from "../fixtures/auth";
import { ApiHelper } from "../fixtures/api";

let api: ApiHelper;

test.describe("Tag filter bar", () => {
  test.beforeEach(async ({ authenticatedPage }) => {
    api = new ApiHelper(authenticatedPage.context());
    await api.deleteAllTasks();
  });

  test.afterEach(async () => {
    await api.deleteAllTasks();
  });

  test("shows tag filter bar with unique tags from list tasks", async ({
    authenticatedPage: page,
  }) => {
    await api.createTask({ title: "Task A #urgent" });
    await api.createTask({ title: "Task B #feature" });
    await api.createTask({ title: "Task C #urgent" });

    await page.goto("/inbox");
    await page.locator('[data-testid="task-list"]').waitFor({ state: "visible" });

    const bar = page.locator('[data-testid="tag-filter-bar"]');
    await expect(bar).toBeVisible();

    const buttons = bar.locator('[data-testid="tag-filter-button"]');
    await expect(buttons).toHaveCount(2);

    const names = await buttons.allTextContents();
    expect(names.map((n) => n.trim()).sort()).toEqual(["#feature", "#urgent"]);
  });

  test("clicking a tag filters tasks to only matching ones", async ({
    authenticatedPage: page,
  }) => {
    await api.createTask({ title: "Fix login #bug" });
    await api.createTask({ title: "Add dashboard #feature" });
    await api.createTask({ title: "Fix signup #bug" });

    await page.goto("/inbox");
    await page.locator('[data-testid="task-list"]').waitFor({ state: "visible" });

    const rows = page.locator('[data-testid="task-row"]');
    await expect(rows).toHaveCount(3);

    // Click the "bug" tag filter
    const bugButton = page.locator(
      '[data-testid="tag-filter-button"]',
    ).filter({ hasText: "bug" });
    await bugButton.click();

    // Only bug-tagged tasks should be visible
    await expect(rows).toHaveCount(2);
    await expect(page.getByText("Fix login")).toBeVisible();
    await expect(page.getByText("Fix signup")).toBeVisible();
    await expect(page.getByText("Add dashboard")).not.toBeVisible();
  });

  test("multiple active tags use AND logic", async ({
    authenticatedPage: page,
  }) => {
    await api.createTask({ title: "Task AB #alpha #beta" });
    await api.createTask({ title: "Task A #alpha" });
    await api.createTask({ title: "Task B #beta" });

    await page.goto("/inbox");
    await page.locator('[data-testid="task-list"]').waitFor({ state: "visible" });

    const rows = page.locator('[data-testid="task-row"]');
    await expect(rows).toHaveCount(3);

    // Activate alpha
    await page
      .locator('[data-testid="tag-filter-button"]')
      .filter({ hasText: "alpha" })
      .click();
    await expect(rows).toHaveCount(2);

    // Activate beta too — AND logic means only Task AB matches
    await page
      .locator('[data-testid="tag-filter-button"]')
      .filter({ hasText: "beta" })
      .click();
    await expect(rows).toHaveCount(1);
    await expect(page.getByText("Task AB")).toBeVisible();
  });

  test("clicking active tag deactivates filter", async ({
    authenticatedPage: page,
  }) => {
    await api.createTask({ title: "Tagged #urgent" });
    await api.createTask({ title: "Untagged task" });

    await page.goto("/inbox");
    await page.locator('[data-testid="task-list"]').waitFor({ state: "visible" });

    const rows = page.locator('[data-testid="task-row"]');
    await expect(rows).toHaveCount(2);

    const urgentBtn = page.locator(
      '[data-testid="tag-filter-button"]',
    ).filter({ hasText: "urgent" });

    // Activate
    await urgentBtn.click();
    await expect(rows).toHaveCount(1);

    // Deactivate — all tasks should return
    await urgentBtn.click();
    await expect(rows).toHaveCount(2);
  });

  test("tag bar hidden when no tasks have tags", async ({
    authenticatedPage: page,
  }) => {
    await api.createTask({ title: "No tags here" });
    await api.createTask({ title: "Also no tags" });

    await page.goto("/inbox");
    await page.locator('[data-testid="task-list"]').waitFor({ state: "visible" });

    const bar = page.locator('[data-testid="tag-filter-bar"]');
    await expect(bar).not.toBeVisible();
  });

  test("new tag added in modal appears and is selectable in toolbar", async ({
    authenticatedPage: page,
  }) => {
    await api.createTask({ title: "Tagged task" });
    await api.createTask({ title: "Other task" });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    const rows = page.locator('[data-testid="task-row"]');
    await expect(rows).toHaveCount(2);

    // No tag filter bar initially
    const bar = page.locator('[data-testid="tag-filter-bar"]');
    await expect(bar).not.toBeVisible();

    // Open task detail modal for first task
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("e");

    const modal = page.locator('[data-testid="task-detail-modal"]');
    await expect(modal).toBeVisible();

    // Add a tag via the tag picker
    await modal.locator('[data-testid="tag-picker-trigger"]').click();
    await modal.locator('[data-testid="tag-picker-input"]').fill("urgent");
    await modal.locator('[data-testid="tag-picker-input"]').press("Enter");

    // Close popover by clicking elsewhere
    await modal.locator('[data-testid="task-detail-title"]').click();

    // Close modal
    await page.locator('[data-testid="task-detail-close"]').click();
    await expect(modal).not.toBeVisible();

    // Tag filter bar should now appear with the new tag
    await expect(bar).toBeVisible();
    const tagBtn = bar.locator('[data-testid="tag-filter-button"]').filter({ hasText: "urgent" });
    await expect(tagBtn).toBeVisible();

    // Click the tag to filter — only the tagged task should remain
    await tagBtn.click();
    await expect(rows).toHaveCount(1);
    await expect(page.getByText("Tagged task")).toBeVisible();
    await expect(page.getByText("Other task")).not.toBeVisible();
  });

  test("new tag created inline appears and is selectable in toolbar", async ({
    authenticatedPage: page,
  }) => {
    await api.createTask({ title: "Existing task" });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    // No tag filter bar initially
    const bar = page.locator('[data-testid="tag-filter-bar"]');
    await expect(bar).not.toBeVisible();

    // Create a new task with a tag using inline input
    const addBtn = page.locator('[data-testid="ttl-add-task"]');
    await addBtn.click();

    const input = page.locator('[data-testid="inline-create-input"]');
    await expect(input).toBeVisible();
    await input.fill("New task #important");
    await input.press("Enter");

    // Wait for the task to appear
    const rows = page.locator('[data-testid="task-row"]');
    await expect(rows).toHaveCount(2);

    // Tag filter bar should now appear with the new tag
    await expect(bar).toBeVisible({ timeout: 5000 });
    const tagBtn = bar.locator('[data-testid="tag-filter-button"]').filter({ hasText: "important" });
    await expect(tagBtn).toBeVisible();

    // Click the tag to filter — only the tagged task should remain
    await tagBtn.click();
    await expect(rows).toHaveCount(1);
    await expect(page.getByText("New task")).toBeVisible();
    await expect(page.getByText("Existing task")).not.toBeVisible();
  });

  test("k:v tag values become selectable as tasks are created incrementally", async ({
    authenticatedPage: page,
  }) => {
    // Seed one task so the list is visible on load
    await api.createTask({ title: "Critical bug #priority:high" });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    const bar = page.locator('[data-testid="tag-filter-bar"]');
    const rows = page.locator('[data-testid="task-row"]');

    // Helper: create task via inline input
    const createTask = async (title: string) => {
      await page.locator('[data-testid="ttl-add-task"]').click();
      const input = page.locator('[data-testid="inline-create-input"]');
      await expect(input).toBeVisible();
      await input.fill(title);
      await input.press("Enter");
    };

    // Helper: open popover, verify value is selectable and filters correctly
    const expectValueSelectable = async (value: string, taskCount: number) => {
      const chip = bar.locator('[data-testid="kv-filter-chip"]').filter({ hasText: "priority" });
      await chip.click();
      const popover = page.locator('[data-testid="kv-filter-popover"]');
      await expect(popover).toBeVisible();
      const option = popover
        .locator('[data-testid="kv-filter-option"]')
        .filter({ hasText: value });
      await expect(option).toBeVisible();
      await option.click();
      await expect(rows).toHaveCount(taskCount);
      // Deselect to reset filter
      await option.click();
      // Close popover
      await chip.click({ force: true });
    };

    // 1. priority:high already exists from seed — verify selectable
    await expect(bar).toBeVisible();
    await expect(rows).toHaveCount(1);
    await expectValueSelectable("high", 1);

    // 2. Create task with #priority:medium — verify it appears and is selectable
    await createTask("Normal work #priority:medium");
    await expect(rows).toHaveCount(2);
    await expectValueSelectable("medium", 1);

    // 3. Create task with #priority:low — verify it appears and is selectable
    await createTask("Minor tweak #priority:low");
    await expect(rows).toHaveCount(3);
    await expectValueSelectable("low", 1);
  });

  test("active tag has distinct styling from inactive", async ({
    authenticatedPage: page,
  }) => {
    await api.createTask({ title: "Task #alpha" });
    await api.createTask({ title: "Task #beta" });

    await page.goto("/inbox");
    await page.locator('[data-testid="task-list"]').waitFor({ state: "visible" });

    const alphaBtn = page.locator(
      '[data-testid="tag-filter-button"]',
    ).filter({ hasText: "alpha" });

    // Before activation — should have inactive attribute
    await expect(alphaBtn).toHaveAttribute("data-active", "false");

    // Activate
    await alphaBtn.click();
    await expect(alphaBtn).toHaveAttribute("data-active", "true");
  });
});
