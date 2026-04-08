import { screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';

import {
  installRuntimeHarness,
  renderAtPath,
  resetRuntimeHarness,
} from '../test-support/runtime-harness';

beforeEach(() => {
  installRuntimeHarness();
});

afterEach(() => {
  resetRuntimeHarness();
});

describe('TransitRoute', () => {
  it('renders the primary transit route through the client router', async () => {
    renderAtPath('/transit');

    expect(await screen.findByText('Turn Steps')).toBeInTheDocument();
    expect(document.getElementById('trace-board')).toBeInTheDocument();
    expect(document.querySelectorAll('#trace-board .trace-node').length).toBeGreaterThan(0);
    expect(screen.queryByTitle('Paddles Runtime')).not.toBeInTheDocument();
  });
});
