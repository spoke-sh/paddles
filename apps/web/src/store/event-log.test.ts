import { describe, expect, it } from 'vitest';

import { appendRuntimeEventRow, sanitizePromptHistory } from './event-log';

describe('event log reduction', () => {
  it('merges tool output rows by stream key', () => {
    const first = appendRuntimeEventRow([], {
      badge: 'term',
      badgeClass: 'tool',
      text: 'inspect stdout',
      output: 'first line\n',
      streamKey: 'tool-stream:inspect-1:stdout',
    });

    const second = appendRuntimeEventRow(first, {
      badge: 'term',
      badgeClass: 'tool',
      text: 'inspect stdout',
      output: 'second line\n',
      streamKey: 'tool-stream:inspect-1:stdout',
    });

    expect(second).toHaveLength(1);
    expect(second[0]).toMatchObject({
      badge: 'term',
      badgeClass: 'tool',
      text: 'inspect stdout',
      output: 'first line\nsecond line\n',
      streamKey: 'tool-stream:inspect-1:stdout',
    });
  });

  it('filters blank prompt history entries during bootstrap', () => {
    expect(
      sanitizePromptHistory(['first prompt', '  ', '', '\n', 'second prompt'])
    ).toEqual(['first prompt', 'second prompt']);
  });
});
