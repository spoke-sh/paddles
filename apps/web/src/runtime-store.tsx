import {
  createContext,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
} from 'react';

import { eventRow } from './runtime-helpers';
import type {
  ConversationBootstrapResponse,
  ConversationProjectionSnapshot,
  ConversationProjectionUpdate,
  ProjectionTurnEvent,
  TurnEvent,
} from './runtime-types';

interface RuntimeEventRow {
  id: string;
  badge: string;
  badgeClass: string;
  text: string;
  diff?: string;
}

interface RuntimeStoreValue {
  connected: boolean;
  error: string | null;
  events: RuntimeEventRow[];
  projection: ConversationProjectionSnapshot | null;
  sending: boolean;
  sessionId: string | null;
  sendTurn: (prompt: string) => Promise<void>;
}

const RuntimeStoreContext = createContext<RuntimeStoreValue | null>(null);

function runtimeUrl(path: string) {
  return new URL(path, window.location.origin).toString();
}

async function fetchJson<T>(input: RequestInfo, init?: RequestInit) {
  const response = await fetch(input, init);
  if (!response.ok) {
    throw new Error(`Request failed with status ${response.status}`);
  }
  return (await response.json()) as T;
}

export function RuntimeStoreProvider({ children }: { children: React.ReactNode }) {
  const [connected, setConnected] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [events, setEvents] = useState<RuntimeEventRow[]>([]);
  const [projection, setProjection] = useState<ConversationProjectionSnapshot | null>(null);
  const [sending, setSending] = useState(false);
  const [sessionId, setSessionId] = useState<string | null>(null);
  const projectionSourceRef = useRef<EventSource | null>(null);

  useEffect(() => {
    let closed = false;

    async function refreshProjection(nextSessionId: string) {
      const snapshot = await fetchJson<ConversationProjectionSnapshot>(
        runtimeUrl(`/sessions/${nextSessionId}/projection`)
      );
      if (!closed) {
        setProjection(snapshot);
      }
    }

    function mountProjectionStream(nextSessionId: string) {
      const projectionSource = new EventSource(
        runtimeUrl(`/sessions/${nextSessionId}/projection/events`)
      );
      projectionSource.addEventListener('projection_update', (message) => {
        const update = JSON.parse(
          (message as MessageEvent<string>).data
        ) as ConversationProjectionUpdate;
        setProjection(update.snapshot);
      });
      projectionSource.addEventListener('turn_event', (message) => {
        const payload = JSON.parse((message as MessageEvent<string>).data) as
          | ProjectionTurnEvent
          | TurnEvent;
        const row = eventRow(payload);
        if (!row) {
          return;
        }
        setEvents((current) =>
          [...current, { id: `${Date.now()}-${current.length}`, ...row }].slice(-64)
        );
      });
      projectionSource.onerror = () => {
        setConnected(false);
      };
      projectionSource.onopen = () => {
        setConnected(true);
        void refreshProjection(nextSessionId);
      };
      projectionSourceRef.current = projectionSource;
    }

    async function bootstrap() {
      try {
        setError(null);
        const bootstrap = await fetchJson<ConversationBootstrapResponse>(
          runtimeUrl('/session/shared/bootstrap')
        );
        if (closed) {
          return;
        }
        setSessionId(bootstrap.session_id);
        setProjection(bootstrap.projection);
        setConnected(true);
        mountProjectionStream(bootstrap.session_id);
      } catch (bootstrapError) {
        if (closed) {
          return;
        }
        const message =
          bootstrapError instanceof Error
            ? bootstrapError.message
            : 'Failed to bootstrap shared conversation projection.';
        setError(message);
        setConnected(false);
      }
    }

    void bootstrap();

    return () => {
      closed = true;
      projectionSourceRef.current?.close();
      projectionSourceRef.current = null;
    };
  }, []);

  async function sendTurn(prompt: string) {
    const text = prompt.trim();
    if (!sessionId || !text) {
      return;
    }

    setSending(true);
    setError(null);
    try {
      await fetchJson(runtimeUrl(`/sessions/${sessionId}/turns`), {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ prompt: text }),
      });
      const nextProjection = await fetchJson<ConversationProjectionSnapshot>(
        runtimeUrl(`/sessions/${sessionId}/projection`)
      );
      setProjection(nextProjection);
    } catch (sendError) {
      setError(
        sendError instanceof Error ? sendError.message : 'Failed to submit conversation turn.'
      );
    } finally {
      setSending(false);
    }
  }

  const value = useMemo<RuntimeStoreValue>(
    () => ({
      connected,
      error,
      events,
      projection,
      sending,
      sessionId,
      sendTurn,
    }),
    [connected, error, events, projection, sending, sessionId]
  );

  return (
    <RuntimeStoreContext.Provider value={value}>
      {children}
    </RuntimeStoreContext.Provider>
  );
}

export function useRuntimeStore() {
  const value = useContext(RuntimeStoreContext);
  if (!value) {
    throw new Error('useRuntimeStore must be used inside RuntimeStoreProvider');
  }
  return value;
}
