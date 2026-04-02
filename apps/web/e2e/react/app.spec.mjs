import { expect, test } from '@playwright/test';

test('react runtime app exposes the core route shells', async ({ page }) => {
  await page.goto('/');

  await expect(page.getByRole('heading', { name: 'Turborepo Runtime Web App' })).toBeVisible();
  await expect(page.getByTestId('route-chat')).toContainText('Conversation Route Shell');
  await expect(page.getByTitle('Conversation Route Shell')).toHaveAttribute('src', '/legacy');

  await page.getByRole('link', { name: 'Transit' }).click();
  await expect(page).toHaveURL(/\/transit$/);
  await expect(page.getByTestId('route-transit')).toContainText('Transit Route Shell');
  await expect(page.getByTitle('Transit Route Shell')).toHaveAttribute('src', '/legacy/transit');

  await page.getByRole('link', { name: 'Manifold' }).click();
  await expect(page).toHaveURL(/\/manifold$/);
  await expect(page.getByTestId('route-manifold')).toContainText('Manifold Route Shell');
  await expect(page.getByTitle('Manifold Route Shell')).toHaveAttribute('src', '/legacy/manifold');
});
