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
  command: 'npm',
  args: ['run', 'dev', '--', '--host', '127.0.0.1', '--port', '4173'],
  cwd: workspaceRoot,
  readyUrl: 'http://127.0.0.1:4173',
  run: async () => {
    const browser = await chromium.launch({
      executablePath: requiredChromiumExecutable(),
      headless: true,
      args: ['--no-sandbox'],
    });
    try {
      const page = await browser.newPage();
      await page.addInitScript(() => {
        window.__PADDLES_DISABLE_RUNTIME_BOOTSTRAP__ = true;
      });
      await page.goto('http://127.0.0.1:4173/');
      await page.locator('[data-testid="runtime-root"]').waitFor({ state: 'visible' });

      assert(
        await page.locator('[data-testid="runtime-root"]').isVisible(),
        'expected the primary conversation route to mount the runtime shell'
      );
      assert((await page.locator('iframe').count()) === 0, 'expected the React runtime to avoid iframe proxies');

      await page.goto('http://127.0.0.1:4173/transit');
      await page.locator('[data-testid="runtime-root"]').waitFor({ state: 'visible' });
      assert(
        await page.locator('[data-testid="runtime-root"]').isVisible(),
        'expected the transit route to render through the client router'
      );

      await page.goto('http://127.0.0.1:4173/manifold');
      await page.locator('[data-testid="runtime-root"]').waitFor({ state: 'visible' });
      assert(
        await page.locator('[data-testid="runtime-root"]').isVisible(),
        'expected the manifold route to render through the client router'
      );
    } finally {
      await browser.close();
    }
  },
});
