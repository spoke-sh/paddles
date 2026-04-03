import { expect, test } from '@playwright/test';

test('react runtime app proxies the legacy runtime routes without outer chrome', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByTestId('runtime-root')).toBeVisible();
  await expect(page.getByTitle('Paddles Runtime')).toHaveAttribute('src', '/legacy');
  await expect(page.getByRole('heading', { name: 'Turborepo Runtime Web App' })).toHaveCount(0);
  await expect(page.getByRole('navigation')).toHaveCount(0);

  await page.goto('/transit');
  await expect(page.getByTitle('Paddles Runtime')).toHaveAttribute('src', '/legacy/transit');

  await page.goto('/manifold');
  await expect(page.getByTitle('Paddles Runtime')).toHaveAttribute('src', '/legacy/manifold');
});
