import { fireEvent, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';

import type { ConversationProjectionSnapshot } from '../runtime-types';
import {
  bootstrapProjection,
  installRuntimeHarness,
  renderAtPath,
  resetRuntimeHarness,
  stubRuntimeFetch,
} from '../test-support/runtime-harness';

beforeEach(() => {
  installRuntimeHarness();
});

afterEach(() => {
  resetRuntimeHarness();
});

describe('InspectorRoute', () => {
  it('collapses forensic selection to turns, moments, and an explicit internals path', async () => {
    const inspectorProjection: ConversationProjectionSnapshot = {
      ...bootstrapProjection,
      forensics: {
        ...bootstrapProjection.forensics,
        turns: [
          {
            turn_id: 'task-123.turn-0001',
            lifecycle: 'final',
            records: [
              bootstrapProjection.forensics.turns[0].records[0],
              {
                lifecycle: 'final',
                superseded_by_record_id: null,
                record: {
                  record_id: 'record-2',
                  sequence: 2,
                  lineage: {
                    task_id: 'task-123',
                    turn_id: 'task-123.turn-0001',
                    branch_id: null,
                    parent_record_id: 'record-1',
                  },
                  kind: {
                    PlannerAction: {
                      action: 'read apps/web/src/runtime-app.tsx',
                    },
                  },
                },
              },
              {
                lifecycle: 'final',
                superseded_by_record_id: null,
                record: {
                  record_id: 'record-3',
                  sequence: 3,
                  lineage: {
                    task_id: 'task-123',
                    turn_id: 'task-123.turn-0001',
                    branch_id: null,
                    parent_record_id: 'record-2',
                  },
                  kind: {
                    ModelExchangeArtifact: {
                      exchange_id: 'exchange-1',
                      lane: 'planner',
                      category: 'request',
                      provider: 'openai',
                      model: 'gpt-5.4',
                      phase: 'completed',
                      artifact: {
                        artifact_id: 'artifact-1',
                        summary: 'Planner request',
                        inline_content: '{"prompt":"inspect runtime route"}',
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
                  record_id: 'record-4',
                  sequence: 4,
                  lineage: {
                    task_id: 'task-123',
                    turn_id: 'task-123.turn-0001',
                    branch_id: null,
                    parent_record_id: 'record-3',
                  },
                  kind: {
                    SignalSnapshot: {
                      kind: 'action_bias',
                      level: 'high',
                      magnitude_percent: 79,
                      summary: 'Action bias stayed fixed on the runtime route.',
                      applies_to: {
                        id: 'planner-step:record-2',
                        kind: 'planner_step',
                        label: 'read runtime route',
                      },
                      contributions: [
                        {
                          source: 'candidate_file_evidence',
                          share_percent: 79,
                          rationale: 'The authored route file became dominant.',
                        },
                      ],
                      artifact: {
                        summary: 'signal snapshot',
                        inline_content: '{"kind":"action_bias"}',
                        mime_type: 'application/json',
                      },
                    },
                  },
                },
              },
            ],
          },
        ],
      },
    };

    stubRuntimeFetch({ projection: inspectorProjection });
    renderAtPath('/');

    await screen.findByText('Forensic Inspector', { selector: '#trace-subhead' });
    expect(document.getElementById('forensic-nav')).toBeNull();
    expect(document.getElementById('forensic-turn-scrubber')).not.toBeNull();
    expect(document.getElementById('forensic-internals-toggle')).not.toBeNull();
    expect(document.getElementById('forensic-machine-summary')).not.toBeNull();
    expect(document.getElementById('forensic-internals-shell')).toBeNull();
    expect(screen.queryByText('All records')).not.toBeInTheDocument();
    expect(document.getElementById('forensic-conversation-button')).toBeNull();

    const momentScrubber = document.querySelector(
      '[data-atlas-scrub-moment-id="task-123.turn-0001.moment-0004"]'
    ) as HTMLButtonElement | null;
    expect(momentScrubber).not.toBeNull();
    fireEvent.click(momentScrubber as HTMLButtonElement);

    await waitFor(() => {
      expect(document.getElementById('forensic-machine-summary')?.textContent).toContain('Force');
    });
    expect(document.getElementById('forensic-machine-summary')?.textContent).toContain('record-4');

    fireEvent.click(
      screen.getByRole('button', {
        name: /show internals/i,
      })
    );
    await waitFor(() => {
      expect(document.getElementById('forensic-internals-shell')).not.toBeNull();
    });
    expect(document.getElementById('forensic-detail')?.textContent).toContain('record-4');
  });

  it('renders a selectable forensic atlas with a bottom scrubber', async () => {
    stubRuntimeFetch();
    renderAtPath('/');

    await screen.findByText('Forensic Inspector', { selector: '#trace-subhead' });
    expect(document.getElementById('forensic-atlas')).not.toBeNull();
    expect(document.getElementById('forensic-atlas-scrubber')).not.toBeNull();

    const atlasPoint = document.querySelector(
      '[data-atlas-moment-id="task-123.turn-0001.moment-0001"]'
    ) as HTMLElement | null;
    expect(atlasPoint).not.toBeNull();
    expect(
      document.querySelector('[data-atlas-scrub-moment-id="task-123.turn-0001.moment-0001"]')
    ).not.toBeNull();

    await waitFor(() => {
      expect(document.getElementById('forensic-atlas-popup')?.textContent).toContain(
        'Action bias strengthened after local evidence.'
      );
      expect(document.getElementById('forensic-atlas-popup')?.textContent).toContain('record-2');
    });
  });
});
