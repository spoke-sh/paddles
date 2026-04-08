import type {
  ConversationBootstrapResponse,
  ConversationProjectionSnapshot,
} from '../runtime-types';

export function runtimeUrl(path: string) {
  return new URL(path, window.location.origin).toString();
}

async function fetchJson<T>(input: RequestInfo, init?: RequestInit) {
  const response = await fetch(input, init);
  if (!response.ok) {
    throw new Error(`Request failed with status ${response.status}`);
  }
  return (await response.json()) as T;
}

export function fetchBootstrap() {
  return fetchJson<ConversationBootstrapResponse>(
    runtimeUrl('/session/shared/bootstrap')
  );
}

export function fetchProjection(sessionId: string) {
  return fetchJson<ConversationProjectionSnapshot>(
    runtimeUrl(`/sessions/${sessionId}/projection`)
  );
}

export function postTurn(sessionId: string, prompt: string) {
  return fetchJson(runtimeUrl(`/sessions/${sessionId}/turns`), {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ prompt }),
  });
}
