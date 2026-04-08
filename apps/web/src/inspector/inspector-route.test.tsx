import { fireEvent, screen, waitFor, within } from '@testing-library/react';
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
  it('preserves inspector focus, record selection, and detail toggles', async () => {
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
    const forensicNav = document.getElementById('forensic-nav');
    expect(forensicNav).not.toBeNull();

    fireEvent.click(
      within(forensicNav as HTMLElement).getByRole('button', {
        name: /read apps\/web\/src\/runtime-app\.tsx/i,
      })
    );

    expect(
      await screen.findByText(/Current focus:\s*planner_step planner-step:record-2/i)
    ).toBeInTheDocument();

    fireEvent.click(
      within(forensicNav as HTMLElement).getByRole('button', {
        name: /All records/i,
      })
    );

    const recordButton = document.querySelector('[data-record-id="record-3"]');
    expect(recordButton).not.toBeNull();
    fireEvent.click(recordButton as HTMLElement);

    expect(await screen.findByText('record-3')).toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: 'Raw' }));
    await waitFor(() => {
      expect(document.getElementById('forensic-detail')?.textContent).toContain(
        'inspect runtime route'
      );
    });

    const conversationButton = document.getElementById('forensic-conversation-button');
    expect(conversationButton).not.toBeNull();
    fireEvent.click(conversationButton as HTMLElement);
    expect(await screen.findByText('Conversation Summary')).toBeInTheDocument();
  });
});
