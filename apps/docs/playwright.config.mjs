import { execFileSync } from 'node:child_process';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

import { defineConfig } from '@playwright/test';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const docsPort = 4176;

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
  testDir: './e2e',
  timeout: 30_000,
  use: {
    baseURL: `http://127.0.0.1:${docsPort}`,
    headless: true,
    launchOptions: {
      executablePath: chromiumExecutablePath(),
      args: ['--no-sandbox'],
    },
  },
  webServer: {
    command: `npm run serve -- --host 127.0.0.1 --port ${docsPort}`,
    cwd: __dirname,
    url: `http://127.0.0.1:${docsPort}`,
    reuseExistingServer: !process.env.CI,
    timeout: 120_000,
  },
});
