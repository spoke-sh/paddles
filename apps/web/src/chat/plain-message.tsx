import { renderInlineCode } from './inline-code';

export function PlainMessage({ content }: { content: string }) {
  return (
    <div className="msg-body">
      <div className="msg-paragraph">{renderInlineCode(content, 'plain')}</div>
    </div>
  );
}
