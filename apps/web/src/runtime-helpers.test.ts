import { describe, expect, it } from 'vitest';

import { eventRow, manifoldPrimitiveBasisLabel, sourceLabel } from './runtime-helpers';

describe('eventRow', () => {
  it('prefers the rust-authored runtime presentation when the projection stream provides it', () => {
    const row = eventRow({
      event: {
        type: 'tool_called',
        tool_name: 'shell',
        invocation: 'pwd',
      },
      presentation: {
        badge: 'tool',
        badge_class: 'tool',
        title: '• Ran shell',
        detail: 'pwd',
        text: 'shell: pwd',
      },
      runtime_items: [
        {
          kind: 'command',
          payload: {
            call_id: 'tool-1',
            tool_name: 'shell',
            phase: 'requested',
            detail: 'pwd',
          },
        },
      ],
    });

    expect(row).toEqual({
      badge: 'tool',
      badgeClass: 'tool',
      text: 'shell: pwd',
    });
  });

  it('preserves terminal output rows from the rust-authored runtime presentation', () => {
    const row = eventRow({
      event: {
        type: 'tool_output',
        call_id: 'tool-1',
        tool_name: 'shell',
        stream: 'stdout',
        output: 'alpha\nbeta',
      },
      presentation: {
        badge: 'term',
        badge_class: 'tool-terminal',
        title: '• shell stdout',
        detail: 'alpha\nbeta',
        text: 'shell stdout',
      },
      runtime_items: [
        {
          kind: 'command',
          payload: {
            call_id: 'tool-1',
            tool_name: 'shell',
            phase: 'streaming_stdout',
            detail: 'alpha\nbeta',
          },
        },
      ],
    });

    expect(row).toEqual({
      badge: 'term',
      badgeClass: 'tool-terminal',
      text: 'shell stdout',
      output: 'alpha\nbeta',
      streamKey: 'tool-stream:tool-1:stdout',
    });
  });

  it('preserves plan checklist rows from the rust-authored runtime presentation', () => {
    const row = eventRow({
      event: {
        type: 'plan_updated',
        items: [],
      },
      presentation: {
        badge: 'plan',
        badge_class: 'planner',
        title: '• Updated Plan',
        detail: '□ Inspect `git status --short`\n✓ Verify the change and summarize the outcome.',
        text: 'Updated Plan',
      },
      runtime_items: [
        {
          kind: 'plan',
          payload: {
            items: [],
          },
        },
      ],
    });

    expect(row).toEqual({
      badge: 'plan',
      badgeClass: 'planner',
      text: 'Updated Plan',
      output: '□ Inspect `git status --short`\n✓ Verify the change and summarize the outcome.',
    });
  });

  it('preserves control detail rows from the rust-authored runtime presentation', () => {
    const row = eventRow({
      event: {
        type: 'control_state_changed',
      },
      presentation: {
        badge: 'control',
        badge_class: 'governor',
        title: '• Control: interrupt unavailable',
        detail: 'planner lane is reconfiguring and cannot honor interrupt yet',
        text: 'interrupt unavailable · session',
      },
      runtime_items: [
        {
          kind: 'control',
          payload: {
            result: {
              operation: {
                scope: 'turn',
                operation: 'interrupt',
              },
              status: 'unavailable',
              subject: {},
              detail:
                'planner lane is reconfiguring and cannot honor interrupt yet',
            },
          },
        },
      ],
    });

    expect(row).toEqual({
      badge: 'control',
      badgeClass: 'governor',
      text: 'interrupt unavailable · session',
      output: 'planner lane is reconfiguring and cannot honor interrupt yet',
    });
  });

  it('preserves command summaries from the rust-authored runtime presentation', () => {
    const row = eventRow({
      event: {
        type: 'tool_finished',
        tool_name: 'shell',
        summary: 'exit 0\n2 files changed',
      },
      presentation: {
        badge: 'tool',
        badge_class: 'tool',
        title: '• Completed shell',
        detail: 'exit 0\n2 files changed',
        text: 'shell finished',
      },
      runtime_items: [
        {
          kind: 'command',
          payload: {
            call_id: 'tool-1',
            tool_name: 'shell',
            phase: 'finished',
            detail: 'exit 0\n2 files changed',
          },
        },
      ],
    });

    expect(row).toEqual({
      badge: 'tool',
      badgeClass: 'tool',
      text: 'shell finished',
      output: 'exit 0\n2 files changed',
    });
  });

  it('surfaces gatherer progress detail instead of collapsing to a generic searching label', () => {
    const row = eventRow({
      type: 'gatherer_search_progress',
      phase: 'Indexing',
      strategy: 'bm25',
      eta_seconds: 12,
      detail:
        'indexing 4/10 files · blobs 7 · fresh 2 · skipped 1 · segments 24 · bm25 cache 0 build 1',
    });

    expect(row).toBeTruthy();
    expect(row?.text).toContain('bm25');
    expect(row?.text).toContain('indexing 4/10 files');
    expect(row?.text).toContain('bm25 cache 0 build 1');
    expect(row?.text).toContain('eta 12s');
  });

  it('uses hunting as the fallback gatherer label when no detail is available', () => {
    const row = eventRow({
      type: 'gatherer_search_progress',
      phase: 'Indexing',
      strategy: '',
      eta_seconds: null,
      detail: '',
    });

    expect(row).toBeTruthy();
    expect(row?.text).toContain('hunting (Indexing)');
  });

  it('surfaces harness governor state with chamber ownership and timeout phase', () => {
    const row = eventRow({
      type: 'harness_state',
      snapshot: {
        chamber: 'gathering',
        governor: {
          status: 'active',
          timeout: {
            phase: 'slow',
            elapsed_seconds: 9,
            deadline_seconds: 30,
          },
          intervention: null,
        },
        detail: 'indexing 4/10 files',
      },
    });

    expect(row).toBeTruthy();
    expect(row?.badge).toBe('gov');
    expect(row?.text).toContain('gathering');
    expect(row?.text).toContain('slow');
    expect(row?.text).toContain('indexing 4/10 files');
  });

  it('labels workspace editor boundaries in the manifold source view', () => {
    expect(sourceLabel('workspace_editor_boundary')).toBe('Workspace editor boundary');
  });

  it('labels steering gate bases in the manifold source view', () => {
    expect(manifoldPrimitiveBasisLabel({ kind: 'steering_gate', gate: 'convergence' })).toBe(
      'Convergence gate'
    );
  });

  it('surfaces applied edits as diff rows instead of collapsing them to tool chatter', () => {
    const row = eventRow({
      type: 'workspace_edit_applied',
      tool_name: 'apply_patch',
      edit: {
        files: ['sample.rs'],
        diff: '--- a/sample.rs\n+++ b/sample.rs\n@@ -1 +1 @@\n-old\n+new',
        insertions: 1,
        deletions: 1,
      },
    });

    expect(row).toEqual({
      badge: 'tool',
      badgeClass: 'tool-diff',
      text: 'apply_patch applied',
      diff: '--- a/sample.rs\n+++ b/sample.rs\n@@ -1 +1 @@\n-old\n+new',
    });
  });
});
