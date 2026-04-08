import type {
  ChangeEvent,
  ClipboardEvent,
  FormEvent,
  KeyboardEvent,
} from 'react';

import type { ComposerPart } from './use-chat-composer';

export function ChatComposer({
  composerParts,
  onPromptChange,
  onPromptKeyDown,
  onPromptPaste,
  onSubmit,
  prompt,
  sending,
}: {
  composerParts: ComposerPart[];
  onPromptChange: (event: ChangeEvent<HTMLInputElement>) => void;
  onPromptKeyDown: (event: KeyboardEvent<HTMLInputElement>) => void;
  onPromptPaste: (event: ClipboardEvent<HTMLInputElement>) => void;
  onSubmit: (event: FormEvent<HTMLFormElement>) => Promise<void>;
  prompt: string;
  sending: boolean;
}) {
  return (
    <form autoComplete="off" className="chat-input" onSubmit={onSubmit}>
      <div className="chat-composer">
        {composerParts.map((part) =>
          part.kind === 'text' ? (
            <span className="composer-inline-text" key={part.id}>
              {part.text}
            </span>
          ) : (
            <span
              className="composer-paste-chip"
              data-lines={part.lines}
              key={part.id}
              title={part.preview}
            >
              <span className="composer-chip-label">{part.lines} lines pasted</span>
              {part.preview ? (
                <span className="composer-chip-preview">{part.preview}</span>
              ) : null}
            </span>
          )
        )}
        <input
          autoFocus
          autoComplete="off"
          className="chat-composer-field"
          id="prompt"
          onChange={onPromptChange}
          onKeyDown={onPromptKeyDown}
          onPaste={onPromptPaste}
          placeholder={composerParts.length > 0 ? '' : 'Ask paddles...'}
          type="text"
          value={prompt}
        />
      </div>
      <button disabled={sending} id="send" type="submit">
        {sending ? 'Sending…' : 'Send'}
      </button>
    </form>
  );
}
