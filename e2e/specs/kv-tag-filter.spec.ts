import { expect } from "@playwright/test";
import { test } from "../fixtures/auth";
import { ApiHelper } from "../fixtures/api";

let api: ApiHelper;

test.describe("kv tag faceted filtering", () => {
  test.beforeEach(async ({ authenticatedPage }) => {
    api = new ApiHelper(authenticatedPage.context());
    await api.deleteAllTasks();
  });

  test.afterEach(async () => {
    await api.deleteAllTasks();
  });

  test("creates tasks with k:v tags via inline hash syntax", async ({
    authenticatedPage: page,
  }) => {
    await api.createTask({ title: "Fix login #priority:high #type:bug" });
    await api.createTask({ title: "Add dashboard #priority:low #type:feature" });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    const rows = page.locator('[data-testid="task-row"]');
    await expect(rows).toHaveCount(2);

    // Tag filter bar should be visible with k:v tags
    const bar = page.locator('[data-testid="tag-filter-bar"]');
    await expect(bar).toBeVisible();
  });

  test("k:v tags display as grouped chips in filter bar", async ({
    authenticatedPage: page,
  }) => {
    await api.createTask({ title: "Task A #priority:high" });
    await api.createTask({ title: "Task B #priority:low" });
    await api.createTask({ title: "Task C #urgent" });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    const bar = page.locator('[data-testid="tag-filter-bar"]');
    await expect(bar).toBeVisible();

    // k:v tags should appear as a grouped chip (one per key)
    const kvChips = bar.locator('[data-testid="kv-filter-chip"]');
    await expect(kvChips).toHaveCount(1);
    await expect(kvChips.first()).toContainText("priority");

    // Simple tags remain as flat toggle buttons
    const simpleButtons = bar.locator('[data-testid="tag-filter-button"]');
    await expect(simpleButtons).toHaveCount(1);
    await expect(simpleButtons.first()).toContainText("urgent");
  });

  test("clicking k:v chip opens popover with value checkboxes", async ({
    authenticatedPage: page,
  }) => {
    await api.createTask({ title: "Task A #priority:high" });
    await api.createTask({ title: "Task B #priority:low" });
    await api.createTask({ title: "Task C #priority:medium" });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    // Click the priority chip to open popover
    const chip = page
      .locator('[data-testid="kv-filter-chip"]')
      .filter({ hasText: "priority" });
    await chip.click();

    // Popover should appear with value options
    const popover = page.locator('[data-testid="kv-filter-popover"]');
    await expect(popover).toBeVisible();

    // Should list all values for the "priority" key
    const options = popover.locator('[data-testid="kv-filter-option"]');
    await expect(options).toHaveCount(3);

    const labels = await options.allTextContents();
    expect(labels.map((l) => l.trim()).sort()).toEqual([
      "high",
      "low",
      "medium",
    ]);
  });

  test("OR filtering within same key", async ({
    authenticatedPage: page,
  }) => {
    await api.createTask({ title: "Critical bug #priority:high" });
    await api.createTask({ title: "Minor fix #priority:low" });
    await api.createTask({ title: "Nice to have #priority:medium" });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    const rows = page.locator('[data-testid="task-row"]');
    await expect(rows).toHaveCount(3);

    // Open priority chip popover
    const chip = page
      .locator('[data-testid="kv-filter-chip"]')
      .filter({ hasText: "priority" });
    await chip.click();

    const popover = page.locator('[data-testid="kv-filter-popover"]');
    await expect(popover).toBeVisible();

    // Select "high" — should filter to 1 task
    await popover
      .locator('[data-testid="kv-filter-option"]')
      .filter({ hasText: "high" })
      .click();
    await expect(rows).toHaveCount(1);
    await expect(page.getByText("Critical bug")).toBeVisible();

    // Also select "low" — OR within key, should show 2 tasks
    await popover
      .locator('[data-testid="kv-filter-option"]')
      .filter({ hasText: "low" })
      .click();
    await expect(rows).toHaveCount(2);
    await expect(page.getByText("Critical bug")).toBeVisible();
    await expect(page.getByText("Minor fix")).toBeVisible();
    await expect(page.getByText("Nice to have")).not.toBeVisible();
  });

  test("AND filtering between different keys", async ({
    authenticatedPage: page,
  }) => {
    await api.createTask({
      title: "High priority bug #priority:high #type:bug",
    });
    await api.createTask({
      title: "High priority feature #priority:high #type:feature",
    });
    await api.createTask({
      title: "Low priority bug #priority:low #type:bug",
    });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    const rows = page.locator('[data-testid="task-row"]');
    await expect(rows).toHaveCount(3);

    // Select priority:high via popover
    const priorityChip = page
      .locator('[data-testid="kv-filter-chip"]')
      .filter({ hasText: "priority" });
    await priorityChip.click();

    const priorityPopover = page
      .locator('[data-testid="kv-filter-popover"]')
      .filter({ has: page.locator('[data-testid="kv-filter-option"]', { hasText: "high" }) });
    await priorityPopover
      .locator('[data-testid="kv-filter-option"]')
      .filter({ hasText: "high" })
      .click();

    // Close priority popover by re-clicking the chip (toggle)
    await priorityChip.click({ force: true });
    await expect(
      page.locator('[data-testid="kv-filter-popover"]').first(),
    ).not.toBeVisible();

    // 2 tasks with priority:high
    await expect(rows).toHaveCount(2);

    // Now select type:bug via its popover
    const typeChip = page
      .locator('[data-testid="kv-filter-chip"]')
      .filter({ hasText: "type" });
    await typeChip.click();

    const typePopover = page.locator('[data-testid="kv-filter-popover"]').last();
    await expect(typePopover).toBeVisible();
    await typePopover
      .locator('[data-testid="kv-filter-option"]')
      .filter({ hasText: "bug" })
      .click();

    // AND between keys: must be priority:high AND type:bug → 1 task
    await expect(rows).toHaveCount(1);
    await expect(page.getByText("High priority bug")).toBeVisible();
  });

  test("mixed simple and k:v tag filtering", async ({
    authenticatedPage: page,
  }) => {
    await api.createTask({
      title: "Urgent high bug #urgent #priority:high",
    });
    await api.createTask({
      title: "Urgent low task #urgent #priority:low",
    });
    await api.createTask({
      title: "Not urgent high #priority:high",
    });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    const rows = page.locator('[data-testid="task-row"]');
    await expect(rows).toHaveCount(3);

    // Activate simple tag "urgent" (flat button, AND logic)
    const urgentBtn = page
      .locator('[data-testid="tag-filter-button"]')
      .filter({ hasText: "urgent" });
    await urgentBtn.click();
    await expect(rows).toHaveCount(2);

    // Also select priority:high via k:v chip
    const priorityChip = page
      .locator('[data-testid="kv-filter-chip"]')
      .filter({ hasText: "priority" });
    await priorityChip.click();

    const popover = page.locator('[data-testid="kv-filter-popover"]');
    await popover
      .locator('[data-testid="kv-filter-option"]')
      .filter({ hasText: "high" })
      .click();

    // Must have #urgent AND priority:high → 1 task
    await expect(rows).toHaveCount(1);
    await expect(page.getByText("Urgent high bug")).toBeVisible();
  });

  test("deselecting all values in a key removes that key filter", async ({
    authenticatedPage: page,
  }) => {
    await api.createTask({ title: "Task A #priority:high" });
    await api.createTask({ title: "Task B #priority:low" });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    const rows = page.locator('[data-testid="task-row"]');
    await expect(rows).toHaveCount(2);

    // Select priority:high
    const chip = page
      .locator('[data-testid="kv-filter-chip"]')
      .filter({ hasText: "priority" });
    await chip.click();

    const popover = page.locator('[data-testid="kv-filter-popover"]');
    const highOption = popover
      .locator('[data-testid="kv-filter-option"]')
      .filter({ hasText: "high" });

    await highOption.click();
    await expect(rows).toHaveCount(1);

    // Deselect priority:high — filter should clear, all tasks visible
    await highOption.click();
    await expect(rows).toHaveCount(2);
  });
});
