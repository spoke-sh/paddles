export function PlainMessage({ content }: { content: string }) {
  return (
    <div className="msg-body">
      <div className="msg-paragraph">{content}</div>
    </div>
  );
}
