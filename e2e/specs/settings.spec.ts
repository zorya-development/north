import { expect } from "@playwright/test";
import { test } from "../fixtures/auth";

test.describe("Settings Page", () => {
  test("displays current settings values", async ({
    authenticatedPage: page,
  }) => {
    await page.goto("/settings");

    const interval = page.locator('[data-testid="settings-review-interval"]');
    await expect(interval).toBeVisible();
    // Default review interval should be pre-filled
    await expect(interval).toHaveValue(/\d+/);

    const timezone = page.locator('[data-testid="settings-timezone"]');
    await expect(timezone).toBeVisible();
  });

  test("saves review interval and persists on reload", async ({
    authenticatedPage: page,
  }) => {
    await page.goto("/settings");

    const interval = page.locator('[data-testid="settings-review-interval"]');
    await interval.fill("14");

    await page.locator('[data-testid="settings-save"]').click();

    // Wait for success toast
    const statusBar = page.locator('[data-testid="status-bar"]');
    await expect(statusBar).toContainText("saved", { ignoreCase: true });

    // Reload and verify persisted
    await page.reload();
    await expect(
      page.locator('[data-testid="settings-review-interval"]'),
    ).toHaveValue("14");
  });

  test("saves timezone and persists on reload", async ({
    authenticatedPage: page,
  }) => {
    await page.goto("/settings");

    const timezone = page.locator('[data-testid="settings-timezone"]');
    await timezone.selectOption("America/New_York");

    await page.locator('[data-testid="settings-save"]').click();

    const statusBar = page.locator('[data-testid="status-bar"]');
    await expect(statusBar).toContainText("saved", { ignoreCase: true });

    // Reload and verify persisted
    await page.reload();
    await expect(
      page.locator('[data-testid="settings-timezone"]'),
    ).toHaveValue("America/New_York");

    // Reset to UTC for other tests
    await page.locator('[data-testid="settings-timezone"]').selectOption("UTC");
    await page.locator('[data-testid="settings-save"]').click();
  });
});
