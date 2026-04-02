import { execFileSync, spawn } from 'node:child_process';
import process from 'node:process';
import { setTimeout as delay } from 'node:timers/promises';

export async function waitForUrl(url, timeoutMs = 30_000) {
  const started = Date.now();
  while (Date.now() - started < timeoutMs) {
    try {
      const response = await fetch(url);
      if (response.ok) {
        return;
      }
    } catch {
      // keep polling until the timeout expires
    }
    await delay(250);
  }
  throw new Error(`Timed out waiting for ${url}`);
}

export async function withServer({
  command,
  args,
  cwd,
  env,
  readyUrl,
  readyTimeoutMs,
  run,
}) {
  const child = spawn(command, args, {
    cwd,
    env: { ...process.env, ...env },
    stdio: 'inherit',
  });

  try {
    await waitForUrl(readyUrl, readyTimeoutMs);
    await run();
  } finally {
    child.kill('SIGTERM');
    await Promise.race([
      new Promise((resolve) => child.once('exit', resolve)),
      delay(5_000).then(() => {
        child.kill('SIGKILL');
      }),
    ]);
  }
}

export function requiredChromiumExecutable() {
  const explicitPath = process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH;
  if (explicitPath) {
    return explicitPath;
  }
  try {
    return execFileSync('which', ['chromium'], { encoding: 'utf8' }).trim();
  } catch {
    throw new Error('PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH is not set and chromium is not on PATH');
  }
}

export function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}
