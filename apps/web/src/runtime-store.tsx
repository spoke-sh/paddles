import {
  createContext,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
} from 'react';

import type {
  ConversationProjectionSnapshot,
} from './runtime-types';
import {
  reduceRuntimeTurnEvent,
  sanitizePromptHistory,
  type RuntimeEventRow,
} from './store/event-log';
import {
  reduceProjectionSnapshot,
  reduceProjectionUpdate,
} from './store/projection-state';
import { mountProjectionStream as openProjectionStream } from './store/projection-stream';
import { fetchBootstrap, fetchProjection, postTurn } from './store/runtime-client';

interface RuntimeStoreValue {
  connected: boolean;
  error: string | null;
  events: RuntimeEventRow[];
  projection: ConversationProjectionSnapshot | null;
  promptHistory: string[];
  sending: boolean;
  sessionId: string | null;
  sendTurn: (prompt: string) => Promise<void>;
}

const RuntimeStoreContext = createContext<RuntimeStoreValue | null>(null);

export function RuntimeStoreProvider({ children }: { children: React.ReactNode }) {
  const [connected, setConnected] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [events, setEvents] = useState<RuntimeEventRow[]>([]);
  const [projection, setProjection] = useState<ConversationProjectionSnapshot | null>(null);
  const [promptHistory, setPromptHistory] = useState<string[]>([]);
  const [sending, setSending] = useState(false);
  const [sessionId, setSessionId] = useState<string | null>(null);
  const projectionSourceRef = useRef<EventSource | null>(null);

  useEffect(() => {
    let closed = false;

    async function refreshProjection(nextSessionId: string) {
      const snapshot = await fetchProjection(nextSessionId);
      if (!closed) {
        setProjection((current) => reduceProjectionSnapshot(current, snapshot));
      }
    }

    function connectProjectionStream(nextSessionId: string) {
      const projectionSource = openProjectionStream(nextSessionId, {
        onConnected: () => {
          setConnected(true);
          void refreshProjection(nextSessionId);
        },
        onDisconnected: () => {
          setConnected(false);
        },
        onProjection: (update) => {
          setProjection((current) => reduceProjectionUpdate(current, update));
        },
        onTurnEvent: (payload) => {
          setEvents((current) => reduceRuntimeTurnEvent(current, payload));
        },
      });
      projectionSourceRef.current = projectionSource;
    }

    async function bootstrap() {
      try {
        setError(null);
        const bootstrap = await fetchBootstrap();
        if (closed) {
          return;
        }
        setSessionId(bootstrap.session_id);
        setProjection(bootstrap.projection);
        setPromptHistory(sanitizePromptHistory(bootstrap.prompt_history));
        setConnected(true);
        connectProjectionStream(bootstrap.session_id);
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

    setPromptHistory((current) => [...current, text]);
    setSending(true);
    setError(null);
    try {
      await postTurn(sessionId, text);
      const nextProjection = await fetchProjection(sessionId);
      setProjection((current) => reduceProjectionSnapshot(current, nextProjection));
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
      promptHistory,
      sending,
      sessionId,
      sendTurn,
    }),
    [connected, error, events, projection, promptHistory, sending, sessionId]
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
