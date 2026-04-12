import type { RefObject, UIEvent } from 'react';

import type { ConversationProjectionSnapshot } from '../runtime-types';
import { AssistantMessage } from './assistant-message';
import { PlainMessage } from './plain-message';

interface RuntimeEventRowLike {
  id: string;
  badge: string;
  badgeClass: string;
  text: string;
  diff?: string;
  output?: string;
}

function responseModeLabel(mode: string | null | undefined) {
  if (!mode) {
    return null;
  }
  return mode.split('_').join(' ');
}

function diffLineClass(line: string) {
  if (
    line.startsWith('+++') ||
    line.startsWith('---') ||
    line.startsWith('diff ') ||
    line.startsWith('index ')
  ) {
    return 'meta';
  }
  if (line.startsWith('+')) {
    return 'add';
  }
  if (line.startsWith('-')) {
    return 'remove';
  }
  if (line.startsWith('@@')) {
    return 'hunk';
  }
  if (line.startsWith('\\')) {
    return 'noop';
  }
  return 'context';
}

function transcriptSpeakerClass(speaker: 'user' | 'assistant' | 'system') {
  if (speaker === 'assistant') {
    return 'assistant';
  }
  if (speaker === 'system') {
    return 'system';
  }
  return 'user';
}

export function TranscriptPane({
  activeView,
  connected,
  error,
  events,
  manifoldTurnIds,
  messagesRef,
  onMessagesScroll,
  onSelectManifoldTurn,
  projection,
  selectedManifoldTurnId,
}: {
  activeView: 'inspector' | 'manifold' | 'transit';
  connected: boolean;
  error: string | null;
  events: RuntimeEventRowLike[];
  manifoldTurnIds: Set<string>;
  messagesRef: RefObject<HTMLDivElement | null>;
  onMessagesScroll: (event: UIEvent<HTMLDivElement>) => void;
  onSelectManifoldTurn: (turnId: string) => void;
  projection: ConversationProjectionSnapshot | null;
  selectedManifoldTurnId: string | null;
}) {
  return (
    <div
      className="chat-messages"
      id="messages"
      onScroll={onMessagesScroll}
      ref={messagesRef}
    >
      {projection?.transcript.entries.map((entry) => {
        const isTurnSelectable =
          activeView === 'manifold' && manifoldTurnIds.has(entry.turn_id);
        const isTurnSelected =
          isTurnSelectable && entry.turn_id === selectedManifoldTurnId;

        return (
          <div
            aria-pressed={isTurnSelectable ? isTurnSelected : undefined}
            className={`msg ${transcriptSpeakerClass(entry.speaker)}${
              isTurnSelectable ? ' is-turn-selectable' : ''
            }${isTurnSelected ? ' is-selected-turn' : ''}`}
            data-message-turn-id={entry.turn_id}
            key={entry.record_id}
            onClick={
              isTurnSelectable ? () => onSelectManifoldTurn(entry.turn_id) : undefined
            }
            onKeyDown={
              isTurnSelectable
                ? (event) => {
                    if (event.key === 'Enter' || event.key === ' ') {
                      event.preventDefault();
                      onSelectManifoldTurn(entry.turn_id);
                    }
                  }
                : undefined
            }
            role={isTurnSelectable ? 'button' : undefined}
            tabIndex={isTurnSelectable ? 0 : undefined}
            title={
              isTurnSelectable
                ? 'Select this turn in the steering gate manifold'
                : undefined
            }
          >
            {entry.speaker === 'assistant' && entry.response_mode ? (
              <div className="msg-meta">
                <span className={`msg-mode-badge is-${entry.response_mode}`}>
                  {responseModeLabel(entry.response_mode)}
                </span>
              </div>
            ) : null}
            {entry.speaker === 'assistant' && entry.render ? (
              <AssistantMessage render={entry.render} />
            ) : (
              <PlainMessage content={entry.content} />
            )}
          </div>
        );
      })}
      {error ? <div className="msg system">Error: {error}</div> : null}
      {!projection && !error ? (
        <div className="msg system">Bootstrapping shared conversation projection...</div>
      ) : null}
      <div className="events-group">
        {events.map((item) => (
          <div className="event-row" data-event-text={item.text} key={item.id}>
            <span className={`event-badge ${item.badgeClass}`}>{item.badge}</span>
            <span>
              <span>{item.text}</span>
              {item.output ? (
                <span className="event-output">
                  {item.output.split('\n').map((line, index) => (
                    <span className="event-output-line" key={`${item.id}-output-${index}`}>
                      {line || '\u00a0'}
                    </span>
                  ))}
                </span>
              ) : null}
              {item.diff ? (
                <span className="diff-lines">
                  {item.diff.split('\n').map((line, index) => (
                    <span
                      className={`diff-line ${diffLineClass(line)}`}
                      key={`${item.id}-${index}`}
                    >
                      {line}
                    </span>
                  ))}
                </span>
              ) : null}
            </span>
          </div>
        ))}
        {!connected && projection ? (
          <div className="event-row" data-event-text="reconnecting live projection stream">
            <span className="event-badge fallback">stream</span>
            <span>Reconnecting live projection stream…</span>
          </div>
        ) : null}
      </div>
    </div>
  );
}
