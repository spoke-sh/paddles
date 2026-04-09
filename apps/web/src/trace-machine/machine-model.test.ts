import { describe, expect, it } from 'vitest';

import {
  DEFAULT_MACHINE_SELECTION,
  MACHINE_MOMENT_LEXICON,
  machineMomentKinds,
  machineMomentLabel,
} from './machine-model';

describe('machine-model', () => {
  it('defines the shared operator vocabulary for machine moments', () => {
    expect(machineMomentKinds()).toEqual(
      expect.arrayContaining([
        'input',
        'evidence_probe',
        'diverter',
        'jam',
        'spring_return',
        'force',
        'output',
      ])
    );
    expect(machineMomentLabel('diverter')).toBe('Diverter');
    expect(machineMomentLabel('spring_return')).toBe('Spring return');
  });

  it('keeps the default machine copy out of raw storage language', () => {
    const rawStorageTerms = /\b(record|node|payload)\b/i;
    for (const entry of Object.values(MACHINE_MOMENT_LEXICON)) {
      expect(entry.label).not.toMatch(rawStorageTerms);
      expect(entry.narrative).not.toMatch(rawStorageTerms);
    }
  });

  it('defines the shared turn moment and internals selection contract', () => {
    expect(DEFAULT_MACHINE_SELECTION).toEqual({
      selectedTurnId: null,
      selectedMomentId: null,
      showInternals: false,
    });
  });
});
