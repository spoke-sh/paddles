import { describe, expect, it } from 'vitest';

import { eventRow } from './runtime-helpers';

describe('eventRow', () => {
  it('surfaces gatherer progress detail instead of collapsing to a generic searching label', () => {
    const row = eventRow({
      type: 'gatherer_search_progress',
      phase: 'Indexing',
      strategy: 'bm25',
      eta_seconds: 12,
      detail:
        'indexing 4/10 files · blobs 7 · fresh 2 · skipped 1 · segments 24 · bm25 cache 0 build 1',
    });

    expect(row).toBeTruthy();
    expect(row?.text).toContain('bm25');
    expect(row?.text).toContain('indexing 4/10 files');
    expect(row?.text).toContain('bm25 cache 0 build 1');
    expect(row?.text).toContain('eta 12s');
  });
});
