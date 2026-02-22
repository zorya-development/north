import { test, expect } from "@playwright/test";
import { ADMIN_USER, loginViaUI } from "../fixtures/auth";

test.describe("Login", () => {
  test("renders the login form", async ({ page }) => {
    await page.goto("/login");

    await expect(page.locator('[data-testid="login-email"]')).toBeVisible();
    await expect(page.locator('[data-testid="login-password"]')).toBeVisible();

    const submit = page.locator('[data-testid="login-submit"]');
    await expect(submit).toBeVisible();
    await expect(submit).toHaveText("Sign in");
  });

  test("successful login redirects to inbox", async ({ page }) => {
    await loginViaUI(page);

    await expect(page).toHaveURL(/\/inbox/);
    await expect(
      page.locator('[data-testid="empty-task-list"]'),
    ).toBeVisible();
  });

  test("failed login with wrong email shows error", async ({ page }) => {
    await page.goto("/login");

    await page
      .locator('[data-testid="login-email"]')
      .fill("wrong@example.com");
    await page
      .locator('[data-testid="login-password"]')
      .fill("wrongpassword");
    await page.locator('[data-testid="login-submit"]').click();

    const error = page.locator('[data-testid="login-error"]');
    await expect(error).toBeVisible();
    await expect(error).toContainText("Invalid credentials");
    await expect(page).toHaveURL(/\/login/);
  });

  test("failed login with wrong password shows error", async ({ page }) => {
    await page.goto("/login");

    await page.locator('[data-testid="login-email"]').fill(ADMIN_USER.email);
    await page.locator('[data-testid="login-password"]').fill("wrongpassword");
    await page.locator('[data-testid="login-submit"]').click();

    const error = page.locator('[data-testid="login-error"]');
    await expect(error).toBeVisible();
    await expect(error).toContainText("Invalid credentials");
    await expect(page).toHaveURL(/\/login/);
  });

  test("empty form does not submit", async ({ page }) => {
    await page.goto("/login");

    await page.locator('[data-testid="login-submit"]').click();

    // HTML5 required validation prevents submission — still on login page
    await expect(page).toHaveURL(/\/login/);
  });
});
