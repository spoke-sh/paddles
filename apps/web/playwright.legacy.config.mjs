import { defineConfig } from '@playwright/test';

const executablePath = process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH || undefined;

export default defineConfig({
  testDir: './e2e/legacy',
  fullyParallel: false,
  workers: 1,
  timeout: 30_000,
  use: {
    baseURL: 'http://127.0.0.1:4174',
    launchOptions: executablePath
      ? { executablePath, headless: true, args: ['--no-sandbox'] }
      : { headless: true, args: ['--no-sandbox'] },
  },
  webServer: {
    command: 'PORT=4174 node ./scripts/serve-live-web-shell-harness.mjs',
    url: 'http://127.0.0.1:4174/health',
    reuseExistingServer: !process.env.CI,
    timeout: 120_000,
  },
});
