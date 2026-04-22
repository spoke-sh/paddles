import type {
  ConversationProjectionUpdate,
  ProjectionTurnEvent,
  TurnEvent,
} from '../runtime-types';
import { runtimeUrl } from './runtime-client';

interface ProjectionStreamHandlers {
  onConnected: () => void;
  onDisconnected: () => void;
  onProjection: (update: ConversationProjectionUpdate) => void;
  onTurnEvent: (payload: ProjectionTurnEvent | TurnEvent) => void;
}

export function mountProjectionStream(
  sessionId: string,
  handlers: ProjectionStreamHandlers
) {
  const projectionSource = new EventSource(
    runtimeUrl(`/sessions/${sessionId}/projection/events`)
  );

  projectionSource.addEventListener('projection_update', (message) => {
    const update = JSON.parse(
      (message as MessageEvent<string>).data
    ) as ConversationProjectionUpdate;
    handlers.onProjection(update);
  });

  projectionSource.addEventListener('turn_event', (message) => {
    const payload = JSON.parse((message as MessageEvent<string>).data) as
      | ProjectionTurnEvent
      | TurnEvent;
    handlers.onTurnEvent(payload);
  });

  projectionSource.onerror = () => {
    handlers.onDisconnected();
  };

  projectionSource.onopen = () => {
    handlers.onConnected();
  };

  return projectionSource;
}
