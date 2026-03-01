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
    expect(names.map((n) => n.trim()).sort()).toEqual(["feature", "urgent"]);
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
