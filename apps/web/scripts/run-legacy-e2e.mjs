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
const repoRoot = path.resolve(workspaceRoot, '../..');

await withServer({
  command: 'node',
  args: ['./apps/web/scripts/serve-live-web-shell-harness.mjs'],
  cwd: repoRoot,
  env: { PORT: '4174' },
  readyUrl: 'http://127.0.0.1:4174/health',
  readyTimeoutMs: 90_000,
  run: async () => {
    const browser = await chromium.launch({
      executablePath: requiredChromiumExecutable(),
      headless: true,
      args: ['--no-sandbox'],
    });
    try {
      const page = await browser.newPage();

      await page.goto('http://127.0.0.1:4174/');
      await page.getByTestId('runtime-root').waitFor({ state: 'visible' });
      assert(
        await page.getByTestId('runtime-root').isVisible(),
        'expected the rust server root route to render the runtime root'
      );
      assert(
        (await page.getByTitle('Paddles Runtime').getAttribute('src')) === '/legacy',
        'expected the root React route to proxy the legacy root runtime'
      );

      await page.goto('http://127.0.0.1:4174/legacy');
      await page.locator('#prompt').fill('CI is failing. Can you debug it on this machine?');
      await page.locator('#send').click();
      await page.waitForFunction(() => {
        const messages = Array.from(document.querySelectorAll('.msg.assistant'));
        return messages.some((node) =>
          (node.textContent || '').includes('Mock provider completed the turn after local inspection.')
        );
      });
      assert(
        (await page.locator('.msg.user').textContent())?.includes(
          'CI is failing. Can you debug it on this machine?'
        ),
        'expected the legacy shell to render the user message'
      );
      assert(
        (await page.locator('.msg.assistant').textContent())?.includes(
          'Mock provider completed the turn after local inspection.'
        ),
        'expected the legacy shell to render the assistant response'
      );

      await page.goto('http://127.0.0.1:4174/legacy/transit');
      await page.waitForFunction(
        () => document.querySelectorAll('#trace-board .trace-node').length > 0
      );
      assert(
        (await page.locator('#trace-transit-meta').textContent())?.includes('significant steps'),
        'expected the transit route to default to significant steps'
      );
      assert(
        (await page.locator('#trace-board .trace-node').count()) > 0,
        'expected the transit route to render trace nodes from the live session'
      );

      await page.locator('button:has-text("Full Trace")').click();
      assert(
        (await page.locator('#trace-transit-meta').textContent())?.includes('full trace'),
        'expected the transit route to expand to the full trace'
      );

      await page.goto('http://127.0.0.1:4174/legacy/manifold');
      assert(
        await page.locator('#manifold-canvas').isVisible(),
        'expected the manifold canvas to be visible'
      );
      await page.waitForFunction(() => {
        const stageMeta = document.querySelector('#manifold-stage-meta');
        return stageMeta && !(stageMeta.textContent || '').includes('Awaiting replay-backed manifold frames');
      });
      assert(
        (await page.locator('#manifold-stage-meta').textContent())?.includes('turns'),
        'expected the manifold route to project live turn state'
      );
      assert(
        (await page.locator('.manifold-node').count()) > 0,
        'expected the manifold route to render steering-signal topology from the live session'
      );
    } finally {
      await browser.close();
    }
  },
});
