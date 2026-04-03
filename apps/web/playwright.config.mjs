import { execFileSync } from 'node:child_process';
import process from 'node:process';

import { defineConfig } from '@playwright/test';

function chromiumExecutablePath() {
  if (process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH) {
    return process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH;
  }

  try {
    return execFileSync('which', ['chromium'], { encoding: 'utf8' }).trim();
  } catch {
    return undefined;
  }
}

export default defineConfig({
  testDir: './e2e/product',
  fullyParallel: false,
  workers: 1,
  timeout: 60_000,
  use: {
    baseURL: 'http://127.0.0.1:4174',
    headless: true,
    launchOptions: {
      executablePath: chromiumExecutablePath(),
      args: ['--no-sandbox'],
    },
  },
  webServer: {
    command: 'PORT=4174 node ./scripts/serve-live-web-shell-harness.mjs',
    url: 'http://127.0.0.1:4174/health',
    reuseExistingServer: !process.env.CI,
    timeout: 120_000,
  },
});
