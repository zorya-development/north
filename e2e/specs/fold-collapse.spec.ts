import { expect } from "@playwright/test";
import { test } from "../fixtures/auth";
import { ApiHelper } from "../fixtures/api";

let api: ApiHelper;

test.describe("Fold/Collapse Subtasks", () => {
  test.beforeEach(async ({ authenticatedPage }) => {
    api = new ApiHelper(authenticatedPage.context());
    await api.deleteAllTasks();
    // Clear fold state from previous runs
    await authenticatedPage.evaluate(() => {
      Object.keys(localStorage)
        .filter((k) => k.startsWith("north:collapsed:"))
        .forEach((k) => localStorage.removeItem(k));
    });
  });

  test.afterEach(async () => {
    await api.deleteAllTasks();
  });

  test("Z key folds and unfolds children", async ({
    authenticatedPage: page,
  }) => {
    const parent = await api.createTask({ title: "Parent" });
    await api.createTask({ title: "Child A", parent_id: parent.id });
    await api.createTask({ title: "Child B", parent_id: parent.id });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    const rows = page.locator('[data-testid="task-row"]');
    await expect(rows).toHaveCount(3);

    // Select parent and fold
    await page.keyboard.press("ArrowDown");
    await expect(
      page.locator('[data-testid="task-row"][data-focused="true"]'),
    ).toContainText("Parent");
    await page.keyboard.press("z");

    // Children should be hidden
    await expect(rows).toHaveCount(1);
    await expect(rows.first()).toContainText("Parent");

    // Unfold
    await page.keyboard.press("z");
    await expect(rows).toHaveCount(3);
  });

  test("Z key is no-op on leaf tasks", async ({
    authenticatedPage: page,
  }) => {
    await api.createTask({ title: "Leaf Task" });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    const rows = page.locator('[data-testid="task-row"]');
    await expect(rows).toHaveCount(1);

    // Select and press Z — nothing should change
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("z");
    await expect(rows).toHaveCount(1);
  });

  test("Right arrow on folded task unfolds and navigates to first child", async ({
    authenticatedPage: page,
  }) => {
    const parent = await api.createTask({ title: "Parent" });
    await api.createTask({ title: "Child", parent_id: parent.id });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    const rows = page.locator('[data-testid="task-row"]');

    // Select parent and fold
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("z");
    await expect(rows).toHaveCount(1);

    // Right arrow should unfold and move cursor to child
    await page.keyboard.press("ArrowRight");
    await expect(rows).toHaveCount(2);
    await expect(
      page.locator('[data-testid="task-row"][data-focused="true"]'),
    ).toContainText("Child");
  });

  test("fold state persists across page navigation", async ({
    authenticatedPage: page,
  }) => {
    const parent = await api.createTask({ title: "Parent" });
    await api.createTask({ title: "Child", parent_id: parent.id });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    const rows = page.locator('[data-testid="task-row"]');
    await expect(rows).toHaveCount(2);

    // Fold
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("z");
    await expect(rows).toHaveCount(1);

    // Navigate away and back
    await page
      .locator('[data-testid="sidebar-nav-item"][data-href="/today"]')
      .click();
    await page.waitForURL("**/today");
    await page
      .locator('[data-testid="sidebar-nav-item"][data-href="/inbox"]')
      .click();
    await page.waitForURL("**/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    // Fold state should be preserved
    await expect(rows).toHaveCount(1);
    await expect(rows.first()).toContainText("Parent");
  });

  test("fold state is per-page", async ({ authenticatedPage: page }) => {
    const parent = await api.createTask({ title: "Parent" });
    await api.createTask({ title: "Child", parent_id: parent.id });

    // Fold on inbox
    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("z");
    await expect(page.locator('[data-testid="task-row"]')).toHaveCount(1);

    // All Tasks page should NOT be folded
    await page
      .locator('[data-testid="sidebar-nav-item"][data-href="/tasks"]')
      .click();
    await page.waitForURL("**/tasks");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });
    await expect(page.locator('[data-testid="task-row"]')).toHaveCount(2);
  });

  test("folding hides nested descendants", async ({
    authenticatedPage: page,
  }) => {
    const grandparent = await api.createTask({ title: "Grandparent" });
    const parent = await api.createTask({
      title: "Parent",
      parent_id: grandparent.id,
    });
    await api.createTask({ title: "Child", parent_id: parent.id });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    const rows = page.locator('[data-testid="task-row"]');
    await expect(rows).toHaveCount(3);

    // Fold grandparent — both parent and child should hide
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("z");
    await expect(rows).toHaveCount(1);
    await expect(rows.first()).toContainText("Grandparent");
  });

  test("keybindings modal shows Z shortcut", async ({
    authenticatedPage: page,
  }) => {
    await page.goto("/inbox");
    await page
      .locator(
        '[data-testid="empty-task-list"], [data-testid="task-list"]',
      )
      .first()
      .waitFor({ state: "visible" });

    await page.keyboard.press("?");

    const modal = page.locator('[data-testid="keybindings-modal"]');
    await expect(modal).toBeVisible();
    await expect(modal).toContainText("Fold/unfold subtasks");

    await page.keyboard.press("Escape");
  });
});
