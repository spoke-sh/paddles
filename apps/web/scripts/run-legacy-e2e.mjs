import { chromium } from 'playwright';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

import {
  assert,
  requiredChromiumExecutable,
  withServer,
} from './e2e-helpers.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const workspaceRoot = path.resolve(__dirname, '..');

await withServer({
  command: 'node',
  args: ['./scripts/serve-web-shell-fixture.mjs'],
  cwd: workspaceRoot,
  env: { PORT: '4174' },
  readyUrl: 'http://127.0.0.1:4174/health',
  run: async () => {
    const browser = await chromium.launch({
      executablePath: requiredChromiumExecutable(),
      headless: true,
      args: ['--no-sandbox'],
    });
    try {
      const page = await browser.newPage();

      await page.goto('http://127.0.0.1:4174/');
      await page.locator('#prompt').fill('Browser fixture prompt');
      await page.locator('#send').click();
      assert(
        (await page.locator('.msg.user').textContent())?.includes('Browser fixture prompt'),
        'expected the legacy shell to render the user message'
      );
      assert(
        (await page.locator('.msg.assistant').textContent())?.includes(
          'Fixture assistant response for: Browser fixture prompt'
        ),
        'expected the legacy shell to render the assistant response'
      );

      await page.goto('http://127.0.0.1:4174/transit');
      assert(
        (await page.locator('#trace-transit-meta').textContent())?.includes('significant steps'),
        'expected the transit route to default to significant steps'
      );

      await page.locator('button:has-text("Full Trace")').click();
      assert(
        (await page.locator('#trace-transit-meta').textContent())?.includes('full trace'),
        'expected the transit route to expand to the full trace'
      );

      await page.goto('http://127.0.0.1:4174/manifold');
      assert(
        await page.locator('#manifold-canvas').isVisible(),
        'expected the manifold canvas to be visible'
      );
    } finally {
      await browser.close();
    }
  },
});
