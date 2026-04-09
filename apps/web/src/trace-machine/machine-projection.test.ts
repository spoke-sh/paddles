import { describe, expect, it } from 'vitest';

import { bootstrapProjection } from '../test-support/runtime-harness';
import { projectConversationMachine } from './machine-projection';

describe('machine-projection', () => {
  it('projects stable machine moments from the shared bootstrap snapshot', () => {
    const projection = projectConversationMachine(bootstrapProjection);
    const turn = projection.turns[0];

    expect(turn.turnId).toBe('task-123.turn-0001');
    expect(turn.moments.map((moment) => moment.momentId)).toEqual([
      'task-123.turn-0001.moment-0001',
      'task-123.turn-0001.moment-0002',
    ]);
    expect(turn.moments.map((moment) => moment.kind)).toEqual(['input', 'force']);
    expect(turn.moments[0].raw.forensicRecordIds).toEqual(['record-1']);
    expect(turn.moments[0].raw.traceNodeIds).toEqual(['record-1']);
    expect(turn.moments[1].raw.forensicRecordIds).toEqual(['record-2']);
    expect(turn.moments[1].raw.traceNodeIds).toEqual(['record-2']);
  });

  it('distinguishes diverters tool runs and outputs without dropping raw ids', () => {
    const projection = projectConversationMachine({
      ...bootstrapProjection,
      forensics: {
        ...bootstrapProjection.forensics,
        turns: [
          {
            turn_id: 'task-123.turn-0007',
            lifecycle: 'final',
            records: [
              {
                lifecycle: 'final',
                superseded_by_record_id: null,
                record: {
                  record_id: 'branch-1',
                  sequence: 1,
                  lineage: {
                    task_id: 'task-123',
                    turn_id: 'task-123.turn-0007',
                    branch_id: null,
                    parent_record_id: null,
                  },
                  kind: {
                    PlannerBranchDeclared: {
                      branch_id: 'branch-main',
                      rationale: 'Split into a child path.',
                    },
                  },
                },
              },
              {
                lifecycle: 'final',
                superseded_by_record_id: null,
                record: {
                  record_id: 'tool-1',
                  sequence: 2,
                  lineage: {
                    task_id: 'task-123',
                    turn_id: 'task-123.turn-0007',
                    branch_id: 'branch-main',
                    parent_record_id: 'branch-1',
                  },
                  kind: {
                    ToolCallCompleted: {
                      tool_name: 'inspect',
                      status: 'ok',
                    },
                  },
                },
              },
              {
                lifecycle: 'final',
                superseded_by_record_id: null,
                record: {
                  record_id: 'out-1',
                  sequence: 3,
                  lineage: {
                    task_id: 'task-123',
                    turn_id: 'task-123.turn-0007',
                    branch_id: 'branch-main',
                    parent_record_id: 'tool-1',
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
          {
            id: 'branch-1',
            kind: 'branch',
            label: 'branch',
            branch_id: null,
            sequence: 1,
          },
          {
            id: 'tool-1',
            kind: 'tool_done',
            label: 'inspect done',
            branch_id: 'branch-main',
            sequence: 2,
          },
          {
            id: 'out-1',
            kind: 'checkpoint',
            label: 'completed',
            branch_id: 'branch-main',
            sequence: 3,
          },
        ],
      },
    });

    expect(projection.turns[0].moments.map((moment) => moment.kind)).toEqual([
      'diverter',
      'tool_run',
      'output',
    ]);
    expect(projection.turns[0].moments[0].raw.traceNodeIds).toEqual(['branch-1']);
    expect(projection.turns[0].moments[1].raw.forensicRecordIds).toEqual(['tool-1']);
    expect(projection.turns[0].moments[2].raw.traceNodeIds).toEqual(['out-1']);
  });

  it('maps fallback-style steering snapshots into jam moments while leaving action bias as force', () => {
    const projection = projectConversationMachine({
      ...bootstrapProjection,
      forensics: {
        ...bootstrapProjection.forensics,
        turns: [
          {
            turn_id: 'task-123.turn-0011',
            lifecycle: 'final',
            records: [
              {
                lifecycle: 'final',
                superseded_by_record_id: null,
                record: {
                  record_id: 'jam-1',
                  sequence: 1,
                  lineage: {
                    task_id: 'task-123',
                    turn_id: 'task-123.turn-0011',
                    branch_id: null,
                    parent_record_id: null,
                  },
                  kind: {
                    SignalSnapshot: {
                      kind: 'fallback',
                      level: 'high',
                      magnitude_percent: 86,
                      summary: 'fallback fired',
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
                  record_id: 'force-1',
                  sequence: 2,
                  lineage: {
                    task_id: 'task-123',
                    turn_id: 'task-123.turn-0011',
                    branch_id: null,
                    parent_record_id: 'jam-1',
                  },
                  kind: {
                    SignalSnapshot: {
                      kind: 'action_bias',
                      level: 'medium',
                      magnitude_percent: 61,
                      summary: 'bias fired',
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
            ],
          },
        ],
      },
      trace_graph: {
        ...bootstrapProjection.trace_graph,
        nodes: [
          {
            id: 'jam-1',
            kind: 'signal',
            label: 'fallback high',
            branch_id: null,
            sequence: 1,
          },
          {
            id: 'force-1',
            kind: 'signal',
            label: 'action bias medium',
            branch_id: null,
            sequence: 2,
          },
        ],
      },
    });

    expect(projection.turns[0].moments.map((moment) => moment.kind)).toEqual(['jam', 'force']);
  });
});
