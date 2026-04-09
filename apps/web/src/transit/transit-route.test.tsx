import { fireEvent, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';

import {
  bootstrapProjection,
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
  it('renders a machine stage and bottom scrubber from shared machine moments', async () => {
    renderAtPath('/transit');

    expect(await screen.findByText('Turn Machine')).toBeInTheDocument();
    expect(document.getElementById('transit-machine-stage')).not.toBeNull();
    expect(document.getElementById('transit-machine-scrubber')).not.toBeNull();
    expect(screen.getAllByText('Input').length).toBeGreaterThan(0);
    expect(screen.getAllByText('Force').length).toBeGreaterThan(0);
    expect(document.getElementById('trace-board')).toBeNull();
    expect(document.getElementById('trace-observatory')).toBeNull();
  });

  it('explains the selected machine moment in causal terms', async () => {
    renderAtPath('/transit');

    await screen.findByText('Turn Machine');
    const scrubberButton = document.querySelector(
      '[data-transit-scrub-moment-id="task-123.turn-0001.moment-0002"]'
    ) as HTMLButtonElement | null;
    expect(scrubberButton).not.toBeNull();

    fireEvent.click(scrubberButton as HTMLButtonElement);

    await waitFor(() => {
      expect(document.getElementById('transit-machine-detail')?.textContent).toContain(
        'Where steering pressure pushes the machine toward evidence, convergence, or containment.'
      );
    });
    expect(document.getElementById('transit-machine-detail')?.textContent).toContain(
      'Action bias strengthened after local evidence.'
    );
    expect(document.getElementById('transit-machine-detail')?.textContent).toContain(
      bootstrapProjection.forensics.turns[0].turn_id
    );
  });
});
