import { cleanup, render } from '@testing-library/react';
import { vi } from 'vitest';

import { RuntimeApp } from '../runtime-app';
import type { ConversationProjectionSnapshot } from '../runtime-types';

export class FakeEventSource {
  static instances: FakeEventSource[] = [];

  listeners = new Map<string, Array<(event: MessageEvent<string>) => void>>();
  addEventListener = vi.fn((type: string, listener: (event: MessageEvent<string>) => void) => {
    const current = this.listeners.get(type) || [];
    current.push(listener);
    this.listeners.set(type, current);
  });
  close = vi.fn();
  onerror: ((this: EventSource, ev: Event) => unknown) | null = null;
  onopen: ((this: EventSource, ev: Event) => unknown) | null = null;
  url: string;

  constructor(url: string | URL) {
    this.url = String(url);
    FakeEventSource.instances.push(this);
  }

  dispatch(type: string, payload: unknown) {
    for (const listener of this.listeners.get(type) || []) {
      listener({ data: JSON.stringify(payload) } as MessageEvent<string>);
    }
  }
}

export const bootstrapProjection: ConversationProjectionSnapshot = {
  task_id: 'task-123',
  transcript: {
    task_id: 'task-123',
    entries: [
      {
        record_id: 'record-1',
        turn_id: 'task-123.turn-0001',
        speaker: 'user',
        content: 'CI is failing. Can you debug it?',
      },
      {
        record_id: 'record-2',
        turn_id: 'task-123.turn-0001',
        speaker: 'assistant',
        content: '**Summary**\n\nMock provider completed the turn after local inspection.',
        response_mode: 'grounded_answer',
        render: {
          blocks: [
            { type: 'heading', text: 'Summary' },
            {
              type: 'paragraph',
              text: 'Mock provider completed the turn after local inspection.',
            },
          ],
        },
      },
    ],
  },
  forensics: {
    task_id: 'task-123',
    turns: [
      {
        turn_id: 'task-123.turn-0001',
        lifecycle: 'final',
        records: [
          {
            lifecycle: 'final',
            superseded_by_record_id: null,
            record: {
              record_id: 'record-1',
              sequence: 1,
              lineage: {
                task_id: 'task-123',
                turn_id: 'task-123.turn-0001',
                branch_id: null,
                parent_record_id: null,
              },
              kind: {
                TaskRootStarted: {
                  prompt: {
                    summary: 'prompt',
                    inline_content: 'CI is failing. Can you debug it?',
                    mime_type: 'text/plain',
                  },
                },
              },
            },
          },
          {
            lifecycle: 'final',
            superseded_by_record_id: null,
            record: {
              record_id: 'record-2',
              sequence: 2,
              lineage: {
                task_id: 'task-123',
                turn_id: 'task-123.turn-0001',
                branch_id: null,
                parent_record_id: 'record-1',
              },
              kind: {
                SignalSnapshot: {
                  kind: 'action_bias',
                  level: 'medium',
                  magnitude_percent: 62,
                  summary: 'Action bias strengthened after local evidence.',
                  contributions: [
                    {
                      source: 'candidate_file_evidence',
                      share_percent: 62,
                      rationale: 'Local evidence tightened the path.',
                    },
                  ],
                  artifact: {
                    summary: 'signal snapshot',
                    inline_content: '{"kind":"action_bias"}',
                    mime_type: 'application/json',
                  },
                },
              },
            },
          },
        ],
      },
    ],
  },
  manifold: {
    task_id: 'task-123',
    turns: [
      {
        turn_id: 'task-123.turn-0001',
        lifecycle: 'final',
        frames: [
          {
            record_id: 'record-2',
            sequence: 2,
            lifecycle: 'final',
            anchor: {
              id: 'planner-step:record-2',
              kind: 'planner_step',
              label: 'inspect git status',
            },
            active_signals: [
              {
                snapshot_record_id: 'record-2',
                lifecycle: 'final',
                kind: 'action_bias',
                gate: 'convergence',
                phase: 'narrowing',
                summary: 'Action bias strengthened after local evidence.',
                level: 'medium',
                magnitude_percent: 62,
                anchor: {
                  id: 'planner-step:record-2',
                  kind: 'planner_step',
                  label: 'inspect git status',
                },
                contributions: [
                  {
                    source: 'candidate_file_evidence',
                    share_percent: 62,
                    rationale: 'Local evidence tightened the path.',
                  },
                ],
                artifact: {
                  summary: 'signal snapshot',
                  inline_content: '{"kind":"action_bias"}',
                  mime_type: 'application/json',
                },
              },
            ],
            gates: [
              {
                gate: 'convergence',
                label: 'convergence gate',
                phase: 'narrowing',
                level: 'medium',
                magnitude_percent: 62,
                anchor: {
                  id: 'planner-step:record-2',
                  kind: 'planner_step',
                  label: 'inspect git status',
                },
                dominant_signal_kind: 'action_bias',
                signal_kinds: ['action_bias'],
                dominant_record_id: 'record-2',
              },
            ],
            primitives: [
              {
                primitive_id: 'gate:convergence',
                kind: 'valve',
                label: 'Convergence gate',
                basis: { kind: 'steering_gate', gate: 'convergence' },
                evidence_record_id: 'record-2',
                anchor: {
                  id: 'planner-step:record-2',
                  kind: 'planner_step',
                  label: 'inspect git status',
                },
                level: 'medium',
                magnitude_percent: 62,
              },
            ],
            conduits: [],
          },
        ],
      },
    ],
  },
  trace_graph: {
    task_id: 'task-123',
    nodes: [
      {
        id: 'record-1',
        kind: 'root',
        label: 'prompt',
        branch_id: null,
        sequence: 1,
      },
      {
        id: 'record-2',
        kind: 'signal',
        label: 'action bias medium',
        branch_id: null,
        sequence: 2,
      },
    ],
    edges: [{ from: 'record-1', to: 'record-2' }],
    branches: [],
  },
  delegation: {
    task_id: 'task-123',
    harness_identity: 'recursive-harness',
    active_worker_count: 0,
    degraded_worker_count: 0,
    workers: [],
  },
};

export const bootstrapPromptHistory = ['first prompt', 'second prompt'];

export function stubRuntimeFetch({
  projection = bootstrapProjection,
  promptHistory = bootstrapPromptHistory,
  turnResponse = { response: 'ok' },
}: {
  projection?: ConversationProjectionSnapshot;
  promptHistory?: string[];
  turnResponse?: unknown;
} = {}) {
  FakeEventSource.instances = [];
  vi.stubGlobal(
    'fetch',
    vi.fn(async (input: RequestInfo | URL) => {
      const url = String(input);
      if (url.endsWith('/session/shared/bootstrap')) {
          return new Response(
            JSON.stringify({
              session_id: 'task-123',
              projection,
              prompt_history: promptHistory,
            }),
            { status: 200, headers: { 'content-type': 'application/json' } }
          );
        }
        if (url.endsWith('/sessions/task-123/projection')) {
          return new Response(JSON.stringify(projection), {
            status: 200,
            headers: { 'content-type': 'application/json' },
          });
        }
        if (url.endsWith('/sessions/task-123/turns')) {
          return new Response(JSON.stringify(turnResponse), {
            status: 200,
            headers: { 'content-type': 'application/json' },
          });
        }
        throw new Error(`unexpected fetch: ${url}`);
      })
  );
}

export function installRuntimeHarness(options?: {
  projection?: ConversationProjectionSnapshot;
  promptHistory?: string[];
  turnResponse?: unknown;
}) {
  FakeEventSource.instances = [];
  vi.stubGlobal('EventSource', FakeEventSource);
  stubRuntimeFetch(options);
}

export function resetRuntimeHarness() {
  cleanup();
  vi.unstubAllGlobals();
}

export function renderAtPath(pathname: string, ui = <RuntimeApp />) {
  window.history.pushState({}, '', pathname);
  return render(ui);
}

export function installScrollMetrics(
  element: HTMLElement,
  {
    clientHeight,
    scrollHeight,
    scrollTop,
  }: { clientHeight: number; scrollHeight: number; scrollTop: number }
) {
  let currentClientHeight = clientHeight;
  let currentScrollHeight = scrollHeight;
  let currentScrollTop = scrollTop;

  Object.defineProperty(element, 'clientHeight', {
    configurable: true,
    get: () => currentClientHeight,
  });
  Object.defineProperty(element, 'scrollHeight', {
    configurable: true,
    get: () => currentScrollHeight,
  });
  Object.defineProperty(element, 'scrollTop', {
    configurable: true,
    get: () => currentScrollTop,
    set: (value: number) => {
      currentScrollTop = Number(value);
    },
  });

  return {
    get scrollTop() {
      return currentScrollTop;
    },
    set scrollHeight(value: number) {
      currentScrollHeight = value;
    },
    set scrollTop(value: number) {
      currentScrollTop = value;
    },
    set clientHeight(value: number) {
      currentClientHeight = value;
    },
  };
}
