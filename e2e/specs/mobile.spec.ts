import { expect } from "@playwright/test";
import { test } from "../fixtures/auth";
import { ApiHelper } from "../fixtures/api";

const MOBILE_VIEWPORT = { width: 375, height: 812 };

let api: ApiHelper;

test.describe("Mobile layout", () => {
  test.use({ viewport: MOBILE_VIEWPORT });

  test.beforeEach(async ({ authenticatedPage }) => {
    api = new ApiHelper(authenticatedPage.context());
    await api.deleteAllTasks();
    await api.deleteAllProjects();
  });

  test.afterEach(async () => {
    await api.deleteAllTasks();
    await api.deleteAllProjects();
  });

  // ── Hamburger menu + sidebar drawer ──────────────────────────

  test("sidebar is hidden, hamburger opens it", async ({
    authenticatedPage: page,
  }) => {
    await page.goto("/inbox");

    const sidebar = page.locator('[data-testid="sidebar"]');
    await expect(sidebar).not.toBeVisible();

    await page.locator('[aria-label="Open menu"]').click();
    await expect(sidebar).toBeVisible();
  });

  test("sidebar closes on navigation", async ({
    authenticatedPage: page,
  }) => {
    await page.goto("/inbox");

    await page.locator('[aria-label="Open menu"]').click();
    const sidebar = page.locator('[data-testid="sidebar"]');
    await expect(sidebar).toBeVisible();

    await page
      .locator('[data-testid="sidebar-nav-item"][data-href="/today"]')
      .click();
    await expect(page).toHaveURL(/\/today/);
    await expect(sidebar).not.toBeVisible();
  });

  test("sidebar closes on backdrop click", async ({
    authenticatedPage: page,
  }) => {
    await page.goto("/inbox");

    await page.locator('[aria-label="Open menu"]').click();
    const sidebar = page.locator('[data-testid="sidebar"]');
    await expect(sidebar).toBeVisible();

    await page
      .locator('[data-testid="mobile-sidebar-backdrop"]')
      .click({ force: true });
    await expect(sidebar).not.toBeVisible();
  });

  // ── Theme color meta tags ────────────────────────────────────

  test("theme-color meta tags exist and sync with toggle", async ({
    authenticatedPage: page,
  }) => {
    await page.goto("/inbox");

    const count = await page.evaluate(
      () => document.querySelectorAll('meta[name="theme-color"]').length,
    );
    expect(count).toBe(2);

    // Open sidebar to access theme toggle
    await page.locator('[aria-label="Open menu"]').click();
    await page.locator('[data-testid="theme-toggle"]').click();

    const colors = await page.evaluate(() =>
      Array.from(document.querySelectorAll('meta[name="theme-color"]')).map(
        (m) => m.getAttribute("content"),
      ),
    );
    expect(colors[0]).toBe(colors[1]);
  });

  // ── Task detail modal ────────────────────────────────────────

  test("modal is fullscreen on mobile", async ({
    authenticatedPage: page,
  }) => {
    await api.createTask({ title: "Mobile task" });
    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    await page.locator('[data-testid="task-row"]').first().click();
    const modal = page.locator('[data-testid="task-detail-modal"]');
    await expect(modal).toBeVisible();

    const box = await modal.boundingBox();
    expect(box).not.toBeNull();
    expect(box!.x).toBeLessThanOrEqual(1);
    expect(box!.width).toBeGreaterThanOrEqual(MOBILE_VIEWPORT.width - 2);
  });

  test("modal settings panel hidden by default, opens via gear", async ({
    authenticatedPage: page,
  }) => {
    await api.createTask({ title: "Settings task" });
    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    await page.locator('[data-testid="task-row"]').first().click();
    const modal = page.locator('[data-testid="task-detail-modal"]');
    await expect(modal).toBeVisible();

    // Settings panel hidden on mobile
    const settings = modal.locator('[data-testid="task-detail-settings"]');
    await expect(settings).not.toBeVisible();

    // Gear button visible on mobile
    const gearBtn = modal.locator('[data-testid="task-detail-settings-btn"]');
    await expect(gearBtn).toBeVisible();

    // Open settings
    await gearBtn.click();
    await expect(settings).toBeVisible();
  });

  // ── Action bar hidden on mobile ──────────────────────────────

  test("task action bar is hidden on mobile", async ({
    authenticatedPage: page,
  }) => {
    await api.createTask({ title: "Action test" });
    await page.goto("/inbox");
    await page
      .locator('[data-testid="task-list"]')
      .waitFor({ state: "visible" });

    const trigger = page.locator('[data-testid="start-date-trigger"]');
    await expect(trigger).not.toBeVisible();
  });

  // ── Viewport meta ────────────────────────────────────────────

  test("viewport meta has viewport-fit=cover", async ({
    authenticatedPage: page,
  }) => {
    await page.goto("/inbox");

    const content = await page.evaluate(() =>
      document
        .querySelector('meta[name="viewport"]')
        ?.getAttribute("content"),
    );
    expect(content).toContain("viewport-fit=cover");
  });
});
