import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { RuntimeApp } from './runtime-app';
import type { ConversationProjectionSnapshot } from './runtime-types';

class FakeEventSource {
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

const bootstrapProjection: ConversationProjectionSnapshot = {
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
};

const bootstrapPromptHistory = ['first prompt', 'second prompt'];

beforeEach(() => {
  FakeEventSource.instances = [];
  vi.stubGlobal('EventSource', FakeEventSource);
  vi.stubGlobal(
    'fetch',
    vi.fn(async (input: RequestInfo | URL) => {
      const url = String(input);
      if (url.endsWith('/session/shared/bootstrap')) {
        return new Response(
          JSON.stringify({
            session_id: 'task-123',
            projection: bootstrapProjection,
            prompt_history: bootstrapPromptHistory,
          }),
          { status: 200, headers: { 'content-type': 'application/json' } }
        );
      }
      if (url.endsWith('/sessions/task-123/projection')) {
        return new Response(JSON.stringify(bootstrapProjection), {
          status: 200,
          headers: { 'content-type': 'application/json' },
        });
      }
      if (url.endsWith('/sessions/task-123/turns')) {
        return new Response(JSON.stringify({ response: 'ok' }), {
          status: 200,
          headers: { 'content-type': 'application/json' },
        });
      }
      throw new Error(`unexpected fetch: ${url}`);
    })
  );
});

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

function renderAtPath(pathname: string) {
  window.history.pushState({}, '', pathname);
  return render(<RuntimeApp />);
}

function installScrollMetrics(
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

describe('RuntimeApp', () => {
  it('renders the primary conversation route through the client router without iframe proxies', async () => {
    renderAtPath('/');

    expect(await screen.findByText('Mock provider completed the turn after local inspection.')).toBeInTheDocument();
    expect(screen.getByText('Summary')).toBeInTheDocument();
    expect(document.querySelector('.runtime-shell-host')).toBeInTheDocument();
    expect(screen.getByText('Forensic Inspector')).toBeInTheDocument();
    expect(document.getElementById('forensic-view')).toBeInTheDocument();
    expect(document.querySelectorAll('iframe')).toHaveLength(0);
    expect(FakeEventSource.instances.map((instance) => instance.url)).toEqual([
      'http://localhost:3000/sessions/task-123/projection/events',
    ]);
  });

  it('applies transcript updates and live event rows from the unified projection stream', async () => {
    renderAtPath('/');

    expect(await screen.findByText('Mock provider completed the turn after local inspection.')).toBeInTheDocument();

    const [stream] = FakeEventSource.instances;
    stream.dispatch('turn_event', {
      type: 'tool_called',
      tool_name: 'shell',
      invocation: 'gh run list --limit 10',
    });
    stream.dispatch('projection_update', {
      task_id: 'task-123',
      kind: 'forensic',
      transcript_update: null,
      forensic_update: {
        task_id: 'task-123',
        turn_id: 'task-123.turn-0002',
        record_id: 'record-3',
      },
      snapshot: {
        ...bootstrapProjection,
        transcript: {
          ...bootstrapProjection.transcript,
          entries: [
            ...bootstrapProjection.transcript.entries,
            {
              record_id: 'record-3',
              turn_id: 'task-123.turn-0002',
              speaker: 'assistant',
              content: 'Projection stream delivered the externally injected turn.',
              render: {
                blocks: [
                  {
                    type: 'paragraph',
                    text: 'Projection stream delivered the externally injected turn.',
                  },
                ],
              },
            },
          ],
        },
      },
    });

    expect(await screen.findByText('shell: gh run list --limit 10')).toBeInTheDocument();
    expect(
      await screen.findByText('Projection stream delivered the externally injected turn.')
    ).toBeInTheDocument();
  });

  it('renders applied edit diffs from the live runtime stream', async () => {
    renderAtPath('/');

    expect(await screen.findByText('Mock provider completed the turn after local inspection.')).toBeInTheDocument();

    const [stream] = FakeEventSource.instances;
    stream.dispatch('turn_event', {
      type: 'workspace_edit_applied',
      tool_name: 'apply_patch',
      edit: {
        files: ['sample.rs'],
        diff: [
          '--- a/sample.rs',
          '+++ b/sample.rs',
          '@@ -1 +1 @@',
          '-    println!("hello");',
          '+    println!("hi");',
        ].join('\n'),
        insertions: 1,
        deletions: 1,
      },
    });

    expect(await screen.findByText('apply_patch applied')).toBeInTheDocument();
    expect(await screen.findByText('--- a/sample.rs')).toBeInTheDocument();
    expect(document.querySelector('.diff-line.add')).toHaveTextContent('println!("hi");');
    expect(document.querySelector('.event-badge.tool-diff')).toBeInTheDocument();
  });

  it('accumulates tool output into one live stream row per call and stream', async () => {
    renderAtPath('/');

    expect(await screen.findByText('Mock provider completed the turn after local inspection.')).toBeInTheDocument();

    const [stream] = FakeEventSource.instances;
    stream.dispatch('turn_event', {
      type: 'tool_output',
      call_id: 'tool-1',
      tool_name: 'shell',
      stream: 'stdout',
      output: 'alpha\n',
    });
    stream.dispatch('turn_event', {
      type: 'tool_output',
      call_id: 'tool-1',
      tool_name: 'shell',
      stream: 'stdout',
      output: 'beta',
    });

    expect(await screen.findByText('shell stdout')).toBeInTheDocument();
    expect(await screen.findByText('alpha')).toBeInTheDocument();
    expect(await screen.findByText('beta')).toBeInTheDocument();
    expect(document.querySelectorAll('.event-badge.tool-terminal')).toHaveLength(1);
  });

  it('keeps live stream rows from snapping chat to the bottom after the user scrolls up', async () => {
    renderAtPath('/');

    expect(await screen.findByText('Mock provider completed the turn after local inspection.')).toBeInTheDocument();

    const messages = document.getElementById('messages');
    expect(messages).not.toBeNull();
    const scrollBox = installScrollMetrics(messages as HTMLElement, {
      clientHeight: 200,
      scrollHeight: 1000,
      scrollTop: 120,
    });

    fireEvent.scroll(messages as HTMLElement);
    scrollBox.scrollHeight = 1120;

    const [stream] = FakeEventSource.instances;
    stream.dispatch('turn_event', {
      type: 'tool_called',
      tool_name: 'shell',
      invocation: 'git status --short',
    });

    expect(await screen.findByText('shell: git status --short')).toBeInTheDocument();
    await waitFor(() => expect(scrollBox.scrollTop).toBe(120));
  });

  it('keeps the chat pinned to the tail when the user is already near the bottom', async () => {
    renderAtPath('/');

    expect(await screen.findByText('Mock provider completed the turn after local inspection.')).toBeInTheDocument();

    const messages = document.getElementById('messages');
    expect(messages).not.toBeNull();
    const scrollBox = installScrollMetrics(messages as HTMLElement, {
      clientHeight: 200,
      scrollHeight: 1000,
      scrollTop: 792,
    });

    fireEvent.scroll(messages as HTMLElement);
    scrollBox.scrollHeight = 1120;

    const [stream] = FakeEventSource.instances;
    stream.dispatch('turn_event', {
      type: 'tool_called',
      tool_name: 'shell',
      invocation: 'git diff --stat',
    });

    expect(await screen.findByText('shell: git diff --stat')).toBeInTheDocument();
    await waitFor(() => expect(scrollBox.scrollTop).toBe(1120));
  });

  it('renders assistant transcript blocks instead of flattening them to plain strings', async () => {
    renderAtPath('/');

    expect(await screen.findByText('Summary')).toBeInTheDocument();
    expect(screen.getByText('Mock provider completed the turn after local inspection.')).toBeInTheDocument();
    expect(document.querySelector('.msg-heading')?.textContent).toBe('Summary');
  });

  it('renders assistant response-state badges from transcript metadata', async () => {
    renderAtPath('/');

    expect(await screen.findByText('grounded answer')).toBeInTheDocument();
  });

  it('preserves newlines in user transcript entries', async () => {
    const multilineProjection: ConversationProjectionSnapshot = {
      ...bootstrapProjection,
      transcript: {
        ...bootstrapProjection.transcript,
        entries: [
          {
            record_id: 'record-multiline-user',
            turn_id: 'task-123.turn-0002',
            speaker: 'user',
            content: 'line one\nline two\nline three',
          },
        ],
      },
    };

    vi.stubGlobal(
      'fetch',
      vi.fn(async (input: RequestInfo | URL) => {
        const url = String(input);
        if (url.endsWith('/session/shared/bootstrap')) {
          return new Response(
            JSON.stringify({
              session_id: 'task-123',
              projection: multilineProjection,
              prompt_history: bootstrapPromptHistory,
            }),
            { status: 200, headers: { 'content-type': 'application/json' } }
          );
        }
        if (url.endsWith('/sessions/task-123/projection')) {
          return new Response(JSON.stringify(multilineProjection), {
            status: 200,
            headers: { 'content-type': 'application/json' },
          });
        }
        if (url.endsWith('/sessions/task-123/turns')) {
          return new Response(JSON.stringify({ response: 'ok' }), {
            status: 200,
            headers: { 'content-type': 'application/json' },
          });
        }
        throw new Error(`unexpected fetch: ${url}`);
      })
    );

    renderAtPath('/');

    const message = await screen.findByText(
      (_content, element) =>
        !!element &&
        element.classList.contains('msg-paragraph') &&
        element.textContent === 'line one\nline two\nline three'
    );
    expect(message.closest('.msg')).toHaveClass('msg', 'user');
    expect(message.textContent).toBe('line one\nline two\nline three');
  });

  it('disables native autocomplete and recalls bootstrapped prompt history with arrow keys', async () => {
    renderAtPath('/');

    await screen.findByText('Mock provider completed the turn after local inspection.');

    const input = screen.getByPlaceholderText('Ask paddles...');
    expect(input).toHaveAttribute('autocomplete', 'off');

    fireEvent.change(input, { target: { value: 'draft prompt' } });
    fireEvent.keyDown(input, { key: 'ArrowUp' });
    expect(input).toHaveValue('second prompt');

    fireEvent.keyDown(input, { key: 'ArrowUp' });
    expect(input).toHaveValue('first prompt');

    fireEvent.keyDown(input, { key: 'ArrowDown' });
    expect(input).toHaveValue('second prompt');

    fireEvent.keyDown(input, { key: 'ArrowDown' });
    expect(input).toHaveValue('draft prompt');
  });

  it('compresses multiline paste into a composer chip but submits the raw pasted text', async () => {
    renderAtPath('/');

    await screen.findByText('Mock provider completed the turn after local inspection.');

    const input = screen.getByPlaceholderText('Ask paddles...');
    fireEvent.paste(input, {
      clipboardData: {
        getData: (type: string) => (type === 'text' ? 'alpha\nbeta\ngamma' : ''),
      },
    });

    expect(await screen.findByText('3 lines pasted')).toBeInTheDocument();
    expect(input).toHaveValue('');

    fireEvent.change(input, { target: { value: ' please fix' } });
    fireEvent.submit(input.closest('form') as HTMLFormElement);

    await waitFor(() => {
      const turnCall = vi
        .mocked(fetch)
        .mock.calls.find(([url]) => String(url).endsWith('/sessions/task-123/turns'));
      expect(turnCall).toBeDefined();
      expect(
        JSON.parse(String((turnCall?.[1] as RequestInit | undefined)?.body || '{}'))
      ).toEqual({
        prompt: 'alpha\nbeta\ngamma please fix',
      });
    });
  });

  it('removes a compressed multiline paste chip with backspace when the prompt is empty', async () => {
    renderAtPath('/');

    await screen.findByText('Mock provider completed the turn after local inspection.');

    const input = screen.getByPlaceholderText('Ask paddles...');
    fireEvent.paste(input, {
      clipboardData: {
        getData: (type: string) => (type === 'text' ? 'alpha\nbeta\ngamma' : ''),
      },
    });

    expect(await screen.findByText('3 lines pasted')).toBeInTheDocument();

    fireEvent.keyDown(input, { key: 'Backspace' });

    expect(screen.queryByText('3 lines pasted')).not.toBeInTheDocument();
    expect(input).toHaveValue('');
  });

  it('renders the primary transit route through the client router', async () => {
    renderAtPath('/transit');

    expect(await screen.findByText('Turn Steps')).toBeInTheDocument();
    expect(document.getElementById('trace-board')).toBeInTheDocument();
    expect(document.querySelectorAll('#trace-board .trace-node').length).toBeGreaterThan(0);
    expect(screen.queryByTitle('Paddles Runtime')).not.toBeInTheDocument();
  });

  it('renders the primary manifold route through the client router', async () => {
    renderAtPath('/manifold');

    expect(await screen.findByText('Steering Gate Manifold', { selector: '#trace-subhead' })).toBeInTheDocument();
    expect(document.getElementById('manifold-canvas')).toBeInTheDocument();
    expect(document.querySelectorAll('.manifold-force-point').length).toBeGreaterThan(0);
    expect(screen.getByText('Evidence gate')).toBeInTheDocument();
    expect(screen.getByText('Convergence gate')).toBeInTheDocument();
    expect(screen.getByText('Containment gate')).toBeInTheDocument();
    expect(screen.queryByText('Timeline')).not.toBeInTheDocument();
    expect(screen.queryByText('Gate Sources')).not.toBeInTheDocument();
    expect(screen.queryByTitle('Paddles Runtime')).not.toBeInTheDocument();
  });

  it('renders the temporal gate field instead of the empty-state filler once frames exist', async () => {
    renderAtPath('/manifold');

    const banner = await screen.findByText('Temporal gate playback is active.');
    expect(banner.closest('.manifold-playback-banner')).toBeInTheDocument();
    expect(banner.closest('.manifold-empty-state')).toBeNull();
    expect(await screen.findByText('Temporal gate field')).toBeInTheDocument();
  });

  it('surfaces deterministic resolver outcomes in the manifold readout', async () => {
    const resolverProjection: ConversationProjectionSnapshot = {
      ...bootstrapProjection,
      forensics: {
        ...bootstrapProjection.forensics,
        turns: [
          {
            turn_id: 'task-123.turn-0001',
            lifecycle: 'final',
            records: [
              {
                lifecycle: 'final',
                superseded_by_record_id: null,
                record: {
                  record_id: 'record-resolver',
                  sequence: 3,
                  lineage: {
                    task_id: 'task-123',
                    turn_id: 'task-123.turn-0001',
                    branch_id: null,
                    parent_record_id: 'record-2',
                  },
                  kind: {
                    SignalSnapshot: {
                      kind: 'action_bias',
                      gate: 'convergence',
                      phase: 'narrowing',
                      summary: 'deterministic resolver resolved apps/web/src/runtime-shell.css',
                      level: 'high',
                      magnitude_percent: 79,
                      applies_to: {
                        id: 'planner-step:record-resolver',
                        kind: 'planner_step',
                        label: 'replace runtime shell padding',
                      },
                      contributions: [
                        {
                          source: 'candidate_file_evidence',
                          share_percent: 60,
                          rationale: 'Authored candidates converged on the runtime shell.',
                        },
                      ],
                      artifact: {
                        summary: 'entity resolution',
                        inline_content: JSON.stringify({
                          stage: 'entity-resolution',
                          status: 'resolved',
                          source: 'bootstrap',
                          path: 'apps/web/src/runtime-shell.css',
                          candidates: [
                            'apps/web/src/runtime-shell.css',
                            'apps/web/src/runtime-app.tsx',
                          ],
                          explanation: 'deterministic ranking selected a single authored target',
                        }),
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
        ...bootstrapProjection.manifold,
        turns: [
          {
            turn_id: 'task-123.turn-0001',
            lifecycle: 'final',
            frames: [
              {
                record_id: 'record-resolver',
                sequence: 3,
                lifecycle: 'final',
                anchor: {
                  id: 'planner-step:record-resolver',
                  kind: 'planner_step',
                  label: 'replace runtime shell padding',
                },
                active_signals: [
                  {
                    snapshot_record_id: 'record-resolver',
                    lifecycle: 'final',
                    kind: 'action_bias',
                    gate: 'convergence',
                    phase: 'narrowing',
                    summary: 'deterministic resolver resolved apps/web/src/runtime-shell.css',
                    level: 'high',
                    magnitude_percent: 79,
                    anchor: {
                      id: 'planner-step:record-resolver',
                      kind: 'planner_step',
                      label: 'replace runtime shell padding',
                    },
                    contributions: [
                      {
                        source: 'candidate_file_evidence',
                        share_percent: 60,
                        rationale: 'Authored candidates converged on the runtime shell.',
                      },
                    ],
                    artifact: {
                      summary: 'entity resolution',
                      inline_content: JSON.stringify({
                        stage: 'entity-resolution',
                        status: 'resolved',
                        source: 'bootstrap',
                        path: 'apps/web/src/runtime-shell.css',
                        candidates: [
                          'apps/web/src/runtime-shell.css',
                          'apps/web/src/runtime-app.tsx',
                        ],
                        explanation: 'deterministic ranking selected a single authored target',
                      }),
                      mime_type: 'application/json',
                    },
                  },
                ],
                gates: [
                  {
                    gate: 'convergence',
                    label: 'convergence gate',
                    phase: 'narrowing',
                    level: 'high',
                    magnitude_percent: 79,
                    anchor: {
                      id: 'planner-step:record-resolver',
                      kind: 'planner_step',
                      label: 'replace runtime shell padding',
                    },
                    dominant_signal_kind: 'action_bias',
                    signal_kinds: ['action_bias'],
                    dominant_record_id: 'record-resolver',
                  },
                ],
                primitives: [
                  {
                    primitive_id: 'gate:convergence',
                    kind: 'valve',
                    label: 'Convergence gate',
                    basis: { kind: 'steering_gate', gate: 'convergence' },
                    evidence_record_id: 'record-resolver',
                    anchor: {
                      id: 'planner-step:record-resolver',
                      kind: 'planner_step',
                      label: 'replace runtime shell padding',
                    },
                    level: 'high',
                    magnitude_percent: 79,
                  },
                ],
                conduits: [],
              },
            ],
          },
        ],
      },
    };

    vi.stubGlobal(
      'fetch',
      vi.fn(async (input: RequestInfo | URL) => {
        const url = String(input);
        if (url.endsWith('/session/shared/bootstrap')) {
          return new Response(
            JSON.stringify({
              session_id: 'task-123',
              projection: resolverProjection,
              prompt_history: bootstrapPromptHistory,
            }),
            { status: 200, headers: { 'content-type': 'application/json' } }
          );
        }
        if (url.endsWith('/sessions/task-123/projection')) {
          return new Response(JSON.stringify(resolverProjection), {
            status: 200,
            headers: { 'content-type': 'application/json' },
          });
        }
        if (url.endsWith('/sessions/task-123/turns')) {
          return new Response(JSON.stringify({ response: 'ok' }), {
            status: 200,
            headers: { 'content-type': 'application/json' },
          });
        }
        throw new Error(`unexpected fetch: ${url}`);
      })
    );

    renderAtPath('/manifold');

    expect(await screen.findByText('Resolved target')).toBeInTheDocument();
    expect(await screen.findByText('apps/web/src/runtime-shell.css')).toBeInTheDocument();
    expect(
      await screen.findByText('deterministic ranking selected a single authored target')
    ).toBeInTheDocument();
  });

  it('supports mouse pan tilt and zoom on the manifold camera', async () => {
    const outerWheel = vi.fn();
    window.history.pushState({}, '', '/manifold');
    render(
      <div onWheel={outerWheel}>
        <RuntimeApp />
      </div>
    );

    const viewport = await screen.findByTestId('manifold-spacefield-viewport');
    const deck = await screen.findByTestId('manifold-spacefield-deck');

    expect(deck.getAttribute('data-pan-x')).toBe('0');
    expect(deck.getAttribute('data-pan-y')).toBe('0');
    expect(deck.getAttribute('data-pitch')).toBe('62');
    expect(deck.getAttribute('data-yaw')).toBe('-18');
    expect(deck.getAttribute('data-roll')).toBe('0');
    expect(deck.getAttribute('data-zoom')).toBe('1.00');

    fireEvent.mouseDown(viewport, { button: 0, clientX: 120, clientY: 120 });
    fireEvent.mouseMove(window, { clientX: 260, clientY: 330 });
    fireEvent.mouseUp(window);

    expect(Math.abs(Number(deck.getAttribute('data-pitch')) - 62)).toBeGreaterThan(30);
    expect(deck.getAttribute('data-yaw')).not.toBe('-18');

    fireEvent.mouseDown(viewport, { button: 0, shiftKey: true, clientX: 160, clientY: 90 });
    fireEvent.mouseMove(window, { clientX: 190, clientY: 130 });
    fireEvent.mouseUp(window);

    expect(deck.getAttribute('data-pan-x')).not.toBe('0');
    expect(deck.getAttribute('data-pan-y')).not.toBe('0');

    fireEvent.mouseDown(viewport, { button: 0, altKey: true, clientX: 160, clientY: 90 });
    fireEvent.mouseMove(window, { clientX: 260, clientY: 90 });
    fireEvent.mouseUp(window);

    expect(deck.getAttribute('data-roll')).not.toBe('0');

    const zoomEvent = new WheelEvent('wheel', {
      deltaY: -120,
      bubbles: true,
      cancelable: true,
    });
    fireEvent(viewport, zoomEvent);

    expect(deck.getAttribute('data-zoom')).not.toBe('1.00');
    expect(outerWheel).not.toHaveBeenCalled();
  });
});
