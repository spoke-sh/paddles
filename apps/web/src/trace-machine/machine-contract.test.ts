import { describe, expect, it } from 'vitest';

import { bootstrapProjection } from '../test-support/runtime-harness';
import {
  MACHINE_NARRATIVE_KINDS,
  MACHINE_SELECTION_FIELDS,
  machineMomentLabel,
} from './machine-model';
import { projectConversationMachine } from './machine-projection';

describe('machine-contract', () => {
  it('pins the shared operator vocabulary and selection fields', () => {
    expect(MACHINE_NARRATIVE_KINDS).toEqual([
      'input',
      'planner',
      'evidence_probe',
      'diverter',
      'jam',
      'spring_return',
      'tool_run',
      'force',
      'output',
      'unknown',
    ]);
    expect(MACHINE_SELECTION_FIELDS).toEqual([
      'selectedTurnId',
      'selectedMomentId',
      'showInternals',
    ]);
  });

  it('projects machine moments that stay aligned with the shared labels', () => {
    const projection = projectConversationMachine(bootstrapProjection);

    for (const turn of projection.turns) {
      for (const moment of turn.moments) {
        expect(moment.label).toBe(machineMomentLabel(moment.kind));
        expect(moment.raw.forensicRecordIds.length + moment.raw.traceNodeIds.length).toBeGreaterThan(0);
      }
    }
  });
});
