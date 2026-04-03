import '@testing-library/jest-dom/vitest';

Object.defineProperty(window, 'scrollTo', {
  writable: true,
  value: () => {},
});

Object.defineProperty(window, '__PADDLES_DISABLE_RUNTIME_BOOTSTRAP__', {
  configurable: true,
  writable: true,
  value: true,
});
