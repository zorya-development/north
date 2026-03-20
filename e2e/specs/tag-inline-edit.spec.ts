import { expect } from "@playwright/test";
import { test } from "../fixtures/auth";
import { ApiHelper } from "../fixtures/api";

let api: ApiHelper;

test.describe("Tag inline edit", () => {
  test.beforeEach(async ({ authenticatedPage }) => {
    api = new ApiHelper(authenticatedPage.context());
    await api.deleteAllTasks();
  });

  test.afterEach(async () => {
    await api.deleteAllTasks();
  });

  test("create with tag via inline, re-edit shows title with tag", async ({
    authenticatedPage: page,
  }) => {
    // Seed a task so the list is visible, then create inline with tag
    await api.createTask({ title: "Seed task" });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    // Use the add-task button to open inline create
    await page.locator('[data-testid="ttl-add-task"]').click();

    const createInput = page.locator('[data-testid="inline-create-input"]');
    await expect(createInput).toBeVisible();
    await createInput.fill("Buy milk #shopping");
    await createInput.press("Enter");

    // Wait for the new task to appear (2 tasks total now)
    const rows = page.locator('[data-testid="task-row"]');
    await expect(rows).toHaveCount(2);

    // Dismiss inline create and navigate to the new task
    await createInput.press("Escape");
    await expect(createInput).not.toBeVisible();

    // ArrowDown selects first task (Buy milk, created at top)
    await page.keyboard.press("ArrowDown");
    const focusedRow = page.locator(
      '[data-testid="task-row"][data-focused="true"]',
    );
    await expect(focusedRow).toContainText("Buy milk");

    // Enter edit mode
    await page.keyboard.press("Enter");

    const editInput = page.locator('[data-testid="inline-edit-input"]');
    await expect(editInput).toBeVisible();

    // Should show the title with reconstructed tag
    await expect(editInput).toHaveValue("Buy milk #shopping");
  });

  test("edit tags: add and remove via inline edit", async ({
    authenticatedPage: page,
  }) => {
    // Create task with a tag via API
    await api.createTask({ title: "Buy milk #shopping" });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    const row = page.locator('[data-testid="task-row"]');
    await expect(row).toHaveCount(1);

    // Enter edit mode
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("Enter");

    const editInput = page.locator('[data-testid="inline-edit-input"]');
    await expect(editInput).toBeVisible();
    await expect(editInput).toHaveValue("Buy milk #shopping");

    // Replace tag: remove #shopping, add #grocery
    await editInput.fill("Buy milk #grocery");
    await editInput.press("Enter");

    // Wait for save to propagate
    await expect(editInput).not.toBeVisible();

    // Re-enter edit to verify the tag was changed
    await page.keyboard.press("Enter");
    await expect(editInput).toBeVisible();
    await expect(editInput).toHaveValue("Buy milk #grocery");
  });

  test("remove all tags via inline edit", async ({
    authenticatedPage: page,
  }) => {
    // Create task with two tags via API
    await api.createTask({ title: "Buy milk #shopping #urgent" });

    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    const row = page.locator('[data-testid="task-row"]');
    await expect(row).toHaveCount(1);

    // Enter edit mode
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("Enter");

    const editInput = page.locator('[data-testid="inline-edit-input"]');
    await expect(editInput).toBeVisible();
    await expect(editInput).toHaveValue("Buy milk #shopping #urgent");

    // Remove all tags — just the title
    await editInput.fill("Buy milk");
    await editInput.press("Enter");

    await expect(editInput).not.toBeVisible();

    // Re-enter edit to verify tags are gone
    await page.keyboard.press("Enter");
    await expect(editInput).toBeVisible();
    await expect(editInput).toHaveValue("Buy milk");
  });
});
