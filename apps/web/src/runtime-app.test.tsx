import { cleanup, render, screen } from '@testing-library/react';
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
            primitives: [
              {
                primitive_id: 'family:action_bias',
                kind: 'valve',
                label: 'Action bias valve',
                basis: { kind: 'signal_family', signal_kind: 'action_bias' },
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

  it('renders the primary transit route through the client router', async () => {
    renderAtPath('/transit');

    expect(await screen.findByText('Turn Steps')).toBeInTheDocument();
    expect(document.getElementById('trace-board')).toBeInTheDocument();
    expect(document.querySelectorAll('#trace-board .trace-node').length).toBeGreaterThan(0);
    expect(screen.queryByTitle('Paddles Runtime')).not.toBeInTheDocument();
  });

  it('renders the primary manifold route through the client router', async () => {
    renderAtPath('/manifold');

    expect(await screen.findByText('Steering Signal Manifold', { selector: '#trace-subhead' })).toBeInTheDocument();
    expect(document.getElementById('manifold-canvas')).toBeInTheDocument();
    expect(document.querySelectorAll('.manifold-node').length).toBeGreaterThan(0);
    expect(screen.queryByTitle('Paddles Runtime')).not.toBeInTheDocument();
  });

  it('renders a compact playback banner instead of the empty-state filler once frames exist', async () => {
    renderAtPath('/manifold');

    const banner = await screen.findByText('Temporal manifold playback is active.');
    expect(banner.closest('.manifold-playback-banner')).toBeInTheDocument();
    expect(banner.closest('.manifold-empty-state')).toBeNull();
  });
});
