import { test as base, type Page } from "@playwright/test";

export interface TestUser {
  email: string;
  password: string;
}

export const ADMIN_USER: TestUser = {
  email: "admin@north.local",
  password: "admin",
};

/**
 * Wait for the inbox page to be fully loaded (either empty or with tasks).
 */
export async function waitForInboxLoaded(page: Page): Promise<void> {
  await page
    .locator(
      '[data-testid="empty-task-list"], [data-testid="task-list"]',
    )
    .first()
    .waitFor({ state: "visible" });
}

export async function loginViaUI(
  page: Page,
  user: TestUser = ADMIN_USER,
): Promise<void> {
  await page.goto("/login");
  await page.locator('[data-testid="login-email"]').fill(user.email);
  await page.locator('[data-testid="login-password"]').fill(user.password);
  await page.locator('[data-testid="login-submit"]').click();
  await page.waitForURL("**/inbox");
  await waitForInboxLoaded(page);
}

type AuthFixtures = {
  authenticatedPage: Page;
};

export const test = base.extend<AuthFixtures>({
  authenticatedPage: async ({ page }, use) => {
    await loginViaUI(page);
    await use(page);
  },
});

export { expect } from "@playwright/test";
