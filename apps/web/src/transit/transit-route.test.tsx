import { fireEvent, screen, waitFor } from '@testing-library/react';
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

  it('renders a selectable observatory with a step scrubber', async () => {
    renderAtPath('/transit');

    await screen.findByText('Turn Steps');
    expect(document.getElementById('trace-observatory')).not.toBeNull();
    expect(document.getElementById('trace-step-scrubber')).not.toBeNull();

    const firstNode = document.querySelector('[data-trace-node-id="record-1"]') as HTMLElement | null;
    expect(firstNode).not.toBeNull();
    fireEvent.click(firstNode as HTMLElement);

    await waitFor(() => {
      expect(document.getElementById('trace-observatory-popup')?.textContent).toContain('prompt');
    });

    const scrubberButton = document.querySelector(
      '[data-trace-scrub-node-id="record-2"]'
    ) as HTMLElement | null;
    expect(scrubberButton).not.toBeNull();
    fireEvent.click(scrubberButton as HTMLElement);

    await waitFor(() => {
      expect(document.getElementById('trace-observatory-popup')?.textContent).toContain(
        'action bias medium'
      );
    });
  });
});
