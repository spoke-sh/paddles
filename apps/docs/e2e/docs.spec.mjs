import { expect, test } from '@playwright/test';

test('docs app serves the landing page and intro route', async ({ page }) => {
  await page.goto('/');

  await expect(page.getByRole('heading', { name: 'Paddles' })).toBeVisible();
  await expect(page.getByRole('link', { name: 'Read the Docs' })).toBeVisible();

  await page.getByRole('link', { name: 'Read the Docs' }).click();
  await expect(page).toHaveURL(/\/docs\/intro$/);
  await expect(page.getByRole('heading', { name: 'Paddles, Explained' })).toBeVisible();
});
