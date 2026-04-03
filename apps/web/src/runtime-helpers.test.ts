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

  it('uses hunting as the fallback gatherer label when no detail is available', () => {
    const row = eventRow({
      type: 'gatherer_search_progress',
      phase: 'Indexing',
      strategy: '',
      eta_seconds: null,
      detail: '',
    });

    expect(row).toBeTruthy();
    expect(row?.text).toContain('hunting (Indexing)');
  });

  it('surfaces harness governor state with chamber ownership and timeout phase', () => {
    const row = eventRow({
      type: 'harness_state',
      snapshot: {
        chamber: 'gathering',
        governor: {
          status: 'active',
          timeout: {
            phase: 'slow',
            elapsed_seconds: 9,
            deadline_seconds: 30,
          },
          intervention: null,
        },
        detail: 'indexing 4/10 files',
      },
    });

    expect(row).toBeTruthy();
    expect(row?.badge).toBe('gov');
    expect(row?.text).toContain('gathering');
    expect(row?.text).toContain('slow');
    expect(row?.text).toContain('indexing 4/10 files');
  });
});
