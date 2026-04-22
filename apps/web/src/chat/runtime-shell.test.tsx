import { fireEvent, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { ConversationProjectionSnapshot } from '../runtime-types';
import { projectionVersion } from '../store/projection-state';
import {
  FakeEventSource,
  bootstrapProjection,
  installRuntimeHarness,
  installScrollMetrics,
  renderAtPath,
  resetRuntimeHarness,
  stubRuntimeFetch,
} from '../test-support/runtime-harness';

beforeEach(() => {
  installRuntimeHarness();
});

afterEach(() => {
  resetRuntimeHarness();
});

describe('Runtime shell and chat', () => {
  it('renders the primary conversation route through the client router without iframe proxies', async () => {
    renderAtPath('/');

    expect(
      await screen.findByText('Mock provider completed the turn after local inspection.')
    ).toBeInTheDocument();
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

    expect(
      await screen.findByText('Mock provider completed the turn after local inspection.')
    ).toBeInTheDocument();

    const [stream] = FakeEventSource.instances;
    stream.dispatch('turn_event', {
      type: 'tool_called',
      tool_name: 'shell',
      invocation: 'gh run list --limit 10',
    });
    stream.dispatch('projection_update', {
      task_id: 'task-123',
      kind: 'forensic',
      reducer: 'replace_snapshot',
      version: projectionVersion(bootstrapProjection) + 1,
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

  it('ignores stale projection snapshots from the live stream', async () => {
    renderAtPath('/');

    expect(
      await screen.findByText('Mock provider completed the turn after local inspection.')
    ).toBeInTheDocument();

    const [stream] = FakeEventSource.instances;
    stream.dispatch('projection_update', {
      task_id: 'task-123',
      kind: 'forensic',
      reducer: 'replace_snapshot',
      version: projectionVersion(bootstrapProjection) - 1,
      transcript_update: null,
      forensic_update: null,
      snapshot: {
        ...bootstrapProjection,
        transcript: {
          ...bootstrapProjection.transcript,
          entries: [
            ...bootstrapProjection.transcript.entries,
            {
              record_id: 'record-stale',
              turn_id: 'task-123.turn-0002',
              speaker: 'assistant',
              content: 'Stale projection should not replace the current transcript.',
            },
          ],
        },
      },
    });

    await waitFor(() => {
      expect(
        screen.queryByText('Stale projection should not replace the current transcript.')
      ).not.toBeInTheDocument();
    });
  });

  it('renders applied edit diffs from the live runtime stream', async () => {
    renderAtPath('/');

    expect(
      await screen.findByText('Mock provider completed the turn after local inspection.')
    ).toBeInTheDocument();

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

    expect(
      await screen.findByText('Mock provider completed the turn after local inspection.')
    ).toBeInTheDocument();

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

    expect(
      await screen.findByText('Mock provider completed the turn after local inspection.')
    ).toBeInTheDocument();

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

    expect(
      await screen.findByText('Mock provider completed the turn after local inspection.')
    ).toBeInTheDocument();

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
    expect(
      screen.getByText('Mock provider completed the turn after local inspection.')
    ).toBeInTheDocument();
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

    stubRuntimeFetch({ projection: multilineProjection });
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

  it('renders system transcript entries as system notices instead of user bubbles', async () => {
    const projectionWithSystemEntry: ConversationProjectionSnapshot = {
      ...bootstrapProjection,
      transcript: {
        ...bootstrapProjection.transcript,
        entries: [
          ...bootstrapProjection.transcript.entries,
          {
            record_id: 'record-system',
            turn_id: 'task-123.turn-0001',
            speaker: 'system',
            content:
              'execution posture recursive-structured-v1 (sandbox=workspace_write, approval=on_request)',
          },
        ],
      },
    };

    stubRuntimeFetch({ projection: projectionWithSystemEntry });
    renderAtPath('/');

    const systemMessage = await screen.findByText(
      'execution posture recursive-structured-v1 (sandbox=workspace_write, approval=on_request)'
    );
    expect(systemMessage.closest('.msg')).toHaveClass('msg', 'system');
    expect(systemMessage.closest('.msg')).not.toHaveClass('user');
  });

  it('renders shared delegation cards from the projection snapshot', async () => {
    const projectionWithDelegation: ConversationProjectionSnapshot = {
      ...bootstrapProjection,
      delegation: {
        task_id: 'task-123',
        harness_identity: 'recursive-harness',
        active_worker_count: 1,
        degraded_worker_count: 1,
        workers: [
          {
            worker_id: 'worker-1',
            role_label: 'Worker',
            ownership_summary: 'Own src/domain/model/delegation.rs',
            read_scopes: ['src/domain/model'],
            write_scopes: ['src/domain/model/delegation.rs'],
            parent_thread: 'mainline',
            worker_thread: 'worker-thread-1',
            status: 'awaiting_integration',
            progress_summary: 'Completion ready with 2 artifacts visible to the parent.',
            latest_detail: 'Delegated review complete.',
            artifact_count: 2,
            completion_recorded: true,
            integration_status: null,
            degraded: false,
          },
          {
            worker_id: 'worker-2',
            role_label: 'Worker',
            ownership_summary: 'Own src/domain/model/delegation.rs',
            read_scopes: ['src/domain/model'],
            write_scopes: ['src/domain/model/delegation.rs'],
            parent_thread: 'mainline',
            worker_thread: 'worker-thread-2',
            status: 'conflict',
            progress_summary: 'Ownership conflict: src/domain/model/delegation.rs',
            latest_detail: 'Ownership conflict: src/domain/model/delegation.rs',
            artifact_count: 0,
            completion_recorded: false,
            integration_status: 'rejected',
            degraded: true,
          },
        ],
      },
    };

    stubRuntimeFetch({ projection: projectionWithDelegation });
    renderAtPath('/');

    expect(await screen.findByText('recursive harness')).toBeInTheDocument();
    expect(screen.getAllByText(/^Worker$/, { selector: '.delegation-card__role' })).toHaveLength(
      2
    );
    expect(screen.getByText('awaiting integration')).toBeInTheDocument();
    expect(screen.getByText('pending')).toBeInTheDocument();
    expect(screen.getByText('rejected')).toBeInTheDocument();
    expect(
      screen.getAllByText(/^Own src\/domain\/model\/delegation\.rs$/, {
        selector: '.delegation-card__ownership',
      })
    ).toHaveLength(2);
    expect(
      screen.getByText('Ownership conflict: src/domain/model/delegation.rs')
    ).toBeInTheDocument();
  });

  it('renders inline code spans in user and assistant transcript text', async () => {
    const inlineCodeProjection: ConversationProjectionSnapshot = {
      ...bootstrapProjection,
      transcript: {
        ...bootstrapProjection.transcript,
        entries: [
          {
            record_id: 'record-inline-code-user',
            turn_id: 'task-123.turn-0002',
            speaker: 'user',
            content: 'Inspect `apps/web/src/chat/plain-message.tsx` before changing it.',
          },
          {
            record_id: 'record-inline-code-assistant',
            turn_id: 'task-123.turn-0002',
            speaker: 'assistant',
            content: 'Updated `pitch` and `yaw` defaults.',
            render: {
              blocks: [
                {
                  type: 'paragraph',
                  text: 'Updated `pitch` and `yaw` defaults.',
                },
              ],
            },
          },
        ],
      },
    };

    stubRuntimeFetch({ projection: inlineCodeProjection });
    renderAtPath('/');

    await screen.findByText('apps/web/src/chat/plain-message.tsx', {
      selector: '.msg-inline-code',
    });
    const inlineCode = Array.from(document.querySelectorAll('.msg-inline-code'));

    expect(inlineCode).toHaveLength(3);
    expect(inlineCode.map((node) => node.textContent)).toEqual([
      'apps/web/src/chat/plain-message.tsx',
      'pitch',
      'yaw',
    ]);
    expect(inlineCode[0]?.closest('.msg')).toHaveClass('msg', 'user');
    expect(inlineCode[1]?.closest('.msg')).toHaveClass('msg', 'assistant');
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

  it('splits compressed pasted closing code fences onto their own lines before submit', async () => {
    renderAtPath('/');

    await screen.findByText('Mock provider completed the turn after local inspection.');

    const input = screen.getByPlaceholderText('Ask paddles...');
    fireEvent.paste(input, {
      clipboardData: {
        getData: (type: string) =>
          type === 'text'
            ? [
                'Why does the trace recorder keep falling back?',
                '',
                '```',
                '• Fell back · 200ms',
                "  └ trace-recorder: trace recording failed: stream 'paddles.task.task-000001.root' already exists```",
              ].join('\n')
            : '',
      },
    });

    expect(await screen.findByText('6 lines pasted')).toBeInTheDocument();
    fireEvent.submit(input.closest('form') as HTMLFormElement);

    await waitFor(() => {
      const turnCall = vi
        .mocked(fetch)
        .mock.calls.find(([url]) => String(url).endsWith('/sessions/task-123/turns'));
      expect(turnCall).toBeDefined();
      expect(
        JSON.parse(String((turnCall?.[1] as RequestInit | undefined)?.body || '{}'))
      ).toEqual({
        prompt: [
          'Why does the trace recorder keep falling back?',
          '',
          '```',
          '• Fell back · 200ms',
          "  └ trace-recorder: trace recording failed: stream 'paddles.task.task-000001.root' already exists",
          '```',
        ].join('\n'),
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
});
