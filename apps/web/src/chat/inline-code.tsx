import type { ReactNode } from 'react';

export function renderInlineCode(text: string, keyPrefix: string): ReactNode[] {
  const segments: ReactNode[] = [];
  const matcher = /`([^`]+)`/g;
  let match: RegExpExecArray | null;
  let lastIndex = 0;

  while ((match = matcher.exec(text)) !== null) {
    if (match.index > lastIndex) {
      segments.push(text.slice(lastIndex, match.index));
    }

    segments.push(
      <code className="msg-inline-code" key={`${keyPrefix}-code-${match.index}`}>
        {match[1]}
      </code>
    );

    lastIndex = match.index + match[0].length;
  }

  if (lastIndex < text.length || segments.length === 0) {
    segments.push(text.slice(lastIndex));
  }

  return segments;
}
