import { expect, test } from '@playwright/test';

test('docs app serves the Keel-style landing page and intro route', async ({
  page,
}) => {
  await page.goto('/');

  await expect(
    page.getByRole('heading', {
      name: 'Make small local models behave like grounded coding agents.',
    }),
  ).toBeVisible();
  await expect(
    page.getByRole('heading', {
      name: 'Paddles changes what a small local model can actually do.',
    }),
  ).toBeVisible();
  await expect(
    page.getByRole('link', { name: 'Read The Story' }),
  ).toBeVisible();

  await page.getByRole('link', { name: 'Read The Story' }).click();
  await expect(page).toHaveURL(/\/docs\/intro$/);
  await expect(page.getByRole('heading', { name: 'Paddles, Explained' })).toBeVisible();

  const fontFamily = await page.locator('body').evaluate((element) => {
    return window.getComputedStyle(element).fontFamily;
  });
  expect(fontFamily).toContain('Geist');
});
