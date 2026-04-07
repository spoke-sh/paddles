import { expect } from 'vitest';
import * as matchers from '@testing-library/jest-dom/matchers';

expect.extend(matchers);

Object.defineProperty(window, 'scrollTo', {
  writable: true,
  value: () => {},
});

Object.defineProperty(window, '__PADDLES_DISABLE_RUNTIME_BOOTSTRAP__', {
  configurable: true,
  writable: true,
  value: true,
});
