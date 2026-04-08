import { expect, test } from '@playwright/test';

test('product routes serve the runtime directly without iframe proxies', async ({ page }) => {
  await page.goto('/');

  await expect(page.locator('#prompt')).toBeVisible();
  await expect(page.locator('#forensic-view')).toBeVisible();
  await expect(page.locator('iframe')).toHaveCount(0);
});

test('externally injected turns flow live through chat, transit, manifold, and survive reload', async ({
  page,
  request,
}) => {
  const prompt = 'CI is failing. Can you debug it on this machine?';

  await page.goto('/');
  await expect(page.locator('#prompt')).toBeVisible();

  const bootstrapResponse = await request.get('/session/shared/bootstrap');
  expect(bootstrapResponse.ok()).toBeTruthy();
  const bootstrap = await bootstrapResponse.json();
  const sessionId = bootstrap.session_id;

  const turnResponse = await request.post(`/sessions/${sessionId}/turns`, {
    data: { prompt },
  });
  expect(turnResponse.ok()).toBeTruthy();

  await expect(page.locator('.msg.user').last()).toContainText(prompt);
  await expect(page.locator('.msg.assistant').last()).toContainText(
    'Mock provider completed the turn after local inspection.'
  );
  await expect(page.locator('.msg.assistant').last()).toContainText('direct answer');

  await page.getByRole('link', { name: 'Transit' }).click();
  await expect(page.locator('#trace-board .trace-node').first()).toBeVisible();
  await expect(page.locator('#trace-transit-meta')).toContainText('significant steps');

  await page.getByRole('link', { name: 'Manifold' }).click();
  await expect(page.locator('#manifold-canvas')).toBeVisible();
  await expect(page.locator('.manifold-force-point').first()).toBeVisible();
  await expect(page.locator('#manifold-stage-meta')).not.toContainText(
    'Awaiting replay-backed manifold frames'
  );
  await expect(page.getByText('Temporal gate field')).toBeVisible();
  await expect(page.getByText('Timeline')).toHaveCount(0);
  await expect(page.getByText('Gate Sources')).toHaveCount(0);

  await page.reload();
  await expect(page.locator('.manifold-force-point').first()).toBeVisible();

  await page.getByRole('link', { name: 'Inspector' }).click();
  await expect(page.locator('.msg.assistant').last()).toContainText(
    'Mock provider completed the turn after local inspection.'
  );
  await expect(page.locator('.msg.assistant').last()).toContainText('direct answer');
});
