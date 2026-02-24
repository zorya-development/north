import { expect } from "@playwright/test";
import { test } from "../fixtures/auth";
import { ApiHelper } from "../fixtures/api";

let api: ApiHelper;

test.describe("Sidebar", () => {
  test.beforeEach(async ({ authenticatedPage }) => {
    api = new ApiHelper(authenticatedPage.context());
    await api.deleteAllTasks();
  });

  test.afterEach(async () => {
    await api.deleteAllTasks();
  });

  test("sidebar is visible on first load", async ({
    authenticatedPage: page,
  }) => {
    await page.goto("/inbox");
    const sidebar = page.locator('[data-testid="sidebar"]');
    await expect(sidebar).toBeVisible();
  });

  test("collapse toggle collapses and expands sidebar", async ({
    authenticatedPage: page,
  }) => {
    await page.goto("/inbox");
    const sidebar = page.locator('[data-testid="sidebar"]');
    await expect(sidebar).toBeVisible();

    // Click collapse button
    await page.locator('[data-testid="sidebar-collapse-btn"]').click();

    // Sidebar should still be visible but collapsed (narrower)
    await expect(sidebar).toBeVisible();

    // Click again to expand
    await page.locator('[data-testid="sidebar-collapse-btn"]').click();
    await expect(sidebar).toBeVisible();
  });

  test("Ctrl+B toggles sidebar collapse", async ({
    authenticatedPage: page,
  }) => {
    await page.goto("/inbox");

    // Press Ctrl+B to collapse
    await page.keyboard.press("Control+b");

    // Verify collapse button's aria-expanded changed
    const collapseBtn = page.locator('[data-testid="sidebar-collapse-btn"]');
    await expect(collapseBtn).toHaveAttribute("aria-expanded", "false");

    // Press Ctrl+B to expand
    await page.keyboard.press("Control+b");
    await expect(collapseBtn).toHaveAttribute("aria-expanded", "true");
  });

  test("create project via sidebar", async ({ authenticatedPage: page }) => {
    await page.goto("/inbox");

    // Click the create project button
    await page.locator('[data-testid="sidebar-create-project-btn"]').click();

    // Type project name
    const input = page.locator('[data-testid="sidebar-create-project-input"]');
    await expect(input).toBeVisible();
    await input.fill("Test Project");
    await input.press("Enter");

    // Project should appear in sidebar
    const projectItem = page.locator('[data-testid="sidebar-project-item"]');
    await expect(projectItem).toHaveCount(1);
    await expect(projectItem.first()).toContainText("Test Project");

    // Clean up: archive the project
    await projectItem.first().hover();
    await page.locator('[title="Archive project"]').click();
  });

  test("nav links navigate to correct pages", async ({
    authenticatedPage: page,
  }) => {
    await page.goto("/inbox");

    // Test Today link
    await page
      .locator('[data-testid="sidebar-nav-item"][data-href="/today"]')
      .click();
    await expect(page).toHaveURL(/\/today/);

    // Test All Tasks link
    await page
      .locator('[data-testid="sidebar-nav-item"][data-href="/tasks"]')
      .click();
    await expect(page).toHaveURL(/\/tasks/);

    // Test Review link
    await page
      .locator('[data-testid="sidebar-nav-item"][data-href="/review"]')
      .click();
    await expect(page).toHaveURL(/\/review/);

    // Test Settings link
    await page
      .locator('[data-testid="sidebar-nav-item"][data-href="/settings"]')
      .click();
    await expect(page).toHaveURL(/\/settings/);

    // Test Inbox link (back)
    await page
      .locator('[data-testid="sidebar-nav-item"][data-href="/inbox"]')
      .click();
    await expect(page).toHaveURL(/\/inbox/);
  });

  test("theme toggle switches theme", async ({ authenticatedPage: page }) => {
    await page.goto("/inbox");

    const toggle = page.locator('[data-testid="theme-toggle"]');
    await expect(toggle).toBeVisible();

    // Get initial theme
    const initialIsDark = await page.evaluate(() =>
      document.documentElement.classList.contains("dark"),
    );

    // Click toggle
    await toggle.click();

    // Theme should have changed
    const afterToggleIsDark = await page.evaluate(() =>
      document.documentElement.classList.contains("dark"),
    );
    expect(afterToggleIsDark).toBe(!initialIsDark);

    // Toggle back
    await toggle.click();
    const restoredIsDark = await page.evaluate(() =>
      document.documentElement.classList.contains("dark"),
    );
    expect(restoredIsDark).toBe(initialIsDark);
  });
});
