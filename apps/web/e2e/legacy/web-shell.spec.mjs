import { expect, test } from '@playwright/test';

test('root route serves the react runtime shell from the rust server', async ({ page }) => {
  await page.goto('/');

  await expect(page.getByTestId('runtime-root')).toBeVisible();
  await expect(page.getByTitle('Paddles Runtime')).toHaveAttribute('src', '/legacy');
  await expect(page.getByRole('heading', { name: 'Turborepo Runtime Web App' })).toHaveCount(0);
});

test('legacy chat route submits a prompt and renders the live shared transcript', async ({ page }) => {
  await page.goto('/legacy');
  await page.locator('#prompt').fill('CI is failing. Can you debug it on this machine?');
  await page.locator('#send').click();

  await expect(page.locator('.msg.user')).toContainText(
    'CI is failing. Can you debug it on this machine?'
  );
  await expect(page.locator('.msg.assistant')).toContainText(
    'Mock provider completed the turn after local inspection.'
  );
});

test('legacy transit route renders significant steps and can expand to the full trace', async ({ page }) => {
  await page.goto('/legacy');
  await page.locator('#prompt').fill('CI is failing. Can you debug it on this machine?');
  await page.locator('#send').click();
  await expect(page.locator('.msg.assistant')).toContainText(
    'Mock provider completed the turn after local inspection.'
  );

  await page.goto('/legacy/transit');
  const nodes = page.locator('#trace-board .trace-node');
  await expect(nodes).not.toHaveCount(0);
  await expect(page.locator('#trace-transit-meta')).toContainText('significant steps');

  await page.getByRole('button', { name: 'Full Trace' }).click();
  await expect(page.locator('#trace-transit-meta')).toContainText('full trace');
});

test('transit route adapts detail density as the trace zoom changes', async ({ page }) => {
  await page.goto('/legacy');
  await page.locator('#prompt').fill('CI is failing. Can you debug it on this machine?');
  await page.locator('#send').click();
  await expect(page.locator('.msg.assistant')).toContainText(
    'Mock provider completed the turn after local inspection.'
  );

  await page.goto('/legacy/transit');

  const board = page.locator('#trace-board');
  await expect(board).toHaveAttribute('data-detail-level', 'balanced');

  const box = await board.boundingBox();
  if (!box) {
    throw new Error('trace board did not expose a bounding box');
  }

  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.mouse.wheel(0, 1800);
  await expect(board).toHaveAttribute('data-detail-level', 'overview');

  await page.mouse.wheel(0, -2400);
  await expect(board).toHaveAttribute('data-detail-level', 'focus');
});

test('legacy manifold route boots the dedicated shell', async ({ page }) => {
  await page.goto('/legacy');
  await page.locator('#prompt').fill('CI is failing. Can you debug it on this machine?');
  await page.locator('#send').click();
  await expect(page.locator('.msg.assistant')).toContainText(
    'Mock provider completed the turn after local inspection.'
  );

  await page.goto('/legacy/manifold');
  await expect(page.locator('#manifold-canvas')).toBeVisible();
  await expect(page.locator('#manifold-stage-meta')).not.toContainText(
    'Awaiting replay-backed manifold frames'
  );
  await expect(page.locator('.manifold-node')).not.toHaveCount(0);
});
