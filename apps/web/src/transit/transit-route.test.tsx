import { fireEvent, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';

import {
  bootstrapProjection,
  installRuntimeHarness,
  renderAtPath,
  resetRuntimeHarness,
} from '../test-support/runtime-harness';
import type { ConversationProjectionSnapshot } from '../runtime-types';

const transitNarrativeProjection: ConversationProjectionSnapshot = {
  ...bootstrapProjection,
  forensics: {
    ...bootstrapProjection.forensics,
    turns: [
      {
        turn_id: 'task-123.turn-0009',
        lifecycle: 'final',
        records: [
          {
            lifecycle: 'final',
            superseded_by_record_id: null,
            record: {
              record_id: 'turn-1',
              sequence: 1,
              lineage: {
                task_id: 'task-123',
                turn_id: 'task-123.turn-0009',
                branch_id: null,
                parent_record_id: null,
              },
              kind: {
                TaskRootStarted: {
                  prompt: {
                    summary: 'prompt',
                    inline_content: 'Explain the machine.',
                    mime_type: 'text/plain',
                  },
                },
              },
            },
          },
          {
            lifecycle: 'final',
            superseded_by_record_id: null,
            record: {
              record_id: 'branch-1',
              sequence: 2,
              lineage: {
                task_id: 'task-123',
                turn_id: 'task-123.turn-0009',
                branch_id: null,
                parent_record_id: 'turn-1',
              },
              kind: {
                PlannerBranchDeclared: {
                  branch_id: 'branch-alt',
                  label: 'branch to alternate line',
                },
              },
            },
          },
          {
            lifecycle: 'final',
            superseded_by_record_id: null,
            record: {
              record_id: 'jam-1',
              sequence: 3,
              lineage: {
                task_id: 'task-123',
                turn_id: 'task-123.turn-0009',
                branch_id: 'branch-alt',
                parent_record_id: 'branch-1',
              },
              kind: {
                SignalSnapshot: {
                  kind: 'fallback',
                  level: 'high',
                  magnitude_percent: 88,
                  summary: 'The machine jammed and needed a fallback route.',
                  contributions: [],
                  artifact: {
                    summary: 'fallback signal',
                    inline_content: '{"kind":"fallback"}',
                    mime_type: 'application/json',
                  },
                },
              },
            },
          },
          {
            lifecycle: 'final',
            superseded_by_record_id: null,
            record: {
              record_id: 'spring-1',
              sequence: 4,
              lineage: {
                task_id: 'task-123',
                turn_id: 'task-123.turn-0009',
                branch_id: 'branch-alt',
                parent_record_id: 'jam-1',
              },
              kind: {
                ThreadMerged: {
                  from_branch_id: 'branch-alt',
                  into_branch_id: null,
                },
              },
            },
          },
          {
            lifecycle: 'final',
            superseded_by_record_id: null,
            record: {
              record_id: 'force-1',
              sequence: 5,
              lineage: {
                task_id: 'task-123',
                turn_id: 'task-123.turn-0009',
                branch_id: null,
                parent_record_id: 'spring-1',
              },
              kind: {
                SignalSnapshot: {
                  kind: 'action_bias',
                  level: 'medium',
                  magnitude_percent: 61,
                  summary: 'Action bias pulled the machine toward the authored path.',
                  contributions: [],
                  artifact: {
                    summary: 'action bias signal',
                    inline_content: '{"kind":"action_bias"}',
                    mime_type: 'application/json',
                  },
                },
              },
            },
          },
          {
            lifecycle: 'final',
            superseded_by_record_id: null,
            record: {
              record_id: 'output-1',
              sequence: 6,
              lineage: {
                task_id: 'task-123',
                turn_id: 'task-123.turn-0009',
                branch_id: null,
                parent_record_id: 'force-1',
              },
              kind: {
                CompletionCheckpoint: {
                  response: {
                    summary: 'final answer',
                  },
                },
              },
            },
          },
        ],
      },
    ],
  },
  trace_graph: {
    ...bootstrapProjection.trace_graph,
    nodes: [
      { id: 'turn-1', kind: 'root', label: 'prompt', branch_id: null, sequence: 1 },
      {
        id: 'branch-1',
        kind: 'branch',
        label: 'diverter',
        branch_id: null,
        sequence: 2,
      },
      {
        id: 'jam-1',
        kind: 'signal',
        label: 'fallback high',
        branch_id: 'branch-alt',
        sequence: 3,
      },
      {
        id: 'spring-1',
        kind: 'merge',
        label: 'spring return',
        branch_id: 'branch-alt',
        sequence: 4,
      },
      {
        id: 'force-1',
        kind: 'signal',
        label: 'action bias medium',
        branch_id: null,
        sequence: 5,
      },
      {
        id: 'output-1',
        kind: 'checkpoint',
        label: 'completed',
        branch_id: null,
        sequence: 6,
      },
    ],
    edges: [
      { from: 'turn-1', to: 'branch-1' },
      { from: 'branch-1', to: 'jam-1' },
      { from: 'jam-1', to: 'spring-1' },
      { from: 'spring-1', to: 'force-1' },
      { from: 'force-1', to: 'output-1' },
    ],
    branches: [
      {
        id: 'branch-alt',
        label: 'alternate line',
        status: 'active',
        parent_branch_id: null,
      },
    ],
  },
};

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

  it('renders diverters jams replans forces and outputs as distinct machine parts', async () => {
    installRuntimeHarness({ projection: transitNarrativeProjection });
    renderAtPath('/transit');

    await screen.findByText('Turn Machine');
    expect(document.querySelector('.transit-machine__part.is-diverter')).not.toBeNull();
    expect(document.querySelector('.transit-machine__part.is-jam')).not.toBeNull();
    expect(document.querySelector('.transit-machine__part.is-spring_return')).not.toBeNull();
    expect(document.querySelector('.transit-machine__part.is-force')).not.toBeNull();
    expect(document.querySelector('.transit-machine__part.is-output')).not.toBeNull();
  });

  it('keeps selected transit detail focused on the causal machine explanation', async () => {
    installRuntimeHarness({ projection: transitNarrativeProjection });
    renderAtPath('/transit');

    await screen.findByText('Turn Machine');
    const jamButton = document.querySelector(
      '[data-transit-scrub-moment-id="task-123.turn-0009.moment-0003"]'
    ) as HTMLButtonElement | null;
    expect(jamButton).not.toBeNull();

    fireEvent.click(jamButton as HTMLButtonElement);

    await waitFor(() => {
      expect(document.getElementById('transit-machine-detail')?.textContent).toContain('Jam');
    });
    expect(document.getElementById('transit-machine-detail')?.textContent).toContain(
      'Where progress catches and the machine has to explain or recover before continuing.'
    );
    expect(document.getElementById('transit-machine-detail')?.textContent).toContain(
      'The machine jammed and needed a fallback route.'
    );
  });
});
