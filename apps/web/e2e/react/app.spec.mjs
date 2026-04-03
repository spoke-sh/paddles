import { expect, test } from '@playwright/test';

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    window.__PADDLES_DISABLE_RUNTIME_BOOTSTRAP__ = true;
  });
});

test('react runtime app serves the primary routes without iframe proxies', async ({ page }) => {
  await page.goto('/');
  await expect(page.locator('#prompt')).toBeVisible();
  await expect(page.locator('#trace-board')).toBeVisible();
  await expect(page.locator('iframe')).toHaveCount(0);

  await page.goto('/transit');
  await expect(page.locator('#prompt')).toBeVisible();
  await expect(page.locator('#trace-board')).toBeVisible();
  await expect(page.locator('iframe')).toHaveCount(0);

  await page.goto('/manifold');
  await expect(page.locator('#manifold-canvas')).toBeVisible();
  await expect(page.locator('iframe')).toHaveCount(0);
});
