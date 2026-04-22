import { describe, expect, it } from 'vitest';

import { bootstrapProjection } from '../test-support/runtime-harness';
import type { ConversationProjectionUpdate } from '../runtime-types';
import {
  projectionVersion,
  reduceProjectionSnapshot,
  reduceProjectionUpdate,
} from './projection-state';

describe('projection-state', () => {
  it('derives the latest version from the projection trace graph', () => {
    expect(projectionVersion(bootstrapProjection)).toBe(
      Math.max(...bootstrapProjection.trace_graph.nodes.map((node) => node.sequence))
    );
  });

  it('ignores stale updates and replaces state with newer authoritative snapshots', () => {
    const stale: ConversationProjectionUpdate = {
      task_id: bootstrapProjection.task_id,
      kind: 'forensic',
      reducer: 'replace_snapshot',
      version: projectionVersion(bootstrapProjection) - 1,
      transcript_update: null,
      forensic_update: null,
      snapshot: {
        ...bootstrapProjection,
        transcript: {
          ...bootstrapProjection.transcript,
          entries: [
            ...bootstrapProjection.transcript.entries,
            {
              record_id: 'record-stale',
              turn_id: 'task-123.turn-0002',
              speaker: 'assistant',
              content: 'stale snapshot',
            },
          ],
        },
      },
    };
    const fresh: ConversationProjectionUpdate = {
      ...stale,
      version: projectionVersion(bootstrapProjection) + 1,
      snapshot: {
        ...stale.snapshot,
        transcript: {
          ...stale.snapshot.transcript,
          entries: [
            ...bootstrapProjection.transcript.entries,
            {
              record_id: 'record-fresh',
              turn_id: 'task-123.turn-0002',
              speaker: 'assistant',
              content: 'fresh snapshot',
            },
          ],
        },
      },
    };

    expect(reduceProjectionUpdate(bootstrapProjection, stale)).toBe(bootstrapProjection);
    expect(reduceProjectionUpdate(bootstrapProjection, fresh)).toEqual(fresh.snapshot);
  });

  it('keeps the newest authoritative snapshot when refetches resolve out of order', () => {
    const staleSnapshot = {
      ...bootstrapProjection,
      trace_graph: {
        ...bootstrapProjection.trace_graph,
        nodes: bootstrapProjection.trace_graph.nodes.map((node) => ({
          ...node,
          sequence: Math.max(1, node.sequence - 1),
        })),
      },
    };
    const freshSnapshot = {
      ...bootstrapProjection,
      trace_graph: {
        ...bootstrapProjection.trace_graph,
        nodes: bootstrapProjection.trace_graph.nodes.map((node) => ({
          ...node,
          sequence: node.sequence + 1,
        })),
      },
    };

    expect(reduceProjectionSnapshot(bootstrapProjection, staleSnapshot)).toBe(
      bootstrapProjection
    );
    expect(reduceProjectionSnapshot(bootstrapProjection, freshSnapshot)).toEqual(freshSnapshot);
  });
});
