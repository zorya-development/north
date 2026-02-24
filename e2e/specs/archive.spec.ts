import { expect } from "@playwright/test";
import { test } from "../fixtures/auth";
import { ApiHelper } from "../fixtures/api";

let api: ApiHelper;

test.describe("Archive Page", () => {
  test.beforeEach(async ({ authenticatedPage }) => {
    api = new ApiHelper(authenticatedPage.context());
    await api.deleteAllTasks();
  });

  test.afterEach(async () => {
    await api.deleteAllTasks();
  });

  test("archived projects appear in list", async ({
    authenticatedPage: page,
  }) => {
    // Create and archive a project
    const project = await api.createProject({ title: "Archived Project" });
    await api.updateProject(project.id, { status: "archived" });

    await page.goto("/archive");

    const projectRow = page.locator('[data-testid="archive-project"]');
    await expect(projectRow).toHaveCount(1);
    await expect(projectRow.first()).toContainText("Archived Project");
  });

  test("unarchive moves project back to sidebar", async ({
    authenticatedPage: page,
  }) => {
    const project = await api.createProject({ title: "Restore Me" });
    await api.updateProject(project.id, { status: "archived" });

    await page.goto("/archive");

    const projectRow = page.locator('[data-testid="archive-project"]');
    await expect(projectRow).toHaveCount(1);

    // Click unarchive
    await page.locator('[data-testid="archive-unarchive-btn"]').click();

    // Project should disappear from archive
    await expect(projectRow).toHaveCount(0);

    // Project should appear in sidebar
    const sidebarProject = page.locator(
      '[data-testid="sidebar-project-item"]',
    );
    await expect(sidebarProject).toHaveCount(1);
    await expect(sidebarProject.first()).toContainText("Restore Me");

    // Clean up: re-archive
    await api.updateProject(project.id, { status: "archived" });
  });

  test("delete archived project removes it", async ({
    authenticatedPage: page,
  }) => {
    const project = await api.createProject({ title: "Delete Me" });
    await api.updateProject(project.id, { status: "archived" });

    await page.goto("/archive");

    const projectRow = page.locator('[data-testid="archive-project"]');
    await expect(projectRow).toHaveCount(1);

    // Click delete — should trigger confirmation
    await page.locator('[data-testid="archive-delete-btn"]').click();

    // A confirm dialog or status bar prompt may appear
    // Accept it (browser dialog) or press Enter (status bar confirmation)
    page.once("dialog", (dialog) => dialog.accept());

    // Project should disappear
    await expect(projectRow).toHaveCount(0, { timeout: 10_000 });
  });
});
