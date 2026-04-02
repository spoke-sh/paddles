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

      const header = page.getByRole('heading', { name: 'Turborepo Runtime Web App' });
      assert(await header.isVisible(), 'expected the React app header to be visible');

      const chatRoute = page.getByTestId('route-chat');
      assert(
        (await chatRoute.textContent())?.includes('Conversation Route Shell'),
        'expected the chat route shell to render'
      );

      await page.getByRole('link', { name: 'Transit' }).click();
      assert(page.url().endsWith('/transit'), 'expected the transit route to be active');
      assert(
        (await page.getByTestId('route-transit').textContent())?.includes('Transit Route Shell'),
        'expected the transit route shell to render'
      );

      await page.getByRole('link', { name: 'Manifold' }).click();
      assert(page.url().endsWith('/manifold'), 'expected the manifold route to be active');
      assert(
        (await page.getByTestId('route-manifold').textContent())?.includes('Manifold Route Shell'),
        'expected the manifold route shell to render'
      );
    } finally {
      await browser.close();
    }
  },
});
