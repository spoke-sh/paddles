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
      await page.goto('http://127.0.0.1:4173/');

      const runtimeRoot = page.getByTestId('runtime-root');
      await runtimeRoot.waitFor({ state: 'visible' });
      assert(
        await runtimeRoot.isVisible(),
        'expected the runtime root to render'
      );
      assert(
        (await page.getByTitle('Paddles Runtime').getAttribute('src')) === '/legacy',
        'expected the runtime root to proxy the legacy root route'
      );
      assert(
        (await page.getByRole('heading', { name: 'Turborepo Runtime Web App' }).count()) === 0,
        'expected the outer shell heading to be absent'
      );
      assert(
        (await page.getByRole('navigation').count()) === 0,
        'expected the outer shell navigation to be absent'
      );

      await page.goto('http://127.0.0.1:4173/transit');
      assert(
        (await page.getByTitle('Paddles Runtime').getAttribute('src')) === '/legacy/transit',
        'expected the transit route to proxy the legacy transit route'
      );

      await page.goto('http://127.0.0.1:4173/manifold');
      assert(
        (await page.getByTitle('Paddles Runtime').getAttribute('src')) === '/legacy/manifold',
        'expected the manifold route to proxy the legacy manifold route'
      );
    } finally {
      await browser.close();
    }
  },
});
